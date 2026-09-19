# Independent bounded review — R9-10C replay cleanup / partial-promotion negatives

**Reviewed repository anchor:** exact reachable `15c0d9f1f15fcbc2f40039d7622bfdce9a33fb26`.

**Source/test anchor for the executable replay repair:** exact reachable `70e87f086b0bbd2d13cd9bae073cb1cdb4c5abd4`.

**Classification:** independent bounded no-finding support for release item 4. This is not a release/security approval, WAN result, performance conclusion, protocol freeze, or production-readiness claim.

## Scope inspected

- `crates/neko-cli/src/main.rs`: automatic-health promotion, authoritative TCP replay source, authenticated TCP `DeliveryAck`, `SessionRuntime::delivery_ack`, retained-identity `confirm`, migration-back ordering, post-return dual Session/Carrier settlement, and final process-success reachability.
- `crates/neko-carrier/src/lib.rs`:
  - `FailoverController::{track_uncertain,apply_manager_decision,tcp_resend,confirm,receive,apply_migration_back}`;
  - `ConcurrentCarrierManager::{activate,fail,assign,confirm,replay_uncertain,finish_drain}` and its retained-range ownership model.
- `crates/neko-session/src/lib.rs`: current `SessionRuntime::delivery_ack` forward-gap rejection and terminal runtime cleanup/check boundary, only as needed to challenge replay confirmation ordering.
- Developer R9-10C note at exact `15c0d9f1f15fcbc2f40039d7622bfdce9a33fb26`.
- Applicable D005/D064 and provisional Session-v0 evidence-domain boundaries.

No reviewer-local command or test execution is claimed in this note. Existing developer-local/hosted results remain separately classified evidence.

## Invariants challenged

### 1. Failed promotion cannot consume replay ownership

The executable automatic-health path calls manager promotion first and requires `FailoverController::apply_manager_decision` to accept the exact UDP -> TCP decision before obtaining `tcp_resend`. `FailoverController::tcp_resend` itself fails with `WrongCarrier` unless TCP is active. No retained identity is removed by either promotion check.

No counterexample found.

### 2. Replay bytes and identity are authoritative retained state

Since exact `70e87f0`, automatic-health replay constructs each `OutboundRecord` directly from every retained `(DataId, bytes)` returned by `FailoverController::tcp_resend()`. It no longer derives replay identity from an equal-count positional slice.

No counterexample found.

### 3. Write/read/auth/Session-proof failure cannot confirm or delete the current retained identity

For each replay record the executable owner performs, in order:

1. TCP write;
2. bounded ACK read;
3. authenticated open;
4. exact `delivery_ack_matches` check;
5. `SessionRuntime::delivery_ack`;
6. only then `FailoverController::confirm(DataId(record.offset))`.

Any failure before step 6 exits the operation before the retained identity is removed. A successful proof removes exactly that identity; a repeated `FailoverController::confirm` is typed `NotFound` rather than a second removal. Exact duplicate receive-side data remains separately idempotent (`Ok(false)`), while conflicting bytes are typed `Conflict`.

No counterexample found.

### 4. Partial replay cannot silently discard later unconfirmed identities or fall through to success

`FailoverController::confirm` removes only the one proved `DataId`. Later retained identities stay in the bounded map if a subsequent replay iteration fails. The executable migration-back block is after the full replay loop, and final `failover_client_ok` is later still; a mid-loop `fail(...)` cannot reach either.

The policy-core `ConcurrentCarrierManager` has the same retention shape: `replay_uncertain` / `finish_drain` reassign stable `LogicalRangeId + bytes` ownership but do not remove the retained range; only explicit `confirm` removes it. A later hard failure can therefore mark an unconfirmed reassigned range uncertain again.

No counterexample found.

### 5. Carrier packet feedback cannot substitute for Session logical proof

The post-return reliable-UDP owner still requires both domains before settlement: Session ownership must drain `post_outstanding`, and Carrier Recovery must drain `in_flight`. Carrier ACK handling never calls `SessionRuntime::delivery_ack`; Session ACK handling does. The terminal settled event is emitted only after both are complete.

No counterexample found.

### 6. Migration-back cannot bypass unresolved TCP replay

The migration-back recovery/hold/promotion block is reached only after the TCP replay loop has completed successfully. A failed replay therefore cannot be converted into a successful migration-back result.

No counterexample found.

### 7. Scope boundary for cleanup

Within R9-10C, operation failure is fail-closed with respect to success evidence and retained logical ownership. This review does **not** claim full socket/Session/Recovery/manager terminal resource closure across every success/failure/timeout/shutdown path; that broader question remains the dedicated R9-11 lifecycle lane.

This exclusion is intentional so a narrow R9-10C no-finding note cannot be promoted into lifecycle closure.

## Result

No concrete correctness defect was found in the bounded R9-10C replay cleanup / partial-promotion scope at exact `15c0d9f1f15fcbc2f40039d7622bfdce9a33fb26` over source/test exact `70e87f086b0bbd2d13cd9bae073cb1cdb4c5abd4`.

The developer note's prose should be read consistently with repository truth: repeated `FailoverController::confirm` is typed `NotFound`; exact duplicate **receive** is the idempotent `Ok(false)` case.

`READY_LIVE: none`. No D019/D064 policy value, wire/crypto/ACK architecture, destructive migration, release authority, or production boundary is changed by this review.
