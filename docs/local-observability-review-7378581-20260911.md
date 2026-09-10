# Local observability/timeline bounded review — `7378581`

Developer-run bounded contract/evidence review and exact-tree validation of reachable commit `7378581ef19f3c27ab45ea7d400f92b583756c1d`.

## Result

The stable v1 observability schemas and mutation tests pass. Focused runtime tests prove monotonic, stable Session event projection, bounded deterministic health evidence, and checked/bounded recovery latency. Current failover process regressions retain ordered structured events and explicit decision/connect/negotiation/authentication/resume/active/first-data/ACK timeline fields. Retained negative artifacts continue to distinguish missing/unavailable evidence from runtime failure rather than inventing zero metrics.

One concrete governance contradiction was repaired: the dated VPS soak plan still claimed that no reusable authorization existed, while the later repository standing authorization is explicitly continuous. The plan now says standing authorization supersedes that anchor-time statement only within its bounded self-owned limits, while authorization alone neither selects a task nor creates a `READY_LIVE` row. Exact-current remains `READY_LIVE: none`; no experiment was run.

No other actually missing advertised bounded metric was found in this pass. The wider stable vocabulary expressly does not claim every producer is wired. Existing negative current lines, absent IPv6 environment, live-PMTUD implementation block, lack of reliability-rate evidence, and release/security gates remain unchanged.

## Verification

Focused:

- `bash scripts/check-observability-contract.sh`: passed
- `bash scripts/check-observability-contract-test.sh`: passed
- Session observable-event monotonic JSON test: passed
- Carrier recovery-latency bounded test: passed
- Carrier bounded deterministic health-evidence test: passed
- `git diff --check` and Markdown link check: passed

Exact-tree:

- `PYTHONDONTWRITEBYTECODE=1 bash scripts/check.sh`: exit `0`
- `git diff --check`: exit `0`
- gate: `2026-09-10T18:36:09Z` -> `2026-09-10T18:38:05Z`
- host: `Linux 6.8.0-137-generic x86_64`
- Rust: `rustc 1.98.0 (88d9e12ae 2026-08-18)`
- initial/final source tree: clean

This closes only the bounded current L-row observability/timeline question. It is not complete producer coverage, a WAN result, reliability proof, audit, RC, release, or production authorization.
