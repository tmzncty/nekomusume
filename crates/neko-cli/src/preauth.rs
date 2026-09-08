use crate::framed::{FrameRead, FrameStage, FramedReader};
use neko_crypto::{
    PreauthBudget, PreauthInputRecord, PreauthInputRecordPermit, PreauthLimits, PreauthQueuePermit,
    PreauthResponsePermit, PreauthStateId, ProcessPreauthAdmission, ProcessPreauthLimits,
};
use std::io::{self, Write};
use std::net::{SocketAddr, TcpStream, UdpSocket};
use std::time::{Duration, Instant};

const RESERVED_STATE_BYTES: usize = 16 * 1024;

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub(crate) enum CarrierKind {
    Tcp,
    Udp,
}

pub(crate) struct ListenerAdmission {
    started: Instant,
    process: ProcessPreauthAdmission,
}

pub(crate) struct AdmissionTicket {
    id: PreauthStateId,
    budget: PreauthBudget,
}

pub(crate) struct TcpInputReservation {
    inner: PreauthInputRecord,
    outer: PreauthInputRecordPermit,
}

impl TcpInputReservation {
    fn abandon(&mut self, admission: &mut ListenerAdmission, ticket: &mut AdmissionTicket) {
        let _ = ticket.budget.abandon_input_record(&mut self.inner);
        let _ = admission
            .process
            .abandon_input_record(&mut self.outer, admission.now_ms());
        admission.process.reject_state(ticket.id);
    }
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

trait BoundedWrite {
    fn write_with_budget(&mut self, bytes: &[u8], budget: Duration) -> io::Result<usize>;
    fn flush_with_budget(&mut self, budget: Duration) -> io::Result<()>;
}

impl BoundedWrite for TcpStream {
    fn write_with_budget(&mut self, bytes: &[u8], budget: Duration) -> io::Result<usize> {
        self.set_write_timeout(Some(budget))?;
        self.write(bytes)
    }

