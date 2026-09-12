# Local TCP reset/close regression closure — `1db7a97`

Developer-run exact-tree validation of reachable test-only commit `1db7a9795c37389cf4bc28af2638228f0f58f5fb`.

## Repair

`executable_rejects_unsupported_only_negotiation_before_noise_or_data` no longer assumes every platform reports a rejected TCP close as `Ok(0)`. It now treats `Ok(0)`, `ConnectionReset`, or `BrokenPipe` as the expected immediate close while hard-failing any received application bytes. Unexpected I/O errors remain failures rather than being silently accepted.

Twenty sequential executions of the focused test passed, followed by all six tests in the `multistream` target. This is a cross-platform negative-test determinism repair only; runtime negotiation, Noise, wire, carrier and authentication behavior were not changed.

## Exact-tree gate

- `PYTHONDONTWRITEBYTECODE=1 bash scripts/check.sh`: exit `0`
- `git diff --check`: exit `0`
- gate: `2026-09-10T20:10:29Z` -> `2026-09-10T20:12:23Z`
- host: `Linux 6.8.0-137-generic x86_64`
- Rust: `rustc 1.98.0 (88d9e12ae 2026-08-18)`
- initial/final source tree: clean

This is developer-local test determinism evidence. It is not an independent review, protocol freeze, WAN result, security approval, RC, release or production authorization.
