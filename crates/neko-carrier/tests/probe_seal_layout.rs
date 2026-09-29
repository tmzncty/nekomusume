//! Cross-crate pin for the D067 opt-in probe seal. neko-crypto recognises a
//! PMTU probe by byte layout (it cannot depend on neko-session), so this test
//! proves the layout agrees with the real `ProcessMessage` encoder. Every
//! other process kind must be refused by the probe path.

use neko_crypto::{
    InitiatorHandshake, LocalIdentity, ProbeFamily, RECORD_SEAL_OVERHEAD, RecordContext,
    ResponderHandshake, SecureSession, SessionRejected, TrustPolicy, TrustRecord, TrustStatus,
};
use neko_session::{
    OutboundRecord, PROCESS_PMTU_PROBE_HEADER_LEN, ProcessMessage, SessionId, StreamId,
};

fn pair() -> (SecureSession, SecureSession) {
    let ci = LocalIdentity::generate().unwrap();
    let si = LocalIdentity::generate().unwrap();
    let policy = TrustPolicy::new(vec![TrustRecord {
        version: 1,
        public_key: ci.public_key().to_vec(),
        scope: b"echo".to_vec(),
        status: TrustStatus::Active,
    }]);
    let context = RecordContext {
        delivery_epoch: 1,
        key_phase: 0,
        path_generation: 1,
        stream_id: 1,
        direction: 0,
    };
    let mut ih = InitiatorHandshake::new(&ci, si.public_key(), b"echo", b"probe").unwrap();
    let first = ih.first_message().unwrap();
    let (response, ss) = ResponderHandshake::new(&si, policy, b"probe")
        .unwrap()
        .receive_first(&first, context)
        .unwrap();
    (ih.finish(&response, context).unwrap(), ss)
}

fn probe_at_family_max(family: ProbeFamily) -> Vec<u8> {
    let padding = family.max_probe_plaintext() - PROCESS_PMTU_PROBE_HEADER_LEN;
    ProcessMessage::PmtuProbe {
        path_generation: 1,
        probe_id: 3,
        probe_size: 1500,
        padding_len: padding as u16,
    }
    .encode()
    .unwrap()
}

#[test]
fn real_encoded_probes_pass_the_probe_seal_at_each_family_bound() {
    for family in [ProbeFamily::V4, ProbeFamily::V6] {
        let (mut a, mut b) = pair();
        let plain = probe_at_family_max(family);
        assert_eq!(plain.len(), family.max_probe_plaintext());
        let record = a.seal_probe(family, &plain).unwrap();
        assert_eq!(record.len(), RECORD_SEAL_OVERHEAD + plain.len());
        let opened = b.open_datagram(&record, Some(family)).unwrap();
        assert!(matches!(
            ProcessMessage::decode(&opened),
            Ok(ProcessMessage::PmtuProbe {
                probe_size: 1500,
                ..
            })
        ));
        // Flag off, the same record is refused.
        let (mut a, mut b) = pair();
        let record = a.seal_probe(family, &plain).unwrap();
        assert_eq!(b.open_datagram(&record, None), Err(SessionRejected));
    }
}

#[test]
fn every_other_real_process_kind_is_refused_by_the_probe_path() {
    let data = ProcessMessage::Data {
        session: SessionId(7001),
        record: OutboundRecord {
            stream: StreamId(1),
            offset: 0,
            data: vec![0; 1270],
        },
    }
    .encode()
    .unwrap();
    assert_eq!(data.len(), 1300);
    let ack = ProcessMessage::PmtuProbeAck {
        path_generation: 1,
        probe_id: 3,
        probe_size: 1500,
    }
    .encode()
    .unwrap();
    for plain in [data, ack] {
        let (mut a, mut b) = pair();
        assert_eq!(a.seal_probe(ProbeFamily::V4, &plain), Err(SessionRejected));
        let record = a.seal(&plain).unwrap();
        assert_eq!(b.open_probe(ProbeFamily::V4, &record), Err(SessionRejected));
    }
}
