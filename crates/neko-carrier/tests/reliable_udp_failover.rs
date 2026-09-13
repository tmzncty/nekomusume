//! S8 — packet-recovery-driven UDP degradation -> existing warm TCP fallback.
//!
//! Instead of the old application-reply-cessation trigger, a real packet-loss
//! condition in `PathRecovery` produces a measured `HealthSample` that degrades
//! the Carrier path; the existing `ConcurrentCarrierManager` then fails UDP and
//! activates a ready warm TCP standby, replaying the uncertain application
//! bytes exactly once. Packet ACKs never validate a Path or confirm Session
//! delivery — they only feed packet-level health evidence.
use neko_carrier::{
    CarrierHealth, CarrierKind, ConcurrentCarrierManager, ConcurrentLimits, ConcurrentPathKey,
    HealthLimits, LogicalRangeId, PathGeneration, PathId, PathRecovery, SwitchReason,
};
use neko_reliable::{AckRanges, FrameId, SentPacket};

const UDP: ConcurrentPathKey = ConcurrentPathKey {
    path: PathId(1),
    generation: PathGeneration(1),
};
const TCP: ConcurrentPathKey = ConcurrentPathKey {
    path: PathId(2),
    generation: PathGeneration(1),
};
const RANGE: LogicalRangeId = LogicalRangeId {
    stream: 1,
    offset: 7,
};

fn manager() -> ConcurrentCarrierManager {
    let mut m = ConcurrentCarrierManager::new(ConcurrentLimits::default()).unwrap();
    m.register(UDP, CarrierKind::Udp).unwrap();
    m.register(TCP, CarrierKind::Tcp).unwrap();
    m
}
fn warm(m: &mut ConcurrentCarrierManager, key: ConcurrentPathKey, at: u64) {
    for i in 0..3 {
        m.observe_readiness(key, true, true, at + i).unwrap();
    }
}
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
fn packet_loss_drives_udp_degrade_then_warm_tcp_fallback_replays_uncertain_once() {
    let mut m = manager();
    let mut health = CarrierHealth::new(HealthLimits {
        degrade_after: 2,
        fail_after: 4,
        recover_after: 2,
        max_paths: 8,
    })
    .unwrap();
    let mut recovery = PathRecovery::new(PathId(1), 1, 1200).unwrap();

    // UDP is warm + active; TCP standby proves readiness independently.
    warm(&mut m, UDP, 0);
    m.activate(UDP, SwitchReason::OperatorRequest, 2, false)
        .unwrap();
    warm(&mut m, TCP, 3);
    m.assign(RANGE, b"payload").unwrap();

    // Real packet loss across THREE distinct recovery-evidence windows: each
    // round sends packets and ACKs only the largest so the rest are lost. The
    // fresh-evidence bridge yields one bad health observation per new evidence
    // epoch — polling the same cumulative snapshot returns None and cannot be
    // replayed into several bad observations.
    let mut packet_no = 0u64;
    let mut rounds = 0u32;
    let mut bad_obs = 0u32;
    let mut poll_only_none = 0u32;
    loop {
        for _ in 0..8 {
            recovery.on_sent(sent(packet_no)).unwrap();
            packet_no += 1;
        }
        let mut ack = AckRanges::new(8).unwrap();
        ack.insert(packet_no - 1).unwrap();
        recovery.on_ack(1, &ack, packet_no * 40_000, 0).unwrap();
        if let Some(s) = recovery.fresh_health_sample() {
            health.observe(PathId(1), s).unwrap();
            bad_obs += 1;
        }
        // A second poll with no new evidence yields nothing (fresh-evidence).
        if recovery.fresh_health_sample().is_none() {
            poll_only_none += 1;
        }
        rounds += 1;
        if health.path(PathId(1)).unwrap().state == neko_carrier::HealthState::Degraded {
            break;
        }
        if rounds > 12 {
            panic!("did not degrade under distinct bad evidence");
        }
    }
    assert!(
        bad_obs >= 2,
        "degrade needs >= degrade_after bad observations"
    );
    // Every evidence epoch produced exactly one observation; the redundant
    // no-evidence polls produced none — no snapshot is replayed.
    assert_eq!(poll_only_none, bad_obs, "no epoch replays as extra bad obs");
    assert_eq!(
        health.path(PathId(1)).unwrap().state,
        neko_carrier::HealthState::Degraded,
        "distinct new bad evidence crosses hysteresis -> Degraded"
    );

    // Degradation fails the active UDP path; the already-ready warm TCP
    // standby is promoted and the uncertain application bytes replay once.
    m.fail(UDP, SwitchReason::UdpPathDegraded, 10).unwrap();
    let event = m
        .activate(TCP, SwitchReason::UdpPathDegraded, 11, true)
        .unwrap();
    assert_eq!(
        event.recovery_class,
        Some(neko_carrier::RecoveryClass::Warm)
    );
    let replayed = m.replay_uncertain().unwrap();
    assert_eq!(
        replayed,
        vec![neko_carrier::ReplayRange {
            id: RANGE,
            bytes: b"payload".to_vec()
        }]
    );
    // Second replay yields nothing — uncertain state drains exactly once.
    assert!(m.replay_uncertain().unwrap().is_empty());
}

#[test]
fn clean_recovery_evidence_does_not_trigger_fallback() {
    let mut m = manager();
    let mut health = CarrierHealth::new(HealthLimits::default()).unwrap();
    let mut recovery = PathRecovery::new(PathId(1), 1, 1200).unwrap();
    warm(&mut m, UDP, 0);
    m.activate(UDP, SwitchReason::OperatorRequest, 2, false)
        .unwrap();
    for n in 0..4u64 {
        recovery.on_sent(sent(n)).unwrap();
    }
    let mut ack = AckRanges::new(8).unwrap();
    for n in 0..4u64 {
        ack.insert(n).unwrap();
    }
    recovery.on_ack(1, &ack, 40_000, 0).unwrap();
    let sample = recovery.health_sample();
    assert_eq!(sample.loss_per_mille, 0);
    // A clean packet-recovery sample keeps the path Healthy — no fallback.
    assert_eq!(
        health.observe(PathId(1), sample).unwrap(),
        neko_carrier::HealthState::Healthy
    );
    assert_eq!(m.active(), Some(UDP));
}
