# Local rejected-negotiation TCP close regression — `dc52a5e`

Developer-local clean exact-tree validation of reachable test repair `dc52a5e38ea3cc1d113b00a1118a146bebbfd91a`.

## Repair and boundary

The prior attempted portability change accidentally created a parameterized `#[test]` that could not compile and left the real unsupported-negotiation read assertion unchanged. Exact `dc52a5e` converts it to a normal helper, calls it from the real process test, and accepts only the platform-equivalent terminal outcomes `Ok(0)` or `ErrorKind::ConnectionReset`. Emitted bytes and all other I/O errors remain failures. Runtime negotiation behavior is unchanged.

## Verification

On a clean detached checkout of exact `dc52a5e`:

- the focused unsupported-negotiation process test passed 20/20;
- the full `neko-cli` multistream target passed;
- `PYTHONDONTWRITEBYTECODE=1 bash scripts/check.sh`: exit `0`;
- `git diff --check`: exit `0`;
- UTC `2026-09-12T11:00:05Z -> 11:02:09Z`;
- Linux x86_64; Rust `1.98.0`;
- initial/final source tree clean.

GitHub Actions run `34689870833` also passed for exact `dc52a5e` (`stable checks` and nightly decode fuzz smoke both green). The preceding exact `89f26b5` stable job failed because the malformed parameterized test did not compile; that red result is not erased by this replacement evidence.

This is a test-semantics repair and validation record, not a runtime feature, protocol change, security audit, release approval, or production evidence.
