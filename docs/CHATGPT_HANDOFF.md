# ChatGPT reviewer handoff — close observer contract, validate CarrierManager margin domain, then continue deep item-4 review

## Reviewed repository truth

- Latest developer source/test head reviewed in this pass: exact `a14cf471b6ef97442f36419ee8df3648037a5ef4` (`fix(carrier): saturate switch-margin addition against i64 overflow`).
- This reviewer pass adds reviewer-only exact `e783ee51fe569405ee5db155cf982dbe67db3913` (`docs(review): challenge CarrierManager negative margin domain`). It does not change runtime behavior and is not a tested-tree anchor.
- The previous handoff at `ae2a28e` remains materially correct about two **still-open** observability stable-v1 findings: O2 sequence exhaustion and O3 required `diagnostic.events_dropped`. Current source exact `a14cf47` still uses `next_sequence.saturating_add(1)` and still evicts/increments `dropped_total` without emitting/coalescing the required drop-gap event.
- The coding agent therefore advanced into Carrier Manager work before closing the earlier `MUST_EXECUTE_LOCAL` observer repair. That work is useful and is retained, but execution order must now return to O2/O3 first.
- Completed source/review work since the high-throughput queue reopened includes:
  - exact `531c82d`: rejects peer ACK with `largest > largest_sent` atomically;
  - exact `02b6eaa`: rejects any nonempty ACK when no packet has ever been sent;
  - exact `8f93b93`: fixes mixed datagram drop attribution (`queue_dropped` subset vs terminal remainder);
  - exact `8a4e465`: bounds datagram event emission work by ring capacity rather than external `u64` counter magnitude;
  - exact `0831443`: dedicated bounded `CarrierState` review, no concrete finding in its generation/validation/hysteresis/single-active scope;
  - exact `a14cf47`: replaces both CarrierManager score-margin plain additions with `saturating_add` and adds extreme-positive-margin regressions.
- GitHub-hosted `stable checks` and `nightly decode fuzz smoke` for exact `a14cf47` are both green. Hosted CI is extra cross-evidence only; it does not replace developer-local exact-tree provenance.
- No VPS/WAN experiment occurred in this sequence. No release flag changed.
- Governance remains: item 3 incomplete, item 4 incomplete, `RELEASE_CANDIDATE=false`, `PRODUCTION_READY=false`, `FREEZE=false`, `RELEASED=false`, D019 policy-blocked, and `READY_LIVE: none`.

## Reviewer verdict on `a14cf47`

### Positive/extreme overflow repair — ACCEPT_WITH_BOUNDS

The source change is correct for the defect it claims to fix. `ManagerLimits::switch_margin` is an unbounded caller-supplied `i64`; adding `i64::MAX` to a bounded score with plain `+` could panic in debug or wrap in release and accidentally lower the threshold. `saturating_add` prevents both failures, and the new tests cover both `migrate_back_to_udp` and `choose` with an extreme positive margin.

Do not revert this repair. A later coherent developer-local full gate on the next corrected source tree may serve as the local closure anchor; there is no need to stop the queue solely to create a provenance-only commit for `a14cf47` if it is immediately superseded by the mandatory observer repair below.

### New MEDIUM candidate C1 — negative `switch_margin` semantic domain

Reviewer note: `docs/reviews/independent-carrier-margin-a14cf47-20260913.md`.

`CarrierManager::new` currently rejects `min_hold_events == 0` and `max_paths == 0`, but accepts every `i64` `switch_margin`, including negative values. The accepted M3 ADR requires voluntary promotion/migration-back to satisfy a **positive improvement margin** over the active path. A negative margin inverts the gate and can admit a strictly worse healthy candidate after other gates are satisfied.

This is separate from numeric overflow: saturating arithmetic is necessary but not sufficient. Before repair, the coding agent must prove the behavior with focused tests. If reproduced and no committed contract explicitly gives negative margin a distinct meaning, reject negative margin at construction via the existing invalid-limit path. Do not invent a new maximum or retune policy values.

## Execution order — continuous, no reviewer wait between dependency-safe slices

### MUST_EXECUTE_LOCAL 1 — close observability O2 + O3 coherently

Primary owner: `crates/neko-observe/src/lib.rs`; authority: `docs/era4-observability-contract.md` + `schema/observability-event.v1.json`.

#### O2 — strict sequence exhaustion

