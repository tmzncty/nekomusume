# R-CS-1 — independent bounded CarrierState generation / validation / hysteresis review

**Exact reviewed repository anchor:** `270265f13c2644f3151dcb21d02ccd1dbbb38902`

**Scope:** exact-current `CarrierState` / `CarrierEvent` / `PathRecord` in `crates/neko-carrier/src/lib.rs`, `docs/spec/m0-carrier-state.md`, the evidence-domain boundary in `docs/specs/nekomusume-session-v0.md`, and the accepted carrier-manager direction in D064 only where it unambiguously applies to this bounded M0 path-evidence model. This is release-item-4 review support, not a security/release approval and not a live-network claim.

## Challenged invariants

1. **Old/future generation rejection is fail-closed and mutation-free — no finding.** `checked(path, generation)` resolves the record and returns `OldGeneration` for a lower generation and `GenerationMismatch` for a higher generation before returning a mutable record to the event branch. Every event that can mutate an existing path routes through `checked`. Rejected generation events return before `tick_dwell`, so they cannot change validation/state/success/dwell/active/epoch as a rejection side effect.
2. **Packet feedback cannot create validation or Session-delivery evidence — no finding.** `PacketFeedback::Ack` increments `successes` only after exact-generation checking and only when validation is already `Validated`; loss/reorder only affect operational health state for the exact active tuple. `ChallengeValidated` remains the only transition to `Validated`; `SessionDelivery` is a separately typed checked no-op in this model.
3. **Validation transition is atomic — no finding.** `ChallengeSent` requires candidate validation + candidate operational state before mutating both to `Validating`; `ChallengeValidated` requires `Validating` before changing validation/state and resetting success evidence. Invalid or wrong-generation attempts return before mutation.
4. **Activation hysteresis gates are checked before activation state/epoch mutation — no finding.** `Activate` requires no existing active owner, exact generation, `Validated`, operational `Candidate`, `successes >= k_successes`, and `dwell_events >= min_dwell_events`; only after all gates pass does it set `Active`, publish the active tuple, and increment `active_epoch`.
5. **Rejected events do not accidentally advance dwell — no finding.** `tick_dwell()` is reached only after a successful event application. The historic M0 audit intentionally constrained dwell accumulation to already-validated paths (`8a7485d` P1-6), and the exact-current owner preserves that rule.

## Hysteresis evidence-scope boundary

`tick_dwell()` deliberately increments every **validated** path after each successful carrier-state event. The current candidate spec defines `min_dwell_events` as a configurable deterministic gate but does not state that a dwell event must be local to the candidate path, and the historical audit repair explicitly changed only *which validation state may accumulate dwell*, not the all-validated-path iteration. I therefore do **not** classify cross-path successful-event accumulation as a defect or invent a path-local policy. A later normative decision may choose a different dwell semantic; that would be a policy/spec change, not a correctness repair justified by current committed text.

## Existing deterministic coverage inspected

- unit: `ack_is_not_validation`
- unit: `pto_is_not_failure`
- unit: `old_generation_and_late_old_path_are_rejected`
- unit: `failover_gate_requires_validation_and_hysteresis`
- unit: `duplicate_path_reports_exists_before_capacity`
- unit: `limits_and_invalid_transitions_are_deterministic`
- integration: `evidence_domains_do_not_cross_promote_delivery`
- integration: `old_generation_and_late_old_path_cannot_advance_watermark`
- integration: `activation_requires_challenge_validation_and_hysteresis`

The future-generation branch is source-provable from `checked`; no new checker/test is warranted solely to duplicate the symmetric comparison. I did not execute reviewer-local Rust tests in this slice, and I do not convert historical/developer-local/hosted results into reviewer-local evidence.

## Exclusions / next lane

This slice does **not** adjudicate drain/fail/activate ownership release, terminal/fresh-generation replacement, ConcurrentCarrierManager health/migration-back, Session delivery, socket adapters, WAN evidence, or policy values. Those remain separate lanes. **R-CS-2** (single-active / drain / fail / activate) is dependency-ready next.

**Result:** bounded no-finding for R-CS-1 at exact `270265f`. Release items 3/4 and all release/governance flags remain unchanged. `READY_LIVE: none` remains the authoritative live classification.