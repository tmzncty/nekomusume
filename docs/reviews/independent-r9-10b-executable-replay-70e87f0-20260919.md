# Independent bounded review — R9-10B executable TCP replay identity / bytes / dedup

**Reviewed source/test anchor:** `70e87f086b0bbd2d13cd9bae073cb1cdb4c5abd4`

**Review-start handoff descendant:** `42947f995d6230102895bf7a8abd1d13989eefac`

**Classification:** item-4 local correctness/review support; no WAN, performance, release, or policy conclusion.

## Scope

Inspected exact-current:

- `crates/neko-cli/src/main.rs` automatic-health UDP -> TCP promotion/replay owner;
- `crates/neko-carrier/src/lib.rs::FailoverController::{track_uncertain,tcp_resend,confirm,receive,apply_manager_decision}`;
- `crates/neko-cli/tests/probe.rs` existing exact TCP replay/session-ACK process oracles;
- D005/D064 and provisional Session v0 evidence separation.

No reviewer-local execution is claimed.

## Invariants challenged

1. Promoted TCP replay must use the authoritative retained logical identity and exact bytes, not merely an equal positional count.
2. Session logical proof, not Carrier packet feedback, retires retained replay ownership.
3. Exact duplicate logical identity/bytes is idempotent; same identity with different bytes fails closed.
4. TCP replay must not manufacture a UDP packet-ACK layer.
5. Confirmation must remove exactly the retained identity whose authenticated Session DeliveryAck was accepted.

## Review result

No concrete defect found at the reviewed anchor.

For the automatic-health path, `FailoverController::uncertain` is the retained `BTreeMap<DataId, Vec<u8>>`; `tcp_resend()` returns clones of those exact `(DataId, bytes)` entries only after TCP owns the failover state. The executable now constructs every replayed `OutboundRecord` directly from that returned pair (`offset = id.0`, `data = bytes`) instead of indexing back into the original positional `records` vector. The prior source-of-truth coupling is therefore removed rather than merely count-checked.

For each replayed record the executable authenticates and validates the TCP Session DeliveryAck against that exact record, applies `SessionRuntime::delivery_ack(stream, offset, len)`, and only then calls `FailoverController::confirm(DataId(offset))`. `confirm` removes the exact retained entry and fails `NotFound` rather than silently succeeding on a missing identity. `receive` keeps exact duplicates idempotent and rejects same-id/different-bytes conflicts before delivered-byte mutation.

Existing process coverage includes an exact replay-identity oracle for the reliable-UDP/failover partition (the expected TCP replay is offset 32 only; wrong reliable-owned/reserved offsets are forbidden) and requires TCP Session delivery-ACK evidence. This is consistent with D005/D064: TCP Session confirmation and UDP packet retirement remain separate evidence domains.

## Evidence boundary / exclusions

- The `70e87f0` source change did not add a new artificial retained-set divergence process seam. This review therefore does not claim mutation-testing proof against every future positional implementation; it closes the current source invariant because the executable's data source is now the retained set itself and existing process oracles bind the expected replay identity.
- The developer-local clean exact-tree gate recorded for short SHA `70e87f0` is accepted as developer-reported provenance only. The GitHub-resolvable full SHA is `70e87f086b0bbd2d13cd9bae073cb1cdb4c5abd4`. Hosted status is supplemental and no reviewer-local gate is claimed here.
- Partial replay/promotion failure cleanup and terminal ownership are deliberately left to R9-10C/R9-11.
- No DataId redesign, capacity change, D019/D064 change, wire/crypto change, or release-state change is implied.

**READY_LIVE: none.**