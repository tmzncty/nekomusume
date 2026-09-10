# Local strict matrix-probe contract closure — `acb81bb`

Developer-run exact-tree validation of reachable implementation/test commit `acb81bb7395d0683c669eac96aaa787dfc0abe66`.

## Operator contract

- `probe --matrix` now uses one matrix-specific sequential parser; every token is consumed exactly once
- `--matrix`, required options, optional value options, and `--json` reject duplicates
- unknown flags, stray positional tokens, and missing values reject before semantic validation, socket work, or artifact output
- malformed numeric values no longer fall back to defaults
- non-loopback target, IP-family mismatch, port zero, timeout outside `1..=5000`, and payload outside `1..=1200` exit `2` with empty stdout
- one held-open silent UDP endpoint supplies a race-free completed-failure fixture: exit `1` and `reachable=false`
- reachable local TCP and UDP cases exit `0`; human pass/fail cats and JSON reachability remain aligned
- the existing v1 `observed_at_unix_ms` field is populated from `SystemTime` when available, and process/unit tests bracket the emitted integer
- top-level help advertises the local-loopback matrix form without creating a second canonical command

Ordinary authenticated `probe` dispatch remains unchanged when `--matrix` is absent. The versioned `payload_bytes` meaning is deliberately unchanged and remains outside this slice.

## Exact-tree gate

- `PYTHONDONTWRITEBYTECODE=1 bash scripts/check.sh`: exit `0`
- `git diff --check`: exit `0`
- gate: `2026-09-10T16:14:07Z` -> `2026-09-10T16:16:03Z`
- host: `Linux 6.8.0-137-generic x86_64`
- Rust: `rustc 1.98.0 (88d9e12ae 2026-08-18)`
- initial/final source tree: clean

This is developer-local operator-contract evidence for bounded loopback behavior. It is not WAN/reachability evidence, independent review, public-listener approval, RC, release, or production authorization.
