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
    domain: String,
    operation: String,
    input: Value,
    bytes_hex: Option<String>,
    expected: Expected,
    oracle: Oracle,
    classification: Vec<String>,
}
#[derive(Deserialize)]
struct Expected {
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
fn hex(s: &str) -> Vec<u8> {
    (0..s.len())
        .step_by(2)
        .map(|i| u8::from_str_radix(&s[i..i + 2], 16).unwrap())
        .collect()
}
fn fixture_bytes(v: &Vector) -> Vec<u8> {
    hex(v
        .bytes_hex
        .as_deref()
        .expect("wire oracle requires bytes_hex"))
}
fn versions(value: &Value, key: &str) -> Vec<u16> {
    value[key]
        .as_array()
        .unwrap()
        .iter()
        .map(|x| x.as_u64().unwrap() as u16)
        .collect()
}
fn error_name<T: std::fmt::Debug>(error: T) -> String {
    format!("{error:?}")
        .split(['(', ' '])
        .next()
        .unwrap()
        .to_owned()
}
fn expected_error(v: &Vector) -> &str {
    v.expected.error.as_deref().unwrap()
}
fn record_type(name: &str) -> RecordType {
    match name {
        "Data" => RecordType::Data,
        "Ack" => RecordType::Ack,
        "PathChallenge" => RecordType::PathChallenge,
        _ => panic!("unknown record type"),
    }
}
fn frame_from(value: &Value) -> Frame {
    let payload = hex(value["payload_hex"].as_str().unwrap_or(""));
    match value["type"].as_str().unwrap() {
        "Data" => Frame::Data(payload),
        "Datagram" => Frame::Datagram(payload),
        "DeliveryAck" => Frame::DeliveryAck(payload),
        "Close" => Frame::Close(payload),
        "PathChallenge" => Frame::PathChallenge(payload.try_into().unwrap()),
        "PathResponse" => Frame::PathResponse(payload.try_into().unwrap()),
        "UnknownIgnorable" => Frame::UnknownIgnorable {
            frame_type: value["frame_type"].as_u64().unwrap() as u8,
            payload,
        },
        _ => panic!("unknown frame type"),
    }
}
fn input_frames(v: &Vector) -> Vec<Frame> {
    if let Some(rows) = v.input["frames"].as_array() {
        return rows.iter().map(frame_from).collect();
    }
    if let Some(repeat) = v.input.get("repeat") {
        return std::iter::repeat_with(|| frame_from(repeat))
            .take(repeat["count"].as_u64().unwrap() as usize)
            .collect();
    }
    vec![]
}
fn expect_object_keys(value: &Value, required: &[&str], optional: &[&str], context: &str) {
    let object = value
        .as_object()
        .unwrap_or_else(|| panic!("{context}: expected object"));
    let allowed: std::collections::BTreeSet<_> = required.iter().chain(optional).copied().collect();
    assert!(
        object.keys().all(|key| allowed.contains(key.as_str())),
        "{context}: unknown expected.value key"
    );
    for key in required {
        assert!(
            object.contains_key(*key),
            "{context}: missing required expected.value key {key}"
        );
    }
}

fn assert_negotiation_selected(id: &str, selected: u16, expected: &Value) {
    expect_object_keys(expected, &["selected"], &[], id);
    assert_eq!(
        selected as u64,
        expected["selected"].as_u64().unwrap(),
        "{id} selected version"
    );
}
fn assert_record_semantics(id: &str, decoded: &Record, value: &Value) {
    assert_eq!(
        decoded.record_type,
        record_type(value["type"].as_str().unwrap()),
        "{id} type"
    );
    assert_eq!(
        decoded.flags as u64,
        value["flags"].as_u64().unwrap(),
        "{id} flags"
    );
    assert_eq!(
        decoded.payload,
        hex(value["payload_hex"].as_str().unwrap()),
        "{id} payload"
    );
}
fn assert_frames_semantics(id: &str, decoded: &[Frame], expected: &Value) {
    assert_eq!(
        decoded.len() as u64,
        expected["frame_count"].as_u64().unwrap(),
        "{id} frame count"
    );
    let expected_frames = expected["frames"]
        .as_array()
        .unwrap()
        .iter()
        .map(frame_from)
        .collect::<Vec<_>>();
    assert_eq!(
        decoded, expected_frames,
        "{id} decoded frame identity/payload"
    );
    if let Some(payload_bytes) = expected.get("payload_bytes") {
        let actual: usize = decoded
            .iter()
            .map(|frame| match frame {
                Frame::Data(p) | Frame::Datagram(p) | Frame::DeliveryAck(p) | Frame::Close(p) => {
                    p.len()
                }
                Frame::PathChallenge(p) | Frame::PathResponse(p) => p.len(),
                Frame::UnknownIgnorable { payload, .. } => payload.len(),
            })
            .sum();
        assert_eq!(
            actual as u64,
            payload_bytes.as_u64().unwrap(),
            "{id} payload bytes"
        );
    }
}

fn assert_success_contract(v: &Vector) {
    if !v.expected.ok || v.classification.iter().any(|c| c == "state_only") {
        return;
    }
    let value = v
        .expected
        .value
        .as_ref()
        .expect("successful row needs value");
    match (v.domain.as_str(), v.operation.as_str()) {
        ("negotiation", "client_hello") => {
            expect_object_keys(value, &["versions", "selected"], &[], &v.id)
        }
        ("negotiation", "server_response") => expect_object_keys(value, &["selected"], &[], &v.id),
        ("wire", "record") => {
            expect_object_keys(value, &["type", "flags", "payload_hex"], &[], &v.id)
        }
        ("wire", "varint") => expect_object_keys(value, &["value"], &[], &v.id),
        ("frame" | "close", "frames") => {
            expect_object_keys(value, &["frame_count", "frames"], &["payload_bytes"], &v.id);
            let frames = value["frames"].as_array().expect("frames must be array");
            for (index, frame) in frames.iter().enumerate() {
                let ty = frame["type"].as_str().unwrap_or("<missing>");
                let context = format!("{} frames[{index}]", v.id);
                match ty {
                    "Close" => expect_object_keys(frame, &["type"], &["payload_hex"], &context),
                    "UnknownIgnorable" => expect_object_keys(
                        frame,
                        &["type", "frame_type", "payload_hex"],
                        &[],
                        &context,
                    ),
                    _ => expect_object_keys(frame, &["type", "payload_hex"], &[], &context),
                }
            }
        }
        other => panic!(
            "{} has no successful expected-value contract for {other:?}",
            v.id
        ),
    }
}

fn assert_oracles_match_classification(v: &Vector) {
    let state_only = v.classification.iter().any(|c| c == "state_only");
    if state_only {
        assert!(
            v.bytes_hex.is_none(),
            "{}: state-only bytes must be null",
            v.id
        );
        assert!(
            !v.oracle.encode_equals_bytes
                && !v.oracle.decode_bytes_equals_expected
                && !v.oracle.roundtrip_equals_bytes,
            "{}",
            v.id
        );
    } else {
        assert!(
            v.bytes_hex.is_some(),
            "{}: executable wire row needs bytes",
            v.id
        );
        assert!(
            v.oracle.encode_equals_bytes
                || v.oracle.decode_bytes_equals_expected
                || v.oracle.roundtrip_equals_bytes,
            "{}: wire row has no executable oracle",
            v.id
        );
    }
    if v.oracle.roundtrip_equals_bytes {
        assert!(
            v.expected.ok && v.oracle.encode_equals_bytes && v.oracle.decode_bytes_equals_expected,
            "{}",
            v.id
        );
    }
}

#[test]
fn every_claimed_oracle_executes_real_implementation_code() {
    for v in corpus().vectors {
        assert_oracles_match_classification(&v);
        assert_success_contract(&v);
        if v.classification.iter().any(|c| c == "state_only") {
            continue;
        }
        let bytes = fixture_bytes(&v);
        match (v.domain.as_str(), v.operation.as_str()) {
            ("negotiation", "client_hello") => {
                let offered = versions(&v.input, "versions");
                let emitted = VersionNegotiator::new(NegotiationRole::Client, &offered)
                    .unwrap()
                    .client_hello()
                    .unwrap();
                if v.oracle.encode_equals_bytes {
                    assert_eq!(emitted, bytes, "{} encode", v.id);
                }
                if v.oracle.decode_bytes_equals_expected {
                    let mut server =
                        VersionNegotiator::new(NegotiationRole::Server, &offered).unwrap();
                    let response = server.server_accept_hello(&bytes).unwrap();
                    let expected = v.expected.value.as_ref().unwrap();
                    assert_eq!(
                        versions(expected, "versions"),
                        offered,
                        "{} offered versions",
                        v.id
                    );
                    assert_eq!(
                        server.selected().unwrap() as u64,
                        expected["selected"].as_u64().unwrap()
                    );
                    let expected_hello = VersionNegotiator::new(
                        NegotiationRole::Client,
                        &versions(expected, "versions"),
                    )
                    .unwrap()
                    .client_hello()
                    .unwrap();
                    assert_eq!(expected_hello, bytes, "{} decoded hello semantics", v.id);
                    if v.oracle.roundtrip_equals_bytes {
                        assert_eq!(
                            server.server_accept_hello(&bytes).unwrap(),
                            response,
                            "{} protocol roundtrip",
                            v.id
                        );
                        assert_eq!(emitted, bytes, "{} byte roundtrip", v.id);
                    }
                }
            }
            ("negotiation", "server_accept_hello") => {
                assert!(!v.oracle.encode_equals_bytes && !v.oracle.roundtrip_equals_bytes);
                let supported = versions(&v.input, "server_versions");
                let mut server =
                    VersionNegotiator::new(NegotiationRole::Server, &supported).unwrap();
                let error = server.server_accept_hello(&bytes).unwrap_err();
                assert_eq!(error_name(error), expected_error(&v), "{} decode", v.id);
            }
            ("negotiation", "server_response") => {
                let client_versions = versions(&v.input, "client_versions");
                if v.oracle.encode_equals_bytes {
                    let server_versions = versions(&v.input, "server_versions");
                    let hello = VersionNegotiator::new(NegotiationRole::Client, &client_versions)
                        .unwrap()
                        .client_hello()
                        .unwrap();
                    let emitted = VersionNegotiator::new(NegotiationRole::Server, &server_versions)
                        .unwrap()
                        .server_accept_hello(&hello)
                        .unwrap();
                    assert_eq!(emitted, bytes, "{} encode", v.id);
                }
                let mut client =
                    VersionNegotiator::new(NegotiationRole::Client, &client_versions).unwrap();
                match client.client_accept_response(&bytes) {
                    Ok(selected) => {
                        assert!(v.expected.ok, "{} unexpectedly succeeded", v.id);
                        assert_negotiation_selected(
                            &v.id,
                            selected,
                            v.expected.value.as_ref().unwrap(),
                        );
                        if v.oracle.roundtrip_equals_bytes {
                            let mut server =
                                VersionNegotiator::new(NegotiationRole::Server, &[selected])
                                    .unwrap();
                            let hello =
                                VersionNegotiator::new(NegotiationRole::Client, &client_versions)
                                    .unwrap()
                                    .client_hello()
                                    .unwrap();
                            assert_eq!(
                                server.server_accept_hello(&hello).unwrap(),
                                bytes,
                                "{} roundtrip",
                                v.id
                            );
                        }
                    }
                    Err(error) => {
                        assert_eq!(error_name(error), expected_error(&v), "{} decode", v.id)
                    }
                }
            }
            ("wire", "record") => {
                if v.oracle.encode_equals_bytes {
                    let record = Record {
                        record_type: record_type(v.input["type"].as_str().unwrap()),
                        flags: v.input["flags"].as_u64().unwrap() as u8,
                        payload: hex(v.input["payload_hex"].as_str().unwrap()),
                    };
                    assert_eq!(encode(&record).unwrap(), bytes, "{} encode", v.id);
                }
                if v.oracle.decode_bytes_equals_expected {
                    match decode(&bytes) {
                        Ok(decoded) => {
                            assert!(v.expected.ok, "{} unexpectedly succeeded", v.id);
                            let value = v.expected.value.as_ref().unwrap();
                            assert_record_semantics(&v.id, &decoded, value);
                            if v.oracle.roundtrip_equals_bytes {
                                assert_eq!(encode(&decoded).unwrap(), bytes, "{} roundtrip", v.id);
                            }
                        }
                        Err(error) => {
                            assert_eq!(error_name(error), expected_error(&v), "{} decode", v.id)
                        }
                    }
                }
            }
            ("error", "decode") => assert_eq!(
                error_name(decode(&bytes).unwrap_err()),
                expected_error(&v),
                "{} decode",
                v.id
            ),
            ("frame" | "close", "frames") => {
                if v.oracle.encode_equals_bytes {
                    assert_eq!(
                        encode_frames(&input_frames(&v)).unwrap(),
                        bytes,
                        "{} encode",
                        v.id
                    );
                }
                if v.oracle.decode_bytes_equals_expected {
                    match decode_frames(&bytes) {
                        Ok(decoded) => {
                            assert!(v.expected.ok, "{} unexpectedly succeeded", v.id);
                            let expected = v.expected.value.as_ref().unwrap();
                            assert_frames_semantics(&v.id, &decoded, expected);
                            if let Some(payload_bytes) = expected.get("payload_bytes") {
                                let actual_payload_bytes: usize = decoded
                                    .iter()
                                    .map(|frame| match frame {
                                        Frame::Data(p)
                                        | Frame::Datagram(p)
                                        | Frame::DeliveryAck(p)
                                        | Frame::Close(p) => p.len(),
                                        Frame::PathChallenge(p) | Frame::PathResponse(p) => p.len(),
                                        Frame::UnknownIgnorable { payload, .. } => payload.len(),
                                    })
                                    .sum();
                                assert_eq!(
                                    actual_payload_bytes as u64,
                                    payload_bytes.as_u64().unwrap(),
                                    "{} payload bytes",
                                    v.id
                                );
                            }
                            if v.oracle.roundtrip_equals_bytes {
                                assert_eq!(
                                    encode_frames(&decoded).unwrap(),
                                    bytes,
                                    "{} roundtrip",
                                    v.id
                                );
                            }
                        }
                        Err(error) => {
                            assert_eq!(error_name(error), expected_error(&v), "{} decode", v.id)
                        }
                    }
                }
            }
            ("wire", "varint") => {
                if v.oracle.encode_equals_bytes {
                    let n = match v.input["value"].as_str() {
                        Some("u64::MAX") => u64::MAX,
                        _ => v.input["value"].as_u64().unwrap(),
                    };
                    let mut emitted = vec![];
                    encode_varint(n, &mut emitted);
                    assert_eq!(emitted, bytes, "{} encode", v.id);
                }
                if v.oracle.decode_bytes_equals_expected {
                    match decode_varint(&bytes) {
                        Ok((n, used)) => {
                            assert!(v.expected.ok && used == bytes.len(), "{} decode", v.id);
                            let expected =
                                match v.expected.value.as_ref().unwrap()["value"].as_str() {
                                    Some("u64::MAX") => u64::MAX,
                                    _ => v.expected.value.as_ref().unwrap()["value"]
                                        .as_u64()
                                        .unwrap(),
                                };
                            assert_eq!(n, expected, "{} value", v.id);
                            if v.oracle.roundtrip_equals_bytes {
                                let mut emitted = vec![];
                                encode_varint(n, &mut emitted);
                                assert_eq!(emitted, bytes, "{} roundtrip", v.id);
                            }
                        }
                        Err(error) => {
                            assert_eq!(error_name(error), expected_error(&v), "{} decode", v.id)
                        }
                    }
                }
            }
            other => panic!("{} has no executable adapter for {other:?}", v.id),
        }
    }
}

#[test]
fn successful_expected_contract_rejects_unknown_and_missing_keys() {
    let mut v = corpus()
        .vectors
        .into_iter()
        .find(|v| v.id == "wire.record-data")
        .unwrap();
    v.expected
        .value
        .as_mut()
        .unwrap()
        .as_object_mut()
        .unwrap()
        .insert("future_field".into(), Value::Null);
    assert!(
        std::panic::catch_unwind(std::panic::AssertUnwindSafe(|| assert_success_contract(&v)))
            .is_err()
    );
    v.expected
        .value
        .as_mut()
        .unwrap()
        .as_object_mut()
        .unwrap()
        .remove("flags");
    assert!(
        std::panic::catch_unwind(std::panic::AssertUnwindSafe(|| assert_success_contract(&v)))
            .is_err()
    );
}

#[test]
fn semantic_mutations_reach_real_oracle_assertions_without_changing_bytes() {
    let vectors = corpus().vectors;
    let record = vectors.iter().find(|v| v.id == "wire.record-data").unwrap();
    let bytes = fixture_bytes(record);
    let decoded = decode(&bytes).unwrap();
    let mut stale = record.expected.value.clone().unwrap();
    stale["type"] = Value::String("Ack".into());
    assert!(
        std::panic::catch_unwind(|| assert_record_semantics(&record.id, &decoded, &stale)).is_err()
    );
    stale = record.expected.value.clone().unwrap();
    stale["flags"] = Value::from(7);
    assert!(
        std::panic::catch_unwind(|| assert_record_semantics(&record.id, &decoded, &stale)).is_err()
    );
    stale = record.expected.value.clone().unwrap();
    stale["payload_hex"] = Value::String("686f".into());
    assert!(
        std::panic::catch_unwind(|| assert_record_semantics(&record.id, &decoded, &stale)).is_err()
    );

    let frame = vectors.iter().find(|v| v.id == "frame.data").unwrap();
    let frame_bytes = fixture_bytes(frame);
    let decoded_frames = decode_frames(&frame_bytes).unwrap();
    let mut stale = frame.expected.value.clone().unwrap();
    stale["frames"][0]["payload_hex"] = Value::String("686f".into());
    assert!(
        std::panic::catch_unwind(|| assert_frames_semantics(&frame.id, &decoded_frames, &stale))
            .is_err()
    );
    stale = frame.expected.value.clone().unwrap();
    stale["payload_bytes"] = Value::from(99);
    assert!(
        std::panic::catch_unwind(|| assert_frames_semantics(&frame.id, &decoded_frames, &stale))
            .is_err()
    );

    let hello = vectors
        .iter()
        .find(|v| v.id == "negotiation/hello.v0-v2" || v.id == "negotiation.hello.v0-v2")
        .unwrap();
    let mut client = VersionNegotiator::new(NegotiationRole::Client, &[0, 2]).unwrap();
    let selected = client
        .client_accept_response(&fixture_bytes(
            vectors
                .iter()
                .find(|v| v.id == "negotiation.response.selected-0")
                .unwrap(),
        ))
        .unwrap();
    let mut stale = hello.expected.value.clone().unwrap();
    stale["selected"] = Value::from(selected + 1);
    assert!(
        std::panic::catch_unwind(|| assert_negotiation_selected(&hello.id, selected, &stale))
            .is_err()
    );
}
