# ChatGPT reviewer handoff — observability stable-v1 HIGH remains first; preserve deep core-review queue

## Reviewed repository truth

- Current default branch before this refresh: exact `496258d80819a3e3fc3b9f36426cb55bd8ef9f11` (`docs(review): independent bounded FairScheduler/flow-control review at 91f8cd8`).
- Latest developer source/test head remains exact `91f8cd8be1dd40c0f58a4857e47bbec2fe78ccde` (`fix(carrier): reject negative switch_margin at construction`). Commits after it are reviewer/review-support docs only; they do not repair runtime source.
- Current source still contains exact `ece67b85a23b62dfba6fe00f817c9badd975fad0` observability O2/O3 implementation underneath the later carrier fixes.
- Hosted GitHub `stable checks` and `nightly decode fuzz smoke` for exact `91f8cd8` are green. This is cross-evidence only; developer-local exact-tree provenance is still required on the next coherent corrected source tree.
- No VPS/WAN experiment, release flag, production state, or live classification changed.
- Governance remains: item 3 incomplete, item 4 incomplete, `RELEASE_CANDIDATE=false`, `PRODUCTION_READY=false`, `FREEZE=false`, `RELEASED=false`, D019 policy-blocked, `READY_LIVE: none`.

This reviewer refresh re-read the current execution/governance surfaces and the exact owners involved in the active HIGH: `README.md`, `AGENTS.md`, `ROADMAP.md`, `IMPLEMENTATION_PLAN.md`, `SECURITY.md`, `docs/status.md`, `docs/standing-vps-lab-authorization.md`, `docs/vps-rental-window-priority.md`, `docs/decisions.md`, `docs/carrier-architecture.md`, `docs/specs/nekomusume-session-v0.md`, `docs/release-security-review-packet.md`, `docs/era4-observability-contract.md`, `schema/observability-event.v1.json`, current `crates/neko-observe/src/lib.rs`, and the new FairScheduler review.

## Verdict on new `496258d` FairScheduler / flow-control review

### `496258d` — ACCEPT_AS_BOUNDED_SUPPORT, but it does not advance the queue front

The note is technically useful and bounded: it challenges scheduler fairness, byte accounting, arithmetic atomicity and monotonic stream-slot retention at exact source tree `91f8cd8`, runs `cargo test -p neko-carrier`, and makes no WAN/performance/release claim. No concrete scheduler defect is established.

However, it was produced after the current handoff had already declared observability O4/O5/O6 an evidence HIGH and explicitly required source closure before expanding farther. Therefore:

- retain `docs/reviews/independent-fair-scheduler-91f8cd8-20260913.md` as useful module-specific support;
- do **not** treat it as closing or bypassing the active observability HIGH;
- do **not** redo the scheduler review after O4/O5/O6 unless the observability repair unexpectedly touches scheduler/carrier code;
- return immediately to the mandatory observability repair below.

This avoids both review-order drift and duplicate work.

## Evidence HIGH remains OPEN — observability stable-v1 O4/O5/O6

Authority: `docs/era4-observability-contract.md` stable v1 plus `schema/observability-event.v1.json`.

Existing reviewer finding: `docs/reviews/independent-observability-followup-91f8cd8-20260913.md`.

Current source still demonstrates all three contradictions:

1. sequence-exhaustion `resource.limit_hit` emits `resource="event_sequence"` while the stable-v1 `resource` enum does not contain `event_sequence`;
2. that event emits `limit="u64_max"` as a string while schema `limit` is a nonnegative integer counter;
3. `diagnostic.events_dropped` emits undeclared `dropped_since_last` even though v1 `data.additionalProperties=false`;
4. `flush_drop_report` inserts the diagnostic with `push_inner(..., false)` while the ring can still be full, so a second retained event can be evicted without incrementing `dropped_total`;
5. `oldest_sequence` is formatted before that diagnostic insertion/eviction, so the emitted retained floor can be stale.

Do not expand into additional core-surface reviews until this HIGH is coherently repaired and re-closed. Existing already-written bounded notes remain retained; the restriction is against further expansion, not against preserving evidence.

## MUST_EXECUTE_LOCAL 1 — repair O4/O5/O6 as one coherent source slice

Primary owners:

- `crates/neko-observe/src/lib.rs`;
- `schema/observability-event.v1.json` only for the minimal truthful append-only enum adjustment required by the stable-v1 contract;
- focused `neko-observe` tests / existing observability contract tests.

