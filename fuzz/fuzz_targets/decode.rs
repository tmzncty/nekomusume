#![no_main]

use libfuzzer_sys::fuzz_target;
use neko_wire::{decode, encode};

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
});
