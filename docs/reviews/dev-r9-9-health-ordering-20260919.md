# Developer bounded R9-9 review — health outcome / promotion / switch ordering

**Anchor:** exact source/test tree `d97a536` + reviewer commits through `3dc2e69`.
**Owners inspected:** `PathRecovery::fresh_health_sample` (resolved-outcome-
gated), `ReliableUdpRuntime::poll_health` (HealthSample/WarmFallback/
FallbackFailed/Idle), `ConcurrentCarrierManager::activate` (Warm + dwell +
cooldown + single-active), `validate_and_promote` / `promote_pending_target`
(migration-back generation/health/hold), `PathRecovery::outcome_epoch` /
`last_health_*` (delta-keyed resolved outcomes).

## Per-invariant coverage

| # | Invariant | Coverage |
|---|---|---|
| recoverable PTO/loss settles on UDP, no spurious TCP promotion | `fresh_health_sample` produces a sample only when `outcome_epoch` advanced (resolved ACK/loss/retransmit outcome); `poll_health` returns `HealthSample`, promotion requires the health hysteresis — a single recovered PTO does not fabricate `WarmFallback` |
| terminal/torn-down UDP makes no fresh health/readiness/promotion input | H-R9-065/066: `poll_health` returns `Idle`, `ready_standby`/`activate_udp`/`manager_mut` gated after `torn_down` |
| promotion requires existing readiness + current health/hysteresis | `activate` rejects non-Warm target (`NotReady`); `Warm` requires `k_ready` consecutive authenticated+admitted observations; `poll_health` routes through `health` hysteresis — promotion is never readiness-free |
| draining/failed UDP takes no new Session Data ownership | `observe_readiness` rejects `Failed`/`Draining`/`Active` (`IllegalState`); `activate` single-active + drain timeout governs old path |
| single-active across fail/activate/drain ordering | `self.active` is a single `Option`; `activate` drains the old path only after Warm target admitted; `warm_switch_is_reason_coded_dwell_guarded_and_drains_uncertain` |
| health/decision evidence distinct from packet/Session evidence | `RuntimeEvent` (`HealthSample`/`WarmFallback`/`FallbackFailed`/`Idle`) is a separate domain from `udp_packet_ack_*`/`udp_delivery_ack_*` — never collapsed |
| migration-back cannot bypass generation/session/readiness | `validate_and_promote` + `promote_pending_target` enforce generation/session/health-margin/hold (R9-2 review); `promote_warm_authenticated_resume` classified by failure order |
| structured health/switch evidence follows the typed state/outcome | `WarmFallback(event)` carries the real `ConcurrentSwitchEvent`; `FallbackFailed` is explicit on failed promote — never silently a fallback; `HealthSample` is produced only by a resolved-outcome epoch |

## Reachable regressions named

- `warm_switch_is_reason_coded_dwell_guarded_and_drains_uncertain` — reason code,
  dwell/cooldown guards, uncertain drain.
- `carrier_scoring_and_hysteresis_bound_oscillation` — hysteresis bounds
  promotion oscillation.
- `exact_warm_generation_promotes_only_after_failure`,
  `warm_and_cold_recovery_are_classified_by_failure_order` — failure-order
  classification.
- `fallback_is_not_fabricated_when_tcp_not_ready` — degraded UDP + no ready
  standby never fabricates WarmFallback.
- `teardown_preserves_lifetime_history_and_gates_control_plane` — torn-down
  runtime emits Idle + no new control evidence.
- `reliable_udp_post_return_data_loss_recovers_via_pto_retransmit` — real
  PTO/loss settles on UDP (recovery, not spurious promotion).

## Result

No concrete defect found across health-outcome/promotion/switch ordering.
Resolved outcomes gate health samples; promotion requires Warm readiness +
hysteresis; single-active and terminal invariants hold; health/switch
evidence is a distinct typed domain.

**READY_LIVE: none** — deterministic local evidence only.
