# Developer bounded R9-10C review — replay cleanup / partial-promotion negatives

**Anchor:** exact source/test tree `70e87f0` + reviewer commits through `bd99575`.
**Owners inspected:** `FailoverController::{apply_manager_decision,tcp_resend,
confirm,receive,apply_migration_back}`, executable automatic-health promotion +
TCP replay loop (uses authoritative `tcp_resend` `(DataId,bytes)` since R9-10B),
`ConcurrentCarrierManager::{mark_owner_uncertain,replay_uncertain,finish_drain,
confirm}`.

## Per-invariant coverage

| # | Invariant | Coverage |
|---|---|---|
| failed auth/readiness/promotion cannot consume retained uncertain ownership or emit success | `tcp_resend` requires `active == Tcp` (`WrongCarrier`); a failed promotion never reaches it; uncertain set unchanged on any failure before confirm |
| a TCP write/read/auth/DeliveryAck failure before confirm cannot fabricate Session confirmation or remove the unsatisfied identity | `confirm` is invoked only after successful authenticated Session proof (see resumed-session ordering fix); a mid-loop TCP `fail` terminates the operation — retained identity stays in `uncertain`, final `failover_client_ok`/`summary` unreachable |
| successful Session proof removes exactly that proved identity once; duplicate confirm/receive idempotent or typed-rejected | `confirm` `uncertain.remove(id)` — second confirm → `NotFound` typed rejection; `receive` on exact-duplicate returns `Ok(false)` + `duplicate_bytes` counter, on differing bytes `Conflict` — never double-counted |
| partial replay cannot silently drop a later unsent/unconfirmed identity; final success impossible | replay iterates the authoritative retained set; a mid-loop failure aborts the operation before `failover_client_ok` — unconfirmed retained identities remain and are never silently dropped |

## Reachable regressions named

- `uncertain_duplicate_and_counter_boundaries_are_atomic` — duplicate confirm
  idempotent (Ok), conflict typed-rejected, counter boundaries atomic.
- `uncertain_limits_are_atomic` — capacity rejection before mutation.
- `hard_failure_switches_and_resends_uncertain_with_dedup` — dedup on stable id.
- `bounded_udp_blackhole_tcp_resume_preserves_order_and_exactly_once_bytes` —
  Session proof before confirm, exactly-once application bytes,
  `tcp_resend` empty only after all identities confirmed.

## Result

No concrete defect found across replay cleanup / partial-promotion negatives.
Confirm removes exactly one proved identity; duplicate receive/confirm is
idempotent or typed-rejected; a partial replay aborts the operation before
final success can be claimed.

**READY_LIVE: none** — deterministic local evidence only.
