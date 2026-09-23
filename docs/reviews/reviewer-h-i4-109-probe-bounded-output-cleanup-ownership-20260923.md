# H-I4-109 — probe bounded-output helper reintroduces cleanup ownership gap

**Severity:** HIGH — release item 4 / test-harness correctness

**Reviewer source anchor:** `4eadea95f2d5cede3fb6d7518506c0dfef92885d`

## Finding

Developer-owned exact `4eadea95f2d5cede3fb6d7518506c0dfef92885d` makes meaningful progress on H-I4-108: the real networked `client` / `failover-client` owners in `crates/neko-cli/tests/probe.rs` are moved from synchronous `Command::output()` into a spawned child wrapped by `bounded_client_output`, with stdout/stderr drained off-thread and a local deadline.

However, the new `probe.rs::bounded_wait_with_output` copies the pre-H-I4-107 cleanup error handling rather than the exact-current hardened `multistream.rs` helper:

- on deadline it executes `let _ = child.kill()`, discarding a kill error even though child exit is still unproven;
- the post-kill reap loop treats `Err(_)` from `child.try_wait()` as equivalent to `Ok(Some(_))` and breaks, so an observation failure is incorrectly accepted as exit/reap proof;
- the initial `child.try_wait()` error branch repeats the same two mistakes: ignored `kill()` result and post-kill `try_wait Err` treated as successful convergence.

The helper then panics. That is bounded caller failure, but it is not truthful child-ownership convergence: a live child and its pipe readers may remain unresolved on exactly the OS/process-error branches that H-I4-107 already repaired in `crates/neko-cli/tests/multistream.rs`.

This is not a production Session/Carrier/ACK/crypto/wire defect. It is a release/item-4 test-harness correctness defect in the new H-I4-108 ownership primitive.

## Invariant challenged

A bounded product-child helper may report one of three states only:

1. child exit is proven;
2. bounded termination/reap completed and the original timeout/error is then reported;
3. cleanup itself failed, with exit explicitly still unproven.

`kill Err` is not success, and `try_wait Err` is not child-exit proof. Reader joins are only valid after exit/pipe closure has been established.

## Exact-current comparison

The current `crates/neko-cli/tests/multistream.rs::bounded_wait_with_output` already contains the intended narrow shape from H-I4-107: it checks `kill()` errors and treats post-kill `try_wait Err` as explicit cleanup failure rather than a successful reap. `probe.rs` should reuse or faithfully mirror that ownership classification rather than maintaining a weaker copy.

The current H-I4-108 `sleep 30` negative regression proves the ordinary deadline -> successful kill/reap path only. It does not prove the error branches above.

## Required closure

1. Repair `crates/neko-cli/tests/probe.rs::bounded_wait_with_output` so both timeout and initial-`try_wait Err` paths check `kill()` and never treat a later `try_wait Err` as exit/reap proof.
2. Prefer a narrow reuse/refactor of the already-hardened local ownership shape; do not build a generic process framework.
3. Join stdout/stderr drain threads only after child exit / pipe closure is proven. If cleanup itself fails, fail explicitly with ownership still classified as unproven; do not silently claim convergence.
4. Preserve the H-I4-108 conversions of real networked clients and the existing live-client timeout regression. If safely forcing kernel `try_wait`/`kill` failure would require unsafe/platform tricks, source-level ownership proof plus the deterministic live-child regression is sufficient; do not add unsafe or brittle fault injection merely to synthesize an OS error.
5. Re-run/classify remaining `probe.rs` process/socket/thread owners after the helper repair. Do not mechanically convert clearly local fail-fast `.output()` sites.
6. No decoder/parser/crypto-framing implementation change is implicated; do not run fuzz mechanically.
7. On the final pushed source/test SHA run `PYTHONDONTWRITEBYTECODE=1 bash scripts/check.sh`, `git diff --check`, confirm clean tree, and persist developer-local exact-tree provenance with reachable SHA, UTC start/end, exit codes, OS/arch, stable Rust and clean-tree state. One final exact-tree provenance note may cover the unchanged accepted H-I4-107 source plus the H-I4-108/109 repair cluster if it states the exact boundaries truthfully.
8. Continue immediately into the remaining ownership/review queue after closure; do not infer repository-wide exhaustion from this helper repair.

## H-I4-108 relationship

H-I4-108 is **not yet fully closed**. Exact `4eadea95` substantially closes the synchronous-network-client owner family, but the new bounded helper itself violates the required cleanup classification, and no final developer-local exact-tree provenance for this source state is yet persisted on the reviewed head.

H-I4-109 is the concrete helper defect exposed while reviewing that H-I4-108 implementation; fixing it should close the source side of the same cluster without protocol-semantic change.

## Evidence boundary

Reviewer work here is exact-current GitHub source/control-flow review only. No reviewer-local Rust/full-gate run, hosted CI, cross-platform execution, fuzz, WAN experiment, or performance conclusion is claimed. At the reviewed head, GitHub combined status and PR-triggered workflow lookup expose no hosted checks for exact `4eadea95`.
