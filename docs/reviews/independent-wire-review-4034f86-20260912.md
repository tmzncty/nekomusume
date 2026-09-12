# Independent bounded wire/parser review and dc52a5e closure decision — exact `4034f86`

**Reviewed revision:** `4034f86ceb356a3e04b8ff713b97338af3e71265` (`origin/main`, worktree clean).
**Review type:** independent bounded fail-closed/allocation-bound challenge of `neko-wire` decoding plus a closure decision on `dc52a5e`. **Date:** 2026-09-12 (UTC `2026-09-12T11:08:40Z`). Host `tmzn@192.168.122.1`, workdir `/media/tmzn/DATA5/nekomusume-work`, rustc 1.98.0, cargo-fuzz 0.13.2.
**Anchoring note:** this review is anchored to reachable `4034f86` only. An earlier request cited `03a9396`; that object does not exist in this repository, so it was not treated as evidence.

## Verdict

1. **`dc52a5e` closure: ACCEPTED.** It repairs a hard compile error; the successor `4034f86` refines the same assertion and also verifies clean.
2. **Wire/parser (REVIEW 4): no concrete current defect found.** Source review and fuzz evidence are reported separately below.
3. **One HIGH evidence-anchoring finding remains** (unreachable SHAs cited as exact-tree evidence). It is a documentation/provenance defect, not a wire-code defect, and is reported with a reproducer and minimal repair; no file was edited.

## `dc52a5e` closure decision

The parent of `dc52a5e` declared a `#[test] fn rejected_negotiation_close_is_either_eof_or_platform_reset(stream: &mut TcpStream)` — a test with a parameter. Reproduced on a worktree of `dc52a5e^`:

```
$ CARGO_TARGET_DIR=… cargo check --test multistream -p neko-cli
error: functions used as tests can not have any arguments
   --> crates/neko-cli/tests/multistream.rs:145:1
error: could not compile `neko-cli` (test "multistream") due to 1 previous error
```

So the pre-`dc52a5e` tree could not build its test target: the fix is necessary, not cosmetic. `dc52a5e` turned the malformed item into a helper and applied it to the real unsupported-negotiation path; `4034f86` then inlined that assertion into `executable_rejects_unsupported_only_negotiation_before_noise_or_data` and removed the one-use helper. Verified at `4034f86`: `cargo check --test multistream` succeeds and the target runs 6 tests, all passing, including `executable_rejects_unsupported_only_negotiation_before_noise_or_data`. The assertion still fails on `Ok(n>0)` (no payload bytes emitted), so the portability change (`Ok(0)` or `Err(ConnectionReset)`) does not weaken the rejection claim; it is in fact stricter than the original by no longer accepting `BrokenPipe`.

## Wire/parser source review (no defect found)

- `decode` (`crates/neko-wire/src/lib.rs:277-310`): length gate before every indexed read; magic/version/type/flags checked before the declared length is read; `declared > MAX_PAYLOAD_LEN (4096)` rejected before use; `available < declared` and `available > declared` distinguished so trailing bytes fail closed; the payload copy is taken only when `available == declared`, so allocation is bounded by the 4096 cap. `HEADER_LEN` is 9 and the `input[5..9]`/`input[9..]` slices are dominated by the initial `len < HEADER_LEN` check.
- `decode_varint` (`:232-255`): `take(MAX_VARINT_BYTES)` bounds the loop; per-byte shift is `index * 7` with the last index pinned to shift 63 and `part > 1` rejected, so no silent u64 truncation; `checked_shl` never returns `None` in range (shift ≤ 63); non-minimal encodings are rejected at the terminating byte only, which cannot misfire on a canonical value; exhaustion yields `Truncated` for short input and `IntegerOverflow` for full-length input.
- `decode_frames` (`:152-199`): empty input rejected; the 64-frame ceiling is tested before each header read; `input.len() < FRAME_HEADER_LEN` rejected; per-type length policy applied before any payload slice; declared length exceeding remaining input yields `Truncated`; payload copies are bounded by remaining input. `validate_frame_type` rejects `0xf0..=0xff` as reserved, enforces `≤1024` for DATA/DATAGRAM/ACK/CLOSE, exactly 8 for PATH_CHALLENGE/PATH_RESPONSE, and distinguishes unknown-critical (reject) from unknown-ignorable (retain) after masking the ignorable bit. `decode_frames`/`encode_frames` bounds are mutually consistent, and free of attacker-driven unbounded allocation.
- Version negotiation (`:583-772`): sorted/unique/count-bounded supported and offered lists; `decode_hello` requires an exact `4 + count*2` length, so the `bytes[index + 1]` read cannot pass the end (count ≤ 16); `decode_response` requires exactly 6 bytes before its indexed reads; malformed input sets a terminal `Rejected` state; a duplicate hello returns the cached response only on byte-exact equality and only while established.
- `neko-session` `ProcessMessage::decode` (`crates/neko-session/src/lib.rs:1039+`): whole-input `PROCESS_FRAME_MAX (4096)` gate first; nested reads via `get(range)`; exact-length requirements per kind; no unchecked indexing.
- CLI `read_frame` (`crates/neko-cli/src/main.rs`): `u32` length compared against the caller's cap before `vec![0; n]`.
- Panic-freedom audit: `expect`/`unwrap` on peer-controlled paths are dominated by prior explicit length or state checks (`input[5..9].try_into().expect("header checked")`; `accepted_response.clone().expect("accepted response")` guarded by the paired assignment). Deterministic encoding holds for records, frames and canonical varints (minimal, ordered, round-trip exact).

