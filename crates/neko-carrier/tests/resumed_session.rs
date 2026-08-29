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
