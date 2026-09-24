# Exact-tree check.sh provenance — 2e88bdf (2026-09-24)

Developer-local exact-tree gate on the final reachable pushed SHA after the
R-MBOX-ROOTLESS-LOSS / H-I4-121 repair (post-auth reply blackhole seam
rename + hard-loss bounded-timeout retention + governance-token repair).

- **SHA:** `2e88bdffaaf62b7a543bb880950342c81cf7760f`
  (`fix(carrier): R-MBOX-ROOTLESS-LOSS — post-auth reply blackhole drives
  health failover (H-I4-121)`)
- **Command:** `PYTHONDONTWRITEBYTECODE=1 bash scripts/check.sh`
- **Tree state:** clean detached worktree (`/tmp/nm62dD9LFLS`), branchless
  HEAD equal to `origin/main`; `git status --short` empty before the run;
  `git diff --check` exit 0
- **UTC start:** 2026-09-24T17:39:47Z (log captured in
  `/tmp/neko-2e88bdf-check.log`)
- **UTC end:** 2026-09-24T17:45:31Z
- **Exit code:** 0
- **OS / arch:** Linux 6.8.0-137-generic, GNU/Linux (x86_64)
- **Rust toolchain:** `rustc 1.98.0 (88d9e12ae 2026-08-18)` stable
  (`cargo 1.98.0 (797e8a9bc 2026-08-05)`)
- **Scope:** full `scripts/check.sh` — `cargo fmt --check`, `cargo check
  --workspace --locked`, `cargo test --workspace --locked` (36 "test result:
  ok" suites incl. neko-cli probe 73/73 with the renamed
  `executable_loopback_post_auth_udp_reply_blackhole_drives_tcp_failover`),
  `cargo clippy --workspace --all-targets --locked -- -D warnings`, plus all
  governance/status/evidence/preauth/shell/release/reproducibility/smoke
  policy scripts and bounded warm-failover adapter tests. All passed.
- **Note:** this gate superseded three earlier developer-local attempts on
  the same repair line (62d5914 → test-support defect exposed by the gate;
  295a713 → clippy `drain_collect` on the new retention code; c043b42 →
  governance grep tripped by PR #4's own non-escalation disclaimer token).
  A failing intermediate gate outranking a stale no-finding is the process
  working as designed; each finding was repaired before the final green run.
