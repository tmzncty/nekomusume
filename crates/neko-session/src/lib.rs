//! Boundary for logical Session identity and delivery state.
//!
//! M0 deliberately contains no state machine or cryptographic implementation.

/// Marker for the logical Session boundary.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct SessionBoundary;
