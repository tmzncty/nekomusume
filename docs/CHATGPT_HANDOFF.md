# ChatGPT reviewer handoff — repair observability buffer contract, then continue deep item-4 review

## Reviewed repository truth

- Default branch source head before this reviewer pass: exact `8a4e465926c4101a3cef61db2e7ef43b00dc99ee` (`fix(observe): bound datagram event emission by ring capacity`).
- This reviewer pass adds exact `9ad927c608f90172ea971c5271a50b9e1cd25513` (`docs(review): challenge observability buffer contract at 8a4e465`) as reviewer-owned evidence only; it does not change runtime behavior.
- The previous handoff was stale at `e3054c9` and still described the no-sent ACK boundary as open. That boundary is now closed by reachable source exact `02b6eaae573187aab2329398e95d69b5808296d7` and the superseding independent review exact `7a0c40034099b939259b12a09fc22192f45bf788`.
- Reachable exact `0831443ecb221a24686c9046c88eaaf7af219996` records a dedicated independent bounded `CarrierState` review with no concrete defect in its stated generation/validation/hysteresis/single-active scope.
- The original observability mixed-drop attribution defect is closed by reachable source exact `8f93b931b42e486882980f44813660ba6edb4267`.
- Exact `8a4e465` then bounds `record_datagrams` work by ring capacity rather than external `u64` counter magnitude. GitHub-hosted `stable checks` and `nightly decode fuzz smoke` for exact `8a4e465` are green; hosted CI is extra cross-evidence only.
- No VPS/WAN experiment occurred in this new sequence. No core Session/Carrier/crypto/wire architecture or release flag changed.
- Authoritative governance remains: item 3 incomplete, item 4 incomplete, `RELEASE_CANDIDATE=false`, `PRODUCTION_READY=false`, `FREEZE=false`, `RELEASED=false`, D019 policy-blocked, and `READY_LIVE: none`.

This pass re-read the required repository truth surfaces (`README.md`, `AGENTS.md`, `ROADMAP.md`, `IMPLEMENTATION_PLAN.md`, `SECURITY.md`, `docs/status.md`, standing VPS authorization, VPS-rental priority, decisions/architecture, Session v0, release packet), the recent source/review commits, the stable Era-4 observability contract/schema, and current `neko-observe`/Carrier source relevant to the queue.

## Reviewer verdict on completed slices

### Reliable UDP R1 — CLOSED / ACCEPT

Exact `02b6eaa` completes the previously missed `largest_sent == None` boundary: any nonempty ACK on a recovery state that has never sent a packet is rejected atomically, alongside the earlier `largest > largest_sent` repair. Already-retired ACK numbers `<= largest_sent` remain allowed and `largest == largest_sent` remains valid. Exact `7a0c400` supersedes the earlier false closure note and records the clean exact-tree developer gate. Do not reopen this family without a new concrete contradiction.

### `CarrierState` review — CLOSED / NO FINDING

Exact `0831443` independently challenged path generation, validation domain, hysteresis, single-active ownership, drain/fail/activate transitions and deterministic error precedence on source exact `8f93b93`. No repairable contradiction was found. The minor fact that packet ACKs may increment success counters on a validated but non-candidate path is inert under current activation state gates and is not a current defect.

### `8a4e465` datagram work-bound repair — ACCEPT_WITH_BOUNDS

The source change correctly removes counter-delta-proportional loops from `record_datagrams`: each event class emits at most `capacity` records and accounts un-emitted logical events in `dropped_total`. The new huge-delta regression exercises this bound. Hosted stable/fuzz checks are green.

Do not spend a separate full provenance cycle on exact `8a4e465` if it is immediately superseded by the required observability repair below; the final coherent corrected source tree is the developer-local exact-tree gate/provenance anchor that matters.

## Open findings — observability stable-v1 evidence correctness

Independent reviewer note: `docs/reviews/independent-observability-8a4e465-20260913.md`.

### MEDIUM O2 — `sequence` saturation can violate strict ordering

Owner: `crates/neko-observe/src/lib.rs`, `Producer::push`.

