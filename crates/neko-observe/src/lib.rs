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
        for _ in 0..admitted {
            self.push(
                at_ms,
                "datagram.admitted",
                "debug",
                self.session_correlation(),
                "{\"error_code\":\"admitted\"}".into(),
            );
        }
        for _ in 0..dropped {
            let code = if queue > 0 { "queue_full" } else { "terminal" };
            self.push(
                at_ms,
                "datagram.dropped",
                "warn",
                self.session_correlation(),
                format!("{{\"error_code\":\"{code}\"}}"),
            );
        }
        for _ in 0..oversize {
            self.push(
                at_ms,
                "datagram.dropped",
                "warn",
                self.session_correlation(),
                "{\"error_code\":\"oversize\"}".into(),
            );
        }
        self.last_datagram = counters;
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

    fn push(
        &mut self,
        at_ms: u64,
        event: &'static str,
        severity: &'static str,
        correlation: Correlation,
        data: String,
    ) {
        let item = Event {
            sequence: self.next_sequence,
            observed_at_ms: at_ms,
            event,
            severity,
            correlation,
            data,
        };
        self.next_sequence = self.next_sequence.saturating_add(1);
        if self.events.len() == self.capacity {
            self.events.pop_front();
            self.dropped_total = self.dropped_total.saturating_add(1);
        }
        self.events.push_back(item);
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
        let mut p = Producer::new(SessionId(9), 4).unwrap();
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
        assert_eq!(p.retained(), 4);
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
}
