//! Small, explicit service lifecycle used by the bounded CLI runners.
use std::sync::{
    Arc,
    atomic::{AtomicU8, Ordering},
};

#[derive(Clone, Copy, Debug, Eq, PartialEq)]
#[repr(u8)]
pub enum State {
    Starting = 0,
    Ready = 1,
    Draining = 2,
    Stopped = 3,
    Failed = 4,
}
impl State {
    pub const fn as_str(self) -> &'static str {
        match self {
            Self::Starting => "STARTING",
            Self::Ready => "READY",
            Self::Draining => "DRAINING",
            Self::Stopped => "STOPPED",
            Self::Failed => "FAILED",
        }
    }
}

#[derive(Clone, Copy, Debug, Eq, PartialEq)]
#[repr(u8)]
pub enum ReadinessPrerequisite {
    ConfigurationAccepted = 1 << 0,
    IdentityInitialized = 1 << 1,
    TrustPolicyInitialized = 1 << 2,
    SocketBound = 1 << 3,
    IoConfigured = 1 << 4,
}

const ALL_PREREQUISITES: u8 = ReadinessPrerequisite::ConfigurationAccepted as u8
    | ReadinessPrerequisite::IdentityInitialized as u8
    | ReadinessPrerequisite::TrustPolicyInitialized as u8
    | ReadinessPrerequisite::SocketBound as u8
    | ReadinessPrerequisite::IoConfigured as u8;

#[derive(Clone)]
pub struct Lifecycle {
    state: Arc<AtomicU8>,
    prerequisites: Arc<AtomicU8>,
}
impl Lifecycle {
    pub fn new() -> Self {
        Self {
            state: Arc::new(AtomicU8::new(State::Starting as u8)),
            prerequisites: Arc::new(AtomicU8::new(0)),
        }
    }
    pub fn state(&self) -> State {
        match self.state.load(Ordering::Acquire) {
            1 => State::Ready,
            2 => State::Draining,
            3 => State::Stopped,
            4 => State::Failed,
            _ => State::Starting,
        }
    }
    pub fn satisfy(&self, prerequisite: ReadinessPrerequisite) {
        self.prerequisites
            .fetch_or(prerequisite as u8, Ordering::AcqRel);
    }
    pub fn readiness(&self) -> bool {
        self.prerequisites.load(Ordering::Acquire) == ALL_PREREQUISITES
            && self.state() == State::Ready
    }
    pub fn finalize_readiness(&self) -> Result<(), ()> {
        if self.prerequisites.load(Ordering::Acquire) == ALL_PREREQUISITES {
            self.state.store(State::Ready as u8, Ordering::Release);
            Ok(())
        } else {
            self.state.store(State::Failed as u8, Ordering::Release);
            Err(())
        }
    }
    pub fn drain(&self) {
        let _ = self.state.compare_exchange(
            State::Ready as u8,
            State::Draining as u8,
            Ordering::AcqRel,
            Ordering::Acquire,
        );
    }
    pub fn stopped(&self) {
        self.state.store(State::Stopped as u8, Ordering::Release);
    }
    pub fn failed(&self) {
        self.state.store(State::Failed as u8, Ordering::Release);
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    const PREREQUISITES: [ReadinessPrerequisite; 5] = [
        ReadinessPrerequisite::ConfigurationAccepted,
        ReadinessPrerequisite::IdentityInitialized,
        ReadinessPrerequisite::TrustPolicyInitialized,
        ReadinessPrerequisite::SocketBound,
        ReadinessPrerequisite::IoConfigured,
    ];

    #[test]
    fn listener_is_rebindable_after_shutdown() {
        let listener = std::net::TcpListener::bind("127.0.0.1:0").unwrap();
        let address = listener.local_addr().unwrap();
        drop(listener);
        let rebound = std::net::TcpListener::bind(address).unwrap();
        drop(rebound);
    }

    #[test]
    fn every_named_readiness_prerequisite_is_required() {
        for missing in PREREQUISITES {
            let lifecycle = Lifecycle::new();
            for prerequisite in PREREQUISITES {
                if prerequisite != missing {
                    lifecycle.satisfy(prerequisite);
                }
            }
            assert_eq!(lifecycle.finalize_readiness(), Err(()));
            assert_eq!(lifecycle.state(), State::Failed);
            assert!(!lifecycle.readiness());
        }
    }

    #[test]
    fn duplicate_prerequisite_is_idempotent() {
        let lifecycle = Lifecycle::new();
        for prerequisite in PREREQUISITES {
            lifecycle.satisfy(prerequisite);
            lifecycle.satisfy(prerequisite);
        }
        assert_eq!(lifecycle.finalize_readiness(), Ok(()));
        assert!(lifecycle.readiness());
        lifecycle.drain();
        assert_eq!(lifecycle.state(), State::Draining);
        lifecycle.stopped();
        assert_eq!(lifecycle.state(), State::Stopped);
    }
}
