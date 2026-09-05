use neko_crypto::{
    PreauthBudget, PreauthLimits, PreauthQueuePermit, PreauthResponsePermit, PreauthStateId,
    ProcessPreauthAdmission, ProcessPreauthLimits,
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

pub(crate) struct QueueReservation {
    permit: Option<PreauthQueuePermit>,
}

impl AdmissionTicket {
    pub(crate) fn was_expired(&self, expired: &[PreauthStateId]) -> bool {
        expired.contains(&self.id)
    }
}

impl QueueReservation {
    pub(crate) fn invalidate_after_process_expiry(&mut self) {
        self.permit = None;
    }
}

fn write_all_until<W: Write, N: FnMut() -> u64>(
    writer: &mut W,
    mut bytes: &[u8],
    now: &mut N,
    deadline_ms: u64,
) -> io::Result<()> {
    while !bytes.is_empty() {
        if now() > deadline_ms {
            return Err(io::Error::new(
                io::ErrorKind::TimedOut,
                "response deadline elapsed",
            ));
        }
        match writer.write(bytes) {
            Ok(0) => {
                return Err(io::Error::new(
                    io::ErrorKind::WriteZero,
                    "response write stalled",
                ));
            }
            Ok(written) => bytes = &bytes[written..],
            Err(error) if error.kind() == io::ErrorKind::Interrupted => {}
            Err(error) => return Err(error),
        }
    }
    Ok(())
}

fn write_frame_until<W: Write, N: FnMut() -> u64>(
    writer: &mut W,
    payload: &[u8],
    frame_len: u32,
    mut now: N,
    deadline_ms: u64,
) -> io::Result<()> {
    write_all_until(writer, &frame_len.to_be_bytes(), &mut now, deadline_ms)?;
    write_all_until(writer, payload, &mut now, deadline_ms)?;
    if now() > deadline_ms {
        return Err(io::Error::new(
            io::ErrorKind::TimedOut,
            "response deadline elapsed",
        ));
    }
    writer.flush()
}

impl ListenerAdmission {
    pub(crate) fn new() -> Self {
        Self {
            started: Instant::now(),
            process: ProcessPreauthAdmission::new(ProcessPreauthLimits::default(), 0)
                .expect("default pre-auth limits are valid"),
        }
    }

    fn now_ms(&self) -> u64 {
        self.started
            .elapsed()
            .as_millis()
            .try_into()
            .unwrap_or(u64::MAX)
    }

    pub(crate) fn expire(&mut self) -> Vec<PreauthStateId> {
        self.process
            .expire_states(self.now_ms())
            .unwrap_or_default()
    }

    pub(crate) fn admit(&mut self, peer: SocketAddr) -> Result<AdmissionTicket, ()> {
        let _ = self.expire();
        let now = self.now_ms();
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
        let now = self.now_ms();
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

    pub(crate) fn charge_response(
        &mut self,
        ticket: &mut AdmissionTicket,
        bytes: usize,
    ) -> Result<PreauthResponsePermit, ()> {
        let now = self.now_ms();
        ticket.budget.charge_response(bytes).map_err(|_| ())?;
        match self.process.charge_response(ticket.id, bytes, now) {
            Ok(permit) => Ok(permit),
            Err(_) => {
                let _ = ticket.budget.rollback_response(bytes);
                Err(())
            }
        }
    }

    fn remaining_response_budget(&self, permit: &PreauthResponsePermit) -> Result<Duration, ()> {
        let now = self.now_ms();
        let remaining_ms = permit.deadline_ms().checked_sub(now).ok_or(())?;
        Ok(Duration::from_millis(remaining_ms.max(1)))
    }

    pub(crate) fn send_tcp_response(
        &mut self,
        stream: &mut TcpStream,
        payload: &[u8],
        permit: PreauthResponsePermit,
    ) -> Result<(), ()> {
        let budget = match self.remaining_response_budget(&permit) {
            Ok(budget) => budget,
            Err(()) => {
                self.process.abandon_response(permit);
                return Err(());
            }
        };
        if stream.set_write_timeout(Some(budget)).is_err() {
            self.process.abandon_response(permit);
            return Err(());
        }
        let frame_len: u32 = match payload.len().try_into() {
            Ok(len) => len,
            Err(_) => {
                self.process.abandon_response(permit);
                return Err(());
            }
        };
        let result = write_frame_until(
            stream,
            payload,
            frame_len,
            || self.now_ms(),
            permit.deadline_ms(),
        );
        if result.is_err() {
            self.process.abandon_response(permit);
            return Err(());
        }
        self.process
            .complete_response(permit, self.now_ms())
            .map_err(|_| ())
    }

    pub(crate) fn send_udp_response(
        &mut self,
        socket: &UdpSocket,
        payload: &[u8],
        peer: SocketAddr,
        permit: PreauthResponsePermit,
    ) -> Result<(), ()> {
        let budget = match self.remaining_response_budget(&permit) {
            Ok(budget) => budget,
            Err(()) => {
                self.process.abandon_response(permit);
                return Err(());
            }
        };
        if socket.set_write_timeout(Some(budget)).is_err() {
            self.process.abandon_response(permit);
            return Err(());
        }
        match socket.send_to(payload, peer) {
            Ok(sent) if sent == payload.len() => self
                .process
                .complete_response(permit, self.now_ms())
                .map_err(|_| ()),
            _ => {
                self.process.abandon_response(permit);
                Err(())
            }
        }
    }

    pub(crate) fn enqueue(&mut self, ticket: &mut AdmissionTicket) -> Result<QueueReservation, ()> {
        let permit = self
            .process
            .enqueue(ticket.id, self.now_ms())
            .map_err(|_| ())?;
        Ok(QueueReservation {
            permit: Some(permit),
        })
    }

    pub(crate) fn dequeue(&mut self, reservation: &mut QueueReservation) -> Result<(), ()> {
        let permit = reservation.permit.take().ok_or(())?;
        self.process.dequeue(permit).map_err(|_| ())
    }

    pub(crate) fn release(&mut self, ticket: AdmissionTicket) {
        let _ = self.process.release(ticket.id);
    }
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
    fn expired_ticket_invalidates_application_queue_owner() {
        let mut admission = ListenerAdmission::new();
        let peer: SocketAddr = "127.0.0.1:40080".parse().unwrap();
        let mut ticket = admission.admit(peer).unwrap();
        let mut queue = admission.enqueue(&mut ticket).unwrap();
        let expired = admission.process.expire_states(5000).unwrap();
        assert!(ticket.was_expired(&expired));
        queue.invalidate_after_process_expiry();
        assert!(admission.dequeue(&mut queue).is_err());
        admission.release(ticket);
        assert_eq!(
            (admission.process.live_states(), admission.process.queued()),
            (0, 0)
        );
    }

    #[test]
    fn framed_response_uses_one_absolute_deadline_across_partial_writes() {
        struct PartialWriter {
            bytes: Vec<u8>,
            chunk: usize,
        }
        impl Write for PartialWriter {
            fn write(&mut self, bytes: &[u8]) -> io::Result<usize> {
                let count = bytes.len().min(self.chunk);
                self.bytes.extend_from_slice(&bytes[..count]);
                Ok(count)
            }
            fn flush(&mut self) -> io::Result<()> {
                Ok(())
            }
        }
        let ticks = [0, 10, 20, 30, 100, 100];
        let mut index = 0;
        let mut writer = PartialWriter {
            bytes: Vec::new(),
            chunk: 2,
        };
        write_frame_until(
            &mut writer,
            b"abcd",
            4,
            || {
                let tick = ticks[index.min(ticks.len() - 1)];
                index += 1;
                tick
            },
            100,
        )
        .unwrap();
        assert_eq!(writer.bytes, [0, 0, 0, 4, b'a', b'b', b'c', b'd']);

        let ticks = [0, 10, 20, 101];
        let mut index = 0;
        let mut writer = PartialWriter {
            bytes: Vec::new(),
            chunk: 2,
        };
        assert_eq!(
            write_frame_until(
                &mut writer,
                b"abcd",
                4,
                || {
                    let tick = ticks[index.min(ticks.len() - 1)];
                    index += 1;
                    tick
                },
                100
            )
            .unwrap_err()
            .kind(),
            io::ErrorKind::TimedOut
        );
        assert_ne!(writer.bytes, [0, 0, 0, 4, b'a', b'b', b'c', b'd']);
    }

    #[test]
    fn response_requires_charged_input_and_release_reopens_source() {
        let mut admission = ListenerAdmission::new();
        let peer: SocketAddr = "127.0.0.1:40080".parse().unwrap();
        let mut ticket = admission.admit(peer).unwrap();
        assert!(admission.charge_response(&mut ticket, 1).is_err());
        admission.charge_input(&mut ticket, 64, 16).unwrap();
        let permit = admission.charge_response(&mut ticket, 64).unwrap();
        admission
            .process
            .complete_response(permit, admission.now_ms())
            .unwrap();
        admission.release(ticket);
        assert!(admission.admit(peer).is_ok());
    }
}