### Required O4 shape

Use the existing stable-v1 semantics; do not weaken the schema.

- If sequence exhaustion stays represented by `resource.limit_hit`, append truthful `event_sequence` to the existing `resource` enum. Stable v1 explicitly permits append-only enum/value additions.
- Keep `limit` numeric. For this bound, emit integer `u64::MAX` / `18446744073709551615`, not string `"u64_max"`.
- `observed` remains numeric.
- Remove `dropped_since_last` from emitted stable-v1 `diagnostic.events_dropped` unless a current committed requirement proves a stable meaning is required. The current contract requires `dropped_total` and retained `oldest_sequence`; adding a new field is unnecessary to close the bug.
- Add focused exact-shape regressions. A generalized runtime-schema framework is not required.

### Required O5/O6 accounting shape

The simplest bounded implementation shape is to make room for the coalesced diagnostic **before** constructing its final data when the ring is full:

1. if `drops_since_report == 0` or no fresh sequence is available, follow the existing bounded no-report path;
2. when a diagnostic must be emitted and the ring is full, reserve one slot by evicting exactly the current oldest retained event;
3. that reservation eviction is a real loss of retained evidence, so increment `dropped_total` for it; do not recurse through normal `push`/`flush`;
4. after any reservation eviction, compute the actual retained `oldest_sequence` that will remain once the diagnostic is inserted;
5. construct one `diagnostic.events_dropped` using the updated `dropped_total` and post-reservation retained floor;
6. insert it into the already-reserved slot without a second eviction and without recursively reporting the diagnostic itself;
7. reset/coalesce pending drop-report state exactly once.

Equivalent implementations are acceptable if regressions prove the same contract. Do not create a second diagnostic queue or a generalized logging subsystem.

### Mandatory focused regressions

At minimum prove:

- sequence-exhaustion `resource.limit_hit` is representable by stable-v1 schema vocabulary and uses integer `limit`;
- `diagnostic.events_dropped` emits only declared stable-v1 fields;
- capacity `1`: ordinary overflow plus diagnostic reservation accounts every ordinary retained event actually lost, terminates without recursion, and final diagnostic `oldest_sequence` equals the actual retained floor;
- small capacity `>1`: exact same accounting/floor property;
- existing mixed queue-full/terminal datagram attribution stays correct;
- huge counter deltas remain bounded by event capacity and terminate;
- emitted event sequences remain strictly increasing until the explicit exhaustion boundary;
- O1 health/switch/recovery/PTO/scheduler/resource projection tests remain green.

Minimum focused commands:

```bash
cargo test -p neko-observe
bash scripts/check-observability-contract.sh
bash scripts/check-observability-contract-test.sh
```

No wire/crypto decoder change is expected, so no new decode-fuzz requirement for this slice.

Commit and push the coherent source/schema repair, then continue immediately to LOCAL 2.

## MUST_EXECUTE_LOCAL 2 — exact-tree local closure/provenance

On the final pushed developer source SHA containing O4/O5/O6 plus the already-landed carrier fixes:

```bash
cargo test -p neko-observe
cargo test -p neko-carrier
PYTHONDONTWRITEBYTECODE=1 bash scripts/check.sh
git diff --check
git status --short
```

Persist one concise sanitized provenance record with exact reachable SHA, commands, UTC start/end, exit codes, OS/arch, stable Rust version and clean initial/final tree. Do not wait for hosted CI.

## REVIEW_LOCAL 3 — independent observability re-close

After the source repair, independently challenge the corrected exact tree. This re-close must cover:

- every runtime-emitted event shape against stable-v1 vocabulary/types;
- strict sequence ordering and explicit exhaustion behavior;
- exact overflow/drop accounting including diagnostic-reservation eviction;
- retained-floor correctness;
- bounded huge counter deltas and mixed datagram attribution;
- health/switch/recovery/PTO/scheduler/resource projections;
- secret-safe correlation/fixed vocabulary;
- no recursive or externally-magnitude-driven diagnostic work.

Also inspect the stable-v1 rule that monotonic metric saturation must emit `resource.limit_hit`: current producer uses several `saturating_add` counters (`dropped_total`, PTO/retransmit/dequeue counters). Do not pre-judge these as defects; determine whether any saturation is reachable through current public producer inputs and can silently contradict the contract. Concrete reachable contradiction -> smallest repair/regression. No finding -> state why.

Once this re-close has no unresolved BLOCKER/HIGH, observability HIGH is closed and the deep queue resumes immediately without reviewer wait.

