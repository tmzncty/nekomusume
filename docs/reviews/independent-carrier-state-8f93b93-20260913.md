# Independent bounded CarrierState review — exact `8f93b93`

Bounded independent review of `CarrierState`/`CarrierEvent`/`PathRecord` in `crates/neko-carrier/src/lib.rs` (`:49-318`) against `docs/spec/m0-carrier-state.md`, at reachable exact `8f93b931b42e486882980f44813660ba6edb4267`. Pure synchronous state model only — not network/failover/tunnel behavior, not Session delivery evidence, not an independent security/release approval.

## Challenged invariants and result

- **Packet feedback never promotes validation or Session delivery — holds.** `PacketFeedback::Ack` only increments `successes` when `validation == Validated` (`:228-231`); it never sets `validation` or emits delivery evidence. `Loss`/`Reordered` only degrade an `is_active` path (`:233-239`). `SessionDelivery` is checked for path/generation but mutates nothing (`:250-252`). Covered by `ack_is_not_validation` and `pto_is_not_failure`.
- **Old/future generation fail-closed and mutation-safe — holds.** `checked` (`:303-317`) returns `OldGeneration`/`GenerationMismatch` before any mutation; `old_generation_and_late_old_path_are_rejected` covers both directions.
- **Activation requires validated path + hysteresis, no two active owners — holds.** `Activate` (`:270-290`) checks `active.is_some()` -> `ActivePathConflict` first, then `ValidationRequired`, `InvalidTransition` (non-Candidate), and `HysteresisGate`, all before any mutation; only then sets `state=Active`, `active`, and `active_epoch += 1`. `failover_gate_requires_validation_and_hysteresis` covers the gate.
- **Single-active ownership across transitions — holds.** `Fail` clears `active` only if the failed path is the current owner (`:266-268`). `BeginDrain` requires `state == Active` and sets `Draining` (`:253-258`) but does **not** release `active` — a draining path still holds the single-active slot, so `Activate` of a sibling is `ActivePathConflict` until the draining path is explicitly `Fail`ed. This matches the spec ("Failure requires an explicit `Fail` event from degraded or draining"; "at most one path is active"), i.e. drain is quiesce-new-data, not release.
- **Drain/fail/activate ordering vs committed semantics — holds.** `Fail` requires `Degraded` or `Draining` (`:262-263`); direct `Fail` from `Candidate`/`Active`/`Validating` is `InvalidTransition`. No path can skip the degraded/draining step to reach `Failed`.
- **Unrelated events cannot satisfy a path-specific invariant — holds.** Every event passes `checked(path, generation)` which binds the event to the exact `(PathId, PathGeneration)` record before inspecting state; a stale or foreign generation cannot pass.
- **Resource-limit/path-exists ordering deterministic, no mutation — holds.** `PathAdded` checks `PathExists` before `ResourceLimit` before `insert` (`:188-194`); both rejections are mutation-free and the order is fixed (`duplicate_path_reports_exists_before_capacity`).
- **`active_epoch` advances only on accepted activation — holds.** It is incremented only on the `Activate` success path (`:290`); all rejection paths return before it.

## Minor observation (not a defect)

`PacketFeedback::Ack` increments `successes` on any path whose `validation == Validated`, including one whose operational `state` is `Degraded`/`Draining`/`Failed`. Such a path cannot be re-activated anyway (`Activate` requires `state == Candidate`), so the counter cannot create delivery or activation evidence — it is inert. Recorded as a loose-end observation only; no claim is contradicted and no repair is warranted.

## Evidence

- `cargo test -p neko-carrier` on exact `8f93b93` covers the state-machine gates above (`ack_is_not_validation`, `pto_is_not_failure`, `old_generation_and_late_old_path_are_rejected`, `failover_gate_requires_validation_and_hysteresis`, `duplicate_path_reports_exists_before_capacity`, and the concurrent drain/fail tests).
- No code change; no defect found. No new `READY_LIVE` question. Release/governance state unchanged.
