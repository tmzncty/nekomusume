# Exact-tree check.sh provenance — 00f0fce (2026-09-27 UTC / 2026-09-28 local)

Developer-local exact-tree gate on the final reachable pushed SHA for the TCP
carrier-event boundary-pin slice.

- **SHA:** `00f0fce0a108dc8b02732e08927f8a68b3862ff6`
  (`test(cli): pin the TCP carrier-event lines and their order`)
- **Command:** `PYTHONDONTWRITEBYTECODE=1 bash scripts/check.sh`
- **Tree state:** clean detached worktree (`/tmp/nmTnG`) created from the
  literal pushed SHA `00f0fce` (worktree head asserted equal to it);
  `git diff --check` exit 0; `git status --short` empty before and after
- **UTC start:** 2026-09-27T16:10:12Z (log `/tmp/nm-tn-gate.log`, 1248 lines)
- **UTC end:** 2026-09-27T16:16:33Z
- **Exit code:** 0 (host load average ~6.7 from unrelated jobs)
- **OS / arch:** Linux 6.8.0-137-generic, GNU/Linux (x86_64)
- **Rust toolchain:** `rustc 1.98.0 (88d9e12ae 2026-08-18)`

File name uses the **local** date (2026-09-28); the run was 2026-09-27 UTC.

Passed on the first attempt; the load-sensitive
`scripts/bench/run-periodic-command-test.py` step printed
`periodic command tests: ok`.

## Pre-commit sequence (strictly serial)

`cargo fmt --all --check` 0, `clippy --workspace --all-targets -D warnings` 0, then
`cargo test --workspace`: 42 suites, 707 passed, 0 failed
(2026-09-27T16:05:37Z-16:09:50Z, load average ~6.8). Commit, push, remote
verification and the gate were separate tool invocations; `origin/main` was
fetch-verified equal to HEAD before the gate. The gate was issued **alone**,
asserting the literal SHA.

## Slice scope

Test-only, **additions only: +35 lines** (0 deletions) in
`crates/neko-cli/tests/probe.rs` - assertions added to one existing test. No source
change.

14 hand-written mutants; **14 killed, 0 survivors** (after the fix below),
0 HANGs, 0 BADANCHOR, 0 COMPILE_ERR, all `rc=101`.

## The gap

Three server-side TCP carrier-event lines had no line-level witness:

- `carrier_event name=tcp_negotiated session=7001 generation=1 version=0` - **zero**
  occurrences in the test tree.
- `carrier_event name=tcp_authenticated session=7001 generation=1` - **zero**
  occurrences; `tcp_authenticated` appeared nowhere at all.
- `carrier_event name=tcp_resume_validated session=7001 generation=1` - the label was
  reached only through a `find(...)` used for ORDERING in a different test, so its
  `session` and `generation` fields were unwitnessed.

All three are printed unconditionally (not behind `if json_mode`) on the existing
`executable_loopback_health_threshold_drives_udp_to_tcp` path, so nothing new had to
be invoked - only asserted.

## Two process findings, both disclosed

**1. I first edited the WRONG test.** I anchored on the `bytes_hex` assertion, assumed
it belonged to `executable_loopback_post_auth_delivery_ack_blackhole_...` (the test
whose name I had in mind), and inserted there. It actually belongs to
`executable_loopback_health_threshold_drives_udp_to_tcp`. Running the former passed
and proved nothing. A `panic!` canary at the insertion point settled it: it did NOT
fire under the test I had been running (so the code was never reached) and DID fire
under the correct test. **Without the canary I would have reported a passing
assertion that never executed.** The insertion was moved to the correct test.

**2. The probe then caught a gap in my own fix.** `TN_auth_res_swap` - exchanging the
two adjacent lines `tcp_authenticated` and `tcp_resume_validated` - SURVIVED, because
independent `contains` assertions cannot see a reordering. The order is semantically
real (authentication precedes resume validation), so the test now also asserts
`negotiated < authenticated < resume_validated` by index. The re-run is 14 killed /
0 survivors.

## Evidence boundary

Developer-local execution only; not reviewer-local, hosted CI, WAN, or performance
evidence. No production source changed. No H-I4-119, D019, release-flag or
policy-value change. `READY_LIVE: none`.
