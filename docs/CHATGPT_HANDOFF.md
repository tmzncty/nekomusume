# ChatGPT reviewer handoff — repair observability stable-v1 HIGH, then continue deep item-4 review

## Reviewed repository truth

- Latest developer source/test head reviewed: exact `91f8cd8be1dd40c0f58a4857e47bbec2fe78ccde` (`fix(carrier): reject negative switch_margin at construction`).
- This reviewer pass adds reviewer-only exact `af0e08a354ccc9dfa3b2127c357bf64079020fd3` (`docs(review): identify observability stable-v1 closure defects`). It does not change runtime behavior and is not a tested-tree anchor.
- Current `main` is reviewer-only `af0e08a` over source head `91f8cd8`.
- Since the high-throughput queue reopened at `e6a0377`, 18 commits landed before this reviewer note. Substantive source work includes:
  - `531c82d`: reject peer ACK `largest > largest_sent` atomically;
  - `8f93b93`: correct mixed datagram-drop attribution;
  - `02b6eaa`: reject any nonempty ACK when no packet has ever been sent;
  - `8a4e465`: bound datagram event emission work by event capacity;
  - `ece67b8`: attempt O2 sequence-exhaustion + O3 in-band drop-gap evidence repair;
  - `a14cf47`: saturate CarrierManager score+margin addition;
  - `91f8cd8`: reject negative `switch_margin` at construction.
- Dedicated bounded review support now exists for UDP recovery (`e3054c9`, `7a0c400`), `CarrierState` (`0831443`), observability challenge (`9ad927c` plus this follow-up), and CarrierManager/health/migration-back (`5f1cb8d`).
- Hosted `stable checks` and `nightly decode fuzz smoke` for exact `91f8cd8` are green. Hosted CI is cross-evidence only and does not replace developer-local exact-tree provenance.
- No VPS/WAN experiment occurred in this sequence. No release flag changed.
- Governance remains: item 3 incomplete, item 4 incomplete, `RELEASE_CANDIDATE=false`, `PRODUCTION_READY=false`, `FREEZE=false`, `RELEASED=false`, D019 policy-blocked, `READY_LIVE: none`.

## Reviewer verdict on recent carrier work

### `a14cf47` positive/extreme margin overflow — ACCEPT

Both selection comparisons now use `saturating_add`, preventing debug panic/release wrap from an extreme positive caller-supplied `switch_margin`. The dedicated CarrierManager review at `5f1cb8d` challenged single-active ownership, warm readiness, generation/validation, health separation, migration-back hold/margin and bounded evidence, and found no additional defect in that scope.

### `91f8cd8` negative margin domain — ACCEPT_WITH_BOUNDS

The accepted M4 semantics define `switch_margin` as a required improvement margin. A negative value has no committed meaning and would invert that gate, so rejecting `< 0` through existing `FlowError::InvalidLimit` is the smallest correct repair. The new regression covers `-1` and `i64::MIN`; no new upper bound or policy value is invented.

Do not reopen the manager margin family absent a new concrete failure. Final developer-local exact-tree provenance may be recorded on the next coherent corrected source tree after the mandatory observability repair below instead of creating provenance-only churn for `91f8cd8`.

## Evidence HIGH — observability O2/O3 implementation is not closed

Authority: `docs/era4-observability-contract.md` stable v1 + `schema/observability-event.v1.json`.

Reviewer report: `docs/reviews/independent-observability-followup-91f8cd8-20260913.md`.

Exact `ece67b8` fixes important parts of O2/O3, but the resulting emitted evidence still contradicts the stable machine contract. Stop expanding other item-4 surfaces until the three findings below are coherently closed.

### HIGH O4 — emitted event shapes violate the closed stable-v1 schema

Current sequence-exhaustion `resource.limit_hit` emits:

- `resource="event_sequence"`, but `event_sequence` is not currently in the stable-v1 `resource` enum;
- `limit="u64_max"`, but `limit` is an integer counter in the schema.

Current `diagnostic.events_dropped` emits `dropped_since_last`, but stable-v1 `data` has `additionalProperties=false` and does not define that field.

Required closure:

1. Every runtime-emitted event must be representable by `schema/observability-event.v1.json`.
2. Do not loosen `additionalProperties=false` merely to hide producer drift.
3. Stable v1 explicitly permits append-only enum/value additions. If sequence exhaustion remains a `resource.limit_hit`, adding truthful `event_sequence` to the existing resource enum is allowed; emit integer `u64::MAX` as `limit`, not a string.
4. Prefer removing `dropped_since_last` from emitted v1 JSON because the committed contract only requires `dropped_total` and retained `oldest_sequence`; only add a new optional field if an existing committed requirement actually needs it.
5. Add focused exact-shape regressions. Do not build a generalized runtime-schema framework.

