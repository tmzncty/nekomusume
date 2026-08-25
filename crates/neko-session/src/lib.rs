//! Pure synchronous candidate delivery ledger and state machine.
//!
//! This models logical delivery evidence only. It opens no connection, uses no
//! runtime or cryptography, and does not implement carrier failover.
use std::collections::BTreeMap;

pub const DEFAULT_MAX_REORDER: u64 = 64;
pub const DEFAULT_MAX_STREAMS: usize = 64;
pub const DEFAULT_MAX_CONNECTION_BYTES: usize = 1 << 20;
pub const DEFAULT_MAX_OFFSET_JUMP: u64 = 1 << 20;

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct DeliveryEpoch(pub u64);
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct KeyPhase(pub u8);
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct PathGeneration(pub u64);

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum DeliveryState {
    Unsent,
    InFlight,
    Uncertain,
    Confirmed,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct SessionContext {
    pub delivery_epoch: DeliveryEpoch,
    pub key_phase: KeyPhase,
    pub path_generation: PathGeneration,
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct DeliverySegment {
    pub stream_id: u64,
    pub offset: u64,
    pub data: Vec<u8>,
    pub state: DeliveryState,
    pub context: SessionContext,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct Limits {
    pub max_reorder: u64,
    pub max_streams: usize,
    pub max_connection_bytes: usize,
    pub max_offset_jump: u64,
}
impl Default for Limits {
    fn default() -> Self {
        Self {
            max_reorder: DEFAULT_MAX_REORDER,
            max_streams: DEFAULT_MAX_STREAMS,
            max_connection_bytes: DEFAULT_MAX_CONNECTION_BYTES,
            max_offset_jump: DEFAULT_MAX_OFFSET_JUMP,
        }
    }
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub enum LedgerError {
    EmptyRange,
    TooManyStreams,
    ConnectionLimit,
    OffsetJump,
    ReorderLimit,
    OffsetOverflow,
    Conflict,
    OldEpoch,
    InvalidTransition,
    RangeNotFound,
}

#[derive(Debug, Default)]
pub struct DeliveryLedger {
    limits: Limits,
    segments: BTreeMap<(u64, u64), DeliverySegment>,
    streams: BTreeMap<u64, u64>,
    bytes: usize,
    context: Option<SessionContext>,
}
impl DeliveryLedger {
    pub fn new(limits: Limits) -> Self {
        Self {
            limits,
            ..Self::default()
        }
    }
    pub fn segments(&self) -> impl Iterator<Item = &DeliverySegment> {
        self.segments.values()
    }
    pub fn watermark(&self, stream: u64) -> u64 {
        self.streams.get(&stream).copied().unwrap_or(0)
    }
    pub fn insert(
        &mut self,
        stream: u64,
        offset: u64,
        data: &[u8],
        context: SessionContext,
    ) -> Result<DeliveryState, LedgerError> {
        if data.is_empty() {
            return Err(LedgerError::EmptyRange);
        }
        let end = offset
            .checked_add(data.len() as u64)
            .ok_or(LedgerError::OffsetOverflow)?;
        if let Some(old) = self.context {
            if context.delivery_epoch != old.delivery_epoch {
                return Err(LedgerError::OldEpoch);
            }
        } else {
            self.context = Some(context);
        }
        if !self.streams.contains_key(&stream) && self.streams.len() >= self.limits.max_streams {
            return Err(LedgerError::TooManyStreams);
        }
        let watermark = self.watermark(stream);
        if offset > watermark.saturating_add(self.limits.max_offset_jump) {
            return Err(LedgerError::OffsetJump);
        }
        if offset > watermark.saturating_add(self.limits.max_reorder) {
            return Err(LedgerError::ReorderLimit);
        }
        for segment in self.segments.values() {
            let old_end = segment.offset + segment.data.len() as u64;
            if segment.stream_id == stream && offset < old_end && segment.offset < end {
                let overlap_start = offset.max(segment.offset);
                let overlap_end = end.min(old_end);
                for position in overlap_start..overlap_end {
                    if data[(position - offset) as usize]
                        != segment.data[(position - segment.offset) as usize]
                    {
                        return Err(LedgerError::Conflict);
                    }
                }
            }
        }
        let key = (stream, offset);
        if self.segments.contains_key(&key) {
            return Ok(self.segments[&key].state);
        }
        if self.bytes + data.len() > self.limits.max_connection_bytes {
            return Err(LedgerError::ConnectionLimit);
        }
        self.bytes += data.len();
        self.streams.entry(stream).or_insert(0);
        self.segments.insert(
            key,
            DeliverySegment {
                stream_id: stream,
                offset,
                data: data.to_vec(),
                state: DeliveryState::Unsent,
                context,
            },
        );
        Ok(DeliveryState::Unsent)
    }
    pub fn mark_in_flight(&mut self, stream: u64, offset: u64) -> Result<(), LedgerError> {
        self.transition(stream, offset, DeliveryState::InFlight)
    }
    pub fn mark_uncertain(&mut self, stream: u64, offset: u64) -> Result<(), LedgerError> {
        self.transition(stream, offset, DeliveryState::Uncertain)
    }
    fn transition(
        &mut self,
        stream: u64,
        offset: u64,
        to: DeliveryState,
    ) -> Result<(), LedgerError> {
        let segment = self
            .segments
            .get_mut(&(stream, offset))
            .ok_or(LedgerError::RangeNotFound)?;
        let valid = matches!(
            (segment.state, to),
            (DeliveryState::Unsent, DeliveryState::InFlight)
                | (DeliveryState::InFlight, DeliveryState::Uncertain)
        );
        if !valid {
            return Err(LedgerError::InvalidTransition);
        }
        segment.state = to;
        Ok(())
    }
    /// Logical delivery ACK. Carrier packet feedback must call no equivalent method.
    pub fn confirm_received(
        &mut self,
        stream: u64,
        offset: u64,
        epoch: DeliveryEpoch,
    ) -> Result<(), LedgerError> {
        let segment = self
            .segments
            .get_mut(&(stream, offset))
            .ok_or(LedgerError::RangeNotFound)?;
        if segment.context.delivery_epoch != epoch {
            return Err(LedgerError::OldEpoch);
        }
        if !matches!(
            segment.state,
            DeliveryState::InFlight | DeliveryState::Uncertain | DeliveryState::Confirmed
        ) {
            return Err(LedgerError::InvalidTransition);
        }
        segment.state = DeliveryState::Confirmed;
        let end = offset + segment.data.len() as u64;
        if end > self.watermark(stream) {
            self.streams.insert(stream, end);
        }
        Ok(())
    }
    pub fn packet_feedback(&self, _stream: u64, _offset: u64) { /* intentionally no delivery transition */
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    fn ctx(epoch: u64) -> SessionContext {
        SessionContext {
            delivery_epoch: DeliveryEpoch(epoch),
            key_phase: KeyPhase(0),
            path_generation: PathGeneration(1),
        }
    }
    fn ledger() -> DeliveryLedger {
        DeliveryLedger::new(Limits {
            max_reorder: 8,
            max_streams: 2,
            max_connection_bytes: 8,
            max_offset_jump: 16,
        })
    }
    #[test]
    fn state_machine_and_packet_feedback_are_orthogonal() {
        let mut l = ledger();
        l.insert(1, 0, b"ab", ctx(1)).unwrap();
        assert_eq!(l.segments().next().unwrap().state, DeliveryState::Unsent);
        l.packet_feedback(1, 0);
        assert_eq!(l.segments().next().unwrap().state, DeliveryState::Unsent);
        l.mark_in_flight(1, 0).unwrap();
        l.mark_uncertain(1, 0).unwrap();
        l.confirm_received(1, 0, DeliveryEpoch(1)).unwrap();
        assert_eq!(l.watermark(1), 2);
    }
    #[test]
    fn duplicate_overlap_and_conflict() {
        let mut l = ledger();
        l.insert(1, 0, b"abcd", ctx(1)).unwrap();
        assert_eq!(l.insert(1, 0, b"abcd", ctx(1)), Ok(DeliveryState::Unsent));
        assert_eq!(l.insert(1, 2, b"cd", ctx(1)), Ok(DeliveryState::Unsent));
        assert_eq!(l.insert(1, 2, b"cx", ctx(1)), Err(LedgerError::Conflict));
    }
    #[test]
    fn old_epoch_ack_does_not_move_watermark() {
        let mut l = ledger();
        l.insert(1, 0, b"a", ctx(2)).unwrap();
        l.mark_in_flight(1, 0).unwrap();
        assert_eq!(
            l.confirm_received(1, 0, DeliveryEpoch(1)),
            Err(LedgerError::OldEpoch)
        );
        assert_eq!(l.watermark(1), 0);
    }
    #[test]
    fn bounds_are_deterministic() {
        let mut l = ledger();
        assert_eq!(l.insert(1, 100, b"a", ctx(1)), Err(LedgerError::OffsetJump));
        l.insert(1, 0, b"abcd", ctx(1)).unwrap();
        assert_eq!(l.insert(1, 9, b"a", ctx(1)), Err(LedgerError::ReorderLimit));
        assert_eq!(l.insert(2, 0, b"abcd", ctx(1)), Ok(DeliveryState::Unsent));
        assert_eq!(
            l.insert(3, 0, b"a", ctx(1)),
            Err(LedgerError::TooManyStreams)
        );
    }
    #[test]
    fn ack_watermark_is_monotonic() {
        let mut l = ledger();
        l.insert(1, 0, b"a", ctx(1)).unwrap();
        l.insert(1, 1, b"b", ctx(1)).unwrap();
        l.mark_in_flight(1, 0).unwrap();
        l.mark_in_flight(1, 1).unwrap();
        l.confirm_received(1, 1, DeliveryEpoch(1)).unwrap();
        assert_eq!(l.watermark(1), 2);
        l.confirm_received(1, 0, DeliveryEpoch(1)).unwrap();
        assert_eq!(l.watermark(1), 2);
    }
}
