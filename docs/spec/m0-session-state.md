# M0 candidate Session delivery state

**Status: candidate implementation, not frozen normative v0.**

`neko-session` is a pure synchronous bounded ledger. It has no network, runtime, cryptography, or live failover connection. `DeliveryEpoch`, `KeyPhase`, and `PathGeneration` are distinct types; none is reused as another.

## Candidate states and evidence

A segment moves `UNSENT -> IN_FLIGHT -> UNCERTAIN -> CONFIRMED`. `CONFIRMED` is monotonic. `packet_feedback` is intentionally orthogonal: TCP write completion and UDP packet ACK do not enter the logical confirmation API. Only a candidate logical Session ACK can call `confirm_received`.

This phase models the weakest candidate proof, `received`/transport delivery: the peer parsed and accepted the logical bytes. It does not claim `delivered` to an application stream and never claims `effect` or side-effect commit.

## Candidate invariants

- Same-byte duplicate and overlap are idempotent; conflicting bytes at one offset are rejected.
- ACKs from an old `DeliveryEpoch` are rejected without changing the watermark.
- Per-stream watermarks never decrease.
- `max_reorder`, `max_streams`, `max_connection_bytes`, and `max_offset_jump` produce deterministic errors.
- All limits and field meanings remain provisional until the normative v0 gates are reviewed.


## Lifecycle and close boundary

The M0 ledger is a delivery-state component, not a live Session owner. Its
explicit lifecycle is `Unsent -> InFlight -> Uncertain -> Confirmed`; invalid
transitions and missing ranges return stable `LedgerError` values and do not
mutate state. Exact duplicate insertion is idempotent, old-epoch confirmation
is rejected as replay/rollback evidence, and the watermark is monotonic.
Transport endpoint close is specified and tested in the carrier layer; M0 does
not invent a second Session close protocol or claim application-level
`Delivered`/`Closed` semantics.
