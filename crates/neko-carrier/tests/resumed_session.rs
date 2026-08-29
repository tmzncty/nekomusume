use neko_crypto::{
    InitiatorHandshake, LocalIdentity, RecordContext, ResponderHandshake, ResumeBinding,
    ResumeGuard, TrustPolicy, TrustRecord, TrustStatus,
};
use neko_session::{InboundRecord, RuntimeLimits, SessionId, SessionRuntime, StreamId};

fn ctx(path_generation: u64) -> RecordContext {
    RecordContext {
        delivery_epoch: 1,
        key_phase: 0,
        path_generation,
        stream_id: 1,
        direction: 0,
    }
}
fn handshake(
    client: &LocalIdentity,
    server: &LocalIdentity,
    policy: TrustPolicy,
    binding: Option<&ResumeBinding>,
    generation: u64,
) -> (
    neko_crypto::SecureSession,
    neko_crypto::SecureSession,
    Option<(Vec<u8>, ResumeBinding)>,
) {
    let mut initiator = match binding {
        Some(b) => InitiatorHandshake::with_resume_binding(
            client,
            server.public_key(),
            b"resume",
            b"carrier-resume-test",
            b,
        )
        .unwrap(),
        None => InitiatorHandshake::new(
            client,
            server.public_key(),
            b"resume",
            b"carrier-resume-test",
        )
        .unwrap(),
    };
    let first = initiator.first_message().unwrap();
    if binding.is_some() {
        let (response, responder, peer, claim) =
            ResponderHandshake::new(server, policy, b"carrier-resume-test")
                .unwrap()
                .receive_first_with_resume(&first, ctx(generation))
                .unwrap();
        let initiator = initiator.finish(&response, ctx(generation)).unwrap();
        (initiator, responder, Some((peer, claim)))
    } else {
        let (response, responder) = ResponderHandshake::new(server, policy, b"carrier-resume-test")
            .unwrap()
            .receive_first(&first, ctx(generation))
            .unwrap();
        let initiator = initiator.finish(&response, ctx(generation)).unwrap();
        (initiator, responder, None)
    }
}
#[test]
fn fresh_tcp_noise_transport_resumes_one_logical_session_without_duplicate_delivery() {
    let client = LocalIdentity::generate().unwrap();
    let server = LocalIdentity::generate().unwrap();
    let policy = TrustPolicy::new(vec![TrustRecord {
        version: 1,
        public_key: client.public_key().to_vec(),
        scope: b"resume".to_vec(),
        status: TrustStatus::Active,
    }]);
    let original = ResumeBinding {
        session_id: 44,
        delivery_epoch: 1,
        key_phase: 0,
        path_generation: 1,
        expires_at_ms: 100,
        token: [7; 32],
    };
    let mut guard = ResumeGuard::new(client.public_key(), &original).unwrap();
    let (mut udp_client, mut udp_server, _) = handshake(&client, &server, policy.clone(), None, 1);
    let limits = RuntimeLimits {
        max_queue_records: 8,
        max_queue_bytes: 64,
        max_total_bytes: 64,
        max_record_bytes: 16,
        idle_timeout_ms: 100,
        close_timeout_ms: 10,
        max_streams: 1,
    };
    let mut runtime = SessionRuntime::new(SessionId(44), limits, 0).unwrap();
    runtime.open_stream(StreamId(1), 0).unwrap();
    let udp = udp_client.seal_unreliable(b"abc").unwrap();
    assert_eq!(udp_server.open_unreliable(&udp).unwrap(), b"abc");
    runtime
        .receive(
            InboundRecord {
                stream: StreamId(1),
                offset: 0,
                data: b"abc".to_vec(),
            },
            1,
        )
        .unwrap();
    let next = ResumeBinding {
        path_generation: 2,
        ..original
    };
    let (mut tcp_client, mut tcp_server, claim) =
        handshake(&client, &server, policy, Some(&next), 2);
    let (peer, claim) = claim.unwrap();
    guard.attach(&peer, &claim, 2).unwrap();
    let resend = tcp_client.seal_unreliable(b"abc").unwrap();
    assert_eq!(tcp_server.open_unreliable(&resend).unwrap(), b"abc");
    runtime
        .receive(
            InboundRecord {
                stream: StreamId(1),
                offset: 0,
                data: b"abc".to_vec(),
            },
            3,
        )
        .unwrap();
    runtime.delivery_ack(StreamId(1), 0, 3, 4).unwrap();
    assert_eq!(runtime.pop_receive(5).unwrap().unwrap().data, b"abc");
    assert!(runtime.pop_receive(6).unwrap().is_none());
    assert_eq!(runtime.confirmed_watermark(StreamId(1)), 3);
}

