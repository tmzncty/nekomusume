# R-CS-2 — independent bounded CarrierState single-active / drain / fail / activate review

**Exact reviewed repository anchor:** `259fb58ec19a47fe4e11900dbe6b338bb09f9467`

**Scope:** the exact-current bounded M0 `CarrierState` path-evidence model in `crates/neko-carrier/src/lib.rs` against `docs/spec/m0-carrier-state.md`. D064's richer concurrent manager / warm-standby migration contract is reviewed under the separate `ConcurrentCarrierManager` lane; this note does not pretend the older M0 model is the runtime manager.

## Challenged invariants

1. **At most one `Active` owner — no finding.** `Activate` rejects while `self.active` is populated, then validates the exact path generation/state/hysteresis before setting both the path state and active tuple. No other event assigns `self.active`.
2. **Drain is legal only from `Active` — no finding.** `BeginDrain` requires exact generation and `PathState::Active`, then changes only that path to `Draining`. Candidate/Validating/Degraded/Failed paths cannot enter drain by this event.
3. **Fail is explicit and legal only from `Degraded` or `Draining` — no finding.** This matches the candidate M0 spec. A successful fail is terminal for that record, and it clears `self.active` only when the failed tuple is the currently published owner. Wrong path/generation and illegal-state failures return before mutation.
4. **Activation cannot skip validation/hysteresis/state gates — no finding.** A failed/degraded/draining/validating record cannot be activated because success requires operational `Candidate`, validation `Validated`, and both committed hysteresis thresholds.
5. **`active_epoch` advances exactly on accepted activation — no finding.** It is incremented only after all activation checks and path-state mutation succeed; drain/fail/rejections do not advance it.
6. **Active clearing is deterministic — no finding within this model.** `BeginDrain` intentionally leaves the active tuple occupied until an explicit `Fail` closes the M0 transition. The candidate spec requires explicit failure from degraded/draining and does not define a separate `draining_owner` plus simultaneous replacement-active representation.

## Important scope boundary: M0 model vs D064 runtime manager

D064 later specifies the real concurrent carrier-manager ownership sequence (`old active -> draining`, then a separately warm candidate may become the sole owner of new Session data while the old path finishes already-assigned work). `CarrierState` predates and does not model `warm`, assigned Session data, drain deadlines, or a simultaneous draining-owner/new-active pair. Therefore:

- retaining `self.active` through `BeginDrain` is **not** used here as evidence that the D064 runtime must wait for terminal failure before promoting a warm standby;
- conversely, D064 is **not** used to manufacture a defect in this explicitly bounded M0 state model;
- the separate `ConcurrentCarrierManager` / runtime lane must prove the richer D064 ordering and single-active-new-data invariant.

Likewise, this M0 owner has no committed in-place replacement operation for reusing the same `PathId` with a fresh generation after a terminal record. D064 fresh-generation behavior belongs to the manager/runtime identity owner; no destructive or canonical generation-migration semantics are invented here.

## Existing deterministic coverage inspected

The exact-current unit/integration tests exercise activation gating, generation rejection, PTO degradation-vs-failure separation, and deterministic invalid transitions. The prior bounded CarrierState review at `8f93b93` is consistent with these M0 invariants; this slice re-challenges the exact-current source rather than extrapolating that older note to D064 runtime behavior.

No reviewer-local Rust/full-gate execution was performed for this docs-only no-finding slice; no developer-local, hosted, WAN, or performance evidence is reclassified.

## Exclusions / next lane

This review excludes `ConcurrentCarrierManager`, `CarrierHealth`, migration-back, warm readiness, Session uncertain replay, sockets, and live evidence. **R-CM-DIFF** is dependency-ready next: verify whether the current concurrent-manager / health / migration-back semantic owners moved relative to their dedicated bounded review anchors, and re-challenge only moved seams.

**Result:** bounded no-finding for R-CS-2 at exact `259fb58`. Release items 3/4 and governance flags remain unchanged; `READY_LIVE: none` remains current.