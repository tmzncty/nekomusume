# M0 candidate wire format

**Status: candidate implementation, not frozen normative v0.**

`neko-wire` currently provides a pure synchronous codec for review. It does not open sockets, select a runtime, authenticate data, or implement failover. The field values and semantics below remain provisional until the normative gates in `docs/specs/nekomusume-session-v0.md` are completed and reviewed.

## Candidate record

All fixed-width integers are big-endian:

| Offset | Size | Field |
|---:|---:|---|
| 0 | 2 | magic `NK` |
| 2 | 1 | version `0` |
| 3 | 1 | record type: `1=Data`, `2=Ack`, `3=PathChallenge` |
| 4 | 1 | flags; candidate currently requires `0` |
| 5 | 4 | payload length, `u32` |
| 9 | N | opaque payload, at most 4096 bytes |

The decoder accepts exactly one record and rejects truncation, unknown version/type, non-zero flags, oversized or mismatched lengths, and trailing bytes.

## Candidate canonical integer

`encode_varint` uses minimal unsigned little-endian base-128 encoding. Non-minimal encodings, truncation, and `u64` overflow are rejected. This helper is not yet assigned to a protocol field.

Golden round-trip vectors and malformed-input tests live beside the implementation. They are evidence for the candidate codec only; they do not constitute a wire freeze.


## Candidate authenticated SessionRecord frames

The existing NK/version/fixed 9-byte header remains the outer Carrier packet
header. Its payload may now be parsed as a bounded authenticated SessionRecord
containing a list of frames. Each frame is `type:u8 || length:u16be || payload`;
maximum frame payload is 1024 bytes, maximum SessionRecord payload remains 4096
bytes, and at most 64 frames are accepted.

The low type bit is the compatibility bit: `0` is critical and `1` is
ignorable. DATA (`0x00`), DELIVERY_ACK (`0x02`), CLOSE (`0x04`),
PATH_CHALLENGE (`0x06`) and PATH_RESPONSE (`0x08`) are candidate known types.
Unknown critical and `0xf0..=0xff` reserved types fail closed; unknown
ignorable types are retained and skipped by higher layers. This is candidate,
non-frozen syntax and does not define authentication/AAD placement.

## Explicit pre-Session version negotiation (N1 candidate)

The opt-in `VersionNegotiator` API runs before Session data admission; existing
record APIs remain unchanged. A client hello is `N1 || 0x01 || count:u8 ||
versions[count]:u16be`, with strictly increasing unique versions and
`1..=16` entries. A server response is `N1 || 0x02 || 0x00 || selected:u16be`.
The server deterministically selects the highest common version. No overlap,
unknown/future-only offers, malformed, duplicate, oversized, unexpected, and
late messages fail closed. After establishment, an exact byte-for-byte duplicate
of the accepted hello replays the exact prior response; contradictory, malformed,
unsupported, and other late messages remain rejected without changing state; received invalid input makes that negotiator
terminal. `admit_data` cannot succeed until selection has completed. This is a
bounded standalone wire/session-boundary primitive, not integration into the
current live carrier/handshake and not a security-closure claim.
