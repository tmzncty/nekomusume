# Local Session/Carrier evidence-domain review — `1be290d`

Historical developer review using pre-rebase labels `1be290d` / `0b87f23`; those objects are not repository-reachable, so this file is quarantined / not accepted as exact-tree evidence. Reachable `d0d2f42` introduced the report; current accepted navigation uses reachable `4034f86`.

## Result

The current bounded contract remains internally consistent across the previously repaired evidence domains:

- `confirm_received` is the only transition that advances Session delivery context, segment state and watermark; rejected confirmation leaves segment/ledger context and bytes unchanged;
- `Unsent -> InFlight` retains assignment-time context and does not manufacture a newer key/path binding; explicit confirmation supplies the newer context;
- fully covered advanced duplicates require ledger-global admissibility and exact evidence-context equality; novel bytes cannot inherit `InFlight`, `Uncertain`, or `Confirmed` state;
- carrier packet/TCP feedback cannot call the logical `confirm_received` path or advance delivery watermark;
- TCP frame reliability does not add a second packet-ACK layer or downgrade `ProcessMessage::DeliveryAck` semantics;
- failover replay/dedup preserves exact-once logical delivery without inventing carrier delivery evidence.

No concrete current contradiction required an implementation change. This review is bounded to the already-specified candidate semantics; it does not redefine ACK architecture, packet feedback, Session delivery, migration policy, wire format, freeze status, independent review, WAN behavior, release or production readiness.

## Verification

Focused tests passed for queued assignment context, global-context duplicate rollback, advanced novel-byte state rejection, confirmation atomicity, carrier/Session evidence separation, TCP no-duplicate-ACK contract, and blackhole TCP resume dedup. Session and Carrier clippy with warnings denied passed.

Exact-tree gate:

- `PYTHONDONTWRITEBYTECODE=1 bash scripts/check.sh`: exit `0`
- `git diff --check`: exit `0`
- gate: `2026-09-10T20:40:19Z` -> `2026-09-10T20:42:14Z`
- host: `Linux 6.8.0-137-generic x86_64`
- Rust: `rustc 1.98.0 (88d9e12ae 2026-08-18)`
- initial/final source tree: clean

This is deterministic local candidate evidence review, not an independent security audit or release/production authorization.
