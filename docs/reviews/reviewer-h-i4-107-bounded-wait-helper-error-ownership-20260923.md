# H-I4-107 — `bounded_wait_with_output` error exits do not converge child/pipe ownership

**Severity:** HIGH — release/item-4 test-harness correctness

**Reviewed anchor:** `0dfed87fc96f1427e333f8d42dde34f96a31ae2a`

**Developer changes reviewed first:**

- `a5e80bb85948419e20a01430ee8381bf475fa3bd` — replaces the three H-I4-106 server-side `wait_with_output()` owners with a local `bounded_wait_with_output` helper and records an exact-tree developer-local gate in `docs/notes/h-i4-106-provenance-a5e80bb-20260922.md`.
- `f3093d5a6c5ec063d29f08b0eada0a4afee6a538` — adds the deterministic no-client timeout regression for that helper.

The original H-I4-106 direct unbounded `wait_with_output()` call sites are repaired on the normal `try_wait -> Ok(None) -> deadline -> kill succeeds -> reap succeeds` path. This finding is a distinct helper-internal failure-path ownership gap.

No exact-`a5e80bb` hosted workflow/status was visible in this reviewer pass. The persisted gate is developer-local provenance, not reviewer-local execution or hosted CI.

## Challenged invariant

A bounded process-owner helper must remain bounded **and ownership-truthful on every result branch**, not only on the nominal timeout branch. In particular:

1. `try_wait()` error does not prove child exit;
2. `kill()` error does not prove child exit;
3. a later `try_wait()` error does not prove reap;
4. stdout/stderr reader threads may be joined only after child exit / pipe closure is proven, or their own completion is otherwise independently bounded.

A failure branch may report that bounded cleanup itself failed, but it must not silently classify unresolved child/pipe ownership as converged.

## Concrete counterexamples

Exact-current `crates/neko-cli/tests/multistream.rs::bounded_wait_with_output` starts stdout/stderr `read_to_end` threads before observing child exit. Its control flow still has two unresolved ownership exits:

### 1. Initial `try_wait()` error

The outer poll loop has:

```rust
Err(e) => panic!("try_wait failed: {e}"),
```

This panics immediately without attempting bounded terminate/reap. At that point:

- child exit is unproven;
- the child handle is abandoned by the unwinding test path;
- both reader threads may still be blocked in `read_to_end()` on live stdout/stderr pipes;
- their join handles are dropped by unwind before any exit/pipe-closure proof.

This is the same ownership class that H-I4-101/H-I4-102 explicitly required other process helpers to fail closed around; the new multistream helper reintroduces it locally.

### 2. Deadline cleanup treats `try_wait()` error as if reap were complete

On deadline the helper executes:

```rust
let _ = child.kill();
...
match child.try_wait() {
    Ok(Some(_)) | Err(_) => break,
    ...
}
panic!("child did not exit within bound");
```

Two facts are lost:

- `kill()` failure is ignored;
- post-kill `try_wait()` error is grouped with `Ok(Some(_))` and exits the reap loop even though no exit proof exists.

The helper then panics before joining the drain threads. If termination/reap did not actually converge, those readers may remain blocked on live pipes. The helper is wall-clock bounded in the common timeout case, but its ownership claim is not true on these error branches.

The `f3093d5` negative regression exercises a live no-client server reaching the ordinary deadline and succeeding through normal kill/reap. It is valuable for H-I4-106's original hang, but it does not challenge either `try_wait Err` branch or ignored `kill Err`, so it cannot close H-I4-107.

## Impact and boundary

A process API/lifecycle error can make the item-4 harness fail while abandoning an unresolved child and live pipe readers, invalidating the stronger repository claim that bounded process-owner failures converge resources/ownership. This is a **test-harness/release-evidence HIGH**.

It is not a production Session/Carrier/ACK/crypto/wire semantic finding. It creates no new WAN question; `READY_LIVE: none` remains authoritative.

## Closure contract

1. Keep the H-I4-106 normal-path deadline behavior, but make **every** `bounded_wait_with_output` failure branch ownership-truthful.
2. Initial `try_wait Err` must enter bounded best-effort termination/reap before failing; do not immediately panic with a live/unresolved child.
3. Deadline cleanup must not ignore `kill()` result and must not treat post-kill `try_wait Err` as `Ok(Some)`. Distinguish:
   - exit proven;
   - bounded termination/reap attempted and succeeded;
   - bounded cleanup itself failed / exit remains unproven.
4. Do not block on stdout/stderr drain joins unless child exit / pipe closure is proven. If cleanup itself fails and a live child may still own the write ends, fail closed without falsely claiming drain convergence. Prefer the smallest test-local shape; do not build a repository-wide process framework.
5. Add focused deterministic regression coverage for the helper's failure classification where it can be done portably without unsafe/kernel fault injection. If forcing real `try_wait`/`kill` errors would require unsafe or platform tricks, source-level ownership proof plus existing live-child timeout regression is acceptable; do not manufacture a brittle test solely to synthesize impossible OS errors.
6. Preserve the three H-I4-106 converted call sites and their existing protocol/JSON/output assertions. No decoder/parser/crypto-framing implementation change is implicated; fuzz is not mechanically required.
7. On the final pushed source/test SHA run `PYTHONDONTWRITEBYTECODE=1 bash scripts/check.sh`, `git diff --check`, confirm clean tree, and persist developer-local exact-tree provenance with reachable SHA, UTC start/end, exit codes, OS/arch and stable Rust.
8. After closure continue immediately into the remaining process/socket/thread ownership sweep and cross-platform reconciliation. Do not infer repository-wide queue exhaustion from this helper repair.

## Reviewer execution / exclusions

This finding comes from exact-current GitHub source/control-flow review. The reviewer did **not** claim reviewer-local Rust/full-gate execution, cross-platform execution, fuzz, WAN/live evidence, or performance evidence in this pass.
