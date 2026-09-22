# H-I4-092 — malformed-churn resource oracle can be split from its phase guards

**Severity:** HIGH (release/item-4 evidence-oracle correctness; not a demonstrated runtime resource leak)

**Reviewed anchor:** `81db25702ffc806dbbde6884faa17f3a9c003502`

## Scope

Independent exact-current I4-PORT-RES causal re-challenge after H-I4-091 closure. Reviewed:

- `crates/neko-cli/tests/probe.rs` malformed-churn resource test and `malformed_classification_barrier` helper;
- exact developer source/test commit `8e507ac09b971ee9c3adbc7d3d452f9a87c29ec7`;
- `docs/notes/h-i4-091-provenance-8e507ac-20260922.md`;
- current `docs/CHATGPT_HANDOFF.md`, `SECURITY.md`, `docs/status.md`, `IMPLEMENTATION_PLAN.md`, `docs/specs/nekomusume-session-v0.md`, and release-review packet boundaries.

No reviewer-local Rust/full-gate execution, fuzz, WAN, or performance run is claimed here. The exact `8e507ac` clean gate remains developer-reported local provenance. No visible hosted status/workflow result was treated only as absence of hosted evidence.

## Accepted H-I4-091 closure facts

The exact-current tree correctly keeps `barrier_complete` Linux-only, retains the bounded off-thread reader barrier, restores child/reader ownership on EOF/timeout/disconnect, and includes the focused early-EOF regression. The actual current execution order is also correct: affirmative Linux baseline precedes malformed sends, and the post `/proc` snapshot follows successful malformed classification/cleanup barrier completion.

## Finding

The mutation-sensitive ordering claims are still separable from the measurements they are intended to guard.

### Pre-churn edge

Current shape is conceptually:

```rust
let mut churn_started = false;
let before = {
    assert!(!churn_started);
    process_resource_snapshot(pid)
};
...
churn_started = true;
// malformed sends
```

This catches moving the **whole block** after `churn_started = true`, but it does not catch a split refactor that leaves the assertion in the pre-churn position and moves only `process_resource_snapshot(pid)` into or after the churn window. The assertion still passes while the baseline becomes post-churn.

### Post-barrier edge

Current shape is conceptually:

```rust
let (...) = malformed_classification_barrier(...)?;
let barrier_complete = true;
...
{
    assert!(barrier_complete);
    let after = process_resource_snapshot(pid);
    // compare before/after
}
```

This catches moving the **whole snapshot block** above the barrier because `barrier_complete` would not yet exist. It does not catch hoisting only `let after = process_resource_snapshot(pid)` above the barrier while leaving the `assert!(barrier_complete)` and comparison below it. That mutation compiles, the assertion remains true later, and the supposedly post-barrier sample is actually pre-barrier.

Therefore the handoff claim that moving/splitting the resource oracle necessarily fails closed is false. The current tree's measurement order is correct, but the regression protection that was used to close H-I4-090/H-I4-091 is not mechanically coupled to the measurement operation.

## Closure contract

Use the smallest test-only coupling repair; do not redesign runtime diagnostics or resource policy.

1. Preserve the current actual ordering, Linux-only `/proc` boundary, FD/RSS margins, bounded malformed attempt count, and post-`preauth.release` diagnostic semantics.
2. Couple each Linux resource snapshot to its phase proof so the assertion/token and `process_resource_snapshot(pid)` cannot be separated by an ordinary refactor. A small Linux-only helper that performs both the phase check and snapshot acquisition is sufficient; no generic phase/checker framework is wanted.
3. For the pre-churn sample, calling the coupled snapshot helper after churn starts must deterministically fail.
4. For the post-churn sample, the coupled snapshot helper must require a success token/value that only exists after `malformed_classification_barrier(...)` returns `Ok`; moving the coupled call before barrier success must fail to compile or fail deterministically.
5. Keep the H-I4-091 early-EOF and missing-classification regressions. Do not weaken child kill/reap + reader join cleanup.
6. Do not change capacity/security/TTL/history values and do not turn the diagnostic into protocol/authentication/Session/Carrier/ACK semantics.
7. No fuzz is required unless decoder/parser/crypto framing owners change.
8. On the final pushed source/test SHA run focused CLI process tests, then `PYTHONDONTWRITEBYTECODE=1 bash scripts/check.sh`, `git diff --check`, and verify a clean tree. Persist exact reachable SHA, UTC start/end, exit codes, OS/arch, stable Rust version, and clean-tree state as developer-local provenance.

## Queue effect

Front H-I4-092 ahead of further I4-PORT-RES closure. After repair, independently re-challenge the coupled pre/post causal edges, then continue the existing pre-auth resource-accounting, cross-platform CLI/process, diagnostic-boundary, boundedness, item-4/release-packet, repository-wide refill, and conditional-live lanes.

`READY_LIVE: none`; release items 3/4 remain incomplete and all release/governance flags remain unchanged.
