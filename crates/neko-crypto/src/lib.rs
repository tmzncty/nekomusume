//! Bounded research crypto boundary.
//!
//! The concrete handshake and record API are intentionally introduced only
//! with tests and explicit domain/nonce/replay invariants.

/// Dependency-review marker for the selected Noise implementation.
pub const SNOW_VERSION: &str = "0.10.0";
