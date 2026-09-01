//! M0 carrier path-evidence state model.
//!
//! This portion of the crate opens no sockets, performs no routing or
//! tunnelling, and never performs a real failover.  The crate also contains
//! the separately scoped M1-S1 loopback UDP slice; evidence domains remain
//! deliberately separate so transport observations cannot become Session
//! delivery or path validation claims by accident.

use std::collections::BTreeMap;

#[derive(Debug, Clone, Copy, PartialEq, Eq, PartialOrd, Ord, Hash)]
pub struct PathId(pub u64);
#[derive(Debug, Clone, Copy, PartialEq, Eq, PartialOrd, Ord, Hash)]
pub struct PathGeneration(pub u64);

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum CarrierKind {
    Tcp,
    Udp,
    Quic,
    Other(u8),
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum PathValidationState {
    Candidate,
    Validating,
    Validated,
}
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum PathState {
    Candidate,
    Validating,
    Active,
    Degraded,
    Draining,
    Failed,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum PacketFeedbackKind {
    Ack,
    Loss,
    Reordered,
}
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct PacketFeedback {
    pub path: PathId,
    pub generation: PathGeneration,
    pub kind: PacketFeedbackKind,
}
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct PathValidated {
    pub path: PathId,
    pub generation: PathGeneration,
}
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct SessionDelivery {
    pub path: PathId,
    pub generation: PathGeneration,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum CarrierEvent {
    PathAdded {
        path: PathId,
        carrier: CarrierKind,
        generation: PathGeneration,
    },
    ChallengeSent {
        path: PathId,
        generation: PathGeneration,
    },
    ChallengeValidated(PathValidated),
    PacketFeedback(PacketFeedback),
    PtoExpired {
        path: PathId,
        generation: PathGeneration,
    },
    SessionDelivery(SessionDelivery),
    BeginDrain {
        path: PathId,
        generation: PathGeneration,
    },
    Fail {
        path: PathId,
        generation: PathGeneration,
    },
    Activate {
        path: PathId,
        generation: PathGeneration,
    },
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct Hysteresis {
    pub min_dwell_events: u32,
    pub k_successes: u32,
}
impl Default for Hysteresis {
    fn default() -> Self {
        Self {
            min_dwell_events: 2,
            k_successes: 2,
        }
    }
}
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct Limits {
    pub max_paths: usize,
    pub hysteresis: Hysteresis,
}
impl Default for Limits {
    fn default() -> Self {
        Self {
            max_paths: 8,
            hysteresis: Hysteresis::default(),
        }
    }
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct PathRecord {
    pub carrier: CarrierKind,
    pub generation: PathGeneration,
    pub validation: PathValidationState,
    pub state: PathState,
    pub successes: u32,
    pub dwell_events: u32,
}
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct PathSnapshot {
    pub paths: BTreeMap<PathId, PathRecord>,
    pub active: Option<(PathId, PathGeneration)>,
    pub active_epoch: u64,
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub enum PathError {
    ResourceLimit,
    PathExists,
    PathNotFound,
    GenerationMismatch,
    OldGeneration,
    InvalidTransition,
    ValidationRequired,
    ValidationDomain,
    ActivePathRequired,
    ActivePathConflict,
    HysteresisGate,
}

#[derive(Debug, Default)]
pub struct CarrierState {
    limits: Limits,
    paths: BTreeMap<PathId, PathRecord>,
    active: Option<(PathId, PathGeneration)>,
    active_epoch: u64,
}
impl CarrierState {
    pub fn new(limits: Limits) -> Self {
        Self {
            limits,
            ..Self::default()
        }
    }
    pub fn snapshot(&self) -> PathSnapshot {
        PathSnapshot {
            paths: self.paths.clone(),
            active: self.active,
            active_epoch: self.active_epoch,
        }
    }
    pub fn path(&self, id: PathId) -> Option<&PathRecord> {
        self.paths.get(&id)
    }
    pub fn active(&self) -> Option<(PathId, PathGeneration)> {
        self.active
    }

    pub fn apply(&mut self, event: CarrierEvent) -> Result<(), PathError> {
        match event {
            CarrierEvent::PathAdded {
                path,
                carrier,
                generation,
            } => {
                if self.paths.contains_key(&path) {
                    return Err(PathError::PathExists);
                }
                if self.paths.len() >= self.limits.max_paths {
                    return Err(PathError::ResourceLimit);
                }
                self.paths.insert(
                    path,
                    PathRecord {
                        carrier,
                        generation,
                        validation: PathValidationState::Candidate,
                        state: PathState::Candidate,
                        successes: 0,
                        dwell_events: 0,
                    },
                );
            }
            CarrierEvent::ChallengeSent { path, generation } => {
                let p = self.checked(path, generation)?;
                if p.validation != PathValidationState::Candidate || p.state != PathState::Candidate
                {
                    return Err(PathError::InvalidTransition);
                }
                p.validation = PathValidationState::Validating;
                p.state = PathState::Validating;
            }
            CarrierEvent::ChallengeValidated(v) => {
                let p = self.checked(v.path, v.generation)?;
                if p.validation != PathValidationState::Validating {
                    return Err(PathError::InvalidTransition);
                }
                p.validation = PathValidationState::Validated;
                p.state = PathState::Candidate;
                p.successes = 0;
            }
            CarrierEvent::PacketFeedback(f) => {
                let is_active = self.active == Some((f.path, f.generation));
                let p = self.checked(f.path, f.generation)?;
                match f.kind {
                    PacketFeedbackKind::Ack => {
                        if p.validation == PathValidationState::Validated {
                            p.successes = p.successes.saturating_add(1);
                        }
                    }
                    PacketFeedbackKind::Loss | PacketFeedbackKind::Reordered => {
                        p.state = if is_active {
                            PathState::Degraded
                        } else {
                            p.state
                        };
                    }
                }
            }
            CarrierEvent::PtoExpired { path, generation } => {
                let is_active = self.active == Some((path, generation));
                let p = self.checked(path, generation)?;
                if is_active {
                    p.state = PathState::Degraded;
                }
                // PTO is only health/probe evidence. It never means failed.
            }
            CarrierEvent::SessionDelivery(d) => {
                self.checked(d.path, d.generation)?;
            }
            CarrierEvent::BeginDrain { path, generation } => {
                let p = self.checked(path, generation)?;
                if p.state != PathState::Active {
                    return Err(PathError::InvalidTransition);
                }
                p.state = PathState::Draining;
            }
            CarrierEvent::Fail { path, generation } => {
                let p = self.checked(path, generation)?;
                if p.state != PathState::Degraded && p.state != PathState::Draining {
                    return Err(PathError::InvalidTransition);
                }
                p.state = PathState::Failed;
                if self.active == Some((path, generation)) {
                    self.active = None;
                }
            }
            CarrierEvent::Activate { path, generation } => {
                let k_successes = self.limits.hysteresis.k_successes;
                let min_dwell_events = self.limits.hysteresis.min_dwell_events;
                if self.active.is_some() {
                    return Err(PathError::ActivePathConflict);
                }
                {
                    let p = self.checked(path, generation)?;
                    if p.validation != PathValidationState::Validated {
                        return Err(PathError::ValidationRequired);
                    }
                    if p.state != PathState::Candidate {
                        return Err(PathError::InvalidTransition);
                    }
                    if p.successes < k_successes || p.dwell_events < min_dwell_events {
                        return Err(PathError::HysteresisGate);
                    }
                    p.state = PathState::Active;
                }
                self.active = Some((path, generation));
                self.active_epoch = self.active_epoch.saturating_add(1);
            }
        }
        self.tick_dwell();
        Ok(())
    }
    fn tick_dwell(&mut self) {
        for p in self.paths.values_mut() {
            if p.validation == PathValidationState::Validated {
                p.dwell_events = p.dwell_events.saturating_add(1);
            }
        }
    }
    fn checked(
        &mut self,
        path: PathId,
        generation: PathGeneration,
    ) -> Result<&mut PathRecord, PathError> {
        let p = self.paths.get_mut(&path).ok_or(PathError::PathNotFound)?;
        if generation != p.generation {
            return if generation.0 < p.generation.0 {
                Err(PathError::OldGeneration)
            } else {
                Err(PathError::GenerationMismatch)
            };
        }
        Ok(p)
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    const P: PathId = PathId(1);
    const G: PathGeneration = PathGeneration(1);
    fn state() -> CarrierState {
        CarrierState::new(Limits {
            max_paths: 2,
            hysteresis: Hysteresis {
                min_dwell_events: 2,
                k_successes: 2,
            },
        })
    }
    fn add(s: &mut CarrierState) {
        s.apply(CarrierEvent::PathAdded {
            path: P,
            carrier: CarrierKind::Udp,
            generation: G,
        })
        .unwrap();
    }
    fn validated(s: &mut CarrierState) {
        add(s);
        s.apply(CarrierEvent::ChallengeSent {
            path: P,
            generation: G,
        })
        .unwrap();
        s.apply(CarrierEvent::ChallengeValidated(PathValidated {
            path: P,
            generation: G,
        }))
        .unwrap();
    }
    #[test]
    fn ack_is_not_validation() {
        let mut s = state();
        add(&mut s);
        s.apply(CarrierEvent::PacketFeedback(PacketFeedback {
            path: P,
            generation: G,
            kind: PacketFeedbackKind::Ack,
        }))
        .unwrap();
        assert_eq!(
            s.path(P).unwrap().validation,
            PathValidationState::Candidate
        );
        assert_eq!(s.path(P).unwrap().successes, 0);
        assert_eq!(s.path(P).unwrap().dwell_events, 0);
    }
    #[test]
    fn pto_is_not_failure() {
        let mut s = state();
        validated(&mut s);
        assert_eq!(
            s.apply(CarrierEvent::PtoExpired {
                path: P,
                generation: G
            }),
            Ok(())
        );
        assert_ne!(s.path(P).unwrap().state, PathState::Failed);
    }
    #[test]
    fn old_generation_and_late_old_path_are_rejected() {
        let mut s = state();
        add(&mut s);
        assert_eq!(
            s.apply(CarrierEvent::ChallengeSent {
                path: P,
                generation: PathGeneration(0)
            }),
            Err(PathError::OldGeneration)
        );
        assert_eq!(
            s.apply(CarrierEvent::SessionDelivery(SessionDelivery {
                path: P,
                generation: PathGeneration(0)
            })),
            Err(PathError::OldGeneration)
        );
    }
    #[test]
    fn failover_gate_requires_validation_and_hysteresis() {
        let mut s = state();
        validated(&mut s);
        assert_eq!(
            s.apply(CarrierEvent::Activate {
                path: P,
                generation: G
            }),
            Err(PathError::HysteresisGate)
        );
        s.apply(CarrierEvent::PacketFeedback(PacketFeedback {
            path: P,
            generation: G,
            kind: PacketFeedbackKind::Ack,
        }))
        .unwrap();
        s.apply(CarrierEvent::PacketFeedback(PacketFeedback {
            path: P,
            generation: G,
            kind: PacketFeedbackKind::Ack,
        }))
        .unwrap();
        assert_eq!(
            s.apply(CarrierEvent::Activate {
                path: P,
                generation: G
            }),
            Ok(())
        );
        assert_eq!(s.active(), Some((P, G)));
    }
    #[test]
    fn duplicate_path_reports_exists_before_capacity() {
        let mut s = CarrierState::new(Limits {
            max_paths: 1,
            ..Limits::default()
        });
        let event = CarrierEvent::PathAdded {
            path: PathId(7),
            carrier: CarrierKind::Tcp,
            generation: PathGeneration(1),
        };
        s.apply(event).unwrap();
        assert_eq!(s.apply(event), Err(PathError::PathExists));
    }

    #[test]
    fn limits_and_invalid_transitions_are_deterministic() {
        let mut s = state();
        add(&mut s);
        assert_eq!(
            s.apply(CarrierEvent::ChallengeValidated(PathValidated {
                path: P,
                generation: G
            })),
            Err(PathError::InvalidTransition)
        );
        s.apply(CarrierEvent::PathAdded {
            path: PathId(2),
            carrier: CarrierKind::Tcp,
            generation: G,
        })
        .unwrap();
        assert_eq!(
            s.apply(CarrierEvent::PathAdded {
                path: PathId(3),
                carrier: CarrierKind::Tcp,
                generation: G
            }),
            Err(PathError::ResourceLimit)
        );
    }
}

/// Resource limits shared by all carrier adapters. Fields describe the
/// opaque message boundary exposed to Session, not a particular transport.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct CarrierLimits {
    pub max_message_bytes: usize,
    pub max_buffered_bytes: usize,
}

/// Carrier-neutral I/O failures. Native adapter errors are deliberately kept
/// behind this boundary; callers can distinguish retryable absence from a
/// terminal close without depending on Memory/TCP/UDP error enums.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum CarrierError {
    InvalidLimits,
    MessageTooLarge,
    BufferFull,
    Closed,
    PeerClosed,
    WouldBlock,
    Truncated,
    Io,
    StatePoisoned,
}

/// Observation returned by an adapter when an operation has no payload.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum IoObservation {
    WouldBlock,
    Closed,
}

/// Synchronous opaque-byte carrier contract. Memory, TCP and UDP are adapters
/// behind this interface; their native limits/errors never appear here.
pub trait Carrier {
    fn kind(&self) -> CarrierKind;
    fn properties(&self) -> CarrierProperties;
    fn limits(&self) -> CarrierLimits;
    fn send(&self, message: &[u8]) -> Result<(), CarrierError>;
    fn recv(&self) -> Result<Option<Vec<u8>>, CarrierError>;
    fn close(&self) -> Result<(), CarrierError>;
}

/// Local-only connected UDP datagrams. Kept separate from the cross-layer
/// Carrier trait so OS errors cannot become session or path evidence.
pub trait UdpCarrier {
    fn send_datagram(&self, message: &[u8]) -> Result<(), UdpError>;
    fn recv_datagram(&self) -> Result<Option<Vec<u8>>, UdpError>;
    fn close(&self) -> Result<(), UdpError>;
}

#[derive(Debug)]
pub enum UdpError {
    InvalidLimits,
    MessageTooLarge,
    WouldBlock,
    Io(std::io::Error),
    StatePoisoned,
}
impl std::fmt::Display for UdpError {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            Self::InvalidLimits => f.write_str("invalid UDP limits"),
            Self::MessageTooLarge => f.write_str("UDP datagram exceeds limit"),
            Self::WouldBlock => f.write_str("UDP receive would block"),
            Self::Io(e) => e.fmt(f),
            Self::StatePoisoned => f.write_str("UDP endpoint state poisoned"),
        }
    }
}
impl std::error::Error for UdpError {}
impl From<std::io::Error> for UdpError {
    fn from(e: std::io::Error) -> Self {
        if e.kind() == std::io::ErrorKind::WouldBlock {
            Self::WouldBlock
        } else {
            Self::Io(e)
        }
    }
}
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct UdpLimits {
    pub max_datagram_bytes: usize,
}
#[derive(Debug)]
pub struct UdpLoopbackEndpoint {
    socket: std::net::UdpSocket,
    limits: UdpLimits,
    closed: std::sync::Mutex<bool>,
}
pub struct UdpLoopbackPair;
impl UdpLoopbackPair {
    #[allow(clippy::new_ret_no_self)]
    pub fn new(limits: UdpLimits) -> Result<(UdpLoopbackEndpoint, UdpLoopbackEndpoint), UdpError> {
        if limits.max_datagram_bytes == 0 {
            return Err(UdpError::InvalidLimits);
        }
        let a = std::net::UdpSocket::bind((std::net::Ipv4Addr::LOCALHOST, 0))?;
        let b = std::net::UdpSocket::bind((std::net::Ipv4Addr::LOCALHOST, 0))?;
        a.connect(b.local_addr()?)?;
        b.connect(a.local_addr()?)?;
        a.set_nonblocking(true)?;
        b.set_nonblocking(true)?;
        Ok((Self::endpoint(a, limits), Self::endpoint(b, limits)))
    }
    fn endpoint(socket: std::net::UdpSocket, limits: UdpLimits) -> UdpLoopbackEndpoint {
        UdpLoopbackEndpoint {
            socket,
            limits,
            closed: std::sync::Mutex::new(false),
        }
    }
}
impl UdpLoopbackEndpoint {
    pub fn local_addr(&self) -> Result<std::net::SocketAddr, UdpError> {
        Ok(self.socket.local_addr()?)
    }
    fn closed(&self) -> Result<bool, UdpError> {
        Ok(*self.closed.lock().map_err(|_| UdpError::StatePoisoned)?)
    }
}
impl Carrier for UdpLoopbackEndpoint {
    fn kind(&self) -> CarrierKind {
        CarrierKind::Udp
    }
    fn properties(&self) -> CarrierProperties {
        CarrierProperties {
            message_boundaries: true,
            reliable: false,
            ordered: false,
        }
    }
    fn limits(&self) -> CarrierLimits {
        CarrierLimits {
            max_message_bytes: self.limits.max_datagram_bytes,
            max_buffered_bytes: self.limits.max_datagram_bytes,
        }
    }
    fn send(&self, message: &[u8]) -> Result<(), CarrierError> {
        self.send_datagram(message).map_err(Into::into)
    }
    fn recv(&self) -> Result<Option<Vec<u8>>, CarrierError> {
        self.recv_datagram().map_err(Into::into)
    }
    fn close(&self) -> Result<(), CarrierError> {
        UdpCarrier::close(self).map_err(Into::into)
    }
}
impl From<UdpError> for CarrierError {
    fn from(e: UdpError) -> Self {
        match e {
            UdpError::InvalidLimits => Self::InvalidLimits,
            UdpError::MessageTooLarge => Self::MessageTooLarge,
            UdpError::WouldBlock => Self::WouldBlock,
            UdpError::StatePoisoned => Self::StatePoisoned,
            UdpError::Io(_) => Self::Io,
        }
    }
}

impl UdpCarrier for UdpLoopbackEndpoint {
    fn send_datagram(&self, m: &[u8]) -> Result<(), UdpError> {
        if m.len() > self.limits.max_datagram_bytes {
            return Err(UdpError::MessageTooLarge);
        }
        if self.closed()? {
            return Err(UdpError::Io(std::io::Error::new(
                std::io::ErrorKind::NotConnected,
                "endpoint is closed",
            )));
        }
        self.socket.send(m).map(|_| ()).map_err(Into::into)
    }
    fn recv_datagram(&self) -> Result<Option<Vec<u8>>, UdpError> {
        if self.closed()? {
            return Ok(None);
        }
        let mut b = vec![0; self.limits.max_datagram_bytes.saturating_add(1)];
        match self.socket.recv(&mut b) {
            Ok(n) if n > self.limits.max_datagram_bytes => Err(UdpError::MessageTooLarge),
            Ok(n) => {
                b.truncate(n);
                Ok(Some(b))
            }
            Err(e) => Err(e.into()),
        }
    }
    fn close(&self) -> Result<(), UdpError> {
        *self.closed.lock().map_err(|_| UdpError::StatePoisoned)? = true;
        Ok(())
    }
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct CarrierProperties {
    pub message_boundaries: bool,
    pub reliable: bool,
    pub ordered: bool,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct MemoryLimits {
    pub max_message_bytes: usize,
    pub max_queue_bytes: usize,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum MemoryPairError {
    InvalidLimits,
    MessageTooLarge,
    QueueFull,
    Closed,
    PeerClosed,
    ArithmeticOverflow,
    StatePoisoned,
}

#[derive(Debug)]
struct MemoryState {
    queues: [std::collections::VecDeque<Vec<u8>>; 2],
    queue_bytes: [usize; 2],
    closed: [bool; 2],
}

#[derive(Debug)]
struct MemoryShared {
    limits: MemoryLimits,
    state: std::sync::Mutex<MemoryState>,
}

/// In-process bounded pair. No runtime, socket, route, tunnel, or service effects.
#[derive(Debug, Clone)]
pub struct MemoryEndpoint {
    shared: std::sync::Arc<MemoryShared>,
    side: usize,
}

#[derive(Debug, Clone, Copy)]
pub struct MemoryPair;

impl MemoryPair {
    #[allow(clippy::new_ret_no_self)]
    pub fn new(limits: MemoryLimits) -> Result<(MemoryEndpoint, MemoryEndpoint), MemoryPairError> {
        if limits.max_message_bytes == 0
            || limits.max_queue_bytes == 0
            || limits.max_message_bytes > limits.max_queue_bytes
        {
            return Err(MemoryPairError::InvalidLimits);
        }
        let shared = std::sync::Arc::new(MemoryShared {
            limits,
            state: std::sync::Mutex::new(MemoryState {
                queues: std::array::from_fn(|_| std::collections::VecDeque::new()),
                queue_bytes: [0; 2],
                closed: [false; 2],
            }),
        });
        Ok((
            MemoryEndpoint {
                shared: shared.clone(),
                side: 0,
            },
            MemoryEndpoint { shared, side: 1 },
        ))
    }
}

impl MemoryEndpoint {
    fn lock(&self) -> Result<std::sync::MutexGuard<'_, MemoryState>, MemoryPairError> {
        self.shared
            .state
            .lock()
            .map_err(|_| MemoryPairError::StatePoisoned)
    }
}

impl Carrier for MemoryEndpoint {
    fn kind(&self) -> CarrierKind {
        CarrierKind::Other(0)
    }
    fn properties(&self) -> CarrierProperties {
        CarrierProperties {
            message_boundaries: true,
            reliable: false,
            ordered: true,
        }
    }
    fn limits(&self) -> CarrierLimits {
        CarrierLimits {
            max_message_bytes: self.shared.limits.max_message_bytes,
            max_buffered_bytes: self.shared.limits.max_queue_bytes,
        }
    }
    fn send(&self, message: &[u8]) -> Result<(), CarrierError> {
        let limits = self.shared.limits;
        if message.len() > limits.max_message_bytes {
            return Err(CarrierError::MessageTooLarge);
        }
        // One pair mutex linearizes local close, peer close, capacity, and enqueue.
        let mut state = self.lock().map_err(|_| CarrierError::StatePoisoned)?;
        if state.closed[self.side] {
            return Err(CarrierError::Closed);
        }
        let peer = 1 - self.side;
        if state.closed[peer] {
            return Err(CarrierError::PeerClosed);
        }
        let total = state.queue_bytes[peer]
            .checked_add(message.len())
            .ok_or(CarrierError::BufferFull)?;
        if total > limits.max_queue_bytes {
            return Err(CarrierError::BufferFull);
        }
        state.queues[peer].push_back(message.to_vec());
        state.queue_bytes[peer] = total;
        Ok(())
    }
    fn recv(&self) -> Result<Option<Vec<u8>>, CarrierError> {
        let mut state = self.lock().map_err(|_| CarrierError::StatePoisoned)?;
        let message = state.queues[self.side].pop_front();
        if let Some(ref bytes) = message {
            state.queue_bytes[self.side] = state.queue_bytes[self.side]
                .checked_sub(bytes.len())
                .ok_or(CarrierError::Io)?;
        }
        Ok(message)
    }
    fn close(&self) -> Result<(), CarrierError> {
        let mut state = self.lock().map_err(|_| CarrierError::StatePoisoned)?;
        // Close is idempotent; queued data remains available to the peer.
        state.closed[self.side] = true;
        Ok(())
    }
}

#[cfg(test)]
mod memory_pair_tests {
    use super::*;
    use crate::Carrier;
    use std::sync::{Arc, Barrier, mpsc};
    use std::time::Duration;

    fn pair() -> (MemoryEndpoint, MemoryEndpoint) {
        MemoryPair::new(MemoryLimits {
            max_message_bytes: 4,
            max_queue_bytes: 6,
        })
        .unwrap()
    }

    #[test]
    fn bidirectional_fifo_boundaries_and_input_copy() {
        let (a, b) = pair();
        let mut input = vec![1, 2];
        a.send(&input).unwrap();
        input[0] = 9;
        a.send(&[]).unwrap();
        a.send(&[3]).unwrap();
        assert_eq!(b.recv().unwrap(), Some(vec![1, 2]));
        assert_eq!(b.recv().unwrap(), Some(vec![]));
        assert_eq!(b.recv().unwrap(), Some(vec![3]));
        assert_eq!(b.recv().unwrap(), None);
        b.send(b"ok").unwrap();
        assert_eq!(a.recv().unwrap(), Some(b"ok".to_vec()));
    }

    #[test]
    fn limits_and_queue_full_are_atomic() {
        for limits in [
            MemoryLimits {
                max_message_bytes: 0,
                max_queue_bytes: 1,
            },
            MemoryLimits {
                max_message_bytes: 2,
                max_queue_bytes: 0,
            },
            MemoryLimits {
                max_message_bytes: 3,
                max_queue_bytes: 2,
            },
            MemoryLimits {
                max_message_bytes: usize::MAX,
                max_queue_bytes: usize::MAX - 1,
            },
        ] {
            assert!(matches!(
                MemoryPair::new(limits),
                Err(MemoryPairError::InvalidLimits)
            ));
        }
        let (a, b) = pair();
        a.send(b"1234").unwrap();
        assert_eq!(a.send(b"567"), Err(CarrierError::BufferFull));
        assert_eq!(b.recv().unwrap(), Some(b"1234".to_vec()));
        assert_eq!(a.send(b"567"), Ok(()));
        assert_eq!(a.send(b"12345"), Err(CarrierError::MessageTooLarge));
    }

    #[test]
    fn close_is_idempotent_and_preserves_queued_data() {
        let (a, b) = pair();
        a.send(b"data").unwrap();
        a.close().unwrap();
        a.close().unwrap();
        assert_eq!(a.send(b"x"), Err(CarrierError::Closed));
        // The peer drains data queued before close, then observes an empty queue.
        assert_eq!(b.recv().unwrap(), Some(b"data".to_vec()));
        assert_eq!(b.recv().unwrap(), None);

        b.close().unwrap();
        b.close().unwrap();
        assert_eq!(b.send(b"x"), Err(CarrierError::Closed));
        let (open, closed_peer) = pair();
        closed_peer.close().unwrap();
        assert_eq!(open.send(b"x"), Err(CarrierError::PeerClosed));
        assert_eq!(open.recv().unwrap(), None);
    }

    #[test]
    fn concurrent_bidirectional_send_completes_without_deadlock() {
        let (a, b) = pair();
        let barrier = Arc::new(Barrier::new(2));
        let (done_tx, done_rx) = mpsc::channel();
        let a_barrier = Arc::clone(&barrier);
        let a_done = done_tx.clone();
        let a_thread = std::thread::spawn(move || {
            a_barrier.wait();
            let result = a.send(b"a");
            a_done.send(result).unwrap();
        });
        let b_barrier = Arc::clone(&barrier);
        let b_done = done_tx;
        let b_thread = std::thread::spawn(move || {
            b_barrier.wait();
            let result = b.send(b"b");
            b_done.send(result).unwrap();
        });

        // Any timeout or send error is an explicit failure. The failure path
        // does not join a potentially stuck worker.
        match done_rx.recv_timeout(Duration::from_secs(2)) {
            Ok(Ok(())) => {}
            Ok(Err(error)) => panic!("concurrent send failed: {error:?}"),
            Err(error) => panic!("concurrent sends did not complete: {error}"),
        }
        match done_rx.recv_timeout(Duration::from_secs(2)) {
            Ok(Ok(())) => {}
            Ok(Err(error)) => panic!("concurrent send failed: {error:?}"),
            Err(error) => panic!("concurrent sends did not complete: {error}"),
        }
        // Joining occurs only after both workers reported Ok(()).
        a_thread.join().unwrap();
        b_thread.join().unwrap();
    }

    #[test]
    fn contract_is_opaque_and_evidence_free() {
        let (a, _) = pair();
        assert_eq!(a.kind(), CarrierKind::Other(0));
        assert_eq!(
            a.properties(),
            CarrierProperties {
                message_boundaries: true,
                reliable: false,
                ordered: true
            }
        );
        assert_eq!(
            a.limits(),
            CarrierLimits {
                max_message_bytes: 4,
                max_buffered_bytes: 6
            }
        );
    }
}

#[cfg(test)]
mod udp_loopback_tests {
    use super::{UdpCarrier, UdpError, UdpLimits, UdpLoopbackPair};
    #[test]
    fn loopback_preserves_boundaries_empty_and_would_block() {
        let (a, b) = UdpLoopbackPair::new(UdpLimits {
            max_datagram_bytes: 8,
        })
        .unwrap();
        assert!(matches!(b.recv_datagram(), Err(UdpError::WouldBlock)));
        assert!(a.local_addr().unwrap().ip().is_loopback());
        assert!(b.local_addr().unwrap().ip().is_loopback());
        a.send_datagram(b"").unwrap();
        a.send_datagram(b"opaque").unwrap();
        assert_eq!(b.recv_datagram().unwrap(), Some(Vec::new()));
        assert_eq!(b.recv_datagram().unwrap(), Some(b"opaque".to_vec()));
        b.send_datagram(b"reply").unwrap();
        assert_eq!(a.recv_datagram().unwrap(), Some(b"reply".to_vec()));
    }
    #[test]
    fn oversize_is_rejected_and_close_is_local_idempotent() {
        let (a, b) = UdpLoopbackPair::new(UdpLimits {
            max_datagram_bytes: 2,
        })
        .unwrap();
        assert!(matches!(
            a.send_datagram(b"123"),
            Err(UdpError::MessageTooLarge)
        ));
        a.close().unwrap();
        a.close().unwrap();
        assert!(a.recv_datagram().unwrap().is_none());
        b.send_datagram(b"ok").unwrap();
        assert!(a.recv_datagram().unwrap().is_none());
    }
}

/// Bounded length-prefixed TCP carrier for loopback research. TCP reliability
/// is used directly; this layer intentionally has no packet ACK mechanism.
pub trait TcpCarrier {
    fn send_frame(&self, frame: &[u8]) -> Result<(), TcpError>;
    fn recv_frame(&self) -> Result<Vec<u8>, TcpError>;
    fn close(&self) -> Result<(), TcpError>;
}

#[derive(Debug)]
pub enum TcpError {
    InvalidLimits,
    FrameTooLarge,
    Closed,
    Truncated,
    Io(std::io::Error),
    StatePoisoned,
}
impl std::fmt::Display for TcpError {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            Self::InvalidLimits => f.write_str("invalid TCP limits"),
            Self::FrameTooLarge => f.write_str("TCP frame exceeds limit"),
            Self::Closed => f.write_str("TCP endpoint closed"),
            Self::Truncated => f.write_str("TCP frame truncated"),
            Self::Io(e) => e.fmt(f),
            Self::StatePoisoned => f.write_str("TCP endpoint state poisoned"),
        }
    }
}
impl std::error::Error for TcpError {}
impl From<std::io::Error> for TcpError {
    fn from(value: std::io::Error) -> Self {
        Self::Io(value)
    }
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct TcpLimits {
    pub max_frame_bytes: usize,
}
#[derive(Debug)]
pub struct TcpLoopbackEndpoint {
    stream: std::sync::Mutex<std::net::TcpStream>,
    limits: TcpLimits,
    closed: std::sync::Mutex<bool>,
}
pub struct TcpLoopbackPair;
impl TcpLoopbackPair {
    #[allow(clippy::new_ret_no_self)]
    pub fn new(limits: TcpLimits) -> Result<(TcpLoopbackEndpoint, TcpLoopbackEndpoint), TcpError> {
        if limits.max_frame_bytes == 0 || limits.max_frame_bytes > u32::MAX as usize {
            return Err(TcpError::InvalidLimits);
        }
        let listener = std::net::TcpListener::bind((std::net::Ipv4Addr::LOCALHOST, 0))?;
        let client = std::net::TcpStream::connect(listener.local_addr()?)?;
        let (server, peer) = listener.accept()?;
        if !peer.ip().is_loopback()
            || !client.local_addr()?.ip().is_loopback()
            || !server.local_addr()?.ip().is_loopback()
        {
            return Err(TcpError::InvalidLimits);
        }
        for stream in [&client, &server] {
            stream.set_nodelay(true)?;
            stream.set_read_timeout(Some(std::time::Duration::from_secs(1)))?;
            stream.set_write_timeout(Some(std::time::Duration::from_secs(1)))?;
        }
        Ok((
            Self::endpoint(client, limits),
            Self::endpoint(server, limits),
        ))
    }
    fn endpoint(stream: std::net::TcpStream, limits: TcpLimits) -> TcpLoopbackEndpoint {
        TcpLoopbackEndpoint {
            stream: std::sync::Mutex::new(stream),
            limits,
            closed: std::sync::Mutex::new(false),
        }
    }
}
impl TcpLoopbackEndpoint {
    pub fn local_addr(&self) -> Result<std::net::SocketAddr, TcpError> {
        Ok(self
            .stream
            .lock()
            .map_err(|_| TcpError::StatePoisoned)?
            .local_addr()?)
    }
    fn is_closed(&self) -> Result<bool, TcpError> {
        Ok(*self.closed.lock().map_err(|_| TcpError::StatePoisoned)?)
    }
}
impl Carrier for TcpLoopbackEndpoint {
    fn kind(&self) -> CarrierKind {
        CarrierKind::Tcp
    }
    fn properties(&self) -> CarrierProperties {
        CarrierProperties {
            message_boundaries: true,
            reliable: true,
            ordered: true,
        }
    }
    fn limits(&self) -> CarrierLimits {
        CarrierLimits {
            max_message_bytes: self.limits.max_frame_bytes,
            max_buffered_bytes: self.limits.max_frame_bytes,
        }
    }
    fn send(&self, message: &[u8]) -> Result<(), CarrierError> {
        self.send_frame(message).map_err(Into::into)
    }
    fn recv(&self) -> Result<Option<Vec<u8>>, CarrierError> {
        self.recv_frame().map(Some).map_err(Into::into)
    }
    fn close(&self) -> Result<(), CarrierError> {
        TcpCarrier::close(self).map_err(Into::into)
    }
}
impl From<TcpError> for CarrierError {
    fn from(e: TcpError) -> Self {
        match e {
            TcpError::InvalidLimits => Self::InvalidLimits,
            TcpError::FrameTooLarge => Self::MessageTooLarge,
            TcpError::Closed => Self::Closed,
            TcpError::Truncated => Self::Truncated,
            TcpError::StatePoisoned => Self::StatePoisoned,
            TcpError::Io(_) => Self::Io,
        }
    }
}

impl TcpCarrier for TcpLoopbackEndpoint {
    fn send_frame(&self, frame: &[u8]) -> Result<(), TcpError> {
        use std::io::Write;
        if self.is_closed()? {
            return Err(TcpError::Closed);
        }
        if frame.len() > self.limits.max_frame_bytes {
            return Err(TcpError::FrameTooLarge);
        }
        let length = u32::try_from(frame.len()).map_err(|_| TcpError::FrameTooLarge)?;
        let mut stream = self.stream.lock().map_err(|_| TcpError::StatePoisoned)?;
        stream.write_all(&length.to_be_bytes())?;
        stream.write_all(frame)?;
        stream.flush()?;
        Ok(())
    }
    fn recv_frame(&self) -> Result<Vec<u8>, TcpError> {
        use std::io::Read;
        if self.is_closed()? {
            return Err(TcpError::Closed);
        }
        let mut stream = self.stream.lock().map_err(|_| TcpError::StatePoisoned)?;
        let mut header = [0; 4];
        stream.read_exact(&mut header).map_err(|e| {
            if e.kind() == std::io::ErrorKind::UnexpectedEof {
                TcpError::Truncated
            } else {
                TcpError::Io(e)
            }
        })?;
        let length = u32::from_be_bytes(header) as usize;
        if length > self.limits.max_frame_bytes {
            return Err(TcpError::FrameTooLarge);
        }
        let mut frame = vec![0; length];
        stream.read_exact(&mut frame).map_err(|e| {
            if e.kind() == std::io::ErrorKind::UnexpectedEof {
                TcpError::Truncated
            } else {
                TcpError::Io(e)
            }
        })?;
        Ok(frame)
    }
    fn close(&self) -> Result<(), TcpError> {
        let mut closed = self.closed.lock().map_err(|_| TcpError::StatePoisoned)?;
        if !*closed {
            self.stream
                .lock()
                .map_err(|_| TcpError::StatePoisoned)?
                .shutdown(std::net::Shutdown::Both)?;
            *closed = true;
        }
        Ok(())
    }
}

/// Deterministic local-only fault wrapper for carrier state-machine tests.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Default)]
pub struct FaultPolicy {
    pub blackhole_after: Option<u64>,
    pub loss_percent: u8,
    pub duplicate: bool,
    pub reorder: bool,
    pub delay_ms: u32,
    pub close_after: Option<u64>,
    pub one_way: bool,
}

#[derive(Debug)]
pub struct FaultInjectCarrier<C: Carrier> {
    inner: C,
    policy: FaultPolicy,
    sent: std::sync::Mutex<u64>,
    seed: std::sync::Mutex<u64>,
    closed: std::sync::Mutex<bool>,
}
impl<C: Carrier> FaultInjectCarrier<C> {
    pub fn new(inner: C, policy: FaultPolicy, seed: u64) -> Result<Self, CarrierError> {
        if policy.loss_percent > 100 {
            return Err(CarrierError::InvalidLimits);
        }
        Ok(Self {
            inner,
            policy,
            sent: std::sync::Mutex::new(0),
            seed: std::sync::Mutex::new(seed),
            closed: std::sync::Mutex::new(false),
        })
    }
    fn draw(&self) -> u8 {
        let mut s = self.seed.lock().expect("fault seed");
        *s = s.wrapping_mul(6364136223846793005).wrapping_add(1);
        (*s >> 56) as u8
    }
    fn blocked(&self, sent: u64) -> bool {
        self.policy.blackhole_after.is_some_and(|n| sent >= n)
            || self.policy.close_after.is_some_and(|n| sent >= n)
    }
    pub fn inner(&self) -> &C {
        &self.inner
    }
    pub fn policy(&self) -> FaultPolicy {
        self.policy
    }
}
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum FaultEvent {
    Insert,
    Send,
    Loss,
    Uncertain,
    GenerationChange,
    Duplicate,
    OldAck,
    NewAck,
    Drain,
    Fail,
    Activate,
}
pub fn generated_fault_sequence(seed: u64, length: usize) -> Vec<FaultEvent> {
    let events = [
        FaultEvent::Insert,
        FaultEvent::Send,
        FaultEvent::Loss,
        FaultEvent::Uncertain,
        FaultEvent::GenerationChange,
        FaultEvent::Duplicate,
        FaultEvent::OldAck,
        FaultEvent::NewAck,
        FaultEvent::Drain,
        FaultEvent::Fail,
        FaultEvent::Activate,
    ];
    let mut x = seed;
    let n = length.min(4096);
    let mut out = Vec::with_capacity(n);
    for _ in 0..n {
        x = x.wrapping_mul(6364136223846793005).wrapping_add(1);
        out.push(events[((x >> 56) as usize) % events.len()]);
    }
    out
}

#[cfg(test)]
mod fault_inject_tests {
    use super::*;
    use crate::Carrier;
    #[test]
    fn deterministic_blackhole_loss_and_close_are_bounded() {
        let (a, b) = MemoryPair::new(MemoryLimits {
            max_message_bytes: 8,
            max_queue_bytes: 64,
        })
        .unwrap();
        let f = FaultInjectCarrier::new(
            a,
            FaultPolicy {
                blackhole_after: Some(2),
                ..Default::default()
            },
            7,
        )
        .unwrap();
        f.send(b"one").unwrap();
        f.send(b"two").unwrap();
        f.send(b"three").unwrap();
        assert_eq!(b.recv().unwrap(), Some(b"one".to_vec()));
        assert_eq!(b.recv().unwrap(), Some(b"two".to_vec()));
        assert_eq!(b.recv().unwrap(), None);
        assert_eq!(
            FaultInjectCarrier::new(
                b,
                FaultPolicy {
                    loss_percent: 101,
                    ..Default::default()
                },
                1
            )
            .unwrap_err(),
            CarrierError::InvalidLimits
        );
    }
}

impl<C: Carrier> Carrier for FaultInjectCarrier<C> {
    fn kind(&self) -> CarrierKind {
        self.inner.kind()
    }
    fn properties(&self) -> CarrierProperties {
        self.inner.properties()
    }
    fn limits(&self) -> CarrierLimits {
        self.inner.limits()
    }
    fn send(&self, message: &[u8]) -> Result<(), CarrierError> {
        // Wrapper is intentionally single-thread deterministic; mutation is managed through interior-free clone construction in tests.
        let mut count = self.sent.lock().map_err(|_| CarrierError::StatePoisoned)?;
        let sent = *count;
        *count = count.saturating_add(1);
        drop(count);
        if *self
            .closed
            .lock()
            .map_err(|_| CarrierError::StatePoisoned)?
            || self.blocked(sent)
            || (self.policy.one_way && sent % 2 == 0)
        {
            return Ok(());
        }
        if self.policy.loss_percent > 0 && self.draw() % 100 < self.policy.loss_percent {
            return Ok(());
        }
        if self.policy.delay_ms > 0 {
            std::thread::sleep(std::time::Duration::from_millis(
                self.policy.delay_ms as u64,
            ));
        }
        self.inner.send(message)
    }
    fn recv(&self) -> Result<Option<Vec<u8>>, CarrierError> {
        self.inner.recv()
    }
    fn close(&self) -> Result<(), CarrierError> {
        self.inner.close()
    }
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct CarrierCapabilities {
    pub message_boundaries: bool,
    pub reliable: bool,
    pub ordered: bool,
    pub packet_feedback: bool,
}
pub const UDP_CAPABILITIES: CarrierCapabilities = CarrierCapabilities {
    message_boundaries: true,
    reliable: false,
    ordered: false,
    packet_feedback: true,
};
pub const TCP_CAPABILITIES: CarrierCapabilities = CarrierCapabilities {
    message_boundaries: false,
    reliable: true,
    ordered: true,
    packet_feedback: false,
};

#[derive(Debug, Clone, Copy, PartialEq, Eq, PartialOrd, Ord)]
pub struct DataId(pub u64);
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum ActiveCarrier {
    Udp,
    Tcp,
}
#[derive(Debug, Clone, Copy, PartialEq, Eq, Default)]
pub struct FailoverMetrics {
    pub switches: u64,
    pub recovery_events: u64,
    pub duplicate_bytes: u64,
    pub delivered_bytes: u64,
    pub last_recovery_latency_us: Option<u64>,
}
#[derive(Debug, Clone, PartialEq, Eq)]
pub enum FailoverError {
    InvalidLimit,
    Capacity,
    Conflict,
    NotFound,
    WrongCarrier,
}
#[derive(Debug)]
pub struct FailoverController {
    active: ActiveCarrier,
    hard_failure_ptos: u32,
    consecutive_ptos: u32,
    max_uncertain_entries: usize,
    max_uncertain_bytes: usize,
    uncertain_bytes: usize,
    failure_started_us: Option<u64>,
    uncertain: BTreeMap<DataId, Vec<u8>>,
    received: BTreeMap<DataId, Vec<u8>>,
    pub metrics: FailoverMetrics,
}
impl FailoverController {
    pub fn new(
        hard_failure_ptos: u32,
        max_uncertain_entries: usize,
        max_uncertain_bytes: usize,
    ) -> Result<Self, FailoverError> {
        if hard_failure_ptos == 0 || max_uncertain_entries == 0 || max_uncertain_bytes == 0 {
            return Err(FailoverError::InvalidLimit);
        }
        Ok(Self {
            active: ActiveCarrier::Udp,
            hard_failure_ptos,
            consecutive_ptos: 0,
            max_uncertain_entries,
            max_uncertain_bytes,
            uncertain_bytes: 0,
            failure_started_us: None,
            uncertain: BTreeMap::new(),
            received: BTreeMap::new(),
            metrics: FailoverMetrics::default(),
        })
    }
    pub const fn active(&self) -> ActiveCarrier {
        self.active
    }
    pub fn track_uncertain(&mut self, id: DataId, data: &[u8]) -> Result<(), FailoverError> {
        if let Some(old) = self.uncertain.get(&id) {
            return if old == data {
                Ok(())
            } else {
                Err(FailoverError::Conflict)
            };
        }
        let total = self
            .uncertain_bytes
            .checked_add(data.len())
            .ok_or(FailoverError::Capacity)?;
        if self.uncertain.len() >= self.max_uncertain_entries || total > self.max_uncertain_bytes {
            return Err(FailoverError::Capacity);
        }
        self.uncertain.insert(id, data.to_vec());
        self.uncertain_bytes = total;
        Ok(())
    }
    pub fn udp_progress(&mut self) {
        self.consecutive_ptos = 0;
        self.failure_started_us = None;
    }
    pub fn udp_pto(&mut self) -> bool {
        self.udp_pto_at(0)
    }
    pub fn udp_pto_at(&mut self, now_us: u64) -> bool {
        if self.active != ActiveCarrier::Udp {
            return false;
        }
        self.failure_started_us.get_or_insert(now_us);
        self.consecutive_ptos = self.consecutive_ptos.saturating_add(1);
        if self.consecutive_ptos >= self.hard_failure_ptos {
            self.active = ActiveCarrier::Tcp;
            self.metrics.switches = self.metrics.switches.saturating_add(1);
            self.metrics.recovery_events = self.metrics.recovery_events.saturating_add(1);
            self.metrics.last_recovery_latency_us = self
                .failure_started_us
                .and_then(|started| now_us.checked_sub(started));
            true
        } else {
            false
        }
    }
    /// Applies a switch already owned and generation-checked by CarrierManager.
    pub fn apply_manager_decision(&mut self, decision: &CarrierSwitchDecision) -> bool {
        if self.active != ActiveCarrier::Udp
            || decision.from != ActiveCarrier::Udp
            || decision.to != ActiveCarrier::Tcp
        {
            return false;
        }
        self.active = ActiveCarrier::Tcp;
        self.metrics.switches = self.metrics.switches.saturating_add(1);
        self.metrics.recovery_events = self.metrics.recovery_events.saturating_add(1);
        true
    }
    pub fn record_recovery_latency(&mut self, latency_us: u64) {
        self.metrics.last_recovery_latency_us = Some(latency_us);
    }
    pub fn tcp_resend(&self) -> Result<Vec<(DataId, Vec<u8>)>, FailoverError> {
        if self.active != ActiveCarrier::Tcp {
            return Err(FailoverError::WrongCarrier);
        }
        Ok(self
            .uncertain
            .iter()
            .map(|(id, v)| (*id, v.clone()))
            .collect())
    }
    pub fn confirm(&mut self, id: DataId) -> Result<(), FailoverError> {
        let v = self.uncertain.remove(&id).ok_or(FailoverError::NotFound)?;
        self.uncertain_bytes = self.uncertain_bytes.saturating_sub(v.len());
        Ok(())
    }
    /// Returns true once for first delivery and false for an exact duplicate.
    pub fn receive(&mut self, id: DataId, data: &[u8]) -> Result<bool, FailoverError> {
        if let Some(old) = self.received.get(&id) {
            if old != data {
                return Err(FailoverError::Conflict);
            }
            self.metrics.duplicate_bytes = self
                .metrics
                .duplicate_bytes
                .saturating_add(data.len() as u64);
            return Ok(false);
        }
        if self.received.len() >= self.max_uncertain_entries {
            return Err(FailoverError::Capacity);
        }
        self.received.insert(id, data.to_vec());
        self.metrics.delivered_bytes = self
            .metrics
            .delivered_bytes
            .saturating_add(data.len() as u64);
        Ok(true)
    }
}

#[cfg(test)]
mod tcp_failover_tests {
    use super::*;
    #[test]
    fn tcp_framing_preserves_empty_and_boundaries() {
        let (a, b) = TcpLoopbackPair::new(TcpLimits { max_frame_bytes: 8 }).unwrap();
        assert!(a.local_addr().unwrap().ip().is_loopback());
        a.send_frame(b"").unwrap();
        a.send_frame(b"12345678").unwrap();
        assert_eq!(b.recv_frame().unwrap(), b"");
        assert_eq!(b.recv_frame().unwrap(), b"12345678");
        assert!(matches!(
            a.send_frame(b"123456789"),
            Err(TcpError::FrameTooLarge)
        ));
        TcpCarrier::close(&a).unwrap();
        TcpCarrier::close(&a).unwrap();
    }
    #[test]
    fn capabilities_do_not_duplicate_tcp_packet_ack() {
        let capabilities = [UDP_CAPABILITIES, TCP_CAPABILITIES];
        assert!(capabilities[0].packet_feedback);
        assert!(!capabilities[1].packet_feedback);
        assert!(capabilities[1].reliable && capabilities[1].ordered);
    }
    #[test]
    fn hard_failure_switches_and_resends_uncertain_with_dedup() {
        let mut f = FailoverController::new(2, 4, 64).unwrap();
        f.track_uncertain(DataId(1), b"cat").unwrap();
        assert!(!f.udp_pto());
        assert!(f.udp_pto());
        assert_eq!(f.active(), ActiveCarrier::Tcp);
        let resend = f.tcp_resend().unwrap();
        assert_eq!(resend, vec![(DataId(1), b"cat".to_vec())]);
        assert_eq!(f.receive(DataId(1), b"cat"), Ok(true));
        assert_eq!(f.receive(DataId(1), b"cat"), Ok(false));
        assert_eq!(f.receive(DataId(1), b"dog"), Err(FailoverError::Conflict));
        f.confirm(DataId(1)).unwrap();
        assert_eq!(f.metrics.delivered_bytes, 3);
        assert_eq!(f.metrics.duplicate_bytes, 3);
    }
    #[test]
    fn uncertain_duplicate_and_counter_boundaries_are_atomic() {
        let mut f = FailoverController::new(u32::MAX, 2, 4).unwrap();
        f.track_uncertain(DataId(u64::MAX), b"cat").unwrap();
        assert_eq!(f.track_uncertain(DataId(u64::MAX), b"cat"), Ok(()));
        assert_eq!(
            f.track_uncertain(DataId(u64::MAX), b"dog"),
            Err(FailoverError::Conflict)
        );
        assert_eq!(f.confirm(DataId(7)), Err(FailoverError::NotFound));
        assert_eq!(f.tcp_resend(), Err(FailoverError::WrongCarrier));
        assert!(!f.udp_pto_at(u64::MAX));
        assert_eq!(f.active(), ActiveCarrier::Udp);
        // The second PTO reaches the configured threshold without wrapping counters.
        let mut g = FailoverController::new(2, 1, 1).unwrap();
        assert!(!g.udp_pto_at(u64::MAX - 1));
        assert!(g.udp_pto_at(u64::MAX));
        assert_eq!(g.metrics.last_recovery_latency_us, Some(1));
    }

    #[test]
    fn recovery_latency_is_monotonic_and_bounded() {
        let mut f = FailoverController::new(2, 1, 8).unwrap();
        assert!(!f.udp_pto_at(10_000));
        assert!(f.udp_pto_at(25_000));
        assert_eq!(f.metrics.last_recovery_latency_us, Some(15_000));
    }
    #[test]
    fn uncertain_limits_are_atomic() {
        let mut f = FailoverController::new(1, 1, 3).unwrap();
        assert_eq!(
            f.track_uncertain(DataId(1), b"four"),
            Err(FailoverError::Capacity)
        );
        f.track_uncertain(DataId(1), b"cat").unwrap();
        assert_eq!(
            f.track_uncertain(DataId(2), b"x"),
            Err(FailoverError::Capacity)
        );
    }
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, PartialOrd, Ord, Hash)]
pub struct StreamId(pub u64);
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct FlowLimits {
    pub max_streams: usize,
    pub max_session_bytes: usize,
    pub max_stream_bytes: usize,
}
impl Default for FlowLimits {
    fn default() -> Self {
        Self {
            max_streams: 64,
            max_session_bytes: 1 << 20,
            max_stream_bytes: 1 << 18,
        }
    }
}
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum StreamPriority {
    Interactive,
    Bulk,
}
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct StreamSnapshot {
    pub id: StreamId,
    pub priority: StreamPriority,
    pub queued_bytes: usize,
    pub max_bytes: usize,
}
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum FlowError {
    InvalidLimit,
    TooManyStreams,
    StreamNotFound,
    StreamLimit,
    SessionLimit,
    EmptyData,
}
#[derive(Debug)]
struct FlowStream {
    priority: StreamPriority,
    queue: std::collections::VecDeque<Vec<u8>>,
    queued_bytes: usize,
    max_bytes: usize,
}
#[derive(Debug)]
pub struct FairScheduler {
    limits: FlowLimits,
    streams: BTreeMap<StreamId, FlowStream>,
    session_bytes: usize,
    cursor: usize,
    order: Vec<StreamId>,
    consecutive_interactive: usize,
}

/// Bound interactive work ahead of bulk work deterministically.
const INTERACTIVE_BURST: usize = 3;
impl FairScheduler {
    pub fn new(limits: FlowLimits) -> Result<Self, FlowError> {
        if limits.max_streams == 0 || limits.max_session_bytes == 0 || limits.max_stream_bytes == 0
        {
            return Err(FlowError::InvalidLimit);
        }
        Ok(Self {
            limits,
            streams: BTreeMap::new(),
            session_bytes: 0,
            cursor: 0,
            order: Vec::new(),
            consecutive_interactive: 0,
        })
    }
    pub fn open(&mut self, id: StreamId, priority: StreamPriority) -> Result<(), FlowError> {
        if self.streams.contains_key(&id) {
            return Ok(());
        }
        if self.streams.len() >= self.limits.max_streams {
            return Err(FlowError::TooManyStreams);
        }
        self.streams.insert(
            id,
            FlowStream {
                priority,
                queue: std::collections::VecDeque::new(),
                queued_bytes: 0,
                max_bytes: self.limits.max_stream_bytes,
            },
        );
        self.order.push(id);
        Ok(())
    }
    pub fn enqueue(&mut self, id: StreamId, data: &[u8]) -> Result<(), FlowError> {
        if data.is_empty() {
            return Err(FlowError::EmptyData);
        }
        let stream = self.streams.get(&id).ok_or(FlowError::StreamNotFound)?;
        let sn = stream
            .queued_bytes
            .checked_add(data.len())
            .ok_or(FlowError::StreamLimit)?;
        let cn = self
            .session_bytes
            .checked_add(data.len())
            .ok_or(FlowError::SessionLimit)?;
        if sn > stream.max_bytes {
            return Err(FlowError::StreamLimit);
        }
        if cn > self.limits.max_session_bytes {
            return Err(FlowError::SessionLimit);
        }
        let stream = self.streams.get_mut(&id).ok_or(FlowError::StreamNotFound)?;
        stream.queue.push_back(data.to_vec());
        stream.queued_bytes = sn;
        self.session_bytes = cn;
        Ok(())
    }
    pub fn next_frame(&mut self) -> Option<(StreamId, Vec<u8>)> {
        if self.order.is_empty() {
            return None;
        }
        let start = self.cursor % self.order.len();
        let find = |priority: Option<StreamPriority>| {
            (0..self.order.len()).find_map(|offset| {
                let index = (start + offset) % self.order.len();
                let stream = self.streams.get(&self.order[index])?;
                (stream.queued_bytes > 0 && priority.is_none_or(|p| p == stream.priority))
                    .then_some(index)
            })
        };
        let preferred = if self.consecutive_interactive < INTERACTIVE_BURST {
            StreamPriority::Interactive
        } else {
            StreamPriority::Bulk
        };
        let index = find(Some(preferred)).or_else(|| find(None))?;
        let id = self.order[index];
        self.cursor = (index + 1) % self.order.len();
        let stream = self.streams.get_mut(&id)?;
        let data = stream.queue.pop_front()?;
        stream.queued_bytes -= data.len();
        self.session_bytes -= data.len();
        if stream.priority == StreamPriority::Interactive {
            self.consecutive_interactive += 1;
        } else {
            self.consecutive_interactive = 0;
        }
        Some((id, data))
    }
    pub fn snapshots(&self) -> impl Iterator<Item = StreamSnapshot> + '_ {
        self.streams.iter().map(|(id, s)| StreamSnapshot {
            id: *id,
            priority: s.priority,
            queued_bytes: s.queued_bytes,
            max_bytes: s.max_bytes,
        })
    }
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct HealthSample {
    pub rtt_us: u64,
    pub loss_per_mille: u16,
    pub pto: u16,
}
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum HealthState {
    Unknown,
    Healthy,
    Degraded,
    Failed,
}
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum HealthFailureCause {
    AuthenticatedDeliveryAckTimeout,
    MeasuredSample,
}
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum HealthObservation {
    Progress,
    Failure(HealthFailureCause),
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct HealthLimits {
    pub degrade_after: u32,
    pub fail_after: u32,
    pub recover_after: u32,
    pub max_paths: usize,
}
impl Default for HealthLimits {
    fn default() -> Self {
        Self {
            degrade_after: 2,
            fail_after: 4,
            recover_after: 2,
            max_paths: 8,
        }
    }
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct HealthRecord {
    pub state: HealthState,
    pub consecutive_bad: u32,
    pub consecutive_good: u32,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum HealthError {
    InvalidLimit,
    ResourceLimit,
}

#[derive(Debug)]
pub struct CarrierHealth {
    limits: HealthLimits,
    records: BTreeMap<PathId, HealthRecord>,
}
impl CarrierHealth {
    pub fn new(limits: HealthLimits) -> Result<Self, HealthError> {
        if limits.degrade_after == 0
            || limits.fail_after < limits.degrade_after
            || limits.recover_after == 0
            || limits.max_paths == 0
        {
            return Err(HealthError::InvalidLimit);
        }
        Ok(Self {
            limits,
            records: BTreeMap::new(),
        })
    }
    pub fn path(&self, path: PathId) -> Option<HealthRecord> {
        self.records.get(&path).copied()
    }
    pub fn observe(
        &mut self,
        path: PathId,
        sample: HealthSample,
    ) -> Result<HealthState, HealthError> {
        let bad = sample.pto >= 3 || sample.loss_per_mille >= 500;
        self.observe_event(
            path,
            if bad {
                HealthObservation::Failure(HealthFailureCause::MeasuredSample)
            } else {
                HealthObservation::Progress
            },
        )
    }
    pub fn observe_event(
        &mut self,
        path: PathId,
        observation: HealthObservation,
    ) -> Result<HealthState, HealthError> {
        if !self.records.contains_key(&path) && self.records.len() >= self.limits.max_paths {
            return Err(HealthError::ResourceLimit);
        }
        let r = self.records.entry(path).or_insert(HealthRecord {
            state: HealthState::Unknown,
            consecutive_bad: 0,
            consecutive_good: 0,
        });
        match observation {
            HealthObservation::Failure(_) => {
                r.consecutive_bad = r.consecutive_bad.saturating_add(1);
                r.consecutive_good = 0;
                if r.consecutive_bad >= self.limits.fail_after {
                    r.state = HealthState::Failed;
                } else if r.consecutive_bad >= self.limits.degrade_after {
                    r.state = HealthState::Degraded;
                }
            }
            HealthObservation::Progress => {
                r.consecutive_good = r.consecutive_good.saturating_add(1);
                r.consecutive_bad = 0;
                match r.state {
                    HealthState::Unknown => r.state = HealthState::Healthy,
                    HealthState::Degraded if r.consecutive_good >= self.limits.recover_after => {
                        r.state = HealthState::Healthy
                    }
                    HealthState::Failed if r.consecutive_good >= self.limits.recover_after => {
                        r.state = HealthState::Degraded;
                        r.consecutive_good = 0;
                    }
                    _ => {}
                }
            }
        }
        Ok(r.state)
    }
}

/// Bounded, local-only health evidence for later workload correlation.
/// Samples and transitions are retained in insertion order up to the same
/// explicit bound, and JSON output is deterministic and timestamp-free.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct HealthEvidenceLimits {
    pub max_samples: usize,
}
impl Default for HealthEvidenceLimits {
    fn default() -> Self {
        Self { max_samples: 128 }
    }
}
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct HealthSampleEvidence {
    pub path: PathId,
    pub sample: HealthSample,
    pub state: HealthState,
}
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct HealthEventEvidence {
    pub path: PathId,
    pub observation: HealthObservation,
    pub state: HealthState,
}
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct HealthTransitionEvidence {
    pub path: PathId,
    pub from: HealthState,
    pub to: HealthState,
}
#[derive(Debug)]
pub struct CarrierHealthEvidence {
    health: CarrierHealth,
    limits: HealthEvidenceLimits,
    samples: Vec<HealthSampleEvidence>,
    events: Vec<HealthEventEvidence>,
    transitions: Vec<HealthTransitionEvidence>,
}
impl CarrierHealthEvidence {
    pub fn new(
        health_limits: HealthLimits,
        evidence_limits: HealthEvidenceLimits,
    ) -> Result<Self, HealthError> {
        if evidence_limits.max_samples == 0 {
            return Err(HealthError::InvalidLimit);
        }
        Ok(Self {
            health: CarrierHealth::new(health_limits)?,
            limits: evidence_limits,
            samples: Vec::new(),
            events: Vec::new(),
            transitions: Vec::new(),
        })
    }
    pub fn observe(
        &mut self,
        path: PathId,
        sample: HealthSample,
    ) -> Result<HealthState, HealthError> {
        let previous = self.health.path(path).map(|record| record.state);
        let state = self.health.observe(path, sample)?;
        if self.samples.len() == self.limits.max_samples {
            self.samples.remove(0);
        }
        self.samples.push(HealthSampleEvidence {
            path,
            sample,
            state,
        });
        if let Some(from) = previous.filter(|from| *from != state) {
            if self.transitions.len() == self.limits.max_samples {
                self.transitions.remove(0);
            }
            self.transitions.push(HealthTransitionEvidence {
                path,
                from,
                to: state,
            });
        }
        Ok(state)
    }
    pub fn observe_event(
        &mut self,
        path: PathId,
        observation: HealthObservation,
    ) -> Result<HealthState, HealthError> {
        let previous = self.health.path(path).map(|record| record.state);
        let state = self.health.observe_event(path, observation)?;
        if self.events.len() == self.limits.max_samples {
            self.events.remove(0);
        }
        self.events.push(HealthEventEvidence {
            path,
            observation,
            state,
        });
        if let Some(from) = previous.filter(|from| *from != state) {
            if self.transitions.len() == self.limits.max_samples {
                self.transitions.remove(0);
            }
            self.transitions.push(HealthTransitionEvidence {
                path,
                from,
                to: state,
            });
        }
        Ok(state)
    }
    pub fn events(&self) -> &[HealthEventEvidence] {
        &self.events
    }
    pub fn samples(&self) -> &[HealthSampleEvidence] {
        &self.samples
    }
    pub fn transitions(&self) -> &[HealthTransitionEvidence] {
        &self.transitions
    }
    pub fn path(&self, path: PathId) -> Option<HealthRecord> {
        self.health.path(path)
    }
    pub fn json(&self) -> String {
        let samples = self.samples.iter().map(|e| format!("{{\"path\":{},\"rtt_us\":{},\"loss_per_mille\":{},\"pto\":{},\"state\":\"{}\"}}", e.path.0, e.sample.rtt_us, e.sample.loss_per_mille, e.sample.pto, health_state_name(e.state))).collect::<Vec<_>>().join(",");
        let transitions = self
            .transitions
            .iter()
            .map(|e| {
                format!(
                    "{{\"path\":{},\"from\":\"{}\",\"to\":\"{}\"}}",
                    e.path.0,
                    health_state_name(e.from),
                    health_state_name(e.to)
                )
            })
            .collect::<Vec<_>>()
            .join(",");
        let events = self
            .events
            .iter()
            .map(|e| {
                let (event, cause) = match e.observation {
                    HealthObservation::Progress => ("progress", None),
                    HealthObservation::Failure(
                        HealthFailureCause::AuthenticatedDeliveryAckTimeout,
                    ) => ("failure", Some("authenticated_delivery_ack_timeout")),
                    HealthObservation::Failure(HealthFailureCause::MeasuredSample) => {
                        ("failure", Some("measured_sample"))
                    }
                };
                match cause {
                    Some(cause) => format!(
                        "{{\"path\":{},\"event\":\"{}\",\"cause\":\"{}\",\"state\":\"{}\"}}",
                        e.path.0,
                        event,
                        cause,
                        health_state_name(e.state)
                    ),
                    None => format!(
                        "{{\"path\":{},\"event\":\"{}\",\"state\":\"{}\"}}",
                        e.path.0,
                        event,
                        health_state_name(e.state)
                    ),
                }
            })
            .collect::<Vec<_>>()
            .join(",");
        format!(
            "{{\"samples\":[{}],\"events\":[{}],\"transitions\":[{}]}}",
            samples, events, transitions
        )
    }
}
fn health_state_name(state: HealthState) -> &'static str {
    match state {
        HealthState::Unknown => "unknown",
        HealthState::Healthy => "healthy",
        HealthState::Degraded => "degraded",
        HealthState::Failed => "failed",
    }
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct CarrierScore {
    pub score: i64,
    pub healthy: bool,
}
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct ManagerLimits {
    pub min_hold_events: u32,
    pub switch_margin: i64,
    pub max_paths: usize,
}
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum CarrierSwitchReason {
    UdpPathDegraded,
}
impl CarrierSwitchReason {
    pub const fn as_str(self) -> &'static str {
        "udp_path_degraded"
    }
}
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct CarrierSwitchDecision {
    pub from: ActiveCarrier,
    pub to: ActiveCarrier,
    pub failed_path: PathId,
    pub active_path: PathId,
    pub generation: PathGeneration,
    pub reason: CarrierSwitchReason,
}
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct PromotionEvidence {
    pub target_path: PathId,
    pub generation: PathGeneration,
    pub authenticated: bool,
    pub resume_validated: bool,
    pub readiness_observations: u32,
}
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct PendingCarrierSwitch {
    pub failed_path: PathId,
    pub target_path: PathId,
    pub generation: PathGeneration,
    pub reason: CarrierSwitchReason,
}

#[derive(Debug)]
pub struct CarrierManager {
    limits: ManagerLimits,
    samples: BTreeMap<PathId, HealthSample>,
    active: Option<PathId>,
    hold: u32,
    pub switches: u64,
    active_generation: Option<PathGeneration>,
    pending_switch: Option<PendingCarrierSwitch>,
    migration_hold: u32,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct MigrationCandidate {
    pub path: PathId,
    pub generation: PathGeneration,
    pub validated: bool,
    pub health: HealthSample,
}
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum MigrationError {
    NotTcp,
    Unvalidated,
    OldGeneration,
    GenerationMismatch,
    Unhealthy,
    ScoreMargin,
    HoldGate,
}
const SCORE_BASE: i64 = 10_000;
const SCORE_RTT_WEIGHT: i64 = 1;
const SCORE_LOSS_WEIGHT: i64 = 5;
const SCORE_PTO_WEIGHT: i64 = 1_000;
const SCORE_MAX_RTT_US: u64 = 10_000;
const HEALTHY_MAX_LOSS_PER_MILLE: u16 = 500;
const HEALTHY_MAX_PTO: u16 = 3;

impl CarrierManager {
    pub fn new(limits: ManagerLimits) -> Result<Self, FlowError> {
        if limits.min_hold_events == 0 || limits.max_paths == 0 {
            return Err(FlowError::InvalidLimit);
        }
        Ok(Self {
            limits,
            samples: BTreeMap::new(),
            active: None,
            hold: 0,
            switches: 0,
            active_generation: None,
            pending_switch: None,
            migration_hold: 0,
        })
    }
    pub fn observe(&mut self, path: PathId, sample: HealthSample) -> Result<(), FlowError> {
        if self.samples.len() >= self.limits.max_paths && !self.samples.contains_key(&path) {
            return Err(FlowError::TooManyStreams);
        }
        self.samples.insert(path, sample);
        Ok(())
    }
    /// Returns the deterministic local score: higher is better. RTT is
    /// capped before conversion; loss and PTO are bounded integer penalties.
    /// A path is eligible only when loss is below 500 per mille and PTO is
    /// below 3.
    pub fn score(sample: HealthSample) -> CarrierScore {
        let score = SCORE_BASE
            - sample.rtt_us.min(SCORE_MAX_RTT_US) as i64 * SCORE_RTT_WEIGHT
            - sample.loss_per_mille as i64 * SCORE_LOSS_WEIGHT
            - sample.pto as i64 * SCORE_PTO_WEIGHT;
        CarrierScore {
            score,
            healthy: sample.pto < HEALTHY_MAX_PTO
                && sample.loss_per_mille < HEALTHY_MAX_LOSS_PER_MILLE,
        }
    }
    pub fn active(&self) -> Option<PathId> {
        self.active
    }
    pub fn set_active_tcp(
        &mut self,
        path: PathId,
        generation: PathGeneration,
    ) -> Result<(), MigrationError> {
        if !self.samples.contains_key(&path) {
            return Err(MigrationError::GenerationMismatch);
        }
        self.active = Some(path);
        self.active_generation = Some(generation);
        self.pending_switch = None;
        self.migration_hold = 0;
        Ok(())
    }

    pub fn set_active_udp(&mut self, path: PathId, generation: PathGeneration) {
        self.active = Some(path);
        self.active_generation = Some(generation);
        self.pending_switch = None;
        self.migration_hold = 0;
    }
    pub fn pending_switch(&self) -> Option<PendingCarrierSwitch> {
        self.pending_switch
    }
    pub fn fail_udp_to_tcp(
        &mut self,
        failed_path: PathId,
        generation: PathGeneration,
        fallback: PathId,
        state: HealthState,
        reason: CarrierSwitchReason,
    ) -> Result<PendingCarrierSwitch, MigrationError> {
        if self.active != Some(failed_path) {
            return Err(MigrationError::NotTcp);
        }
        let current = self
            .active_generation
            .ok_or(MigrationError::GenerationMismatch)?;
        if generation.0 < current.0 {
            return Err(MigrationError::OldGeneration);
        }
        if generation != current || state != HealthState::Failed {
            return Err(MigrationError::GenerationMismatch);
        }
        let pending = PendingCarrierSwitch {
            failed_path,
            target_path: fallback,
            generation: PathGeneration(generation.0.saturating_add(1)),
            reason,
        };
        self.active = None;
        self.pending_switch = Some(pending);
        self.migration_hold = 0;
        Ok(pending)
    }

    pub fn promote_failed_udp_target(
        &mut self,
        evidence: PromotionEvidence,
    ) -> Result<CarrierSwitchDecision, MigrationError> {
        let pending = self
            .pending_switch
            .ok_or(MigrationError::GenerationMismatch)?;
        if evidence.generation.0 < pending.generation.0 {
            return Err(MigrationError::OldGeneration);
        }
        if evidence.target_path != pending.target_path || evidence.generation != pending.generation
        {
            return Err(MigrationError::GenerationMismatch);
        }
        if !evidence.authenticated
            || !evidence.resume_validated
            || evidence.readiness_observations < 3
        {
            return Err(MigrationError::Unvalidated);
        }
        let decision = CarrierSwitchDecision {
            from: ActiveCarrier::Udp,
            to: ActiveCarrier::Tcp,
            failed_path: pending.failed_path,
            active_path: pending.target_path,
            generation: pending.generation,
            reason: pending.reason,
        };
        self.active = Some(pending.target_path);
        self.active_generation = Some(pending.generation);
        self.pending_switch = None;
        self.migration_hold = 0;
        self.switches = self.switches.saturating_add(1);
        Ok(decision)
    }

    /// Propose migration from active TCP back to UDP. Every gate is checked
    /// before mutation; rejected candidates leave active path, hold and metrics
    /// unchanged. `validated` represents explicit path-challenge evidence and
    /// is intentionally separate from health/packet feedback.
    pub fn migrate_back_to_udp(
        &mut self,
        candidate: MigrationCandidate,
    ) -> Result<bool, MigrationError> {
        let active = self.active.ok_or(MigrationError::NotTcp)?;
        let active_generation = self.active_generation.ok_or(MigrationError::NotTcp)?;
        if candidate.path == active {
            return Err(MigrationError::NotTcp);
        }
        if candidate.generation.0 < active_generation.0 {
            return Err(MigrationError::OldGeneration);
        }
        if candidate.generation.0 != active_generation.0 {
            return Err(MigrationError::GenerationMismatch);
        }
        if !candidate.validated {
            return Err(MigrationError::Unvalidated);
        }
        let candidate_score = Self::score(candidate.health);
        if !candidate_score.healthy {
            return Err(MigrationError::Unhealthy);
        }
        let current = self
            .samples
            .get(&active)
            .copied()
            .ok_or(MigrationError::NotTcp)?;
        let current_score = Self::score(current);
        if !current_score.healthy
            || candidate_score.score < current_score.score + self.limits.switch_margin
        {
            return Err(MigrationError::ScoreMargin);
        }
        if self.migration_hold < self.limits.min_hold_events {
            self.migration_hold = self.migration_hold.saturating_add(1);
            return Err(MigrationError::HoldGate);
        }
        self.active = Some(candidate.path);
        self.active_generation = Some(candidate.generation);
        self.migration_hold = 0;
        self.switches = self.switches.saturating_add(1);
        Ok(true)
    }

    pub fn migration_hold(&self) -> u32 {
        self.migration_hold
    }

    pub fn choose(&mut self) -> Option<PathId> {
        let best = self
            .samples
            .iter()
            .filter_map(|(p, s)| {
                let x = Self::score(*s);
                x.healthy.then_some((*p, x.score))
            })
            // BTreeMap gives stable traversal, but max_by_key keeps the
            // later equal item. Compare explicitly so equal scores always
            // prefer the smaller PathId.
            .min_by(|(left_path, left_score), (right_path, right_score)| {
                right_score
                    .cmp(left_score)
                    .then_with(|| left_path.cmp(right_path))
            })
            .map(|(p, _)| p);
        let current_score = self
            .active
            .and_then(|p| self.samples.get(&p).map(|s| Self::score(*s).score));
        if let Some(candidate) = best {
            if self.active == Some(candidate) {
                self.hold = 0
            } else if self.active.is_none()
                || self.hold >= self.limits.min_hold_events
                    && Some(candidate) != self.active
                    && current_score.is_none_or(|x| {
                        Self::score(*self.samples.get(&candidate).unwrap()).score
                            >= x + self.limits.switch_margin
                    })
            {
                self.active = Some(candidate);
                self.hold = 0;
                self.switches = self.switches.saturating_add(1)
            } else {
                self.hold = self.hold.saturating_add(1)
            }
        }
        self.active
    }
}

#[cfg(test)]
mod manager_tests {
    use super::*;

    const GOOD: HealthSample = HealthSample {
        rtt_us: 100,
        loss_per_mille: 0,
        pto: 0,
    };
    const BAD: HealthSample = HealthSample {
        rtt_us: 100,
        loss_per_mille: 500,
        pto: 0,
    };

    #[test]
    fn score_formula_and_health_boundary_are_deterministic() {
        assert_eq!(
            CarrierManager::score(HealthSample {
                rtt_us: 12_000,
                loss_per_mille: 100,
                pto: 2,
            }),
            CarrierScore {
                score: -2_500,
                healthy: true,
            }
        );
        assert!(
            !CarrierManager::score(HealthSample {
                rtt_us: 0,
                loss_per_mille: 500,
                pto: 0,
            })
            .healthy
        );
        assert!(
            !CarrierManager::score(HealthSample {
                rtt_us: 0,
                loss_per_mille: 0,
                pto: 3,
            })
            .healthy
        );
    }

    #[test]
    fn choose_prefers_lowest_path_id_on_equal_score() {
        let mut m = CarrierManager::new(ManagerLimits {
            min_hold_events: 1,
            switch_margin: 0,
            max_paths: 2,
        })
        .unwrap();
        let sample = HealthSample {
            rtt_us: 100,
            loss_per_mille: 2,
            pto: 0,
        };
        m.observe(PathId(20), sample).unwrap();
        m.observe(PathId(10), sample).unwrap();
        assert_eq!(m.choose(), Some(PathId(10)));
        assert_eq!(m.choose(), Some(PathId(10)));
    }

    #[test]
    fn health_transitions_are_bounded_and_staged() {
        let mut h = CarrierHealth::new(HealthLimits {
            degrade_after: 2,
            fail_after: 3,
            recover_after: 2,
            max_paths: 1,
        })
        .unwrap();
        let p = PathId(9);
        assert_eq!(h.observe(p, GOOD), Ok(HealthState::Healthy));
        assert_eq!(h.observe(p, BAD), Ok(HealthState::Healthy));
        assert_eq!(h.observe(p, BAD), Ok(HealthState::Degraded));
        assert_eq!(h.observe(p, BAD), Ok(HealthState::Failed));
        assert_eq!(h.observe(p, GOOD), Ok(HealthState::Failed));
        assert_eq!(h.observe(p, GOOD), Ok(HealthState::Degraded));
        assert_eq!(h.observe(p, GOOD), Ok(HealthState::Degraded));
        assert_eq!(h.observe(p, GOOD), Ok(HealthState::Healthy));
    }

    #[test]
    fn health_limits_and_capacity_are_deterministic() {
        assert!(matches!(
            CarrierHealth::new(HealthLimits {
                degrade_after: 0,
                ..HealthLimits::default()
            }),
            Err(HealthError::InvalidLimit)
        ));
        let mut h = CarrierHealth::new(HealthLimits {
            max_paths: 1,
            ..HealthLimits::default()
        })
        .unwrap();
        h.observe(PathId(1), GOOD).unwrap();
        assert_eq!(h.observe(PathId(2), GOOD), Err(HealthError::ResourceLimit));
    }
    #[test]
    fn bulk_does_not_starve_interactive() {
        let mut s = FairScheduler::new(FlowLimits {
            max_streams: 2,
            max_session_bytes: 100,
            max_stream_bytes: 80,
        })
        .unwrap();
        s.open(StreamId(1), StreamPriority::Bulk).unwrap();
        s.open(StreamId(2), StreamPriority::Interactive).unwrap();
        for _ in 0..5 {
            s.enqueue(StreamId(1), b"bulk").unwrap()
        }
        s.enqueue(StreamId(2), b"ping").unwrap();
        let first = (0..3).filter_map(|_| s.next_frame()).collect::<Vec<_>>();
        assert!(first.iter().any(|(id, _)| *id == StreamId(2)));
    }
    #[test]
    fn interactive_burst_is_bounded_and_order_is_repeatable() {
        fn sequence() -> Vec<StreamId> {
            let mut s = FairScheduler::new(FlowLimits {
                max_streams: 2,
                max_session_bytes: 64,
                max_stream_bytes: 64,
            })
            .unwrap();
            s.open(StreamId(10), StreamPriority::Interactive).unwrap();
            s.open(StreamId(20), StreamPriority::Bulk).unwrap();
            for _ in 0..4 {
                s.enqueue(StreamId(10), b"i").unwrap();
                s.enqueue(StreamId(20), b"b").unwrap();
            }
            (0..8)
                .filter_map(|_| s.next_frame().map(|(id, _)| id))
                .collect()
        }

        let expected = vec![
            StreamId(10),
            StreamId(10),
            StreamId(10),
            StreamId(20),
            StreamId(10),
            StreamId(20),
            StreamId(20),
            StreamId(20),
        ];
        assert_eq!(sequence(), expected);
        assert_eq!(sequence(), expected);
    }

    #[test]
    fn flow_limits_are_atomic() {
        let mut s = FairScheduler::new(FlowLimits {
            max_streams: 1,
            max_session_bytes: 4,
            max_stream_bytes: 4,
        })
        .unwrap();
        s.open(StreamId(1), StreamPriority::Bulk).unwrap();
        assert_eq!(
            s.enqueue(StreamId(1), b"12345"),
            Err(FlowError::StreamLimit)
        );
        assert_eq!(s.snapshots().next().unwrap().queued_bytes, 0);
    }
    #[test]
    fn carrier_scoring_and_hysteresis_bound_oscillation() {
        let mut m = CarrierManager::new(ManagerLimits {
            min_hold_events: 3,
            switch_margin: 10,
            max_paths: 2,
        })
        .unwrap();
        m.observe(
            PathId(1),
            HealthSample {
                rtt_us: 100,
                loss_per_mille: 0,
                pto: 0,
            },
        )
        .unwrap();
        m.observe(
            PathId(2),
            HealthSample {
                rtt_us: 500,
                loss_per_mille: 0,
                pto: 0,
            },
        )
        .unwrap();
        assert_eq!(m.choose(), Some(PathId(1)));
        m.observe(
            PathId(2),
            HealthSample {
                rtt_us: 50,
                loss_per_mille: 0,
                pto: 0,
            },
        )
        .unwrap();
        assert_eq!(m.choose(), Some(PathId(1)));
        assert_eq!(m.switches, 1);
        m.observe(
            PathId(2),
            HealthSample {
                rtt_us: 50,
                loss_per_mille: 0,
                pto: 0,
            },
        )
        .unwrap();
        assert_eq!(m.choose(), Some(PathId(1)));
        m.observe(
            PathId(2),
            HealthSample {
                rtt_us: 50,
                loss_per_mille: 0,
                pto: 0,
            },
        )
        .unwrap();
        m.hold = 3;
        assert_eq!(m.choose(), Some(PathId(2)));
        assert_eq!(m.switches, 2);
    }
    #[test]
    fn changing_best_path_does_not_accumulate_hold_across_candidates() {
        let mut m = CarrierManager::new(ManagerLimits {
            min_hold_events: 3,
            switch_margin: 0,
            max_paths: 3,
        })
        .unwrap();
        for (path, rtt) in [(PathId(1), 100), (PathId(2), 200), (PathId(3), 300)] {
            m.observe(
                path,
                HealthSample {
                    rtt_us: rtt,
                    loss_per_mille: 0,
                    pto: 0,
                },
            )
            .unwrap();
        }
        assert_eq!(m.choose(), Some(PathId(1)));
        for (path, rtt) in [(PathId(2), 10), (PathId(3), 1), (PathId(2), 0)] {
            m.observe(
                path,
                HealthSample {
                    rtt_us: rtt,
                    loss_per_mille: 0,
                    pto: 0,
                },
            )
            .unwrap();
            assert_eq!(m.choose(), Some(PathId(1)));
        }
        assert_eq!(m.switches, 1);
    }

    #[test]
    fn migration_back_requires_validation_generation_health_margin_and_hold() {
        let mut m = CarrierManager::new(ManagerLimits {
            min_hold_events: 2,
            switch_margin: 10,
            max_paths: 3,
        })
        .unwrap();
        m.observe(
            PathId(10),
            HealthSample {
                rtt_us: 100,
                loss_per_mille: 0,
                pto: 0,
            },
        )
        .unwrap();
        m.observe(
            PathId(20),
            HealthSample {
                rtt_us: 20,
                loss_per_mille: 0,
                pto: 0,
            },
        )
        .unwrap();
        m.set_active_tcp(PathId(10), PathGeneration(7)).unwrap();
        let base = MigrationCandidate {
            path: PathId(20),
            generation: PathGeneration(7),
            validated: true,
            health: HealthSample {
                rtt_us: 20,
                loss_per_mille: 0,
                pto: 0,
            },
        };
        assert_eq!(
            m.migrate_back_to_udp(MigrationCandidate {
                validated: false,
                ..base
            }),
            Err(MigrationError::Unvalidated)
        );
        assert_eq!(
            m.migrate_back_to_udp(MigrationCandidate {
                generation: PathGeneration(6),
                ..base
            }),
            Err(MigrationError::OldGeneration)
        );
        assert_eq!(
            m.migrate_back_to_udp(MigrationCandidate {
                generation: PathGeneration(8),
                ..base
            }),
            Err(MigrationError::GenerationMismatch)
        );
        assert_eq!(
            m.migrate_back_to_udp(MigrationCandidate {
                health: HealthSample {
                    rtt_us: 20,
                    loss_per_mille: 600,
                    pto: 0
                },
                ..base
            }),
            Err(MigrationError::Unhealthy)
        );
        assert_eq!(
            m.migrate_back_to_udp(MigrationCandidate {
                health: HealthSample {
                    rtt_us: 95,
                    loss_per_mille: 0,
                    pto: 0
                },
                ..base
            }),
            Err(MigrationError::ScoreMargin)
        );
        assert_eq!(m.active(), Some(PathId(10)));
        assert_eq!(m.migrate_back_to_udp(base), Err(MigrationError::HoldGate));
        assert_eq!(m.migrate_back_to_udp(base), Err(MigrationError::HoldGate));
        assert_eq!(m.migrate_back_to_udp(base), Ok(true));
        assert_eq!(m.active(), Some(PathId(20)));
        assert_eq!(m.switches, 1);
        assert_eq!(m.migrate_back_to_udp(base), Err(MigrationError::NotTcp));
    }
}

#[cfg(test)]
mod health_evidence_tests {
    use super::*;

    #[test]
    fn d064_failure_and_target_scoped_promotion_are_atomic() {
        let limits = HealthLimits {
            degrade_after: 2,
            fail_after: 3,
            recover_after: 2,
            max_paths: 2,
        };
        let mut evidence =
            CarrierHealthEvidence::new(limits, HealthEvidenceLimits { max_samples: 16 }).unwrap();
        let mut manager = CarrierManager::new(ManagerLimits {
            min_hold_events: 1,
            switch_margin: 0,
            max_paths: 2,
        })
        .unwrap();
        let udp = PathId(9);
        let tcp = PathId(10);
        manager.set_active_udp(udp, PathGeneration(1));
        let mut failover = FailoverController::new(3, 2, 32).unwrap();
        let failure =
            HealthObservation::Failure(HealthFailureCause::AuthenticatedDeliveryAckTimeout);
        assert_ne!(
            evidence.observe_event(udp, failure).unwrap(),
            HealthState::Failed
        );
        assert_ne!(
            evidence.observe_event(udp, failure).unwrap(),
            HealthState::Failed
        );
        assert_eq!(manager.active(), Some(udp));
        let failed = evidence.observe_event(udp, failure).unwrap();
        let pending = manager
            .fail_udp_to_tcp(
                udp,
                PathGeneration(1),
                tcp,
                failed,
                CarrierSwitchReason::UdpPathDegraded,
            )
            .unwrap();
        assert_eq!(pending.generation, PathGeneration(2));
        assert_eq!(pending.reason.as_str(), "udp_path_degraded");
        assert_eq!(manager.active(), None);
        assert_eq!(manager.switches, 0);
        assert_eq!(failover.active(), ActiveCarrier::Udp);

        let valid = PromotionEvidence {
            target_path: tcp,
            generation: PathGeneration(2),
            authenticated: true,
            resume_validated: true,
            readiness_observations: 3,
        };
        for (bad, error) in [
            (
                PromotionEvidence {
                    target_path: udp,
                    ..valid
                },
                MigrationError::GenerationMismatch,
            ),
            (
                PromotionEvidence {
                    generation: PathGeneration(1),
                    ..valid
                },
                MigrationError::OldGeneration,
            ),
            (
                PromotionEvidence {
                    generation: PathGeneration(3),
                    ..valid
                },
                MigrationError::GenerationMismatch,
            ),
            (
                PromotionEvidence {
                    authenticated: false,
                    ..valid
                },
                MigrationError::Unvalidated,
            ),
            (
                PromotionEvidence {
                    resume_validated: false,
                    ..valid
                },
                MigrationError::Unvalidated,
            ),
            (
                PromotionEvidence {
                    readiness_observations: 2,
                    ..valid
                },
                MigrationError::Unvalidated,
            ),
        ] {
            assert_eq!(manager.promote_failed_udp_target(bad), Err(error));
            assert_eq!(manager.active(), None);
            assert_eq!(manager.pending_switch(), Some(pending));
            assert_eq!(manager.switches, 0);
        }

        let decision = manager.promote_failed_udp_target(valid).unwrap();
        assert_eq!(decision.active_path, tcp);
        assert_eq!(decision.generation, PathGeneration(2));
        assert_eq!(manager.active(), Some(tcp));
        assert_eq!(manager.pending_switch(), None);
        assert_eq!(manager.switches, 1);
        assert!(failover.apply_manager_decision(&decision));
        assert_eq!(failover.active(), ActiveCarrier::Tcp);
        assert!(!failover.apply_manager_decision(&decision));
        assert_eq!(
            manager.promote_failed_udp_target(valid),
            Err(MigrationError::GenerationMismatch)
        );
        assert_eq!(manager.switches, 1);
    }

    #[test]
    fn progress_resets_d064_failure_counter() {
        let limits = HealthLimits {
            degrade_after: 2,
            fail_after: 3,
            recover_after: 2,
            max_paths: 2,
        };
        let mut evidence =
            CarrierHealthEvidence::new(limits, HealthEvidenceLimits { max_samples: 16 }).unwrap();
        let failure =
            HealthObservation::Failure(HealthFailureCause::AuthenticatedDeliveryAckTimeout);
        assert_ne!(
            evidence.observe_event(PathId(9), failure).unwrap(),
            HealthState::Failed
        );
        assert_ne!(
            evidence.observe_event(PathId(9), failure).unwrap(),
            HealthState::Failed
        );
        evidence
            .observe_event(PathId(9), HealthObservation::Progress)
            .unwrap();
        assert_eq!(evidence.path(PathId(9)).unwrap().consecutive_bad, 0);
        assert_ne!(
            evidence.observe_event(PathId(9), failure).unwrap(),
            HealthState::Failed
        );
        assert_ne!(
            evidence.observe_event(PathId(9), failure).unwrap(),
            HealthState::Failed
        );
        assert_eq!(
            evidence.observe_event(PathId(9), failure).unwrap(),
            HealthState::Failed
        );
        assert_eq!(
            evidence.samples().len(),
            0,
            "events must not fabricate metrics"
        );
        assert_eq!(evidence.events().len(), 6);
    }

    #[test]
    fn evidence_is_bounded_and_json_is_deterministic() {
        let mut evidence = CarrierHealthEvidence::new(
            HealthLimits {
                degrade_after: 2,
                fail_after: 4,
                recover_after: 2,
                max_paths: 2,
            },
            HealthEvidenceLimits { max_samples: 2 },
        )
        .unwrap();
        let good = HealthSample {
            rtt_us: 1200,
            loss_per_mille: 0,
            pto: 0,
        };
        let bad = HealthSample {
            rtt_us: 9000,
            loss_per_mille: 500,
            pto: 3,
        };
        assert_eq!(evidence.observe(PathId(7), good), Ok(HealthState::Healthy));
        assert_eq!(evidence.observe(PathId(7), bad), Ok(HealthState::Healthy));
        assert_eq!(evidence.observe(PathId(7), bad), Ok(HealthState::Degraded));
        assert_eq!(evidence.samples().len(), 2);
        assert_eq!(
            evidence.transitions(),
            &[HealthTransitionEvidence {
                path: PathId(7),
                from: HealthState::Healthy,
                to: HealthState::Degraded
            }]
        );
        assert_eq!(
            evidence.json(),
            "{\"samples\":[{\"path\":7,\"rtt_us\":9000,\"loss_per_mille\":500,\"pto\":3,\"state\":\"healthy\"},{\"path\":7,\"rtt_us\":9000,\"loss_per_mille\":500,\"pto\":3,\"state\":\"degraded\"}],\"events\":[],\"transitions\":[{\"path\":7,\"from\":\"healthy\",\"to\":\"degraded\"}]}"
        );
    }

    #[test]
    fn evidence_rejects_zero_bound_without_mutation() {
        assert_eq!(
            CarrierHealthEvidence::new(
                HealthLimits::default(),
                HealthEvidenceLimits { max_samples: 0 }
            )
            .unwrap_err(),
            HealthError::InvalidLimit
        );
    }
}

/// D064's single-active, multi-ready runtime state. `Failed` is terminal for a
/// path generation; registering a greater generation creates a fresh standby.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum ConcurrentPathState {
    Standby,
    Warm,
    Active,
    Draining,
    Failed,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, PartialOrd, Ord)]
pub struct ConcurrentPathKey {
    pub path: PathId,
    pub generation: PathGeneration,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum SwitchReason {
    UdpBlackhole,
    UdpPathDegraded,
    TcpReadyPreferred,
    AddressChange,
    OperatorRequest,
    DrainDeadline,
    ResumeRejected,
    CarrierError,
    Shutdown,
    Other(u16),
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum RecoveryClass {
    Warm,
    Cold,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct ConcurrentSwitchEvent {
    pub from: Option<ConcurrentPathKey>,
    pub to: Option<ConcurrentPathKey>,
    pub active_epoch: u64,
    pub reason: SwitchReason,
    pub recovery_class: Option<RecoveryClass>,
    pub decided_at_ms: u64,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct ConcurrentLimits {
    pub k_ready: u32,
    pub min_active_dwell_ms: u64,
    pub voluntary_cooldown_ms: u64,
    pub drain_timeout_ms: u64,
    pub max_paths: usize,
    pub max_uncertain_ranges: usize,
    pub max_uncertain_bytes: usize,
    pub max_switch_events: usize,
}

impl Default for ConcurrentLimits {
    fn default() -> Self {
        Self {
            k_ready: 3,
            min_active_dwell_ms: 5_000,
            voluntary_cooldown_ms: 10_000,
            drain_timeout_ms: 3_000,
            max_paths: 4,
            max_uncertain_ranges: 128,
            max_uncertain_bytes: 1 << 20,
            max_switch_events: 128,
        }
    }
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum ConcurrentError {
    InvalidLimit,
    Capacity,
    UnknownPath,
    OldGeneration,
    GenerationMismatch,
    IllegalState,
    NotReady,
    Dwell,
    Cooldown,
    NoActive,
    Conflict,
    NotFound,
    DrainPending,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, PartialOrd, Ord)]
pub struct LogicalRangeId {
    pub stream: u64,
    pub offset: u64,
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct ReplayRange {
    pub id: LogicalRangeId,
    pub bytes: Vec<u8>,
}

#[derive(Debug, Clone, Copy)]
struct ConcurrentPathRecord {
    key: ConcurrentPathKey,
    kind: CarrierKind,
    state: ConcurrentPathState,
    ready: u32,
    warm_at_ms: Option<u64>,
    drain_deadline_ms: Option<u64>,
}

#[derive(Debug, Clone)]
struct AssignedRange {
    bytes: Vec<u8>,
    owner: ConcurrentPathKey,
    uncertain: bool,
}

/// A deterministic clock-injected policy core. It owns path selection only;
/// Session delivery remains explicit through `confirm` and stable range IDs.
#[derive(Debug)]
pub struct ConcurrentCarrierManager {
    limits: ConcurrentLimits,
    paths: BTreeMap<PathId, ConcurrentPathRecord>,
    active: Option<ConcurrentPathKey>,
    active_since_ms: Option<u64>,
    last_switch_ms: Option<u64>,
    last_failure_ms: Option<u64>,
    active_epoch: u64,
    ranges: BTreeMap<LogicalRangeId, AssignedRange>,
    retained_bytes: usize,
    events: Vec<ConcurrentSwitchEvent>,
}

impl ConcurrentCarrierManager {
    pub fn new(limits: ConcurrentLimits) -> Result<Self, ConcurrentError> {
        if limits.k_ready == 0
            || limits.drain_timeout_ms == 0
            || limits.max_paths == 0
            || limits.max_uncertain_ranges == 0
            || limits.max_uncertain_bytes == 0
            || limits.max_switch_events == 0
        {
            return Err(ConcurrentError::InvalidLimit);
        }
        Ok(Self {
            limits,
            paths: BTreeMap::new(),
            active: None,
            active_since_ms: None,
            last_switch_ms: None,
            last_failure_ms: None,
            active_epoch: 0,
            ranges: BTreeMap::new(),
            retained_bytes: 0,
            events: Vec::new(),
        })
    }

    pub fn register(
        &mut self,
        key: ConcurrentPathKey,
        kind: CarrierKind,
    ) -> Result<(), ConcurrentError> {
        if let Some(old) = self.paths.get(&key.path) {
            if key.generation.0 < old.key.generation.0 {
                return Err(ConcurrentError::OldGeneration);
            }
            if key.generation == old.key.generation {
                return Err(ConcurrentError::GenerationMismatch);
            }
            if old.state != ConcurrentPathState::Failed {
                return Err(ConcurrentError::IllegalState);
            }
        } else if self.paths.len() == self.limits.max_paths {
            return Err(ConcurrentError::Capacity);
        }
        self.paths.insert(
            key.path,
            ConcurrentPathRecord {
                key,
                kind,
                state: ConcurrentPathState::Standby,
                ready: 0,
                warm_at_ms: None,
                drain_deadline_ms: None,
            },
        );
        Ok(())
    }

    pub fn state(&self, key: ConcurrentPathKey) -> Result<ConcurrentPathState, ConcurrentError> {
        Ok(self.path(key)?.state)
    }

    pub fn active(&self) -> Option<ConcurrentPathKey> {
        self.active
    }

    pub fn active_epoch(&self) -> u64 {
        self.active_epoch
    }

    pub fn events(&self) -> &[ConcurrentSwitchEvent] {
        &self.events
    }

    /// Readiness is authenticated evidence, deliberately separate from packet
    /// feedback. Failure resets the bounded consecutive-success counter.
    pub fn observe_readiness(
        &mut self,
        key: ConcurrentPathKey,
        authenticated: bool,
        admitted: bool,
        now_ms: u64,
    ) -> Result<ConcurrentPathState, ConcurrentError> {
        let k_ready = self.limits.k_ready;
        let path = self.path_mut(key)?;
        if path.state == ConcurrentPathState::Failed
            || path.state == ConcurrentPathState::Draining
            || path.state == ConcurrentPathState::Active
        {
            return Err(ConcurrentError::IllegalState);
        }
        if !authenticated || !admitted {
            path.ready = 0;
            return Ok(path.state);
        }
        path.ready = path.ready.saturating_add(1).min(k_ready);
        if path.ready == k_ready {
            path.state = ConcurrentPathState::Warm;
            path.warm_at_ms.get_or_insert(now_ms);
        }
        Ok(path.state)
    }

    pub fn activate(
        &mut self,
        to: ConcurrentPathKey,
        reason: SwitchReason,
        now_ms: u64,
        hard: bool,
    ) -> Result<ConcurrentSwitchEvent, ConcurrentError> {
        let target = *self.path(to)?;
        if target.state != ConcurrentPathState::Warm {
            return Err(ConcurrentError::NotReady);
        }
        let from = self.active;
        if let Some(active_since) = self.active_since_ms
            && !hard
            && now_ms.saturating_sub(active_since) < self.limits.min_active_dwell_ms
        {
            return Err(ConcurrentError::Dwell);
        }
        if let Some(last_switch) = self.last_switch_ms
            && !hard
            && now_ms.saturating_sub(last_switch) < self.limits.voluntary_cooldown_ms
        {
            return Err(ConcurrentError::Cooldown);
        }
        if let Some(old) = from {
            self.ensure_uncertain_capacity(old)?;
        }

        if let Some(old) = from {
            self.mark_owner_uncertain(old);
            let timeout = self.limits.drain_timeout_ms;
            let old_path = self.path_mut(old)?;
            old_path.state = ConcurrentPathState::Draining;
            old_path.drain_deadline_ms = Some(now_ms.saturating_add(timeout));
        }
        self.path_mut(to)?.state = ConcurrentPathState::Active;
        self.active = Some(to);
        self.active_since_ms = Some(now_ms);
        self.last_switch_ms = Some(now_ms);
        self.active_epoch = self.active_epoch.saturating_add(1);
        let recovery_class = self.last_failure_ms.map(|failure| {
            if target.warm_at_ms.is_some_and(|warm| warm <= failure) {
                RecoveryClass::Warm
            } else {
                RecoveryClass::Cold
            }
        });
        let event = ConcurrentSwitchEvent {
            from,
            to: Some(to),
            active_epoch: self.active_epoch,
            reason,
            recovery_class,
            decided_at_ms: now_ms,
        };
        self.record_event(event);
        Ok(event)
    }

    /// Hard failure removes ownership immediately. Unproved assigned ranges are
    /// retained as uncertain; capacity rejection is atomic and fail-closed.
    pub fn fail(
        &mut self,
        key: ConcurrentPathKey,
        reason: SwitchReason,
        now_ms: u64,
    ) -> Result<Option<ConcurrentSwitchEvent>, ConcurrentError> {
        self.path(key)?;
        let was_active = self.active == Some(key);
        if was_active {
            self.ensure_uncertain_capacity(key)?;
            self.mark_owner_uncertain(key);
            self.active = None;
            self.active_since_ms = None;
            self.last_failure_ms = Some(now_ms);
        }
        self.path_mut(key)?.state = ConcurrentPathState::Failed;
        if !was_active {
            return Ok(None);
        }
        let event = ConcurrentSwitchEvent {
            from: Some(key),
            to: None,
            active_epoch: self.active_epoch,
            reason,
            recovery_class: None,
            decided_at_ms: now_ms,
        };
        self.record_event(event);
        Ok(Some(event))
    }

    pub fn assign(&mut self, id: LogicalRangeId, bytes: &[u8]) -> Result<(), ConcurrentError> {
        let owner = self.active.ok_or(ConcurrentError::NoActive)?;
        if let Some(existing) = self.ranges.get(&id) {
            return if existing.bytes == bytes {
                Ok(())
            } else {
                Err(ConcurrentError::Conflict)
            };
        }
        let total = self
            .retained_bytes
            .checked_add(bytes.len())
            .ok_or(ConcurrentError::Capacity)?;
        if self.ranges.len() == self.limits.max_uncertain_ranges
            || total > self.limits.max_uncertain_bytes
        {
            return Err(ConcurrentError::Capacity);
        }
        self.ranges.insert(
            id,
            AssignedRange {
                bytes: bytes.to_vec(),
                owner,
                uncertain: false,
            },
        );
        self.retained_bytes = total;
        Ok(())
    }

    pub fn confirm(&mut self, id: LogicalRangeId) -> Result<(), ConcurrentError> {
        let range = self.ranges.remove(&id).ok_or(ConcurrentError::NotFound)?;
        self.retained_bytes = self.retained_bytes.saturating_sub(range.bytes.len());
        Ok(())
    }

    pub fn uncertain_ranges(&self) -> usize {
        self.ranges.values().filter(|range| range.uncertain).count()
    }

    /// Reassign uncertain ranges left by a hard-failed generation to the new
    /// sole active owner. Confirmation is still required before retention ends.
    pub fn replay_uncertain(&mut self) -> Result<Vec<ReplayRange>, ConcurrentError> {
        let active = self.active.ok_or(ConcurrentError::NoActive)?;
        let mut replay = Vec::new();
        for (id, range) in &mut self.ranges {
            if range.uncertain {
                replay.push(ReplayRange {
                    id: *id,
                    bytes: range.bytes.clone(),
                });
                range.owner = active;
                range.uncertain = false;
            }
        }
        Ok(replay)
    }

    /// At deadline, replay every still-unconfirmed old-owner range on the sole
    /// active path. Returned bytes keep stable stream/offset identities.
    pub fn finish_drain(
        &mut self,
        old: ConcurrentPathKey,
        now_ms: u64,
    ) -> Result<Vec<ReplayRange>, ConcurrentError> {
        let deadline = {
            let path = self.path(old)?;
            if path.state != ConcurrentPathState::Draining {
                return Err(ConcurrentError::IllegalState);
            }
            path.drain_deadline_ms
                .ok_or(ConcurrentError::DrainPending)?
        };
        if now_ms < deadline {
            return Err(ConcurrentError::DrainPending);
        }
        let active = self.active.ok_or(ConcurrentError::NoActive)?;
        let mut replay = Vec::new();
        for (id, range) in &mut self.ranges {
            if range.owner == old && range.uncertain {
                replay.push(ReplayRange {
                    id: *id,
                    bytes: range.bytes.clone(),
                });
                range.owner = active;
                range.uncertain = false;
            }
        }
        self.path_mut(old)?.state = ConcurrentPathState::Failed;
        Ok(replay)
    }

    fn record_event(&mut self, event: ConcurrentSwitchEvent) {
        if self.events.len() == self.limits.max_switch_events {
            self.events.remove(0);
        }
        self.events.push(event);
    }

    fn ensure_uncertain_capacity(&self, owner: ConcurrentPathKey) -> Result<(), ConcurrentError> {
        let count = self
            .ranges
            .values()
            .filter(|range| range.owner == owner)
            .count();
        let bytes = self
            .ranges
            .values()
            .filter(|range| range.owner == owner)
            .try_fold(0usize, |sum, range| sum.checked_add(range.bytes.len()))
            .ok_or(ConcurrentError::Capacity)?;
        if count > self.limits.max_uncertain_ranges || bytes > self.limits.max_uncertain_bytes {
            Err(ConcurrentError::Capacity)
        } else {
            Ok(())
        }
    }

    fn mark_owner_uncertain(&mut self, owner: ConcurrentPathKey) {
        for range in self
            .ranges
            .values_mut()
            .filter(|range| range.owner == owner)
        {
            range.uncertain = true;
        }
    }

    fn path(&self, key: ConcurrentPathKey) -> Result<&ConcurrentPathRecord, ConcurrentError> {
        let path = self
            .paths
            .get(&key.path)
            .ok_or(ConcurrentError::UnknownPath)?;
        if key.generation.0 < path.key.generation.0 {
            Err(ConcurrentError::OldGeneration)
        } else if key.generation != path.key.generation {
            Err(ConcurrentError::GenerationMismatch)
        } else {
            Ok(path)
        }
    }

    fn path_mut(
        &mut self,
        key: ConcurrentPathKey,
    ) -> Result<&mut ConcurrentPathRecord, ConcurrentError> {
        let path = self
            .paths
            .get_mut(&key.path)
            .ok_or(ConcurrentError::UnknownPath)?;
        if key.generation.0 < path.key.generation.0 {
            Err(ConcurrentError::OldGeneration)
        } else if key.generation != path.key.generation {
            Err(ConcurrentError::GenerationMismatch)
        } else {
            Ok(path)
        }
    }

    pub fn kind(&self, key: ConcurrentPathKey) -> Result<CarrierKind, ConcurrentError> {
        Ok(self.path(key)?.kind)
    }
}

#[cfg(test)]
mod concurrent_manager_tests {
    use super::*;

    const UDP: ConcurrentPathKey = ConcurrentPathKey {
        path: PathId(10),
        generation: PathGeneration(1),
    };
    const TCP: ConcurrentPathKey = ConcurrentPathKey {
        path: PathId(20),
        generation: PathGeneration(1),
    };
    const R: LogicalRangeId = LogicalRangeId {
        stream: 7,
        offset: 0,
    };

    fn manager() -> ConcurrentCarrierManager {
        ConcurrentCarrierManager::new(ConcurrentLimits {
            k_ready: 2,
            min_active_dwell_ms: 5,
            voluntary_cooldown_ms: 10,
            drain_timeout_ms: 3,
            max_paths: 3,
            max_uncertain_ranges: 2,
            max_uncertain_bytes: 8,
            max_switch_events: 8,
        })
        .unwrap()
    }

    fn warm(m: &mut ConcurrentCarrierManager, key: ConcurrentPathKey, now: u64) {
        assert_eq!(
            m.observe_readiness(key, true, true, now).unwrap(),
            ConcurrentPathState::Standby
        );
        assert_eq!(
            m.observe_readiness(key, true, true, now + 1).unwrap(),
            ConcurrentPathState::Warm
        );
    }

    #[test]
    fn readiness_is_bounded_consecutive_and_generation_scoped() {
        let mut m = manager();
        m.register(UDP, CarrierKind::Udp).unwrap();
        m.observe_readiness(UDP, true, true, 0).unwrap();
        assert_eq!(
            m.observe_readiness(UDP, false, true, 1).unwrap(),
            ConcurrentPathState::Standby
        );
        warm(&mut m, UDP, 2);
        assert_eq!(m.kind(UDP), Ok(CarrierKind::Udp));
        assert_eq!(
            m.observe_readiness(
                ConcurrentPathKey {
                    generation: PathGeneration(0),
                    ..UDP
                },
                true,
                true,
                5
            ),
            Err(ConcurrentError::OldGeneration)
        );
    }

    #[test]
    fn warm_switch_is_reason_coded_dwell_guarded_and_drains_uncertain() {
        let mut m = manager();
        m.register(UDP, CarrierKind::Udp).unwrap();
        m.register(TCP, CarrierKind::Tcp).unwrap();
        warm(&mut m, UDP, 0);
        m.activate(UDP, SwitchReason::OperatorRequest, 2, false)
            .unwrap();
        m.assign(R, b"cat").unwrap();
        warm(&mut m, TCP, 3);
        assert_eq!(
            m.activate(TCP, SwitchReason::TcpReadyPreferred, 6, false),
            Err(ConcurrentError::Dwell)
        );
        let event = m
            .activate(TCP, SwitchReason::UdpBlackhole, 7, true)
            .unwrap();
        assert_eq!(event.from, Some(UDP));
        assert_eq!(event.to, Some(TCP));
        assert_eq!(event.reason, SwitchReason::UdpBlackhole);
        assert_eq!(m.state(UDP), Ok(ConcurrentPathState::Draining));
        assert_eq!(m.uncertain_ranges(), 1);
        assert_eq!(m.finish_drain(UDP, 9), Err(ConcurrentError::DrainPending));
        assert_eq!(
            m.finish_drain(UDP, 10).unwrap(),
            vec![ReplayRange {
                id: R,
                bytes: b"cat".to_vec()
            }]
        );
        assert_eq!(m.state(UDP), Ok(ConcurrentPathState::Failed));
        assert_eq!(m.uncertain_ranges(), 0);
        assert_eq!(m.assign(R, b"dog"), Err(ConcurrentError::Conflict));
        m.confirm(R).unwrap();
    }

    #[test]
    fn warm_and_cold_recovery_are_classified_by_failure_order() {
        let mut warm_case = manager();
        warm_case.register(UDP, CarrierKind::Udp).unwrap();
        warm_case.register(TCP, CarrierKind::Tcp).unwrap();
        warm(&mut warm_case, UDP, 0);
        warm_case
            .activate(UDP, SwitchReason::OperatorRequest, 2, false)
            .unwrap();
        warm(&mut warm_case, TCP, 3);
        warm_case.assign(R, b"cat").unwrap();
        warm_case.fail(UDP, SwitchReason::CarrierError, 8).unwrap();
        assert_eq!(warm_case.uncertain_ranges(), 1);
        let event = warm_case
            .activate(TCP, SwitchReason::UdpBlackhole, 9, true)
            .unwrap();
        assert_eq!(event.recovery_class, Some(RecoveryClass::Warm));
        assert_eq!(
            warm_case.replay_uncertain().unwrap(),
            vec![ReplayRange {
                id: R,
                bytes: b"cat".to_vec()
            }]
        );

        let mut cold_case = manager();
        cold_case.register(UDP, CarrierKind::Udp).unwrap();
        cold_case.register(TCP, CarrierKind::Tcp).unwrap();
        warm(&mut cold_case, UDP, 0);
        cold_case
            .activate(UDP, SwitchReason::OperatorRequest, 2, false)
            .unwrap();
        cold_case.fail(UDP, SwitchReason::CarrierError, 8).unwrap();
        warm(&mut cold_case, TCP, 9);
        let event = cold_case
            .activate(TCP, SwitchReason::UdpBlackhole, 11, true)
            .unwrap();
        assert_eq!(event.recovery_class, Some(RecoveryClass::Cold));
    }

    #[test]
    fn both_fail_then_new_generation_recovers_in_readiness_order() {
        let mut m = manager();
        m.register(UDP, CarrierKind::Udp).unwrap();
        m.register(TCP, CarrierKind::Tcp).unwrap();
        warm(&mut m, UDP, 0);
        m.activate(UDP, SwitchReason::OperatorRequest, 2, false)
            .unwrap();
        m.fail(TCP, SwitchReason::CarrierError, 3).unwrap();
        m.fail(UDP, SwitchReason::CarrierError, 4).unwrap();
        assert_eq!(m.active(), None);
        let tcp2 = ConcurrentPathKey {
            generation: PathGeneration(2),
            ..TCP
        };
        let udp2 = ConcurrentPathKey {
            generation: PathGeneration(2),
            ..UDP
        };
        m.register(tcp2, CarrierKind::Tcp).unwrap();
        m.register(udp2, CarrierKind::Udp).unwrap();
        m.observe_readiness(udp2, true, true, 5).unwrap();
        warm(&mut m, tcp2, 5);
        m.activate(tcp2, SwitchReason::CarrierError, 7, true)
            .unwrap();
        assert_eq!(m.active(), Some(tcp2));
        assert_eq!(m.state(udp2), Ok(ConcurrentPathState::Standby));
        assert_eq!(
            m.observe_readiness(TCP, true, true, 8),
            Err(ConcurrentError::OldGeneration)
        );
    }

    #[test]
    fn cooldown_and_uncertain_capacity_rejections_are_atomic() {
        let mut m = manager();
        m.register(UDP, CarrierKind::Udp).unwrap();
        m.register(TCP, CarrierKind::Tcp).unwrap();
        warm(&mut m, UDP, 0);
        warm(&mut m, TCP, 0);
        m.activate(UDP, SwitchReason::OperatorRequest, 2, false)
            .unwrap();
        m.assign(R, b"12345678").unwrap();
        assert_eq!(
            m.assign(
                LogicalRangeId {
                    stream: 7,
                    offset: 8
                },
                b"x"
            ),
            Err(ConcurrentError::Capacity)
        );
        assert_eq!(m.active(), Some(UDP));
        m.activate(TCP, SwitchReason::CarrierError, 7, true)
            .unwrap();
        // UDP is draining rather than eligible for an immediate reverse flap.
        assert_eq!(
            m.activate(UDP, SwitchReason::TcpReadyPreferred, 8, false),
            Err(ConcurrentError::NotReady)
        );
        assert_eq!(m.active(), Some(TCP));

        let third = ConcurrentPathKey {
            path: PathId(30),
            generation: PathGeneration(1),
        };
        m.register(third, CarrierKind::Tcp).unwrap();
        warm(&mut m, third, 8);
        assert_eq!(
            m.activate(third, SwitchReason::OperatorRequest, 12, false),
            Err(ConcurrentError::Cooldown)
        );
        assert_eq!(m.active(), Some(TCP));
        assert_eq!(m.state(third), Ok(ConcurrentPathState::Warm));
    }
}
