# Developer bounded R9-8 review — warm concurrent TCP readiness / directional forwarding

**Anchor:** exact source/test tree `d97a536` + reviewer commits through `c282705`.
**Owners inspected:** `ConcurrentCarrierManager::observe_readiness` / `activate`
/ `path_mut` (generation gate), `ConcurrentPathState` transitions,
`ReliableUdpRuntime::ready_standby` / `activate_udp` / `manager_mut`
(torn-down gates), `validate_and_promote` / `promote_warm_authenticated_resume`
(D064 warm/cold recovery classification).

## Per-invariant coverage

| # | Invariant | Coverage |
|---|---|---|
| 1 | resume-bound TCP warm standby carries readiness/control only before promotion | `observe_readiness` only transitions Standby->Warm on `k_ready` consecutive authenticated+admitted observations; `ConcurrentPathState::Warm` is the typed ready transition, no Data admission on Warm |
| 2 | no application Data on TCP while UDP is active | `activate` rejects non-Warm target (`NotReady`); only the `active` path carries transport work — `ConcurrentPathState::Active` is single and `activate` enforces one-active |
| 3 | readiness not inferred from TCP connect / UDP feedback / Session ACK / stale generation | `observe_readiness` requires explicit `authenticated && admitted` arguments per observation, keyed to `ConcurrentPathKey{path,generation}` — `path_mut` rejects `UnknownPath`/`OldGeneration`/`GenerationMismatch` |
| 4 | readiness admission bound to session/epoch/path generation + direction | `path_mut` generation gate; per-path state — a wrong-generation observation returns the typed error, never mutates the live path |
| 5 | failed/incomplete readiness emits no positive ready/promotion evidence | `!authenticated || !admitted` resets `ready = 0` and returns current (non-Warm) state — never fabricates Warm; `activate` `NotReady` on any non-Warm target |
| 6 | single-active + multi-ready through standby/activation/drain/failure | one `self.active`; `warm_switch_is_reason_coded_dwell_guarded_and_drains_uncertain` proves dwell/cooldown/drain guards; `both_fail_then_new_generation_recovers_in_readiness_order` |
| 7 | torn-down runtime manufactures no new readiness/control evidence | H-R9-065/066: `ready_standby`/`activate_udp` no-op, `manager_mut` returns `None` after `torn_down` |
| 8 | positive structured readiness evidence follows the actual typed transition | `ConcurrentPathState::Warm`/`Active`/`Draining`/`Failed` are the only observable states; `observe_readiness` returns the real state after mutation |

## Reachable regressions named

- `readiness_is_bounded_consecutive_and_generation_scoped` — k_ready consecutive
  bound + generation-scoped admission.
- `warm_switch_is_reason_coded_dwell_guarded_and_drains_uncertain` — activate
  reason code, dwell/cooldown guards, uncertain drain.
- `warm_and_cold_recovery_are_classified_by_failure_order`,
  `both_fail_then_new_generation_recovers_in_readiness_order` — failure-order
  warm/cold classification + new-generation recovery order.
- `endpoint_rebind_requires_fresh_exact_challenge_and_promotes_atomically` —
  authenticated-resume rebind is exact-challenge gated and atomic.
- `teardown_preserves_lifetime_history_and_gates_control_plane` — torn-down
  runtime: no readiness/activation/mutable-manager escape.

## Result

No concrete defect found across warm-readiness invariants. Readiness is
control-only, generation-scoped, consecutive-bounded, single-active, and
terminal-gated after teardown. The typed `ConcurrentPathState` transition
is the only observable readiness evidence.

**READY_LIVE: none** — deterministic local evidence only.
