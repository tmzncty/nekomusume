use neko_crypto::{
    PreauthBudget, PreauthLimits, PreauthStateId, ProcessPreauthAdmission, ProcessPreauthLimits,
    ResponseSendPermit as ProcessResponseSendPermit,
};
use std::io::{self, Write};
use std::net::{SocketAddr, TcpStream, UdpSocket};
use std::time::{Duration, Instant};

const RESERVED_STATE_BYTES: usize = 16 * 1024;

pub(crate) struct ListenerAdmission {
    started: Instant,
    process: ProcessPreauthAdmission,
}

pub(crate) struct AdmissionTicket {
    id: PreauthStateId,
    budget: PreauthBudget,
}

struct ResponseSendPermit {
    inner: ProcessResponseSendPermit,
    clock_origin: Instant,
    deadline: Instant,
}

struct TimedResponseAttempt {
    result: io::Result<()>,
    finished_at: Instant,
}

impl ResponseSendPermit {
    fn deadline(&self) -> Instant {
        self.deadline
    }
}

impl ListenerAdmission {
    pub(crate) fn new() -> Self {
        Self {
            started: Instant::now(),
            process: ProcessPreauthAdmission::new(ProcessPreauthLimits::default(), 0)
                .expect("default pre-auth limits are valid"),
        }
    }

    fn now_ms(&self) -> Result<u64, ()> {
        monotonic_ms(self.started, Instant::now()).ok_or(())
    }

    pub(crate) fn expire(&mut self) {
        if let Ok(now) = self.now_ms() {
            let _ = self.process.expire(now);
        }
    }

    pub(crate) fn admit(&mut self, peer: SocketAddr) -> Result<AdmissionTicket, ()> {
        self.expire();
        let now = self.now_ms()?;
        let id = self
            .process
            .admit_state(&source_key(peer), RESERVED_STATE_BYTES, now)
            .map_err(|_| ())?;
        let budget = match PreauthBudget::new(PreauthLimits::default()) {
            Ok(budget) => budget,
            Err(_) => {
                let _ = self.process.release(id);
                return Err(());
            }
        };
        Ok(AdmissionTicket { id, budget })
    }

    pub(crate) fn charge_input(
        &mut self,
        ticket: &mut AdmissionTicket,
        bytes: usize,
        work_units: usize,
    ) -> Result<(), ()> {
        let now = self.now_ms()?;
        ticket.budget.charge_input(bytes).map_err(|_| ())?;
        if self
            .process
            .charge_input(ticket.id, bytes, work_units, now)
            .is_err()
        {
            let _ = ticket.budget.rollback_input(bytes);
            return Err(());
        }
        Ok(())
    }

    fn admit_response(
        &mut self,
        ticket: &mut AdmissionTicket,
        bytes: usize,
    ) -> Result<ResponseSendPermit, ()> {
        let clock_origin = self.started;
        let now = self.now_ms()?;
        ticket.budget.charge_response(bytes).map_err(|_| ())?;
        let mut inner = match self.process.admit_response(ticket.id, bytes, now) {
            Ok(permit) => permit,
            Err(_) => {
                let _ = ticket.budget.rollback_response(bytes);
                return Err(());
            }
        };
        let deadline = match clock_origin.checked_add(Duration::from_millis(inner.deadline_ms())) {
            Some(deadline) => deadline,
            None => {
                // Both accounting layers have admitted this scheduled attempt;
                // failing to represent its deadline must not refund it.
                let _ = self.process.expire_response(&mut inner);
                return Err(());
            }
        };
        Ok(ResponseSendPermit {
            inner,
            clock_origin,
            deadline,
        })
    }