### HIGH O5 — diagnostic insertion silently undercounts a second real eviction

Stable v1 §6 requires **every overflow** to evict oldest-first and increment `dropped_total`.

Current full-ring path can evict twice:

1. normal event append evicts one retained ordinary event and counts it;
2. `flush_drop_report` then appends `diagnostic.events_dropped` to the still-full ring via `push_inner(..., false)`, evicting another retained ordinary event without incrementing `dropped_total`.

The diagnostic itself must not recursively count as dropped, but the ordinary evidence displaced to make room for it is genuinely lost and must be reflected in the coalesced count. Repair without recursion. Cover capacity `1` and a small capacity `>1`, and prove `dropped_total` equals actual ordinary retained evidence evicted/truncated.

### HIGH O6 — `oldest_sequence` reports the pre-diagnostic floor

`flush_drop_report` formats `oldest_sequence()` before inserting the diagnostic. If the ring is full, that insertion evicts the current front, so the emitted floor is stale.

Stable v1 requires the retained sequence floor. Regression must compare the diagnostic field to the actual oldest retained sequence after the coalesced diagnostic is present.

## Continuous execution order

### MUST_EXECUTE_LOCAL 1 — repair O4/O5/O6 coherently

Owners:

- `crates/neko-observe/src/lib.rs`;
- `schema/observability-event.v1.json` only for the minimal truthful append-only `resource` enum adjustment if the sequence marker keeps `resource.limit_hit`;
- focused observability contract tests as needed.

Keep already-correct behavior green:

- O1 mixed queue-full/terminal attribution;
- bounded huge counter deltas;
- strict sequence ordering at exhaustion;
- one bounded/coalesced drop diagnostic rather than recursive emission;
- health/switch/recovery/PTO/scheduler/resource projections;
- secret-free correlation and fixed event vocabulary.

Minimum focused validation:

```bash
cargo test -p neko-observe
bash scripts/check-observability-contract.sh
bash scripts/check-observability-contract-test.sh
```

No wire/crypto decoder change is required, so no new decode-fuzz requirement for this repair.

Commit/push one coherent repair and continue immediately to LOCAL 2.

### MUST_EXECUTE_LOCAL 2 — coherent exact-tree local gate/provenance

On the final pushed source tree containing O4/O5/O6 plus the already-landed carrier repairs:

```bash
cargo test -p neko-observe
cargo test -p neko-carrier
PYTHONDONTWRITEBYTECODE=1 bash scripts/check.sh
git diff --check
git status --short
```

Persist concise sanitized provenance with reachable exact SHA, commands, UTC start/end, exit codes, OS/arch, stable Rust version, clean initial/final tree. GitHub-hosted CI is optional cross-evidence and not a wait condition.

### REVIEW_LOCAL 3 — independently re-close corrected observability producer

Challenge the corrected reachable tree against stable v1:

- every emitted event shape is schema-valid;
- strict sequence ordering/exhaustion;
- exact overflow/drop accounting including diagnostic-induced eviction;
- retained floor correctness;
- bounded huge counter deltas;
- mixed datagram attribution;
- health/switch/recovery/PTO projections;
- scheduler/resource high-water projection;
- secret-safe correlation/fixed vocabulary;
- no recursive/unbounded diagnostic work.

Concrete defect -> smallest repair/regression -> push -> exact-tree gate. No finding -> bounded independent note.

After this re-close, the observability HIGH is closed and the deep queue resumes without reviewer wait.

### REVIEW_LOCAL 4 — FairScheduler + multistream + flow-control accounting

Primary owner: `crates/neko-carrier/src/lib.rs` `FairScheduler`, `FlowLimits`, stream queues and directly related tests/specs.

Challenge:

- checked stream/session byte accounting before allocation/enqueue;
- exact stream/session caps and arithmetic overflow atomicity;
- round-robin cursor behavior across empty/nonempty streams;
- interactive burst preference without permanent bulk starvation;
- fallback selection when preferred class is empty;
- duplicate stream open semantics against current spec/callers;
- lifecycle/resource retention: determine whether stream closure/removal is intentionally owned elsewhere or whether scheduler state can consume `max_streams` permanently contrary to a current claim;
- dequeue releases exact bytes and cannot underflow internal counters under public API sequences.

