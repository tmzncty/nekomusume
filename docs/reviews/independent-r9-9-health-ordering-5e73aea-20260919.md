# Independent bounded R9-9 review — health outcome / promotion / switch ordering

**Repository anchor reviewed:** `5e73aead6d16506a6b299a459928b9c001022405`.
**Source/test anchor:** `d97a536414a78ff44ef80b1a6d428ddf96ae6e22`; commits after that anchor through `5e73aea` are review/handoff docs only and do not change the inspected implementation/tests.

## Scope and owners inspected

This pass independently challenged the integrated reliable-UDP resolved-outcome -> health/hysteresis -> Carrier promotion boundary, rather than accepting the developer no-finding note as evidence by itself.

Inspected exact-current owners and reachable tests include:

- `PathRecovery::{on_ack,on_pto,fresh_health_sample,quiesce}` and its resolved counters / `outcome_epoch` / `health_epoch` bridge;
- `ReliableUdpRuntime::{poll_health,ready_standby,activate_udp,teardown}` and terminal control-plane gating;
- `ConcurrentCarrierManager::{observe_readiness,activate,fail,assign}` including Warm-only activation, single-active ownership, drain/fail state, and uncertain-range admission;
- the separate D064 `CarrierManager` warm/cold recovery and migration-back generation/session/readiness/health-margin/hold gates;
- existing deterministic regressions including `fresh_health_bridge_emits_one_observation_per_resolved_outcome`, `warm_switch_is_reason_coded_dwell_guarded_and_drains_uncertain`, `carrier_scoring_and_hysteresis_bound_oscillation`, `exact_warm_generation_promotes_only_after_failure`, `warm_and_cold_recovery_are_classified_by_failure_order`, `migration_back_requires_validation_generation_health_margin_and_hold`, `fallback_is_not_fabricated_when_tcp_not_ready`, `teardown_preserves_lifetime_history_and_gates_control_plane`, and the reliable-UDP post-return PTO recovery process coverage.

## Independent challenges

1. **Freshness is tied to resolved transport outcomes, not sends.** A bare packet reservation/send does not advance `outcome_epoch`; authenticated ACK/loss retirement or a PTO that actually schedules probes does. `fresh_health_sample` consumes an epoch once and uses resolved-packet interval deltas, so cumulative historical loss is not replayed merely by later traffic or repeated polling.

2. **A recoverable PTO/loss is not itself a promotion proof.** Recovery/PTO state first becomes a typed health sample. Carrier promotion still requires the health state transition plus an already-Warm target; packet ACK, PTO, socket activity, or Session DeliveryAck alone cannot satisfy readiness.

3. **Promotion is readiness-gated and single-active.** `ConcurrentCarrierManager::activate` rejects a target whose state is not `Warm`, applies dwell/cooldown guards for non-hard switches, marks the old active owner uncertain/draining when appropriate, and stores one active key. `assign` takes new logical-range ownership only from that active key, so a failed/draining path is not independently selectable for new Session ownership.

4. **Failure ordering does not fabricate success.** `ReliableUdpRuntime::poll_health` reports `WarmFallback` only after both manager fail/switch operations complete; transition errors become `FallbackFailed`. The existing negative where TCP is not ready does not create successful fallback evidence.

5. **Terminal runtime state is inert for this surface.** `torn_down` makes `poll_health` return `Idle`, and the terminal guards already reviewed under H-R9-065/066 prevent new readiness/activation mutation through the runtime. Quiesce preserves lifetime diagnostics while live Recovery/Reno/plaintext ownership is dropped.

6. **Migration-back remains a distinct validated decision.** The current D064 manager requires the expected generation/session/readiness and its migration-back health-margin/hold conditions; it is not reachable merely from UDP packet feedback or the generic health bridge.

7. **Evidence domains remain separate.** Health/switch `RuntimeEvent` values are not Session delivery acknowledgement or Carrier packet-ACK evidence. The inspected code does not promote one evidence class into another.

I also challenged a potentially dangerous ordering case: `poll_health` fails the degraded UDP before attempting warm-TCP activation. This can truthfully produce `FallbackFailed` when the standby is not Warm, but the behavior is explicit in the current owner/tests and does not fabricate a successful switch or assign data to a non-active path. Changing that ordering would alter the current failover contract rather than repair an unambiguous implementation defect, so this review does not invent a different policy.

## Result

**No concrete correctness/security/evidence defect found in the bounded R9-9 scope.** R9-9 may close as independent no-finding support for release item 4.

This is source/test/review evidence only. No reviewer-local shell execution is claimed: the automation environment could not materialize a GitHub checkout, so it did not run `scripts/check.sh` or `git diff --check`. The accepted developer-local clean exact-tree gate for source/test anchor `d97a536` and its hosted CI remain separate cross-evidence; this docs-only review commit does not claim to retest that tree.

## Exclusions / unchanged gates

- no wire decoder/parser/crypto framing change; no fuzz run is implied;
- no WAN/live run or performance conclusion; `READY_LIVE: none` remains appropriate;
- no change to D019, hysteresis/capacity/security numeric policy, Session/Carrier/ACK/crypto/wire architecture, signing/SBOM/publication policy, previous release policy, or release authority;
- release evidence item 3 and independent-review item 4 remain repository-level incomplete until their remaining lanes/reconciliation close;
- `RELEASE_CANDIDATE=false`, `PRODUCTION_READY=false`, `FREEZE=false`, `RELEASED=false` remain unchanged.
