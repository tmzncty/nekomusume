//! Pure synchronous candidate delivery ledger and state machine.
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
pub struct SessionContext {
    pub delivery_epoch: DeliveryEpoch,
    pub key_phase: KeyPhase,
    pub path_generation: PathGeneration,
}
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum DeliveryState {
    Unsent,
    InFlight,
    Uncertain,
    Confirmed,
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
    ByteCountOverflow,
    OffsetJump,
    ReorderLimit,
    OffsetOverflow,
    Conflict,
    OldEpoch,
    ContextMismatch,
    InvalidTransition,
    RangeNotFound,
}

fn checked_total(current: usize, added: usize, limit: usize) -> Result<usize, LedgerError> {
    let total = current
        .checked_add(added)
        .ok_or(LedgerError::ByteCountOverflow)?;
    if total > limit {
        return Err(LedgerError::ConnectionLimit);
    }
    Ok(total)
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
    fn context_ok(&mut self, context: SessionContext) -> Result<(), LedgerError> {
        match self.context {
            Some(old) if old != context => Err(LedgerError::ContextMismatch),
            None => {
                self.context = Some(context);
                Ok(())
            }
            _ => Ok(()),
        }
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
        let len = data.len() as u64;
        let end = offset.checked_add(len).ok_or(LedgerError::OffsetOverflow)?;
        if !self.streams.contains_key(&stream) && self.streams.len() >= self.limits.max_streams {
            return Err(LedgerError::TooManyStreams);
        }
        let watermark = self.watermark(stream);
        let jump = watermark
            .checked_add(self.limits.max_offset_jump)
            .ok_or(LedgerError::OffsetOverflow)?;
        let reorder = watermark
            .checked_add(self.limits.max_reorder)
            .ok_or(LedgerError::OffsetOverflow)?;
        if offset > jump {
            return Err(LedgerError::OffsetJump);
        }
        if offset > reorder {
            return Err(LedgerError::ReorderLimit);
        }
        let overlaps: Vec<(u64, u64)> = self
            .segments
            .iter()
            .filter_map(|(k, s)| {
                let old_end = s.offset.checked_add(s.data.len() as u64)?;
                (s.stream_id == stream && offset < old_end && s.offset < end).then_some(*k)
            })
            .collect();
        for key in &overlaps {
            let s = &self.segments[key];
            let old_end = s
                .offset
                .checked_add(s.data.len() as u64)
                .ok_or(LedgerError::OffsetOverflow)?;
            for pos in offset.max(s.offset)..end.min(old_end) {
                if data[(pos - offset) as usize] != s.data[(pos - s.offset) as usize] {
                    return Err(LedgerError::Conflict);
                }
            }
        }
        if overlaps.is_empty() {
            let new_bytes =
                checked_total(self.bytes, data.len(), self.limits.max_connection_bytes)?;
            self.context_ok(context)?;
            self.bytes = new_bytes;
            self.streams.entry(stream).or_insert(0);
            self.segments.insert(
                (stream, offset),
                DeliverySegment {
                    stream_id: stream,
                    offset,
                    data: data.to_vec(),
                    state: DeliveryState::Unsent,
                    context,
                },
            );
            return Ok(DeliveryState::Unsent);
        }
        let mut start = offset;
        let mut finish = end;
        let mut state = DeliveryState::Unsent;
        let mut removed = 0usize;
        for key in &overlaps {
            let s = &self.segments[key];
            start = start.min(s.offset);
            finish = finish.max(
                s.offset
                    .checked_add(s.data.len() as u64)
                    .ok_or(LedgerError::OffsetOverflow)?,
            );
            state = s.state;
            removed = removed
                .checked_add(s.data.len())
                .ok_or(LedgerError::ByteCountOverflow)?;
        }
        // Never synthesize bytes across a hole. A new fragment may merge only
        // when the existing ranges plus the new range cover the whole result.
        let mut ranges: Vec<(u64, u64)> = overlaps
            .iter()
            .map(|key| {
                let s = &self.segments[key];
                (s.offset, s.offset + s.data.len() as u64)
            })
            .chain(std::iter::once((offset, end)))
            .collect();
        ranges.sort_unstable();
        let mut covered = start;
        for (range_start, range_end) in ranges {
            if range_start > covered {
                return Err(LedgerError::Conflict);
            }
            covered = covered.max(range_end);
        }
        if covered < finish {
            return Err(LedgerError::Conflict);
        }
        let merged_len = finish
            .checked_sub(start)
            .ok_or(LedgerError::OffsetOverflow)? as usize;
        let remaining = self
            .bytes
            .checked_sub(removed)
            .ok_or(LedgerError::ByteCountOverflow)?;
        let new_total = checked_total(remaining, merged_len, self.limits.max_connection_bytes)?;
        let mut merged = vec![0u8; merged_len];
        for key in &overlaps {
            let s = &self.segments[key];
            merged[(s.offset - start) as usize..(s.offset - start) as usize + s.data.len()]
                .copy_from_slice(&s.data);
        }
        merged[(offset - start) as usize..(offset - start) as usize + data.len()]
            .copy_from_slice(data);
        self.context_ok(context)?;
        for key in overlaps {
            self.segments.remove(&key);
        }
        self.segments.insert(
            (stream, start),
            DeliverySegment {
                stream_id: stream,
                offset: start,
                data: merged,
                state,
                context,
            },
        );
        self.bytes = new_total;
        Ok(state)
    }
    pub fn mark_in_flight(&mut self, s: u64, o: u64) -> Result<(), LedgerError> {
        self.transition(s, o, DeliveryState::InFlight)
    }
    pub fn mark_uncertain(&mut self, s: u64, o: u64) -> Result<(), LedgerError> {
        self.transition(s, o, DeliveryState::Uncertain)
    }
    fn transition(&mut self, s: u64, o: u64, to: DeliveryState) -> Result<(), LedgerError> {
        let x = self
            .segments
            .get_mut(&(s, o))
            .ok_or(LedgerError::RangeNotFound)?;
        if !matches!(
            (x.state, to),
            (DeliveryState::Unsent, DeliveryState::InFlight)
                | (DeliveryState::InFlight, DeliveryState::Uncertain)
        ) {
            return Err(LedgerError::InvalidTransition);
        }
        x.state = to;
        Ok(())
    }
    pub fn confirm_received(
        &mut self,
        s: u64,
        o: u64,
        context: SessionContext,
    ) -> Result<(), LedgerError> {
        let x = self
            .segments
            .get_mut(&(s, o))
            .ok_or(LedgerError::RangeNotFound)?;
        if x.context != context {
            return if x.context.delivery_epoch != context.delivery_epoch {
                Err(LedgerError::OldEpoch)
            } else {
                Err(LedgerError::ContextMismatch)
            };
        }
        if !matches!(
            x.state,
            DeliveryState::InFlight | DeliveryState::Uncertain | DeliveryState::Confirmed
        ) {
            return Err(LedgerError::InvalidTransition);
        }
        x.state = DeliveryState::Confirmed;
        let end = o
            .checked_add(x.data.len() as u64)
            .ok_or(LedgerError::OffsetOverflow)?;
        if end > self.watermark(s) {
            self.streams.insert(s, end);
        }
        Ok(())
    }
    pub fn packet_feedback(&self, _s: u64, _o: u64) {}
}
#[cfg(test)]
mod tests {
    use super::*;
    fn c(e: u64, k: u8, p: u64) -> SessionContext {
        SessionContext {
            delivery_epoch: DeliveryEpoch(e),
            key_phase: KeyPhase(k),
            path_generation: PathGeneration(p),
        }
    }
    fn l() -> DeliveryLedger {
        DeliveryLedger::new(Limits {
            max_reorder: 8,
            max_streams: 2,
            max_connection_bytes: 8,
            max_offset_jump: 16,
        })
    }
    #[test]
    fn states_and_feedback() {
        let mut x = l();
        x.insert(1, 0, b"ab", c(1, 0, 1)).unwrap();
        x.packet_feedback(1, 0);
        assert_eq!(x.segments().next().unwrap().state, DeliveryState::Unsent);
        x.mark_in_flight(1, 0).unwrap();
        x.mark_uncertain(1, 0).unwrap();
        x.confirm_received(1, 0, c(1, 0, 1)).unwrap();
        assert_eq!(x.watermark(1), 2)
    }
    #[test]
    fn overlap_merge_and_conflict() {
        let mut x = l();
        x.insert(1, 0, b"abcd", c(1, 0, 1)).unwrap();
        x.insert(1, 2, b"cdef", c(1, 0, 1)).unwrap();
        assert_eq!(x.segments().next().unwrap().data, b"abcdef");
        assert_eq!(x.bytes, 6);
        assert_eq!(x.insert(1, 3, b"X", c(1, 0, 1)), Err(LedgerError::Conflict))
    }
    #[test]
    fn context_and_old_epoch_rejected() {
        let mut x = l();
        x.insert(1, 0, b"a", c(2, 0, 1)).unwrap();
        assert_eq!(
            x.insert(1, 1, b"b", c(2, 1, 1)),
            Err(LedgerError::ContextMismatch)
        );
        x.mark_in_flight(1, 0).unwrap();
        assert_eq!(
            x.confirm_received(1, 0, c(1, 0, 1)),
            Err(LedgerError::OldEpoch)
        );
        assert_eq!(x.watermark(1), 0)
    }
    #[test]
    fn rejected_limits_do_not_commit_context() {
        for (limits, stream, offset, data) in [
            (
                Limits {
                    max_streams: 0,
                    ..l().limits
                },
                1,
                0,
                b"a".as_slice(),
            ),
            (
                Limits {
                    max_offset_jump: 0,
                    ..l().limits
                },
                1,
                1,
                b"a".as_slice(),
            ),
            (
                Limits {
                    max_reorder: 0,
                    ..l().limits
                },
                1,
                1,
                b"a".as_slice(),
            ),
            (
                Limits {
                    max_connection_bytes: 0,
                    ..l().limits
                },
                1,
                0,
                b"a".as_slice(),
            ),
        ] {
            let mut ledger = DeliveryLedger::new(limits);
            assert!(ledger.insert(stream, offset, data, c(1, 0, 1)).is_err());
            assert_eq!(ledger.context, None);
        }
    }

