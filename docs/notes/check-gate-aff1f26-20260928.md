# Exact-tree check.sh provenance — aff1f26 (2026-09-27 UTC / 2026-09-28 local)

Developer-local exact-tree gate on the final reachable pushed SHA for the
carrier-event line boundary-pin slice.

- **SHA:** `aff1f269aa6207de351c11b591f2f3ccdc76913d`
  (`test(cli): pin four carrier_event lines whose fields were still free`)
- **Command:** `PYTHONDONTWRITEBYTECODE=1 bash scripts/check.sh`
- **Tree state:** clean detached worktree (`/tmp/nmCeG`) created from the
  literal pushed SHA `aff1f26` (worktree head asserted equal to it);
  `git diff --check` exit 0; `git status --short` empty before and after
- **UTC start:** 2026-09-27T21:39:20Z (log `/tmp/nm-ce-gate.log`, 1258 lines)
- **UTC end:** 2026-09-27T21:45:41Z
- **Exit code:** 0 (host load average ~4.7 from unrelated jobs)
- **OS / arch:** Linux 6.8.0-137-generic, GNU/Linux (x86_64)
- **Rust toolchain:** `rustc 1.98.0 (88d9e12ae 2026-08-18)`

File name uses the **local** date (2026-09-28); the run was 2026-09-27 UTC.

Passed on the first attempt; the load-sensitive
`scripts/bench/run-periodic-command-test.py` step printed
`periodic command tests: ok`.

## Pre-commit sequence (strictly serial)

`cargo fmt --all --check` 0, `clippy --workspace --all-targets -D warnings` 0, then
`cargo test --workspace`: 42 suites, 716 passed, 0 failed
(2026-09-27T21:34:44Z-21:38:57Z, load average ~4.3). Commit, push, remote
verification and the gate were separate tool invocations; `origin/main` was
fetch-verified equal to HEAD before the gate. The gate was issued **alone**,
asserting the literal SHA.

## Slice scope

Test-only, **additions only: +28 lines** (0 deletions) in
`crates/neko-cli/tests/probe.rs` - assertions added to three existing tests. No
source change.

18 hand-written mutants; **18 killed, 0 survivors**, 0 HANGs, 0 BADANCHOR, 0
COMPILE_ERR, all `rc=101`.

## The gap

Four `carrier_event` lines were covered only at the **label** level, or only
negatively, so every field after the label was free:

- `carrier_event name=udp_negotiated session=7001 generation=0 version=0` - the
  existing assertion pinned only the label and that it occurs exactly once.
- `carrier_event name=udp_recovery_owner_started session=7001 generation=1` - label
  only.
- `carrier_event name=controlled_udp_stop session=7001 generation=0
  reason=bounded_application_fault_injection` - previously asserted only **absent**
  (three places), so the whole positive line was free, including the `reason` text.
- `carrier_event name=ordered_records_complete session=7001 count=3 bytes=16` - the
  label reached only through a `find(...)` and through negative assertions.

Each is now asserted in full inside the test that already reaches it, so nothing new
had to be invoked. `count=3` and `bytes=16` differ, so a count/bytes swap is visible.

## Method notes, disclosed

- These lines are emitted from `failover_server` / `failover_client`, whose tests run
  in 0.6s, 0.6s and 9.9s individually, so a **multi-filter oracle**
  (`cargo test -- <t1> <t2> <t3>`, 11.2s per mutant, verified with
  `running 3 tests`) keeps a 18-mutant round to a few minutes.
- My first attempt to read the instantiated lines inserted dump statements at anchors
  that were **inside multi-line `assert!` argument lists**, which does not compile
  (the failure appeared as a grep hit on the source text echoed by the compiler, not
  as a test failure - a reminder that a grep "hit" in probe output is not evidence by
  itself). The dumps were moved to statement boundaries, and the resulting text is
  what the assertions now use.

## Evidence boundary

Developer-local execution only; not reviewer-local, hosted CI, WAN, or performance
evidence. No production source changed. No H-I4-119, D019, release-flag or
policy-value change. `READY_LIVE: none`.
