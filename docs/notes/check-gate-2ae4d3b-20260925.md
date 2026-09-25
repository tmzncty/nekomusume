# Exact-tree check.sh provenance — 2ae4d3b (2026-09-25)

Developer-local exact-tree gate on the final reachable pushed SHA for the
R-MBOX packet indexing + 13-surface owner-diff reuse check
(documentation-only slice).

- **SHA:** `2ae4d3b592f8bc1c1197665d66edad13de32708a`
  (`docs(packet): R-MBOX group indexed + 13-surface owner-diff reuse
  check`)
- **Command:** `PYTHONDONTWRITEBYTECODE=1 bash scripts/check.sh`
- **Tree state:** clean detached worktree (`/tmp/nmPK`) at the pushed SHA,
  `git status --short` empty before and after the run; `git diff --check`
  exit 0
- **UTC start:** 2026-09-25T05:06:37Z (log captured in
  `/tmp/neko-2ae4d3b-check.log`)
- **UTC end:** 2026-09-25T05:12:43Z
- **Exit code:** 0
- **OS / arch:** Linux 6.8.0-137-generic, GNU/Linux (x86_64)
- **Rust toolchain:** `rustc 1.98.0 (88d9e12ae 2026-08-18)` stable
  (`cargo 1.98.0 (797e8a9bc 2026-08-05)`)
- **Scope:** full `scripts/check.sh` — `cargo fmt --check`, `cargo check
  --workspace --locked`, `cargo test --workspace --locked` (36 "test result:
  ok" suites; neko-carrier lib 98/98), `cargo clippy --workspace
  --all-targets --locked -- -D warnings`, plus all
  governance/status/evidence/preauth/shell/release/reproducibility/smoke
  policy scripts (including release-boundary checks over the refreshed
  packet). All passed.
- **Slice scope:** documentation-only. The release/security packet evidence
  index gained the R-MBOX group row (H-I4-121/122/123 repair chain + six
  R-MBOX slice closures with per-gate provenance links) and the owner-diff
  reuse determination: since reviewer anchor `bb686ce`, only
  `crates/neko-carrier/src/lib.rs` moved, entirely within the test-support
  `FaultInjectCarrier`/`FaultPolicy`/`fault_inject_tests` fixture surface
  (plus docs) — no production owner moved; surfaces 1–12 remain
  CURRENT/REUSE; this row refreshes surface 13. No policy, no release flag;
  item 4 remains incomplete.
