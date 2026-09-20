# H-I4-086 exact-tree provenance

**Exact pushed source/test SHA:** `3acba049fee5f9297cc8a4c3867250635c255717`
(`test(session): H-I4-086 resumed_session drain-then-ACK ordering + terminal sent cleanup`).

The substantive repair is at `ca629d7` (`fix(session): H-I4-086 DeliveryAck
must not confirm queued-but-undrained bytes`); `3acba04` is the companion
test-ordering + terminal-cleanup commit.

## Gate record

- worktree: clean `git worktree` at exact `3acba04`
- `PYTHONDONTWRITEBYTECODE=1 bash scripts/check.sh` — **exit 0**
- `git diff --check` — **exit 0**
- `git status --short` — clean
- UTC start 2026-09-20T21:05:25Z → UTC end 2026-09-20T21:11:30Z
- OS/arch: Linux x86_64
- Rust: rustc 1.98.0 (88d9e12ae 2026-08-18), stable

This is developer-reported local provenance, not reviewer-local execution or
hosted CI. No decoder/parser/crypto framing change — no fuzz required. No
release flag, D019, or policy value touched. `READY_LIVE: none`.
