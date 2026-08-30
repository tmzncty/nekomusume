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

#[derive(Clone)]
pub struct Lifecycle {
    state: Arc<AtomicU8>,
    ready: Arc<AtomicU8>,
}
impl Lifecycle {
    pub fn new() -> Self {
        Self {
            state: Arc::new(AtomicU8::new(State::Starting as u8)),
            ready: Arc::new(AtomicU8::new(0)),
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
    pub fn mark_ready(&self) {
        self.ready.fetch_add(1, Ordering::AcqRel);
    }
    pub fn readiness(&self) -> bool {
        self.ready.load(Ordering::Acquire) >= 5 && self.state() == State::Ready
    }
    pub fn finalize_readiness(&self) {
        if self.ready.load(Ordering::Acquire) >= 5 {
            self.state.store(State::Ready as u8, Ordering::Release);
        } else {
            self.state.store(State::Failed as u8, Ordering::Release);
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
    #[test]
    fn listener_is_rebindable_after_shutdown() {
        let listener = std::net::TcpListener::bind("127.0.0.1:0").unwrap();
        let address = listener.local_addr().unwrap();
        drop(listener);
        let rebound = std::net::TcpListener::bind(address).unwrap();
        drop(rebound);
    }

    #[test]
    fn readiness_requires_all_prerequisites() {
        let l = Lifecycle::new();
        for _ in 0..4 {
            l.mark_ready();
        }
        l.finalize_readiness();
        assert_eq!(l.state(), State::Failed);
        let l = Lifecycle::new();
        for _ in 0..5 {
            l.mark_ready();
        }
        l.finalize_readiness();
        assert!(l.readiness());
        l.drain();
        assert_eq!(l.state(), State::Draining);
        l.stopped();
        assert_eq!(l.state(), State::Stopped);
    }
}
