# Exact-tree check.sh provenance — e2dcbbd (2026-09-27 local / 2026-09-27 UTC)

Developer-local exact-tree gate on the final reachable pushed SHA for the grouped
capability-line boundary-pin slice.

- **SHA:** `e2dcbbd06ceea6ee623d312da9c3355ceb59e86f`
  (`test(cli): pin the whole grouped command line in the human capability report`)
- **Command:** `PYTHONDONTWRITEBYTECODE=1 bash scripts/check.sh`
- **Tree state:** clean detached worktree (`/tmp/nmCgG`) created from the
  literal pushed SHA `e2dcbbd` (worktree head asserted equal to it);
  `git diff --check` exit 0; `git status --short` empty before and after
- **UTC start:** 2026-09-27T12:37:24Z (log `/tmp/nm-cg-gate.log`, 1248 lines)
- **UTC end:** 2026-09-27T12:43:45Z
- **Exit code:** 0 (host load average ~7.5 from unrelated jobs)
- **OS / arch:** Linux 6.8.0-137-generic, GNU/Linux (x86_64)
- **Rust toolchain:** `rustc 1.98.0 (88d9e12ae 2026-08-18)`

Passed on the first attempt; the load-sensitive
`scripts/bench/run-periodic-command-test.py` step printed
`periodic command tests: ok`.

## Pre-commit sequence (strictly serial)

`cargo fmt --all --check` 0, `clippy --workspace --all-targets -D warnings` 0, then
`cargo test --workspace`: 42 suites, 707 passed, 0 failed
(2026-09-27T12:32:47Z-12:36:59Z, load average ~11.0). Commit, push, remote
verification and the gate were separate tool invocations; `origin/main` was
fetch-verified equal to HEAD before the gate. The gate was issued **alone**,
asserting the literal SHA.

## Slice scope

Test-only, **+15/-2** in `crates/neko-cli/tests/capabilities.rs`. The two deleted
lines are the previous partial `contains` assertions on `research=` and
`aliases=`, both subsumed by the single new exact assertion on the whole line, so
coverage strictly increases. No source change.

6 hand-written mutants; **6 killed, 0 survivors, 0 HANGs, 0 BADANCHOR, 0
COMPILE_ERR**.

## The gap, and how it was proven real

The human capability report's grouped command line has four categories, but only
`research=` and `aliases=` were asserted. The `experimental=`, `fixtures=` and
`utilities=` groups - labels, membership and order - had **zero** references in the
test tree.

This was not assumed: the six mutants were built FIRST and run against the
unmodified test suite, where **all six survived**:

- renaming `experimental=` / `fixtures=` / `utilities=` -> survived
- moving `multistream` from `experimental` to `fixtures` -> survived
- **dropping `workload` from the line entirely** -> survived
- swapping the `fixtures` and `utilities` groups wholesale -> survived

After replacing the two partial assertions with one exact whole-line assertion,
all six are killed.

The JSON capability report does pin each command's maturity class (an earlier
slice), but the human grouping is a separate output surface, so that did not cover
this.

## Process note, disclosed

My first scan of production-region output strings returned 566 candidates because it
did not exclude `#[cfg(test)]` blocks - the inline test modules' own assertion
messages were being counted as unreferenced output. Cutting each file at its first
`#[cfg(test)]` is what made the scan usable. Format templates containing `{}` were
also filtered out, because they legitimately appear once even when well covered
(tests assert the INSTANTIATED text, not the template).

## Evidence boundary

Developer-local execution only; not reviewer-local, hosted CI, WAN, or performance
evidence. No production source changed. No H-I4-119, D019, release-flag or
policy-value change. `READY_LIVE: none`.
