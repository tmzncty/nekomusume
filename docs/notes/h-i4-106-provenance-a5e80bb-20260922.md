# H-I4-106 exact-tree provenance

**Exact pushed source/test SHA:** `a5e80bb85948419e20a01430ee8381bf475fa3bd`
(`test(cli): H-I4-106 bounded_wait_with_output for multistream process owners`).

`bounded_wait_with_output` polls `try_wait` against a caller-supplied
deadline; on expiry it kills and poll-reaps so a lifecycle regression
becomes bounded failure instead of an unbounded `wait_with_output`. Child
exit then makes stdout/stderr EOF causally available for the final drain.
Three owners converted: `bounded_tcp_multistream_loopback`,
`unauthorized_client_is_rejected_by_allowlist`,
`executable_rejects_unsupported_only_negotiation_before_noise_or_data`.

## Gate record

- worktree: clean `git worktree` at exact `a5e80bb`
- `PYTHONDONTWRITEBYTECODE=1 bash scripts/check.sh` — **exit 0**
- `git diff --check` — **exit 0**
- `git status --short` — clean
- UTC start 2026-09-22T22:50:56Z → UTC end 2026-09-22T22:56:40Z
- OS/arch: Linux x86_64
- Rust: rustc 1.98.0 (88d9e12ae 2026-08-18), stable

This is developer-reported local provenance, not reviewer-local execution or
hosted CI. No decoder/parser/crypto framing change — no fuzz required. No
release flag, D019, or policy value touched. `READY_LIVE: none`.