    fn flush_with_budget(&mut self, budget: Duration) -> io::Result<()> {
        self.set_write_timeout(Some(budget))?;
        self.flush()
    }
}

fn remaining_budget(now_ms: u64, deadline_ms: u64) -> io::Result<Duration> {
    let remaining_ms = deadline_ms
        .checked_sub(now_ms)
        .ok_or_else(|| io::Error::new(io::ErrorKind::TimedOut, "response deadline elapsed"))?;
    if remaining_ms == 0 {
        return Err(io::Error::new(
            io::ErrorKind::TimedOut,
            "response deadline elapsed",
        ));
    }
    Ok(Duration::from_millis(remaining_ms))
}

fn write_all_until<W: BoundedWrite, N: FnMut() -> u64>(
    writer: &mut W,
    mut bytes: &[u8],
    now: &mut N,
    deadline_ms: u64,
) -> io::Result<()> {
    while !bytes.is_empty() {
        let budget = remaining_budget(now(), deadline_ms)?;
        match writer.write_with_budget(bytes, budget) {
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

fn write_frame_until<W: BoundedWrite, N: FnMut() -> u64>(
    writer: &mut W,
    payload: &[u8],
    frame_len: u32,
    mut now: N,
    deadline_ms: u64,
) -> io::Result<()> {
    write_all_until(writer, &frame_len.to_be_bytes(), &mut now, deadline_ms)?;
    write_all_until(writer, payload, &mut now, deadline_ms)?;
    let budget = remaining_budget(now(), deadline_ms)?;
    writer.flush_with_budget(budget)
}

#[allow(clippy::too_many_arguments)]
pub(crate) fn read_staged_frame(
    reader: &mut FramedReader,
    stream: &mut TcpStream,
    max_frame_len: usize,
    deadline: Instant,
    admission: &mut ListenerAdmission,
    ticket: &mut AdmissionTicket,
    header_work: usize,
    body_work: usize,
) -> io::Result<Vec<u8>> {
    reader.set_max_frame_len(max_frame_len)?;
    let mut reservation: Option<TcpInputReservation> = None;
    let result = reader.read_until_staged(stream, deadline, |stage| {
        let rejected = || {
            io::Error::new(
                io::ErrorKind::PermissionDenied,
                "pre-auth admission rejected",
            )
        };
        match stage {
            FrameStage::Header { bytes, work } => {
                let work = header_work
                    .checked_add(work.saturating_sub(64))
                    .ok_or_else(rejected)?;
                reservation = Some(
                    admission
                        .begin_tcp_input_record(ticket, bytes, work)
                        .map_err(|_| rejected())?,
                );
            }
            FrameStage::Body { bytes, work } => {
                let r = reservation.as_mut().ok_or_else(rejected)?;
                admission
                    .extend_tcp_input_record(
                        ticket,
                        r,
                        bytes,
                        body_work.saturating_sub(header_work).min(work),
                    )
                    .map_err(|_| rejected())?;
            }
            FrameStage::Complete => {
                let r = reservation.as_mut().ok_or_else(rejected)?;
                admission
                    .complete_tcp_input_record(ticket, r)
                    .map_err(|_| rejected())?;
            }
        }
        Ok(())
    });
    let complete = matches!(result, Ok(FrameRead::Complete(_)));
    if !complete && let Some(r) = reservation.as_mut() {
        r.abandon(admission, ticket);
    }
    match result {
        Ok(FrameRead::Complete(frame)) => Ok(frame),
        Ok(FrameRead::Deadline) => Err(io::Error::new(
            io::ErrorKind::TimedOut,
            "frame deadline elapsed",
        )),
        Ok(FrameRead::CleanEof | FrameRead::Truncated) => Err(io::Error::new(
            io::ErrorKind::UnexpectedEof,
            "truncated frame",
        )),
        Err(error) => Err(error),
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

    #[allow(dead_code)]
    pub(crate) fn admit(&mut self, peer: SocketAddr) -> Result<AdmissionTicket, ()> {
        self.admit_carrier(CarrierKind::Tcp, peer)
    }

    pub(crate) fn admit_carrier(
        &mut self,
        carrier: CarrierKind,
        peer: SocketAddr,
    ) -> Result<AdmissionTicket, ()> {
        let _ = self.expire();
        let now = self.now_ms();
        let id = self
            .process
            .admit_state(&source_key(carrier, peer), RESERVED_STATE_BYTES, now)
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

    pub(crate) fn begin_tcp_input_record(
        &mut self,
        ticket: &mut AdmissionTicket,
        header_bytes: usize,
        header_work: usize,
    ) -> Result<TcpInputReservation, ()> {
        let inner = match ticket.budget.begin_input_record(header_bytes) {
            Ok(permit) => permit,
            Err(_) => {
                self.process.reject_state(ticket.id);
                return Err(());
            }
        };
        let outer = match self.process.begin_input_record(
            ticket.id,
            header_bytes,
            header_work,
            self.now_ms(),
        ) {
            Ok(permit) => permit,
            Err(_) => {
                self.process.reject_state(ticket.id);
                return Err(());
            }
        };
        Ok(TcpInputReservation { inner, outer })
    }

    pub(crate) fn extend_tcp_input_record(
        &mut self,
        ticket: &mut AdmissionTicket,
        reservation: &mut TcpInputReservation,
        body_bytes: usize,
        body_work: usize,
    ) -> Result<(), ()> {
        if ticket
            .budget
            .extend_input_record(&mut reservation.inner, body_bytes)
            .is_err()
        {
            self.process.reject_state(ticket.id);
            return Err(());
        }
        if self
            .process
            .extend_input_record(&mut reservation.outer, body_bytes, body_work, self.now_ms())
            .is_err()
        {
            self.process.reject_state(ticket.id);
            return Err(());
        }
        Ok(())
    }

    pub(crate) fn complete_tcp_input_record(
        &mut self,
        ticket: &mut AdmissionTicket,
        reservation: &mut TcpInputReservation,
    ) -> Result<(), ()> {
        if ticket
            .budget
            .complete_input_record(&mut reservation.inner)
            .is_err()
        {
            self.process.reject_state(ticket.id);
            return Err(());
        }
        if self
            .process
            .complete_input_record(&mut reservation.outer, self.now_ms())
            .is_err()
        {
            self.process.reject_state(ticket.id);
            return Err(());
        }
        Ok(())
    }

    pub(crate) fn charge_input(
        &mut self,
        ticket: &mut AdmissionTicket,
        bytes: usize,
        work_units: usize,
    ) -> Result<(), ()> {
        let now = self.now_ms();
        if ticket.budget.charge_input(bytes).is_err() {
            self.process.reject_state(ticket.id);
            return Err(());
        }
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
        if ticket.budget.charge_response(bytes).is_err() {
            self.process.reject_state(ticket.id);
            return Err(());
        }
        match self.process.charge_response(ticket.id, bytes, now) {
            Ok(permit) => Ok(permit),
            Err(_) => {
                let _ = ticket.budget.rollback_response(bytes);
                Err(())
            }
        }
    }

    fn remaining_response_budget(&self, permit: &PreauthResponsePermit) -> Result<Duration, ()> {
        remaining_budget(self.now_ms(), permit.deadline_ms()).map_err(|_| ())
    }

    pub(crate) fn send_tcp_response(
        &mut self,
        stream: &mut TcpStream,
        payload: &[u8],
        permit: PreauthResponsePermit,
    ) -> Result<(), ()> {
        let previous_timeout = match stream.write_timeout() {
            Ok(timeout) => timeout,
            Err(_) => {
                self.process.abandon_response(permit);
                return Err(());
            }
        };
        if self.remaining_response_budget(&permit).is_err() {
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
        if stream.set_write_timeout(previous_timeout).is_err() {
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
        let previous_timeout = match socket.write_timeout() {
            Ok(timeout) => timeout,
            Err(_) => {
                self.process.abandon_response(permit);
                return Err(());
            }
        };
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
        let sent = socket.send_to(payload, peer);
        if socket.set_write_timeout(previous_timeout).is_err() {
            self.process.abandon_response(permit);
            return Err(());
        }
        match sent {
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

fn source_key(carrier: CarrierKind, peer: SocketAddr) -> Vec<u8> {
    let mut key = Vec::with_capacity(20);
    key.push(match carrier {
        CarrierKind::Tcp => 1,
        CarrierKind::Udp => 2,
    });
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
        assert_ne!(
            source_key(CarrierKind::Tcp, a),
            source_key(CarrierKind::Tcp, b)
        );
        assert_ne!(
            source_key(CarrierKind::Tcp, a),
            source_key(CarrierKind::Tcp, v6)
        );
        assert_ne!(
            source_key(CarrierKind::Tcp, a),
            source_key(CarrierKind::Udp, a)
        );
        assert_eq!(source_key(CarrierKind::Tcp, a).len(), 8);
        assert_eq!(source_key(CarrierKind::Udp, v6).len(), 20);
    }

    #[test]
    fn expired_ticket_invalidates_application_queue_owner() {
        let mut admission = ListenerAdmission::new();
        let peer: SocketAddr = "127.0.0.1:40080".parse().unwrap();
        let mut ticket = admission.admit_carrier(CarrierKind::Tcp, peer).unwrap();
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
    fn ordinary_expiry_invalidates_pending_owner_without_stopping_controller() {
        let mut admission = ListenerAdmission::new();
        let first_peer: SocketAddr = "127.0.0.1:40080".parse().unwrap();
        let next_peer: SocketAddr = "127.0.0.1:40081".parse().unwrap();
        let mut ticket = admission.admit(first_peer).unwrap();
        let mut queue = admission.enqueue(&mut ticket).unwrap();
        let expired = admission.process.expire_states(1_000).unwrap();
        assert!(ticket.was_expired(&expired));
        queue.invalidate_after_process_expiry();
        assert!(admission.dequeue(&mut queue).is_err());
        admission.release(ticket);
        let next = admission.admit(next_peer).unwrap();
        admission.release(next);
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
        impl BoundedWrite for PartialWriter {
            fn write_with_budget(&mut self, bytes: &[u8], _budget: Duration) -> io::Result<usize> {
                let count = bytes.len().min(self.chunk);
                self.bytes.extend_from_slice(&bytes[..count]);
                Ok(count)
            }
            fn flush_with_budget(&mut self, _budget: Duration) -> io::Result<()> {
                Ok(())
            }
        }
        let ticks = [0, 10, 20, 30, 99];
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

        let ticks = [0, 10, 20, 100];
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
    fn response_send_restores_socket_write_timeouts() {
        let mut admission = ListenerAdmission::new();
        let peer: SocketAddr = "127.0.0.1:40080".parse().unwrap();
        let listener = std::net::TcpListener::bind(peer).unwrap();
        let address = listener.local_addr().unwrap();
        let client = std::thread::spawn(move || TcpStream::connect(address).unwrap());
        let (mut server, remote) = listener.accept().unwrap();
        let _client = client.join().unwrap();
        let previous = Some(Duration::from_secs(2));
        server.set_write_timeout(previous).unwrap();
        let mut ticket = admission.admit(remote).unwrap();
        admission.charge_input(&mut ticket, 64, 16).unwrap();
        let permit = admission.charge_response(&mut ticket, 8).unwrap();
        admission
            .send_tcp_response(&mut server, b"test", permit)
            .unwrap();
        assert_eq!(server.write_timeout().unwrap(), previous);

        let udp = UdpSocket::bind("127.0.0.1:0").unwrap();
        let receiver = UdpSocket::bind("127.0.0.1:0").unwrap();
        udp.set_write_timeout(previous).unwrap();
        let mut ticket = admission.admit(receiver.local_addr().unwrap()).unwrap();
        admission.charge_input(&mut ticket, 64, 16).unwrap();
        let permit = admission.charge_response(&mut ticket, 4).unwrap();
        admission
            .send_udp_response(&udp, b"test", receiver.local_addr().unwrap(), permit)
            .unwrap();
        assert_eq!(udp.write_timeout().unwrap(), previous);
    }

    #[test]
    fn inner_budget_rejection_terminalizes_outer_ticket() {
        let mut admission = ListenerAdmission::new();
        let peer: SocketAddr = "127.0.0.1:40080".parse().unwrap();
        let mut input = admission.admit_carrier(CarrierKind::Tcp, peer).unwrap();
        assert!(admission.charge_input(&mut input, usize::MAX, 1).is_err());
        assert!(admission.charge_input(&mut input, 1, 1).is_err());
        assert!(admission.enqueue(&mut input).is_err());
        assert!(admission.charge_response(&mut input, 1).is_err());
        admission.release(input);

        let mut response = admission.admit_carrier(CarrierKind::Tcp, peer).unwrap();
        admission.charge_input(&mut response, 1, 1).unwrap();
        assert!(admission.charge_response(&mut response, 4).is_err());
        assert!(admission.charge_input(&mut response, 1, 1).is_err());
        assert!(admission.enqueue(&mut response).is_err());
        admission.release(response);
    }

    #[test]
    fn staged_outer_rejection_terminalizes_inner_and_ticket() {
        let mut admission = ListenerAdmission::new();
        let peer: SocketAddr = "127.0.0.1:40081".parse().unwrap();
        let mut ticket = admission.admit_carrier(CarrierKind::Tcp, peer).unwrap();
        assert!(
            admission
                .begin_tcp_input_record(&mut ticket, 8192, 4097)
                .is_err()
        );
        assert!(admission.charge_input(&mut ticket, 1, 1).is_err());
        assert!(admission.charge_response(&mut ticket, 1).is_err());
        assert!(admission.enqueue(&mut ticket).is_err());
        admission.release(ticket);
    }

    #[test]
    fn response_requires_charged_input_and_release_reopens_source() {
        let mut admission = ListenerAdmission::new();
        let peer: SocketAddr = "127.0.0.1:40080".parse().unwrap();
        let mut rejected = admission.admit(peer).unwrap();
        assert!(admission.charge_response(&mut rejected, 1).is_err());
        assert!(admission.charge_input(&mut rejected, 64, 16).is_err());
        admission.release(rejected);

        let mut ticket = admission.admit_carrier(CarrierKind::Tcp, peer).unwrap();
        admission.charge_input(&mut ticket, 64, 16).unwrap();
        let permit = admission.charge_response(&mut ticket, 64).unwrap();
        admission
            .process
            .complete_response(permit, admission.now_ms())
            .unwrap();
        admission.release(ticket);
        assert!(admission.admit(peer).is_ok());
    }

    #[test]
    fn adversarial_same_source_reservations_are_bounded_and_redacted() {
        let mut admission = ListenerAdmission::new();
        let peer: SocketAddr = "198.51.100.7:40100".parse().unwrap();
        let mut tickets = Vec::new();
        for _ in 0..8 {
            tickets.push(admission.admit_carrier(CarrierKind::Udp, peer).unwrap());
        }
        assert!(admission.admit_carrier(CarrierKind::Udp, peer).is_err());
        assert_eq!(admission.process.live_states(), 8);
        let debug = format!("{:?}", admission.process);
        assert_eq!(
            debug,
            "ProcessPreauthAdmission { live_states: 8, source_count: 1, memory_bytes: 131072, queued: 0, window_input_bytes: 0, window_input_packets: 0, window_work_units: 0, window_response_bytes: 0, window_response_packets: 0 }"
        );
        assert!(!debug.contains("[198, 51, 100, 7]"));
        for ticket in tickets {
            admission.release(ticket);
        }
        assert_eq!(admission.process.live_states(), 0);
        assert_eq!(admission.process.memory_bytes(), 0);
    }

    #[test]
    fn adversarial_input_budget_rejects_before_parse_and_reopens_source() {
        let mut admission = ListenerAdmission::new();
        let peer: SocketAddr = "127.0.0.1:40101".parse().unwrap();
        let mut ticket = admission.admit_carrier(CarrierKind::Udp, peer).unwrap();
        for _ in 0..4 {
            admission.charge_input(&mut ticket, 64, 64).unwrap();
        }
        assert!(admission.charge_input(&mut ticket, 64, 64).is_err());
        assert!(admission.charge_response(&mut ticket, 1).is_err());
        admission.release(ticket);
        let next = admission.admit_carrier(CarrierKind::Udp, peer).unwrap();
        admission.release(next);
        assert_eq!(admission.process.live_states(), 0);
    }
}