Do not invent new scheduling weights, burst size or flow-control policy. Existing-semantics defect -> smallest repair/regression. No finding -> dedicated bounded note.

### REVIEW_LOCAL 5 — Carrier adapters close/error/resource semantics

Review Memory/UDP/TCP adapters plus fault wrapper for:

- local vs peer close and queued-data drain;
- would-block/EOF/reset/truncated distinctions where contract requires them;
- message/frame boundaries and empty messages;
- oversize/truncation atomicity before allocation/state promotion;
- buffer/resource release and close idempotence;
- mutex poison/OS error mapping;
- adapter observations never becoming Session delivery/path-validation evidence.

No transport redesign or platform-normalization framework.

### REVIEW_LOCAL 6 — `SessionRuntime` lifecycle/resource/DeliveryAck accounting

Challenge:

- stream/session windows;
- queue/total byte accounting and exact release;
- send/receive offset progression and arithmetic boundaries;
- DeliveryAck bounds/release and no packet-ACK conflation;
- duplicate receive/dedup/conflict behavior;
- close/idle/deadline/cancel cleanup;
- atomic invalid transitions;
- process-message bounded decode/admission;
- reset/close state and resource cleanup.

Do not redesign the process codec or Session semantics.

### REVIEW_LOCAL 7 — package / reproducibility / operator implementation

Review current `scripts/release/` behavior rather than old evidence prose:

- clean-source refusal before mutation;
- archive path/type/root/layout/mode/checksum rejection ordering;
- reproducibility identity;
- installed-package lifecycle and A/B/A state boundary;
- failure cleanup/listener/process residue handling;
- secret-safe retained evidence;
- partial/negative result truthfulness.

Do not invent signing/key-custody/SBOM/publication policy.

### REVIEW_LOCAL 8 — dependency/build independent pass

Inspect Cargo manifests/lock/features/build scripts/native hooks and workspace unsafe inheritance. Challenge executable dependency graph consistency, unexpected native/build behavior, feature drift, and crates that fail to inherit workspace safety policy. No scanner framework, dependency upgrade or SBOM policy.

### REVIEW_LOCAL 9 — cross-platform CLI/process semantics

Challenge remaining OS-artifact assumptions: socket terminal outcomes, filesystem permission/rename/unlink behavior, signals/shutdown, process exit/status, listener release/rebind, platform API/error mapping. Preserve security invariants; do not broaden errors merely to make tests portable.

### REVIEW_LOCAL 10 — CLI machine/human/exit-code contract

Challenge success JSON timing, nonzero rejection exits, exact human probe output versus JSON authority, secret-safe stderr, benchmark partial/blocked semantics, and READY/DRAINING/STOPPED stage truth.

### REVIEW_LOCAL 11 — algorithmic resource boundedness, static/deterministic only

Search implemented core owners for:

- loops driven by external numeric magnitude instead of retained capacity;
- clone/allocation before validated bound;
- retry loops without bounded progress;
- collection growth beyond declared retained state;
- arithmetic overflow/saturation that can invert a gate or fabricate evidence/state;
- diagnostic/evidence work that grows faster than retained state.

This is not adversarial capacity testing and sets no new policy values.

### REVIEW_SUPPORT 12 — release/item-4 factual reconciliation

After roughly 3–4 further coherent source/review slices, reconcile `docs/release-security-review-packet.md`, `docs/reviews/release-item4-subgates-20260909.md`, review indexes, and `docs/status.md` only where current facts require it. Index reachable evidence; do not self-declare independent security approval or item-4 completion.

## Queue discipline

This is still a deep queue. Do not collapse it after each small commit. A bounded no-finding independent review of a previously unreviewed implemented core surface is useful item-4 work.

Unresolved O4/O5/O6 is an evidence HIGH, so fix it before expanding into REVIEW_LOCAL 4+. After closure, continue through the pre-authorized queue without waiting for reviewer cadence.

Stop only for unresolved BLOCKER/HIGH, core architecture/policy decision, destructive/canonical migration, action outside standing authorization, production/third-party action, new credential/server permission, maintainer-valued capacity/pressure conditions, repository breakage, or actual runtime/tool-budget exhaustion.

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

Standing VPS authorization remains valid, but no new dependency-ready live question is created here. Do not mechanically rerun HY2, repeated warm failover, periodic/soak, package lifecycle, migration-back, endpoint migration, key update, IPv6, PLPMTUD, or Experimental Track work.