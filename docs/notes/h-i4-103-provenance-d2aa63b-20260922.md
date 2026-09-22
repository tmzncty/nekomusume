# H-I4-103 exact-tree provenance

**Exact pushed source/test SHA:** `d2aa63b3b02f988a580919f397fabd6db65e4bcf`
(`test(cli): H-I4-103 bounded reader join after barrier success`).

After `malformed_classification_barrier` succeeds, the post-barrier
`reader_handle.join()` now follows `bounded_wait_exit` — child exit makes
stdout EOF so the `read_line` loop returns and the join is EOF-bounded. A
live child with open stdout can no longer hang the join. The
`barrier_success_reader_join_bounded_when_child_lives_and_silent` negative
regression proves bounded join after the barrier count is satisfied.

## Gate record

- worktree: clean `git worktree` at exact `d2aa63b`
- `PYTHONDONTWRITEBYTECODE=1 bash scripts/check.sh` — **exit 0**
- `git diff --check` — **exit 0**
- `git status --short` — clean
- UTC start 2026-09-22T16:50:41Z → UTC end 2026-09-22T16:56:35Z
- OS/arch: Linux x86_64
- Rust: rustc 1.98.0 (88d9e12ae 2026-08-18), stable

This is developer-reported local provenance, not reviewer-local execution or
hosted CI. No decoder/parser/crypto framing change — no fuzz required. No
release flag, D019, or policy value touched. `READY_LIVE: none`.