Stable Era-4 v1 requires retained events to be ordered by **strictly increasing** `sequence`. Current code assigns `sequence = next_sequence` and advances with `saturating_add(1)`. After reaching `u64::MAX`, later events can reuse `u64::MAX`, producing duplicate sequence values. Stable v1 also says saturation must be represented with `resource.limit_hit`, not silently wrap/saturate.

This is an explicit contract contradiction, even though the boundary is enormous. The answer is already bounded by committed semantics; no maintainer policy decision is required.

### MEDIUM O3 — ring eviction omits required `diagnostic.events_dropped`

Owner: `crates/neko-observe/src/lib.rs`, bounded event-buffer path.

Stable Era-4 v1 §6 requires overflow to evict oldest-first, increment `dropped_total`, advance retained sequence floor, **and emit/coalesce `diagnostic.events_dropped`**. The current `push` implementation evicts and increments the counter but does not emit/coalesce the in-band drop/gap event, even though the v1 schema already defines `diagnostic.events_dropped`, `dropped_total`, and `oldest_sequence`.

This is evidence-integrity behavior, not transport semantics. Minimal implementation shape is left to the coding agent under `AGENTS.md` proposal authority, but it must avoid recursive/unbounded self-emission.

## Continuous execution queue

The external coding agent is explicitly pre-authorized to continue through the queue below without waiting for another reviewer whenever the next slice dependencies are satisfied. Reviewer cadence is not a work-ticket cadence.

### MUST_EXECUTE_LOCAL 1 — close O2 + O3 coherently

Primary owner: `crates/neko-observe/src/lib.rs` and its unit tests. Applicable authority: `docs/era4-observability-contract.md` and `schema/observability-event.v1.json`.

Prefer one coherent observability-buffer repair when practical.

#### O2 regression / repair contract

- Add a focused test that places test-only producer state near `u64::MAX` and proves no two emitted events ever share a sequence.
- Do not wrap and do not widen/change the v1 schema.
- At exhaustion, follow the existing stable saturation rule with a bounded `resource.limit_hit` indication; subsequent ordinary pushes must not create duplicate sequence values.
- Preserve bounded work and `dropped_total` accounting.
- Do not turn this into a generalized logging framework.

#### O3 regression / repair contract

- Force ring overflow with a tiny valid capacity.
- Verify oldest-first retention and strictly increasing retained sequences.
- Verify `dropped_total`/retained floor truth.
- Require one bounded/coalesced `diagnostic.events_dropped` representation with current `dropped_total` and `oldest_sequence` according to the existing schema/contract.
- The drop diagnostic must not recursively create unbounded work or an endless self-eviction loop.
- Do not invent new event vocabulary, capacities, or retention policy.

Also keep the already-correct O1 mixed datagram drop attribution and the `8a4e465` delta-work bound green.

Focused validation at minimum:

```bash
cargo test -p neko-observe
```

No wire/parser/crypto decoder changes are expected, so no decode fuzz is required for this slice.

Commit/push the coherent repair, then continue directly to LOCAL 2.

### MUST_EXECUTE_LOCAL 2 — exact-tree local closure/provenance on corrected observer source

On the exact pushed source commit from LOCAL 1, from a clean detached checkout/worktree:

```bash
cargo test -p neko-observe
PYTHONDONTWRITEBYTECODE=1 bash scripts/check.sh
git diff --check
git status --short
```

Persist concise sanitized developer provenance: reachable exact SHA, commands, UTC start/end, exit codes, OS/arch, stable Rust version, clean initial/final tree. Hosted CI remains optional cross-evidence and is never a wait condition.

When green, immediately continue to REVIEW 3.

### REVIEW_LOCAL 3 — independently re-close the observability producer

Use the corrected reachable source anchor and independently challenge at least:

- strict sequence ordering and exhaustion;
- event-buffer eviction/drop-gap evidence;
- bounded event generation for huge counter deltas;
- mixed datagram attribution;
- `record_health`, `record_switch`, recovery/PTO projections;
- scheduler/resource high-water projection;
- secret-safe correlation and fixed-vocabulary output;
- schema-valid event/data field shapes actually emitted by current producer methods.

