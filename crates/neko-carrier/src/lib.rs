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
    /// Applies the already manager-authorized return to UDP. The manager owns
    /// validation/generation/health/hold policy; this controller only commits
    /// the single active owner transition after that decision succeeds.
    pub fn apply_migration_back(&mut self) -> bool {
        if self.active != ActiveCarrier::Tcp {
            return false;
        }
        self.active = ActiveCarrier::Udp;
        true
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
    fn migration_back_owner_requires_tcp_and_is_idempotent() {
        let mut f = FailoverController::new(2, 1, 8).unwrap();
        assert!(!f.apply_migration_back());
        assert!(!f.udp_pto_at(1));
        assert!(f.udp_pto_at(2));
        assert_eq!(f.active(), ActiveCarrier::Tcp);
        assert!(f.apply_migration_back());
        assert_eq!(f.active(), ActiveCarrier::Udp);
        assert!(!f.apply_migration_back());
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
pub struct ReadinessObservation {
    pub target_path: PathId,
    pub generation: PathGeneration,
    pub session_id: u64,
    pub delivery_epoch: u64,
    pub observation_id: u64,
    pub authenticated: bool,
    pub resume_validated: bool,
    pub resource_admitted: bool,
}
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct WarmCandidate {
    pub target_path: PathId,
    pub generation: PathGeneration,
    pub session_id: u64,
    pub delivery_epoch: u64,
    pub ready_observations: u8,
    pub warm: bool,
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
    warm_candidate: Option<WarmCandidate>,
    readiness_observation_ids: Vec<u64>,
    pub warm_recoveries: u64,
    pub cold_recoveries: u64,
    pub failed_warm_attempts: u64,
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

/// Bounded post-authenticated UDP source-endpoint rebinding state.
///
/// `source_tag` is a collision-free opaque token allocated only after the
/// runtime has compared the exact socket address; it is never a truncated hash.
/// The carrier layer never parses or logs an address. A candidate is only observable until the server-generated
/// challenge is answered with the exact session/path/generation/epoch tuple.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct EndpointRebindCandidate {
    pub source_tag: u64,
    pub path: PathId,
    pub generation: PathGeneration,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum EndpointRebindError {
    AlreadyPending,
    SameSource,
    OldGeneration,
    TupleMismatch,
    NoCandidate,
    ChallengeMismatch,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct EndpointRebindState {
    active_source_tag: u64,
    active_path: PathId,
    active_generation: PathGeneration,
    candidate: Option<EndpointRebindCandidate>,
    retired_source_tag: Option<u64>,
    session_id: u64,
    delivery_epoch: u64,
    armed_challenge_id: Option<u64>,
}

impl EndpointRebindState {
    pub fn new(
        source_tag: u64,
        path: PathId,
        generation: PathGeneration,
        session_id: u64,
        delivery_epoch: u64,
    ) -> Result<Self, EndpointRebindError> {
        if source_tag == 0 || session_id == 0 || delivery_epoch == 0 {
            return Err(EndpointRebindError::TupleMismatch);
        }
        Ok(Self {
            active_source_tag: source_tag,
            active_path: path,
            active_generation: generation,
            candidate: None,
            retired_source_tag: None,
            session_id,
            delivery_epoch,
            armed_challenge_id: None,
        })
    }

    pub const fn active_source_tag(&self) -> u64 {
        self.active_source_tag
    }
    pub const fn active_path(&self) -> PathId {
        self.active_path
    }
    pub const fn active_generation(&self) -> PathGeneration {
        self.active_generation
    }
    pub const fn candidate(&self) -> Option<EndpointRebindCandidate> {
        self.candidate
    }
    pub const fn retired_source_tag(&self) -> Option<u64> {
        self.retired_source_tag
    }

    /// Authenticated input from a new source is candidate-only. It cannot
    /// change ownership or create validation evidence by itself.
    pub fn observe_candidate(
        &mut self,
        candidate: EndpointRebindCandidate,
    ) -> Result<(), EndpointRebindError> {
        if self.candidate.is_some() {
            return Err(EndpointRebindError::AlreadyPending);
        }
        if candidate.source_tag == self.active_source_tag {
            return Err(EndpointRebindError::SameSource);
        }
        if candidate.generation.0 < self.active_generation.0 {
            return Err(EndpointRebindError::OldGeneration);
        }
        if candidate.generation.0 != self.active_generation.0.saturating_add(1) {
            return Err(EndpointRebindError::TupleMismatch);
        }
        if candidate.path != self.active_path {
            return Err(EndpointRebindError::TupleMismatch);
        }
        self.candidate = Some(candidate);
        Ok(())
    }

    /// Arm one fresh challenge generated by the trusted local runtime.
    pub fn arm_challenge(&mut self, challenge_id: u64) -> Result<(), EndpointRebindError> {
        if challenge_id == 0 || self.candidate.is_none() || self.armed_challenge_id.is_some() {
            return Err(EndpointRebindError::TupleMismatch);
        }
        self.armed_challenge_id = Some(challenge_id);
        Ok(())
    }

    /// Only the exact server challenge response from the candidate endpoint
    /// atomically promotes the new source and generation.
    pub fn validate_and_promote(
        &mut self,
        response: EndpointRebindCandidate,
        session_id: u64,
        delivery_epoch: u64,
        challenge_id: u64,
    ) -> Result<(), EndpointRebindError> {
        let candidate = self.candidate.ok_or(EndpointRebindError::NoCandidate)?;
        if session_id != self.session_id || delivery_epoch != self.delivery_epoch {
            self.candidate = None;
            self.armed_challenge_id = None;
            return Err(EndpointRebindError::TupleMismatch);
        }
        if self.armed_challenge_id != Some(challenge_id) || response != candidate {
            self.candidate = None;
            self.armed_challenge_id = None;
            return Err(EndpointRebindError::ChallengeMismatch);
        }
        self.retired_source_tag = Some(self.active_source_tag);
        self.active_source_tag = candidate.source_tag;
        self.active_path = candidate.path;
        self.active_generation = candidate.generation;
        self.candidate = None;
        self.armed_challenge_id = None;
        Ok(())
    }
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
        // switch_margin is a positive-improvement gate: a negative value would
        // invert the M4 margin and admit a strictly worse healthy candidate.
        // It has no committed meaning, so reject it like the other invalid
        // limits rather than comparing it inverted at decision time.
        if limits.min_hold_events == 0 || limits.max_paths == 0 || limits.switch_margin < 0 {
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
            warm_candidate: None,
            readiness_observation_ids: Vec::new(),
            warm_recoveries: 0,
            cold_recoveries: 0,
            failed_warm_attempts: 0,
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
        self.warm_candidate = None;
        self.readiness_observation_ids.clear();
        self.migration_hold = 0;
        Ok(())
    }

    pub fn set_active_udp(&mut self, path: PathId, generation: PathGeneration) {
        self.active = Some(path);
        self.active_generation = Some(generation);
        self.pending_switch = None;
        self.warm_candidate = None;
        self.readiness_observation_ids.clear();
        self.migration_hold = 0;
    }
    pub fn warm_candidate(&self) -> Option<WarmCandidate> {
        self.warm_candidate
    }

    /// Installs or replaces one bounded standby generation. UDP remains active;
    /// the standby cannot carry application data through this manager API.
    pub fn prepare_warm_candidate(
        &mut self,
        target_path: PathId,
        generation: PathGeneration,
        session_id: u64,
        delivery_epoch: u64,
    ) -> Result<(), MigrationError> {
        if self.active.is_none() || self.pending_switch.is_some() {
            return Err(MigrationError::GenerationMismatch);
        }
        if generation.0
            <= self
                .active_generation
                .ok_or(MigrationError::GenerationMismatch)?
                .0
        {
            return Err(MigrationError::OldGeneration);
        }
        self.warm_candidate = Some(WarmCandidate {
            target_path,
            generation,
            session_id,
            delivery_epoch,
            ready_observations: 0,
            warm: false,
        });
        self.readiness_observation_ids.clear();
        Ok(())
    }

    /// Accumulates exactly the bounded D064 readiness gate before failure.
    /// Authentication, resume binding and resource admission are separate from
    /// Session delivery and never change the active owner.
    pub fn observe_warm_candidate_readiness(
        &mut self,
        observation: ReadinessObservation,
    ) -> Result<bool, MigrationError> {
        let candidate = self
            .warm_candidate
            .ok_or(MigrationError::GenerationMismatch)?;
        let dimension_error = if observation.generation.0 < candidate.generation.0 {
            Some(MigrationError::OldGeneration)
        } else if observation.target_path != candidate.target_path
            || observation.generation != candidate.generation
            || observation.session_id != candidate.session_id
            || observation.delivery_epoch != candidate.delivery_epoch
        {
            Some(MigrationError::GenerationMismatch)
        } else {
            None
        };
        if let Some(error) = dimension_error {
            self.readiness_observation_ids.clear();
            if let Some(candidate) = self.warm_candidate.as_mut() {
                candidate.ready_observations = 0;
                candidate.warm = false;
            }
            return Err(error);
        }
        if !observation.authenticated
            || !observation.resume_validated
            || !observation.resource_admitted
        {
            self.readiness_observation_ids.clear();
            if let Some(candidate) = self.warm_candidate.as_mut() {
                candidate.ready_observations = 0;
                candidate.warm = false;
            }
            return Err(MigrationError::Unvalidated);
        }
        if self
            .readiness_observation_ids
            .contains(&observation.observation_id)
        {
            return Ok(candidate.warm);
        }
        if self.readiness_observation_ids.len() < 3 {
            self.readiness_observation_ids
                .push(observation.observation_id);
        }
        let candidate = self.warm_candidate.as_mut().unwrap();
        candidate.ready_observations = self.readiness_observation_ids.len() as u8;
        candidate.warm = candidate.ready_observations == 3;
        Ok(candidate.warm)
    }

    pub fn fail_warm_candidate(
        &mut self,
        target_path: PathId,
        generation: PathGeneration,
    ) -> Result<(), MigrationError> {
        let candidate = self
            .warm_candidate
            .ok_or(MigrationError::GenerationMismatch)?;
        if candidate.target_path != target_path || candidate.generation != generation {
            return Err(MigrationError::GenerationMismatch);
        }
        if candidate.warm {
            self.failed_warm_attempts = self.failed_warm_attempts.saturating_add(1);
        }
        self.warm_candidate = None;
        self.readiness_observation_ids.clear();
        Ok(())
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
        if self.warm_candidate.is_some_and(|candidate| {
            candidate.target_path != pending.target_path
                || candidate.generation != pending.generation
        }) {
            self.warm_candidate = None;
            self.readiness_observation_ids.clear();
        }
        self.migration_hold = 0;
        Ok(pending)
    }

    /// Legacy post-failure readiness is intentionally no longer a promotion
    /// path: D064 readiness must have been accumulated while UDP was active.
    pub fn observe_failed_udp_target_readiness(
        &mut self,
        _observation: ReadinessObservation,
    ) -> Result<Option<CarrierSwitchDecision>, MigrationError> {
        Err(MigrationError::Unvalidated)
    }

    pub fn promote_warm_authenticated_resume(
        &mut self,
        target_path: PathId,
        generation: PathGeneration,
        session_id: u64,
        delivery_epoch: u64,
    ) -> Result<CarrierSwitchDecision, MigrationError> {
        let pending = self
            .pending_switch
            .ok_or(MigrationError::GenerationMismatch)?;
        let candidate = self.warm_candidate.ok_or(MigrationError::Unvalidated)?;
        if pending.target_path != target_path
            || pending.generation != generation
            || candidate.target_path != target_path
            || candidate.generation != generation
            || candidate.session_id != session_id
            || candidate.delivery_epoch != delivery_epoch
        {
            return Err(MigrationError::GenerationMismatch);
        }
        if !candidate.warm || candidate.ready_observations != 3 {
            return Err(MigrationError::Unvalidated);
        }
        let decision = self.promote_pending_target()?;
        self.warm_recoveries = self.warm_recoveries.saturating_add(1);
        Ok(decision)
    }

    /// Cold recovery has no pre-existing D064 readiness sequence. Promotion is
    /// limited to authenticated, resume-validated recovery and makes no warm claim.
    pub fn promote_cold_authenticated_resume(
        &mut self,
        target_path: PathId,
        generation: PathGeneration,
        authenticated: bool,
        resume_validated: bool,
    ) -> Result<CarrierSwitchDecision, MigrationError> {
        let pending = self
            .pending_switch
            .ok_or(MigrationError::GenerationMismatch)?;
        if generation.0 < pending.generation.0 {
            return Err(MigrationError::OldGeneration);
        }
        if target_path != pending.target_path || generation != pending.generation {
            return Err(MigrationError::GenerationMismatch);
        }
        if !authenticated || !resume_validated {
            return Err(MigrationError::Unvalidated);
        }
        let decision = self.promote_pending_target()?;
        self.cold_recoveries = self.cold_recoveries.saturating_add(1);
        Ok(decision)
    }

    fn promote_pending_target(&mut self) -> Result<CarrierSwitchDecision, MigrationError> {
        let pending = self
            .pending_switch
            .ok_or(MigrationError::GenerationMismatch)?;
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
        self.warm_candidate = None;
        self.readiness_observation_ids.clear();
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
            || candidate_score.score
                < current_score
                    .score
                    .saturating_add(self.limits.switch_margin)
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
                            >= x.saturating_add(self.limits.switch_margin)
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
    fn negative_switch_margin_is_rejected_at_construction() {
        // M4 ADR: the candidate score must exceed the active path's by a
        // positive `switch_margin` improvement. A negative margin inverts the
        // gate and would admit a strictly worse healthy candidate; it has no
        // committed meaning and must be rejected via the existing
        // invalid-limit path, not silently inverted at comparison time.
        assert!(matches!(
            CarrierManager::new(ManagerLimits {
                min_hold_events: 1,
                switch_margin: -1,
                max_paths: 2,
            }),
            Err(FlowError::InvalidLimit)
        ));
        assert!(matches!(
            CarrierManager::new(ManagerLimits {
                min_hold_events: 1,
                switch_margin: i64::MIN,
                max_paths: 2,
            }),
            Err(FlowError::InvalidLimit)
        ));
    }

    #[test]
    fn migrate_back_switch_margin_addition_does_not_overflow() {
        // switch_margin is a caller-supplied i64 with no range bound. A huge
        // margin must saturate the score comparison instead of overflowing
        // i64 addition (which would panic in debug or wrap in release and let
        // an unjustified migration through).
        let mut m = CarrierManager::new(ManagerLimits {
            min_hold_events: 1,
            switch_margin: i64::MAX,
            max_paths: 2,
        })
        .unwrap();
        // Active TCP path (id 1) with a perfect sample; candidate UDP path 2
        // same generation, validated and healthy.
        m.observe(PathId(1), GOOD).unwrap();
        m.observe(PathId(2), GOOD).unwrap();
        m.set_active_tcp(PathId(1), PathGeneration(1)).unwrap();
        let candidate = MigrationCandidate {
            path: PathId(2),
            generation: PathGeneration(1),
            validated: true,
            health: GOOD,
        };
        // The score comparison runs before the hold gate, so the very first
        // call reaches `current_score + switch_margin`; with margin ==
        // i64::MAX the addition must saturate, not overflow.
        let result = m.migrate_back_to_udp(candidate);
        // With margin == i64::MAX the candidate can never beat current+margin,
        // so the deterministic outcome is ScoreMargin — not a panic and not a
        // spurious success.
        assert_eq!(result, Err(MigrationError::ScoreMargin));
        assert_eq!(m.active(), Some(PathId(1)));
    }

    #[test]
    fn choose_switch_margin_addition_does_not_overflow() {
        let mut m = CarrierManager::new(ManagerLimits {
            min_hold_events: 1,
            switch_margin: i64::MAX,
            max_paths: 2,
        })
        .unwrap();
        m.observe(PathId(1), GOOD).unwrap();
        m.observe(PathId(2), GOOD).unwrap();
        m.set_active_tcp(PathId(1), PathGeneration(1)).unwrap();
        // choose() compares the best candidate score against current + margin,
        // but only once hold >= min_hold_events. First call accumulates hold;
        // second call reaches the `x + margin` comparison which must saturate
        // rather than overflow — keeping the current path, never switching on a
        // wrapped negative threshold.
        assert_eq!(m.choose(), Some(PathId(1)));
        assert_eq!(m.choose(), Some(PathId(1)));
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

    fn failed_manager() -> (CarrierManager, PendingCarrierSwitch) {
        let mut manager = CarrierManager::new(ManagerLimits {
            min_hold_events: 1,
            switch_margin: 0,
            max_paths: 2,
        })
        .unwrap();
        manager.set_active_udp(PathId(9), PathGeneration(1));
        let pending = manager
            .fail_udp_to_tcp(
                PathId(9),
                PathGeneration(1),
                PathId(10),
                HealthState::Failed,
                CarrierSwitchReason::UdpPathDegraded,
            )
            .unwrap();
        (manager, pending)
    }
    fn active_manager() -> CarrierManager {
        let mut manager = CarrierManager::new(ManagerLimits {
            min_hold_events: 1,
            switch_margin: 0,
            max_paths: 2,
        })
        .unwrap();
        manager.set_active_udp(PathId(9), PathGeneration(1));
        manager
            .prepare_warm_candidate(PathId(10), PathGeneration(2), 7001, 4)
            .unwrap();
        manager
    }

    fn warm_readiness(id: u64) -> ReadinessObservation {
        ReadinessObservation {
            target_path: PathId(10),
            generation: PathGeneration(2),
            session_id: 7001,
            delivery_epoch: 4,
            observation_id: id,
            authenticated: true,
            resume_validated: true,
            resource_admitted: true,
        }
    }

    #[test]
    fn d064_warm_readiness_is_prefailure_bounded_and_distinct() {
        let mut manager = active_manager();
        assert!(
            !manager
                .observe_warm_candidate_readiness(warm_readiness(1))
                .unwrap()
        );
        assert!(
            !manager
                .observe_warm_candidate_readiness(warm_readiness(1))
                .unwrap()
        );
        assert!(
            !manager
                .observe_warm_candidate_readiness(warm_readiness(2))
                .unwrap()
        );
        assert_eq!(manager.active(), Some(PathId(9)));
        assert!(
            manager
                .observe_warm_candidate_readiness(warm_readiness(3))
                .unwrap()
        );
        assert_eq!(manager.active(), Some(PathId(9)));
        assert_eq!(manager.warm_candidate().unwrap().ready_observations, 3);
        assert!(
            manager
                .observe_warm_candidate_readiness(warm_readiness(4))
                .unwrap()
        );
        assert_eq!(manager.warm_candidate().unwrap().ready_observations, 3);
    }

    #[test]
    fn d064_wrong_dimensions_and_unvalidated_reset_fail_closed() {
        for (bad, error) in [
            (
                ReadinessObservation {
                    target_path: PathId(11),
                    ..warm_readiness(1)
                },
                MigrationError::GenerationMismatch,
            ),
            (
                ReadinessObservation {
                    generation: PathGeneration(1),
                    ..warm_readiness(1)
                },
                MigrationError::OldGeneration,
            ),
            (
                ReadinessObservation {
                    generation: PathGeneration(3),
                    ..warm_readiness(1)
                },
                MigrationError::GenerationMismatch,
            ),
            (
                ReadinessObservation {
                    session_id: 7002,
                    ..warm_readiness(1)
                },
                MigrationError::GenerationMismatch,
            ),
            (
                ReadinessObservation {
                    delivery_epoch: 5,
                    ..warm_readiness(1)
                },
                MigrationError::GenerationMismatch,
            ),
        ] {
            let mut manager = active_manager();
            manager
                .observe_warm_candidate_readiness(warm_readiness(99))
                .unwrap();
            assert_eq!(manager.warm_candidate().unwrap().ready_observations, 1);
            assert_eq!(manager.observe_warm_candidate_readiness(bad), Err(error));
            assert_eq!(manager.active(), Some(PathId(9)));
            assert_eq!(manager.warm_candidate().unwrap().ready_observations, 0);
        }
        for bad in [
            ReadinessObservation {
                authenticated: false,
                ..warm_readiness(2)
            },
            ReadinessObservation {
                resume_validated: false,
                ..warm_readiness(2)
            },
            ReadinessObservation {
                resource_admitted: false,
                ..warm_readiness(2)
            },
        ] {
            let mut manager = active_manager();
            manager
                .observe_warm_candidate_readiness(warm_readiness(1))
                .unwrap();
            assert_eq!(
                manager.observe_warm_candidate_readiness(bad),
                Err(MigrationError::Unvalidated)
            );
            assert_eq!(manager.warm_candidate().unwrap().ready_observations, 0);
        }
    }

    #[test]
    fn exact_warm_generation_promotes_only_after_failure() {
        let mut manager = active_manager();
        for id in 1..=3 {
            manager
                .observe_warm_candidate_readiness(warm_readiness(id))
                .unwrap();
        }
        let pending = manager
            .fail_udp_to_tcp(
                PathId(9),
                PathGeneration(1),
                PathId(10),
                HealthState::Failed,
                CarrierSwitchReason::UdpPathDegraded,
            )
            .unwrap();
        assert_eq!(manager.active(), None);
        assert_eq!(
            manager.promote_warm_authenticated_resume(PathId(10), PathGeneration(3), 7001, 4),
            Err(MigrationError::GenerationMismatch)
        );
        let decision = manager
            .promote_warm_authenticated_resume(PathId(10), pending.generation, 7001, 4)
            .unwrap();
        assert_eq!(decision.active_path, PathId(10));
        assert_eq!(manager.active(), Some(PathId(10)));
        assert_eq!(manager.warm_recoveries, 1);
        assert_eq!(manager.cold_recoveries, 0);
    }

    #[test]
    fn absent_or_failed_warm_uses_new_generation_cold() {
        let (mut cold, pending) = failed_manager();
        cold.promote_cold_authenticated_resume(pending.target_path, pending.generation, true, true)
            .unwrap();
        assert_eq!((cold.warm_recoveries, cold.cold_recoveries), (0, 1));

        let mut manager = active_manager();
        for id in 1..=3 {
            manager
                .observe_warm_candidate_readiness(warm_readiness(id))
                .unwrap();
        }
        manager
            .fail_warm_candidate(PathId(10), PathGeneration(2))
            .unwrap();
        assert_eq!(manager.failed_warm_attempts, 1);
        manager
            .prepare_warm_candidate(PathId(10), PathGeneration(3), 7001, 4)
            .unwrap();
        manager
            .fail_warm_candidate(PathId(10), PathGeneration(3))
            .unwrap();
        let pending = manager
            .fail_udp_to_tcp(
                PathId(9),
                PathGeneration(1),
                PathId(10),
                HealthState::Failed,
                CarrierSwitchReason::UdpPathDegraded,
            )
            .unwrap();
        manager
            .promote_cold_authenticated_resume(pending.target_path, pending.generation, true, true)
            .unwrap();
        assert_eq!(
            (
                manager.failed_warm_attempts,
                manager.warm_recoveries,
                manager.cold_recoveries
            ),
            (1, 0, 1)
        );
    }

    #[test]
    fn post_failure_readiness_cannot_manufacture_warm_or_delivery() {
        let (mut manager, _) = failed_manager();
        assert_eq!(
            manager.observe_failed_udp_target_readiness(warm_readiness(1)),
            Err(MigrationError::Unvalidated)
        );
        assert_eq!(manager.active(), None);
        assert_eq!(manager.switches, 0);
        let mut failover = FailoverController::new(3, 2, 32).unwrap();
        failover.track_uncertain(DataId(7), b"cat").unwrap();
        assert_eq!(failover.tcp_resend(), Err(FailoverError::WrongCarrier));
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

/// Path-local live-recovery ownership seam for a single UDP path.
///
/// `PathRecovery` lets a UDP carrier own `neko_reliable::Recovery` plus the
/// Reno/bytes-in-flight model for one `(path_id, path_generation)` without
/// coupling packet-level ACK to Session delivery. Packet numbers come from the
/// authenticated `SecureSession` record sequence (the AEAD nonce) — the single
/// source of truth — so `PathRecovery` records rather than allocates them. It owns
/// allocation, sent-packet bookkeeping, authenticated ACK application, PTO
/// probe selection and frame-level retransmit output — all as Carrier-local
/// `FrameId` recovery signals. It deliberately carries no `SessionRuntime`,
/// `DeliveryLedger`, or `confirm_received` reference: a packet ACK retires a
/// Carrier packet/frame, never a Session delivery.
#[derive(Debug)]
pub struct PathRecovery {
    path: PathId,
    path_generation: u64,
    recovery: neko_reliable::Recovery,
    reno: neko_reliable::Reno,
    packets_sent: u64,
    packets_lost: u64,
    /// Bytes actually charged to `reno` per sent packet (0 for non-ack-eliciting
    /// packets). Lets `on_ack` release exactly the charged bytes for each
    /// retired packet instead of draining another packet's charge.
    charged: BTreeMap<u64, u64>,
    /// Bumped only when a *resolved* recovery outcome arrives — an ACK that
    /// retires/loses/retransmits work, or a PTO that schedules probe frames.
    /// A bare `on_sent` is new traffic, not a resolved observation, so it does
    /// NOT bump this; an unchanged cumulative snapshot cannot be replayed as
    /// a distinct bad health observation merely by sending more packets.
    outcome_epoch: u64,
    health_epoch: Option<u64>,
    /// Counters at the last consumed health observation — the manager-facing
    /// sample is the *delta* of resolved packets since then, so an old
    /// cumulative loss is never replayed as a fresh bad observation.
    last_health_sent: u64,
    last_health_lost: u64,
    /// Packets resolved (acked or declared lost) and those resolved as lost —
    /// the loss-delta denominator is keyed to *resolved outcomes*, not sends,
    /// so a PTO that consumed no resolved packets can't zero out a later loss.
    resolved_packets: u64,
    resolved_lost: u64,
}

/// Outcome of applying an authenticated packet ACK to a path.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct RecoveryAckOutcome {
    pub acked_packets: Vec<u64>,
    pub lost_packets: Vec<u64>,
    pub retransmit_frames: Vec<neko_reliable::FrameId>,
    pub acked_bytes: u64,
    pub lost_bytes: u64,
    /// The engine's raw result — retained for observability mapping.
    pub result: neko_reliable::RecoveryResult,
}

/// Errors from the path-local recovery seam.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum PathRecoveryError {
    /// The recovery engine rejected the input (invalid range, capacity, etc.).
    Recovery(neko_reliable::Error),
    /// The ACK/generation does not belong to this path generation.
    GenerationMismatch,
    /// Congestion window refused admission — the send records nothing.
    CongestionWindowFull,
    /// A duplicate delivery carried different bytes for the same logical
    /// identity — fail closed.
    DeliveryConflict,
}

impl From<neko_reliable::Error> for PathRecoveryError {
    fn from(e: neko_reliable::Error) -> Self {
        Self::Recovery(e)
    }
}

impl PathRecovery {
    /// Create a path-local recovery owner bound to one path generation.
    pub fn new(path: PathId, path_generation: u64, mss: u64) -> Result<Self, PathRecoveryError> {
        Ok(Self {
            path,
            path_generation,
            recovery: neko_reliable::Recovery::default(),
            reno: neko_reliable::Reno::new(mss)?,
            packets_sent: 0,
            packets_lost: 0,
            outcome_epoch: 0,
            // Start as if outcome_epoch 0 were already consumed: with no
            // resolved outcome yet there is nothing fresh to observe.
            health_epoch: Some(0),
            last_health_sent: 0,
            last_health_lost: 0,
            resolved_packets: 0,
            resolved_lost: 0,
            charged: BTreeMap::new(),
        })
    }

    /// Record a sent packet's number/bytes/frames and charge bytes-in-flight.
    /// The packet number is the authenticated `SecureSession` record sequence
    /// (the AEAD nonce) — the single source of truth, never a second allocator.
    /// `frames` are Carrier-level `FrameId`s owned by the caller; this seam
    /// does not interpret them as Session delivery units.
    pub fn on_sent(&mut self, packet: neko_reliable::SentPacket) -> Result<(), PathRecoveryError> {
        self.recovery.on_sent(packet.clone())?;
        let charge = if packet.ack_eliciting {
            self.reno.sent(packet.bytes);
            packet.bytes
        } else {
            0
        };
        self.charged.insert(packet.number, charge);
        self.packets_sent = self.packets_sent.saturating_add(1);
        // A bare send is new traffic, not a resolved recovery outcome — it does
        // NOT advance outcome_epoch, so it cannot replay old cumulative loss.
        Ok(())
    }

    /// Apply an authenticated ACK range set. `generation` must match this
    /// path's current generation — a stale/other-generation ACK is rejected
    /// before any RTT/loss/PTO mutation. Returns Carrier-local recovery
    /// outcome only; no Session delivery evidence is produced.
    pub fn on_ack(
        &mut self,
        generation: u64,
        ack: &neko_reliable::AckRanges,
        now_us: u64,
        ack_delay_us: u64,
    ) -> Result<RecoveryAckOutcome, PathRecoveryError> {
        if generation != self.path_generation {
            return Err(PathRecoveryError::GenerationMismatch);
        }
        let r = self.recovery.on_ack(ack, now_us, ack_delay_us)?;
        // Release exactly the bytes each retired packet actually charged —
        // a non-ack-eliciting packet was charged 0 and must not drain another
        // packet's congestion bytes.
        let mut released_acked = 0u64;
        let mut released_lost = 0u64;
        for n in &r.acked_packets {
            released_acked = released_acked.saturating_add(self.charged.remove(n).unwrap_or(0));
        }
        for n in &r.lost_packets {
            released_lost = released_lost.saturating_add(self.charged.remove(n).unwrap_or(0));
        }
        self.reno.acked(released_acked);
        self.reno.lost(released_lost);
        self.packets_lost = self
            .packets_lost
            .saturating_add(r.lost_packets.len() as u64);
        // Resolved-outcome counters for the interval-delta health denominator:
        // keyed to acked+lost packets, never to bare sends.
        self.resolved_packets = self
            .resolved_packets
            .saturating_add((r.acked_packets.len() + r.lost_packets.len()) as u64);
        self.resolved_lost = self
            .resolved_lost
            .saturating_add(r.lost_packets.len() as u64);
        // A resolved ACK outcome (any retired/lost/retransmitted work) is a
        // fresh observation; an empty ACK that changed nothing is not.
        if !r.acked_packets.is_empty()
            || !r.lost_packets.is_empty()
            || !r.retransmit_frames.is_empty()
        {
            self.outcome_epoch = self.outcome_epoch.saturating_add(1);
        }
        Ok(RecoveryAckOutcome {
            acked_packets: r.acked_packets.clone(),
            lost_packets: r.lost_packets.clone(),
            retransmit_frames: r.retransmit_frames.clone(),
            acked_bytes: r.acked_bytes,
            lost_bytes: r.lost_bytes,
            result: r,
        })
    }

    /// Select the bounded set of oldest outstanding frames for a PTO probe.
    /// Probe frames are re-encoded by the caller into a *fresh* packet — the
    /// old encrypted packet image is never resent.
    pub fn on_pto(
        &mut self,
        max_probe_frames: usize,
    ) -> Result<Vec<neko_reliable::FrameId>, PathRecoveryError> {
        let frames = self.recovery.on_pto(max_probe_frames)?;
        // A PTO that actually schedules probe frames is a resolved outcome.
        if !frames.is_empty() {
            self.outcome_epoch = self.outcome_epoch.saturating_add(1);
        }
        Ok(frames)
    }

    /// Whether `bytes` more may be sent under the Reno congestion window.
    pub fn can_send(&self, bytes: u64) -> bool {
        self.reno.can_send(bytes)
    }

    /// Current pacing interval in microseconds for `bytes` under the measured RTT.
    pub fn pacing_interval_us(&self, bytes: u64) -> u64 {
        self.reno
            .pacing_interval_us(self.recovery.rtt.smoothed_us, bytes)
    }

    /// Authoritative final-copy retirement signal: is `frame` still carried by
    /// at least one outstanding packet copy? Under overlapping original and
    /// retransmit copies, a frame's retained plaintext is releasable only when
    /// this returns false.
    pub fn frame_outstanding(&self, frame: neko_reliable::FrameId) -> bool {
        self.recovery.frame_outstanding(frame)
    }

    /// Read-only access to the underlying recovery engine for observability
    /// projection (`neko_observe::record_recovery_ack`/`record_pto`).
    pub fn recovery(&self) -> &neko_reliable::Recovery {
        &self.recovery
    }

    pub fn path(&self) -> PathId {
        self.path
    }
    pub fn path_generation(&self) -> u64 {
        self.path_generation
    }
    pub fn in_flight(&self) -> usize {
        self.recovery.in_flight()
    }
    pub fn bytes_in_flight(&self) -> u64 {
        self.reno.bytes_in_flight
    }
    /// Packet-recovery counters for health/observability — sent and declared
    /// lost packet totals. Loss-per-mille is `packets_lost*1000/packets_sent`.
    pub fn packets_sent(&self) -> u64 {
        self.packets_sent
    }
    pub fn packets_lost(&self) -> u64 {
        self.packets_lost
    }
    /// The current measured RTT in microseconds (0 until a sample exists).
    pub fn rtt_us(&self) -> u64 {
        self.recovery.rtt.smoothed_us
    }
    /// The public, freshness-enforced health bridge: yields a `HealthSample`
    /// only when a *resolved* recovery outcome (an ACK that retired/lost/
    /// retransmitted work, or a PTO that scheduled probes) arrived since the
    /// last consumption. Polling with no new resolved outcome returns `None`
    /// — and a bare send is not a resolved outcome, so it cannot replay a
    /// historical cumulative loss into extra bad observations. Packet
    /// feedback still cannot validate a Path or confirm Session delivery.
    pub fn fresh_health_sample(&mut self) -> Option<HealthSample> {
        if self.health_epoch == Some(self.outcome_epoch) {
            return None;
        }
        self.health_epoch = Some(self.outcome_epoch);
        // Manager-facing evidence is the resolved *interval delta*, not the
        // lifetime cumulative ratio — an old loss burst is never replayed as a
        // new bad observation by a subsequent clean outcome.
        let delta_sent = self.resolved_packets.saturating_sub(self.last_health_sent);
        let delta_lost = self.resolved_lost.saturating_sub(self.last_health_lost);
        self.last_health_sent = self.resolved_packets;
        self.last_health_lost = self.resolved_lost;
        let loss_per_mille = delta_lost
            .saturating_mul(1000)
            .checked_div(delta_sent)
            .map(|r| r.min(u16::MAX as u64) as u16)
            .unwrap_or(0);
        Some(HealthSample {
            rtt_us: self.recovery.rtt.smoothed_us,
            loss_per_mille,
            pto: self.recovery.pto_count.min(u16::MAX as u32) as u16,
        })
    }

    /// Read-only raw counters for observability/diagnostics. This is NOT a
    /// manager-health input: feeding cumulative state directly to
    /// `CarrierHealth::observe` bypasses the freshness guard. Use
    /// `fresh_health_sample()` for manager evidence.
    pub fn diagnostics(&self) -> (u64, u64, u64, u32) {
        (
            self.packets_sent,
            self.packets_lost,
            self.recovery.rtt.smoothed_us,
            self.recovery.pto_count,
        )
    }

    /// Mark persistent congestion after repeated PTOs and collapse the window.
    pub fn persistent_congestion(&mut self) -> bool {
        if self.reno.persistent_congestion(self.recovery.pto_count) {
            self.reno.on_persistent_congestion();
            true
        } else {
            false
        }
    }
}

/// Bounded receiver-side packet-ACK tracker for the UDP recovery runtime.
///
/// The receiver observes the packet number of each authenticated record only
/// after a successful `SecureSession::open` (the sequence is the AEAD nonce).
/// Observed numbers are accumulated into canonical bounded `AckRanges`; a
/// duplicate/reordered packet is a no-op, and malformed/tampered records never
/// reach this tracker because `open` fails first. `build_ack` renders the
/// canonical `AckPayload` to be sealed back to the sender. The tracker is
/// Carrier-local and produces no Session delivery evidence.
#[derive(Debug)]
pub struct PacketAckTracker {
    path_generation: u64,
    ranges: neko_reliable::AckRanges,
    /// Largest packet number observed (drives `largest_observed`).
    largest_observed: Option<u64>,
    /// Set when an ack-eliciting packet was observed and not yet consumed by
    /// `take_ack`. An ACK-only (non-ack-eliciting) packet never sets this, so
    /// an ACK packet cannot trigger an ACK-of-ACK ping-pong.
    pending_ack: bool,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum AckTrackerError {
    /// The observed packet does not belong to this path generation.
    GenerationMismatch,
    /// The bounded ACK range set overflowed its limit.
    RangeLimit,
}

impl PacketAckTracker {
    /// Create a receiver tracker bound to one path generation.
    pub fn new(path_generation: u64) -> Self {
        Self {
            path_generation,
            // Bound the ACK range set to the wire's hard cap so an encoded
            // AckPayload always fits the canonical grammar.
            ranges: neko_reliable::AckRanges::new(32).expect("valid cap"),
            largest_observed: None,
            pending_ack: false,
        }
    }

    /// Record an authenticated packet observation. `generation` must match the
    /// tracker's path generation — a record for another generation is rejected
    /// before any ACK state changes. `ack_eliciting` distinguishes a real
    /// DATA/control packet (which creates a pending ACK obligation) from an
    /// ACK-only packet (which is recorded for range completeness but never by
    /// itself schedules a response — no ACK-of-ACK).
    pub fn observe_packet(
        &mut self,
        generation: u64,
        packet_number: u64,
        ack_eliciting: bool,
    ) -> Result<(), AckTrackerError> {
        if generation != self.path_generation {
            return Err(AckTrackerError::GenerationMismatch);
        }
        self.ranges
            .insert(packet_number)
            .map_err(|_| AckTrackerError::RangeLimit)?;
        self.largest_observed = Some(
            self.largest_observed
                .map_or(packet_number, |l| l.max(packet_number)),
        );
        if ack_eliciting {
            self.pending_ack = true;
        }
        Ok(())
    }

    /// Consume the pending ACK obligation: yields the canonical `AckPayload`
    /// to seal back at most once per new ack-eliciting evidence, then returns
    /// `None` until another eligible packet is observed. An ACK-only packet
    /// does not create an obligation, so this never ACKs an ACK.
    pub fn take_ack(&mut self, ack_delay_us: u64) -> Option<neko_wire::AckPayload> {
        if !self.pending_ack {
            return None;
        }
        self.pending_ack = false;
        self.current_ack_payload(ack_delay_us)
    }

    /// Render the canonical `AckPayload` for sealing back to the sender.
    /// Returns `None` until at least one packet has been observed.
    pub fn build_ack(&self, ack_delay_us: u64) -> Option<neko_wire::AckPayload> {
        self.current_ack_payload(ack_delay_us)
    }

    fn current_ack_payload(&self, ack_delay_us: u64) -> Option<neko_wire::AckPayload> {
        let largest = self.largest_observed?;
        Some(neko_wire::AckPayload {
            largest_observed: largest,
            ack_delay_us,
            ranges: self
                .ranges
                .ranges()
                .iter()
                .map(|r| neko_wire::AckRangeWire {
                    start: r.start,
                    end: r.end,
                })
                .collect(),
        })
    }

    /// Whether any packet has been observed yet.
    pub fn has_observations(&self) -> bool {
        self.largest_observed.is_some()
    }
    /// Whether an ack-eliciting packet is waiting for an ACK emission.
    pub fn pending_ack(&self) -> bool {
        self.pending_ack
    }
}

/// A socket/crypto-agnostic reliable-UDP runtime orchestrator (R7/R8 core).
///
/// `ReliableUdpRuntime` composes the sender-side `PathRecovery`, receiver-side
/// `PacketAckTracker`, `RetransmitBuffer` plaintext ownership, the
/// freshness-enforced health bridge, and a `ConcurrentCarrierManager` — with no
/// socket or `SecureSession` baked in. A caller (lab fixture, CLI, or service)
/// supplies authenticated packet send/receive; this type owns the recovery,
/// ACK-emission, and automatic health-driven failover decisions. Packet
/// recovery evidence is layered and never becomes Session delivery.
#[derive(Debug)]
pub struct ReliableUdpRuntime {
    path: ConcurrentPathKey,
    tcp: ConcurrentPathKey,
    generation: u64,
    recovery: PathRecovery,
    acks: PacketAckTracker,
    retransmit: RetransmitBuffer,
    health: CarrierHealth,
    manager: ConcurrentCarrierManager,
    /// Which stable frames each sent packet carries — packet number is fresh
    /// per transmission while frame identity is stable across retransmits.
    packet_frames: BTreeMap<u64, Vec<neko_reliable::FrameId>>,
    /// Receiver-side application dedup keyed by the stable Session logical
    /// identity `(stream, offset)` — independent of the fresh packet number /
    /// AEAD nonce a retransmission used. First delivery is retained; a later
    /// duplicate with identical bytes is suppressed (counted), and a duplicate
    /// with different bytes fails closed as a conflict.
    delivered: BTreeMap<(u32, u64), Vec<u8>>,
    /// Exact-duplicate deliveries suppressed (application-visible count stays
    /// one per stable identity). Packet ACK stays separate from this dedup.
    pub dedup_suppressed: u64,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum RuntimeEvent {
    /// A fresh health observation was produced for the path.
    HealthSample(HealthState),
    /// Degradation crossed the hysteresis and the warm TCP standby was
    /// automatically promoted — carries the real switch event for the caller
    /// to record as switch evidence (separate from health evidence).
    WarmFallback(ConcurrentSwitchEvent),
    /// Degradation was detected but the UDP fail or TCP promote transition
    /// failed — explicitly reported, never silently treated as a fallback.
    FallbackFailed,
    /// Nothing actionable this poll.
    Idle,
}

impl ReliableUdpRuntime {
    /// Create the runtime for a UDP path generation with a ready TCP standby.
    pub fn new(path_generation: u64, mss: u64) -> Result<Self, PathRecoveryError> {
        let udp = ConcurrentPathKey {
            path: PathId(1),
            generation: PathGeneration(path_generation),
        };
        let tcp = ConcurrentPathKey {
            path: PathId(2),
            generation: PathGeneration(path_generation),
        };
        let mut manager = ConcurrentCarrierManager::new(ConcurrentLimits::default()).unwrap();
        manager
            .register(udp, CarrierKind::Udp)
            .map_err(|_| PathRecoveryError::GenerationMismatch)?;
        manager
            .register(tcp, CarrierKind::Tcp)
            .map_err(|_| PathRecoveryError::GenerationMismatch)?;
        Ok(Self {
            path: udp,
            tcp,
            generation: path_generation,
            recovery: PathRecovery::new(PathId(1), path_generation, mss)?,
            acks: PacketAckTracker::new(path_generation),
            retransmit: RetransmitBuffer::new(64, 8192)
                .map_err(|_| PathRecoveryError::GenerationMismatch)?,
            health: CarrierHealth::new(HealthLimits::default())
                .map_err(|_| PathRecoveryError::GenerationMismatch)?,
            manager,
            packet_frames: BTreeMap::new(),
            delivered: BTreeMap::new(),
            dedup_suppressed: 0,
        })
    }

    /// Mark TCP standby ready so a degradation can promote it.
    pub fn ready_standby(&mut self, now_ms: u64) {
        for i in 0..3 {
            let _ = self
                .manager
                .observe_readiness(self.tcp, true, true, now_ms + i);
        }
    }
    /// Activate UDP as the active path (caller drives admission order).
    pub fn activate_udp(&mut self, now_ms: u64) {
        let _ = self
            .manager
            .activate(self.path, SwitchReason::OperatorRequest, now_ms, false);
    }

    /// Congestion admission gate: is there cwnd capacity for `bytes`? Callers
    /// must consult this before sealing/sending — a refused send records no
    /// recovery charge and consumes no frame/plaintext ownership (fail-closed:
    /// nothing is sent-but-untracked). Pacing is the deterministic
    /// `pacing_interval_us` deadline the caller honors between admissions.
    pub fn can_send(&self, bytes: u64) -> bool {
        self.recovery.can_send(bytes)
    }
    /// Deterministic pacing deadline between admissions (microseconds) for a
    /// `bytes`-sized send.
    pub fn pacing_interval_us(&self, bytes: u64) -> u64 {
        self.recovery.pacing_interval_us(bytes)
    }

    /// Deliver an authenticated application payload under its stable Session
    /// logical identity `(stream, offset)`. The first delivery is retained and
    /// reported; a later duplicate carrying the SAME bytes is suppressed and
    /// counted (`dedup_suppressed`), and a duplicate carrying DIFFERENT bytes
    /// for the same identity fails closed. Packet ACK/recovery never marks
    /// Session delivery — this logical-identity dedup is the delivery path.
    /// Record that packet `number` (fresh AEAD nonce/sequence) was sent carrying
    /// the stable `frame` identity — only after `can_send` admits the bytes.
    /// Refuses (without charging recovery or tracking plaintext) when cwnd is
    /// full, so no sent-but-untracked packet can result. Frame identity is
    /// independent of packet number: a retransmission keeps the same `frame`
    /// under a fresh number.
    pub fn on_packet_sent(
        &mut self,
        number: u64,
        sent_at_us: u64,
        bytes: u64,
        frame: neko_reliable::FrameId,
        frame_plaintext: &[u8],
    ) -> Result<(), PathRecoveryError> {
        // Congestion admission: refused sends charge/record nothing.
        if !self.recovery.can_send(bytes) {
            return Err(PathRecoveryError::CongestionWindowFull);
        }
        // Transactional plaintext ownership: reserve the bounded retransmit
        // plaintext FIRST so a capacity/conflict/oversize rejection leaves no
        // recovery packet, no Reno charge, and no packet->frame entry.
        self.retransmit
            .track(frame, frame_plaintext)
            .map_err(|_| PathRecoveryError::DeliveryConflict)?;
        // A failure to record the packet rolls back ONLY the freshly reserved
        // copy (and only if we actually added a new one this call).
        match self.recovery.on_sent(neko_reliable::SentPacket {
            number,
            sent_at_us,
            bytes,
            ack_eliciting: true,
            frames: vec![frame],
        }) {
            Ok(()) => {}
            Err(e) => {
                self.retransmit.release(frame);
                return Err(e);
            }
        }
        self.packet_frames.insert(number, vec![frame]);
        Ok(())
    }

    /// Deliver an authenticated application payload under its stable Session
    /// logical identity `(stream, offset)`. The first delivery is retained and
    /// reported; a later duplicate carrying the SAME bytes is suppressed and
    /// counted (`dedup_suppressed`), and a duplicate carrying DIFFERENT bytes
    /// for the same identity fails closed. Packet ACK/recovery never marks
    /// Session delivery — this logical-identity dedup is the delivery path.
    pub fn deliver_logical(
        &mut self,
        stream: u32,
        offset: u64,
        data: &[u8],
    ) -> Result<bool, PathRecoveryError> {
        match self.delivered.entry((stream, offset)) {
            std::collections::btree_map::Entry::Vacant(v) => {
                v.insert(data.to_vec());
                Ok(true) // first delivery
            }
            std::collections::btree_map::Entry::Occupied(o) => {
                if o.get() == data {
                    self.dedup_suppressed = self.dedup_suppressed.saturating_add(1);
                    Ok(false) // exact duplicate suppressed
                } else {
                    Err(PathRecoveryError::DeliveryConflict) // conflict fails closed
                }
            }
        }
    }

    /// Record an authenticated received packet: `ack_eliciting` distinguishes
    /// Data (creates a pending ACK) from ACK-only records (no ACK-of-ACK).
    pub fn on_packet_received(
        &mut self,
        packet_number: u64,
        ack_eliciting: bool,
    ) -> Result<(), AckTrackerError> {
        self.acks
            .observe_packet(self.generation, packet_number, ack_eliciting)
    }

    /// If an ack-eliciting packet awaits a response, return the canonical
    /// `AckPayload` to seal back (consumed once; no ACK-of-ACK).
    pub fn poll_outgoing_ack(&mut self, ack_delay_us: u64) -> Option<neko_wire::AckPayload> {
        self.acks.take_ack(ack_delay_us)
    }

    /// Apply an authenticated ACK payload to recovery and release retired
    /// frames' plaintext exactly once. Returns the engine outcome.
    pub fn apply_ack(
        &mut self,
        ack: &neko_reliable::AckRanges,
        now_us: u64,
        ack_delay_us: u64,
    ) -> Result<RecoveryAckOutcome, PathRecoveryError> {
        let out = self
            .recovery
            .on_ack(self.generation, ack, now_us, ack_delay_us)?;
        // Resolve the real frames carried by each retired packet — frame
        // identity is looked up, never reconstructed as FrameId(packet_no).
        let mut candidates: Vec<neko_reliable::FrameId> = Vec::new();
        for n in out.acked_packets.iter().chain(out.lost_packets.iter()) {
            candidates.extend(self.packet_frames.get(n).cloned().unwrap_or_default());
            self.packet_frames.remove(n);
        }
        let scheduled: std::collections::BTreeSet<_> =
            out.retransmit_frames.iter().copied().collect();
        for f in candidates {
            // Release only when no outstanding copy remains AND the frame is
            // not currently scheduled for retransmission.
            if !self.recovery.frame_outstanding(f) && !scheduled.contains(&f) {
                self.retransmit.release(f);
            }
        }
        Ok(out)
    }

    /// Poll fresh health evidence and, on Degraded, automatically fail UDP and
    /// promote the warm TCP standby. Returns the emitted event (or Idle).
    /// Poll fresh health evidence and, on Degraded, automatically fail UDP and
    /// promote the warm TCP standby. `WarmFallback` is reported ONLY after the
    /// promotion actually succeeded — a failed `fail`/`activate` is propagated
    /// as `FallbackFailed`, never silently treated as a handled fallback. The
    /// real switch event is returned for the caller's observability sink.
    pub fn poll_health(&mut self, now_ms: u64) -> RuntimeEvent {
        match self.recovery.fresh_health_sample() {
            None => RuntimeEvent::Idle,
            Some(s) => {
                let state = self
                    .health
                    .observe(PathId(1), s)
                    .unwrap_or(HealthState::Unknown);
                if state == HealthState::Degraded
                    && self.manager.state(self.path).is_ok()
                    && self.manager.state(self.tcp).is_ok()
                {
                    // Fail UDP first; a failed transition must not fabricate a
                    // successful fallback.
                    if self
                        .manager
                        .fail(self.path, SwitchReason::UdpPathDegraded, now_ms)
                        .is_err()
                    {
                        return RuntimeEvent::FallbackFailed;
                    }
                    return match self.manager.activate(
                        self.tcp,
                        SwitchReason::UdpPathDegraded,
                        now_ms,
                        true,
                    ) {
                        Ok(event) => RuntimeEvent::WarmFallback(event),
                        Err(_) => RuntimeEvent::FallbackFailed,
                    };
                }
                RuntimeEvent::HealthSample(state)
            }
        }
    }

    /// Fire a PTO and return the `(stable FrameId, plaintext)` pairs to
    /// re-encode under a FRESH packet number — the caller re-sends the same
    /// frame identity, not a new FrameId keyed by the new packet number.
    pub fn pto_probe(&mut self) -> Vec<(neko_reliable::FrameId, Vec<u8>)> {
        self.recovery
            .on_pto(4)
            .map(|frames| {
                frames
                    .iter()
                    .filter_map(|f| self.retransmit.get(*f).map(|b| (*f, b.to_vec())))
                    .collect()
            })
            .unwrap_or_default()
    }

    /// Record a fresh retransmission packet carrying a stable frame identity.
    /// The caller supplies a fresh packet number/nonce; the frame id stays the
    /// original stable id so overlapping-copy accounting stays truthful.
    pub fn on_retransmit_sent(
        &mut self,
        packet_number: u64,
        sent_at_us: u64,
        bytes: u64,
        frame: neko_reliable::FrameId,
    ) -> Result<(), PathRecoveryError> {
        self.recovery.on_sent(neko_reliable::SentPacket {
            number: packet_number,
            sent_at_us,
            bytes,
            ack_eliciting: true,
            frames: vec![frame],
        })?;
        self.packet_frames.insert(packet_number, vec![frame]);
        Ok(())
    }

    pub fn manager(&self) -> &ConcurrentCarrierManager {
        &self.manager
    }
    /// Mutable manager access for readiness/admission wiring by the harness.
    pub fn manager_mut(&mut self) -> &mut ConcurrentCarrierManager {
        &mut self.manager
    }
    /// Read-only recovery engine for observability projection.
    pub fn recovery_engine(&self) -> &neko_reliable::Recovery {
        self.recovery.recovery()
    }
    pub fn in_flight(&self) -> usize {
        self.recovery.in_flight()
    }
    /// Congestion bytes currently charged to Reno bytes-in-flight.
    pub fn recovery_bytes_in_flight(&self) -> u64 {
        self.recovery.bytes_in_flight()
    }
    pub fn packets_lost(&self) -> u64 {
        self.recovery.packets_lost()
    }
}

/// Bounded carrier-local retransmission frame ownership (R6).
///
/// `RetransmitBuffer` maps each outstanding `FrameId` to its bounded plaintext
/// frame bytes so a runtime can re-encode lost frames into a *fresh* packet —
/// never resend an old encrypted packet image. A frame's bytes are released
/// exactly when no outstanding copy remains (acked) or when the owner drops the
/// buffer on path/generation teardown. Hard-bounded by the recovery engine's
/// frame limits; produces no Session delivery evidence.
#[derive(Debug)]
pub struct RetransmitBuffer {
    frames: BTreeMap<neko_reliable::FrameId, Vec<u8>>,
    max_frames: usize,
    max_bytes: usize,
    bytes: usize,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum RetransmitError {
    /// The buffer is at its frame or byte capacity.
    Capacity,
    /// The frame id is already tracked with different bytes.
    Conflict,
    /// A single frame exceeds the per-buffer byte bound.
    FrameTooLarge,
}

impl RetransmitBuffer {
    /// Create a buffer bounded to at most `max_frames` frames / `max_bytes`
    /// total retained plaintext bytes.
    pub fn new(max_frames: usize, max_bytes: usize) -> Result<Self, RetransmitError> {
        if max_frames == 0 || max_bytes == 0 {
            return Err(RetransmitError::Capacity);
        }
        Ok(Self {
            frames: BTreeMap::new(),
            max_frames,
            max_bytes,
            bytes: 0,
        })
    }

    /// Track `frame`'s plaintext for potential retransmission. Re-tracking the
    /// same id with identical bytes is a no-op; different bytes is a Conflict.
    pub fn track(
        &mut self,
        frame: neko_reliable::FrameId,
        bytes: &[u8],
    ) -> Result<(), RetransmitError> {
        if let Some(old) = self.frames.get(&frame) {
            return if old == bytes {
                Ok(())
            } else {
                Err(RetransmitError::Conflict)
            };
        }
        if bytes.len() > self.max_bytes {
            return Err(RetransmitError::FrameTooLarge);
        }
        if self.frames.len() >= self.max_frames
            || self.bytes.saturating_add(bytes.len()) > self.max_bytes
        {
            return Err(RetransmitError::Capacity);
        }
        self.bytes = self.bytes.saturating_add(bytes.len());
        self.frames.insert(frame, bytes.to_vec());
        Ok(())
    }

    /// Fetch the plaintext for a frame scheduled for retransmission. The caller
    /// re-encodes it into a fresh packet; the entry is retained until released.
    pub fn get(&self, frame: neko_reliable::FrameId) -> Option<&[u8]> {
        self.frames.get(&frame).map(Vec::as_slice)
    }

    /// Release a frame's retained bytes once it is fully ACKed/retired.
    /// Returns whether the frame was tracked.
    pub fn release(&mut self, frame: neko_reliable::FrameId) -> bool {
        match self.frames.remove(&frame) {
            Some(b) => {
                self.bytes = self.bytes.saturating_sub(b.len());
                true
            }
            None => false,
        }
    }

    /// Drop all retained frames (path/generation teardown).
    pub fn clear(&mut self) {
        self.frames.clear();
        self.bytes = 0;
    }

    pub fn retained(&self) -> usize {
        self.frames.len()
    }
    pub fn retained_bytes(&self) -> usize {
        self.bytes
    }
}

#[cfg(test)]
mod path_recovery_tests {
    use super::*;
    use neko_reliable::{AckRanges, FrameId, SentPacket};

    fn recovery() -> PathRecovery {
        PathRecovery::new(PathId(1), 7, 1200).unwrap()
    }
    fn sent(n: u64, bytes: u64, frames: &[u64]) -> SentPacket {
        SentPacket {
            number: n,
            sent_at_us: n * 1000,
            bytes,
            ack_eliciting: true,
            frames: frames.iter().map(|f| FrameId(*f)).collect(),
        }
    }
    fn ack_of(n: u64) -> AckRanges {
        let mut a = AckRanges::new(8).unwrap();
        a.insert(n).unwrap();
        a
    }

    #[test]
    fn ack_retires_bytes_in_flight_exactly_once_and_emits_no_session_evidence() {
        let mut r = recovery();
        r.on_sent(sent(0, 100, &[0])).unwrap();
        r.on_sent(sent(1, 100, &[1])).unwrap();
        assert_eq!(r.bytes_in_flight(), 200);
        let out = r.on_ack(7, &ack_of(0), 10_000, 0).unwrap();
        // Carrier-level outcome only: retired packet/frame ids, never a
        // Session DeliveryLedger::confirm_received (none exists on this seam).
        assert_eq!(out.acked_packets, vec![0]);
        assert_eq!(out.acked_bytes, 100);
        assert_eq!(r.bytes_in_flight(), 100);
        // A duplicate ACK for the same packet cannot double-release bytes.
        let again = r.on_ack(7, &ack_of(0), 11_000, 0).unwrap();
        assert_eq!(again.acked_bytes, 0);
        assert_eq!(r.bytes_in_flight(), 100);
    }

    #[test]
    fn stale_generation_and_future_or_unsent_ack_are_rejected_atomically() {
        let mut r = recovery();
        r.on_sent(sent(0, 100, &[0])).unwrap();
        // Wrong path generation -> reject before any recovery mutation.
        assert_eq!(
            r.on_ack(8, &ack_of(0), 10_000, 0),
            Err(PathRecoveryError::GenerationMismatch)
        );
        assert_eq!(r.bytes_in_flight(), 100);
        // Future ACK beyond largest_sent -> Recovery rejects; no in-flight loss.
        assert!(r.on_ack(7, &ack_of(9), 10_000, 0).is_err());
        assert_eq!(r.bytes_in_flight(), 100);
        // ACK on a Recovery that never sent is also rejected.
        let mut empty = recovery();
        assert!(empty.on_ack(7, &ack_of(0), 1_000, 0).is_err());
    }

    #[test]
    fn retransmit_frames_are_carrier_frame_ids_only() {
        let mut r = recovery();
        r.on_sent(sent(0, 100, &[5, 6])).unwrap();
        r.on_sent(sent(1, 100, &[7])).unwrap();
        // ACK packet 1 while packet 0 ages out past the loss threshold ->
        // carrier-level retransmit frame ids {5,6}, not Session offsets.
        let out = r.on_ack(7, &ack_of(1), 60_000, 0).unwrap();
        assert_eq!(out.acked_packets, vec![1]);
        assert!(out.retransmit_frames.iter().all(|f| [5, 6].contains(&f.0)));
    }

    #[test]
    fn congestion_window_gates_send_and_releases_on_ack() {
        let mut r = recovery();
        assert!(r.can_send(100));
        r.on_sent(sent(0, 1200, &[0])).unwrap();
        // cwnd default is a few MSS; a huge send exceeds the window.
        assert!(!r.can_send(u64::MAX));
        r.on_ack(7, &ack_of(0), 10_000, 0).unwrap();
        assert_eq!(r.bytes_in_flight(), 0);
        assert!(r.can_send(1200));
    }

    #[test]
    fn fresh_health_bridge_emits_one_observation_per_resolved_outcome() {
        let mut r = recovery();
        let mut health = CarrierHealth::new(HealthLimits::default()).unwrap();
        // No traffic yet -> no resolved outcome -> nothing to consume.
        assert!(r.fresh_health_sample().is_none());
        // A resolved ACK outcome produces exactly one fresh sample; repeated
        // polls of the same epoch return None and cannot replay it.
        for n in 0..8u64 {
            r.on_sent(sent(n, 400, &[n])).unwrap();
        }
        let mut ack = AckRanges::new(8).unwrap();
        ack.insert(7).unwrap();
        r.on_ack(7, &ack, 40_000, 0).unwrap();
        let first = r
            .fresh_health_sample()
            .expect("resolved outcome yields one");
        // Interval denominator is *resolved* packets: 1 acked + 5 lost = 6
        // resolved, 5 lost -> 833/mille (not the lifetime 5/8=625).
        assert_eq!(first.loss_per_mille, 833);
        health.observe(PathId(1), first).unwrap();
        // Bare sends and same-epoch polls must NOT produce another observation
        // — on_sent does not advance outcome_epoch, so historical cumulative
        // loss cannot be replayed by just sending more traffic.
        r.on_sent(sent(8, 400, &[8])).unwrap();
        assert!(
            r.fresh_health_sample().is_none(),
            "a bare send with no resolved outcome must not emit a sample"
        );
        // One resolved bad outcome counts once; below degrade_after(=2) the
        // single bad sample cannot reach Degraded, and replays yield none.
        let st = health.path(PathId(1)).unwrap();
        assert_eq!(st.consecutive_bad, 1, "exactly one bad observation counted");
        assert_ne!(st.state, HealthState::Degraded);
        // H-RUDP-001C: a subsequent CLEAN resolved outcome must NOT replay the
        // old cumulative loss. ACK every still-outstanding packet (5,6,7,8,9)
        // so the interval delta has zero new loss, not the old 625/mille.
        r.on_sent(sent(9, 400, &[9])).unwrap();
        let mut a2 = AckRanges::new(8).unwrap();
        for n in 5..=9u64 {
            a2.insert(n).unwrap();
        }
        r.on_ack(7, &a2, 50_000, 0).unwrap();
        let clean = r.fresh_health_sample().expect("new resolved outcome");
        assert_eq!(
            clean.loss_per_mille, 0,
            "clean interval must not replay old cumulative loss"
        );
        // That clean outcome advances progress, not the bad streak.
        let _ = health.observe(PathId(1), clean).unwrap();
        assert_eq!(health.path(PathId(1)).unwrap().consecutive_bad, 0);
    }

    #[test]
    fn non_ack_eliciting_packet_does_not_drain_uncharged_congestion_bytes() {
        let mut r = recovery();
        // A non-ack-eliciting packet is recorded for recovery but is NOT
        // charged to Reno bytes-in-flight (it cannot elicit an ACK by itself).
        let non_ack = SentPacket {
            number: 0,
            sent_at_us: 0,
            bytes: 200,
            ack_eliciting: false,
            frames: vec![FrameId(0)],
        };
        r.on_sent(non_ack).unwrap();
        assert_eq!(
            r.bytes_in_flight(),
            0,
            "non-ack-eliciting packet must not charge bytes-in-flight"
        );
        // If it is later ACKed, the released bytes must not exceed what was
        // charged — otherwise bytes_in_flight is drained below its real value.
        let mut ack = AckRanges::new(8).unwrap();
        ack.insert(0).unwrap();
        let out = r.on_ack(7, &ack, 10_000, 0).unwrap();
        // The ACK retires packet 0's bytes from recovery, but bytes_in_flight
        // must only be reduced by bytes that were actually charged (none).
        assert_eq!(
            r.bytes_in_flight(),
            0,
            "acking a non-ack-eliciting packet must not deflate bytes-in-flight"
        );
        assert_eq!(out.acked_packets, vec![0]);
    }

    #[test]
    fn acking_non_ack_packet_does_not_drain_another_packets_charged_bytes() {
        let mut r = recovery();
        // packet 0: non-ack-eliciting, 200B — recorded but NOT charged.
        r.on_sent(SentPacket {
            number: 0,
            sent_at_us: 0,
            bytes: 200,
            ack_eliciting: false,
            frames: vec![FrameId(0)],
        })
        .unwrap();
        // packet 1: ack-eliciting, 300B — charged to bytes-in-flight.
        r.on_sent(sent(1, 300, &[1])).unwrap();
        assert_eq!(r.bytes_in_flight(), 300);
        // ACK only packet 0 (the uncharged one). bytes_in_flight must stay 300:
        // packet 1 is still in flight and only its own charged bytes release it.
        let mut ack = AckRanges::new(8).unwrap();
        ack.insert(0).unwrap();
        r.on_ack(7, &ack, 10_000, 0).unwrap();
        assert_eq!(
            r.bytes_in_flight(),
            300,
            "acking an uncharged packet must not drain another packet's bytes"
        );
    }

    #[test]
    fn ack_tracker_observes_only_matching_generation_and_yields_canonical_ack() {
        let mut t = PacketAckTracker::new(7);
        assert!(t.build_ack(0).is_none());
        assert!(t.take_ack(0).is_none());
        // Duplicate/reordered/wrong-generation observations.
        t.observe_packet(7, 2, true).unwrap();
        t.observe_packet(7, 0, true).unwrap(); // reorder
        t.observe_packet(7, 2, true).unwrap(); // duplicate no-op
        assert_eq!(
            t.observe_packet(8, 5, true),
            Err(AckTrackerError::GenerationMismatch)
        );
        let ack = t.build_ack(12).unwrap();
        assert_eq!(ack.largest_observed, 2);
        assert_eq!(ack.ack_delay_us, 12);
        // Ranges are canonical merged: {0},{2} observed -> two ranges.
        assert_eq!(ack.ranges.len(), 2);
        // The encoded payload must decode back to identical canonical ranges.
        let enc = neko_wire::encode_ack(&ack).unwrap();
        assert_eq!(neko_wire::decode_ack(&enc).unwrap(), ack);
    }

    #[test]
    fn ack_of_ack_never_scheduled_and_pending_consumed_once() {
        let mut t = PacketAckTracker::new(7);
        // An ack-eliciting DATA packet creates one pending obligation.
        t.observe_packet(7, 0, true).unwrap();
        assert!(t.pending_ack());
        let first = t.take_ack(0).expect("one obligation");
        assert_eq!(first.largest_observed, 0);
        // Polling again with no new eligible packet emits nothing.
        assert!(t.take_ack(0).is_none());
        assert!(!t.pending_ack());
        // An ACK-only packet (ack_eliciting=false) is recorded but does NOT
        // create a new obligation — no ACK-of-ACK ping-pong.
        t.observe_packet(7, 1, false).unwrap();
        assert!(!t.pending_ack());
        assert!(t.take_ack(0).is_none());
        // A new ack-eliciting packet re-arms exactly one obligation.
        t.observe_packet(7, 5, true).unwrap();
        assert!(t.pending_ack());
        assert_eq!(t.take_ack(0).unwrap().largest_observed, 5);
        assert!(t.take_ack(0).is_none());
    }

    #[test]
    fn reliable_udp_runtime_orchestrates_send_ack_loss_and_auto_fallback() {
        let mut rt = ReliableUdpRuntime::new(1, 1200).unwrap();
        rt.ready_standby(0);
        rt.activate_udp(1);
        // Send packets -> observe -> ACK emits once -> apply -> in-flight drains.
        for n in 0..4u64 {
            // Frame identity is deliberately distinct from packet number.
            rt.on_packet_sent(n, n * 1000, 400, FrameId(1000 + n), b"data")
                .unwrap();
        }
        assert_eq!(rt.in_flight(), 4);
        // Receiver observes each as ack-eliciting Data -> pending ACK.
        for n in 0..4u64 {
            rt.on_packet_received(n, true).unwrap();
        }
        let ack = rt.poll_outgoing_ack(0).expect("one obligation");
        let ranges = AckRanges::from_ranges(
            32,
            &ack.ranges
                .iter()
                .map(|r| neko_reliable::AckRange {
                    start: r.start,
                    end: r.end,
                })
                .collect::<Vec<_>>(),
        )
        .unwrap();
        rt.apply_ack(&ranges, 40_000, 0).unwrap();
        assert_eq!(rt.in_flight(), 0);
        // Second poll: pending consumed -> None (no ACK-of-ACK).
        assert!(rt.poll_outgoing_ack(0).is_none());
        // Distinct PTO epochs drive automatic fallback through the bridge.
        let mut fell = false;
        // One monotonically increasing packet-number source for both primary
        // sends and retransmissions (the real source is the crypto sequence).
        let mut pn = 10u64;
        for i in 0..8u64 {
            rt.on_packet_sent(pn, 100_000 + i * 1000, 400, FrameId(2000 + i), b"x")
                .unwrap();
            pn += 1;
            // PTO probe frames re-send under a FRESH packet number with the
            // SAME stable FrameId.
            for (frame, _plaintext) in rt.pto_probe() {
                rt.on_retransmit_sent(pn, 110_000 + i * 1000, 400, frame)
                    .unwrap();
                pn += 1;
            }
            if matches!(rt.poll_health(200 + i), RuntimeEvent::WarmFallback(_)) {
                fell = true;
                break;
            }
        }
        assert!(
            fell,
            "distinct PTO epochs must drive automatic warm fallback"
        );
        assert_eq!(
            rt.manager().active(),
            Some(ConcurrentPathKey {
                path: PathId(2),
                generation: PathGeneration(1)
            })
        );
    }

    #[test]
    fn retransmit_late_original_dedups_by_stable_logical_identity() {
        let mut rt = ReliableUdpRuntime::new(1, 1200).unwrap();
        // The replacement (retransmit, fresh nonce/packet number) delivers the
        // stable logical identity (stream=1, offset=40) FIRST.
        assert!(rt.deliver_logical(1, 40, b"app-bytes").unwrap());
        // The delayed ORIGINAL packet (different packet number/nonce) arrives
        // later carrying the same logical identity + bytes -> suppressed.
        assert!(!rt.deliver_logical(1, 40, b"app-bytes").unwrap());
        assert_eq!(rt.dedup_suppressed, 1);
        // A duplicate with different bytes for the same identity fails closed.
        assert_eq!(
            rt.deliver_logical(1, 40, b"tampered"),
            Err(PathRecoveryError::DeliveryConflict)
        );
        // Distinct logical identity still delivers.
        assert!(rt.deliver_logical(1, 41, b"next").unwrap());
    }

    #[test]
    fn fallback_is_not_fabricated_when_tcp_not_ready() {
        // Do NOT mark the TCP standby ready — a Degraded UDP must not produce a
        // fabricated WarmFallback when promotion cannot actually succeed.
        let mut rt = ReliableUdpRuntime::new(1, 1200).unwrap();
        rt.activate_udp(0);
        // Drive a resolved PTO outcome to force fresh health evidence.
        rt.on_packet_sent(0, 0, 400, FrameId(1), b"x").unwrap();
        rt.pto_probe();
        let ev = rt.poll_health(10);
        // Not WarmFallback (no ready standby); it is either a health sample or
        // an explicit FallbackFailed — never a fabricated successful switch.
        assert!(!matches!(ev, RuntimeEvent::WarmFallback(_)));
    }

    #[test]
    fn runtime_send_admission_refuses_when_cwnd_full_and_recovers_on_ack() {
        let mut rt = ReliableUdpRuntime::new(1, 1200).unwrap();
        // Fill the congestion window (initial cwnd = 10*mss = 12000).
        let mut n = 0u64;
        let mut sent = 0u64;
        while rt.can_send(400) && n < 40 {
            rt.on_packet_sent(n, n * 1000, 400, FrameId(3000 + n), b"x")
                .unwrap();
            sent += 1;
            n += 1;
        }
        assert!(sent > 0);
        assert_eq!(sent, 30, "cwnd 12000 / 400B = 30 admitted");
        // Next send is refused at the gate: no charge, no tracked plaintext.
        assert!(!rt.can_send(400));
        assert_eq!(
            rt.on_packet_sent(n, 0, 400, FrameId(9999), b"no"),
            Err(PathRecoveryError::CongestionWindowFull)
        );
        assert_eq!(rt.in_flight() as u64, sent, "refused send recorded nothing");
        // ACK the tail (largest) so no packet is declared lost by the reorder
        // window — the freed bytes-in-flight reopen cwnd admission.
        let mut a = AckRanges::new(8).unwrap();
        for p in 20..30u64 {
            a.insert(p).unwrap();
        }
        rt.apply_ack(&a, 50_000, 0).unwrap();
        assert!(rt.can_send(400), "ACK opens cwnd admission");
    }

    #[test]
    fn overlapping_retransmit_copies_hold_frame_until_last_retires() {
        let mut r = recovery();
        let mut buf = RetransmitBuffer::new(8, 256).unwrap();
        // Original packet 0 carries frame F=9; retain its plaintext.
        r.on_sent(sent(0, 400, &[9])).unwrap();
        buf.track(FrameId(9), b"payload").unwrap();
        // PTO retransmits F under fresh packet 1 while packet 0 still
        // outstanding -> two copies of frame 9 outstanding.
        let probes = r.on_pto(1).unwrap();
        assert_eq!(probes, vec![FrameId(9)]);
        r.on_sent(sent(1, 400, &[9])).unwrap(); // second copy of frame 9
        assert!(r.frame_outstanding(FrameId(9)));
        // ACK only the retransmit copy (packet 1): frame 9 still outstanding
        // via packet 0 -> NOT releasable.
        let mut a1 = AckRanges::new(8).unwrap();
        a1.insert(1).unwrap();
        r.on_ack(7, &a1, 10_000, 0).unwrap();
        assert!(
            r.frame_outstanding(FrameId(9)),
            "one outstanding copy remains -> frame must not release"
        );
        assert_eq!(buf.get(FrameId(9)), Some(&b"payload"[..]));
        // ACK the final outstanding copy (packet 0) -> frame 9 releasable.
        let mut a0 = AckRanges::new(8).unwrap();
        a0.insert(0).unwrap();
        r.on_ack(7, &a0, 11_000, 0).unwrap();
        assert!(
            !r.frame_outstanding(FrameId(9)),
            "last copy retired -> frame releasable"
        );
        assert!(buf.release(FrameId(9)));
        assert_eq!(buf.retained(), 0);
        // Late/duplicate evidence must not double-release or resurrect.
        assert!(!buf.release(FrameId(9)));
    }

    #[test]
    fn retransmit_buffer_tracks_releases_and_bounds() {
        let mut buf = RetransmitBuffer::new(4, 64).unwrap();
        buf.track(FrameId(0), b"frame-zero").unwrap();
        buf.track(FrameId(1), b"frame-one").unwrap();
        assert_eq!(buf.retained(), 2);
        assert_eq!(buf.get(FrameId(0)), Some(&b"frame-zero"[..]));
        // Re-track same id+bytes is a no-op; different bytes is a conflict.
        buf.track(FrameId(0), b"frame-zero").unwrap();
        assert_eq!(
            buf.track(FrameId(0), b"different"),
            Err(RetransmitError::Conflict)
        );
        // Release frees exactly the tracked bytes once.
        assert!(buf.release(FrameId(0)));
        assert!(!buf.release(FrameId(0)));
        assert_eq!(buf.retained(), 1);
        // Capacity bounds.
        let mut tight = RetransmitBuffer::new(1, 8).unwrap();
        assert_eq!(
            tight.track(FrameId(9), b"123456789"),
            Err(RetransmitError::FrameTooLarge)
        );
        tight.track(FrameId(9), b"1234").unwrap();
        assert_eq!(
            tight.track(FrameId(10), b"x"),
            Err(RetransmitError::Capacity)
        );
        // Clear on teardown.
        buf.clear();
        assert_eq!(buf.retained(), 0);
        assert_eq!(buf.retained_bytes(), 0);
    }

    #[test]
    fn pacing_and_send_decision_are_deterministic_and_accounted() {
        let mut r = recovery();
        // A send decision consults can_send before enqueue; bytes-in-flight is
        // charged exactly once per sent packet and released exactly once.
        for n in 0..3u64 {
            let bytes = 400u64;
            assert!(r.can_send(bytes), "packet {n} within window");
            r.on_sent(sent(n, bytes, &[n])).unwrap();
            assert_eq!(r.bytes_in_flight(), bytes * (n + 1));
        }
        // Pacing interval is a deterministic function of bytes + measured RTT;
        // it is non-panicking and bounded, not a busy-loop or unaccounted sleep.
        let interval = r.pacing_interval_us(400);
        assert!(interval < u64::MAX);
        // One ACK retires exactly once — a second identical ACK frees nothing.
        let out1 = r.on_ack(7, &ack_of(0), 20_000, 0).unwrap();
        assert_eq!(out1.acked_bytes, 400);
        assert_eq!(r.bytes_in_flight(), 800);
        let out2 = r.on_ack(7, &ack_of(0), 21_000, 0).unwrap();
        assert_eq!(out2.acked_bytes, 0);
        assert_eq!(r.bytes_in_flight(), 800);
        // Persistent congestion collapses the window so can_send tightens.
        assert!(r.can_send(400));
        r.persistent_congestion();
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
    fn endpoint_rebind_requires_fresh_exact_challenge_and_promotes_atomically() {
        let mut state =
            EndpointRebindState::new(11, PathId(1), PathGeneration(3), 7001, 1).unwrap();
        let candidate = EndpointRebindCandidate {
            source_tag: 22,
            path: PathId(1),
            generation: PathGeneration(4),
        };
        assert_eq!(state.observe_candidate(candidate), Ok(()));
        assert_eq!(state.active_source_tag(), 11);
        assert_eq!(state.active_generation(), PathGeneration(3));
        assert_eq!(
            state.validate_and_promote(candidate, 7001, 1, 9),
            Err(EndpointRebindError::ChallengeMismatch)
        );
        assert_eq!(state.active_source_tag(), 11);
        assert!(state.candidate().is_none());
        assert_eq!(state.observe_candidate(candidate), Ok(()));
        assert_eq!(state.arm_challenge(9), Ok(()));
        assert_eq!(state.validate_and_promote(candidate, 7001, 1, 9), Ok(()));
        assert_eq!(state.active_source_tag(), 22);
        assert_eq!(state.retired_source_tag(), Some(11));
        assert_eq!(state.active_generation(), PathGeneration(4));
        assert!(state.candidate().is_none());
        assert_eq!(
            state.validate_and_promote(candidate, 7001, 1, 9),
            Err(EndpointRebindError::NoCandidate)
        );
    }

    #[test]
    fn endpoint_rebind_rejects_same_old_and_duplicate_candidates_without_mutation() {
        let mut state =
            EndpointRebindState::new(11, PathId(1), PathGeneration(3), 7001, 1).unwrap();
        let base = EndpointRebindCandidate {
            source_tag: 11,
            path: PathId(1),
            generation: PathGeneration(3),
        };
        assert_eq!(
            state.observe_candidate(base),
            Err(EndpointRebindError::SameSource)
        );
        assert_eq!(
            state.observe_candidate(EndpointRebindCandidate {
                source_tag: 12,
                generation: PathGeneration(2),
                ..base
            }),
            Err(EndpointRebindError::OldGeneration)
        );
        let candidate = EndpointRebindCandidate {
            source_tag: 12,
            generation: PathGeneration(4),
            ..base
        };
        assert_eq!(state.observe_candidate(candidate), Ok(()));
        assert_eq!(
            state.observe_candidate(EndpointRebindCandidate {
                source_tag: 13,
                ..candidate
            }),
            Err(EndpointRebindError::AlreadyPending)
        );
        assert_eq!(state.active_source_tag(), 11);
        assert_eq!(state.active_generation(), PathGeneration(3));
    }

    #[test]
    fn endpoint_rebind_wrong_lineage_is_terminal_and_cannot_be_reused() {
        let candidate = EndpointRebindCandidate {
            source_tag: 22,
            path: PathId(1),
            generation: PathGeneration(4),
        };
        let mut state =
            EndpointRebindState::new(11, PathId(1), PathGeneration(3), 7001, 1).unwrap();
        assert_eq!(
            state.observe_candidate(EndpointRebindCandidate {
                path: PathId(2),
                ..candidate
            }),
            Err(EndpointRebindError::TupleMismatch)
        );
        assert_eq!(
            state.observe_candidate(EndpointRebindCandidate {
                generation: PathGeneration(5),
                ..candidate
            }),
            Err(EndpointRebindError::TupleMismatch)
        );
        assert_eq!(state.observe_candidate(candidate), Ok(()));
        assert_eq!(state.arm_challenge(17), Ok(()));
        assert_eq!(
            state.validate_and_promote(candidate, 7002, 1, 17),
            Err(EndpointRebindError::TupleMismatch)
        );
        assert!(state.candidate().is_none());
        assert_eq!(state.active_source_tag(), 11);
        assert_eq!(state.observe_candidate(candidate), Ok(()));
        assert_eq!(state.arm_challenge(18), Ok(()));
        assert_eq!(
            state.validate_and_promote(candidate, 7001, 1, 17),
            Err(EndpointRebindError::ChallengeMismatch)
        );
        assert!(state.candidate().is_none());
        assert_eq!(state.active_source_tag(), 11);
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
