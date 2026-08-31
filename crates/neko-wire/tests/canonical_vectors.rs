use neko_wire::{
    Frame, NegotiationRole, Record, RecordType, VersionNegotiator, decode, decode_frames,
    decode_varint, encode, encode_frames, encode_varint,
};
use serde::Deserialize;
use serde_json::Value;
use std::{fs, path::PathBuf};

#[derive(Deserialize)]
struct Corpus {
    vectors: Vec<Vector>,
}
#[derive(Deserialize)]
struct Vector {
    id: String,
    input: Value,
    bytes_hex: String,
    expected: Expected,
    oracle: Oracle,
    classification: Vec<String>,
}
#[derive(Deserialize)]
struct Expected {
    #[allow(dead_code)]
    ok: bool,
    value: Option<Value>,
    error: Option<String>,
}
#[derive(Deserialize)]
struct Oracle {
    encode_equals_bytes: bool,
    decode_bytes_equals_expected: bool,
    roundtrip_equals_bytes: bool,
}
fn corpus() -> Corpus {
    let root = PathBuf::from(env!("CARGO_MANIFEST_DIR"));
    serde_json::from_str(
        &fs::read_to_string(root.join("../../fixtures/canonical-vectors.v1.json")).unwrap(),
    )
    .unwrap()
}
fn bytes(s: &str) -> Vec<u8> {
    (0..s.len())
        .step_by(2)
        .map(|i| u8::from_str_radix(&s[i..i + 2], 16).unwrap())
        .collect()
}
fn val(v: &Vector) -> Value {
    v.expected.value.clone().unwrap()
}

#[test]
fn canonical_vectors_execute_real_wire_and_negotiation_codecs() {
    for v in corpus().vectors {
        match v.id.as_str() {
            "negotiation.hello.v0-v2" => {
                let versions: Vec<u16> = v.input["versions"]
                    .as_array()
                    .unwrap()
                    .iter()
                    .map(|x| x.as_u64().unwrap() as u16)
                    .collect();
                let n = VersionNegotiator::new(NegotiationRole::Client, &versions).unwrap();
                let got = n.client_hello().unwrap();
                assert_eq!(got, bytes(&v.bytes_hex));
                assert!(v.oracle.encode_equals_bytes);
                assert_eq!(got, bytes(&v.bytes_hex));
                assert!(v.oracle.decode_bytes_equals_expected);
            }
            "negotiation.no-overlap" | "negotiation.duplicate" => {
                let versions: Vec<u16> = v.input["versions"]
                    .as_array()
                    .unwrap()
                    .iter()
                    .map(|x| x.as_u64().unwrap() as u16)
                    .collect();
                let mut n = VersionNegotiator::new(NegotiationRole::Server, &[0]).unwrap();
                let hello = {
                    let mut b = vec![b'N', b'1', 1, versions.len() as u8];
                    for x in versions {
                        b.extend_from_slice(&x.to_be_bytes());
                    }
                    b
                };
                let e = format!("{:?}", n.server_accept_hello(&hello).unwrap_err());
                assert!(e.starts_with(v.expected.error.as_ref().unwrap()));
                assert!(
                    !v.oracle.encode_equals_bytes
                        && v.oracle.decode_bytes_equals_expected
                        && !v.oracle.roundtrip_equals_bytes
                );
                assert!(v.classification.contains(&"expected_failure".into()));
            }
            "wire.record-data" => {
                let r = Record {
                    record_type: RecordType::Data,
                    flags: 0,
                    payload: bytes(v.input["payload_hex"].as_str().unwrap()),
                };
                let b = encode(&r).unwrap();
                assert_eq!(b, bytes(&v.bytes_hex));
                let d = decode(&b).unwrap();
                assert_eq!(encode(&d).unwrap(), b);
                assert_eq!(d, r);
                assert_eq!(val(&v)["payload_hex"], v.input["payload_hex"]);
                assert!(v.oracle.roundtrip_equals_bytes);
            }
            "wire.bad-version" | "error.trailing" => {
                let e = format!("{:?}", decode(&bytes(&v.bytes_hex)).unwrap_err());
                assert!(e.starts_with(v.expected.error.as_ref().unwrap()));
                assert!(
                    !v.oracle.encode_equals_bytes
                        && v.oracle.decode_bytes_equals_expected
                        && !v.oracle.roundtrip_equals_bytes
                );
            }
            "frame.datagram-max" => {
                let f = vec![Frame::Datagram(vec![0])];
                let b = encode_frames(&f).unwrap();
                assert_eq!(b, bytes(&v.bytes_hex));
                assert_eq!(decode_frames(&b).unwrap(), f);
            }
            "frame.oversized" => {
                let e = encode_frames(&[Frame::Datagram(vec![0; 1025])]).unwrap_err();
                assert!(format!("{e:?}").starts_with(v.expected.error.as_ref().unwrap()));
                assert!(
                    !v.oracle.encode_equals_bytes
                        && v.oracle.decode_bytes_equals_expected
                        && !v.oracle.roundtrip_equals_bytes
                );
            }
            "varint.noncanonical" => {
                assert!(
                    format!("{:?}", decode_varint(&bytes(&v.bytes_hex)).unwrap_err())
                        .starts_with(v.expected.error.as_ref().unwrap())
                );
                assert!(
                    !v.oracle.encode_equals_bytes
                        && v.oracle.decode_bytes_equals_expected
                        && !v.oracle.roundtrip_equals_bytes
                );
            }
            _ => {
                assert!(
                    v.classification
                        .iter()
                        .any(|c| c == "conceptual" || c == "state_only")
                );
                assert!(
                    !v.oracle.encode_equals_bytes
                        && !v.oracle.decode_bytes_equals_expected
                        && !v.oracle.roundtrip_equals_bytes
                );
            }
        }
    }
}

#[test]
fn canonical_varint_roundtrip_is_byte_exact() {
    for n in [0, 1, 127, 128, u64::MAX] {
        let mut b = vec![];
        encode_varint(n, &mut b);
        assert_eq!(decode_varint(&b).unwrap(), (n, b.len()));
    }
}
