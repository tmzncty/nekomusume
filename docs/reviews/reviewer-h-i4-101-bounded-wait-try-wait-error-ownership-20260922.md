# H-I4-101 — bounded child-exit helpers still lose ownership on `try_wait` error

**Severity:** HIGH for release item 4 / process-test evidence correctness.

**Reviewed current tree:** `daaec2fe6d366f288aa40fe3f0ab5e8491ceb0c3` (`main` at review start).

**Scope:** exact-current `crates/neko-cli/tests/probe.rs`, especially `finish_server` and the new H-I4-100 `bounded_wait_exit` helper after accepting source/test commit `833962e4499d2d6b969485176f35151a242e43f9` and its developer-reported exact-tree provenance.

## Accepted preceding repair

H-I4-100 is closed on the original direct-wait defect: `udp_application_wait_fails_at_bounded_overall_deadline` no longer calls blocking `Child::wait()` on an unproven-live child. It calls `bounded_wait_exit(&mut server.child, Duration::from_secs(5))`; the deadline path routes through `bounded_reap_or_kill`, and the existing semantic timing oracle remains in place. Exact-tree developer provenance is retained in `docs/notes/h-i4-100-provenance-833962e-20260922.md`.

## Concrete counterexample

The new helper documents a stronger ownership postcondition than it actually implements:

```rust
fn bounded_wait_exit(
    child: &mut Child,
    deadline: Duration,
) -> Result<std::process::ExitStatus, String> {
    // ...
    match child.try_wait() {
        Ok(Some(status)) => return Ok(status),
        Ok(None) => {
            if Instant::now() >= end {
                bounded_reap_or_kill(child);
                return Err("bounded_wait_exit: child did not exit within deadline".to_string());
            }
        }
        Err(e) => return Err(format!("bounded_wait_exit: try_wait failed: {e}")),
    }
}
```

The `Err(e)` branch returns immediately without establishing child exit and without attempting the repository's bounded termination/reap path. Its only current caller immediately applies `.expect(...)`; therefore this error shape panics while the child is still ownership-unresolved. Dropping `std::process::Child` is not a kill/reap guarantee, so a failed exit observation can leave the process and its pipe/resource ownership outside the claimed cleanup contract.

`finish_server` has the same exact-current shape:

```rust
Err(e) => {
    panic!("finish_server: try_wait failed: {e}");
}
```

At that point its stdout reader thread may still be blocked on the live child's pipe. The branch is bounded in *caller wall-clock control flow* because it panics, but it is not bounded cleanup/resource convergence, which is the property H-I4-097..100 and the item-4 process evidence are claiming.

This finding does **not** establish a production/runtime transport leak and does not reopen Session/Carrier/ACK/crypto/wire architecture. It is a process-test ownership/evidence defect: the exact-current source still has failure results where exit is unproven and cleanup is abandoned.

## Closure contract

1. Repair `bounded_wait_exit` so every return/panic shape either has an exit proof or performs a bounded best-effort termination/reap convergence before ownership is abandoned. A `try_wait` error must not simply return an ordinary `Err` that the caller then panics on while the child remains unresolved.
2. Apply the same ownership rule to `finish_server`'s `try_wait` error branch. Preserve its off-thread stdout drain and the existing normal-exit log collection; do not reintroduce a blocking drain or blocking `wait()` on an unproven-live child.
3. Reuse/refactor the current test-local cleanup primitive only as needed. If `bounded_reap_or_kill` cannot safely express cleanup after an initial `try_wait` error because it immediately repeats `try_wait`, split the minimum test-local termination/reap primitive needed to make error ownership explicit. Do not create a general process framework or invent a repository-wide timeout/security policy.
4. The repair must remain fail-closed if termination/reap itself errors. It is acceptable for an unrecoverable OS cleanup error to fail the test explicitly; it is not acceptable to silently return/panic while claiming cleanup convergence that did not occur. Keep the distinction between “bounded failure” and “successful cleanup” truthful in comments/provenance.
5. Add focused deterministic regression/owner tests sufficient to exercise the helper's live-child deadline behavior and the cleanup postcondition. Do not attempt to synthesize arbitrary kernel `try_wait` failures if doing so would require a framework or unsafe platform trick; source-level ownership shape plus the existing live-child regression may be the appropriate bounded proof. Continue the owner-by-owner sweep after this repair.
6. Final pushed source/test SHA: run `PYTHONDONTWRITEBYTECODE=1 bash scripts/check.sh`, `git diff --check`, confirm clean tree, and persist developer-local exact-tree provenance with exact reachable SHA, UTC start/end, exit codes, OS/arch and stable Rust. No fuzz unless decoder/parser/crypto-framing owners change.

## Evidence boundary

This review is exact-current GitHub source/control-flow reasoning only. The reviewer did not execute Rust tests, the full local gate, cross-platform tests, fuzz, WAN, or performance work in this review. `READY_LIVE` remains `none`; release items 3 and 4 remain incomplete; `RELEASE_CANDIDATE=false`, `PRODUCTION_READY=false`, `FREEZE=false`, and `RELEASED=false` remain unchanged.