Concrete existing-semantics defect -> smallest repair/regression -> push -> exact-tree local gate/provenance before continuing. No finding -> bounded independent review note with precise exclusions. This remains item-4 support, not a full audit.

### REVIEW_LOCAL 4 — Concurrent Carrier Manager / health / migration-back

Primary owner: current `ConcurrentCarrierManager` implementation/tests in `crates/neko-carrier/src/lib.rs` plus D064 / directly applicable manager specs.

Challenge:

- single-active ownership and active-epoch monotonicity;
- authenticated readiness is not packet feedback or Session delivery;
- warm/cold classification;
- failure -> uncertain ownership before replay;
- replay/dedup range ownership and retained-byte bounds;
- failed/draining/warm transition legality;
- health-state transitions and selection hysteresis;
- migration-back validation/generation/hold/margin gates;
- bounded switch-event history and deterministic error mutation boundaries.

Do not change the D064 constants/policy values or invent a new manager architecture. Existing-semantics defect -> smallest repair/regression. No finding -> independent bounded note.

### REVIEW_LOCAL 5 — FairScheduler + multistream + flow-control accounting

Primary owner: scheduler/flow portions of `crates/neko-carrier/src/lib.rs` and directly related Session runtime tests/specs.

Challenge:

- stream/session queue limits before allocation/enqueue;
- queued-byte counters on enqueue/dequeue/close/reset/error;
- fairness / starvation guard behavior;
- interactive-vs-bulk priority without permanent bulk starvation;
- stream-close removal and queue-byte release;
- arithmetic saturation/overflow cannot admit excess state;
- deterministic behavior at exact limit boundaries.

No new scheduling policy or numeric limits. Repair only contradictions already determined by committed semantics.

### REVIEW_LOCAL 6 — Carrier adapter close/error/resource semantics

Primary owners: Memory/UDP/TCP adapter code and direct adapter tests.

Challenge:

- local close vs peer close vs would-block semantics;
- queued-data drain after close where current contract says so;
- message/datagram/frame boundary preservation;
- truncation/oversize atomicity;
- queue/buffer accounting and release;
- poison/OS error mapping without cross-layer evidence promotion;
- no adapter observation becomes Session delivery/path validation by accident.

Bounded no-finding review is acceptable; no generic adapter rewrite.

### REVIEW_LOCAL 7 — `SessionRuntime` lifecycle/resource/DeliveryAck accounting

Primary owner: `crates/neko-session/src/lib.rs` runtime/process-message surfaces and current specs/tests.

Challenge:

- stream/session windows and queue/total-byte accounting;
- outbound/inbound offset progression;
- DeliveryAck bounds and release accounting;
- duplicate receive/dedup versus logical delivery evidence;
- close/idle/deadline/cancel terminalization and cleanup;
- invalid/replayed/unknown-stream transitions are atomic;
- process message decode lengths and state admission remain bounded;
- close/reset cannot leak queued/window accounting.

Do not merge Session delivery with carrier packet ACK and do not redesign the wire/process codec.

### REVIEW_LOCAL 8 — package / reproducibility / operator implementation

Primary owners: `scripts/release/` build/check/smoke/package paths and linked package tests/evidence.

Review implementation, not merely old notes:

- clean-source refusal before output mutation;
- archive path/type/root/mode/checksum rejection ordering;
- reproducibility inputs and produced identity;
- install/upgrade/rollback state boundary;
- lifecycle/readiness/shutdown cleanup semantics;
- no secret identity bytes in evidence/logging;
- script failure leaves truthful partial/cleanup evidence.

Do not invent signing, key-custody, SBOM or publication policy. Those remain separate gates.

### REVIEW_LOCAL 9 — dependency/build surface, independent pass

Developer factual dependency note exists, but a dedicated independent challenge remains useful.

