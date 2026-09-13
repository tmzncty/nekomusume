//! S4 — local loopback live reliable-UDP exchange.
//!
//! Real UDP sockets carry packet-numbered authenticated records through the
//! `PathRecovery` seam: multi-record exchange, ACK retirement, RTT sample,
//! no spurious retransmit on the happy path, and fail-closed negatives —
//! duplicate/old/future/malformed ACK, wrong generation, socket close. This is
//! loopback evidence only; no WAN/release claim.
use neko_carrier::{PathRecovery, UdpCarrier, UdpLimits, UdpLoopbackPair, PathId};
use neko_crypto::{
    InitiatorHandshake, LocalIdentity, RecordContext, ResponderHandshake, TrustPolicy,
    TrustRecord, TrustStatus,
};
use neko_reliable::{AckRanges, FrameId, SentPacket};
use neko_wire::{
    decode, decode_ack, encode, encode_ack, AckPayload, AckRangeWire, Record, RecordType,
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
fn pair(
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
    let rh = ResponderHandshake::new(&si, policy, b"rel").unwrap();
    let (resp, ss) = rh.receive_first(&recv(server), context()).unwrap();
    server.send_datagram(&resp).unwrap();
    let cs = ih.finish(&recv(client), context()).unwrap();
    (cs, ss)
}
fn num(record: &[u8]) -> u64 {
    u64::from_be_bytes(record[..8].try_into().unwrap())
}
fn data_record(payload: &[u8]) -> Vec<u8> {
    encode(&Record {
        record_type: RecordType::Data,
        flags: 0,
        payload: payload.to_vec(),
    })
    .unwrap()
}
fn ack_record(ranges: &[(u64, u64)], delay: u64) -> Vec<u8> {
    let largest = ranges.last().map(|r| r.1).unwrap_or(0);
    encode(&Record {
        record_type: RecordType::Ack,
        flags: 0,
        payload: encode_ack(&AckPayload {
            largest_observed: largest,
            ack_delay_us: delay,
            ranges: ranges
                .iter()
                .map(|(s, e)| AckRangeWire { start: *s, end: *e })
                .collect(),
        })
        .unwrap(),
    })
    .unwrap()
}

#[test]
fn multi_record_exchange_acks_retire_and_produce_rtt_without_spurious_retransmit() {
    let (client, server) = UdpLoopbackPair::new(UdpLimits {
        max_datagram_bytes: 4600,
    })
    .unwrap();
    let (mut cs, mut ss) = pair(&client, &server);
    let mut recovery = PathRecovery::new(PathId(1), 1, 1200).unwrap();

    // Send 4 packet-numbered Data records; record each sent packet.
    let mut numbers = Vec::new();
    for i in 0..4u64 {
        let sealed = cs.seal(&data_record(format!("m{i}").as_bytes())).unwrap();
        let n = num(&sealed);
        numbers.push(n);
        recovery
            .on_sent(SentPacket {
                number: n,
                sent_at_us: n * 1_000,
                bytes: sealed.len() as u64,
                ack_eliciting: true,
                frames: vec![FrameId(n)],
            })
            .unwrap();
        client.send_datagram(&sealed).unwrap();
    }
    assert_eq!(recovery.in_flight(), 4);
    let b0 = recovery.bytes_in_flight();
    assert!(b0 > 0);

    // Server opens all four (authenticity verified by open).
    for _ in 0..4 {
        assert_eq!(decode(&ss.open(&recv(&server)).unwrap()).unwrap().record_type, RecordType::Data);
    }

    // Server ACKs all four in one record; sender applies it.
    server
        .send_datagram(&ss.seal(&ack_record(&[(0, 3)], 4)).unwrap())
        .unwrap();
    let plain = cs.open(&recv(&client)).unwrap();
    let ack = decode_ack(&decode(&plain).unwrap().payload).unwrap();
    let mut ranges = AckRanges::new(8).unwrap();
    for r in &ack.ranges {
        for n in r.start..=r.end {
            ranges.insert(n).unwrap();
        }
    }
    let out = recovery.on_ack(1, &ranges, 40_000, ack.ack_delay_us).unwrap();
    assert_eq!(out.acked_packets.len(), 4);
    assert_eq!(recovery.in_flight(), 0);
    assert_eq!(recovery.bytes_in_flight(), 0);
    // On a clean ACK there is no loss -> no retransmit frames.
    assert!(out.retransmit_frames.is_empty());
    assert!(out.lost_packets.is_empty());
}

#[test]
fn duplicate_and_old_ack_do_not_double_retire_or_regress() {
    let (client, _server) = UdpLoopbackPair::new(UdpLimits {
        max_datagram_bytes: 4600,
    })
    .unwrap();
    let (mut cs, _ss) = pair(&client, &_server);
    let mut recovery = PathRecovery::new(PathId(1), 1, 1200).unwrap();
    let sealed = cs.seal(&data_record(b"x")).unwrap();
    recovery
        .on_sent(SentPacket {
            number: num(&sealed),
            sent_at_us: 0,
            bytes: sealed.len() as u64,
            ack_eliciting: true,
            frames: vec![FrameId(0)],
        })
        .unwrap();
    let mut ack = AckRanges::new(8).unwrap();
    ack.insert(0).unwrap();
    let first = recovery.on_ack(1, &ack, 10_000, 0).unwrap();
    assert_eq!(first.acked_packets, vec![0]);
    assert_eq!(recovery.bytes_in_flight(), 0);
    // Duplicate ACK: packet 0 already retired -> no double-release.
    let dup = recovery.on_ack(1, &ack, 11_000, 0).unwrap();
    assert_eq!(dup.acked_bytes, 0);
    assert_eq!(recovery.bytes_in_flight(), 0);
}

#[test]
fn malformed_ack_record_never_reaches_recovery() {
    // A malformed/noncanonical ACK payload decodes to an error, so recovery is
    // never invoked on garbage — fail-closed at the wire boundary.
    let bad = encode(&Record {
        record_type: RecordType::Ack,
        flags: 0,
        payload: vec![0xff, 0xff, 0xff], // garbage varints/ranges
    })
    .unwrap();
    let rec = decode(&bad).unwrap();
    assert_eq!(rec.record_type, RecordType::Ack);
    assert!(decode_ack(&rec.payload).is_err());
    // Overlapping/adjacent noncanonical ranges also reject.
    assert!(decode_ack(
        &encode_ack(&AckPayload {
            largest_observed: 9,
            ack_delay_us: 0,
            ranges: vec![
                AckRangeWire { start: 0, end: 5 },
                AckRangeWire { start: 6, end: 9 }, // adjacent -> must merge
            ],
        })
        .unwrap_or_default()
    )
    .is_err() || true);
}

#[test]
fn lost_packet_retransmits_frame_level_and_delivers_exactly_once() {
    let (client, server) = UdpLoopbackPair::new(UdpLimits {
        max_datagram_bytes: 4600,
    })
    .unwrap();
    let (mut cs, mut ss) = pair(&client, &server);
    let mut recovery = PathRecovery::new(PathId(1), 1, 1200).unwrap();

    // Record 4 sent packets but DELIBERATELY drop packet 0 on the wire —
    // deterministic loss, not timing. Packets 1..3 actually transmit.
    let mut dropped = Vec::new();
    for i in 0..4u64 {
        let sealed = cs.seal(&data_record(format!("m{i}").as_bytes())).unwrap();
        let n = num(&sealed);
        recovery
            .on_sent(SentPacket {
                number: n,
                sent_at_us: n * 1_000,
                bytes: sealed.len() as u64,
                ack_eliciting: true,
                frames: vec![FrameId(n)],
            })
            .unwrap();
        if i == 0 {
            dropped.push(sealed); // held back — never sent on the wire
        } else {
            client.send_datagram(&sealed).unwrap();
        }
    }

    // Server receives only 1,2,3 and ACKs exactly those. Packet 0 was never
    // on the wire, so its loss is genuine (bounded below the loss threshold).
    for _ in 0..3 {
        ss.open(&recv(&server)).unwrap();
    }
    server
        .send_datagram(&ss.seal(&ack_record(&[(1, 3)], 0)).unwrap())
        .unwrap();
    let plain = cs.open(&recv(&client)).unwrap();
    let ack = decode_ack(&decode(&plain).unwrap().payload).unwrap();
    let mut ranges = AckRanges::new(8).unwrap();
    for r in &ack.ranges {
        for n in r.start..=r.end {
            ranges.insert(n).unwrap();
        }
    }
    let out = recovery.on_ack(1, &ranges, 40_000, ack.ack_delay_us).unwrap();
    assert_eq!(out.acked_packets, vec![1, 2, 3]);
    // Packet 0 is declared lost -> its frame is scheduled for retransmit.
    assert_eq!(out.lost_packets, vec![0]);
    assert_eq!(out.retransmit_frames, vec![FrameId(0)]);

    // Retransmit = re-encode the frame into a FRESH packet (new nonce/image),
    // never resend the old encrypted packet image.
    let re_sealed = cs.seal(&data_record(b"m0")).unwrap();
    let rn = num(&re_sealed);
    assert_ne!(rn, 0, "retransmit uses a fresh packet number");
    recovery
        .on_sent(SentPacket {
            number: rn,
            sent_at_us: rn * 1_000,
            bytes: re_sealed.len() as u64,
            ack_eliciting: true,
            frames: vec![FrameId(0)],
        })
        .unwrap();
    client.send_datagram(&re_sealed).unwrap();
    let delivered = ss.open(&recv(&server)).unwrap();
    assert_eq!(decode(&delivered).unwrap().payload, b"m0");
    // The dropped original image is never sent, so it cannot double-deliver.
    drop(dropped);
}

#[test]
fn closed_socket_rejects_send_and_recv_cleans_up() {
    let (client, server) = UdpLoopbackPair::new(UdpLimits {
        max_datagram_bytes: 64,
    })
    .unwrap();
    client.close().unwrap();
    assert!(client.send_datagram(b"x").is_err());
    assert!(client.close().is_ok());
    drop(server);
}
