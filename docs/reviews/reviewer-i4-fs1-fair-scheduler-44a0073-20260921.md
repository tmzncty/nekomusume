# Independent bounded I4-FS1 FairScheduler re-challenge — current owner at `44a0073`

**Repository anchor before this note:** reachable `main` exact `44a0073a4eacc8d9a78790c7b9ff37909de8e532`.
**Executable source/test anchor:** exact `8cbd9af39a600f4ddd5fbc50208791e8584c6d74`; commits after it through the repository anchor are review/evidence/navigation docs only.
**Owner inspected:** `FairScheduler`, `FlowLimits`, `FlowStream`, `StreamPriority`, `INTERACTIVE_BURST`, and current inline scheduler regressions in `crates/neko-carrier/src/lib.rs`; current owner blob `822707a331a360a939caa8c06c22a27d72fa30df`.

This is an independent bounded source/evidence challenge for release item 4. It is not a WAN/performance result, security approval, release decision, or reviewer-local test run.

## Inputs reconciled

- current `README.md`, `ROADMAP.md`, `IMPLEMENTATION_PLAN.md`, `SECURITY.md`, `docs/status.md`, `docs/decisions.md`, `docs/carrier-architecture.md`, `docs/specs/nekomusume-session-v0.md`, and `docs/release-security-review-packet.md`;
- developer I4-FS1 notes at `dab0be3`, `c3fe145`, and wording correction `9adf1d2`;
- prior independent scheduler review `docs/reviews/independent-fair-scheduler-91f8cd8-20260913.md` at exact `91f8cd8be1dd40c0f58a4857e47bbec2fe78ccde`;
- exact-current scheduler owner and inline regressions.

The current FairScheduler implementation range remains semantically unchanged from the prior exact-`91f8cd8` reviewed owner. The wider `neko-carrier` file has grown substantially since that review, so the current owner was nevertheless re-read rather than inheriting the old note mechanically.

## Challenged invariants

### 1. `open` idempotence and limit rejection before mutation — holds

`open(id, priority)` returns `Ok(())` immediately for an existing stream and therefore neither duplicates `order` nor consumes another stream slot. For a new stream, `streams.len() >= max_streams` is checked before insertion or `order.push`. Re-opening an existing ID with a different priority does not reconfigure the existing stream; no current spec/ADR defines `open` as a priority-update API, so inventing that behavior would be a policy/API change rather than a repair.

### 2. Enqueue accounting and atomic rejection — holds

`enqueue` rejects empty data and unknown streams first, computes per-stream and session totals with `checked_add`, checks both limits before taking the mutable stream reference, and only then pushes the record and commits `queued_bytes`/`session_bytes`. Therefore stream/session-limit and integer-overflow rejection cannot partially enqueue or advance counters. Current `flow_limits_are_atomic` is a direct mutation-sensitive regression for this boundary.

### 3. Dequeue accounting, cursor rotation, and drained-stream behavior — holds

`next_frame` scans from the current cursor, only considers streams with `queued_bytes > 0`, advances the cursor to one slot after the selected stream, pops exactly one queued record, and subtracts exactly that record length from both stream and session counters. Drained streams remain present in `streams`/`order` but are scheduler-inert and are skipped; there is no `close`/terminal/remove API in this owner. The developer wording correction at `9adf1d2` is therefore factually correct.

The monotonic occupied-stream-slot behavior is bounded by `max_streams`; adding teardown semantics would be a feature/API decision, not a correctness repair for current accounting.

### 4. Interactive burst is bounded while bulk is queued; both classes progress — holds

While `consecutive_interactive < INTERACTIVE_BURST`, Interactive is preferred. At the configured burst boundary, Bulk is preferred. If the preferred class is absent, `find(None)` falls back to any queued stream, so an empty preferred class cannot stall the scheduler. A Bulk dequeue resets the interactive counter; an Interactive dequeue advances it. Thus, whenever Bulk is actually queued, no more than the configured interactive burst can run before Bulk is preferred, while absence of Bulk does not artificially stall Interactive work.

The current deterministic regression `interactive_burst_is_bounded_and_order_is_repeatable` pins the representative `I,I,I,B,I,B,B,B` sequence and repeats it to check deterministic ordering. `bulk_does_not_starve_interactive` covers the opposite-pressure direction.

### 5. Empty/drained streams do not corrupt cursor/accounting — holds

The search walks at most `order.len()` entries from `cursor`, skips inert streams, and returns `None` only when no queued stream is found. Because `order` is append-only and every ID in it is inserted into `streams` in the same `open` operation, the `streams.get` lookup does not create a reachable hole through public APIs. No current source path can orphan queued byte accounting from its stream entry.

## Result

**No concrete defect found in I4-FS1.** Current FairScheduler semantics remain internally consistent for idempotent open, bounded stream/session accounting, reject-before-mutate behavior, cursor/rotation across inert streams, deterministic Interactive/Bulk scheduling, and the absence of a scheduler close/terminal API.

No code repair, new policy value, scheduler redesign, fuzz run, or live/WAN work is warranted by this slice.

## Evidence boundary

- This reviewer pass performed source/spec/evidence reasoning only; it does **not** claim reviewer-local command or test execution.
- The prior exact-`91f8cd8` note records its own `cargo test -p neko-carrier` execution and remains historical independent evidence for the unchanged scheduler owner.
- The exact `8cbd9af` clean exact-tree gate remains developer-reported provenance recorded elsewhere; it is not reclassified as reviewer-local or hosted-CI evidence here.
- I4-FS2 remains separate: Session/runtime multi-stream and session+stream flow-control integration, delivery ownership, and cross-stream isolation are not promoted by this scheduler-only review.
- Carrier adapter close/resource semantics, CLI/process portability, machine/human output contracts, algorithmic boundedness, release item 3, and final release authority are excluded.

**READY_LIVE: none.** No new code/instrumentation/hypothesis/path condition creates an unresolved real-network question.