#![no_main]

use libfuzzer_sys::fuzz_target;
use neko_wire::decode;

fuzz_target!(|input: &[u8]| {
    // The decoder owns its bounded payload copy. The fuzz oracle is that every
    // byte string returns or rejects without panic, out-of-bounds access, or
    // unbounded allocation; libFuzzer supplies and bounds the input buffer.
    let _ = decode(input);
});
