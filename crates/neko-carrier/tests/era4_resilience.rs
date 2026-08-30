//! Bounded Era-4 E resilience slice.
//!
//! These tests deliberately exercise repeated in-process lifecycles. They do
//! not claim persistence across a process restart: no durable session store or
//! restart/reconnect protocol is present in this repository.

use neko_carrier::{DataId, FailoverController};
use neko_session::{
    InboundRecord, RuntimeLimits, RuntimeState, SessionId, SessionRuntime, StreamId,
};

fn limits() -> RuntimeLimits {
    RuntimeLimits {
        max_streams: 1,
        max_queue_records: 4,
        max_queue_bytes: 64,
        max_total_bytes: 64,
        max_record_bytes: 16,
        max_session_window: 64,
        max_stream_window: 64,
        idle_timeout_ms: 100,
        close_timeout_ms: 5,
    }
}

#[test]
fn repeated_stream_open_close_and_peer_disappearance_are_bounded() {
    for cycle in 0..64u64 {
        let mut runtime = SessionRuntime::new(SessionId(cycle), limits(), 0).unwrap();
        runtime.open_stream(StreamId(1), cycle).unwrap();
        let offset = runtime.queue_send(StreamId(1), b"ok", cycle + 1).unwrap();
        let sent = runtime.pop_send(cycle + 2).unwrap().unwrap();
        assert_eq!(sent.offset, offset);
        runtime
            .receive(
                InboundRecord {
                    stream: sent.stream,
                    offset: sent.offset,
                    data: sent.data,
                },
                cycle + 3,
            )
            .unwrap();
        assert_eq!(runtime.pop_receive(cycle + 4).unwrap().unwrap().data, b"ok");
        runtime.close_graceful(cycle + 5).unwrap();
        runtime.tick(cycle + 11).unwrap();
        assert_eq!(runtime.state(), RuntimeState::Closed);
        // A peer disappearing after local closure is idempotent and cannot
        // resurrect queues or streams.
        runtime.close_remote(cycle + 12).unwrap();
        assert_eq!(runtime.queued_records(), 0);
        assert_eq!(runtime.queued_bytes(), 0);
    }
}

#[test]
fn repeated_failover_recovery_cycles_clear_uncertain_data() {
    for cycle in 0..32u64 {
        let mut failover = FailoverController::new(2, 4, 64).unwrap();
        let id = DataId(cycle);
        failover.track_uncertain(id, b"replay-once").unwrap();
        assert!(failover.udp_pto_at(20));
        assert_eq!(
            failover.tcp_resend().unwrap(),
            vec![(id, b"replay-once".to_vec())]
        );
        failover.confirm(id).unwrap();
        assert!(failover.tcp_resend().unwrap().is_empty());
        assert!(!failover.udp_pto_at(21));
    }
}
