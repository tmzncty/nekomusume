//! Deterministic, socket-free M3 long-session and migration gates.
use neko_carrier::{
    CarrierManager, HealthSample, ManagerLimits, MigrationCandidate, MigrationError,
    PathGeneration, PathId,
};
use neko_crypto::{CryptoError, NonceManager};
use neko_session::{
    DeliveryEpoch, DeliveryLedger, DeliveryState, InboundRecord, KeyPhase, Limits as LedgerLimits,
    RuntimeLimits, RuntimeState, SessionContext, SessionId, SessionRuntime, StreamId,
};

fn context(path: u64) -> SessionContext {
    SessionContext {
        delivery_epoch: DeliveryEpoch(1),
        key_phase: KeyPhase(0),
        path_generation: neko_session::PathGeneration(path),
    }
}

#[test]
fn bounded_long_session_preserves_exact_order_through_uncertain_ack_and_close() {
    const N: u64 = 512;
    let runtime_limits = RuntimeLimits {
        max_streams: 1,
        max_queue_records: N as usize,
        max_queue_bytes: N as usize,
        max_total_bytes: N as usize,
        max_record_bytes: 1,
        max_session_window: 512,
        max_stream_window: 512,
        idle_timeout_ms: 10_000,
        close_timeout_ms: 10,
    };
    let mut sender = SessionRuntime::new(SessionId(9), runtime_limits, 0).unwrap();
    let mut receiver = SessionRuntime::new(SessionId(9), runtime_limits, 0).unwrap();
    sender.open_stream(StreamId(1), 1).unwrap();
    receiver.open_stream(StreamId(1), 1).unwrap();
    let mut ledger = DeliveryLedger::new(LedgerLimits {
        max_reorder: N,
        max_streams: 1,
        max_connection_bytes: N as usize,
        max_offset_jump: N,
    });
    for n in 0..N {
        let byte = [n as u8];
        let offset = sender.queue_send(StreamId(1), &byte, n + 2).unwrap();
        assert_eq!(offset, n);
        ledger.insert(1, offset, &byte, context(1)).unwrap();
        ledger.mark_in_flight(1, offset).unwrap();
        if n >= N / 2 {
            ledger.mark_uncertain(1, offset).unwrap();
        }
        let record = sender.pop_send(n + 3).unwrap().unwrap();
        receiver
            .receive(
                InboundRecord {
                    stream: record.stream,
                    offset: record.offset,
                    data: record.data,
                },
                n + 4,
            )
            .unwrap();
        if n < N / 2 {
            ledger.confirm_received(1, offset, context(1)).unwrap();
        }
    }
    let mut ordered = Vec::with_capacity(N as usize);
    while let Some(record) = receiver.pop_receive(1_000).unwrap() {
        ordered.extend(record.data);
    }
    assert_eq!(ordered, (0..N).map(|n| n as u8).collect::<Vec<_>>());
    for n in N / 2..N {
        ledger.confirm_received(1, n, context(2)).unwrap();
    }
    assert!(
        ledger
            .segments()
            .all(|segment| segment.state == DeliveryState::Confirmed)
    );
    sender.close_graceful(2_000).unwrap();
    sender.tick(2_010).unwrap();
    assert_eq!(sender.state(), RuntimeState::Closed);
    assert_eq!(sender.queued_records(), 0);
    assert_eq!(sender.queued_bytes(), 0);
}

#[test]
fn migration_requires_validation_current_generation_and_hold_then_switches_once() {
    let mut manager = CarrierManager::new(ManagerLimits {
        min_hold_events: 2,
        switch_margin: 10,
        max_paths: 2,
    })
    .unwrap();
    manager
        .observe(
            PathId(1),
            HealthSample {
                rtt_us: 500,
                loss_per_mille: 0,
                pto: 0,
            },
        )
        .unwrap();
    manager
        .observe(
            PathId(2),
            HealthSample {
                rtt_us: 10,
                loss_per_mille: 0,
                pto: 0,
            },
        )
        .unwrap();
    manager
        .set_active_tcp(PathId(1), PathGeneration(7))
        .unwrap();
    let candidate = MigrationCandidate {
        path: PathId(2),
        generation: PathGeneration(7),
        validated: true,
        health: HealthSample {
            rtt_us: 10,
            loss_per_mille: 0,
            pto: 0,
        },
    };
    assert_eq!(
        manager.migrate_back_to_udp(MigrationCandidate {
            validated: false,
            ..candidate
        }),
        Err(MigrationError::Unvalidated)
    );
    assert_eq!(
        manager.migrate_back_to_udp(MigrationCandidate {
            generation: PathGeneration(6),
            ..candidate
        }),
        Err(MigrationError::OldGeneration)
    );
    assert_eq!(
        manager.migrate_back_to_udp(candidate),
        Err(MigrationError::HoldGate)
    );
    assert_eq!(
        manager.migrate_back_to_udp(candidate),
        Err(MigrationError::HoldGate)
    );
    assert_eq!(manager.migrate_back_to_udp(candidate), Ok(true));
    assert_eq!(manager.active(), Some(PathId(2)));
    assert_eq!(manager.switches, 1);
    // A late/replayed old candidate cannot switch again.
    assert_eq!(
        manager.migrate_back_to_udp(candidate),
        Err(MigrationError::NotTcp)
    );
}

#[test]
fn sequence_and_counter_exhaustion_are_fail_closed() {
    let mut nonce = NonceManager::new(u64::MAX - 1);
    assert_eq!(nonce.next_nonce(), Ok(u64::MAX - 1));
    assert_eq!(nonce.next_nonce(), Ok(u64::MAX));
    assert_eq!(nonce.next_nonce(), Err(CryptoError::NonceExhausted));
    let mut ledger = DeliveryLedger::new(LedgerLimits {
        max_reorder: u64::MAX,
        max_streams: 1,
        max_connection_bytes: 8,
        max_offset_jump: u64::MAX,
    });
    assert_eq!(
        ledger.insert(1, u64::MAX, b"xx", context(1)),
        Err(neko_session::LedgerError::OffsetOverflow)
    );
}
