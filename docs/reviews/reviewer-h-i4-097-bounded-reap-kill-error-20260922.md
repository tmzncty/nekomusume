# H-I4-097 — bounded process cleanup can still block after cleanup error

**Severity:** HIGH (item-4 / release-evidence process-oracle correctness; not a runtime transport defect)

**Exact reviewed anchor:** `34be59192d6233acd4ee6f1c5387b3dbad8d8abd`

## Scope inspected

- `crates/neko-cli/tests/probe.rs`
  - `bounded_reap_or_kill`
  - `wait_for_ready_marker`
  - `malformed_classification_barrier`
  - the bounded negative process regressions
- `docs/reviews/independent-cross-platform-cli-process-b4af007-20260922.md`
- `docs/reviews/independent-i4-bnd-reconciliation-5b7b22b-20260922.md`
- current item-4/release reconciliation and repository-wide refill at `34be591`

No reviewer-local Rust/full-gate, cross-platform execution, fuzz, WAN or performance run is claimed here; this finding is from exact-current source/control-flow review.

## Concrete counterexample

The current helper is documented as a deterministic bounded reap:

```rust
fn bounded_reap_or_kill(mut child: Child) {
    match child.try_wait() {
        Ok(Some(_)) | Err(_) => {}
        Ok(None) => {
            let _ = child.kill();
        }
    }
    let _ = child.wait();
}
```

Two failure shapes violate the helper's own boundedness claim:

1. `child.try_wait()` returns `Err(_)`. The helper has no proof that the child exited, but it still falls through to blocking `child.wait()`.
2. `child.try_wait()` returns `Ok(None)`, `child.kill()` returns `Err(_)` while the child may remain live, and the ignored kill result is followed by blocking `child.wait()`.

Both are ordinary API-result paths in the source model. Existing silent/live negative regressions exercise the normal successful-kill path, so they do not falsify either cleanup-error path.

`malformed_classification_barrier` duplicates the same shape in its timeout, EOF and channel-error exits:

```rust
let _ = child.kill();
let _ = child.wait();
let _ = reader_handle.join();
```

If termination fails and the child remains live, `wait()` can block past the advertised deadline; the reader may also remain blocked on the child's stdout, so the subsequent `join()` is not independently bounded.

## Why the current no-finding/queue-exhausted state is not valid

`49925ac` states that process ownership converges on every failure shape and that there is no unbounded `wait()` on an unproven-exited child. `d814457` likewise closes I4-BND after claiming timeout/EOF/disconnect cleanup is bounded. The exact-current code above contains a counterexample to those claims. Therefore `34be591`'s repository-wide `queue exhausted` classification cannot remain authoritative until this HIGH is closed and the dependent item-4 facts are reconciled.

This does **not** prove a production/runtime resource leak and does not reopen Session/Carrier/ACK/crypto/wire semantics. It is a deterministic test/process-cleanup evidence defect.

## Closure contract

Use the smallest process-test repair that preserves current runtime semantics:

1. No blocking `wait()` may occur unless exit is already proven (`try_wait -> Some`) or termination has succeeded and reaping itself is bounded by an explicit local deadline/poll.
2. `try_wait` failure and `kill` failure must be explicit fail-closed cleanup errors; they must not fall through to an unbounded wait.
3. `wait_for_ready_marker` and every `malformed_classification_barrier` failure exit must share or otherwise obey the same bounded cleanup rule.
4. Do not `join()` a reader that can still be blocked on a live child's stdout after cleanup failure. The caller must return/fail within the process-test bound and preserve the cleanup failure as evidence rather than hang.
5. Add the narrowest deterministic regression(s) practical for the revised helper/control flow. Do not build a general process framework merely to manufacture a synthetic OS failure.
6. On the final pushed source/test SHA run `PYTHONDONTWRITEBYTECODE=1 bash scripts/check.sh`, `git diff --check`, verify clean tree, and persist exact-tree developer-local provenance (SHA, UTC start/end, exit codes, OS/arch, stable Rust).
7. No fuzz is required unless decoder/parser/crypto-framing owners change.

Do not invent a new timeout/capacity/security policy value. A short test-local cleanup deadline may reuse an existing process-test bound or another already-committed test-local bound; if a new security/capacity policy value would be required, stop and classify that separately.

## After repair

Immediately re-challenge the process-cleanup causal surface, then reconcile the cross-platform/I4-BND no-finding notes and release packet before any new queue-exhaustion claim. `READY_LIVE` remains `none` unless a new code/instrumentation/hypothesis/path condition creates a concrete unresolved real-network question.