//! M1-S2 candidate seam: one bounded wire record per carrier unit.
use neko_carrier::{Carrier, MemoryLimits, MemoryPair, UdpCarrier, UdpLimits, UdpLoopbackPair};
use neko_wire::{DecodeError, HEADER_LEN, MAX_PAYLOAD_LEN, Record, RecordType, decode, encode};

fn rec(kind: RecordType, payload: &[u8]) -> Record {
    Record {
        record_type: kind,
        flags: 0,
        payload: payload.to_vec(),
    }
}
fn memory(max: usize) -> (impl Carrier, impl Carrier) {
    MemoryPair::new(MemoryLimits {
        max_message_bytes: max,
        max_queue_bytes: max * 4 + 1,
    })
    .unwrap()
}

#[test]
fn memory_roundtrip_fifo_and_empty() {
    let (a, b) = memory(HEADER_LEN + 8);
    let rs = [rec(RecordType::Data, b"one"), rec(RecordType::Ack, b"two")];
    for r in &rs {
        a.send(&encode(r).unwrap()).unwrap();
    }
    assert_eq!(decode(&b.recv().unwrap().unwrap()).unwrap(), rs[0]);
    assert_eq!(decode(&b.recv().unwrap().unwrap()).unwrap(), rs[1]);
    assert_eq!(b.recv().unwrap(), None);
    let reply = rec(RecordType::PathChallenge, b"back");
    a.send(&encode(&reply).unwrap()).unwrap();
    assert_eq!(decode(&b.recv().unwrap().unwrap()).unwrap(), reply);
}

#[test]
fn udp_roundtrip_fifo_and_would_block() {
    let max = HEADER_LEN + 8;
    let (a, b) = UdpLoopbackPair::new(UdpLimits {
        max_datagram_bytes: max,
    })
    .unwrap();
    assert!(matches!(
        b.recv_datagram(),
        Err(neko_carrier::UdpError::WouldBlock)
    ));
    let rs = [rec(RecordType::Data, b"udp"), rec(RecordType::Ack, b"ok")];
    for r in &rs {
        a.send_datagram(&encode(r).unwrap()).unwrap();
    }
    assert_eq!(decode(&b.recv_datagram().unwrap().unwrap()).unwrap(), rs[0]);
    assert_eq!(decode(&b.recv_datagram().unwrap().unwrap()).unwrap(), rs[1]);
    let reply = rec(RecordType::PathChallenge, b"reply");
    b.send_datagram(&encode(&reply).unwrap()).unwrap();
    assert_eq!(decode(&a.recv_datagram().unwrap().unwrap()).unwrap(), reply);
}

#[test]
fn empty_max_and_max_plus_one_are_bounded() {
    let max = encode(&rec(RecordType::Data, &vec![0; MAX_PAYLOAD_LEN])).unwrap();
    assert_eq!(max.len(), HEADER_LEN + MAX_PAYLOAD_LEN);
    assert!(encode(&rec(RecordType::Data, &vec![0; MAX_PAYLOAD_LEN + 1])).is_err());
    let (a, b) = memory(max.len());
    a.send(&encode(&rec(RecordType::Data, &[])).unwrap())
        .unwrap();
    a.send(&max).unwrap();
    assert_eq!(
        decode(&b.recv().unwrap().unwrap()).unwrap().payload.len(),
        0
    );
    assert_eq!(
        decode(&b.recv().unwrap().unwrap()).unwrap().payload.len(),
        MAX_PAYLOAD_LEN
    );
    assert!(a.send(&vec![0; max.len() + 1]).is_err());
    let (u, v) = UdpLoopbackPair::new(UdpLimits {
        max_datagram_bytes: max.len(),
    })
    .unwrap();
    u.send_datagram(&max).unwrap();
    assert_eq!(
        decode(&v.recv_datagram().unwrap().unwrap())
            .unwrap()
            .payload
            .len(),
        MAX_PAYLOAD_LEN
    );
    assert!(matches!(
        u.send_datagram(&vec![0; max.len() + 1]),
        Err(neko_carrier::UdpError::MessageTooLarge)
    ));
}

#[test]
fn malformed_wire_is_rejected_and_evidence_is_untouched() {
    let valid = encode(&rec(RecordType::Data, b"safe")).unwrap();
    let malformed = [
        vec![],
        b"badbadbad".to_vec(),
        {
            let mut v = valid.clone();
            v[2] = 1;
            v
        },
        {
            let mut v = valid.clone();
            v[3] = 99;
            v
        },
        {
            let mut v = valid.clone();
            v[4] = 1;
            v
        },
        {
            let mut v = valid.clone();
            v[5..9].copy_from_slice(&(MAX_PAYLOAD_LEN as u32 + 1).to_be_bytes());
            v
        },
        {
            let mut v = valid.clone();
            v.pop();
            v
        },
        {
            let mut v = valid.clone();
            v.push(0);
            v
        },
    ];
    for bytes in malformed {
        assert!(decode(&bytes).is_err());
    }
    // Decode rejection is local and does not construct SessionDelivery,
    // PathValidated, ACK, or auth evidence; the carrier remains usable.
    let (a, b) = memory(valid.len());
    assert_eq!(b.recv().unwrap(), None);
    a.send(&valid).unwrap();
    assert_eq!(
        decode(&b.recv().unwrap().unwrap()).unwrap(),
        rec(RecordType::Data, b"safe")
    );
}

#[test]
fn trailing_datagram_is_one_rejected_record() {
    let mut bytes = encode(&rec(RecordType::Ack, b"ack")).unwrap();
    bytes.push(0);
    let (a, b) = UdpLoopbackPair::new(UdpLimits {
        max_datagram_bytes: bytes.len(),
    })
    .unwrap();
    a.send_datagram(&bytes).unwrap();
    assert!(matches!(
        decode(&b.recv_datagram().unwrap().unwrap()),
        Err(DecodeError::TrailingBytes(1))
    ));
    assert!(matches!(
        b.recv_datagram(),
        Err(neko_carrier::UdpError::WouldBlock)
    ));
}