    fn complete_response_at(
        &mut self,
        permit: &mut ResponseSendPermit,
        now: Instant,
    ) -> Result<(), ()> {
        if now > permit.deadline {
            let _ = self.process.expire_response(&mut permit.inner);
            return Err(());
        }
        let now_ms = match monotonic_ms(permit.clock_origin, now) {
            Some(now_ms) => now_ms,
            None => {
                let _ = self.process.expire_response(&mut permit.inner);
                return Err(());
            }
        };
        self.process
            .complete_response(&mut permit.inner, now_ms)
            .map_err(|_| ())
    }

    fn abandon_response(&mut self, permit: &mut ResponseSendPermit) -> Result<(), ()> {
        self.process
            .abandon_response(&mut permit.inner)
            .map_err(|_| ())
    }

    fn fail_response_at(
        &mut self,
        permit: &mut ResponseSendPermit,
        finished_at: Instant,
    ) -> Result<(), ()> {
        if finished_at > permit.deadline {
            self.process
                .expire_response(&mut permit.inner)
                .map_err(|_| ())
        } else {
            self.abandon_response(permit)
        }
    }

    fn attempt_response(
        &mut self,
        ticket: &mut AdmissionTicket,
        bytes: usize,
        attempt: impl FnOnce(Instant) -> TimedResponseAttempt,
    ) -> Result<(), ()> {
        let mut permit = self.admit_response(ticket, bytes)?;
        let attempt = attempt(permit.deadline());
        match attempt.result {
            Ok(()) => self.complete_response_at(&mut permit, attempt.finished_at),
            Err(_) => {
                let _ = self.fail_response_at(&mut permit, attempt.finished_at);
                Err(())
            }
        }
    }

    pub(crate) fn write_tcp_response(
        &mut self,
        ticket: &mut AdmissionTicket,
        stream: &mut TcpStream,
        payload: &[u8],
        outer_deadline: Option<Instant>,
    ) -> Result<(), ()> {
        let length = u32::try_from(payload.len()).map_err(|_| ())?;
        let header = length.to_be_bytes();
        let charged_bytes = payload.len().checked_add(header.len()).ok_or(())?;
        let original_timeout = stream.write_timeout().map_err(|_| ())?;
        let timeout_deadline =
            original_timeout.and_then(|timeout| Instant::now().checked_add(timeout));
        let attempt = self.attempt_response(ticket, charged_bytes, |permit_deadline| {
            let deadline = cap_deadline(permit_deadline, &[outer_deadline, timeout_deadline]);
            let result = (|| {
                write_all_until(stream, &header, deadline)?;
                write_all_until(stream, payload, deadline)?;
                set_tcp_write_deadline(stream, deadline)?;
                stream.flush()
            })();
            finish_attempt(result, deadline)
        });
        let restored = stream.set_write_timeout(original_timeout).map_err(|_| ());
        attempt.and(restored)
    }

    pub(crate) fn send_udp_response(
        &mut self,
        ticket: &mut AdmissionTicket,
        socket: &UdpSocket,
        payload: &[u8],
        peer: SocketAddr,
    ) -> Result<(), ()> {
        let original_timeout = socket.write_timeout().map_err(|_| ())?;
        let timeout_deadline =
            original_timeout.and_then(|timeout| Instant::now().checked_add(timeout));
        let attempt = self.attempt_response(ticket, payload.len(), |permit_deadline| {
            let deadline = cap_deadline(permit_deadline, &[timeout_deadline]);
            let result = (|| {
                set_udp_write_deadline(socket, deadline)?;
                let sent = socket.send_to(payload, peer)?;
                if sent != payload.len() {
                    return Err(io::Error::new(
                        io::ErrorKind::WriteZero,
                        "partial pre-auth datagram send",
                    ));
                }
                Ok(())
            })();
            finish_attempt(result, deadline)
        });
        let restored = socket.set_write_timeout(original_timeout).map_err(|_| ());
        attempt.and(restored)
    }

    /// Model a bounded test carrier accepting a response and then injecting
    /// delivery loss after the local attempt completed.
    pub(crate) fn complete_fault_injected_response(
        &mut self,
        ticket: &mut AdmissionTicket,
        bytes: usize,
    ) -> Result<(), ()> {
        let mut permit = self.admit_response(ticket, bytes)?;
        self.complete_response_at(&mut permit, Instant::now())
    }