## Deep pre-authorized queue after observability closure

The queue remains intentionally deep; preserve completed no-finding reviews and skip duplicates.

### REVIEW_LOCAL 4 — Carrier adapters close/error/resource semantics

Review Memory/UDP/TCP adapters plus fault wrapper for local vs peer close, queued-data drain, EOF/reset/would-block/truncation distinctions, message/frame boundaries, empty messages, oversize/truncation atomicity, resource release, close idempotence, poison/OS-error mapping, and evidence-domain separation. No transport redesign or portability framework.

### REVIEW_LOCAL 5 — `SessionRuntime` lifecycle/resource/DeliveryAck accounting

Challenge stream/session windows, queue/total bytes, send/receive offsets, DeliveryAck release, duplicate/dedup/conflict handling, idle/close/deadline/cancel cleanup, terminal transitions, process-message decode bounds, and reset/close resource cleanup. Packet ACK must remain separate from Session delivery evidence. No process-codec redesign.

### REVIEW_LOCAL 6 — package / reproducibility / operator implementation

Review current `scripts/release/` source behavior: clean-source refusal before mutation, archive shape/type/root/mode/checksum ordering, reproducibility identity, installed-package lifecycle/A-B-A boundary, cleanup/listener/process residue, secret-safe retained evidence, and negative/partial result truth. Do not invent signing/key-custody/SBOM/publication policy.

### REVIEW_LOCAL 7 — dependency/build independent pass

Inspect Cargo manifests/lock/features/build scripts/native hooks and workspace unsafe inheritance. Challenge executable dependency graph consistency, unexpected native/build behavior, feature drift, and crates that fail to inherit workspace safety policy. No scanner framework, dependency upgrade or SBOM policy.

### REVIEW_LOCAL 8 — cross-platform CLI/process semantics

Challenge remaining OS-artifact assumptions: socket terminal outcomes, filesystem permission/rename/unlink behavior, signals/shutdown, process exit/status, listener release/rebind and platform error mapping. Preserve fail-closed semantics; do not broaden accepted errors merely for portability.

### REVIEW_LOCAL 9 — CLI machine/human/exit-code contract

Challenge success JSON timing, nonzero rejection exits, human probe strings versus JSON authority, secret-safe stderr, benchmark partial/blocked semantics and READY/DRAINING/STOPPED stage truth.

### REVIEW_LOCAL 10 — algorithmic resource boundedness, static/deterministic only

Search implemented core owners for external-magnitude loops, allocation/clone before validated bounds, retry loops without bounded progress, collection growth beyond declared retained state, arithmetic saturation/overflow that can invert gates or fabricate evidence/state, and diagnostic work growing faster than retained state. This is not adversarial capacity testing and sets no new policy values.

### REVIEW_SUPPORT 11 — release/item-4 factual reconciliation

After roughly 3–4 further coherent source/review slices, reconcile `docs/release-security-review-packet.md`, `docs/reviews/release-item4-subgates-20260909.md`, review indexes and `docs/status.md` only where facts require it. Index reachable review/repair evidence, including the already-retained FairScheduler note when appropriate. Do not self-declare independent security approval or item-4 completion.

### REVIEW_LOCAL 12 — broad core-surface inventory before any future `queue exhausted`

Only after the above, inventory all implemented core crates and release tooling for remaining surfaces without dedicated bounded independent challenge. `queue exhausted` is permitted only if this repository-wide inventory finds no unreviewed implemented core surface, no concrete defect, no READY review-support and no READY live question, with all remaining work genuinely external/policy/environment/release-authority gated.

## Queue discipline

- O4/O5/O6 is the active evidence HIGH. Source repair comes first.
- `496258d` FairScheduler review is retained; do not redo it unless touched.
- After HIGH closure, continue the deep queue without waiting for another reviewer pass.
- A bounded no-finding independent review of a previously unreviewed implemented core surface is valid item-4 support and is not filler.
- Do not collapse this queue to one ticket after a small commit.
- Every concrete defect whose answer is already determined by committed semantics becomes smallest repair -> regression -> push -> exact-tree local gate -> provenance -> continue.

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

Standing VPS authorization remains valid, but current repository truth creates no dependency-ready live question. Do not mechanically rerun HY2, repeated warm failover, periodic/soak, package lifecycle, migration-back, endpoint migration, key update, IPv6, PLPMTUD, or Experimental Track work.