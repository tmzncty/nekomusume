# Independent R9-11B re-review — H-R9-081 post-quiesce interval oracle gap

**Severity:** HIGH — release-gate evidence / closure truth

**Exact reviewed anchor:** reachable `main` exact `cdce5b8da56b80fcfc19bf9b82ccd0629f37fcbe`; current source/test repair is developer exact `c52efef3069c8b5ba8430ec4fee6657365f9ee92` on top of `4f40b2d` / `18a67a7` / `3202796`.

**Owners inspected:** `crates/neko-carrier/src/lib.rs::{PathRecovery::on_sent,PathRecovery::on_ack,PathRecovery::quiesce,PathRecovery::fresh_health_sample}`, the original H-R9-081 contract at `eeb49e4`, the closure-gap contract at `16a6413`, the new regressions in exact `c52efef`, and `docs/CHATGPT_HANDOFF.md`.

## What remains accepted

The source repair in exact `4f40b2d` remains correct for the demonstrated freshness defect. `quiesce()` marks the current outcome consumed and aligns `last_health_sent` / `last_health_lost` with the current resolved counters. The exact `c52efef` regressions now directly cover:

- an already-consumed pre-quiesce outcome staying consumed; and
- a genuinely post-quiesce resolved outcome becoming fresh once and then consumed.

No source rollback or redesign is requested.

## Finding

The accepted H-R9-081 closure contract also required the post-quiesce interval sample to prove that **pre-quiesce resolved loss/history cannot leak into the next interval**. Exact `c52efef` does not make that assertion mutation-sensitive.

`post_quiesce_outcome_is_fresh_once_then_consumed` sends packet numbers 0 and 1, ACKs only packet 0, consumes that health outcome, quiesces, then sends and ACKs packet 3. It never creates a pre-quiesce resolved loss. After the post-quiesce ACK it only checks `fresh_health_sample().is_some()` and then `None`; it does not inspect `HealthSample.loss_per_mille` (or any equivalent interval value).

Therefore an implementation mutation that accidentally preserved/replayed pre-quiesce `resolved_lost` in the first post-quiesce interval could still satisfy the committed regression. The test comment says interval deltas start at the quiescent boundary, but the oracle does not prove that claim.

This is not a new source-correctness accusation against exact `4f40b2d`; current source visibly aligns `last_health_*` at quiesce. It is a release-item-4 closure-truth gap against an explicit previously accepted regression contract.

## Minimal closure contract

Do not redesign Recovery/health, Session/Carrier layering, ACK semantics, wire/crypto, or any policy value.

1. Keep the current consumed/unconsumed freshness regressions.
2. Make the post-quiesce reuse regression create a **real pre-quiesce resolved loss** (not merely an outstanding packet) and prove that lifetime loss diagnostics still survive quiesce.
3. After quiesce, resolve a clean new packet and assert the first post-quiesce `HealthSample` is derived only from post-quiesce resolved counters — in particular, the clean interval must report `loss_per_mille == 0` even though lifetime/pre-quiesce loss is nonzero.
4. Repeated polling must still return `None`; keep the public reusable `PathRecovery` behavior.
5. If the focused regression exposes a source defect, apply the smallest exact-current-semantic repair. Otherwise test-only closure is sufficient.
6. Run focused `neko-carrier` tests, then on the final pushed developer SHA run `PYTHONDONTWRITEBYTECODE=1 bash scripts/check.sh`, `git diff --check`, and verify a clean tree with exact pushed SHA, UTC start/end, exit codes, OS/arch, and stable Rust version.
7. No decoder/parser/crypto-framing code is involved; do not mechanically run fuzz.

## Evidence boundary

Developer-local clean exact-tree provenance for exact `c52efef` is recorded in the handoff as green. The GitHub connector exposes no hosted workflow run or combined status for exact `c52efef`; absence of a hosted record is not treated as failure. No reviewer-local test execution is claimed here.

`READY_LIVE: none`. This finding creates no new real-network question.
