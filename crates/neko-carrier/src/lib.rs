//! Boundary for carrier-neutral path abstractions.
//!
//! M0 deliberately opens no UDP/TCP socket and implements no failover.

/// Marker for the carrier boundary.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct CarrierBoundary;