    pub(crate) fn release(&mut self, ticket: AdmissionTicket) {
        let _ = self.process.release(ticket.id);
    }
}

fn monotonic_ms(origin: Instant, now: Instant) -> Option<u64> {
    now.checked_duration_since(origin)?
        .as_millis()
        .try_into()
        .ok()
}

fn remaining_timeout_at(deadline: Instant, now: Instant) -> io::Result<Duration> {
    let remaining = deadline
        .checked_duration_since(now)
        .ok_or_else(timeout_error)?;
    if remaining.is_zero() {
        return Err(timeout_error());
    }
    Ok(remaining)
}

fn remaining_timeout(deadline: Instant) -> io::Result<Duration> {
    remaining_timeout_at(deadline, Instant::now())
}

fn finish_attempt(result: io::Result<()>, deadline: Instant) -> TimedResponseAttempt {
    finish_attempt_at(result, deadline, Instant::now())
}

fn finish_attempt_at(
    result: io::Result<()>,
    deadline: Instant,
    finished_at: Instant,
) -> TimedResponseAttempt {
    let result = result.and_then(|()| {
        if finished_at > deadline {
            Err(timeout_error())
        } else {
            Ok(())
        }
    });
    TimedResponseAttempt {
        result,
        finished_at,
    }
}

fn cap_deadline(deadline: Instant, caps: &[Option<Instant>]) -> Instant {
    caps.iter().flatten().copied().fold(deadline, Instant::min)
}

fn timeout_error() -> io::Error {
    io::Error::new(
        io::ErrorKind::TimedOut,
        "pre-auth response deadline elapsed",
    )
}

fn set_tcp_write_deadline(stream: &TcpStream, deadline: Instant) -> io::Result<()> {
    stream.set_write_timeout(Some(remaining_timeout(deadline)?))
}

fn write_all_until(stream: &mut TcpStream, mut bytes: &[u8], deadline: Instant) -> io::Result<()> {
    while !bytes.is_empty() {
        set_tcp_write_deadline(stream, deadline)?;
        match stream.write(bytes) {
            Ok(0) => {
                return Err(io::Error::new(
                    io::ErrorKind::WriteZero,
                    "failed to write pre-auth response",
                ));
            }
            Ok(written) => bytes = &bytes[written..],
            Err(error) if error.kind() == io::ErrorKind::Interrupted => {}
            Err(error) => return Err(error),
        }
    }
    Ok(())
}

fn set_udp_write_deadline(socket: &UdpSocket, deadline: Instant) -> io::Result<()> {
    socket.set_write_timeout(Some(remaining_timeout(deadline)?))
}

fn source_key(peer: SocketAddr) -> Vec<u8> {
    let mut key = Vec::with_capacity(19);
    match peer.ip() {
        std::net::IpAddr::V4(ip) => {
            key.push(4);
            key.extend_from_slice(&ip.octets());
        }
        std::net::IpAddr::V6(ip) => {
            key.push(6);
            key.extend_from_slice(&ip.octets());
        }
    }
    key.extend_from_slice(&peer.port().to_be_bytes());
    key
}

#[cfg(test)]
mod tests {
    use super::*;
    use std::io::Read;
    use std::net::TcpListener;
    use std::thread;

    #[test]
    fn source_projection_is_family_and_port_bound_without_text() {
        let a: SocketAddr = "127.0.0.1:40080".parse().unwrap();
        let b: SocketAddr = "127.0.0.1:40081".parse().unwrap();
        let v6: SocketAddr = "[::1]:40080".parse().unwrap();
        assert_ne!(source_key(a), source_key(b));
        assert_ne!(source_key(a), source_key(v6));
        assert_eq!(source_key(a).len(), 7);
        assert_eq!(source_key(v6).len(), 19);
    }

