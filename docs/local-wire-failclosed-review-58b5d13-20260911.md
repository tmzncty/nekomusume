# Local wire fail-closed/allocation-bound review — `58b5d13`

Historical developer review using pre-rebase labels `58b5d13` / `77d3b4a`; those objects are not repository-reachable, so this file is quarantined / not accepted as exact-tree evidence. Reachable `407bfea` introduced the report; current accepted wire navigation uses independent review anchored at reachable `4034f86`.

## Result

No concrete fail-closed/allocation defect was found in the current `neko-wire` API:

- fixed outer header is checked before type/version/flags validation;
- declared record payload is capped at 4096 bytes and decoded as one record only;
- frame payload and frame count are capped before allocation;
- canonical varint rejects truncation, non-minimal encoding and overflow;
- unknown version/type, reserved/critical frame type, trailing bytes and oversized lengths fail closed;
- VersionNegotiator enforces count/order/uniqueness, complete response shape, selected-version support, terminal state and data admission;
- all known unchecked slicing points are protected by earlier explicit length bounds;
- allocations are derived from already-validated bounded lengths, with no attacker-driven unbounded allocation;
- all `neko-wire` targets (18 unit, 3 canonical-vector, 2 deterministic property, 3 compatibility) and clippy passed.

No decoder/parser change was made, so the required pinned decode fuzz was not rerun solely for this review. The existing corpus/fuzz boundary remains separate evidence.

## Verification

- `cargo test -p neko-wire --all-targets`: passed (`26` total)
- `cargo clippy -p neko-wire --all-targets -- -D warnings`: passed
- `PYTHONDONTWRITEBYTECODE=1 bash scripts/check.sh`: exit `0`
- `git diff --check`: exit `0`
- gate: `2026-09-10T20:28:01Z` -> `2026-09-10T20:29:56Z`
- host: `Linux 6.8.0-137-generic x86_64`
- Rust: `rustc 1.98.0 (88d9e12ae 2026-08-18)`
- initial/final source tree: clean

This is a bounded local parser/bounds review, not a wire freeze, proof against every hostile input, independent review, RC, release, public-listener or production authorization.
