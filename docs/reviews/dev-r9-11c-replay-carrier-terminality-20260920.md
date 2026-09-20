# Developer bounded R9-11C review — retained Session replay / Carrier-generation terminality

**Anchor:** exact source/test tree `730f993` + reviewer commits through `57b7912`.
**Owners inspected:** `FailoverController` (`uncertain`/`received`/`confirm`/
`tcp_resend`/`apply_migration_back`), `ConcurrentCarrierManager` (`ranges`/
`mark_owner_uncertain`/`replay_uncertain`/`finish_drain`/`confirm`/`activate`/
`path_mut` generation gate), `ReliableUdpRuntime` torn-down gates,
retained `(DataId, bytes)` replay-set ownership.

## Per-invariant coverage

| # | Invariant | Coverage |
|---|---|---|
| 1 | unresolved Session replay not erased by UDP/path-recovery terminalization | retained `uncertain`/`ranges` live in `FailoverController`/`ConcurrentCarrierManager`, not in the torn-down `ReliableUdpRuntime` — UDP teardown does not erase the replay ledger |
| 2 | confirmed replay identity disappears only after Session logical delivery proof | `FailoverController::confirm(id)`/`ConcurrentCarrierManager::confirm(id)` remove the identity; Carrier packet feedback is a separate domain (H-R9-024/048) |
| 3 | failed/retired generation cannot become active/own replay/gain readiness without fresh transition | `path_mut`/`path` reject `UnknownPath`/`OldGeneration`/`GenerationMismatch`; `observe_readiness` rejects `Failed`/`Draining`/`Active`; `activate` requires `Warm` |
| 4 | migration-back cannot resurrect stale failed-generation ownership or bypass gates | `validate_and_promote` + `promote_pending_target` enforce generation/session/health-margin/hold; `promote_warm_authenticated_resume` is failure-order classified |
| 5 | duplicate/conflicting replay identity deterministic/bounded; no positional substitution | `LogicalRangeId`/`DataId` is the single owner key; `receive` exact-duplicate → `Ok(false)` + `duplicate_bytes`, differing bytes → `Conflict`; replay constructs records from authoritative `tcp_resend()` `(DataId, bytes)` (R9-10B) — never a positional slice |
| 6 | cleanup/partial-promotion negatives fail-closed, no manufactured success | capacity `ensure_uncertain_capacity` atomic before mutation; `tcp_resend` `WrongCarrier` on non-TCP; partial replay aborts before final success evidence (R9-10C) |

## Reachable regressions named

- `warm_switch_is_reason_coded_dwell_guarded_and_drains_uncertain`,
  `uncertain_duplicate_and_counter_boundaries_are_atomic`,
  `cooldown_and_uncertain_capacity_rejections_are_atomic`.
- `both_fail_then_new_generation_recovers_in_readiness_order`,
  `warm_and_cold_recovery_are_classified_by_failure_order`,
  `exact_warm_generation_promotes_only_after_failure`.
- `migration_back_requires_validation_generation_health_margin_and_hold`.
- `hard_failure_switches_and_resends_uncertain_with_dedup`,
  `udp_blackhole_recovers_over_tcp_without_loss_and_deduplicates`.
- `bounded_udp_blackhole_tcp_resume_preserves_order_and_exactly_once_bytes`.

## Result

No concrete defect found across retained Session replay / Carrier-generation
terminality. The replay ledger survives UDP/path-recovery teardown; only
Session proof removes an identity; generation/readiness gates reject stale
ownership; migration-back is validated; dedup/conflict is deterministic.

**READY_LIVE: none** — deterministic local evidence only.
