# Exact-tree check.sh provenance — f491cdf (2026-09-28 UTC/local)

Developer-local exact-tree gate on the final reachable pushed SHA for the neko-observe
event-payload slice.

- **SHA:** `f491cdfa300a2ac3c5d160159f2e25e13f700a82`
  (`test(observe): pin the RTT, switch-path and scheduler-queue event payloads`)
- **Command:** `PYTHONDONTWRITEBYTECODE=1 bash scripts/check.sh`
- **Tree state:** clean detached worktree (`/tmp/nmObG`) created from the literal
  pushed SHA `f491cdf` (worktree head asserted equal to it); `git diff --check` exit 0;
  `git status --short` empty before and after
- **UTC start:** 2026-09-28T05:39:17Z (log `/tmp/nm-ob-gate.log`, 1257 lines)
- **UTC end:** 2026-09-28T05:45:37Z
- **Exit code:** 0 (host load average ~4.6 from unrelated jobs)
- **OS / arch:** Linux 6.8.0-137-generic, GNU/Linux (x86_64)
- **Rust toolchain:** `rustc 1.98.0 (88d9e12ae 2026-08-18)`

Local and UTC dates are both 2026-09-28.

Passed on the **first** attempt; the load-sensitive
`scripts/bench/run-periodic-command-test.py` step printed
`periodic command tests: ok`.

## Pre-commit sequence (strictly serial)

`cargo fmt --all --check` 0, `clippy --workspace --all-targets -D warnings` 0, then
`cargo test --workspace`: 42 suites, 716 passed, 0 failed
(2026-09-28T05:33:51Z-05:38:03Z, load average ~4.0). Commit, push, remote verification
and the gate were separate tool invocations; `origin/main` was fetch-verified equal to
HEAD before the gate. The gate asserted the **literal** SHA.

## Slice scope

Test-only: **+75 lines**, all inside the `#[cfg(test)]` module of
`crates/neko-observe/src/lib.rs`. The production region (everything before the first
`#[cfg(test)]`) was checked **byte-identical** to HEAD. No new test functions; assertions
were added to five existing ones.

14 hand-written mutants; 0 HANGs, 0 BADANCHOR, 0 COMPILE_ERR, all kills `rc=101`.

- **Baseline (unmodified tests): 14/14 SURVIVED** - the gap was real.
- **After the change: 14/14 killed.**

## How the gap was found

Every JSON key emitted in production was counted against **all** test text - inline
`#[cfg(test)]` modules and `tests/` dirs. That left 27 keys with no witness. Most were
false positives for this purpose (the `failover_timing` keys are asserted by a
`format!` key loop; `reversed`, `classification`, `resumed`, `lost_packets` and others
are asserted in other literal shapes). The neko-observe event payloads were the genuine
gaps: the inline tests reach every one of these events but asserted only the event
**name** plus at most one field.

| Event | Previously asserted | Unwitnessed |
|---|---|---|
| `recovery.rtt_updated` | `latest_rtt_us` | `min_rtt_us`, `smoothed_rtt_us`, `rtt_variance_us`, and which estimator field feeds each |
| `carrier.switch_completed` | name, kind, reason, `path_id` | `to_path_id`; dropping it survived |
| `carrier.switch_failed` | name, severity, outcome, generation | `from_path_id`; dropping it survived |
| `scheduler.dequeued` | priority, `stream_id` | `session_queued_bytes` (sum over all streams) |
| `scheduler.starvation_guard` | name | `session_queued_bytes` |

Each payload is now asserted as its whole `data` object. The completed and failed
switch cases are mirror images (`to_path_id` present with no `from_path_id`, and the
reverse), so both presence and absence are pinned.

## Attempt history (two after-rounds, disclosed)

- **After-round 1: 11 killed / 1 survived.** `RTT_min_src` (feeding `min_rtt_us` from
  `latest_us`) survived because on the first RTT sample min = smoothed = latest = 5000;
  that swap is invisible with equal values.
- **Fix:** a second RTT sample (sent at 5000us, acked at 13000us, i.e. 8000us) makes all
  four values distinct: latest 8000, min 5000, smoothed (7*5000+8000)/8 = **5375**,
  variance (3*2500+|5000-8000|)/4 = **2625**. These values were computed from the
  estimator's update rule before running and matched on the first try. Two more
  source-swap mutants (`smoothed <- latest`, `latest <- smoothed`) were added; both
  **survived the unmodified tests** and are killed now.
- **After-round 2 (recorded here): 14/14 killed.** The test had changed, so round 1's
  evidence was treated as void and the full set was re-run.

## Method note

The neko-observe oracle runs in ~0.3s, so every mutant reported `elapsed=0s`. That
looked like the oracle might not be running at all, so one mutant (`SW_from_drop`) was
applied by hand and rebuilt: `Compiling neko-observe` appeared and all 26 tests ran
against the mutated source and passed. The zero times are just incremental rebuilds.

## Evidence boundary

Developer-local execution only; not reviewer-local, hosted CI, WAN, or performance
evidence. No production source changed. No H-I4-119, D019, release-flag or policy-value
change. `READY_LIVE: none`.