#[test]
fn bounded_udp_blackhole_tcp_resume_preserves_order_and_exactly_once_bytes() {
    use neko_carrier::{DataId, FailoverController};
    let client = LocalIdentity::generate().unwrap();
    let server = LocalIdentity::generate().unwrap();
    let policy = TrustPolicy::new(vec![TrustRecord {
        version: 1,
        public_key: client.public_key().to_vec(),
        scope: b"resume".to_vec(),
        status: TrustStatus::Active,
    }]);
    let original = ResumeBinding {
        session_id: 9001,
        delivery_epoch: 1,
        key_phase: 0,
        path_generation: 1,
        expires_at_ms: 10_000,
        token: [3; 32],
    };
    let mut guard = ResumeGuard::new(client.public_key(), &original).unwrap();
    let (mut udp_tx, mut udp_rx, _) = handshake(&client, &server, policy.clone(), None, 1);
    let limits = RuntimeLimits {
        max_queue_records: 8,
        max_queue_bytes: 128,
        max_total_bytes: 128,
        max_record_bytes: 32,
        idle_timeout_ms: 1_000,
        close_timeout_ms: 100,
        max_streams: 1,
    };
    let mut sender_runtime = SessionRuntime::new(SessionId(9001), limits, 0).unwrap();
    let mut receiver_runtime = SessionRuntime::new(SessionId(9001), limits, 0).unwrap();
    sender_runtime.open_stream(StreamId(1), 0).unwrap();
    receiver_runtime.open_stream(StreamId(1), 0).unwrap();
    let records = [
        b"alpha".as_slice(),
        b"-bounded".as_slice(),
        b"-exactly-once".as_slice(),
    ];
    let mut failover = FailoverController::new(2, 8, 128).unwrap();
    let mut ids = Vec::new();
    for data in records {
        let offset = sender_runtime.queue_send(StreamId(1), data, 1).unwrap();
        let record = sender_runtime.pop_send(2).unwrap().unwrap();
        let id = DataId(offset);
        ids.push((id, record));
        failover.track_uncertain(id, data).unwrap();
        // The first UDP record arrives; the remaining records are blackholed.
        if offset == 0 {
            let wire = udp_tx.seal_unreliable(data).unwrap();
            let plain = udp_rx.open_unreliable(&wire).unwrap();
            receiver_runtime
                .receive(
                    InboundRecord {
                        stream: StreamId(1),
                        offset,
                        data: plain,
                    },
                    3,
                )
                .unwrap();
        }
    }
    assert!(!failover.udp_pto_at(10));
    assert!(failover.udp_pto_at(20));
    let next = ResumeBinding {
        path_generation: 2,
        ..original
    };
    let (mut tcp_tx, mut tcp_rx, claim) = handshake(&client, &server, policy, Some(&next), 2);
    let (peer, claim) = claim.unwrap();
    guard.attach(&peer, &claim, 2).unwrap();
    // A fresh Noise transport resends every uncertain logical record. The
    // receiver's Session is the sole ordering/deduplication authority.
    for (id, record) in ids.iter().skip(1) {
        let wire = tcp_tx.seal_unreliable(&record.data).unwrap();
        let plain = tcp_rx.open_unreliable(&wire).unwrap();
        receiver_runtime
            .receive(
                InboundRecord {
                    stream: StreamId(1),
                    offset: record.offset,
                    data: plain.clone(),
                },
                4,
            )
            .unwrap();
        receiver_runtime
            .delivery_ack(StreamId(1), record.offset, record.data.len(), 5)
            .unwrap();
        failover.confirm(*id).unwrap();
    }
    // Resending an already delivered first record is harmless and emits no
    // second application byte range.
    let duplicate = tcp_tx.seal_unreliable(b"alpha").unwrap();
    let duplicate = tcp_rx.open_unreliable(&duplicate).unwrap();
    receiver_runtime
        .receive(
            InboundRecord {
                stream: StreamId(1),
                offset: 0,
                data: duplicate,
            },
            6,
        )
        .unwrap();
    failover.confirm(ids[0].0).unwrap();
    let mut application = Vec::new();
    while let Some(record) = receiver_runtime.pop_receive(8).unwrap() {
        application.extend_from_slice(&record.data);
    }
    assert_eq!(application, b"alpha-bounded-exactly-once");
    assert_eq!(
        receiver_runtime.confirmed_watermark(StreamId(1)),
        application.len() as u64
    );
    assert!(failover.tcp_resend().unwrap().is_empty());
    receiver_runtime.close_graceful(9).unwrap();
    receiver_runtime.close_remote(10).unwrap();
    assert_eq!(receiver_runtime.queued_records(), 0);
    assert_eq!(receiver_runtime.queued_bytes(), 0);
}
