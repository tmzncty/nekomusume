# Local Session confirmation-context closure — `6a458ff`

Developer-run exact-tree validation of reachable implementation/test commit `6a458ffa9e133cece2b2dbd4e382948d19df698c`.

## Defect and repair

`DeliveryLedger::confirm_received` could advance an individual segment's key phase or path generation while leaving the ledger-wide context stale. A later insertion using the older key/path context could therefore pass global validation and reintroduce context that the confirmed segment had already advanced beyond.

The repair validates and commits the confirmation context through the existing ledger-wide component-wise monotonic rule before mutating the segment, then commits the same context and `Confirmed` state together. A regression advances confirmation from `(epoch=1,key=0,path=1)` to `(1,1,2)`, proves both global and segment contexts agree, and proves a later `(1,0,1)` insertion fails with `OldEpoch` without changing segment count, bytes or watermark.

No wire, cryptographic, carrier, ACK-evidence, numeric-limit, or migration-policy semantics were introduced.

## Exact-tree gate

- `PYTHONDONTWRITEBYTECODE=1 bash scripts/check.sh`: exit `0`
- `git diff --check`: exit `0`
- gate: `2026-09-09T22:26:40Z` -> `2026-09-09T22:28:36Z`
- host: `Linux 6.8.0-137-generic x86_64`
- Rust: `rustc 1.98.0 (88d9e12ae 2026-08-18)`
- initial/final source tree: clean

This is developer-local deterministic Session-state evidence, not a protocol freeze, independent review, public-listener approval, RC, release or production authorization.
