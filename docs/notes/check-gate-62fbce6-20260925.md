# Exact-tree check.sh provenance — 62fbce6 (2026-09-25)

Developer-local exact-tree gate on the final reachable pushed SHA for
R-MBOX-TRANSITION-MATRIX (impairment-class vs tested-boundary reconciliation
document).

- **SHA:** `62fbce61b430fef60997fc73d7ec8ad3f9708b1e`
  (`docs(matrix): R-MBOX-TRANSITION-MATRIX — impairment classes vs tested
  boundaries`)
- **Command:** `PYTHONDONTWRITEBYTECODE=1 bash scripts/check.sh`
- **Tree state:** clean detached worktree (`/tmp/nmTM`) at the pushed SHA,
  `git status --short` empty before and after the run; `git diff --check`
  exit 0
- **UTC start:** 2026-09-25T04:58:19Z (log captured in
  `/tmp/neko-62fbce6-check.log`)
- **UTC end:** 2026-09-25T05:04:25Z
- **Exit code:** 0
- **OS / arch:** Linux 6.8.0-137-generic, GNU/Linux (x86_64)
- **Rust toolchain:** `rustc 1.98.0 (88d9e12ae 2026-08-18)` stable
  (`cargo 1.98.0 (797e8a9bc 2026-08-05)`)
- **Scope:** full `scripts/check.sh` — `cargo fmt --check`, `cargo check
  --workspace --locked`, `cargo test --workspace --locked` (36 "test result:
  ok" suites; neko-carrier lib 98/98), `cargo clippy --workspace
  --all-targets --locked -- -D warnings`, plus all
  governance/status/evidence/preauth/shell/release/reproducibility/smoke
  policy scripts (including markdown-link validation over the new matrix
  note). All passed.
- **Slice scope:** documentation-only reconciliation
  (`docs/notes/r-mbox-transition-matrix-20260925.md`): every
  FaultInjectCarrier primitive mapped to its deterministic behavior, proving
  regression, and explicit unresolved boundary; drop-class isolation and
  transition-ownership expectations each mapped to exact test names;
  explicit not-tested/not-claimed list; no source change, no policy, no
  release flag.