## Wire test evidence (reproduced at `4034f86`)

`cargo test -p neko-wire --all-targets` -> 18 unit + 3 canonical-vector + 2 deterministic property + 3 compatibility = **26 passed, 0 failed**. `cargo clippy -p neko-wire --all-targets -- -D warnings` -> clean.

## Fuzz evidence (separate from the source review)

Pinned smoke executed via `scripts/fuzz-smoke.sh` (cargo-fuzz 0.13.2, nightly, seed corpus `fuzz/corpus/decode`), which is the repository's own decode fuzz entry point covering `neko_wire::decode`/`encode`, `neko_session::ProcessMessage::decode` and `VersionNegotiator::server_accept_hello`:

- `FUZZ_TIME=60` -> `Done 18487710 runs in 61 second(s)`, exit `0`, no crash, no OOM.
- `FUZZ_TIME=20` -> `Done 6328245 runs in 21 second(s)`, exit `0`.
- `fuzz/artifacts/decode/` remained empty; the smoke script fails closed on a crash artifact.

This is bounded fuzz-saturation evidence for the current tree, not a proof of parser safety, and it is not a substitute for the source review above.

## Finding: unreachable SHAs cited as exact-tree evidence (HIGH, evidence only)

**Where:** `docs/release-security-review-packet.md:34`, `docs/reviews/release-item4-subgates-20260909.md:50`, `docs/local-item4-review-reconciliation-6f50d70-20260911.md:12`.
**Reproducer:**

```
$ git cat-file -t 58b5d13   # fatal: could not get object info
$ git cat-file -t c5d0b15   # fatal: Not a valid object name
$ git cat-file -t 77d3b4a   # fatal: Not a valid object name
$ git cat-file -t 03a9396   # fatal: Not a valid object name
```

A scan of every hex token in those three documents at `4034f86` found **8 unreachable commit references presented as exact-tree anchors**: `58b5d13`, `c5d0b15`, `1db7a97`, `7275062`, `7711162`, `1be290d`, `6f50d70` (and its 40-hex form `6f50d700b673b18679ee1ca3e2f6423a5ded3d3`). One further token, `88d9e12ae`, is the rustc build revision, not a repository commit, and is not a defect. The packet's `Exact repository evidence` column therefore cites provenance that cannot be resolved in this repository — the same class the handoff at `ca818c1` already exposed. The associated review notes are present and their reachable introducing commits exist:

| Unreachable anchor | Reachable commit that introduced the note |
|---|---|
| `58b5d13` | `407bfea` |
| `c5d0b15` | `0774ab6` |
| `1db7a97` | `1cf6c79` |
| `7275062` | `68e96e7` |
| `7711162` | `96af2be` |
| `1be290d` | `d0d2f42` |
| `6f50d70` | `89f26b5` |

**Impact:** bounded to evidence anchoring. The underlying code these notes reviewed is present and independently reviewed above with no defect found, so no technical claim is invalidated; but the cited anchors are unverifiable and the referenced review notes must not be counted as exact-tree evidence.
**Minimal repair (not applied here):** in the packet and item-4 documents, re-point each unreachable short SHA to the reachable commit that introduced the corresponding note (table above), and add one line recording that repeated local rebases invalidated pre-rebase SHAs so future provenance is written from pushed commits only.

## Exclusions and remaining gates

Performed: source audit of `neko-wire`/`neko-session` decoding and CLI frame reading, focused test/clippy reproduction, pinned fuzz smoke, and a provenance scan. **Not** performed: wire-format redesign, canonical-corpus migration, independent cryptanalysis, adversarial-load or WAN/VPS execution, penetration testing, or secrets access. No source file was edited; `docs/CHATGPT_HANDOFF.md` was not modified.

Unchanged: `RELEASE_CANDIDATE=false`, `PRODUCTION_READY=false`, `FREEZE=false`, `RELEASED=false`; item 3 and natural-loss unchecked; `D019` `SOURCE_RETENTION_POLICY_BLOCKED`; `RSEC-001` open; HY2/periodic/repeated-warm-failover blocked; IPv6 `BLOCKED_ENVIRONMENT`; PMTUD `BLOCKED_IMPLEMENTATION`; item 4 incomplete.
