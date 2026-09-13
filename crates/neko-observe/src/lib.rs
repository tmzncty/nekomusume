//! Secret-free, bounded runtime projection onto the Era-4 v1 event contract.
use std::collections::VecDeque;

use neko_carrier::{
    ConcurrentCarrierManager, ConcurrentPathKey, ConcurrentSwitchEvent, FairScheduler,
    HealthSample, HealthState, PathId, StreamPriority, SwitchReason,
};
use neko_reliable::{Recovery, RecoveryResult};
use neko_session::{DatagramCounters, SessionId};

pub const MAX_EVENTS: usize = 1024;

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct Correlation {
    pub session_id: String,
    pub stream_id: Option<String>,
    pub carrier_id: Option<String>,
    pub path_id: Option<String>,
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct Event {
    pub sequence: u64,
    pub observed_at_ms: u64,
    pub event: &'static str,
    pub severity: &'static str,
    pub correlation: Correlation,
    /// Preformatted JSON object assembled only from numeric values and fixed vocabulary.
    pub data: String,
}
impl Event {
    pub fn to_json_line(&self) -> String {
        format!(
            "{{\"schema\":\"nekomusume.observability-event.v1\",\"schema_version\":1,\"event\":\"{}\",\"sequence\":{},\"observed_at_ms\":{},\"severity\":\"{}\",\"correlation\":{},\"data\":{}}}",
            self.event,
            self.sequence,
            self.observed_at_ms,
            self.severity,
            correlation_json(&self.correlation),
            self.data
        )
    }
}

/// Runtime-owned producer. Callers supply monotonic observation times and opaque,
/// process-generated identifiers; no network locator or arbitrary error text is accepted.
#[derive(Debug)]
pub struct Producer {
    session_id: String,
    capacity: usize,
    next_sequence: u64,
    dropped_total: u64,
    /// Whether the single sequence-exhaustion `resource.limit_hit` was emitted.
    limit_hit_emitted: bool,
    /// Evictions since the last emitted `diagnostic.events_dropped` report.
    drops_since_report: u64,
    events: VecDeque<Event>,
    last_datagram: DatagramCounters,
    retransmit_frames_total: u64,
    retransmit_bytes_total: u64,
    pto_total: u64,
    interactive_dequeues_total: u64,
    bulk_dequeues_total: u64,
    starvation_guard_total: u64,
    max_queued_bytes: u64,
    max_open_streams: u64,
}
impl Producer {
    pub fn new(session: SessionId, capacity: usize) -> Option<Self> {
        if !(1..=MAX_EVENTS).contains(&capacity) {
            return None;
        }
        Some(Self {
            session_id: format!("session:{}", session.0),
            capacity,
            next_sequence: 0,
            dropped_total: 0,
            limit_hit_emitted: false,
            drops_since_report: 0,
            events: VecDeque::with_capacity(capacity),
            last_datagram: DatagramCounters::default(),
            retransmit_frames_total: 0,
            retransmit_bytes_total: 0,
            pto_total: 0,
            interactive_dequeues_total: 0,
            bulk_dequeues_total: 0,
            starvation_guard_total: 0,
            max_queued_bytes: 0,
            max_open_streams: 0,
        })
    }
    pub fn events(&self) -> impl Iterator<Item = &Event> {
        self.events.iter()
    }
    pub fn capacity(&self) -> usize {
        self.capacity
    }
    pub fn dropped_total(&self) -> u64 {
        self.dropped_total
    }
    pub fn retained(&self) -> usize {
        self.events.len()
    }
    pub fn retransmit_totals(&self) -> (u64, u64) {
        (self.retransmit_frames_total, self.retransmit_bytes_total)
    }
    pub fn pto_total(&self) -> u64 {
        self.pto_total
    }
    pub fn dequeue_totals(&self) -> (u64, u64, u64) {
        (
            self.interactive_dequeues_total,
            self.bulk_dequeues_total,
            self.starvation_guard_total,
        )
    }
    pub fn resource_high_water(&self) -> (u64, u64) {
        (self.max_open_streams, self.max_queued_bytes)
    }

    pub fn record_health(
        &mut self,
        at_ms: u64,
        carrier_id: u64,
        path: PathId,
        previous: Option<HealthState>,
        state: HealthState,
        sample: HealthSample,
    ) {
        let correlation = self.carrier_correlation(carrier_id, path);
        self.push(at_ms, "carrier.health_sample", "debug", correlation.clone(), format!(
            "{{\"health_state\":\"{}\",\"latest_rtt_us\":{},\"loss_per_mille\":{},\"pto_count\":{}}}",
            health_name(state), sample.rtt_us, sample.loss_per_mille, sample.pto));
        if let Some(old) = previous.filter(|old| *old != state) {
            self.push(
                at_ms,
                "carrier.health_transition",
                if state == HealthState::Failed {
                    "error"
                } else {
                    "info"
                },
                correlation,
                format!(
                    "{{\"previous_health_state\":\"{}\",\"health_state\":\"{}\"}}",
                    health_name(old),
                    health_name(state)
                ),
            );
        }
    }

    pub fn record_switch(
        &mut self,
        manager: &ConcurrentCarrierManager,
        event: ConcurrentSwitchEvent,
    ) {
        let to = event.to.or(event.from);
        let correlation = to.map_or_else(
            || self.session_correlation(),
            |key| Correlation {
                session_id: self.session_id.clone(),
                stream_id: None,
                carrier_id: Some(format!("carrier:{}", key.path.0)),
                path_id: Some(path_id(key)),
            },
        );
        let outcome = if event.to.is_some() {
            "succeeded"
        } else {
            "failed"
        };
        let name = if event.to.is_some() {
            "carrier.switch_completed"
        } else {
            "carrier.switch_failed"
        };
        let from = event
            .from
            .map(path_id)
            .map_or(String::new(), |v| format!(",\"from_path_id\":\"{v}\""));
        let to_path = event
            .to
            .map(path_id)
            .map_or(String::new(), |v| format!(",\"to_path_id\":\"{v}\""));
        let kind = event
            .to
            .and_then(|key| manager.kind(key).ok())
            .map_or("other", carrier_kind);
        self.push(event.decided_at_ms, name, if event.to.is_some() { "info" } else { "warn" }, correlation, format!(
            "{{\"carrier_kind\":\"{}\",\"switch_reason\":\"{}\",\"outcome\":\"{}\",\"path_generation\":{}{}{} }}",
            kind, switch_reason(event.reason), outcome, to.map_or(0, |key| key.generation.0), from, to_path));
    }

    pub fn record_recovery_ack(
        &mut self,
        at_ms: u64,
        carrier_id: u64,
        path: PathId,
        recovery: &Recovery,
        result: &RecoveryResult,
    ) {
        let correlation = self.carrier_correlation(carrier_id, path);
        if recovery.rtt.latest_us != 0 {
            self.push(at_ms, "recovery.rtt_updated", "debug", correlation.clone(), format!(
                "{{\"latest_rtt_us\":{},\"min_rtt_us\":{},\"smoothed_rtt_us\":{},\"rtt_variance_us\":{}}}",
                recovery.rtt.latest_us, recovery.rtt.min_us, recovery.rtt.smoothed_us, recovery.rtt.variance_us));
        }
        if !result.lost_packets.is_empty() {
            self.push(
                at_ms,
                "recovery.loss_detected",
                "warn",
                correlation.clone(),
                format!(
                    "{{\"lost_packets\":{},\"lost_bytes\":{}}}",
                    result.lost_packets.len(),
                    result.lost_bytes
                ),
            );
        }
        if !result.retransmit_frames.is_empty() {
            let frames = u64::try_from(result.retransmit_frames.len()).unwrap_or(u64::MAX);
            self.retransmit_frames_total = self.retransmit_frames_total.saturating_add(frames);
            self.retransmit_bytes_total = self
                .retransmit_bytes_total
                .saturating_add(result.retransmit_bytes);
            self.push(
                at_ms,
                "recovery.frame_retransmitted",
                "info",
                correlation,
                format!(
                    "{{\"retransmit_frames\":{},\"retransmit_bytes\":{}}}",
                    frames, result.retransmit_bytes
                ),
            );
        }
    }

    pub fn record_pto(
        &mut self,
        at_ms: u64,
        carrier_id: u64,
        path: PathId,
        recovery: &Recovery,
        probe_frames: usize,
    ) {
        self.pto_total = self.pto_total.saturating_add(1);
        self.push(
            at_ms,
            "recovery.pto_fired",
            "warn",
            self.carrier_correlation(carrier_id, path),
            format!(
                "{{\"pto_count\":{},\"pto_total\":{},\"probe_frames\":{}}}",
                recovery.pto_count, self.pto_total, probe_frames
            ),
        );
    }

    /// Emits exact deltas from the Session datagram runtime. The v1 schema has
    /// no datagram-specific fields, so stable error codes carry only decisions.
    pub fn record_datagrams(&mut self, at_ms: u64, counters: DatagramCounters) {
        let admitted = counters
            .admitted
            .saturating_sub(self.last_datagram.admitted);
        let dropped = counters.dropped.saturating_sub(self.last_datagram.dropped);
        let oversize = counters
            .rejected_oversize
            .saturating_sub(self.last_datagram.rejected_oversize);
        let queue = counters
            .queue_dropped
            .saturating_sub(self.last_datagram.queue_dropped);
        // Bounded per-event emission: the retained ring holds at most
        // `capacity` events, so emitting more than `capacity` events of one
        // kind can never keep them all. Cap the emitted count at `capacity`
        // and account the un-emitted remainder as dropped, so an arbitrarily
        // large `u64` counter delta cannot drive unbounded CPU work or
        // underflow a `usize` conversion.
        let emit_admitted = admitted.min(self.capacity as u64);
        let truncated = admitted.saturating_sub(emit_admitted);
        self.dropped_total = self.dropped_total.saturating_add(truncated);
        self.drops_since_report = self.drops_since_report.saturating_add(truncated);
        for _ in 0..emit_admitted {
            self.push(
                at_ms,
                "datagram.admitted",
                "debug",
                self.session_correlation(),
                "{\"error_code\":\"admitted\"}".into(),
            );
        }
        // `queue_dropped` is a subset of the generic `dropped` counter. Emit
        // `queue_full` only for that subset and classify the remainder as
        // `terminal`. Clamp the subset to the generic delta so an inconsistent
        // external counter can never emit more queue_full events than drops.
        let queue_full = queue.min(dropped);
        let terminal = dropped.saturating_sub(queue_full);
        // Bound emission at `capacity`; excess events of each kind are
        // accounted as dropped rather than looping over an unbounded delta.
        let emit_queue_full = queue_full.min(self.capacity as u64);
        let emit_terminal = terminal.min(self.capacity as u64);
        let truncated_dropped = queue_full
            .saturating_sub(emit_queue_full)
            .saturating_add(terminal.saturating_sub(emit_terminal));
        self.dropped_total = self.dropped_total.saturating_add(truncated_dropped);
        self.drops_since_report = self.drops_since_report.saturating_add(truncated_dropped);
        for code in std::iter::repeat_n("queue_full", emit_queue_full as usize)
            .chain(std::iter::repeat_n("terminal", emit_terminal as usize))
        {
            self.push(
                at_ms,
                "datagram.dropped",
                "warn",
                self.session_correlation(),
                format!("{{\"error_code\":\"{code}\"}}"),
            );
        }
        let emit_oversize = oversize.min(self.capacity as u64);
        let truncated_oversize = oversize.saturating_sub(emit_oversize);
        self.dropped_total = self.dropped_total.saturating_add(truncated_oversize);
        self.drops_since_report = self.drops_since_report.saturating_add(truncated_oversize);
        for _ in 0..emit_oversize {
            self.push(
                at_ms,
                "datagram.dropped",
                "warn",
                self.session_correlation(),
                "{\"error_code\":\"oversize\"}".into(),
            );
        }
        self.last_datagram = counters;
        // Report drops accumulated from clamped (un-emitted) deltas and any
        // real eviction during this batch, as one coalesced diagnostic.
        self.flush_drop_report(at_ms);
    }

    pub fn record_scheduler(
        &mut self,
        at_ms: u64,
        scheduler: &FairScheduler,
        selected: Option<(neko_carrier::StreamId, StreamPriority)>,
        starvation_guard: bool,
    ) {
        let snapshots: Vec<_> = scheduler.snapshots().collect();
        let queued = snapshots.iter().map(|s| s.queued_bytes as u64).sum::<u64>();
        self.max_open_streams = self.max_open_streams.max(snapshots.len() as u64);
        self.max_queued_bytes = self.max_queued_bytes.max(queued);
        if let Some((stream, priority)) = selected {
            match priority {
                StreamPriority::Interactive => {
                    self.interactive_dequeues_total =
                        self.interactive_dequeues_total.saturating_add(1)
                }
                StreamPriority::Bulk => {
                    self.bulk_dequeues_total = self.bulk_dequeues_total.saturating_add(1)
                }
            }
            let correlation = Correlation {
                session_id: self.session_id.clone(),
                stream_id: Some(format!("stream:{}", stream.0)),
                carrier_id: None,
                path_id: None,
            };
            self.push(
                at_ms,
                "scheduler.dequeued",
                "debug",
                correlation,
                format!(
                    "{{\"priority\":\"{}\",\"session_queued_bytes\":{queued}}}",
                    priority_name(priority)
                ),
            );
        }
        if starvation_guard {
            self.starvation_guard_total = self.starvation_guard_total.saturating_add(1);
            self.push(
                at_ms,
                "scheduler.starvation_guard",
                "info",
                self.session_correlation(),
                format!("{{\"session_queued_bytes\":{queued}}}"),
            );
        }
    }

    /// Oldest retained sequence, or the next sequence that will be assigned
    /// when the buffer is empty.
    fn oldest_sequence(&self) -> u64 {
        self.events
            .front()
            .map(|e| e.sequence)
            .unwrap_or(self.next_sequence)
    }

    /// Appends one event, evicting oldest-first on overflow. Every evicted
    /// retained event is real dropped evidence and increments `dropped_total`.
    /// Returns true when this append evicted an event. Never recurses into
    /// drop diagnostics — the caller decides whether to coalesce a report.
    fn push_inner(&mut self, item: Event) -> bool {
        let evicted = self.events.len() == self.capacity;
        if evicted {
            self.events.pop_front();
            self.dropped_total = self.dropped_total.saturating_add(1);
        }
        self.events.push_back(item);
        evicted
    }

    fn push(
        &mut self,
        at_ms: u64,
        event: &'static str,
        severity: &'static str,
        correlation: Correlation,
        data: String,
    ) {
        // Sequence must be strictly increasing. Once `next_sequence` reaches
        // u64::MAX the final usable value is consumed by a single
        // `resource.limit_hit` saturation marker; subsequent ordinary events
        // are dropped rather than reusing u64::MAX and producing duplicates.
        if self.next_sequence == u64::MAX {
            if !self.limit_hit_emitted {
                self.limit_hit_emitted = true;
                let limit_item = Event {
                    sequence: u64::MAX,
                    observed_at_ms: at_ms,
                    event: "resource.limit_hit",
                    severity: "error",
                    correlation: self.session_correlation(),
                    data: format!(
                        "{{\"resource\":\"event_sequence\",\"limit\":{},\"observed\":{}}}",
                        u64::MAX,
                        self.next_sequence
                    ),
                };
                self.push_inner(limit_item);
            }
            // The ordinary event could not be assigned a fresh sequence.
            self.dropped_total = self.dropped_total.saturating_add(1);
            return;
        }
        let item = Event {
            sequence: self.next_sequence,
            observed_at_ms: at_ms,
            event,
            severity,
            correlation,
            data,
        };
        self.next_sequence = self.next_sequence.saturating_add(1);
        if self.push_inner(item) {
            // A real (non-diagnostic) retained event was just evicted; count
            // it toward the next coalesced drop report.
            self.drops_since_report = self.drops_since_report.saturating_add(1);
        }
        self.flush_drop_report(at_ms);
    }

    /// Emits one coalesced `diagnostic.events_dropped` for the drops
    /// accumulated since the last report, if any. The diagnostic occupies the
    /// ring; any retained event it displaces is genuine lost evidence and is
    /// counted via `push_inner`, but the diagnostic itself does not re-report
    /// (no recursion). Its `oldest_sequence` reflects the retained floor after
    /// the diagnostic is present.
    fn flush_drop_report(&mut self, at_ms: u64) {
        if self.drops_since_report == 0 || self.next_sequence == u64::MAX {
            return;
        }
        self.drops_since_report = 0;
        // `push_inner` counts the diagnostic's own displacement of a retained
        // event as a real drop (O5); it does not recurse into another report.
        self.push_inner(Event {
            sequence: self.next_sequence,
            observed_at_ms: at_ms,
            event: "diagnostic.events_dropped",
            severity: "warn",
            correlation: self.session_correlation(),
            data: String::new(),
        });
        self.next_sequence = self.next_sequence.saturating_add(1);
        // Report the post-insertion retained floor (O6): the diagnostic is now
        // the back of the ring, so the front is the true oldest retained seq.
        let floor = self.oldest_sequence();
        let total = self.dropped_total;
        if let Some(back) = self.events.back_mut() {
            back.data = format!(
                "{{\"dropped_total\":{},\"oldest_sequence\":{}}}",
                total, floor
            );
        }
    }
    fn session_correlation(&self) -> Correlation {
        Correlation {
            session_id: self.session_id.clone(),
            stream_id: None,
            carrier_id: None,
            path_id: None,
        }
    }
    fn carrier_correlation(&self, carrier_id: u64, path: PathId) -> Correlation {
        Correlation {
            session_id: self.session_id.clone(),
            stream_id: None,
            carrier_id: Some(format!("carrier:{carrier_id}")),
            path_id: Some(format!("path:{}", path.0)),
        }
    }
}

fn path_id(key: ConcurrentPathKey) -> String {
    format!("path:{}:g{}", key.path.0, key.generation.0)
}
fn correlation_json(c: &Correlation) -> String {
    let mut out = format!("{{\"session_id\":\"{}\"", c.session_id);
    if let Some(v) = &c.stream_id {
        out.push_str(&format!(",\"stream_id\":\"{v}\""));
    }
    if let Some(v) = &c.carrier_id {
        out.push_str(&format!(",\"carrier_id\":\"{v}\""));
    }
    if let Some(v) = &c.path_id {
        out.push_str(&format!(",\"path_id\":\"{v}\""));
    }
    out.push('}');
    out
}
fn health_name(v: HealthState) -> &'static str {
    match v {
        HealthState::Unknown => "unknown",
        HealthState::Healthy => "healthy",
        HealthState::Degraded => "degraded",
        HealthState::Failed => "failed",
    }
}
fn priority_name(v: StreamPriority) -> &'static str {
    match v {
        StreamPriority::Interactive => "interactive",
        StreamPriority::Bulk => "bulk",
    }
}
fn carrier_kind(v: neko_carrier::CarrierKind) -> &'static str {
    match v {
        neko_carrier::CarrierKind::Udp => "udp",
        neko_carrier::CarrierKind::Tcp => "tcp",
        _ => "other",
    }
}
fn switch_reason(v: SwitchReason) -> &'static str {
    match v {
        SwitchReason::UdpBlackhole => "pto_threshold",
        SwitchReason::UdpPathDegraded => "health_failed",
        SwitchReason::TcpReadyPreferred => "migration_preferred",
        SwitchReason::AddressChange => "path_unavailable",
        SwitchReason::OperatorRequest => "operator_requested",
        SwitchReason::CarrierError => "carrier_error",
        _ => "recovery_hysteresis",
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use neko_carrier::{CarrierKind, ConcurrentLimits, PathGeneration};
    use neko_reliable::{AckRange, AckRanges, FrameId, SentPacket};
    use neko_session::DatagramRuntime;

    #[test]
    fn runtime_facts_are_correlated_secret_free_and_bounded() {
        let mut p = Producer::new(SessionId(7), 8).unwrap();
        p.record_health(
            1,
            1,
            PathId(2),
            Some(HealthState::Healthy),
            HealthState::Degraded,
            HealthSample {
                rtt_us: 12_000,
                loss_per_mille: 600,
                pto: 3,
            },
        );
        let mut r = Recovery::new(4, 2).unwrap();
        r.on_sent(SentPacket {
            number: 1,
            sent_at_us: 0,
            bytes: 10,
            ack_eliciting: true,
            frames: vec![FrameId(1)],
        })
        .unwrap();
        r.on_sent(SentPacket {
            number: 4,
            sent_at_us: 10,
            bytes: 20,
            ack_eliciting: true,
            frames: vec![FrameId(4)],
        })
        .unwrap();
        let ack = AckRanges::from_ranges(4, &[AckRange { start: 4, end: 4 }]).unwrap();
        let result = r.on_ack(&ack, 20_000, 0).unwrap();
        p.record_recovery_ack(20, 1, PathId(2), &r, &result);
        let probes = r.on_pto(1).unwrap();
        p.record_pto(21, 1, PathId(2), &r, probes.len());
        assert_eq!(p.retransmit_totals(), (1, 10));
        assert!(p.events().all(|e| !e.to_json_line().contains("payload")));
        assert!(p.events().all(|e| e.to_json_line().len() < 16 * 1024));
    }

    #[test]
    fn switch_datagram_scheduler_and_eviction_are_observed() {
        // Capacity sized so the burst overflows once, emitting one coalesced
        // `diagnostic.events_dropped`, while the asserted datagram/scheduler
        // evidence is still retained.
        let mut p = Producer::new(SessionId(9), 5).unwrap();
        let mut m = ConcurrentCarrierManager::new(ConcurrentLimits {
            k_ready: 1,
            ..Default::default()
        })
        .unwrap();
        let key = ConcurrentPathKey {
            path: PathId(3),
            generation: PathGeneration(1),
        };
        m.register(key, CarrierKind::Udp).unwrap();
        m.observe_readiness(key, true, true, 1).unwrap();
        let switch = m
            .activate(key, SwitchReason::OperatorRequest, 2, true)
            .unwrap();
        p.record_switch(&m, switch);
        let mut d = DatagramRuntime::new(1, 2).unwrap();
        d.send(b"ok").unwrap();
        assert!(d.send(b"x").is_err());
        assert!(d.send(b"big").is_err());
        p.record_datagrams(3, d.counters());
        let mut scheduler = FairScheduler::new(Default::default()).unwrap();
        scheduler
            .open(neko_carrier::StreamId(4), StreamPriority::Bulk)
            .unwrap();
        scheduler
            .enqueue(neko_carrier::StreamId(4), b"abc")
            .unwrap();
        p.record_scheduler(
            4,
            &scheduler,
            Some((neko_carrier::StreamId(4), StreamPriority::Bulk)),
            true,
        );
        // `dropped_total` counts every retained event evicted on overflow,
        // including the retained event displaced to make room for the drop
        // diagnostic itself (O5). The diagnostic never counts itself.
        assert_eq!(p.retained(), 5);
        assert_eq!(p.dropped_total(), 2);
        assert_eq!(p.dequeue_totals(), (0, 1, 1));
        assert_eq!(p.resource_high_water(), (1, 3));
        let lines: Vec<_> = p.events().map(Event::to_json_line).collect();
        assert!(lines.iter().any(|line| line.contains("datagram.dropped")));
        assert!(
            lines
                .iter()
                .any(|line| line.contains("scheduler.starvation_guard"))
        );
    }

    #[test]
    fn datagram_mixed_drop_window_classifies_queue_and_terminal_separately() {
        use neko_session::DatagramRuntime;
        let mut p = Producer::new(neko_session::SessionId(9), 16).unwrap();
        let mut d = DatagramRuntime::new(1, 64).unwrap();
        // One queue-full drop.
        d.send(b"a").unwrap();
        assert_eq!(d.send(b"b"), Err(neko_session::DatagramError::QueueFull));
        // One terminal drop (send after close): counted in `dropped` but not
        // in `queue_dropped`.
        d.close();
        assert_eq!(d.send(b"c"), Err(neko_session::DatagramError::Terminal));
        let counters = d.counters();
        assert_eq!(counters.dropped, 2);
        assert_eq!(counters.queue_dropped, 1);
        p.record_datagrams(1, counters);
        let lines: Vec<_> = p.events().map(Event::to_json_line).collect();
        let queue_full = lines
            .iter()
            .filter(|l| l.contains("\"error_code\":\"queue_full\""))
            .count();
        let terminal = lines
            .iter()
            .filter(|l| l.contains("\"error_code\":\"terminal\""))
            .count();
        // queue_dropped is a subset of dropped: exactly one queue_full, and the
        // remaining generic dropped delta is terminal, not mislabelled.
        assert_eq!(queue_full, 1, "lines={lines:?}");
        assert_eq!(terminal, 1, "lines={lines:?}");
    }

    #[test]
    fn sequence_exhaustion_emits_limit_hit_and_never_duplicates() {
        // Drive next_sequence to the u64::MAX boundary, then overflow with
        // further pushes. The final usable sequence must be a single
        // `resource.limit_hit`; later ordinary events must not reuse u64::MAX.
        let mut p = Producer::new(SessionId(9), 4).unwrap();
        p.next_sequence = u64::MAX;
        // Three pushes past the boundary.
        for _ in 0..3 {
            let c = p.session_correlation();
            p.push(1, "session.started", "info", c, "{}".into());
        }
        let seqs: Vec<u64> = p.events().map(|e| e.sequence).collect();
        // No duplicate sequence values; strictly increasing.
        let mut sorted = seqs.clone();
        sorted.dedup();
        assert_eq!(seqs.len(), sorted.len(), "seqs={seqs:?}");
        assert!(seqs.windows(2).all(|w| w[0] < w[1]), "seqs={seqs:?}");
        assert!(
            p.events()
                .any(|e| e.event == "resource.limit_hit" && e.sequence == u64::MAX)
        );
        // O4: limit_hit data must be schema-valid — `limit` is the integer
        // counter u64::MAX, not the string "u64_max"; resource uses the
        // append-only `event_sequence` enum value.
        let hit = p
            .events()
            .find(|e| e.event == "resource.limit_hit")
            .unwrap();
        assert!(
            hit.data.contains("\"resource\":\"event_sequence\""),
            "{}",
            hit.data
        );
        assert!(
            hit.data.contains("\"limit\":18446744073709551615"),
            "{}",
            hit.data
        );
        assert!(!hit.data.contains("u64_max\""), "{}", hit.data);
    }

    #[test]
    fn ring_overflow_emits_events_dropped_with_floor() {
        // Small ring; force overflow so a coalesced diagnostic.events_dropped
        // is emitted carrying dropped_total and the retained sequence floor.
        let mut p = Producer::new(SessionId(9), 3).unwrap();
        // 6 admitted events -> more than capacity 3 -> overflow + report.
        p.record_datagrams(
            1,
            DatagramCounters {
                admitted: 6,
                ..Default::default()
            },
        );
        let lines: Vec<_> = p.events().map(Event::to_json_line).collect();
        assert!(
            lines
                .iter()
                .any(|l| l.contains("diagnostic.events_dropped")),
            "lines={lines:?}"
        );
        // sequences still strictly increasing
        let seqs: Vec<u64> = p.events().map(|e| e.sequence).collect();
        assert!(seqs.windows(2).all(|w| w[0] < w[1]), "seqs={seqs:?}");
        // O6: the diagnostic's oldest_sequence must equal the actual oldest
        // retained sequence AFTER the diagnostic is present (post-insertion
        // floor), and it must carry only schema-valid dropped_total +
        // oldest_sequence (no dropped_since_last).
        let diag = p
            .events()
            .find(|e| e.event == "diagnostic.events_dropped")
            .expect("drop diagnostic present");
        let actual_floor = p.events().next().unwrap().sequence;
        assert!(
            diag.data
                .contains(&format!("\"oldest_sequence\":{}", actual_floor)),
            "diag={} actual_floor={}",
            diag.data,
            actual_floor
        );
        assert!(!diag.data.contains("dropped_since_last"), "{}", diag.data);
    }

    #[test]
    fn datagram_inconsistent_queue_delta_is_bounded_conservatively() {
        use neko_session::DatagramCounters;
        let mut p = Producer::new(neko_session::SessionId(9), 16).unwrap();
        // Externally supplied counters that are internally inconsistent
        // (queue_dropped delta exceeds the generic dropped delta) must be
        // projected conservatively: never emit more queue_full events than the
        // generic dropped delta, and never underflow.
        let counters = DatagramCounters {
            dropped: 1,
            queue_dropped: 3,
            ..Default::default()
        };
        p.record_datagrams(1, counters);
        let lines: Vec<_> = p.events().map(Event::to_json_line).collect();
        let queue_full = lines
            .iter()
            .filter(|l| l.contains("\"error_code\":\"queue_full\""))
            .count();
        let terminal = lines
            .iter()
            .filter(|l| l.contains("\"error_code\":\"terminal\""))
            .count();
        assert_eq!(queue_full, 1, "lines={lines:?}");
        assert_eq!(terminal, 0, "lines={lines:?}");
    }

    #[test]
    fn datagram_huge_delta_emits_bounded_events() {
        use neko_session::DatagramCounters;
        // A counter delta far beyond ring capacity must not drive unbounded
        // per-event work: emission is capped at capacity and the un-emitted
        // remainder is accounted as dropped. This must terminate quickly.
        let mut p = Producer::new(neko_session::SessionId(9), 16).unwrap();
        let counters = DatagramCounters {
            admitted: u64::MAX / 2,
            dropped: u64::MAX / 2,
            rejected_oversize: u64::MAX / 2,
            queue_dropped: u64::MAX / 4,
            ..Default::default()
        };
        p.record_datagrams(1, counters);
        // Ring retains at most `capacity` events; the rest are dropped_total.
        assert!(p.events().count() <= 16);
        assert!(p.dropped_total() > 0);
    }
}
