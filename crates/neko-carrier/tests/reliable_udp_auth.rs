//! S3 — packet number / packet-ACK bound to the authenticated UDP boundary.
//!
//! A UDP datagram is the `SecureSession` record: an 8-byte sequence (the AEAD
//! nonce, which doubles as the packet number) plus ciphertext whose plaintext
//! is one NK record. A `RecordType::Ack` NK record in that plaintext carries
//! the packet-recovery ACK. Because the packet number is the AEAD nonce and
//! the ACK payload lives inside the ciphertext, neither can be tampered with
//! or forged without failing `open` — and a failed `open` never advances
//! recovery state.
use neko_carrier::{PathId, PathRecovery, UdpCarrier, UdpLimits, UdpLoopbackPair};
use neko_crypto::{
    InitiatorHandshake, LocalIdentity, PreauthBudget, PreauthLimits, RecordContext,
    ResponderHandshake, TrustPolicy, TrustRecord, TrustStatus,
};
use neko_reliable::{AckRanges, FrameId, SentPacket};
use neko_wire::{
    AckPayload, AckRangeWire, Record, RecordType, decode, decode_ack, encode, encode_ack,
};
use std::{
    thread,
    time::{Duration, Instant},
};

fn recv(endpoint: &impl UdpCarrier) -> Vec<u8> {
    let deadline = Instant::now() + Duration::from_secs(1);
    loop {
        match endpoint.recv_datagram() {
            Ok(Some(v)) => return v,
            Err(neko_carrier::UdpError::WouldBlock) if Instant::now() < deadline => {
                thread::sleep(Duration::from_millis(1))
            }
            other => panic!("receive failed: {other:?}"),
        }
    }
}
fn context() -> RecordContext {
    RecordContext {
        delivery_epoch: 1,
        key_phase: 0,
        path_generation: 1,
        stream_id: 0,
        direction: 0,
    }
}
/// The packet number is the 8-byte record sequence (the AEAD nonce) prefixed
/// by `seal`/`open`. Tampering with it changes the nonce and fails decrypt.
fn packet_number(record: &[u8]) -> u64 {
    u64::from_be_bytes(record[..8].try_into().unwrap())
}

fn session_pair(
    client: &impl UdpCarrier,
    server: &impl UdpCarrier,
) -> (neko_crypto::SecureSession, neko_crypto::SecureSession) {
    let ci = LocalIdentity::generate().unwrap();
    let si = LocalIdentity::generate().unwrap();
    let policy = TrustPolicy::new(vec![TrustRecord {
        version: 1,
        public_key: ci.public_key().to_vec(),
        scope: b"echo".to_vec(),
        status: TrustStatus::Active,
    }]);
    let mut ih = InitiatorHandshake::new(&ci, si.public_key(), b"echo", b"rel").unwrap();
    client.send_datagram(&ih.first_message().unwrap()).unwrap();
    let mut budget = PreauthBudget::new(PreauthLimits::default()).unwrap();
    let first = recv(server);
    budget.charge_input(first.len()).unwrap();
    let rh = ResponderHandshake::new(&si, policy, b"rel").unwrap();
    let (response, ss) = rh.receive_first(&first, context()).unwrap();
    server.send_datagram(&response).unwrap();
    let cs = ih.finish(&recv(client), context()).unwrap();
    (cs, ss)
}

#[test]
fn authenticated_packet_ack_advances_recovery_but_tamper_cannot() {
    let (client, server) = UdpLoopbackPair::new(UdpLimits {
        max_datagram_bytes: 4600,
    })
    .unwrap();
    let (mut cs, mut ss) = session_pair(&client, &server);
    let mut recovery = PathRecovery::new(PathId(1), 1, 1200).unwrap();

    // Sender sends two Data records, recording each packet number sent.
    for _ in 0..2 {
        let app = encode(&Record {
            record_type: RecordType::Data,
            flags: 0,
            payload: b"ping".to_vec(),
        })
        .unwrap();
        let sealed = cs.seal(&app).unwrap();
        let number = packet_number(&sealed);
        recovery
            .on_sent(SentPacket {
                number,
                sent_at_us: number * 1000,
                bytes: sealed.len() as u64,
                ack_eliciting: true,
                frames: vec![FrameId(number)],
            })
            .unwrap();
        client.send_datagram(&sealed).unwrap();
    }
    assert_eq!(recovery.bytes_in_flight() > 0, true);

    // Receiver opens both; each yields an authenticated NK Data record.
    for _ in 0..2 {
        let plain = ss.open(&recv(&server)).unwrap();
        let rec = decode(&plain).unwrap();
        assert_eq!(rec.record_type, RecordType::Data);
    }

    // Receiver ACKs packet 0 via an authenticated NK Ack record.
    let ack_payload = encode_ack(&AckPayload {
        largest_observed: 0,
        ack_delay_us: 0,
        ranges: vec![AckRangeWire { start: 0, end: 0 }],
    })
    .unwrap();
    let ack_record = encode(&Record {
        record_type: RecordType::Ack,
        flags: 0,
        payload: ack_payload,
    })
    .unwrap();
    server
        .send_datagram(&ss.seal(&ack_record).unwrap())
        .unwrap();

    // Sender opens the ACK record and applies it to recovery.
    let plain = cs.open(&recv(&client)).unwrap();
    let rec = decode(&plain).unwrap();
    assert_eq!(rec.record_type, RecordType::Ack);
    let ack = decode_ack(&rec.payload).unwrap();
    let mut ranges = AckRanges::new(8).unwrap();
    for r in &ack.ranges {
        for n in r.start..=r.end {
            ranges.insert(n).unwrap();
        }
    }
    let out = recovery
        .on_ack(1, &ranges, 20_000, ack.ack_delay_us)
        .unwrap();
    assert_eq!(out.acked_packets, vec![0]);

    // Tampered ACK ciphertext cannot reach recovery at all.
    let mut bad = ss.seal(&ack_record).unwrap();
    *bad.last_mut().unwrap() ^= 1;
    server.send_datagram(&bad).unwrap();
    assert!(cs.open(&recv(&client)).is_err());
}

#[test]
fn wrong_generation_ack_and_future_ack_are_rejected() {
    let (client, server) = UdpLoopbackPair::new(UdpLimits {
        max_datagram_bytes: 4600,
    })
    .unwrap();
    let (mut cs, _ss) = session_pair(&client, &server);
    let mut recovery = PathRecovery::new(PathId(1), 1, 1200).unwrap();
    let app = encode(&Record {
        record_type: RecordType::Data,
        flags: 0,
        payload: b"x".to_vec(),
    })
    .unwrap();
    let sealed = cs.seal(&app).unwrap();
    recovery
        .on_sent(SentPacket {
            number: packet_number(&sealed),
            sent_at_us: 0,
            bytes: sealed.len() as u64,
            ack_eliciting: true,
            frames: vec![FrameId(0)],
        })
        .unwrap();
    let mut ack = AckRanges::new(8).unwrap();
    ack.insert(0).unwrap();
    // Wrong generation -> atomic reject before mutation.
    assert_eq!(
        recovery.on_ack(2, &ack, 10_000, 0).unwrap_err(),
        neko_carrier::PathRecoveryError::GenerationMismatch
    );
    // Future ACK beyond largest sent -> engine reject.
    let mut future = AckRanges::new(8).unwrap();
    future.insert(99).unwrap();
    assert!(recovery.on_ack(1, &future, 10_000, 0).is_err());
}
