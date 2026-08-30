#![no_main]

use libfuzzer_sys::fuzz_target;
use neko_wire::{decode, encode, NegotiationRole, VersionNegotiator};

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

    // Negotiation input is peer-controlled. Every byte string must reject or
    // establish within the fixed version-count/resource bounds without panic.
    let result = std::panic::catch_unwind(|| {
        let mut server = VersionNegotiator::new(NegotiationRole::Server, &[0, 1]).unwrap();
        let _ = server.server_accept_hello(input);
    });
    assert!(result.is_ok(), "negotiation parser panicked on fuzz input");
});
