# Local Session global-context duplicate closure — `0f8f192`

Developer-run exact-tree validation of reachable implementation/test commit `0f8f19251bdb08c9179265ff26ed432fd07b894d`.

## Defect and repair

A fully covered advanced-state duplicate could return idempotent success after matching only its overlapped segment context. If a disjoint segment had already advanced the ledger-global key/path context, a duplicate carrying the older context could therefore bypass the global rollback rule.

The repair separates the existing context rule into a pure `validate_context` step and the existing mutating commit step. Advanced duplicates now:

1. validate against ledger-global context without mutation;
2. require exact context equality with every covered advanced segment;
3. return idempotent success without replacing segments or changing context only when both conditions hold.

Thus global rollback returns `OldEpoch`; a globally current context that differs from an advanced segment's evidence context returns `InvalidMigration`; exact admissible equality remains idempotent. `Unsent` merges and explicit `confirm_received` context advancement retain their existing behavior.

The disjoint A/B regression matrix covers `InFlight`, `Uncertain`, and `Confirmed`: A retains `(1,0,1)`, B advances global context to `(1,1,2)`, stale duplicate A rejects as `OldEpoch`, globally current duplicate A rejects as `InvalidMigration`, and both failures preserve global context, segment topology/context/state, bytes, and watermark exactly. Existing same-context, multi-segment, novel-byte extension/bridge, mixed-state, conflict, bounds, and confirmation-context tests remain green.

No wire, ACK, cryptographic, Carrier, capacity, or migration-policy semantics changed.

## Exact-tree gate

- `PYTHONDONTWRITEBYTECODE=1 bash scripts/check.sh`: exit `0`
- `git diff --check`: exit `0`
- gate: `2026-09-10T01:11:28Z` -> `2026-09-10T01:13:23Z`
- host: `Linux 6.8.0-137-generic x86_64`
- Rust: `rustc 1.98.0 (88d9e12ae 2026-08-18)`
- initial/final source tree: clean

This is developer-local deterministic Session-state evidence. It is not a protocol freeze, interoperability proof, independent review, public-listener approval, RC, release, or production authorization.
