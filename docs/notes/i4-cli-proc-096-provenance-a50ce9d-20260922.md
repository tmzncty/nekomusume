# I4-CLI-PROC-096 exact-tree provenance

**Exact pushed source/test SHA:** `a50ce9d37c24646c5824f3dd738942a3ded16b36`
(`test(cli): I4-CLI-PROC-096 bounded endpoint-rebind readiness wait`).

`ready_endpoint_rebind_server` moved its `read_line` loop off-thread and
bounds the wait on a `recv_timeout(5s)` channel; on timeout or stdout EOF the
child is killed+reaped so the reader gets EOF instead of stranding the test.
The `ready_endpoint_rebind_bounded_when_child_is_silent` negative regression
proves bounded panic on a live-but-silent child, not a hang. The reviewer
also re-challenged the I4 resource-causality chain and found no new defect
(no-finding) at this anchor.

## Gate record

- worktree: clean `git worktree` at exact `a50ce9d`
- `PYTHONDONTWRITEBYTECODE=1 bash scripts/check.sh` — **exit 0**
- `git diff --check` — **exit 0**
- `git status --short` — clean
- UTC start 2026-09-22T06:50:43Z → UTC end 2026-09-22T06:56:40Z
- OS/arch: Linux x86_64
- Rust: rustc 1.98.0 (88d9e12ae 2026-08-18), stable

This is developer-reported local provenance, not reviewer-local execution or
hosted CI. No decoder/parser/crypto framing change — no fuzz required. No
release flag, D019, or policy value touched. `READY_LIVE: none`.
