# Local Session overlap-state closure — `1a562ef`

Developer-run exact-tree validation of reachable implementation/test commit `1a562ef61ca7293575e9ae85fa6d758d32c3740e`.

## Defect and selected repair

`DeliveryLedger::insert` previously merged a byte-identical overlap extension into one segment carrying the old segment state. If the old range was `InFlight`, `Uncertain`, or `Confirmed`, novel bytes therefore inherited delivery evidence they had never acquired.

Three small shapes were considered: splitting old and novel ranges at every state boundary, rejecting only advanced-state extensions, or introducing a new composite interval representation. The second is the minimum fail-closed change compatible with the current representation:

- a fully contained byte-identical duplicate remains idempotent and preserves existing state;
- `Unsent` overlap/contiguous extensions continue to merge normally;
- if an overlap operation would introduce any novel byte while the overlapping state is `InFlight`, `Uncertain`, or `Confirmed`, it rejects with `InvalidMigration` before context, segments, bytes, or watermark mutate;
- same-state advanced ranges cannot use an overlapping bridge insertion to promote newly filled middle bytes;
- mixed-state, conflicting-byte, uncovered-gap, limit, and context checks remain unchanged.

Focused coverage includes left and right extensions for all three advanced states, a same-state bridge, contained confirmed duplicate, mixed-state overlap, context rollback, byte count, segment count and watermark atomicity.

No wire, ACK format, cryptographic, Carrier, capacity, or migration-policy semantics changed.

## Exact-tree gate

- `PYTHONDONTWRITEBYTECODE=1 bash scripts/check.sh`: exit `0`
- `git diff --check`: exit `0`
- gate: `2026-09-09T23:11:35Z` -> `2026-09-09T23:13:31Z`
- host: `Linux 6.8.0-137-generic x86_64`
- Rust: `rustc 1.98.0 (88d9e12ae 2026-08-18)`
- initial/final source tree: clean

This is developer-local deterministic Session-state evidence. It is not a protocol freeze, interoperability proof, independent review, public-listener approval, RC, release, or production authorization.
