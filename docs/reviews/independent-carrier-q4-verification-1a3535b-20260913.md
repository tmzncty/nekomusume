# Independent bounded verification — H-RUDP-014 repair and Q4 byte-accurate suite at exact `1a3535b`

**Reviewed revisions:** `1a3535bf630fc9b317e75fae39cd73c1d2dcfebd` (`fix(carrier): byte-accurate Session offsets in the R7 fixture (H-RUDP-014)`), on top of `96b43b0` (`test(carrier): Q4 coherent R7 scenario suite`) and `f40a6c3`.
**Scope:** `crates/neko-carrier/tests/reliable_udp_runtime.rs` (sha256 `4b34b6cbd1ffe7b4f4a3bf2c29b609d04e95ee3811e2cfc49de660a4093b0491`) and the committed layering state. Read-only; no source edit.
**Date:** 2026-09-13 (UTC `2026-09-13T15:31:34Z`). **Not** an audit or release authorization.

## Verdict

**H-RUDP-014 repair: CORRECT.** **Q4 byte-accurate suite: non-vacuous.** **No correctness defect found.** Two bounded robustness nits recorded (neither currently reachable, no repair required now).

## H-RUDP-014 repair — validated

The finding had two parts; both are fixed and the fixture no longer contains the defect class:

1. **Byte-accurate identity.** `send_data` (`tests/reliable_udp_runtime.rs:131-140`) now advances `next_offset` by `payload.len()` with checked arithmetic instead of one per record, and the clean scenario uses variable-length records `2,3,1,4` → offsets `0,2,5,6`, asserting four first deliveries, zero conflicts, zero duplicate suppression and no retained Session error. Under message-index offsets the second record (offset `1 < next_receive 2`) is rejected by the Session layer as a protocol gap — which the parent independently hit empirically — so this regression is genuinely load-bearing, not cosmetic.
2. **No offset consumed on refusal.** The new offset is committed only after `send_data_at` returns the admitted packet number (`:136-139`), so a cwnd refusal leaves `next_offset` unchanged. `scenario_cwnd_refusal_consumes_no_session_byte_offset` proves this end-to-end: it fills a small window (MSS 120/explicit probe 32), asserts `admitted > 0 && admitted < 64` and that the refused send returns `None`, then after the ACK frees the window asserts the next admitted record lands on the offset the refusal would have used with `session_conflicts == 0` and `session_delivered == admitted + 1`. Vacuity is excluded by the `admitted` bounds.

## Q4 suite — validated

Nine tests in the fixture pass, six of them the new Q4 scenarios: no-loss round trip (variable-length, in-flight drain, zero loss, four byte-accurate deliveries); lost-ACK recovered by a PTO replacement with `session_delivered == 1` and `session_duplicates == 1`; replacement-before-late-original (`session_delivered == 1`, duplicates `1`, conflicts `0`); atomic typed plaintext-bound refusal (`FrameTooLarge`, unchanged in-flight/charged bytes, empty `pto_probe`); clean recovery after old loss without replay (asserts `packets_lost()` unchanged and the later health sample is not degraded); invalid standby never produces a false switch (asserts both that no `WarmFallback` occurs **and** that a `FallbackFailed` was actually attempted, which excludes a vacuous pass). Retransmit path (`pto_retransmit:302-330`) re-encodes the retained plaintext under a fresh packet number/nonce while keeping the stable frame identity, and `packets_lost()`-derived assertions avoid hard-coding recovery reorder policy.

Reproduced: `cargo test -p neko-carrier --all-targets` → lib **70 passed**, fixture **9 passed**, all 15 targets green; `cargo clippy -p neko-carrier --all-targets --locked` warning-free.

## Committed state / anomaly re-verification

- `crates/neko-carrier/Cargo.toml`: production `[dependencies]` remain `neko-reliable` + `neko-wire`; `neko-session` is `[dev-dependencies]` only. `cargo tree -p neko-carrier -e normal` shows **zero** `neko-session` in the production graph.
- Production `crates/neko-carrier/src/`: zero `neko_session` references; `delivered` / `deliver_logical` / `dedup_suppressed` absent.
- `git status` clean; no staged or untracked files; `Cargo.toml`/`Cargo.lock` show no drift; reflog contains only the lane's rebase and its own commits.
- Conclusion: the reported concurrent-editor rework (promoting `neko-session` to `[dependencies]`, and earlier re-adding `delivered`/`dedup_suppressed`) has **not** entered history and is fully reverted at this head. Consistent with the earlier provenance verdict that it did not originate from the review session.

## Bounded observations (nits; not currently reachable)

- **[nit] Fixture aliases Carrier `FrameId.0` to the Session byte offset.** `pto_retransmit` (`:305-330`) encodes `offset: frame.0` with the comment "frame id IS the stable logical offset". That invariant holds only for frames created via `send_data`/`send_data_at`, which use `FrameId(offset)`. `session_layer_suppresses_exact_duplicates_and_fails_closed_on_conflict:501` deliberately constructs `FrameId(5000)` for byte offset `0`, breaking the invariant. It is **not reachable today**: that test never calls `pto_probe`/`pto_retransmit`, and its ACK retires the frame before any PTO. When Q4-F extends the coherent path, a PTO selecting a non-offset-derived frame would re-encode the wrong Session offset and surface as a false Session gap/conflict. Minimal repair for that expansion: keep an explicit `FrameId -> byte offset` map in the fixture (populated in `send_data_frame`) and have `pto_retransmit` read the offset from it rather than from `frame.0`.
- **[nit] `send_data` reports `None` after a send that already succeeded** if `next_offset.checked_add(payload.len())` overflows (`:138`). A caller would then read a completed send as refused. It requires `next_offset` near `u64::MAX` (≈2^64 bytes in one fixture run), i.e. unreachable in bounded runs; noted only because `None` is the fixture's "refused, nothing sent" contract.

## Not claimed / still open

This is a bounded verification of the two commits above; it does not accept R7. Still open per the parent: **Q4-F** (PTO-sample-before-later-loss invariant inside the coherent path — currently proven at the `PathRecovery` unit seam via `901991d` plus the fixture health scenarios), **Q4-E** plaintext-side at the unit seam, the Q4 closure package, **R8/Q8**, **R9/Q9**, **Q10-Q12**, `SessionRuntime.events`, D019, RSEC-001, signing/key-custody/SBOM, previous-frozen-release interoperability, item 3, item 4, and release authority. Release flags unchanged: `RELEASE_CANDIDATE=false`, `PRODUCTION_READY=false`, `FREEZE=false`, `RELEASED=false`; `READY_LIVE: none`.
