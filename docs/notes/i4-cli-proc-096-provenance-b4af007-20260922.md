# I4-CLI-PROC-096 exact-tree provenance

**Exact pushed source/test SHA:** `b4af007df3b5c602e2735eb7bf349de6b4da7dff`
(`test(cli): I4-CLI-PROC-096 bounded readiness — shared wait_for_ready_marker
+ bounded_reap_or_kill`).

`wait_for_ready_marker(child, marker, timeout)` performs the blocking
`read_line` off-thread and bounds the wait on `recv_timeout`; on every
failure shape (timeout, stdout EOF, read error, channel disconnect) the owned
child is killed+reaped via `bounded_reap_or_kill` — no unbounded `wait()` on
a child not proven exited. `start_server_for`, `ready_failover_server` and
`ready_endpoint_rebind_server` all reuse the narrow primitive.

Negative regressions cover: silent-but-live child, binary that exits
silently, and a child that closes stdout while staying alive — each proves
bounded panic, not a hang.

## Gate record

- worktree: clean `git worktree` at exact `b4af007`
- `PYTHONDONTWRITEBYTECODE=1 bash scripts/check.sh` — **exit 0**
- `git diff --check` — **exit 0**
- `git status --short` — clean
- UTC start 2026-09-22T08:00:27Z → UTC end 2026-09-22T08:06:30Z
- OS/arch: Linux x86_64
- Rust: rustc 1.98.0 (88d9e12ae 2026-08-18), stable

This is developer-reported local provenance, not reviewer-local execution or
hosted CI. No decoder/parser/crypto framing change — no fuzz required. No
release flag, D019, or policy value touched. `READY_LIVE: none`.
