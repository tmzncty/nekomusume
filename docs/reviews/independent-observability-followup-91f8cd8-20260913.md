# Independent observability stable-v1 follow-up — exact `91f8cd8`

Bounded reviewer-side source/contract challenge of the current `crates/neko-observe/src/lib.rs` after exact `ece67b85a23b62dfba6fe00f817c9badd975fad0`, reviewed on reachable current source tree `91f8cd8be1dd40c0f58a4857e47bbec2fe78ccde` against `docs/era4-observability-contract.md` and `schema/observability-event.v1.json`.

This is static independent review plus hosted-CI cross-evidence. It is not developer-local exact-tree CI, WAN evidence, security approval, RC, release, or production authorization.

## Verdict

The O2/O3 direction in `ece67b8` is useful but **not accepted as closed**. Three evidence-contract defects remain and must be repaired before expanding item-4 review farther.

### HIGH O4 — newly emitted stable-v1 events are schema-invalid

`schema/observability-event.v1.json` closes `data` with `additionalProperties=false`.

Current sequence-exhaustion `resource.limit_hit` emits:

- `resource = "event_sequence"`, but the stable v1 `resource` enum does not contain `event_sequence`;
- `limit = "u64_max"`, but `limit` is a nonnegative integer counter in the machine schema.

Current `diagnostic.events_dropped` emits `dropped_since_last`, but that field is not present in the closed stable-v1 `data` schema.

Therefore the new code can emit documents that contradict the repository's authoritative machine contract even though `scripts/check-observability-contract.sh` validates only schema structure and vocabulary presence, not these runtime-generated event shapes.

Required repair is bounded and already determined by the committed contract:

- every emitted event must be representable by `observability-event.v1.json`;
- do not weaken `additionalProperties=false` merely to hide the mismatch;
- the contract explicitly allows append-only v1 enum/value additions, so if sequence exhaustion remains represented as `resource.limit_hit`, a minimal semantic resource value such as `event_sequence` may be added append-only to the existing resource enum; `limit` itself must remain an integer (for this boundary, `u64::MAX` is representable as the intended configured limit);
- alternatively use an already-valid existing field/value only if its meaning is truthful; do not mislabel sequence exhaustion as an unrelated resource;
- remove `dropped_since_last` from emitted stable-v1 JSON unless the schema is intentionally extended append-only and the field has a stable documented meaning. The existing contract only requires current `dropped_total` and retained `oldest_sequence` for this event, so omission is the smaller closure unless a current committed requirement proves otherwise.

Add focused exact-shape regressions for both emitted events. Do not build a generalized logging/schema framework.

### HIGH O5 — drop diagnostic insertion undercounts real evictions

Stable v1 section 6 says: on overflow, evict oldest-first, increment `dropped_total`, advance `oldest_sequence`, and emit/coalesce `diagnostic.events_dropped`.

Current flow can overflow twice in one ordinary push:

1. `push` appends a normal event to an already-full ring; `push_inner(..., true)` evicts one retained event and increments `dropped_total`;
2. `flush_drop_report` then appends the drop diagnostic to the still-full ring via `push_inner(..., false)`, evicting a second retained event **without incrementing `dropped_total`**.

The diagnostic itself must not recursively count as a dropped event, but the ordinary retained event displaced to make room for that diagnostic is genuinely no longer retained. The stable contract does not permit silently losing that second eviction.

Repair must coalesce this self-induced overflow without recursion while accounting every ordinary retained event actually evicted. A small-capacity regression (including capacity 1 and >1) should prove that reported/health `dropped_total` equals the actual count of ordinary evidence no longer retained due to overflow.

### HIGH O6 — `oldest_sequence` is computed before the diagnostic's own eviction

`flush_drop_report` formats `oldest_sequence` from the current ring and only then calls `push_inner` for the diagnostic. If the ring is full, that call evicts the current front, so the diagnostic reports the **pre-insertion** floor rather than the retained floor after the diagnostic is present.

Stable v1 requires `diagnostic.events_dropped` to carry the retained sequence floor. Repair must compute/report the post-coalescing retained floor consistently with the final ring contents.

A focused regression should reconstruct the retained `sequence` values after overflow and assert that the diagnostic's `oldest_sequence` equals the actual oldest retained sequence, not the pre-diagnostic value.

## Accepted adjacent work

- `91f8cd8` negative `switch_margin` rejection is source-shape-correct against the accepted M4 positive-improvement meaning: it rejects negative margins through the existing invalid-limit path and does not invent a new numeric maximum. Hosted `stable checks` and `nightly decode fuzz smoke` for exact `91f8cd8` are green. Final developer-local provenance may be anchored on the next coherent source tree after the mandatory observability repair rather than creating provenance-only churn now.
- `a14cf47` positive/extreme margin overflow repair remains accepted.
- the dedicated CarrierManager/health/migration-back review at `5f1cb8d` remains useful; after `91f8cd8`, only the negative-margin constructor boundary needs re-close, not a full duplicate manager audit.

## Required closure

1. Repair O4/O5/O6 coherently in `neko-observe` plus the minimal stable-v1 schema adjustment only if needed for truthful sequence-exhaustion representation.
2. Focused tests must cover schema-valid event shapes, strict sequence exhaustion, exact eviction/drop accounting, post-coalescing retained floor, huge counter deltas, mixed datagram attribution, and no recursive diagnostic emission.
3. Run at least `cargo test -p neko-observe`; then on the final pushed source tree run developer-local `PYTHONDONTWRITEBYTECODE=1 bash scripts/check.sh`, `git diff --check`, and clean-tree confirmation. Persist sanitized exact-tree provenance.
4. Re-close the corrected observability producer independently, then resume the deep item-4 queue at FairScheduler/multistream/flow-control.

No decoder/crypto framing changes are required by these findings, so decode fuzz is not a new local requirement unless the repair unexpectedly touches those owners.

No release flag or live classification changes. `READY_LIVE: none` remains unchanged.