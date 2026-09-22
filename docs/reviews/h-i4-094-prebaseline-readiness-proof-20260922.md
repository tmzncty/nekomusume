# H-I4-094 — pre-churn resource baseline is not mechanically bound to server start

**Severity:** HIGH (release/item-4 causal resource-evidence correctness)

**Reviewed exact tree:** `76a04b6b00201032b000646ed497dc98adb4fa4b`

**Developer commits reviewed:**

- `b103c9784805b48f8b488d0af705c940c1f82993` — H-I4-093 non-Linux Unix `BarrierProof` sink.
- `76a04b6b00201032b000646ed497dc98adb4fa4b` — handoff/provenance reconciliation only.

This is an exact-current static/source review. No reviewer-local Rust/full gate, cross-platform execution, WAN run, fuzz run, or performance conclusion is claimed. The persisted `b103c97` clean gate remains developer-reported Linux x86_64 provenance; absence of visible hosted status is only absence of hosted evidence.

## Accepted from H-I4-093

H-I4-093 is closed at `b103c97`: on non-Linux Unix the barrier-derived proof is explicitly sunk, while Linux still consumes the proof inside the post-resource snapshot gate. The current post-churn causal edge remains materially coupled:

`all ATTEMPTS malformed inputs classified after corresponding cleanup -> BarrierProof -> post /proc snapshot`.

The existing bounded timeout/disconnect/EOF cleanup, reader ownership, post-`preauth.release(...)` diagnostic ordering, FD/RSS margins and `ATTEMPTS` are retained.

## Concrete defect

The current test physically does the right thing:

```text
spawn server -> wait for diagnostic start -> pre-churn baseline -> mark churn_started -> first malformed send
```

and the server's diagnostic `start` is emitted only after the UDP/TCP binds, socket setup, pre-auth controller construction, SessionRuntime construction/open-stream and related startup ownership have been established.

However, the **baseline oracle itself only proves `!churn_started`**. The Linux baseline is currently acquired as:

```rust
let server = ready_failover_server(server);
let mut churn_started = false;
let before = gated_resource_snapshot(
    || assert!(!churn_started, "baseline must precede the churn"),
    pid,
);
```

`pid` is available before `ready_failover_server(...)`, and `gated_resource_snapshot(...)` consumes no value that can only exist after the server diagnostic `start` has been observed.

Therefore this whole-block mutation is still legal:

1. move `let mut churn_started = false;` and the baseline `gated_resource_snapshot(...)` block above `ready_failover_server(server)`;
2. leave `churn_started = true` before the first malformed send.

The code still compiles and the baseline guard still passes because `churn_started == false`, but the resource measurement can now occur **before the server start barrier**. No current oracle mechanically detects that loss of the intended first causal edge.

This violates the current I4-PORT-RES challenge contract:

`server READY/start -> affirmative pre-churn snapshot -> first malformed send`.

The exact-current physical ordering is correct, so this finding is **not** a demonstrated runtime leak or transport defect. It is an evidence-oracle defect: a later refactor can silently move the baseline into server startup and invalidate the intended bounded-growth comparison.

## Closure contract

Apply the smallest test-only coupling repair:

1. Introduce a private proof/value that is obtainable only after `ready_failover_server(...)` has observed the server diagnostic `start`, or an equivalent ownership/type coupling.
2. On Linux, the **actual pre-churn `/proc` acquisition must consume that readiness-derived proof inside the same helper call that invokes `process_resource_snapshot(pid)`**. Moving the measurement above the ready/start barrier must therefore fail to compile or deterministically fail.
3. Keep the existing `!churn_started` proof in the same measurement path so the baseline still cannot move after the first malformed send.
4. Preserve the accepted post side: successful `malformed_classification_barrier(...)` derives `BarrierProof`, Linux post snapshot consumes it, and non-Linux Unix remains warning-clean.
5. Preserve H-I4-090/H-I4-091 semantics: bounded off-thread classification wait; timeout/disconnect/EOF kill/reap + reader join; `malformed_or_unadmitted` only after `preauth.release(admission)` where an admission exists; current FD/RSS margins and `ATTEMPTS`.
6. Do not change runtime Session/Carrier/ACK/crypto/wire semantics, D019, TTL/LRU/history/capacity/security values, release flags or publication/signing policy.
7. No fuzz is required unless decoder/parser/crypto framing owners change.
8. Run focused CLI process tests, then on the final pushed source/test SHA run `PYTHONDONTWRITEBYTECODE=1 bash scripts/check.sh`, `git diff --check`, verify a clean tree, and persist exact reachable SHA, UTC start/end, exit codes, OS/arch, Rust stable version and clean-tree state.

A small `ReadyProof` returned alongside `ReadyServer` is one acceptable shape; do not build a generic framework merely for this oracle.

## Queue consequence

Front H-I4-094 before expanding the review surface. After repair/provenance, complete the I4-PORT-RES causal re-challenge, then continue the retained rolling queue: pre-auth malformed/rejection accounting; cross-platform CLI/process semantics; diagnostic/machine-output boundary; algorithmic/resource boundedness; package/build reproducibility spot re-challenge; item-4/release-packet factual reconciliation; repository-wide 13-surface refill; conditional live only if a materially new real-network question becomes READY.

`READY_LIVE: none`; release items 3/4 remain incomplete; `RELEASE_CANDIDATE=false`, `PRODUCTION_READY=false`, `FREEZE=false`, and `RELEASED=false` remain unchanged.
