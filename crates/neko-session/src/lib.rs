//! Pure synchronous candidate delivery ledger and state machine.
use std::collections::{BTreeMap, VecDeque};

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
    InvalidMigration,
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
    /// Context migration is component-wise monotonic. Delivery epochs and key
    /// phases may advance only (never regress); a path generation may advance
    /// independently for a Carrier change. A single operation cannot advance
    /// delivery epoch while regressing either crypto or path context.
    fn context_ok(&mut self, context: SessionContext) -> Result<(), LedgerError> {
        match self.context {
            None => {
                self.context = Some(context);
                Ok(())
            }
            Some(old) => {
                if context.delivery_epoch.0 < old.delivery_epoch.0
                    || context.key_phase.0 < old.key_phase.0
                    || context.path_generation.0 < old.path_generation.0
                {
                    return Err(LedgerError::OldEpoch);
                }
                if context.delivery_epoch.0 > old.delivery_epoch.0
                    && (context.key_phase.0 != old.key_phase.0
                        || context.path_generation.0 != old.path_generation.0)
                {
                    return Err(LedgerError::InvalidMigration);
                }
                self.context = Some(context);
                Ok(())
            }
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
        let mut state = None;
        let mut removed = 0usize;
        for key in &overlaps {
            let s = &self.segments[key];
            start = start.min(s.offset);
            finish = finish.max(
                s.offset
                    .checked_add(s.data.len() as u64)
                    .ok_or(LedgerError::OffsetOverflow)?,
            );
            if state.is_some_and(|old| old != s.state) {
                return Err(LedgerError::InvalidMigration);
            }
            state = Some(s.state);
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
                s.offset
                    .checked_add(s.data.len() as u64)
                    .map(|end| (s.offset, end))
                    .ok_or(LedgerError::OffsetOverflow)
            })
            .collect::<Result<Vec<_>, _>>()?;
        ranges.push((offset, end));
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
                state: state.expect("overlaps is non-empty"),
                context,
            },
        );
        self.bytes = new_total;
        Ok(state.expect("overlaps is non-empty"))
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
        if context.delivery_epoch.0 < x.context.delivery_epoch.0
            || context.key_phase.0 < x.context.key_phase.0
            || context.path_generation.0 < x.context.path_generation.0
        {
            return Err(LedgerError::OldEpoch);
        }
        if context.delivery_epoch.0 == x.context.delivery_epoch.0
            && context.key_phase.0 == x.context.key_phase.0
            && context.path_generation.0 == x.context.path_generation.0
        {
        } else if context.delivery_epoch.0 > x.context.delivery_epoch.0
            && (context.key_phase.0 != x.context.key_phase.0
                || context.path_generation.0 != x.context.path_generation.0)
        {
            return Err(LedgerError::InvalidMigration);
        } else {
            x.context = context;
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
    fn monotonic_path_and_key_migrations_are_allowed_but_regressions_rejected() {
        let mut x = l();
        x.insert(1, 0, b"a", c(1, 0, 1)).unwrap();
        x.insert(1, 1, b"b", c(1, 1, 1)).unwrap();
        x.insert(1, 2, b"c", c(1, 1, 2)).unwrap();
        assert_eq!(x.insert(1, 3, b"d", c(1, 0, 2)), Err(LedgerError::OldEpoch));
        assert_eq!(
            x.insert(1, 4, b"e", c(2, 2, 2)),
            Err(LedgerError::InvalidMigration)
        );
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
        assert_eq!(x.insert(1, 1, b"b", c(2, 1, 1)), Ok(DeliveryState::Unsent));
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
    fn mixed_state_overlap_is_not_collapsed() {
        let mut x = l();
        x.insert(1, 0, b"ab", c(1, 0, 1)).unwrap();
        x.insert(1, 2, b"cd", c(1, 0, 1)).unwrap();
        x.mark_in_flight(1, 0).unwrap();
        assert_eq!(
            x.insert(1, 1, b"bc", c(1, 0, 1)),
            Err(LedgerError::InvalidMigration)
        );
        let states: Vec<_> = x.segments().map(|s| s.state).collect();
        assert_eq!(states, vec![DeliveryState::InFlight, DeliveryState::Unsent]);
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
    fn exact_duplicate_is_idempotent_and_state_transition_is_bounded() {
        let mut x = l();
        x.insert(1, 0, b"abc", c(1, 0, 1)).unwrap();
        x.mark_in_flight(1, 0).unwrap();
        x.confirm_received(1, 0, c(1, 0, 1)).unwrap();
        assert_eq!(
            x.insert(1, 0, b"abc", c(1, 0, 1)),
            Ok(DeliveryState::Confirmed)
        );
        assert_eq!(x.segments().count(), 1);
        assert_eq!(x.bytes, 3);
        assert_eq!(x.mark_uncertain(1, 0), Err(LedgerError::InvalidTransition));
        assert_eq!(x.confirm_received(1, 0, c(1, 0, 1)), Ok(()));
        assert_eq!(x.watermark(1), 3);
    }

    #[test]
    fn old_epoch_replay_cannot_confirm_or_change_state() {
        let mut x = l();
        x.insert(1, 0, b"abc", c(7, 2, 9)).unwrap();
        x.mark_in_flight(1, 0).unwrap();
        x.mark_uncertain(1, 0).unwrap();
        assert_eq!(
            x.confirm_received(1, 0, c(6, 2, 9)),
            Err(LedgerError::OldEpoch)
        );
        assert_eq!(x.segments().next().unwrap().state, DeliveryState::Uncertain);
        assert_eq!(x.watermark(1), 0);
    }

    #[test]
    fn missing_range_and_invalid_transition_are_stable_errors() {
        let mut x = l();
        assert_eq!(x.mark_in_flight(99, 0), Err(LedgerError::RangeNotFound));
        x.insert(1, 0, b"x", c(1, 0, 1)).unwrap();
        assert_eq!(x.mark_uncertain(1, 0), Err(LedgerError::InvalidTransition));
        assert_eq!(
            x.confirm_received(1, 0, c(1, 0, 1)),
            Err(LedgerError::InvalidTransition)
        );
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

/// M3-alpha bounded Session runtime state. Carrier I/O is deliberately outside
/// this type: callers feed authenticated records and drain accepted outbound
/// records, while a carrier adapter performs socket/path mechanics.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct SessionId(pub u64);
#[derive(Debug, Clone, Copy, PartialEq, Eq, PartialOrd, Ord)]
pub struct StreamId(pub u64);
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct RuntimeLimits {
    pub max_streams: usize,
    pub max_queue_records: usize,
    pub max_queue_bytes: usize,
    pub max_total_bytes: usize,
    pub max_record_bytes: usize,
    pub idle_timeout_ms: u64,
    pub close_timeout_ms: u64,
}
impl Default for RuntimeLimits {
    fn default() -> Self {
        Self {
            max_streams: 1,
            max_queue_records: 64,
            max_queue_bytes: 64 * 1024,
            max_total_bytes: 1 << 20,
            max_record_bytes: 1200,
            idle_timeout_ms: 30_000,
            close_timeout_ms: 5_000,
        }
    }
}
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum RuntimeState {
    Open,
    Closing,
    Closed,
    Error,
}
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum StreamState {
    Open,
    HalfClosedLocal,
    HalfClosedRemote,
    Closed,
    Reset,
}
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum RuntimeError {
    InvalidLimits,
    Terminal,
    QueueFull,
    RecordTooLarge,
    TotalLimit,
    StreamLimit,
    UnknownStream,
    InvalidTransition,
    Deadline,
    IdleTimeout,
    Cancelled,
    Protocol,
}
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct OutboundRecord {
    pub stream: StreamId,
    pub offset: u64,
    pub data: Vec<u8>,
}
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct InboundRecord {
    pub stream: StreamId,
    pub offset: u64,
    pub data: Vec<u8>,
}
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum RuntimeEventKind {
    SessionOpened,
    StreamOpened,
    DataQueued,
    DataReceived,
    DuplicateDedup,
    DeliveryAck,
    StreamClosed,
    CloseSent,
    SessionClosed,
    Error,
}
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct RuntimeEvent {
    pub seq: u64,
    pub at_ms: u64,
    pub kind: RuntimeEventKind,
}
#[derive(Debug, Clone)]
struct RuntimeStream {
    state: StreamState,
    next_send: u64,
    next_receive: u64,
}
#[derive(Debug)]
pub struct SessionRuntime {
    id: SessionId,
    limits: RuntimeLimits,
    state: RuntimeState,
    streams: BTreeMap<StreamId, RuntimeStream>,
    send: VecDeque<OutboundRecord>,
    recv: VecDeque<InboundRecord>,
    received: BTreeMap<(StreamId, u64), Vec<u8>>,
    confirmed: BTreeMap<StreamId, u64>,
    queued_bytes: usize,
    total_bytes: usize,
    last_activity_ms: u64,
    close_deadline_ms: Option<u64>,
    cancelled: bool,
    events: Vec<RuntimeEvent>,
    next_event: u64,
}
impl SessionRuntime {
    pub fn new(id: SessionId, limits: RuntimeLimits, now_ms: u64) -> Result<Self, RuntimeError> {
        if limits.max_streams == 0
            || limits.max_queue_records == 0
            || limits.max_queue_bytes == 0
            || limits.max_total_bytes == 0
            || limits.max_record_bytes == 0
            || limits.idle_timeout_ms == 0
            || limits.close_timeout_ms == 0
        {
            return Err(RuntimeError::InvalidLimits);
        }
        let mut r = Self {
            id,
            limits,
            state: RuntimeState::Open,
            streams: BTreeMap::new(),
            send: VecDeque::new(),
            recv: VecDeque::new(),
            received: BTreeMap::new(),
            confirmed: BTreeMap::new(),
            queued_bytes: 0,
            total_bytes: 0,
            last_activity_ms: now_ms,
            close_deadline_ms: None,
            cancelled: false,
            events: Vec::new(),
            next_event: 0,
        };
        r.event(now_ms, RuntimeEventKind::SessionOpened);
        Ok(r)
    }
    pub fn id(&self) -> SessionId {
        self.id
    }
    pub fn state(&self) -> RuntimeState {
        self.state
    }
    pub fn queued_bytes(&self) -> usize {
        self.queued_bytes
    }
    pub fn queued_records(&self) -> usize {
        self.send.len() + self.recv.len()
    }
    pub fn total_bytes(&self) -> usize {
        self.total_bytes
    }
    pub fn close_remote(&mut self, now_ms: u64) -> Result<(), RuntimeError> {
        if self.state == RuntimeState::Closed {
            return Ok(());
        }
        self.check(now_ms)?;
        self.state = RuntimeState::Closed;
        self.send.clear();
        self.recv.clear();
        self.received.clear();
        self.confirmed.clear();
        self.queued_bytes = 0;
        self.event(now_ms, RuntimeEventKind::SessionClosed);
        Ok(())
    }
    pub fn events(&self) -> impl Iterator<Item = &RuntimeEvent> {
        self.events.iter()
    }
    pub fn open_stream(&mut self, stream: StreamId, now_ms: u64) -> Result<(), RuntimeError> {
        self.check(now_ms)?;
        if self.streams.contains_key(&stream) {
            return Err(RuntimeError::InvalidTransition);
        }
        if self.streams.len() >= self.limits.max_streams {
            return Err(RuntimeError::StreamLimit);
        }
        self.streams.insert(
            stream,
            RuntimeStream {
                state: StreamState::Open,
                next_send: 0,
                next_receive: 0,
            },
        );
        self.event(now_ms, RuntimeEventKind::StreamOpened);
        Ok(())
    }
    pub fn queue_send(
        &mut self,
        stream: StreamId,
        data: &[u8],
        now_ms: u64,
    ) -> Result<u64, RuntimeError> {
        self.check(now_ms)?;
        let s = self
            .streams
            .get_mut(&stream)
            .ok_or(RuntimeError::UnknownStream)?;
        if s.state != StreamState::Open && s.state != StreamState::HalfClosedRemote {
            return Err(RuntimeError::InvalidTransition);
        }
        if data.is_empty() {
            return Ok(s.next_send);
        }
        if data.len() > self.limits.max_record_bytes {
            return Err(RuntimeError::RecordTooLarge);
        }
        if self.send.len() >= self.limits.max_queue_records
            || self
                .queued_bytes
                .checked_add(data.len())
                .ok_or(RuntimeError::TotalLimit)?
                > self.limits.max_queue_bytes
        {
            return Err(RuntimeError::QueueFull);
        }
        if self
            .total_bytes
            .checked_add(data.len())
            .ok_or(RuntimeError::TotalLimit)?
            > self.limits.max_total_bytes
        {
            return Err(RuntimeError::TotalLimit);
        }
        let off = s.next_send;
        s.next_send += data.len() as u64;
        self.total_bytes += data.len();
        self.queued_bytes += data.len();
        self.send.push_back(OutboundRecord {
            stream,
            offset: off,
            data: data.to_vec(),
        });
        self.touch(now_ms);
        self.event(now_ms, RuntimeEventKind::DataQueued);
        Ok(off)
    }
    pub fn pop_send(&mut self, now_ms: u64) -> Result<Option<OutboundRecord>, RuntimeError> {
        self.check(now_ms)?;
        let x = self.send.pop_front();
        if let Some(ref r) = x {
            self.queued_bytes -= r.data.len();
            self.touch(now_ms);
        }
        Ok(x)
    }
    pub fn receive(&mut self, record: InboundRecord, now_ms: u64) -> Result<(), RuntimeError> {
        self.check(now_ms)?;
        if record.data.is_empty() || record.data.len() > self.limits.max_record_bytes {
            return Err(RuntimeError::RecordTooLarge);
        }
        let next_receive = self
            .streams
            .get(&record.stream)
            .ok_or(RuntimeError::UnknownStream)?
            .next_receive;
        if record.offset < next_receive {
            if self.received.get(&(record.stream, record.offset)) == Some(&record.data) {
                self.event(now_ms, RuntimeEventKind::DuplicateDedup);
                return Ok(());
            }
            return Err(RuntimeError::Protocol);
        }
        if record.offset != next_receive {
            return Err(RuntimeError::Protocol);
        }
        if self.recv.len() >= self.limits.max_queue_records
            || self
                .queued_bytes
                .checked_add(record.data.len())
                .ok_or(RuntimeError::TotalLimit)?
                > self.limits.max_queue_bytes
        {
            return Err(RuntimeError::QueueFull);
        }
        let s = self
            .streams
            .get_mut(&record.stream)
            .ok_or(RuntimeError::UnknownStream)?;
        s.next_receive += record.data.len() as u64;
        self.received
            .insert((record.stream, record.offset), record.data.clone());
        self.recv.push_back(record);
        self.queued_bytes += self.recv.back().unwrap().data.len();
        self.touch(now_ms);
        self.event(now_ms, RuntimeEventKind::DataReceived);
        Ok(())
    }
    pub fn delivery_ack(
        &mut self,
        stream: StreamId,
        offset: u64,
        len: usize,
        now_ms: u64,
    ) -> Result<(), RuntimeError> {
        self.check(now_ms)?;
        let end = offset
            .checked_add(len as u64)
            .ok_or(RuntimeError::TotalLimit)?;
        let current = self.confirmed.get(&stream).copied().unwrap_or(0);
        if end < current {
            return Err(RuntimeError::Protocol);
        }
        if end > current {
            self.confirmed.insert(stream, end);
        }
        self.event(now_ms, RuntimeEventKind::DeliveryAck);
        Ok(())
    }
    pub fn confirmed_watermark(&self, stream: StreamId) -> u64 {
        self.confirmed.get(&stream).copied().unwrap_or(0)
    }

    pub fn pop_receive(&mut self, now_ms: u64) -> Result<Option<InboundRecord>, RuntimeError> {
        self.check(now_ms)?;
        let x = self.recv.pop_front();
        if let Some(ref r) = x {
            self.queued_bytes -= r.data.len();
            self.touch(now_ms);
        }
        Ok(x)
    }
    pub fn close_graceful(&mut self, now_ms: u64) -> Result<(), RuntimeError> {
        self.check(now_ms)?;
        if self.state == RuntimeState::Open {
            self.state = RuntimeState::Closing;
            self.close_deadline_ms = Some(now_ms.saturating_add(self.limits.close_timeout_ms));
            self.event(now_ms, RuntimeEventKind::CloseSent);
        }
        Ok(())
    }
    pub fn cancel(&mut self, now_ms: u64) -> Result<(), RuntimeError> {
        if self.state == RuntimeState::Closed {
            return Ok(());
        }
        self.cancelled = true;
        self.state = RuntimeState::Error;
        self.send.clear();
        self.recv.clear();
        self.received.clear();
        self.confirmed.clear();
        self.queued_bytes = 0;
        self.event(now_ms, RuntimeEventKind::Error);
        Ok(())
    }
    pub fn tick(&mut self, now_ms: u64) -> Result<(), RuntimeError> {
        if self.state == RuntimeState::Closed || self.state == RuntimeState::Error {
            return Ok(());
        }
        if now_ms.saturating_sub(self.last_activity_ms) >= self.limits.idle_timeout_ms {
            self.state = RuntimeState::Closed;
            self.send.clear();
            self.recv.clear();
            self.queued_bytes = 0;
            self.event(now_ms, RuntimeEventKind::SessionClosed);
            return Err(RuntimeError::IdleTimeout);
        }
        if let Some(d) = self.close_deadline_ms
            && now_ms >= d
        {
            self.state = RuntimeState::Closed;
            self.send.clear();
            self.recv.clear();
            self.queued_bytes = 0;
            self.event(now_ms, RuntimeEventKind::SessionClosed);
        }
        Ok(())
    }
    fn check(&mut self, now_ms: u64) -> Result<(), RuntimeError> {
        if self.cancelled || self.state == RuntimeState::Error {
            return Err(RuntimeError::Cancelled);
        }
        if self.state == RuntimeState::Closed {
            return Err(RuntimeError::Terminal);
        }
        if now_ms.saturating_sub(self.last_activity_ms) >= self.limits.idle_timeout_ms {
            return self.tick(now_ms).and(Err(RuntimeError::IdleTimeout));
        }
        Ok(())
    }
    fn touch(&mut self, now: u64) {
        self.last_activity_ms = now
    }
    fn event(&mut self, at: u64, kind: RuntimeEventKind) {
        let seq = self.next_event;
        self.next_event += 1;
        self.events.push(RuntimeEvent {
            seq,
            at_ms: at,
            kind,
        });
    }
}

/// Versioned, transport-neutral event projection shared by lab and WAN runners.
/// The projection is intentionally plain data: serialization belongs to the
/// adapter, and observing an event cannot mutate Session state.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct ObservableEvent {
    pub schema: u16,
    pub seq: u64,
    pub at_ms: u64,
    pub session: SessionId,
    pub stream: Option<StreamId>,
    pub kind: RuntimeEventKind,
}
impl RuntimeEventKind {
    pub const fn name(self) -> &'static str {
        match self {
            Self::SessionOpened => "session_opened",
            Self::StreamOpened => "stream_opened",
            Self::DataQueued => "data_queued",
            Self::DataReceived => "data_received",
            Self::DuplicateDedup => "duplicate_dedup",
            Self::DeliveryAck => "delivery_ack",
            Self::StreamClosed => "stream_closed",
            Self::CloseSent => "close_sent",
            Self::SessionClosed => "session_closed",
            Self::Error => "error",
        }
    }
}
impl ObservableEvent {
    /// Stable JSON-lines representation shared by lab/WAN adapters.
    pub fn to_json_line(self) -> String {
        format!(
            "{{\"schema\":{},\"seq\":{},\"at_ms\":{},\"session\":{},\"stream\":{},\"kind\":\"{}\"}}",
            self.schema,
            self.seq,
            self.at_ms,
            self.session.0,
            self.stream
                .map_or_else(|| "null".to_string(), |x| x.0.to_string()),
            self.kind.name()
        )
    }
}
impl SessionRuntime {
    pub fn observable_events(&self) -> impl Iterator<Item = ObservableEvent> + '_ {
        self.events.iter().map(move |e| ObservableEvent {
            schema: 1,
            seq: e.seq,
            at_ms: e.at_ms,
            session: self.id,
            stream: None,
            kind: e.kind,
        })
    }
}

#[cfg(test)]
mod runtime_tests {
    use super::*;

    fn limits() -> RuntimeLimits {
        RuntimeLimits {
            max_streams: 1,
            max_queue_records: 2,
            max_queue_bytes: 4,
            max_total_bytes: 8,
            max_record_bytes: 4,
            idle_timeout_ms: 10,
            close_timeout_ms: 5,
        }
    }

    #[test]
    fn lifecycle_ordered_exchange_and_graceful_close_are_bounded() {
        let mut r = SessionRuntime::new(SessionId(7), limits(), 0).unwrap();
        assert_eq!(r.open_stream(StreamId(1), 1), Ok(()));
        assert_eq!(r.queue_send(StreamId(1), b"ab", 2), Ok(0));
        assert_eq!(r.pop_send(3).unwrap().unwrap().data, b"ab");
        assert_eq!(
            r.receive(
                InboundRecord {
                    stream: StreamId(1),
                    offset: 0,
                    data: b"xy".to_vec()
                },
                4
            ),
            Ok(())
        );
        assert_eq!(r.pop_receive(5).unwrap().unwrap().data, b"xy");
        assert_eq!(r.close_graceful(6), Ok(()));
        assert_eq!(r.state(), RuntimeState::Closing);
        assert_eq!(r.close_graceful(7), Ok(()));
        assert_eq!(r.tick(11), Ok(()));
        assert_eq!(r.state(), RuntimeState::Closed);
    }

    #[test]
    fn queue_limits_and_cancel_are_atomic_terminal_operations() {
        let mut r = SessionRuntime::new(SessionId(1), limits(), 0).unwrap();
        r.open_stream(StreamId(1), 0).unwrap();
        assert_eq!(r.queue_send(StreamId(1), b"abcd", 1), Ok(0));
        assert_eq!(
            r.queue_send(StreamId(1), b"e", 2),
            Err(RuntimeError::QueueFull)
        );
        assert_eq!(r.pop_send(3).unwrap().unwrap().data, b"abcd");
        assert_eq!(r.cancel(4), Ok(()));
        assert_eq!(r.state(), RuntimeState::Error);
        assert_eq!(
            r.queue_send(StreamId(1), b"x", 5),
            Err(RuntimeError::Cancelled)
        );
        assert_eq!(r.pop_send(5), Err(RuntimeError::Cancelled));
    }

    #[test]
    fn idle_deadline_prevents_post_timeout_mutation() {
        let mut r = SessionRuntime::new(SessionId(1), limits(), 0).unwrap();
        r.open_stream(StreamId(1), 1).unwrap();
        assert_eq!(r.tick(11), Err(RuntimeError::IdleTimeout));
        assert_eq!(r.state(), RuntimeState::Closed);
        assert_eq!(
            r.queue_send(StreamId(1), b"x", 12),
            Err(RuntimeError::Terminal)
        );
    }
    #[test]
    fn observable_event_json_is_stable_and_monotonic() {
        let mut r = SessionRuntime::new(SessionId(9), limits(), 0).unwrap();
        r.open_stream(StreamId(1), 1).unwrap();
        let e: Vec<_> = r.observable_events().collect();
        assert_eq!(e[0].seq, 0);
        assert_eq!(e[1].seq, 1);
        assert_eq!(
            e[0].to_json_line(),
            "{\"schema\":1,\"seq\":0,\"at_ms\":0,\"session\":9,\"stream\":null,\"kind\":\"session_opened\"}"
        );
    }

    #[test]
    fn duplicate_is_deduplicated_and_delivery_ack_advances_watermark() {
        let duplicate_limits = RuntimeLimits {
            max_queue_records: 4,
            max_queue_bytes: 8,
            ..limits()
        };
        let mut r = SessionRuntime::new(SessionId(12), duplicate_limits, 0).unwrap();
        r.open_stream(StreamId(1), 1).unwrap();
        let record = InboundRecord {
            stream: StreamId(1),
            offset: 0,
            data: b"abc".to_vec(),
        };
        assert_eq!(r.receive(record.clone(), 2), Ok(()));
        assert_eq!(r.receive(record, 3), Ok(()));
        assert_eq!(r.pop_receive(4).unwrap().unwrap().data, b"abc");
        assert!(r.pop_receive(5).unwrap().is_none());
        assert_eq!(r.delivery_ack(StreamId(1), 0, 3, 6), Ok(()));
        assert_eq!(r.confirmed_watermark(StreamId(1)), 3);
        assert_eq!(
            r.delivery_ack(StreamId(1), 1, 1, 7),
            Err(RuntimeError::Protocol)
        );
        assert_eq!(
            r.observable_events()
                .filter(|e| e.kind == RuntimeEventKind::DuplicateDedup)
                .count(),
            1
        );
        assert_eq!(
            r.observable_events()
                .filter(|e| e.kind == RuntimeEventKind::DeliveryAck)
                .count(),
            1
        );
    }

    #[test]
    fn virtual_clock_udp_death_preserves_order_after_tcp_recovery() {
        let sim_limits = RuntimeLimits {
            max_streams: 1,
            max_queue_records: 8,
            max_queue_bytes: 16,
            max_total_bytes: 16,
            max_record_bytes: 4,
            idle_timeout_ms: 100,
            close_timeout_ms: 10,
        };
        let mut sender = SessionRuntime::new(SessionId(41), sim_limits, 0).unwrap();
        let mut receiver = SessionRuntime::new(SessionId(41), sim_limits, 0).unwrap();
        sender.open_stream(StreamId(1), 0).unwrap();
        receiver.open_stream(StreamId(1), 0).unwrap();
        for n in 0..8u8 {
            sender
                .queue_send(StreamId(1), &[n], u64::from(n) + 1)
                .unwrap();
        }
        let mut delivered = Vec::new();
        for n in 0..8u8 {
            let record = sender.pop_send(u64::from(n) + 10).unwrap().unwrap();
            if n < 3 {
                receiver
                    .receive(
                        InboundRecord {
                            stream: record.stream,
                            offset: record.offset,
                            data: record.data,
                        },
                        20 + u64::from(n),
                    )
                    .unwrap();
            }
        }
        for n in 3..8u8 {
            receiver
                .receive(
                    InboundRecord {
                        stream: StreamId(1),
                        offset: u64::from(n),
                        data: vec![n],
                    },
                    40 + u64::from(n),
                )
                .unwrap();
        }
        while let Some(r) = receiver.pop_receive(60).unwrap() {
            delivered.extend(r.data);
        }
        assert_eq!(delivered, (0u8..8).collect::<Vec<_>>());
        assert_eq!(
            receiver
                .observable_events()
                .filter(|e| e.kind == RuntimeEventKind::DataReceived)
                .count(),
            8
        );
    }
}
