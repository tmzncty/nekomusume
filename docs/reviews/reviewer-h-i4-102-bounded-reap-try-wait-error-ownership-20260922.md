# H-I4-102 — `bounded_reap_or_kill` still abandons ownership on its own `try_wait` error

**Severity:** HIGH for release item 4 / process-test evidence correctness.

**Reviewed current tree:** `6ed7dc28ddf5a32ff07c3adf0e3c127983d1e6f9` (`main` at review start).

**Scope:** exact-current `crates/neko-cli/tests/probe.rs`, the developer-owned H-I4-101 repair `5da05085c1b006633174bba123db0f68ff88af8c`, its persisted developer-local provenance, and the remaining process-cleanup owner sweep required by the current handoff.

## Accepted preceding repair

H-I4-101 is closed on its two original call-site claims. Exact `5da0508` routes `bounded_wait_exit`'s `try_wait` error and `finish_server`'s `try_wait` error through the new `bounded_kill_reap` primitive before the error/panic escapes. Developer-local exact-tree provenance is retained in `docs/notes/h-i4-101-provenance-5da0508-20260922.md`; GitHub-hosted Rust CI for exact `5da0508` also completed successfully. Those are distinct evidence classes and neither is reviewer-local execution.

## Concrete counterexample

The shared cleanup wrapper itself still has the old ownership hole:

```rust
fn bounded_reap_or_kill(child: &mut Child) {
    match child.try_wait() {
        Ok(Some(_)) => return,
        Err(e) => {
            panic!("bounded_reap_or_kill: try_wait failed before cleanup: {e}");
        }
        Ok(None) => {}
    }
    bounded_kill_reap(child);
}
```

If this wrapper's own initial `try_wait()` returns `Err`, child exit is unproven and the function panics **before any termination/reap attempt**. The new H-I4-101 primitive therefore closes only callers whose *prior* exit observation failed; it does not close the same failure class when the observation happens inside `bounded_reap_or_kill` itself.

This is not a dead branch. Exact-current callers use `bounded_reap_or_kill` as the convergence primitive on readiness timeout/EOF/read-error/channel-disconnect, malformed-classification timeout/EOF/channel-error, bounded-wait deadline expiry, and other process-test cleanup sites. Every one of those callers can still reach this initial observation error and abandon ownership while comments claim that timeout/failure cleanup is bounded and deterministic.

A representative current readiness failure path is:

```rust
Err(_) => {
    bounded_reap_or_kill(&mut child);
    Err("child produced no ready marker within bound".to_string())
}
```

The outer path is wall-clock bounded until cleanup starts, but it does not have the claimed cleanup/resource-convergence property if the wrapper's first `try_wait` errors. Dropping `std::process::Child` is not a kill/reap guarantee.

This finding does **not** establish a production/runtime transport leak and does not reopen Session/Carrier/ACK/crypto/wire architecture. It is an item-4 process-harness ownership/evidence defect.

## Closure contract

1. Make `bounded_reap_or_kill`'s own `try_wait`-error branch obey the same ownership rule already adopted for H-I4-101: before an error/panic escapes, perform the existing bounded best-effort termination/reap convergence (normally `bounded_kill_reap`) instead of panicking immediately with child exit unproven.
2. Keep the distinction between `Ok(Some(_))` (exit already proven), `Ok(None)` (live child -> bounded termination/reap), and `Err(_)` (observation failed -> bounded best-effort termination/reap, then explicit failure) truthful. Do not silently relabel an OS cleanup failure as successful cleanup.
3. Do not weaken H-I4-097..101 bounds, H-I4-099's off-thread stdout drain, readiness/barrier reader ownership, or the H-I4-090..095 malformed-resource causality proofs. No runtime/session/wire semantics change is needed.
4. Keep the repair test-local and narrow. Reuse `bounded_kill_reap`; do not build a generic process framework or invent repository-wide timeout/security numbers.
5. A synthetic kernel `try_wait` failure is not required if exercising one would need unsafe/platform-specific machinery. Source-level ownership proof plus existing deterministic live-child deadline/EOF regressions is acceptable; add only a small deterministic regression if a natural seam exists.
6. Final pushed source/test SHA: run `PYTHONDONTWRITEBYTECODE=1 bash scripts/check.sh`, `git diff --check`, confirm clean tree, and persist developer-local exact-tree provenance with exact reachable SHA, UTC start/end, exit codes, OS/arch and stable Rust. No fuzz unless decoder/parser/crypto-framing owners change.
7. After repair, continue the owner-by-owner process `try_wait` / direct or indirect `wait` / `wait_with_output` / `output` / stdout-stderr drain / reader-join / child-ownership sweep; do not wait for reviewer cadence and do not declare repository-wide queue exhaustion from this narrow closure.

## Evidence boundary

This review is exact-current GitHub source/control-flow reasoning only. The reviewer did not execute Rust tests, the full local gate, cross-platform tests, fuzz, WAN, or performance work in this pass. `READY_LIVE` remains `none`; release items 3 and 4 remain incomplete; `RELEASE_CANDIDATE=false`, `PRODUCTION_READY=false`, `FREEZE=false`, and `RELEASED=false` remain unchanged.