    #[test]
    fn response_requires_charged_input_and_release_reopens_source() {
        let mut admission = ListenerAdmission::new();
        let peer: SocketAddr = "127.0.0.1:40080".parse().unwrap();
        let mut ticket = admission.admit(peer).unwrap();
        assert!(admission.admit_response(&mut ticket, 1).is_err());
        admission.charge_input(&mut ticket, 64, 16).unwrap();
        let mut permit = admission.admit_response(&mut ticket, 64).unwrap();
        admission.abandon_response(&mut permit).unwrap();
        assert!(
            admission
                .complete_response_at(&mut permit, Instant::now())
                .is_err()
        );
        admission.release(ticket);
        assert!(admission.admit(peer).is_ok());
    }

    #[test]
    fn tcp_response_restores_timeout_and_allows_followup_read() {
        let listener = TcpListener::bind("127.0.0.1:0").unwrap();
        listener.set_nonblocking(true).unwrap();
        let address = listener.local_addr().unwrap();
        let client = thread::spawn(move || {
            let mut stream = TcpStream::connect(address).unwrap();
            let mut response = [0_u8; 5];
            stream.read_exact(&mut response).unwrap();
            stream.write_all(b"next").unwrap();
            response
        });
        let (mut stream, peer) = loop {
            match listener.accept() {
                Ok(accepted) => break accepted,
                Err(error) if error.kind() == io::ErrorKind::WouldBlock => thread::yield_now(),
                Err(error) => panic!("accept failed: {error}"),
            }
        };
        stream.set_nonblocking(false).unwrap();
        stream
            .set_read_timeout(Some(Duration::from_secs(1)))
            .unwrap();
        let mut admission = ListenerAdmission::new();
        let mut ticket = admission.admit(peer).unwrap();
        admission.charge_input(&mut ticket, 64, 16).unwrap();

        admission
            .write_tcp_response(&mut ticket, &mut stream, b"x", None)
            .unwrap();
        assert_eq!(stream.write_timeout().unwrap(), None);
        let mut next = [0_u8; 4];
        stream.read_exact(&mut next).unwrap();

        assert_eq!(next, *b"next");
        assert_eq!(client.join().unwrap(), [0, 0, 0, 1, b'x']);
    }

    #[test]
    fn response_timeout_requires_positive_remaining_time() {
        let now = Instant::now();
        assert_eq!(
            remaining_timeout_at(now + Duration::from_millis(1), now).unwrap(),
            Duration::from_millis(1)
        );
        assert_eq!(
            remaining_timeout_at(now, now).unwrap_err().kind(),
            io::ErrorKind::TimedOut
        );
        assert_eq!(
            remaining_timeout_at(now, now + Duration::from_nanos(1))
                .unwrap_err()
                .kind(),
            io::ErrorKind::TimedOut
        );
    }

    #[test]
    fn response_attempt_uses_one_completion_instant_and_earliest_cap() {
        let started = Instant::now();
        let permit_deadline = started + Duration::from_millis(100);
        let outer_deadline = started + Duration::from_millis(40);
        let timeout_deadline = started + Duration::from_millis(60);
        assert_eq!(
            cap_deadline(
                permit_deadline,
                &[Some(timeout_deadline), Some(outer_deadline)]
            ),
            outer_deadline
        );

        let at_boundary = finish_attempt_at(Ok(()), outer_deadline, outer_deadline);
        assert!(at_boundary.result.is_ok());
        assert_eq!(at_boundary.finished_at, outer_deadline);
        let after_boundary = finish_attempt_at(
            Ok(()),
            outer_deadline,
            outer_deadline + Duration::from_nanos(1),
        );
        assert_eq!(
            after_boundary.result.unwrap_err().kind(),
            io::ErrorKind::TimedOut
        );
        assert_eq!(
            after_boundary.finished_at,
            outer_deadline + Duration::from_nanos(1)
        );
    }
}
