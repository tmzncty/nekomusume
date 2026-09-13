#![no_main]

use libfuzzer_sys::fuzz_target;
use neko_session::ProcessMessage;
use neko_wire::{
    decode, decode_ack, decode_packet, encode, NegotiationRole, VersionNegotiator,
};

fuzz_target!(|input: &[u8]| {
    // The decoder owns its bounded payload copy. The fuzz oracle is that every
    // byte string returns or rejects without panic, out-of-bounds access, or
    // unbounded allocation; libFuzzer supplies and bounds the input buffer.
    let result = std::panic::catch_unwind(|| {
        if let Ok(record) = decode(input) {
            // Successful decodes preserve the candidate codec invariants. This is
            // a round-trip/property oracle, not a claim of complete protocol
            // validation.
            let encoded = encode(&record).expect("decoder accepts encodable records");
            assert_eq!(encoded.as_slice(), input);
            assert!(record.flags == 0);
            assert!(record.payload.len() <= neko_wire::MAX_PAYLOAD_LEN);
        }
    });
    assert!(result.is_ok(), "decoder panicked on fuzz input");

    // Runtime process messages are also peer-controlled after authentication.
    // Successful decodes must round-trip exactly and remain under the fixed
    // process-frame bound. The corpus seeds include ReadinessRequest/Response.
    let result = std::panic::catch_unwind(|| {
        if let Ok(message) = ProcessMessage::decode(input) {
            let encoded = message.encode().expect("decoder accepts encodable process messages");
            assert_eq!(encoded.as_slice(), input);
            assert!(encoded.len() <= neko_session::PROCESS_FRAME_MAX);
        }
    });
    assert!(result.is_ok(), "process message decoder panicked on fuzz input");

    // Negotiation input is peer-controlled. Every byte string must reject or
    // establish within the fixed version-count/resource bounds without panic.
    let result = std::panic::catch_unwind(|| {
        let mut server = VersionNegotiator::new(NegotiationRole::Server, &[0, 1]).unwrap();
        let first = server.server_accept_hello(input);
        if let Ok(response) = first {
            let state = server.state();
            assert_eq!(server.server_accept_hello(input), Ok(response));
            assert_eq!(server.state(), state);
        }
    });
    assert!(result.is_ok(), "negotiation parser panicked on fuzz input");

    // Packet-recovery wire surfaces are peer-controlled before authentication.
    // The ACK range decoder and packet-header splitter must never panic,
    // index out of bounds, or allocate unboundedly. Successful ACK decodes
    // must re-encode to a canonical form (the canonical form may differ from
    // the input if the input was noncanonical-but-accepted — canonical ACK
    // decode only accepts canonical bytes, so decode==encode for accepted
    // inputs).
    let result = std::panic::catch_unwind(|| {
        if let Ok(ack) = decode_ack(input) {
            assert!(ack.ranges.len() <= neko_wire::MAX_ACK_RANGES);
            let encoded = neko_wire::encode_ack(&ack)
                .expect("decoder accepts canonical ACK payloads");
            assert_eq!(encoded.as_slice(), input, "canonical ACK round-trip");
        }
        if let Ok((_n, rest)) = decode_packet(input) {
            // The packet header consumes exactly 8 bytes; the remainder is the
            // untouched record bytes.
            assert_eq!(rest.len(), input.len().saturating_sub(8));
        }
    });
    assert!(result.is_ok(), "packet-recovery decoder panicked on fuzz input");
});
