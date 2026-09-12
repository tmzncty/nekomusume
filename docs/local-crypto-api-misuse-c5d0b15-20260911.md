# Local crypto API-misuse/security-invariant challenge — `c5d0b15`

Historical developer review using pre-rebase labels `c5d0b15` / `d2bc0ef`; those objects are not repository-reachable, so this file is quarantined / not accepted as exact-tree evidence. Reachable `0774ab6` introduced the report; current accepted navigation uses reachable `4034f86`.

## Challenge result

The current bounded `neko-crypto` implementation resists the required API-misuse classes at the tested boundaries:

- direction-local nonce allocation is monotonic and terminal at `u64` exhaustion; it never wraps or reuses a nonce;
- checked arithmetic/bounds reject overflow rather than truncating;
- direction, delivery epoch, key phase, path generation, stream and RecordContext are independent authenticated domains; one mismatched transcript/context bit fails before session admission;
- replay state advances only after AEAD authentication and context validation; tamper and exact replay collapse to uniform `SessionRejected`;
- trusted-scope authorization rejects wrong/revoked peers;
- synchronized key update resets direction-local nonce and replay state and rejects old-phase records; unsynchronized update fails without changing peer state;
- reliable and bounded unreliable datagrams reject oversize before replay-window mutation;
- `LocalIdentity` deliberately implements no `Debug`, and process diagnostics aggregate without private-key or plaintext fields.

This is a bounded implementation/invariant review, not cryptanalysis, formal proof, independent security audit, persistent restart/rollback replay safety, D019 source-retention resolution, public-listener, RC, release or production approval.

## Verification

Focused tests passed for nonce exhaustion, tamper/replay/context mismatch, exact transcript binding, authorization, synchronized and unsynchronized key update. The complete `neko-crypto` package (`37` tests) and clippy with warnings denied also passed.

Exact-tree gate:

- `PYTHONDONTWRITEBYTECODE=1 bash scripts/check.sh`: exit `0`
- `git diff --check`: exit `0`
- gate: `2026-09-10T20:22:55Z` -> `2026-09-10T20:24:51Z`
- host: `Linux 6.8.0-137-generic x86_64`
- Rust: `rustc 1.98.0 (88d9e12ae 2026-08-18)`
- initial/final source tree: clean
