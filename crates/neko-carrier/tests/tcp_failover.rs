use neko_carrier::{DataId, FailoverController, TcpCarrier, TcpLimits, TcpLoopbackPair};
use neko_crypto::{
    InitiatorHandshake, LocalIdentity, RecordContext, ResponderHandshake, TrustPolicy, TrustRecord,
    TrustStatus,
};

fn context() -> RecordContext {
    RecordContext {
        delivery_epoch: 2,
        key_phase: 0,
        path_generation: 2,
        stream_id: 9,
        direction: 0,
    }
}
fn encode_data(id: DataId, data: &[u8]) -> Vec<u8> {
    let mut out = Vec::with_capacity(8 + data.len());
    out.extend_from_slice(&id.0.to_be_bytes());
    out.extend_from_slice(data);
    out
}
fn decode_data(bytes: &[u8]) -> (DataId, &[u8]) {
    assert!(bytes.len() >= 8);
    (
        DataId(u64::from_be_bytes(bytes[..8].try_into().unwrap())),
        &bytes[8..],
    )
}
fn crypto_pair() -> (neko_crypto::SecureSession, neko_crypto::SecureSession) {
    let ci = LocalIdentity::generate().unwrap();
    let si = LocalIdentity::generate().unwrap();
    let policy = TrustPolicy::new(vec![TrustRecord {
        version: 1,
        public_key: ci.public_key().to_vec(),
        scope: b"failover".to_vec(),
        status: TrustStatus::Active,
    }]);
    let mut i =
        InitiatorHandshake::new(&ci, si.public_key(), b"failover", b"tcp-fallback").unwrap();
    let first = i.first_message().unwrap();
    let r = ResponderHandshake::new(&si, policy, b"tcp-fallback").unwrap();
    let (response, rs) = r.receive_first(&first, context()).unwrap();
    let is = i.finish(&response, context()).unwrap();
    (is, rs)
}

#[test]
fn udp_blackhole_recovers_over_tcp_without_loss_and_deduplicates() {
    let (tcp_client, tcp_server) = TcpLoopbackPair::new(TcpLimits {
        max_frame_bytes: 4600,
    })
    .unwrap();
    let (mut sender_crypto, mut receiver_crypto) = crypto_pair();
    let mut sender = FailoverController::new(2, 8, 4096).unwrap();
    let mut receiver = FailoverController::new(2, 8, 4096).unwrap();
    let id = DataId(41);
    let payload = b"uncertain bytes survive carrier change";
    sender.track_uncertain(id, payload).unwrap();
    // Model a UDP blackhole: two PTOs produce no ACK/delivery evidence.
    assert!(!sender.udp_pto());
    assert!(sender.udp_pto());
    for (data_id, data) in sender.tcp_resend().unwrap() {
        tcp_client
            .send_frame(&sender_crypto.seal(&encode_data(data_id, &data)).unwrap())
            .unwrap();
    }
    let plain = receiver_crypto
        .open(&tcp_server.recv_frame().unwrap())
        .unwrap();
    let (received_id, received) = decode_data(&plain);
    assert!(receiver.receive(received_id, received).unwrap());
    sender.confirm(received_id).unwrap();
    // A safe resend uses a fresh AEAD sequence but the same logical DataId.
    tcp_client
        .send_frame(&sender_crypto.seal(&encode_data(id, payload)).unwrap())
        .unwrap();
    let duplicate = receiver_crypto
        .open(&tcp_server.recv_frame().unwrap())
        .unwrap();
    let (duplicate_id, duplicate_data) = decode_data(&duplicate);
    assert!(!receiver.receive(duplicate_id, duplicate_data).unwrap());
    assert_eq!(receiver.metrics.delivered_bytes, payload.len() as u64);
    assert_eq!(receiver.metrics.duplicate_bytes, payload.len() as u64);
    assert_eq!(sender.metrics.switches, 1);
}
