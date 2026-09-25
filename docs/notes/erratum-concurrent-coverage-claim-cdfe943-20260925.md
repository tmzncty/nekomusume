# Erratum — cdfe943 commit-message coverage claim

**Date:** 2026-09-25. **Erratum for:** commit `cdfe94310634ecc59f5c88b2dcd9704ad196203d`
(provenance `docs/notes/check-gate-cdfe943-20260925.md`).

## What the commit message claimed vs. source truth

The cdfe943 commit message states the ConcurrentCarrierManager surface
"had zero unit coverage outside indirect use by other suites." That claim
is **wrong**. The `concurrent_manager_tests` module in the same file
(`crates/neko-carrier/src/lib.rs`, in-repo before cdfe943) provides direct
dedicated coverage of: readiness boundedness/generation scoping
(`readiness_is_bounded_consecutive_and_generation_scoped`), warm switch
with reason coding + dwell guard + drain/replay of uncertain ranges
(`warm_switch_is_reason_coded_dwell_guarded_and_drains_uncertain`),
warm/cold recovery classification by failure order
(`warm_and_cold_recovery_are_classified_by_failure_order`), both-fail +
new-generation recovery ordering
(`both_fail_then_new_generation_recovers_in_readiness_order`), endpoint
rebind semantics, and cooldown/uncertain-capacity atomic rejections
(`cooldown_and_uncertain_capacity_rejections_are_atomic`).

The false claim originated from an incomplete search: the coverage scan
grep'd for the type name `ConcurrentRuntime`, which does not exist (the
type is `ConcurrentCarrierManager`), so the existing module was missed.
This is the same evidence-truth class as the H-I4-121/122/123 lesson: a
claim in commit/docs text that the reachable source disproves.

## What cdfe943 actually adds (verified by line-level diff)

The two new `concurrent_tests` regressions are partially redundant with
`concurrent_manager_tests` but do add focused assertions not present
before:

- idempotent same-bytes reassignment of a live unconfirmed range returns
  `Ok(())` (only the conflicting-bytes `Conflict` path was previously
  asserted);
- double-`confirm` of a released range returns `NotFound` for
  `ConcurrentCarrierManager::confirm` (previously asserted only for
  `FailoverController::confirm`);
- `assign`/`replay_uncertain` are rejected with `NoActive` while the
  manager has no active owner after a hard failure (fail-closed
  rejections were previously unasserted);
- the drain→Failed path accepts a new-generation re-registration of the
  drained path (previously shown only for the hard-fail path);
- exact switch-event log ordering across initial activation, hard
  failure, and replacement activation with epochs 1/1/2.

The soft-replacement (Draining/deadline/replay) and hard-failure
retention/replay stories themselves were already covered by
`concurrent_manager_tests`; the cdfe943 versions restate them from an
explicit `LogicalRangeId`-construction angle.

## Consequence

The cdfe943 gate run and provenance remain valid execution evidence for
that tree (100/100 tests). This erratum corrects only the coverage claim.
No release flag, policy, or classification is affected; the 13-surface
owner-diff reuse determination is unaffected (test-support surface only).
