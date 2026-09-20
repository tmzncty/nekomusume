# R9-12 final exact-tree developer-local provenance

**Exact pushed developer source/test SHA:** `8cbd9af39a600f4ddd5fbc50208791e8584c6d74`
(`fix(bench): H-R9-084 sampler exit status follows terminal cleanup truth`).

Later commits on `main` are docs-only review/reconciliation notes and do not
change the exercised source/test tree.

## Gate record

- worktree: clean `git worktree` at exact `8cbd9af`
- `PYTHONDONTWRITEBYTECODE=1 bash scripts/check.sh` — **exit 0**
- `git diff --check` — **exit 0**
- `git status --short` — clean
- UTC start 2026-09-20T15:55:57Z → UTC end 2026-09-20T16:01:40Z
- OS/arch: Linux x86_64
- Rust: rustc 1.98.0 (88d9e12ae 2026-08-18), stable

This is developer-reported local provenance, not reviewer-local execution and
not hosted CI. No decode/framing change occurred in the R9-11 repair group —
pinned decode fuzz was not mechanically re-run. No release flag, D019, or
policy value was touched. `READY_LIVE: none`.