Stable v1 requires strictly increasing `sequence` and says saturation must be represented by `resource.limit_hit`, not silently reused. Current `Producer::push` still assigns `next_sequence` then advances with `saturating_add(1)`, so after `u64::MAX` later pushes can reuse the same sequence.

Required regression/repair:

- place test-only producer sequence state near `u64::MAX`;
- prove retained emitted events never share a sequence;
- do not wrap or widen/change v1 schema;
- exhaustion must be represented in a bounded way using existing `resource.limit_hit` vocabulary;
- subsequent ordinary pushes must not create duplicate sequence values;
- preserve bounded work and truthful `dropped_total`.

#### O3 — required in-band buffer-gap evidence

Stable v1 §6 requires overflow to evict oldest-first, increment `dropped_total`, advance retained sequence floor, **and emit/coalesce `diagnostic.events_dropped`**. Current `push` still evicts and increments only.

Required regression/repair:

- tiny-capacity overflow;
- oldest-first retention and strict retained sequence order;
- truthful `dropped_total` and retained floor;
- one bounded/coalesced `diagnostic.events_dropped` representation carrying current `dropped_total` and `oldest_sequence` per existing schema;
- no recursive/unbounded self-emission or endless self-eviction loop;
- no new event vocabulary, capacities, or retention policy.

Keep mixed-drop attribution and huge-delta work bound green. Focused minimum:

```bash
cargo test -p neko-observe
```

No decoder/framing change => no decode fuzz requirement for this repair itself.

Commit/push one coherent observer repair and immediately continue to LOCAL 2.

### MUST_EXECUTE_LOCAL 2 — validate/repair C1 negative margin

Owners: `crates/neko-carrier/src/lib.rs` `ManagerLimits`, `CarrierManager::new`, `choose`, `migrate_back_to_udp`.

1. Add focused regressions showing whether `switch_margin < 0` currently permits a strictly worse healthy candidate to replace/migrate against a better active path once the existing hold/validation/generation gates are satisfied.
2. Cover both voluntary `choose` and `migrate_back_to_udp`.
3. If reproduced and no current committed spec defines negative margin as meaningful, reject negative margin in `CarrierManager::new` using existing `FlowError::InvalidLimit`.
4. Rejection must occur before any path/sample state exists.
5. Keep zero/nonnegative margins and the `i64::MAX` overflow regressions green.
6. Do not invent a new positive upper bound and do not retune the D064 margin policy.

Then continue directly to LOCAL 3.

### MUST_EXECUTE_LOCAL 3 — coherent exact-tree local closure/provenance

On the exact pushed source tree containing the observer repair plus any confirmed C1 repair:

```bash
cargo test -p neko-observe
cargo test -p neko-carrier
PYTHONDONTWRITEBYTECODE=1 bash scripts/check.sh
git diff --check
git status --short
```

Persist concise sanitized developer provenance: reachable exact SHA, commands, UTC start/end, exit codes, OS/arch, stable Rust version, clean initial/final tree. GitHub-hosted CI remains optional cross-evidence and never a wait condition.

### REVIEW_LOCAL 4 — independently re-close observability producer

On the corrected reachable source anchor challenge:

- strict sequence ordering/exhaustion;
- overflow eviction/drop-gap evidence;
- bounded huge counter deltas;
- mixed datagram attribution;
- health/switch/recovery/PTO projections;
- scheduler/resource high-water projection;
- secret-safe correlation and fixed vocabulary;
- emitted schema field shapes.

Concrete defect -> smallest repair/regression -> push -> exact-tree gate. No finding -> bounded independent note.

### REVIEW_LOCAL 5 — complete Concurrent Carrier Manager / health / migration-back review

`CarrierState` is already independently reviewed; this slice is the larger manager/health/migration layer.

Challenge:

- single-active ownership and active-generation/epoch behavior;
- warm candidate readiness dimensions and duplicate observation handling;
- fail -> pending -> warm/cold promotion ordering;
- failed/draining/warm transition legality;
- health-state transitions and deterministic selection;
- migration-back validation/generation/hold/margin gates including extreme limits;
- bounded switch/readiness history and mutation atomicity;
- arithmetic overflow/saturation cannot become a fail-open selection gate.

Do not change D064 policy constants or invent a new manager architecture. Existing-semantics defect -> smallest repair/regression. No finding -> independent bounded note.