    #[test]
    fn overlap_never_zero_fills_a_gap() {
        let mut x = l();
        x.insert(1, 0, b"ab", c(1, 0, 1)).unwrap();
        x.insert(1, 4, b"ef", c(1, 0, 1)).unwrap();
        // This fragment overlaps the first segment but does not cover the
        // unknown gap before the second; merge only the covered range.
        x.insert(1, 1, b"bc", c(1, 0, 1)).unwrap();
        let segments: Vec<_> = x.segments().collect();
        assert_eq!(segments.len(), 2);
        assert_eq!(segments[0].data, b"abc");
        assert_eq!(segments[1].data, b"ef");
        assert_eq!(x.bytes, 5);
    }

    #[test]
    fn duplicate_and_contiguous_bytes_merge() {
        let mut x = l();
        x.insert(1, 0, b"ab", c(1, 0, 1)).unwrap();
        x.insert(1, 2, b"cd", c(1, 0, 1)).unwrap();
        x.insert(1, 1, b"bc", c(1, 0, 1)).unwrap();
        assert_eq!(x.segments().next().unwrap().data, b"abcd");
        assert_eq!(x.bytes, 4);
    }

    #[test]
    fn overlap_preserves_delivered_bytes() {
        let mut x = l();
        x.insert(1, 0, b"ab", c(1, 0, 1)).unwrap();
        x.mark_in_flight(1, 0).unwrap();
        x.confirm_received(1, 0, c(1, 0, 1)).unwrap();
        x.insert(1, 1, b"bc", c(1, 0, 1)).unwrap();
        assert_eq!(x.segments().next().unwrap().data, b"abc");
        assert_eq!(x.segments().next().unwrap().state, DeliveryState::Confirmed);
    }

