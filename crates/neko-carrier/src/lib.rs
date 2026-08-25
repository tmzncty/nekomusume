//! Pure carrier path evidence model.
//!
//! This is an M0 candidate: it opens no sockets, performs no routing or
//! tunnelling, and never performs a real failover.  Evidence domains are
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
                if self.paths.len() >= self.limits.max_paths {
                    return Err(CarrierError::ResourceLimit);
                }
                if self.paths.contains_key(&path) {
                    return Err(CarrierError::PathExists);
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
