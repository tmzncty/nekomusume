# Exact-tree check.sh provenance — 4fdacd4 (2026-09-25)

Developer-local exact-tree gate on the final reachable pushed SHA for the
self-directed Track-A slice: uncertain capacity invariant pinned at the
switch/fail boundary.

- **SHA:** `4fdacd41e00701ae5795b167ab0aca83113c86b0`
  (`test(carrier): self-directed Track-A — uncertain capacity invariant
  pinned at the switch/fail boundary`)
- **Command:** `PYTHONDONTWRITEBYTECODE=1 bash scripts/check.sh`
- **Tree state:** clean detached worktree (`/tmp/nmCAP`) at the pushed SHA,
  `git status --short` empty before and after the run; `git diff --check`
  exit 0
- **UTC start:** 2026-09-25T08:17:24Z (log captured in `/tmp/nmcap-check.log`)
- **UTC end:** 2026-09-25T08:23:33Z
- **Exit code:** 0
- **OS / arch:** Linux 6.8.0-137-generic, GNU/Linux (x86_64)
- **Rust toolchain:** `rustc 1.98.0 (88d9e12ae 2026-08-18)` stable
  (`cargo 1.98.0 (797e8a9bc 2026-08-05)`)
- **Scope:** full `scripts/check.sh` — `cargo fmt --check`, `cargo check
  --workspace --locked`, `cargo test --workspace --locked` (36 "test result:
  ok" suites; neko-carrier lib 101/101 including the new
  `uncertain_capacity_conversion_bounded_at_assign_time`), `cargo clippy
  --workspace --all-targets --locked -- -D warnings`, plus all
  governance/status/evidence/preauth/shell/release/reproducibility/smoke
  policy scripts and bounded warm-failover adapter tests. All passed.
- **Slice scope:** test-only. Pins the layered capacity invariant over
  ConcurrentCarrierManager retained ranges: assign() bounds total retained
  ranges/bytes, so activate/fail conversion reaches exactly the bounds but
  never exceeds them via public API; the boundary check tolerates equality
  (strict-exceeds) and is a defensive invariant subsumed by the assign-time
  bound. Exact-boundary conversion at soft-switch and hard-fail, retention
  through NoActive, drain-deadline replay of both ranges, post-release
  capacity reuse, and the Draining path's re-registration gate are asserted.
  A first draft that attempted to reach a conversion rejection through
  public API proved it unreachable (assign rejects first); the commit
  message records this negative result. No implementation change.
