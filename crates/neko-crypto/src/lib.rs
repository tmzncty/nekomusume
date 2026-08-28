//! Bounded cryptographic/session primitives for local research.
//!
//! This crate deliberately exposes no listener, runtime, key loading, or
//! production configuration.  It provides the small fail-closed state pieces
//! that a future Noise session must compose.

use snow::params::NoiseParams;

pub const SNOW_VERSION: &str = "0.10.0";
pub const MAX_REPLAY_WINDOW: u64 = 64;

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum CryptoError {
    NonceExhausted,
    Replay,
    TooOld,
    InvalidWindow,
}

/// Direction-local monotonically increasing nonce counter. Counter exhaustion
/// is terminal: it never wraps and never returns a reused nonce.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct NonceManager {
    next: u64,
    exhausted: bool,
}
impl NonceManager {
    pub const fn new(start: u64) -> Self {
        Self {
            next: start,
            exhausted: false,
        }
    }
    pub const fn next_value(&self) -> u64 {
        self.next
    }
    pub const fn is_exhausted(&self) -> bool {
        self.exhausted
    }
    pub fn next_nonce(&mut self) -> Result<u64, CryptoError> {
        if self.exhausted {
            return Err(CryptoError::NonceExhausted);
        }
        let value = self.next;
        match self.next.checked_add(1) {
            Some(next) => self.next = next,
            None => self.exhausted = true,
        }
        Ok(value)
    }
}

/// Bounded sliding replay window. Authentication must be performed by the
/// caller before `accept`; this type only tracks authenticated sequence IDs.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct ReplayWindow {
    width: u64,
    highest: Option<u64>,
    seen: u64,
}
impl ReplayWindow {
    pub fn new(width: u64) -> Result<Self, CryptoError> {
        if width == 0 || width > MAX_REPLAY_WINDOW {
            return Err(CryptoError::InvalidWindow);
        }
        Ok(Self {
            width,
            highest: None,
            seen: 0,
        })
    }
    pub const fn highest(&self) -> Option<u64> {
        self.highest
    }
    pub fn accept(&mut self, sequence: u64) -> Result<(), CryptoError> {
        let Some(highest) = self.highest else {
            self.highest = Some(sequence);
            self.seen = 1;
            return Ok(());
        };
        if sequence > highest {
            let shift = sequence - highest;
            self.seen = if shift >= self.width {
                1
            } else {
                (self.seen << shift) | 1
            };
            self.highest = Some(sequence);
            return Ok(());
        }
        let distance = highest - sequence;
        if distance >= self.width {
            return Err(CryptoError::TooOld);
        }
        let bit = 1u64 << distance;
        if self.seen & bit != 0 {
            return Err(CryptoError::Replay);
        }
        self.seen |= bit;
        Ok(())
    }
}

/// The only handshake pattern admitted by this research boundary.
pub fn noise_ik_params() -> NoiseParams {
    "Noise_IK_25519_ChaChaPoly_SHA256"
        .parse()
        .expect("constant Noise params")
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn nonce_is_direction_local_and_fails_closed_at_wrap() {
        let mut n = NonceManager::new(u64::MAX - 1);
        assert_eq!(n.next_nonce(), Ok(u64::MAX - 1));
        assert_eq!(n.next_nonce(), Ok(u64::MAX));
        assert_eq!(n.next_nonce(), Err(CryptoError::NonceExhausted));
        assert_eq!(n.next_nonce(), Err(CryptoError::NonceExhausted));
        assert!(n.is_exhausted());
    }

    #[test]
    fn replay_window_rejects_duplicates_and_old_values() {
        let mut w = ReplayWindow::new(4).unwrap();
        assert_eq!(w.accept(10), Ok(()));
        assert_eq!(w.accept(10), Err(CryptoError::Replay));
        assert_eq!(w.accept(9), Ok(()));
        assert_eq!(w.accept(9), Err(CryptoError::Replay));
        assert_eq!(w.accept(5), Err(CryptoError::TooOld));
        assert_eq!(w.accept(14), Ok(()));
        assert_eq!(w.accept(10), Err(CryptoError::TooOld));
    }

    #[test]
    fn replay_window_shift_discards_only_outside_window() {
        let mut w = ReplayWindow::new(4).unwrap();
        w.accept(1).unwrap();
        w.accept(2).unwrap();
        w.accept(5).unwrap();
        assert_eq!(w.accept(4), Ok(()));
        assert_eq!(w.accept(1), Err(CryptoError::TooOld));
    }

    #[test]
    fn invalid_replay_windows_are_rejected() {
        assert_eq!(ReplayWindow::new(0), Err(CryptoError::InvalidWindow));
        assert_eq!(
            ReplayWindow::new(MAX_REPLAY_WINDOW + 1),
            Err(CryptoError::InvalidWindow)
        );
        assert_eq!(noise_ik_params().name, "Noise_IK_25519_ChaChaPoly_SHA256");
    }
}
