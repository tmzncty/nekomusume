# Independent R9-11B re-review — H-R9-081 closure oracle gap

**Severity:** HIGH — release-gate evidence / closure truth

**Exact reviewed anchor:** reachable `main` exact `2b660d3fc0b5da47a44cc4807f8b93dc04b43e8c`; current source/test repair is exact `320279601ed7a414dfef81e667aad6b30526907e` (`4f40b2d` source fix + `18a67a7` focused regression + `3202796` restored adjacent `#[test]`).

**Owners inspected:** `crates/neko-carrier/src/lib.rs::{PathRecovery::on_sent,PathRecovery::on_ack,PathRecovery::on_pto,PathRecovery::quiesce,PathRecovery::fresh_health_sample,ReliableUdpRuntime::teardown,ReliableUdpRuntime::poll_health}` plus `health_evidence_tests::quiesce_cannot_manufacture_fresh_health_from_pre_quiesce_outcome`, the exact H-R9-081 repair contract at `eeb49e4`, and the current handoff.

## What is accepted

The source repair itself is correct for the originally demonstrated defect: `PathRecovery::quiesce()` now marks the current `outcome_epoch` consumed and aligns `last_health_sent` / `last_health_lost` to the current resolved counters. Therefore an unconsumed pre-quiesce outcome cannot be re-projected merely because quiesce ran. The current focused regression exercises that unconsumed case and requires the immediate post-quiesce `fresh_health_sample()` to be `None` while lifetime diagnostics survive.

GitHub-hosted Rust CI run `35488510885` completed `success` on exact `3202796`. The current handoff also records developer-local clean exact-tree provenance for exact `3202796`: `PYTHONDONTWRITEBYTECODE=1 bash scripts/check.sh` exit 0, `git diff --check` exit 0, clean tree, Linux x86_64, rustc 1.98.0. Those remain developer/hosted evidence respectively; no reviewer-local execution is claimed here.

## Finding

The closure is still stronger than the committed regression coverage required by the accepted H-R9-081 repair contract.

The finding contract explicitly required both pre-quiesce freshness shapes:

1. a resolved health outcome that was **already consumed** before `quiesce()`; and
2. a resolved health outcome/delta that remained **unconsumed** before `quiesce()`.

Exact `18a67a7` / `3202796` covers only shape 2. The test itself says `Pre-quiesce resolved outcome exists but unconsumed`, then immediately calls `quiesce()` and checks `None`. There is no direct regression for `resolved outcome -> fresh_health_sample() consumes it -> quiesce() -> fresh_health_sample() == None`.

The same accepted contract also called out public-`PathRecovery` reuse: because `PathRecovery::quiesce()` does not terminalize the object and its public `on_sent` / `on_ack` methods remain callable, a strictly post-quiesce resolved outcome must become fresh exactly once, with interval deltas starting at the quiescent boundary rather than replaying pre-quiesce loss. The current source appears to implement this correctly, but there is no mutation-sensitive regression pinning that behavior.

This note therefore does **not** claim a current source correctness failure after `4f40b2d`; it is a release-item-4 closure-truth gap against an explicit previously accepted regression contract. The source fix should remain; do not redesign it merely to satisfy the tests.

## Minimal closure contract

Do not redesign Recovery/health, Session/Carrier layering, ACK semantics, wire/crypto, or any policy value.

1. Keep the existing unconsumed-outcome regression.
2. Add a focused consumed-outcome case: produce a real resolved outcome, consume it once with `fresh_health_sample()`, call `quiesce()`, and require the immediate next `fresh_health_sample()` to be `None` with lifetime diagnostics unchanged.
3. Add a focused post-quiesce reuse case: after the quiescent boundary, send and resolve genuinely new work; require exactly one fresh sample for that new resolved outcome and then `None` on a repeated poll. The interval result must be derived only from post-quiesce resolved counters; pre-quiesce resolved loss/history may survive diagnostically but must not leak into the new interval.
4. Preserve existing H-R9-081 source semantics unless the new regression exposes a concrete defect. No terminal flag is requested for public `PathRecovery`.
5. Run focused `neko-carrier` tests, then on the final pushed developer SHA run `PYTHONDONTWRITEBYTECODE=1 bash scripts/check.sh`, `git diff --check`, and verify a clean tree with exact pushed SHA, UTC start/end, exit codes, OS/arch, and stable Rust version.
6. No decoder/parser/crypto-framing code is involved; do not mechanically run fuzz.

After these oracle gaps are closed, immediately redo the R9-11B bounded terminal-ownership remainder on the repaired exact tree and continue to R9-11C without waiting for reviewer cadence.

`READY_LIVE: none`. No new real-network question is created by this finding.
