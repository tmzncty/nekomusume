//! Boundary for the provisional Nekomusume wire model.
//!
//! M0 deliberately contains no codec, socket, or cryptographic implementation.

/// Marker for the not-yet-frozen v0 wire boundary.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct WireBoundary;