Inspect Cargo manifests, committed lockfile, features/build scripts/native hooks and workspace unsafe inheritance. Look for contradictions such as an unreviewed direct security-sensitive dependency, feature drift, unlocked build path, unexpected build script/native link behavior, or a crate not inheriting intended workspace lint policy.

Do not add a dependency scanner framework, upgrade dependencies, or choose signing/SBOM policy merely to create work.

### REVIEW_LOCAL 10 — cross-platform CLI/process semantics

Audit deterministic CLI/process tests and platform-dependent code for concrete OS-artifact assumptions beyond the already-closed EOF/RST seam:

- socket EOF/reset/error distinctions;
- file permission / rename / unlink semantics;
- signal/shutdown assumptions;
- process exit/status handling;
- listener release/rebind assumptions;
- conditional platform APIs and error mapping.

Do not weaken assertions merely to make multiple OSes pass. Preserve the semantic invariant and use `cfg`/bounded alternate terminal outcome only when current contract justifies it.

### REVIEW_LOCAL 11 — CLI machine/human/exit-code contract

Challenge the executable boundary end to end:

- JSON success only after actual success/admission;
- nonzero exits for rejected/invalid/incomplete operations;
- exact human probe outputs remain separate from machine authority;
- stderr never contains secrets/plaintext/arbitrary peer text;
- partial/blocked benchmark evidence cannot emit comparative success summary;
- READY/DRAINING/STOPPED and structured events cannot claim stages not reached.

Concrete contradiction -> repair/regression; no finding -> bounded note.

### REVIEW_LOCAL 12 — algorithmic resource boundedness, non-pressure

Static/deterministic review only; this is **not** capacity benchmarking and sets no policy numbers.

Challenge attacker/caller-controlled loops and collections across core crates for work proportional to unchecked numeric magnitude rather than bounded retained state, clone-before-bound patterns, quadratic paths under allowed maxima, retry loops without bounded progress, and counters that can overflow into incorrect evidence/state.

The `8a4e465` datagram-delta loop is the model for a legitimate finding. Do not convert this into speculative micro-optimization.

### REVIEW_SUPPORT 13 — factual reconciliation checkpoint

After roughly 3–4 additional coherent review/repair slices, reconcile:

- `docs/release-security-review-packet.md`;
- `docs/reviews/release-item4-subgates-20260909.md`;
- `docs/status.md` only if capability/governance status actually changes;
- relevant current review indexes.

Index new reachable independent evidence and concrete repairs without claiming item 4 complete merely because more sub-surfaces are reviewed.

Then continue remaining pre-authorized slices; do not pause just to wait for reviewer cadence.

## Queue sizing / continuation rule

This is intentionally a deep queue. Do not collapse it after each small commit. If the coding/review agent completes slices in 10–30 minutes with clean gates and no rising defect rate, continue through the next dependency-safe slice and retain the remaining queue.

A bounded no-finding independent review of a previously unreviewed implemented core surface is useful item-4 work; it is not filler. Conversely, do not create generalized checker/schema/framework/doc churn solely to stay busy.

Only unresolved BLOCKER/HIGH, a core architecture/policy decision, destructive/canonical migration, action outside standing authorization, production/third-party action, new credential/server permission, maintainer-valued capacity/pressure conditions, repository breakage, or actual runtime/tool-budget exhaustion stops continuous execution.

## Live / policy boundary

- item 3 remains incomplete;
- item 4 remains incomplete;
- `RELEASE_CANDIDATE=false`;
- `PRODUCTION_READY=false`;
- `FREEZE=false`;
- `RELEASED=false`;
- D019 remains a maintainer policy/value decision;
- RSEC-001 representative adversarial-load/capacity suitability remains unestablished and is not silently converted into a local pressure test;
- signing/key custody/SBOM/publication trust and previous-frozen-release interoperability remain separate gates/dependencies;
- `READY_LIVE: none` remains authoritative.

Standing VPS authorization remains valid, but no new dependency-ready live question is created by the current local observability/review work. Do not mechanically rerun HY2, repeated warm failover, periodic/soak, package lifecycle, migration-back, endpoint migration, key update, IPv6, PLPMTUD, or Experimental Track work.
