# Exact-tree check.sh provenance — cdfe943 (2026-09-25)

Developer-local exact-tree gate on the final reachable pushed SHA for the
self-directed Track-A slice: ConcurrentCarrierManager in-flight
path-replacement coverage.

- **SHA:** `cdfe94310634ecc59f5c88b2dcd9704ad196203d`
  (`test(carrier): self-directed Track-A — ConcurrentCarrierManager
  in-flight path-replacement coverage`)
- **Command:** `PYTHONDONTWRITEBYTECODE=1 bash scripts/check.sh`
- **Tree state:** clean detached worktree (`/tmp/nmCC`) at the pushed SHA,
  `git status --short` empty before and after the run; `git diff --check`
  exit 0
- **UTC start:** 2026-09-25T06:10:50Z (log captured in `/tmp/nmcc-check.log`)
- **UTC end:** 2026-09-25T06:16:55Z
- **Exit code:** 0
- **OS / arch:** Linux 6.8.0-137-generic, GNU/Linux (x86_64)
- **Rust toolchain:** `rustc 1.98.0 (88d9e12ae 2026-08-18)` stable
  (`cargo 1.98.0 (797e8a9bc 2026-08-05)`)
- **Scope:** full `scripts/check.sh` — `cargo fmt --check`, `cargo check
  --workspace --locked`, `cargo test --workspace --locked` (36 "test result:
  ok" suites; neko-carrier lib 100/100 including the two new
  `concurrent_tests` regressions:
  `path_replacement_with_in_flight_ranges_drains_and_replays`,
  `hard_failure_retains_uncertain_and_reassigns_to_new_owner`), `cargo
  clippy --workspace --all-targets --locked -- -D warnings`, plus all
  governance/status/evidence/preauth/shell/release/reproducibility/smoke
  policy scripts and bounded warm-failover adapter tests. All passed.
- **Slice scope:** test-only. The ConcurrentCarrierManager surface
  (register/observe_readiness/activate/fail/assign/confirm/replay_uncertain/
  finish_drain) had zero dedicated unit coverage; this pins the research
  plan Track-A scenario "path replacement while Session data is in flight"
  at the in-memory manager seam: unconfirmed-range retention through soft
  replacement (Draining + bounded deadline + DrainPending + exact replay
  with stable id/bytes), hard-failure retention (immediate ownership
  removal, fail-closed NoActive rejections, Cold recovery class, exact
  three-event ordering with epochs 1/1/2). No implementation change.
- **Note:** an earlier gate run on `dc94e2a` (06:04:29Z–06:10:34Z, exit 0)
  was triggered by a race between the commit and the worktree creation and
  therefore gated the pre-slice tree; this run is the authoritative
  provenance for the pushed slice SHA.
