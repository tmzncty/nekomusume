# Exact-tree check.sh provenance — ec911db (2026-09-26)

Developer-local exact-tree gate on the final reachable pushed SHA for the
in-process Memory carrier boundary-pin slice (neko-carrier).

- **SHA:** `ec911dbcd34a21f74589006e07749d0d2c426a45`
  (`test(carrier): pin in-process Memory carrier equal limits and byte-budget accounting`)
- **Command:** `PYTHONDONTWRITEBYTECODE=1 bash scripts/check.sh`
- **Tree state:** clean detached worktree (`/tmp/nmMG`) created from the
  literal pushed SHA `ec911db` (worktree head asserted equal to it);
  `git diff --check` exit 0; `git status --short` empty before and after
- **UTC start:** 2026-09-26T09:37:05Z (log `/tmp/nm-mem-gate.log`, 1089 lines)
- **UTC end:** 2026-09-26T09:43:12Z
- **Exit code:** 0 (host load average ~13.8 from unrelated jobs)
- **OS / arch:** Linux 6.8.0-137-generic, GNU/Linux (x86_64)
- **Rust toolchain:** `rustc 1.98.0 (88d9e12ae 2026-08-18)`

## Pre-commit sequence (strictly serial)

`cargo fmt --all --check` 0, `clippy --workspace --all-targets -D warnings` 0,
then `cargo test --workspace`: 36 suites, 585 passed, 0 failed
(09:32:40Z-09:36:40Z, load average ~14.4). Commit, push, remote verification and
the gate were separate tool invocations; `origin/main` was fetch-verified equal
to HEAD before the gate. The gate was issued **alone**, asserting the literal
SHA.

## Slice scope

Test-only, +57 lines in `crates/neko-carrier/src/lib.rs` (two tests added to the
existing `mod memory_pair_tests`), no semantics change.

27 hand-written mutants over `MemoryPair::new` and the `Carrier` impl for
`MemoryEndpoint` (kind/properties/limits/send/recv/close); **23 killed, 4
equivalent, 0 invalid constructions, 0 BADANCHOR**. The pre-existing seven
memory_pair_tests left 5 survivors; the two new tests close the one that was a
real gap (`max_message_bytes > max_queue_bytes` shifted to `>=`, which would
reject the legal equal-limits configuration).

Anchor method for this file: ~9k lines with many similar adapter methods, so
every mutant anchors on a whole function body (verified unique in the file) and
edits exactly one fragment inside it.

Oracle: `cargo test -q -p neko-carrier`. The pre-run baseline was measured
green in a **separate worktree** (`/tmp/nmMb`), because the probe worktree's
source is mutated during the run - running a baseline in the probe tree would
have raced the probe.

The four equivalences (argued from source, not from a green run):

- `N_cond2_off`: `max_queue_bytes == 0` is redundant against the other two
  clauses - with max_queue_bytes == 0, either max_message_bytes == 0 (first
  clause) or max_message_bytes > 0 == max_queue_bytes (third clause).
- `S_queue_bytes_set`: reassigning `queue_bytes[peer] = total` where total is
  `queue_bytes[peer].checked_add(message.len())` is the same value as
  `queue_bytes[peer] += message.len()`, by construction.
- `S_add_wrapping`: `checked_add` -> `wrapping_add` can only differ on overflow,
  which needs max_queue_bytes > usize::MAX/2 - unreachable for any state a test
  can build.
- `R_sub_sat`: `checked_sub` -> `saturating_sub` on recv can only differ if
  queue_bytes[side] < bytes.len(), which the exported invariant
  queue_bytes == sum of queued lengths forbids; the new byte-budget test
  exercises that invariant at several points.

One in-slice correction: the first version of the byte-budget test sent a
4-byte message to prove the budget had been restored under a 2-byte message
limit, which correctly failed with MessageTooLarge on unmutated code; it now
proves the restored budget with two maximum-size messages instead.

## Evidence fidelity

The recorded probe run was made against content byte-identical to the committed
file: `cargo fmt` ran before the pre-probe snapshot, and the snapshot diffed
clean against the committed file afterwards.

## Evidence boundary

Developer-local execution only; not reviewer-local, hosted CI, WAN, or
performance evidence. No production source changed. No H-I4-119, D019,
release-flag or policy-value change. `READY_LIVE: none`.
