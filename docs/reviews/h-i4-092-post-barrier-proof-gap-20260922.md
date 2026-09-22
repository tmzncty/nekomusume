# H-I4-092 continuation — post-barrier proof is not barrier-derived

**Severity:** HIGH (release/item-4 evidence-oracle correctness; not a demonstrated runtime leak)

**Reviewed anchor:** `5cfb2e9d79b7f20807a2a3192560c5b7ec35f6ee`

## Scope

Independent exact-current re-review of the developer closure at `7f678fcac75233a1eac5e59875d56e12941e15cf` and the resulting handoff/provenance state. Reviewed the current `crates/neko-cli/tests/probe.rs` resource snapshot helper, malformed classification barrier, malformed-churn process test, `docs/notes/h-i4-092-provenance-7f678fc-20260922.md`, current handoff, release packet, `SECURITY.md`, `IMPLEMENTATION_PLAN.md`, Session v0 boundary and standing VPS policy.

No reviewer-local Rust/full-gate execution, fuzz, WAN or performance run is claimed. The `7f678fc` clean exact-tree record remains developer-reported local provenance. No visible hosted status/workflow result was treated only as absence of hosted evidence.

## Accepted closure facts

`gated_resource_snapshot(gate, pid)` does fix the original split-refactor defect where `process_resource_snapshot(pid)` could be moved independently from a nearby assertion. The affirmative pre-churn snapshot now executes its `!churn_started` assertion inside the same helper invocation as the measurement, so moving that helper invocation after `churn_started = true` deterministically fails. The exact-current post snapshot is also physically after `malformed_classification_barrier(...)`, and the current runtime order remains correct.

## Remaining finding

The original H-I4-092 closure contract required the post-barrier snapshot helper to require a value/token that **only exists after successful `malformed_classification_barrier(...)`**. The current implementation does not satisfy that part.

Current shape:

```rust
let (child, reader_handle) = malformed_classification_barrier(...)?;
#[cfg(target_os = "linux")]
let barrier_complete = true;
...
let after = gated_resource_snapshot(
    || assert!(barrier_complete),
    pid,
);
```

`gated_resource_snapshot` accepts any `FnOnce()` gate. `barrier_complete` is a freely constructed boolean, not a proof returned by the barrier. Therefore a refactor can move both `let barrier_complete = true` and the coupled snapshot above the barrier call (or invoke the helper with an always-successful closure), and the post snapshot still compiles and can pass while occurring before malformed classification/cleanup completion.

That is materially different from the pre-churn edge: the pre edge is tied to mutable phase state that is already changed before the first malformed send, whereas the post edge currently depends only on a manually placed constant `true`.

This does **not** show that the current runtime leaks resources. It shows that the release/item-4 resource oracle can silently regress back into a pre-barrier sample while preserving a green test, so H-I4-092 cannot truthfully be considered mechanically closed yet.

## Closure contract

Keep this test-only and minimal.

1. Preserve exact-current runtime order, Linux-only `/proc` evidence, FD/RSS margins, `ATTEMPTS`, bounded stdout wait, kill/reap + reader-join failure cleanup, and post-`preauth.release(admission)` diagnostic ordering.
2. Replace the freely constructed `barrier_complete = true` proof with a barrier-derived success value/token. The simplest shape is for `malformed_classification_barrier(...)` to return a small proof value alongside the child/reader handle, or for a tiny wrapper to return such a proof only on its `Ok` path.
3. The Linux post snapshot helper must require that proof value as an argument; it must not accept a generic always-successful closure for the post edge.
4. Moving the post snapshot call before successful barrier completion must therefore fail to compile or deterministically fail without relying on later log counting.
5. Keep the pre-churn coupled helper behavior and existing missing-event / early-EOF regressions unchanged.
6. Do not introduce new capacity/security/TTL/history values, do not change Session/Carrier/ACK/crypto/wire semantics, and do not promote `malformed_or_unadmitted` beyond diagnostic evidence.
7. No fuzz is required unless decoder/parser/crypto framing owners change.
8. On the final pushed source/test SHA run focused CLI process tests, `PYTHONDONTWRITEBYTECODE=1 bash scripts/check.sh`, `git diff --check`, verify a clean tree, and persist exact reachable SHA + UTC start/end + exit codes + OS/arch + stable Rust version as developer-local provenance.

## Queue effect

Reopen H-I4-092 at the front only for this post-barrier proof seam. After closure continue immediately with the already queued exact-current I4-PORT-RES causal re-challenge, pre-auth malformed/rejection resource accounting, cross-platform CLI/process semantics, diagnostic/machine-output boundary, boundedness reconciliation, item-4/release-packet factual reconciliation, repository-wide 13-surface refill, and conditional live.

`READY_LIVE: none`; release items 3 and 4 remain incomplete. `RELEASE_CANDIDATE=false`, `PRODUCTION_READY=false`, `FREEZE=false`, and `RELEASED=false` remain unchanged.
