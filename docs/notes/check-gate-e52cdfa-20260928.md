# Exact-tree check.sh provenance — e52cdfa (2026-09-27 UTC / 2026-09-28 local)

Developer-local exact-tree gate on the final reachable pushed SHA for the
`tcp_resource_admitted` boundary-pin slice.

- **SHA:** `e52cdfaa4051ee32ac5446a139afb36ac89bc829`
  (`test(cli): pin the tcp_resource_admitted line in full`)
- **Command:** `PYTHONDONTWRITEBYTECODE=1 bash scripts/check.sh`
- **Tree state:** clean detached worktree (`/tmp/nmRaG`) created from the
  literal pushed SHA `e52cdfa` (worktree head asserted equal to it);
  `git diff --check` exit 0; `git status --short` empty before and after
- **UTC start:** 2026-09-27T22:36:03Z (log `/tmp/nm-ra-gate.log`, 1257 lines)
- **UTC end:** 2026-09-27T22:42:24Z
- **Exit code:** 0 (host load average ~7.2 from unrelated jobs)
- **OS / arch:** Linux 6.8.0-137-generic, GNU/Linux (x86_64)
- **Rust toolchain:** `rustc 1.98.0 (88d9e12ae 2026-08-18)`

File name uses the **local** date (2026-09-28); the run was 2026-09-27 UTC.

Passed on the first attempt; the load-sensitive
`scripts/bench/run-periodic-command-test.py` step printed
`periodic command tests: ok`.

## Pre-commit sequence (strictly serial)

`cargo fmt --all --check` 0, `clippy --workspace --all-targets -D warnings` 0, then
`cargo test --workspace`: 42 suites, 716 passed, 0 failed
(2026-09-27T22:31:28Z-22:35:41Z, load average ~6.0). Commit, push, remote
verification and the gate were separate tool invocations; `origin/main` was
fetch-verified equal to HEAD before the gate. The gate was issued **alone**,
asserting the literal SHA.

## Slice scope

Test-only, **additions only: +9 lines** (0 deletions) in
`crates/neko-cli/tests/probe.rs`. No source change.

10 hand-written mutants; **10 killed, 0 survivors**, 0 HANGs, 0 BADANCHOR, 0
COMPILE_ERR, all `rc=101`.

## The gap

`carrier_event name=tcp_resource_admitted session=7001 target_path=2 generation=1
delivery_epoch=1 final_challenge=3 source=runtime_limits` had **zero** field-level
coverage. Its only reference anywhere in the tree was a **negative** assertion (the
warm-readiness failure test requires it to be ABSENT), so the whole positive line was
free, including all five fields and the `source=` text.

It is emitted by the server unconditionally on the admission path, and the test that
already asserts the adjacent `tcp_warm` line captures that same server log, so the
assertion went into that test with nothing new invoked.

## Checked and NOT gaps

Already fully pinned elsewhere, so deliberately not touched: `tcp_warm` (verbatim at
`probe.rs:6340`, plus a count check at `:6351`), `tcp_resumed` (verbatim match at
`:6660`), `tcp_resume_guard` (verbatim at `:6972`), `udp_health_failed` (verbatim at
`:3088` and `:6303`, with differing `reason` / `diagnostic_cause` values). Only
`tcp_resource_admitted` was label-only.

## Note on a predicted-but-absent survivor

The emitter is a single literal string, so a verbatim assertion pins it completely. I
built a "swap the two fields that both hold `1`" mutant **expecting it to survive**,
but it is killed: what that mutation actually does is **reorder the two `key=value`
tokens**, which changes the output text. Swapping only the VALUES of two keys that
both hold `1` would produce a textually identical line, i.e. it is **not a
constructible mutant at all** (the constructor rejects `old == new`), so there is no
unobservability to report either.

## Evidence boundary

Developer-local execution only; not reviewer-local, hosted CI, WAN, or performance
evidence. No production source changed. No H-I4-119, D019, release-flag or
policy-value change. `READY_LIVE: none`.
