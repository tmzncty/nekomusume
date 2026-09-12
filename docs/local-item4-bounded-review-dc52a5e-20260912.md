# Consolidated bounded item-4 local review — `dc52a5e`

Historical developer consolidated review of reachable source tree `dc52a5e`. Its claimed evidence-note tree `eb5f0e5` is a pre-rebase label that is not repository-reachable, so that exact-tree gate claim is **quarantined / not accepted**. Reachable commit `c0b2ee0` introduced this report; current accepted review navigation uses reachable `4034f86` and later evidence.

## Bounded no-finding review

Five current item-4-adjacent surfaces were re-reviewed together on this reachable anchor:

- **Process determinism:** the rejected unsupported-negotiation test ran 20 consecutive times and all six `multistream` tests passed; clean EOF and platform reset after rejection remain the accepted local behavior while any received bytes still fail.
- **Crypto API misuse:** all 37 `neko-crypto` tests and clippy passed, covering nonce uniqueness/exhaustion, overflow/bounds, direction/context/transcript binding, replay, authorization, synchronized/unsynchronized key update and secret-safe output; `LocalIdentity` still exposes no Debug.
- **Wire fail-closed/allocation bounds:** all 26 `neko-wire` targets and clippy passed; outer/inner length/count checks precede bounded allocation, malformed/truncated/unknown/trailing input fails closed, and no panic/unbounded-allocation contradiction was found.
- **CLI secret/admission:** the nine responder surfaces across seven admission sites remain inventoried; no private key, identity contents, Noise transcript, plaintext or arbitrary peer text is emitted; deterministic invalid config does not create identity or premature READY/success; malformed distinct-source UDP churn recovers into one authenticated exchange.
- **Session/Carrier evidence domains:** assignment-time context, ledger-global duplicates, advanced novel-byte overlap, explicit `confirm_received`, packet feedback versus Session delivery, TCP reliability versus `DeliveryAck`, and failover replay/dedup remain consistent; no concrete contradiction was found.

Focused checks included ordinary authenticated TCP/UDP probes, deterministic absent-identity rejection, malformed distinct-source recovery, and pre-auth responder inventory (`9 surfaces`, `7 admission sites`). No code change was required.

## Exact-tree gate

- `PYTHONDONTWRITEBYTECODE=1 bash scripts/check.sh`: exit `0`
- `git diff --check`: exit `0`
- gate: `2026-09-12T11:02:33Z` -> `2026-09-12T11:04:27Z`
- host: `Linux 6.8.0-137-generic x86_64`
- Rust: `rustc 1.98.0 (88d9e12ae 2026-08-18)`
- initial/final source tree: clean

This is developer-local bounded challenge evidence only. It is not independent security review, cryptanalysis, adversarial-load/public capacity suitability, persistent restart/rollback replay safety, D019 source-retention resolution, release item 3 or 4 completion, RC, release, public-listener or production authorization.
