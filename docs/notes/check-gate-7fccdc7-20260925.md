# Exact-tree check.sh provenance — 7fccdc7 (2026-09-25)

Developer-local exact-tree gate on the final reachable pushed SHA for the
D057 generated_fault_sequence contract pin.

- **SHA:** `7fccdc7687679f90c94dda90a19a19891d849cc8`
  (`test(carrier): pin the D057 generated_fault_sequence contract`)
- **Command:** `PYTHONDONTWRITEBYTECODE=1 bash scripts/check.sh`
- **Tree state:** clean detached worktree (`/tmp/nmD57`) at the pushed SHA,
  `git status --short` empty before and after the run; `git diff --check`
  exit 0
- **UTC start:** 2026-09-25T12:34:33Z (log captured in `/tmp/nmd57-check.log`)
- **UTC end:** 2026-09-25T12:40:39Z
- **Exit code:** 0
- **OS / arch:** Linux 6.8.0-137-generic, GNU/Linux (x86_64)
- **Rust toolchain:** `rustc 1.98.0 (88d9e12ae 2026-08-18)` stable
  (`cargo 1.98.0 (797e8a9bc 2026-08-05)`)
- **Scope:** full `scripts/check.sh` — `cargo fmt --check`, `cargo check
  --workspace --locked`, `cargo test --workspace --locked` (36 "test result:
  ok" suites; neko-carrier lib 107/107 including the new
  `generated_fault_sequence_is_deterministic_and_bounded`), `cargo clippy
  --workspace --all-targets --locked -- -D warnings`, plus all
  governance/status/evidence/preauth/shell/release/reproducibility/smoke
  policy scripts and bounded warm-failover adapter tests. All passed.
- **Slice scope:** test-only. The D057 decision record (2026-08-29,
  candidate state-machine tooling) declared three core properties of the
  seeded fault-event generator with zero test coverage: determinism, the
  4096-event cap, and the closed eleven-event kind set. All three pinned:
  same-seed exact reproduction, cross-seed divergence, cap clamping at
  4096 (at/just-below boundaries), empty input, closed-kind assertion.
  Any silent change to the LCG constants, event order, or cap now fails a
  named regression instead of invalidating downstream inputs silently.
