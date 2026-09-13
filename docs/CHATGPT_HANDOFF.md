# ChatGPT reviewer handoff — observability HIGH closed; continue deep item-4 core review

## Reviewed repository truth

- Current default branch before this reviewer refresh: exact `18f7c58fa9504c9f0c0bd4e4bb1f17017083d09a` (`docs(review): independent observability stable-v1 re-close at 8e11de0`).
- Latest developer source/test head is exact `8e11de0e5915efa43591069172b97f311aae2c6e` (`fix(observe): close stable-v1 event schema and drop-evidence contract`).
- Exact `8e11de0` has clean developer-local exact-tree provenance recorded in the independent re-close: `cargo test -p neko-observe`, observability contract checks, `PYTHONDONTWRITEBYTECODE=1 bash scripts/check.sh`, `git diff --check`, and clean tree all passed. Hosted GitHub `stable checks` and `nightly decode fuzz smoke` for exact `8e11de0` also completed successfully; hosted CI remains extra cross-evidence only.
- No VPS/WAN experiment, release flag, production state or live classification changed.
- Governance remains: item 3 incomplete, item 4 incomplete, `RELEASE_CANDIDATE=false`, `PRODUCTION_READY=false`, `FREEZE=false`, `RELEASED=false`, D019 policy-blocked, `READY_LIVE: none`.

This reviewer refresh re-read the current handoff, current observability re-close, release packet, current carrier adapter owner code, and the already-landed bounded reviews/repairs for UDP recovery, CarrierState, CarrierManager/migration-back and FairScheduler/flow-control.

## Verdict on new source/review sequence

### `8e11de0` observability O4/O5/O6 source closure — ACCEPT

The source/schema repair closes the previously open stable-v1 evidence HIGH:

- `resource.limit_hit` for sequence exhaustion now uses append-only stable-v1 vocabulary `resource="event_sequence"` and numeric `limit=u64::MAX`;
- `diagnostic.events_dropped` emits only declared stable-v1 fields;
- every ordinary retained event displaced by overflow/diagnostic insertion is counted in `dropped_total`;
- diagnostic insertion does not recursively report/count itself;
- `oldest_sequence` is computed from the actual post-insertion retained floor;
- previously-fixed bounded huge-delta emission and mixed queue-full/terminal attribution remain intact.

The only schema change is the allowed append-only resource enum value. No schema weakening or second diagnostic store was introduced.

### `18f7c58` independent observability re-close — ACCEPT_AS_BOUNDED_SUPPORT

The review reproduces the corrected source/tree behavior with focused and full local gates and carries the exclusions honestly. O2/O3/O4/O5/O6 are closed on reachable exact `8e11de0`.

**The observability evidence HIGH is CLOSED.** Do not reopen or redo this surface unless later source/schema changes materially touch it.

## Already completed core-review surfaces — preserve, do not redo

The following high-throughput item-4 work is already retained and should be treated as completed bounded support unless touched by later fixes:

1. UDP recovery review plus future/never-sent ACK repairs (`531c82d`, `02b6eaa`, independent close notes);
2. `CarrierState` generation/validation/hysteresis review (`0831443` at source `8f93b93`);
3. CarrierManager/migration-back review plus switch-margin overflow/negative-domain repairs (`a14cf47`, `91f8cd8`, `5f1cb8d`);
4. FairScheduler/flow-control review (`496258d` at source `91f8cd8`);
5. observability projection/buffer/drop/schema review and repairs (`8f93b93`, `8a4e465`, `ece67b8`, `8e11de0`, final re-close `18f7c58`).

These bounded reviews do not close item 4 or constitute security/release approval, but they remove those surfaces from the immediate queue.

## Deep pre-authorized queue — continue without reviewer wait

The external coding/review agent should proceed continuously through the following dependency-independent slices. A concrete defect whose correct answer is already determined by current committed semantics becomes smallest repair -> regression -> commit/push -> exact-tree local gate/provenance -> immediately continue. A no-finding review becomes a scope-precise independent bounded note and then continue.

