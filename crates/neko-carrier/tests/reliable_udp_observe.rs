//! S9 — layered observability for live packet recovery.
//!
//! `PathRecovery` packet-level evidence maps onto the existing `neko-observe`
//! recovery/carrier events: `recovery.rtt_updated`, `recovery.loss_detected`,
//! `recovery.frame_retransmitted`, `recovery.pto_fired`, `carrier.health_sample`
//! and `carrier.switch_completed` stay layered — a packet-recovery event is
//! never mislabeled as logical Session delivery.
use neko_carrier::{
    CarrierKind, ConcurrentCarrierManager, ConcurrentLimits, ConcurrentPathKey, PathGeneration,
    PathId, PathRecovery, SwitchReason,
};
use neko_observe::Producer;
use neko_reliable::{AckRanges, FrameId, SentPacket};
use neko_session::SessionId;

const UDP: ConcurrentPathKey = ConcurrentPathKey {
    path: PathId(1),
    generation: PathGeneration(1),
};
const TCP: ConcurrentPathKey = ConcurrentPathKey {
    path: PathId(2),
    generation: PathGeneration(1),
};

fn sent(n: u64) -> SentPacket {
    SentPacket {
        number: n,
        sent_at_us: n * 1_000,
        bytes: 400,
        ack_eliciting: true,
        frames: vec![FrameId(n)],
    }
}

#[test]
fn packet_recovery_events_are_layered_and_not_session_delivery() {
    let mut obs = Producer::new(SessionId(1), 64).unwrap();
    let mut recovery = PathRecovery::new(PathId(1), 1, 1200).unwrap();

    // Sent packets -> RTT sample on ACK; one loss -> loss_detected + retransmit.
    for n in 0..8u64 {
        recovery.on_sent(sent(n)).unwrap();
    }
    let mut ack = AckRanges::new(8).unwrap();
    ack.insert(7).unwrap();
    let out = recovery.on_ack(1, &ack, 40_000, 0).unwrap();

    // Map the raw engine result + health sample onto the observer.
    obs.record_recovery_ack(10, 1, PathId(1), recovery.recovery(), &out.result);
    obs.record_health(
        11,
        1,
        PathId(1),
        None,
        neko_carrier::HealthState::Degraded,
        recovery.health_sample(),
    );
    // A PTO probe produces recovery.pto_fired.
    let probes = recovery.on_pto(2).unwrap();
    obs.record_pto(12, 1, PathId(1), recovery.recovery(), probes.len());

    // A degradation-driven switch emits carrier.switch_completed, not a
    // Session delivery event.
    let mut m = ConcurrentCarrierManager::new(ConcurrentLimits::default()).unwrap();
    m.register(UDP, CarrierKind::Udp).unwrap();
    m.register(TCP, CarrierKind::Tcp).unwrap();
    for i in 0..3 {
        m.observe_readiness(UDP, true, true, i).unwrap();
        m.observe_readiness(TCP, true, true, i).unwrap();
    }
    m.activate(UDP, SwitchReason::OperatorRequest, 2, false)
        .unwrap();
    m.fail(UDP, SwitchReason::UdpPathDegraded, 10).unwrap();
    let ev = m
        .activate(TCP, SwitchReason::UdpPathDegraded, 11, true)
        .unwrap();
    obs.record_switch(&m, ev);

    let events: Vec<String> = obs.events().map(|e| e.event.to_string()).collect();
    let text = events.join("\n");
    // Packet-recovery layer is present and distinct.
    assert!(text.contains("recovery.loss_detected"), "{text}");
    assert!(text.contains("recovery.pto_fired"), "{text}");
    // Health + switch are carrier-layer events.
    assert!(text.contains("carrier."), "{text}");
    // No Session delivery was produced anywhere in this seam.
    assert!(!text.contains("delivery"), "{text}");
    assert!(!text.contains("confirm_received"), "{text}");
}
