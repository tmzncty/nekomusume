# Exact-tree check.sh provenance — aedb7d3 (2026-09-26)

Developer-local exact-tree gate on the final reachable pushed SHA for the
neko-observe Producer contract/counter/drop-accounting pins.

- **SHA:** `aedb7d3afdece0810fbbc5bda47d1aa4ca4647d0`
  (`test(observe): pin Producer contract vocabulary, counters and drop accounting`)
- **Command:** `PYTHONDONTWRITEBYTECODE=1 bash scripts/check.sh`
- **Tree state:** clean detached worktree (`/tmp/nmOBS`) at the pushed SHA;
  `git diff --check` exit 0; `git status --short` empty before and after
- **UTC start:** 2026-09-25T21:29:13Z (log captured in `/tmp/nmobs-check-run2.log`)
- **UTC end:** 2026-09-25T21:35:21Z
- **Exit code:** 0
- **OS / arch:** Linux 6.8.0-137-generic, GNU/Linux (x86_64)
- **Rust toolchain:** `rustc 1.98.0 (88d9e12ae 2026-08-18)` (`cargo 1.98.0 (797e8a9bc 2026-08-05)`)
- **Scope:** full `scripts/check.sh`. Pre-commit local workspace run: 36
  suites, 484 tests passed, 0 failed; neko-observe lib 19/19.

## Invalid earlier attempt (retained, not evidence)

An earlier invocation (2026-09-25T21:24:45Z–21:28:39Z, EXIT=101) is **not**
gate evidence for any SHA: the worktree was requested at a mistyped SHA
(`3aa6a06`, invalid reference), so `git worktree add` failed and
`check.sh` ran in the **main working tree** (then also at `aedb7d3`) rather
than a clean detached worktree. In that run `neko-cli --test probe` reported
5/74 failures, all loopback-process readiness/timing failures
("child stdout ended before ready", and one stderr wording assertion in
`udp_reply_cessation_seam_is_bounded_and_off_by_default`), while host load
average was ~24 from unrelated concurrent GPU/data jobs. The neko-observe
change does not touch neko-cli or probe code. The subsequent clean exact-tree
run above passed all suites including `probe` (74/74). These probe tests are
recorded here as load-sensitive; no retry-until-green beyond this single
clean rerun was performed.

## Slice scope

Test-only, no semantics change. Mutation probing on `0d28d68` of
`neko-observe` `Producer` (39 mutants; neko-observe tests plus the
neko-carrier integration tests `reliable_udp_observe`/`reliable_udp_runtime`
that consume `Producer`) found 29 survivors. Twelve new tests kill 27
(re-verified per mutant), pinning: capacity range `[1,1024]`
(`docs/era4-observability-contract.md`); health transition only on change
and `Failed` at `error`; the contract switch-reason vocabulary and
carrier_kind mapping; `switch_failed` severity `warn` with from-path
correlation fallback (real `ConcurrentCarrierManager::fail` event);
evidence-gated `recovery.rtt_updated`/`loss_detected`; `pto_total`;
datagram deltas relative to the previous snapshot; exact clamped-delta
`dropped_total` and coalesced diagnostic floor for admitted/dropped/oversize;
scheduler high-water maxima; `stream_id` serialization; every refused event
counted after sequence exhaustion; no diagnostic at sequence `u64::MAX`.

Two equivalent mutants are deliberately not tested:
1. Removing the trailing `flush_drop_report` in `record_datagrams`: every
   clamped remainder is accounted before at least one push of the same kind
   (clamping only occurs when that kind emits `capacity >= 1` events), and
   each push flushes, so the trailing flush always finds zero pending drops.
2. The empty-ring fallback in `oldest_sequence`: its only caller is
   `flush_drop_report` immediately after pushing the diagnostic, so the ring
   is never empty there.

## Evidence boundary

Developer-local execution only; not reviewer-local, hosted CI, WAN, or
performance evidence. No H-I4-119, D019, release-flag or policy-value change.
`READY_LIVE: none`.