### REVIEW_LOCAL 1 — Carrier adapters close/error/resource semantics

Primary owner: `crates/neko-carrier/src/lib.rs` adapter surfaces and focused tests.

Challenge Memory/UDP/TCP adapters plus `FaultInjectCarrier` for:

- local close versus peer close;
- queued-data drain semantics;
- EOF/reset/would-block/truncation distinctions;
- message/frame boundaries and empty payloads;
- oversize/truncation atomicity;
- resource release and close idempotence;
- poison/OS-error mapping;
- carrier-neutral `CarrierError` consistency;
- no promotion of adapter observations into Session delivery/path-validation evidence.

**Concrete candidate to investigate first:** the carrier-neutral contract says callers can distinguish retryable absence from terminal close. TCP and Memory expose explicit `Closed`/`PeerClosed` outcomes, while current UDP local-close behavior appears to return `UdpError::Io(NotConnected)` on send (mapped to `CarrierError::Io`) and `Ok(None)` on receive. Determine from current committed adapter/decision semantics whether this is a real carrier-neutral contract contradiction. If yes, add focused regressions and make the smallest mapping/behavior repair; if no, document the exact reason and continue. Do not invent a portability framework or redesign adapter APIs.

Focused minimum: `cargo test -p neko-carrier` plus any exact targeted tests. Code change -> exact pushed SHA full local gate/provenance before continuing.

### REVIEW_LOCAL 2 — `SessionRuntime` lifecycle/resource/DeliveryAck accounting

Primary owner: `crates/neko-session/src/lib.rs` runtime/process-message surfaces plus directly relevant specs/tests.

Challenge:

- stream/session windows and queue/total byte accounting;
- send/receive offset arithmetic and atomicity;
- DeliveryAck release semantics and duplicate/conflict behavior;
- idle/close/deadline/cancel cleanup;
- terminal transition idempotence;
- resource reset/release on remote/local close/error;
- process-message decode bounds and malformed/trailing behavior;
- separation of packet ACK from Session delivery evidence.

No process-codec redesign or new Session semantics. Concrete existing-semantics defect -> smallest repair/regression; otherwise bounded no-finding review.

### REVIEW_LOCAL 3 — package / reproducibility / operator implementation

Primary owners: current `scripts/release/` implementation/tests plus package/operator evidence.

Challenge implementation behavior, not policy invention:

- clean-source refusal occurs before build/output mutation;
- archive member type/root/path/mode/checksum validation is fail-closed and ordered safely;
- reproducible archive/binary identity claims match actual script behavior;
- installed-package lifecycle and A/B/A evidence boundaries remain truthful;
- cleanup/listener/process-residue handling does not overwrite partial/negative evidence;
- retained logs/evidence are secret-safe;
- interrupted/partial paths do not silently claim complete success.

Do not invent signing, key-custody, SBOM or publication-trust policy. No VPS rerun merely to review scripts.

### REVIEW_LOCAL 4 — dependency/build independent pass

Inspect Cargo manifests, lockfile, feature selections, build scripts/native hooks and workspace lint inheritance.

Challenge:

- actual executable dependency graph against current factual documentation;
- unexpected build/native behavior;
- feature drift or accidental default-feature expansion;
- crates failing to inherit `unsafe_code = "forbid"` where current workspace policy says they should;
- platform-specific hooks without bounded cfg/test treatment.

No new scanner framework, dependency upgrade, signing/SBOM policy or cryptographic suitability approval. Existing developer factual dependency note is not a substitute for this independent bounded pass.

### REVIEW_LOCAL 5 — cross-platform CLI/process semantics

Challenge remaining process/integration tests and CLI owners for accidental OS-artifact assumptions:

- orderly EOF/reset/broken-pipe distinctions only where current contract permits them;
- filesystem permission/rename/unlink behavior and Unix-only cfg boundaries;
- signals/shutdown/process exit/status;
- listener release/rebind;
- platform error mapping and timeout semantics.

