# Local process-test determinism closure — `7275062`

Historical developer report for pre-rebase label `7275062`; that object is not repository-reachable, so this file is quarantined / not accepted as exact-tree evidence. Reachable `68e96e7` introduced the report; current accepted navigation uses reachable `4034f86`.

## Review result

The neko-cli process tests were audited for the same OS-dependent close/assertion defect discovered in the rejected-negotiation test. A shared `rejected_negotiation_close_is_either_eof_or_platform_reset` helper now applies the identical contract in all three rejected TCP negotiation paths:

- unsupported selected version;
- one-byte-different negotiation binding;
- unauthorized client.

The tests accept clean `Ok(0)` EOF plus platform-dependent `ConnectionReset`/`BrokenPipe` after rejection, still hard-fail any received bytes, and still fail unexpected I/O errors. No framework was added and no OS-dependent diagnostic is treated as protocol truth.

## Verification

- focused unsupported-version close: passed
- focused binding-mismatch close: passed
- focused unauthorized close: passed
- all six `multistream` tests: passed
- `PYTHONDONTWRITEBYTECODE=1 bash scripts/check.sh`: exit `0`
- `git diff --check`: exit `0`
- gate: `2026-09-10T20:16:24Z` -> `2026-09-10T20:18:20Z`
- host: `Linux 6.8.0-137-generic x86_64`
- Rust: `rustc 1.98.0 (88d9e12ae 2026-08-18)`
- initial/final source tree: clean

This is deterministic negative-test review evidence only, not a runtime/WAN/security/release/production claim.
