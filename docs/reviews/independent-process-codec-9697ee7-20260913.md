# Independent bounded ProcessMessage / Resume / readiness codec review — exact `9697ee7`

Bounded independent review of `ProcessMessage`/`ResumeWireBinding`/`ReadinessRequest`/`ReadinessResponse` encode/decode (`ProcessCodecError`, `PROCESS_FRAME_MAX`, `encode`, `decode`) in `crates/neko-session/src/lib.rs` (`:908-1144`), at reachable exact `9697ee7a0045b39c6ef46c1951fefea486ccf13b`. Static decoder/framing review; the pinned decode-fuzz policy applies if this surface's byte handling is later changed. Not a WAN claim, not an independent security/release approval.

## Challenged invariants and result

- **Exact length + trailing-byte rejection — holds.** `decode` rejects `len < 4`, `len > PROCESS_FRAME_MAX`, wrong magic, and `version != 1` up front (`:1040-1045`), then enforces an exact total per kind: Data `input.len() == 30 + len` (`:1062`), DeliveryAck `== 36`, Resume `== 69`, ReadinessRequest `== 44`, ReadinessResponse `== 45`. No trailing bytes are tolerated. `readiness_codec_rejects_every_truncation_trailing_and_mutated_boolean` covers truncation/trailing on the readiness pair.
- **`usize`/`u64` conversion + offset/length overflow — holds.** `u64_at` uses `get(at..at+8)` bounds-checked slices (`:1048-1053`); Data `len` is a `u16` so `30 + len` cannot overflow; DeliveryAck `len` round-trips through `u64::try_from` on encode (`:979`) and `usize::try_from` on decode (`:1079`, `LengthOverflow` on failure). All arithmetic is bounds- or `checked`-safe before any `input[...]` index.
- **Malformed `admitted` values — holds.** `ReadinessResponse` accepts only `0`/`1` at `input[44]`; any other byte is `Malformed` (`:1126-1130`). The mutated-boolean regression covers this.
- **Data/DeliveryAck/Resume/Readiness semantic separation — holds.** Kind tags 1/2/3/4/5 decode to disjoint variants; a `Data` frame cannot be reinterpreted as an ACK or resume token.
- **Session/path-generation/delivery-epoch tuple preserved — holds.** `Resume` carries `delivery_epoch`/`key_phase`/`path_generation`/`expires_at_ms`/`token`; the readiness pair carries `path_generation`/`delivery_epoch`/`challenge_id`/`target_path` — each field round-trips byte-exact.
- **Decode success cannot silently create auth/admission/delivery/readiness evidence — holds.** `decode` returns only the message structure; it performs no state transition, trust, admission, or watermark change. All evidence is produced by the `SessionRuntime`/`DeliveryLedger` validated transitions, not by parsing.
- **Deterministic round-trip + unknown-kind/version failure — holds.** `decode(encode(m)) == m` is tested for all five kinds (`:1811-1830`, `:2388-2399`); unknown kind (`NK\x02…`) and unknown version (`NK\x02`) are `Malformed`.

## Evidence

- `cargo test -p neko-session` on exact `9697ee7`: codec round-trip, `readiness_codec_rejects_every_truncation_trailing_and_mutated_boolean`, and `d064_readiness_codec` tests all pass.
- No code change; no defect found. No new `READY_LIVE` question; release/governance state unchanged.
