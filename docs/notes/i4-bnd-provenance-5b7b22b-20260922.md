# I4-BND exact-tree provenance

**Exact pushed source/test SHA:** `5b7b22b34e3e52d4bdde7bcc5a16bcb81f9342e2`
(`test(cli): I4-BND malformed barrier channel bound — one-item handoff`).

`malformed_classification_barrier`'s reader channel changed from
`sync_channel(64)` (unexplained buffer) to `sync_channel(1)` — the
repository's established one-item handoff pattern shared by
`wait_for_ready_marker`. The reader sends each stdout line only when the
barrier is ready to consume it; bounded per-event, no arbitrary capacity.

## Gate record

- worktree: clean `git worktree` at exact `5b7b22b`
- `PYTHONDONTWRITEBYTECODE=1 bash scripts/check.sh` — **exit 0**
- `git diff --check` — **exit 0**
- `git status --short` — clean
- UTC start 2026-09-22T08:55:53Z → UTC end 2026-09-22T09:01:40Z
- OS/arch: Linux x86_64
- Rust: rustc 1.98.0 (88d9e12ae 2026-08-18), stable

This is developer-reported local provenance, not reviewer-local execution or
hosted CI. No decoder/parser/crypto framing change — no fuzz required. No
release flag, D019, or policy value touched. `READY_LIVE: none`.
