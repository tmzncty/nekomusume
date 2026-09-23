# H-I4-112 exact-tree provenance

**Exact pushed source/test SHA:** `fc23a7a88c917e116f1140333faa6d8f441ca557`
(`test(cli): H-I4-112 expired_preprogress clients bounded — correct source`).

`expired_preprogress_udp_session_is_retired_before_delivery_and_fresh_
handshake_recovers`'s sequential expired (`--duration 2 → 7s`) and recovery
(`--duration 3 → 8s`) clients now use `bounded_client_output`. This corrects
the `1d727f0` provenance claim that described this pair as already bounded —
the gate record for `1d727f0` was genuine as an execution record, but the
source-coverage claim about the `expired_preprogress` pair was false.

## Gate record

- worktree: clean `git worktree` at exact `fc23a7a`
- `PYTHONDONTWRITEBYTECODE=1 bash scripts/check.sh` — **exit 0**
- `git diff --check` — **exit 0**
- `git status --short` — clean
- UTC start 2026-09-23T07:50:14Z → UTC end 2026-09-23T07:56:00Z
- OS/arch: Linux x86_64
- Rust: rustc 1.98.0 (88d9e12ae 2026-08-18), stable

This is developer-reported local provenance, not reviewer-local execution or
hosted CI. No decoder/parser/crypto framing change — no fuzz required. No
release flag, D019, or policy value touched. `READY_LIVE: none`.