Preserve fail-closed/security invariants. Do not broaden accepted errors simply to make tests portable.

### REVIEW_LOCAL 6 — CLI machine/human/exit-code contract

Challenge executable output truth:

- success JSON only after the claimed authenticated/admitted operation actually succeeds;
- nonzero exit on rejected/blocked/invalid paths;
- human `喵~！` / `喵呜呜呜呜…` strings never substitute for machine JSON/exit code;
- secret-safe stderr;
- READY/DRAINING/STOPPED stages are not emitted prematurely;
- benchmark partial/blocked results cannot produce complete/comparative summaries.

Concrete contradiction -> smallest repair/regression; otherwise bounded review note.

### REVIEW_LOCAL 7 — algorithmic resource boundedness, static/deterministic only

Across implemented core owners, search specifically for:

- loops whose work is controlled by externally supplied numeric magnitude rather than bounded retained state;
- allocation/clone before validated length/count limits;
- retry loops without bounded progress/deadline;
- collections whose growth exceeds declared retained limits;
- arithmetic overflow/saturation that can invert gates or fabricate evidence/state;
- diagnostic/event work growing faster than the bounded retained store.

This is not RSEC-001 capacity/adversarial-load testing and must not select new policy numbers or run pressure benchmarks. Concrete local algorithmic defect under current semantics may be repaired.

### REVIEW_SUPPORT 8 — release/item-4 factual reconciliation

After roughly 3–4 more coherent source/review slices from the queue above, reconcile:

- `docs/release-security-review-packet.md`;
- `docs/reviews/release-item4-subgates-20260909.md`;
- review indexes;
- `docs/status.md` only if a status fact actually changes.

Index reachable accepted review/repair evidence from UDP recovery, CarrierState, CarrierManager, FairScheduler and observability as appropriate. Preserve exact tested-tree anchors and exclusions. Do not self-declare independent security approval or item-4 completion.

### REVIEW_LOCAL 9 — broad core-surface inventory before any future `queue exhausted`

After the above queue, inventory every implemented core crate and release tool for remaining substantial surfaces without a dedicated bounded independent challenge. Include at least:

- crypto API/record boundary versus already-reviewed scope;
- wire/parser versus existing independent wire review;
- reliable/Carrier/Session/observe coverage;
- benchmark/result validators and release scripts;
- package/operator/reproducibility;
- CLI/process/output boundaries.

Only classify `queue exhausted` if this repository-wide inventory finds no unreviewed implemented core surface, no concrete defect, no READY review-support and no READY live question, and every remaining item is genuinely external/policy/environment/release-authority gated.

## Queue discipline

- Observability HIGH is closed. Do not keep it at queue front.
- Completed module reviews remain accepted bounded support; skip duplicates unless touched.
- Preserve this deep queue across small commits; do not collapse it to one task.
- A bounded no-finding independent challenge of a previously unreviewed implemented core surface is valid item-4 support, not filler.
- Every concrete defect with an answer fixed by current committed semantics becomes immediate repair/regression/exact-tree closure, then continue.
- Every code/test repair requires clean exact pushed developer SHA validation with at least `PYTHONDONTWRITEBYTECODE=1 bash scripts/check.sh`, `git diff --check`, clean-tree confirmation and sanitized provenance. Hosted CI is extra cross-evidence, not a wait condition.
- Wire decoder/parser/crypto framing changes require pinned decode fuzz per repository policy; ordinary adapter/Session/package/CLI/docs reviews do not mechanically require fuzz.

Stop only for unresolved BLOCKER/HIGH, core architecture/policy decision, destructive/canonical migration, action outside standing authorization, production/third-party action, new credential/server permission, maintainer-valued capacity/pressure conditions, repository breakage or actual runtime/tool-budget exhaustion.

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

Standing VPS authorization remains valid, but current repository truth creates no dependency-ready live question. Do not mechanically rerun HY2, repeated warm failover, periodic/soak, package lifecycle, migration-back, endpoint migration, key update, IPv6, PLPMTUD or Experimental Track work.