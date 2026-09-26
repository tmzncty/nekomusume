# Exact-tree check.sh provenance — eb81ae0 (2026-09-26)

Developer-local exact-tree gate on the final reachable pushed SHA for the
neko-carrier `PathRecovery` boundary-pin slice.

- **SHA:** `eb81ae0793f4aca6b52f422fab72216c255cd525`
  (`test(carrier): pin PathRecovery identity, abandon rollback, health freshness and pto clamp`)
- **Command:** `PYTHONDONTWRITEBYTECODE=1 bash scripts/check.sh`
- **Tree state:** clean detached worktree (`/tmp/nmPRG`) created from
  `git rev-parse origin/main` (asserted equal to local HEAD, to the literal
  pushed SHA, and to worktree HEAD); `git diff --check` exit 0;
  `git status --short` empty before and after
- **UTC start:** 2026-09-26T06:29:56Z (log `/tmp/nmpr-gate.log`)
- **UTC end:** 2026-09-26T06:36:02Z
- **Exit code:** 0 (host load average ~21.6 from unrelated jobs)
- **OS / arch:** Linux 6.8.0-137-generic, GNU/Linux (x86_64)
- **Rust toolchain:** `rustc 1.98.0 (88d9e12ae 2026-08-18)` (`cargo 1.98.0 (797e8a9bc 2026-08-05)`)

## Pre-commit sequence (strictly serial)

fmt --check 0, `clippy --workspace --all-targets -D warnings` 0, then
`cargo test --workspace`: 36 suites, 549 passed, 0 failed
(06:21:30Z-06:29:28Z, load average ~16.8), no re-run needed. Commit, push,
remote verification and the gate were four separate tool invocations;
`origin/main` was fetch-verified equal to HEAD before the gate was issued.

## Slice scope

Test-only, +130 lines in `crates/neko-carrier/src/lib.rs` (six tests added
inside the existing `mod path_recovery_tests`), no semantics change. 44
hand-written mutants over PathRecovery::new, on_sent, abandon_sent, on_ack,
on_pto, quiesce, can_send, pacing_interval_us, frame_outstanding, recovery,
the accessors, fresh_health_sample, diagnostics and persistent_congestion
were probed with the full neko-carrier package (16 suites, 187 tests) as
the oracle: 41 killed, 3 equivalent. The six new tests kill 11 survivors;
the equivalence reasons for the other three are in the commit message.

The exact values asserted (29_000us RTT, 833/mille interval, the
u16::MAX/u16::MAX+1 pto clamp pair, the loss set [8,9,10,11,12] for an acked
packet 15) were measured on canonical code first in a throwaway probe
module inside the scratch worktree (`/tmp/nmPR`, not committed); the
u16::MAX-PTO loop there runs in about 17ms, so the clamp boundary is
cheaply reachable.

One witness was corrected: the first quiesce witness asserted that a clean
post-quiesce outcome reports 0/mille, which holds for any sent baseline
because a clean interval has zero lost packets - it therefore masked the
mutant. It now resolves a second lossy interval and asserts 833/mille.

## Evidence boundary

Developer-local execution only; not reviewer-local, hosted CI, WAN, or
performance evidence. No production source changed. No H-I4-119, D019,
release-flag or policy-value change. `READY_LIVE: none`.
