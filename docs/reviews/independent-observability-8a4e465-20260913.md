# Independent bounded observability producer review — exact `8a4e465`

Bounded reviewer-side source/contract challenge of `crates/neko-observe/src/lib.rs` against the stable v1 contract in `docs/era4-observability-contract.md`, at reachable exact `8a4e465926c4101a3cef61db2e7ef43b00dc99ee`.

This is a static independent review plus hosted-CI cross-evidence. It is **not** developer-local exact-tree CI, an exhaustive observability audit, a security approval, a WAN result, RC, release, or production authorization.

## Reviewed new source — `8a4e465` ACCEPT_WITH_BOUNDS

The new `record_datagrams` work bound is directionally correct. Per-kind event generation is now capped by the producer's configured event capacity rather than by arbitrarily large external `u64` counter deltas; un-emitted logical events are added to `dropped_total`, while ordinary ring eviction continues to add one drop per displaced retained event. The new huge-delta regression exercises the intended termination boundary.

GitHub-hosted cross-evidence for exact `8a4e465` is green (`stable checks` and `nightly decode fuzz smoke`, run `34713721724`). This does not replace the repository-required developer-local clean exact-tree gate/provenance for the pushed source anchor.

## MEDIUM O2 — event sequence saturation violates the stable v1 ordering contract

The stable contract requires events to be ordered by **strictly increasing** `sequence`, and separately requires metric/counter saturation to emit `resource.limit_hit` rather than silently wrap/saturate.

Current `Producer::push` assigns `sequence = self.next_sequence` and then advances with `self.next_sequence = self.next_sequence.saturating_add(1)`. Once `next_sequence` reaches `u64::MAX`, later pushes continue emitting `sequence == u64::MAX`, creating duplicate sequence values. That contradicts the stable v1 ordering contract even though the boundary is extremely large.

This is mechanically testable without changing architecture or choosing a new policy. Add a focused unit regression using module-private/test-only state setup near `u64::MAX` and prove that the producer never emits two events with the same sequence. The minimal repair must fail closed at exhaustion and satisfy the existing stable saturation contract: one bounded `resource.limit_hit` indication for sequence/event-buffer exhaustion before further ordinary events can violate ordering, with no numeric wrap and no duplicate sequence. Do not redesign the event schema or widen the sequence type.

## MEDIUM O3 — ring eviction does not emit/coalesce `diagnostic.events_dropped`

Stable v1 §6 says that when the bounded producer overflows it evicts oldest-first, increments `dropped_total`, advances the retained sequence floor, **and emits/coalesces `diagnostic.events_dropped`**. The schema already includes that event and its `dropped_total` / `oldest_sequence` fields.

Current `Producer::push` evicts oldest-first and increments `dropped_total`, but then only appends the requested event. There is no producer path in this owner that emits or coalesces `diagnostic.events_dropped` for its own ring eviction. This loses the in-band evidence required to distinguish a complete retained slice from one with evicted history.

Add a focused small-capacity regression (for example capacity 2–3) that forces overflow and verifies all of the existing committed requirements together: retained `sequence` values stay strictly increasing, `dropped_total` reflects eviction, retained sequence floor advances, and a bounded/coalesced `diagnostic.events_dropped` record reports the drop/floor without recursive unbounded self-emission. The exact coalescing implementation may be the smallest shape consistent with the stable v1 contract; do not create a generalized logging framework.

## Boundaries / exclusions

- The prior O1 mixed datagram-drop attribution defect is closed by exact `8f93b93`; do not reopen it.
- The `8a4e465` delta-work repair does not change transport or Session semantics.
- Caller-supplied observation time is documented as a caller monotonicity contract; this review does not manufacture a producer-side timestamp policy absent a concrete contradictory caller.
- This review does not invent capacity values, retention policy, D019 semantics, signing/SBOM policy, or live/VPS work.
- `READY_LIVE: none` remains unchanged by these findings.

## Required closure order

1. Persist developer-local exact-tree provenance for current pushed source if not already superseded by the O2/O3 repair tree.
2. Repair O2/O3 with focused regressions on one coherent reachable source commit when practical.
3. Run `cargo test -p neko-observe`, then the full developer-local exact-tree `PYTHONDONTWRITEBYTECODE=1 bash scripts/check.sh`, `git diff --check`, and clean-tree confirmation; persist sanitized provenance.
4. Re-review the corrected producer against the stable v1 buffer/sequence contract, then continue the pre-authorized deep item-4 queue.