# I4-PORT-01 exact-tree provenance

**Exact pushed source/test SHA:** `42be539` (`test(cli): I4-PORT-01 scope SIGTERM/proc lifecycle fixtures to unix`).

## Gate record

- worktree: clean `git worktree` at exact `42be539`
- `PYTHONDONTWRITEBYTECODE=1 bash scripts/check.sh` — **exit 0**
- `git diff --check` — **exit 0**
- `git status --short` — clean
- UTC start 2026-09-21T00:53:58Z → UTC end 2026-09-21T00:59:40Z
- OS/arch: Linux x86_64
- Rust: rustc 1.98.0 (88d9e12ae 2026-08-18), stable

This is developer-reported local provenance, not reviewer-local execution or
hosted CI. No decoder/parser/crypto framing change — no fuzz required. No
release flag, D019, or policy value touched. `READY_LIVE: none`.
