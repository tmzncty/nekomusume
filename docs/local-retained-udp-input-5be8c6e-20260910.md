# Local retained-owner UDP input closure — `5be8c6e`

Developer-run exact-tree validation of reachable implementation/test commit `5be8c6e38e2f70a8d6766eb7337e156640a5b565`.

## Accounting and regression evidence

- while the cached first-Noise response owner remains live, every datagram from the owned failover UDP peer is charged for exact input bytes and work before cached-message comparison or AEAD open
- a byte-identical cached first-Noise retry is input-charged exactly once, then exact response-charged and sent through the bounded response helper
- the first non-matching authenticated Data datagram is charged through the same owner before AEAD/classification; accepting it releases the owner and cache exactly once, so later authenticated Data is outside pre-auth ownership
- a focused model regression combines one cached retry with three non-matching datagrams under one four-packet input ceiling; the first excess input fails closed, response cannot reopen the terminal state, and release leaves zero live states
- the existing first-Noise-response-loss process positive records exactly two retained-input charges: one cached retry and one first authenticated Data datagram
- the independent selection-loss fixture no longer injects an unrelated post-Noise datagram that exhausted the deliberately small four-packet pre-auth budget; its pending-selection duplicate/recovery assertions remain intact
- responder inventory remains nine surfaces / seven admission sites with charge ordering before cached classification and AEAD work

## Exact-tree gate

The first exact `e32a113` gate was red because the independent selection-loss fixture combined its selection retry, Noise exchange, late negotiation hello and first Data under the newly truthful four-packet accounting. Exact `5be8c6e` removed only that fixture's unrelated late-hello injection and is the replacement tested tree.

- `PYTHONDONTWRITEBYTECODE=1 bash scripts/check.sh`: exit `0`
- `git diff --check`: exit `0`
- gate: `2026-09-09T21:18:07Z` -> `2026-09-09T21:19:59Z`
- host: `Linux 6.8.0-137-generic x86_64`
- Rust: `rustc 1.98.0 (88d9e12ae 2026-08-18)`
- initial/final source tree: clean

This is developer-local bounded correctness evidence. It does not resolve D019 source-retention policy, establish adversarial-load or production-capacity suitability, constitute independent security review, approve a public listener, or change RC/release/production flags.
