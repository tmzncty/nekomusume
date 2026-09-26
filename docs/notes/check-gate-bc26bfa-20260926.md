# Exact-tree check.sh provenance — bc26bfa (2026-09-26)

Developer-local exact-tree gate on the final reachable pushed SHA for the
neko-carrier CarrierState mutation-pin slice.

- **SHA:** `bc26bfa8cdd9f2d0bdd898bd40e41d6ab7ffdac6`
  (`test(carrier): pin CarrierState transitions, hysteresis gates and path identity`)
- **Command:** `PYTHONDONTWRITEBYTECODE=1 bash scripts/check.sh`
- **Tree state:** clean detached worktree (`/tmp/nmCSG`) created from
  `git rev-parse origin/main` (asserted equal to local HEAD, to the literal
  pushed SHA, and to worktree HEAD); `git diff --check` exit 0;
  `git status --short` empty before and after
- **UTC start:** 2026-09-26T04:55:32Z (log `/tmp/nmcs-gate.log`)
- **UTC end:** 2026-09-26T05:01:47Z
- **Exit code:** 0 (host load average ~34.7 from unrelated jobs)
- **OS / arch:** Linux 6.8.0-137-generic, GNU/Linux (x86_64)
- **Rust toolchain:** `rustc 1.98.0 (88d9e12ae 2026-08-18)` (`cargo 1.98.0 (797e8a9bc 2026-08-05)`)

## Pre-commit sequence (strictly serial)

fmt --check 0, `clippy --workspace --all-targets -D warnings` 0, then
`cargo test --workspace`: 36 suites, 531 passed, 0 failed
(04:46:59Z-04:55:02Z, load average ~48), no re-run needed. Commit, push,
remote verification and the gate were four separate tool invocations;
`origin/main` was fetch-verified equal to HEAD before the gate was issued.

## Slice scope

Test-only, +5 tests in `crates/neko-carrier/src/lib.rs`
(`mod carrier_state_boundary_tests`), no semantics change. 28 hand-written
mutants over CarrierState::new/snapshot/path/active, `apply` for every
CarrierEvent variant, `tick_dwell` and `checked` were probed with the full
neko-carrier package (16 suites, 181 tests) as the oracle: 27 killed, 1
equivalent. The five new tests kill 15 survivors; CHK_old_le
(`generation.0 < p.generation.0` -> `<=`) is equivalent because `checked`
returns on `generation != p.generation` immediately before it.

The canonical outcomes asserted by the tests were confirmed first in
throwaway probe modules inside the scratch worktree (`/tmp/nmCS`, not
committed). Two witnesses were corrected there: the reorder witness first
ran after a loss assertion that had already degraded the active path, and
a first PTO witness reused that degraded path; both are now checked on a
fresh activation via an `activated()` helper, and the probe was re-run on
the final tree.

## Evidence boundary

Developer-local execution only; not reviewer-local, hosted CI, WAN, or
performance evidence. No production source changed. No H-I4-119, D019,
release-flag or policy-value change. `READY_LIVE: none`.
