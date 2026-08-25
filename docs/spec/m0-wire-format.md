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