### REVIEW_LOCAL 6 — FairScheduler + multistream + flow-control accounting

Challenge queue limits before enqueue/allocation, session/stream byte accounting, dequeue/close/reset release, fairness/starvation guard, priority without permanent starvation, arithmetic boundary atomicity, and exact-limit determinism. No new scheduling policy or numeric limits.

### REVIEW_LOCAL 7 — Carrier adapter close/error/resource semantics

Review Memory/UDP/TCP adapters for local/peer close, would-block, queued-data drain, message/frame boundaries, truncation/oversize atomicity, buffer release, poison/OS mapping, and no promotion of adapter observations into Session delivery/path validation.

### REVIEW_LOCAL 8 — `SessionRuntime` lifecycle/resource/DeliveryAck accounting

Challenge stream/session windows, queue/total bytes, offset progression, DeliveryAck bounds/release, duplicate receive/dedup, close/idle/deadline/cancel cleanup, atomic invalid transitions, process-message bounded decode/admission, and reset/close accounting release. Do not merge Session delivery with packet ACK or redesign the codec.

### REVIEW_LOCAL 9 — package / reproducibility / operator implementation

Review current `scripts/release/` implementation: clean-source refusal before mutation, archive path/type/root/mode/checksum rejection ordering, reproducibility identity, A/B/A state boundary, lifecycle cleanup, secret-safe evidence, truthful partial failure/cleanup. Do not invent signing/key-custody/SBOM/publication policy.

### REVIEW_LOCAL 10 — dependency/build independent pass

Inspect Cargo manifests/lock/features/build scripts/native hooks and workspace unsafe inheritance. Look for executable graph contradiction, unexpected build/native behavior, feature drift, or non-inherited safety policy. No scanner framework, dependency upgrade, signing or SBOM policy.

### REVIEW_LOCAL 11 — cross-platform CLI/process semantics

Challenge remaining OS-artifact assumptions: socket terminal states, filesystem permissions/rename/unlink, signals/shutdown, process exit/status, listener release/rebind, platform APIs/error mapping. Do not weaken invariants merely for portability.

### REVIEW_LOCAL 12 — CLI machine/human/exit-code contract

Challenge success JSON timing, nonzero rejection exits, exact human probe output vs machine authority, secret-safe stderr, benchmark partial/blocked semantics, and READY/DRAINING/STOPPED stage truth.

### REVIEW_LOCAL 13 — algorithmic resource boundedness, static/deterministic only

Continue looking for unchecked numeric-magnitude loops, clone-before-bound, retry loops without bounded progress, collection growth beyond declared retained state, and arithmetic overflow that changes evidence/state. This is not capacity pressure testing and sets no new policy values.

### REVIEW_SUPPORT 14 — factual reconciliation checkpoint

After roughly 3–4 additional coherent source/review slices, reconcile `docs/release-security-review-packet.md`, `docs/reviews/release-item4-subgates-20260909.md`, relevant review indexes, and `docs/status.md` only if capability/governance status truly changes. Index reachable evidence without claiming item 4 complete automatically.

## Queue discipline

This remains a deep queue. Do not collapse it after each small commit. A bounded no-finding independent review of a previously unreviewed implemented core surface is useful item-4 work. Do not create generalized checker/schema/framework/docs churn solely to stay busy.

Only unresolved BLOCKER/HIGH, a core architecture/policy decision, destructive/canonical migration, action outside standing authorization, production/third-party action, new credential/server permission, maintainer-valued capacity/pressure conditions, repository breakage, or actual runtime/tool-budget exhaustion stops continuous execution.

## Live / policy boundary

- item 3 incomplete;
- item 4 incomplete;
- `RELEASE_CANDIDATE=false`;
- `PRODUCTION_READY=false`;
- `FREEZE=false`;
- `RELEASED=false`;
- D019 remains maintainer policy/value;
- RSEC-001 representative adversarial-load/capacity suitability remains unestablished and is not converted into a local pressure test;
- signing/key custody/SBOM/publication trust and previous-frozen-release interoperability remain separate gates/dependencies;
- `READY_LIVE: none` remains authoritative.

Standing VPS authorization remains valid, but current work creates no new dependency-ready live question. Do not mechanically rerun HY2, repeated warm failover, periodic/soak, package lifecycle, migration-back, endpoint migration, key update, IPv6, PLPMTUD, or Experimental Track work.