    #[test]
    fn overflow_and_bounds_are_deterministic() {
        let mut x = l();
        assert_eq!(
            x.insert(1, u64::MAX - 1, b"xx", c(1, 0, 1)),
            Err(LedgerError::OffsetOverflow)
        );
        let mut y = DeliveryLedger::new(Limits {
            max_reorder: u64::MAX,
            max_streams: 1,
            max_connection_bytes: usize::MAX,
            max_offset_jump: u64::MAX,
        });
        assert_eq!(
            y.insert(1, u64::MAX, b"x", c(1, 0, 1)),
            Err(LedgerError::OffsetOverflow)
        );
        assert_eq!(
            checked_total(usize::MAX, 1, usize::MAX),
            Err(LedgerError::ByteCountOverflow)
        );
    }
    #[test]
    fn watermark_monotonic() {
        let mut x = l();
        x.insert(1, 0, b"a", c(1, 0, 1)).unwrap();
        x.insert(1, 1, b"b", c(1, 0, 1)).unwrap();
        x.mark_in_flight(1, 0).unwrap();
        x.mark_in_flight(1, 1).unwrap();
        x.confirm_received(1, 1, c(1, 0, 1)).unwrap();
        x.confirm_received(1, 0, c(1, 0, 1)).unwrap();
        assert_eq!(x.watermark(1), 2)
    }
}
