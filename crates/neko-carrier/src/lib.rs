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
pub enum CarrierError {
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

    pub fn apply(&mut self, event: CarrierEvent) -> Result<(), CarrierError> {
        match event {
            CarrierEvent::PathAdded {
                path,
                carrier,
                generation,
            } => {
                if self.paths.contains_key(&path) {
                    return Err(CarrierError::PathExists);
                }
                if self.paths.len() >= self.limits.max_paths {
                    return Err(CarrierError::ResourceLimit);
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
                    return Err(CarrierError::InvalidTransition);
                }
                p.validation = PathValidationState::Validating;
                p.state = PathState::Validating;
            }
            CarrierEvent::ChallengeValidated(v) => {
                let p = self.checked(v.path, v.generation)?;
                if p.validation != PathValidationState::Validating {
                    return Err(CarrierError::InvalidTransition);
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
                    return Err(CarrierError::InvalidTransition);
                }
                p.state = PathState::Draining;
            }
            CarrierEvent::Fail { path, generation } => {
                let p = self.checked(path, generation)?;
                if p.state != PathState::Degraded && p.state != PathState::Draining {
                    return Err(CarrierError::InvalidTransition);
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
                    return Err(CarrierError::ActivePathConflict);
                }
                {
                    let p = self.checked(path, generation)?;
                    if p.validation != PathValidationState::Validated {
                        return Err(CarrierError::ValidationRequired);
                    }
                    if p.state != PathState::Candidate {
                        return Err(CarrierError::InvalidTransition);
                    }
                    if p.successes < k_successes || p.dwell_events < min_dwell_events {
                        return Err(CarrierError::HysteresisGate);
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
    ) -> Result<&mut PathRecord, CarrierError> {
        let p = self
            .paths
            .get_mut(&path)
            .ok_or(CarrierError::PathNotFound)?;
        if generation != p.generation {
            return if generation.0 < p.generation.0 {
                Err(CarrierError::OldGeneration)
            } else {
                Err(CarrierError::GenerationMismatch)
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
            Err(CarrierError::OldGeneration)
        );
        assert_eq!(
            s.apply(CarrierEvent::SessionDelivery(SessionDelivery {
                path: P,
                generation: PathGeneration(0)
            })),
            Err(CarrierError::OldGeneration)
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
            Err(CarrierError::HysteresisGate)
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
        assert_eq!(s.apply(event), Err(CarrierError::PathExists));
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
            Err(CarrierError::InvalidTransition)
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
            Err(CarrierError::ResourceLimit)
        );
    }
}

/// Synchronous, non-blocking opaque-byte carrier contract.
pub trait Carrier {
    fn kind(&self) -> CarrierKind;
    fn properties(&self) -> CarrierProperties;
    fn limits(&self) -> MemoryLimits;
    fn send(&self, message: &[u8]) -> Result<(), MemoryPairError>;
    fn recv(&self) -> Result<Option<Vec<u8>>, MemoryPairError>;
    fn close(&self) -> Result<(), MemoryPairError>;
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
    fn limits(&self) -> MemoryLimits {
        self.shared.limits
    }
    fn send(&self, message: &[u8]) -> Result<(), MemoryPairError> {
        let limits = self.shared.limits;
        if message.len() > limits.max_message_bytes {
            return Err(MemoryPairError::MessageTooLarge);
        }
        // One pair mutex linearizes local close, peer close, capacity, and enqueue.
        let mut state = self.lock()?;
        if state.closed[self.side] {
            return Err(MemoryPairError::Closed);
        }
        let peer = 1 - self.side;
        if state.closed[peer] {
            return Err(MemoryPairError::PeerClosed);
        }
        let total = state.queue_bytes[peer]
            .checked_add(message.len())
            .ok_or(MemoryPairError::ArithmeticOverflow)?;
        if total > limits.max_queue_bytes {
            return Err(MemoryPairError::QueueFull);
        }
        state.queues[peer].push_back(message.to_vec());
        state.queue_bytes[peer] = total;
        Ok(())
    }
    fn recv(&self) -> Result<Option<Vec<u8>>, MemoryPairError> {
        let mut state = self.lock()?;
        let message = state.queues[self.side].pop_front();
        if let Some(ref bytes) = message {
            state.queue_bytes[self.side] = state.queue_bytes[self.side]
                .checked_sub(bytes.len())
                .ok_or(MemoryPairError::ArithmeticOverflow)?;
        }
        Ok(message)
    }
    fn close(&self) -> Result<(), MemoryPairError> {
        let mut state = self.lock()?;
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
        assert_eq!(a.send(b"567"), Err(MemoryPairError::QueueFull));
        assert_eq!(b.recv().unwrap(), Some(b"1234".to_vec()));
        assert_eq!(a.send(b"567"), Ok(()));
        assert_eq!(a.send(b"12345"), Err(MemoryPairError::MessageTooLarge));
    }

    #[test]
    fn close_is_idempotent_and_preserves_queued_data() {
        let (a, b) = pair();
        a.send(b"data").unwrap();
        a.close().unwrap();
        a.close().unwrap();
        assert_eq!(a.send(b"x"), Err(MemoryPairError::Closed));
        // The peer drains data queued before close, then observes an empty queue.
        assert_eq!(b.recv().unwrap(), Some(b"data".to_vec()));
        assert_eq!(b.recv().unwrap(), None);

        b.close().unwrap();
        b.close().unwrap();
        assert_eq!(b.send(b"x"), Err(MemoryPairError::Closed));
        let (open, closed_peer) = pair();
        closed_peer.close().unwrap();
        assert_eq!(open.send(b"x"), Err(MemoryPairError::PeerClosed));
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
            MemoryLimits {
                max_message_bytes: 4,
                max_queue_bytes: 6
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
        self.consecutive_ptos = 0
    }
    pub fn udp_pto(&mut self) -> bool {
        if self.active != ActiveCarrier::Udp {
            return false;
        }
        self.consecutive_ptos = self.consecutive_ptos.saturating_add(1);
        if self.consecutive_ptos >= self.hard_failure_ptos {
            self.active = ActiveCarrier::Tcp;
            self.metrics.switches = self.metrics.switches.saturating_add(1);
            self.metrics.recovery_events = self.metrics.recovery_events.saturating_add(1);
            true
        } else {
            false
        }
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
        a.close().unwrap();
        a.close().unwrap();
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
