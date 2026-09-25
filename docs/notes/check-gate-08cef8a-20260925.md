# Exact-tree check.sh provenance — 08cef8a (2026-09-25)

Developer-local exact-tree gate on the final reachable pushed SHA for the
explicit-close unification in FaultInjectCarrier.

- **SHA:** `08cef8aa95734a696fcd0e9ef783e3c6bb0ce159`
  (`fix(carrier): explicit close() unifies with close_after visible-close
  semantics`)
- **Command:** `PYTHONDONTWRITEBYTECODE=1 bash scripts/check.sh`
- **Tree state:** clean detached worktree (`/tmp/nmEC`) at the pushed SHA,
  `git status --short` empty before and after the run; `git diff --check`
  exit 0
- **UTC start:** 2026-09-25T11:25:16Z (log captured in `/tmp/nmec-check.log`)
- **UTC end:** 2026-09-25T11:31:22Z
- **Exit code:** 0
- **OS / arch:** Linux 6.8.0-137-generic, GNU/Linux (x86_64)
- **Rust toolchain:** `rustc 1.98.0 (88d9e12ae 2026-08-18)` stable
  (`cargo 1.98.0 (797e8a9bc 2026-08-05)`)
- **Scope:** full `scripts/check.sh` — `cargo fmt --check`, `cargo check
  --workspace --locked`, `cargo test --workspace --locked` (36 "test result:
  ok" suites; neko-carrier lib 106/106 including the new
  `explicit_close_discards_withheld_and_rejects_sends_visibly`), `cargo
  clippy --workspace --all-targets --locked -- -D warnings`, plus all
  governance/status/evidence/preauth/shell/release/reproducibility/smoke
  policy scripts and bounded warm-failover adapter tests. All passed.
- **Slice scope:** fixture-repair (test-support code only, no production
  owner). The explicit Carrier::close() path had the same defects the
  3d2f960 close_after repair fixed for its own trigger: a withheld reorder
  record stayed stranded in the pending slot, and post-close sends either
  silently dropped (non-reorder) or returned a false Ok(()) (reorder mode,
  empty slot — the record buffered on a closed connection). close() now
  sets the closed flag, discards a withheld record with the connection,
  closes the inner carrier, and post-close sends return a visible
  Err(Closed) on every policy shape — the closed check runs before the
  drop-class guards so one_way/blackhole/window/oversized never mask a
  close. Transition-matrix close_after row wording already covers the
  unified semantics (both close paths behave identically).
