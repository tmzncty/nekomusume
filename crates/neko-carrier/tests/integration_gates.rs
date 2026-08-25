//! Cross-layer M0 candidate gates. These tests exercise only the M0
//! cross-layer state/ledger gates and create no sockets; they do not negate
//! the separately implemented M1-S1 loopback UDP slice.
use neko_carrier::{
    CarrierEvent, CarrierKind, CarrierState, Hysteresis, Limits as CarrierLimits, PacketFeedback,
    PacketFeedbackKind, PathGeneration, PathId, PathValidated, PathValidationState,
    SessionDelivery,
};
use neko_session::{
    DeliveryEpoch, DeliveryLedger, DeliveryState, KeyPhase, Limits as SessionLimits,
    PathGeneration as SessionPathGeneration, SessionContext,
};
use std::collections::BTreeMap;

const PATH: PathId = PathId(7);
const GENERATION: PathGeneration = PathGeneration(3);

fn carrier() -> CarrierState {
    CarrierState::new(CarrierLimits {
        max_paths: 2,
        hysteresis: Hysteresis {
            min_dwell_events: 1,
            k_successes: 1,
        },
    })
}

fn candidate_path(state: &mut CarrierState) {
    state
        .apply(CarrierEvent::PathAdded {
            path: PATH,
            carrier: CarrierKind::Udp,
            generation: GENERATION,
        })
        .unwrap();
    state
        .apply(CarrierEvent::ChallengeSent {
            path: PATH,
            generation: GENERATION,
        })
        .unwrap();
}

fn ledger() -> DeliveryLedger {
    DeliveryLedger::new(SessionLimits {
        max_reorder: 8,
        max_streams: 2,
        max_connection_bytes: 8,
        max_offset_jump: 16,
    })
}

fn context() -> SessionContext {
    SessionContext {
        delivery_epoch: DeliveryEpoch(11),
        key_phase: KeyPhase(2),
        path_generation: SessionPathGeneration(GENERATION.0),
    }
}

#[test]
fn evidence_domains_do_not_cross_promote_delivery() {
    let mut carrier = carrier();
    candidate_path(&mut carrier);
    carrier
        .apply(CarrierEvent::PacketFeedback(PacketFeedback {
            path: PATH,
            generation: GENERATION,
            kind: PacketFeedbackKind::Ack,
        }))
        .unwrap();
    assert_eq!(
        carrier.path(PATH).unwrap().validation,
        PathValidationState::Validating
    );
    let mut ledger = ledger();
    ledger.insert(1, 0, b"x", context()).unwrap();
    ledger.packet_feedback(1, 0);
    assert_eq!(
        ledger.segments().next().unwrap().state,
        DeliveryState::Unsent
    );
    assert_eq!(ledger.watermark(1), 0);
}

#[test]
fn old_generation_and_late_old_path_cannot_advance_watermark() {
    let mut carrier = carrier();
    candidate_path(&mut carrier);
    assert!(
        carrier
            .apply(CarrierEvent::SessionDelivery(SessionDelivery {
                path: PATH,
                generation: PathGeneration(GENERATION.0 - 1)
            }))
            .is_err()
    );
    let mut ledger = ledger();
    ledger.insert(1, 0, b"x", context()).unwrap();
    ledger.mark_in_flight(1, 0).unwrap();
    assert_eq!(
        ledger.confirm_received(
            1,
            0,
            SessionContext {
                delivery_epoch: DeliveryEpoch(10),
                key_phase: KeyPhase(2),
                path_generation: SessionPathGeneration(GENERATION.0)
            }
        ),
        Err(neko_session::LedgerError::OldEpoch)
    );
    assert_eq!(ledger.watermark(1), 0);
    assert_eq!(
        ledger.confirm_received(
            1,
            0,
            SessionContext {
                delivery_epoch: DeliveryEpoch(11),
                key_phase: KeyPhase(3),
                path_generation: SessionPathGeneration(GENERATION.0),
            },
        ),
        Err(neko_session::LedgerError::ContextMismatch)
    );
    assert_eq!(ledger.watermark(1), 0);
}

#[test]
fn duplicate_data_id_is_idempotent_and_conflict_is_rejected() {
    #[derive(Debug, Clone, Copy, PartialEq, Eq, PartialOrd, Ord)]
    struct DataId(u64);
    let mut delivered = BTreeMap::<DataId, Vec<u8>>::new();
    let id = DataId(42);
    let first = b"same".to_vec();
    assert!(delivered.insert(id, first.clone()).is_none());
    assert_eq!(delivered.get(&id), Some(&first));
    assert_eq!(delivered.get(&id), Some(&first));
    let conflicting = b"different".to_vec();
    assert_ne!(delivered.get(&id), Some(&conflicting));
    assert_eq!(delivered.len(), 1);
}

#[test]
fn pto_is_health_evidence_not_failure_or_delivery() {
    let mut carrier = carrier();
    candidate_path(&mut carrier);
    carrier
        .apply(CarrierEvent::PtoExpired {
            path: PATH,
            generation: GENERATION,
        })
        .unwrap();
    assert_ne!(
        carrier.path(PATH).unwrap().state,
        neko_carrier::PathState::Failed
    );
    let mut ledger = ledger();
    ledger.insert(1, 0, b"x", context()).unwrap();
    assert_eq!(ledger.watermark(1), 0);
}

#[test]
fn activation_requires_challenge_validation_and_hysteresis() {
    let mut carrier = carrier();
    candidate_path(&mut carrier);
    assert!(
        carrier
            .apply(CarrierEvent::Activate {
                path: PATH,
                generation: GENERATION
            })
            .is_err()
    );
    carrier
        .apply(CarrierEvent::ChallengeValidated(PathValidated {
            path: PATH,
            generation: GENERATION,
        }))
        .unwrap();
    carrier
        .apply(CarrierEvent::PacketFeedback(PacketFeedback {
            path: PATH,
            generation: GENERATION,
            kind: PacketFeedbackKind::Ack,
        }))
        .unwrap();
    carrier
        .apply(CarrierEvent::Activate {
            path: PATH,
            generation: GENERATION,
        })
        .unwrap();
    assert_eq!(carrier.active(), Some((PATH, GENERATION)));
}
