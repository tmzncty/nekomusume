# Local HY2 blocked-artifact schema closure — `aca2842`

Developer-local clean exact-tree validation of reachable implementation/test commit `aca2842bed699448b19b0d1ef19b3acaf3bcecdf`.

## Change and boundary

`BLOCKED_HARNESS` artifacts now require exactly the canonical nine top-level keys, including `contract`. Noncanonical comparative-looking fields such as `median_exchange_latency_ms`, `p95_latency_seconds`, `superiority`, and `comparative_summary` reject. Existing retained blocked artifacts remain valid and required no rewrite.

This closes only the hand-authored blocked-artifact key-exclusivity defect found by the bounded independent HY2 methodology review. No live HY2 run occurred; there is still no complete pair, median/P95 comparison, performance result, or superiority claim.

## Verification

On a clean detached checkout of exact `aca2842`:

- `python3 scripts/bench/validate-hy2-owned-lab-test.py`: exit `0`;
- `bash scripts/bench/compare-hy2-owned-lab-test.sh`: exit `0`;
- `bash scripts/bench/compare-hy2-test.sh`: exit `0`;
- `validate-hy2-owned-lab.py validate-result` for every retained `artifacts/hy2-owned-lab/*/result.json`: exit `0`;
- `PYTHONDONTWRITEBYTECODE=1 bash scripts/check.sh`: exit `0`;
- `git diff --check`: exit `0`;
- UTC full-gate interval `2026-09-10T22:13:24Z -> 22:15:20Z`;
- Linux x86_64; Rust `1.98.0`;
- initial and final source tree clean.

This is developer-local evidence, not reviewer-executed CI, a security audit, release approval, or production evidence.
