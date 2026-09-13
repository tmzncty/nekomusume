# Independent bounded DeliveryLedger review — exact `9697ee7`

Bounded independent review of the delivery-ledger half of `crates/neko-session/src/lib.rs` — `DeliveryLedger`, `DeliverySegment`, `DeliveryState`, `Limits`, `insert`, `transition`, `confirm_received`, `validate_context`/`context_ok`, `watermark` (`:23-382`), against `docs/spec/m0-session-state.md` and `docs/specs/nekomusume-session-v0.md`, at reachable exact `9697ee7a0045b39c6ef46c1951fefea486ccf13b`. Not a WAN/live claim, not an independent security/release approval.

## Challenged invariants and result

- **Byte-exact overlap — holds.** `insert` compares every overlapping byte `data[pos-offset] != s.data[pos-s.offset]` and rejects `Conflict` (`:170-181`); divergent bytes at a shared offset never merge.
- **`Unsent -> InFlight -> Uncertain/Confirmed` ownership — holds.** `transition` admits only `Unsent->InFlight` and `InFlight->Uncertain` (`:323-328`); `confirm_received` admits `InFlight|Uncertain|Confirmed -> Confirmed` (`:345-349`, so re-confirmation is idempotent) and rejects `Unsent -> Confirmed` and other invalid transitions atomically.
- **Ledger-global context monotonicity + atomic rejection — holds.** `validate_context` is component-wise monotonic (`delivery_epoch`/`key_phase`/`path_generation` never regress → `OldEpoch`) and rejects a compound epoch advance that also changes crypto/path (`InvalidMigration`) (`:112-125`); `context_ok` validates before committing the ledger-wide context (`:129-132`), and `confirm_received` validates the segment state **and** global context before mutating (`:343-371`).
- **Watermark monotonic under out-of-order confirmation — holds.** `confirm_received` only raises `streams[s]` when `end > watermark` (`:376-377`); the watermark is a monotonic max-confirmed-end (never decreases, per spec §17/§28), and out-of-order confirmations cannot lower it.
- **Per-stream/global resource + offset/reorder/overflow bounds — holds.** `insert` checks `max_streams`, `max_offset_jump` (`OffsetJump`), `max_reorder` (`ReorderLimit`), `checked_add`/`checked_total` for `end`/`bytes`/`max_connection_bytes` (`:144-160`, `:182-186`, `:216-218`, `:279-286`) — all before mutation.
- **Advanced-state exact duplicate vs novel-byte overlap — holds.** When overlapping segments share a non-`Unsent` state, `insert` requires full `existing_coverage >= end` plus matching context on every covered segment, else `InvalidMigration` (`:247-262`) — an advanced-state extension carrying novel bytes cannot manufacture InFlight/Uncertain/Confirmed evidence. `advanced_overlap_extensions_cannot_manufacture_delivery_state` and `confirmation_context_advance_prevents_later_insert_rollback` cover it.
- **Never synthesize bytes across a hole — holds.** A merge requires `existing_ranges + [offset,end]` to cover `[start,finish]` contiguously, else `Conflict` (`:264-278`); no zero-fill across gaps (`overlap_never_zero_fills_a_gap`).
- **No packet/path observation manufactures Session delivery — holds.** `packet_feedback` is a deliberate no-op (`:381`); delivery evidence only enters via the validated `confirm_received` transition.

## Evidence

- `cargo test -p neko-session` on exact `9697ee7`: 43 lib tests pass — including `overlap_merge_and_conflict`, `mixed_state_overlap_is_not_collapsed`, `overlap_never_zero_fills_a_gap`, `confirmation_context_advance_prevents_later_insert_rollback`, `advanced_overlap_extensions_cannot_manufacture_delivery_state`, `watermark_monotonic`, and the out-of-order/old-epoch/invalid-migration rejection cases.
- Exact-tree local provenance for `9697ee7`: `docs/local-gate-9697ee7-20260913.md`.
- No code change; no defect found in this scope. No new `READY_LIVE` question; release/governance state unchanged.
