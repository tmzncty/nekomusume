# ChatGPT reviewer handoff — adapter review accepted; challenge SessionRuntime terminal cleanup next

## Reviewed repository truth

- Current default branch before this refresh includes reviewer handoff `8d443d3cc48eaa49c4a2b8c8e4cfc3162d2044d1` and concurrent bounded carrier-adapter review `31c17695c695747c2e46f4cf43f9fd10a17d2927`.
- Latest developer source/test head remains exact `8e11de0e5915efa43591069172b97f311aae2c6e` (`fix(observe): close stable-v1 event schema and drop-evidence contract`).
- Exact `8e11de0` has clean developer-local exact-tree provenance; hosted `stable checks` and `nightly decode fuzz smoke` also succeeded. Hosted CI remains extra cross-evidence only.
- No VPS/WAN experiment, release flag, production state or live classification changed.
- Governance remains: item 3 incomplete, item 4 incomplete, `RELEASE_CANDIDATE=false`, `PRODUCTION_READY=false`, `FREEZE=false`, `RELEASED=false`, D019 policy-blocked, `READY_LIVE: none`.

## Accepted completed slices — do not repeat

### Observability stable-v1 closure — ACCEPT

Exact source `8e11de0` plus independent re-close `18f7c58` closes O2/O3/O4/O5/O6. Sequence exhaustion, schema-valid `resource.limit_hit`, drop diagnostics, exact drop accounting, retained floor, bounded huge deltas and mixed datagram attribution are closed on that reachable tree. Do not reopen unless later source/schema changes touch this surface.

### Carrier adapter review `31c1769` — ACCEPT_AS_BOUNDED_SUPPORT

The independent review of Memory/UDP/TCP adapters on exact `8e11de0` is retained as useful item-4 support. It finds no repairable defect in bounded message/allocation checks, local close idempotence, memory queued-data drain, UDP connectionless close behavior, TCP framing/truncation, mutex poison/I/O mapping or evidence-domain separation. Do not redo this review unless adapter code changes.

The reviewer had specifically challenged UDP local-close mapping. `31c1769` resolves that current bounded adapter contract as acceptable: UDP is connectionless and has no queued-drain/peer-close contract; local close stops sends and makes receive terminal for that adapter without creating Session/path evidence. No new cross-adapter framework is warranted from current semantics.

## Already completed core-review surfaces — preserve

1. UDP recovery review plus future/never-sent ACK repairs (`531c82d`, `02b6eaa` and independent close notes);
2. `CarrierState` generation/validation/hysteresis (`0831443` at source `8f93b93`);
3. CarrierManager/migration-back plus switch-margin overflow/negative-domain repairs (`a14cf47`, `91f8cd8`, `5f1cb8d`);
4. FairScheduler/flow-control (`496258d` at source `91f8cd8`);
5. observability projection/buffer/drop/schema (`8f93b93`, `8a4e465`, `ece67b8`, `8e11de0`, re-close `18f7c58`);
6. carrier adapter close/error/resource review (`31c1769` at source `8e11de0`).

These remain bounded review support, not release/security approval or item-4 completion.

## Deep pre-authorized queue — continue without reviewer wait

### REVIEW_LOCAL 1 — `SessionRuntime` lifecycle/resource/DeliveryAck accounting

Primary owner: `crates/neko-session/src/lib.rs` runtime/process-message surfaces plus applicable specs/tests.

Challenge:

- stream/session windows and queue/total byte accounting;
- send/receive offset arithmetic and atomicity;
- DeliveryAck release semantics and duplicate/conflict behavior;
- idle/close/deadline/cancel cleanup;
- terminal transition idempotence;
- process-message decode bounds and malformed/trailing behavior;
- separation of packet ACK from Session delivery evidence.

**Concrete cleanup candidate to test first:** current exact `8e11de0` has asymmetric terminal cleanup paths. `close_remote()` and `cancel()` clear `send`, `recv`, `received`, `confirmed`, `send_inflight`, `recv_window_used`, session send/receive window accounting and `queued_bytes`. In contrast, `tick()` when idle timeout closes the Session, and `tick()` when the graceful-close deadline expires, currently clear only `send`, `recv` and `queued_bytes` before entering `Closed`; they appear to retain dedup history (`received`), confirmation state, per-stream inflight/window maps and session-level window counters until the closed runtime object itself is dropped.

Do not pre-judge this as a bug. First build focused in-module regressions that populate receive/dedup and send/window state, then trigger:

1. idle timeout terminalization;
2. graceful-close deadline terminalization;
3. existing `close_remote()`/`cancel()` as control paths.

Determine from current committed runtime/resource semantics whether all terminal paths are required to release the same bounded runtime-owned retained state. If yes, this is a concrete resource-cleanup correctness defect: make the smallest shared cleanup or equivalent local repair and prove terminal state, counters/maps/queues, events and later operation failures remain correct. Do **not** reset lifetime/cumulative facts whose semantics intentionally survive terminalization, and do not redesign Session lifecycle. If current semantics intentionally retain dedup/window state after timeout closure, document the exact committed reason and continue.

Also challenge `event()` sequence arithmetic (`next_event += 1`) near exhaustion as part of this review only if a current public path can reach it under bounded runtime operation; do not manufacture a new event policy absent a reachable contradiction.

Focused minimum: `cargo test -p neko-session`; code change -> exact pushed SHA full local gate/provenance before continuing.

### REVIEW_LOCAL 2 — package / reproducibility / operator implementation

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

### REVIEW_LOCAL 3 — dependency/build independent pass

Inspect Cargo manifests, lockfile, feature selections, build scripts/native hooks and workspace lint inheritance.

Challenge executable dependency graph against factual docs, unexpected build/native behavior, feature drift/default-feature expansion, workspace `unsafe_code = "forbid"` inheritance, and platform-specific hooks/cfg boundaries. No scanner framework, dependency upgrade, SBOM/signing policy or cryptographic suitability approval.

### REVIEW_LOCAL 4 — cross-platform CLI/process semantics

Challenge remaining OS-artifact assumptions: socket terminal outcomes only where committed contract permits them; filesystem permission/rename/unlink behavior and cfg boundaries; signals/shutdown/process exit/status; listener release/rebind; platform error/timeout mapping. Preserve fail-closed semantics; do not broaden accepted errors merely for portability.

### REVIEW_LOCAL 5 — CLI machine/human/exit-code contract

Challenge success JSON timing, rejected/blocked/invalid nonzero exits, human `喵~！` / `喵呜呜呜呜…` versus machine authority, secret-safe stderr, READY/DRAINING/STOPPED stage truth, and benchmark partial/blocked result semantics. Concrete contradiction -> smallest repair/regression; otherwise bounded review note.

### REVIEW_LOCAL 6 — algorithmic resource boundedness, static/deterministic only

Across implemented core owners, search for external-magnitude loops, allocation/clone before validated bounds, retry loops without bounded progress/deadline, collection growth beyond retained limits, arithmetic overflow/saturation that can invert gates or fabricate evidence/state, and diagnostic work growing faster than bounded retained state.

This is not RSEC-001 capacity/adversarial-load testing and sets no new policy values or pressure conditions.

### REVIEW_SUPPORT 7 — release/item-4 factual reconciliation

After roughly 3–4 more coherent source/review slices, reconcile `docs/release-security-review-packet.md`, `docs/reviews/release-item4-subgates-20260909.md`, review indexes, and `docs/status.md` only where current facts require changes. Index reachable accepted evidence from reliable recovery, CarrierState, CarrierManager, FairScheduler, observability and adapter reviews as appropriate. Preserve exclusions and exact tested-tree anchors. Do not self-declare independent security approval or item-4 completion.

### REVIEW_LOCAL 8 — broad core-surface inventory before any future `queue exhausted`

Inventory every implemented core crate and release tool for remaining substantial surfaces without dedicated bounded independent challenge. Include crypto API/record boundary versus existing scope, wire/parser versus independent wire review, reliable/Carrier/Session/observe coverage, benchmark/result validators, release scripts, package/operator/reproducibility and CLI/process/output boundaries.

Only classify `queue exhausted` if this repository-wide inventory finds no unreviewed implemented core surface, no concrete defect, no READY review-support and no READY live question, with every remaining item genuinely external/policy/environment/release-authority gated.

## Queue discipline

- Observability HIGH is closed; carrier adapter review is complete. `SessionRuntime` is the next core lane.
- Preserve completed no-finding reviews; do not redo them unless source changes.
- Preserve this deep queue across small commits; do not collapse it to one task.
- A bounded no-finding independent challenge of a previously unreviewed implemented core surface is valid item-4 support, not filler.
- Every concrete defect with an answer fixed by current committed semantics becomes immediate smallest repair -> regression -> push -> exact-tree local gate/provenance -> continue.
- Every code/test repair requires clean exact pushed developer SHA validation with at least `PYTHONDONTWRITEBYTECODE=1 bash scripts/check.sh`, `git diff --check`, clean-tree confirmation and sanitized provenance. Hosted CI is extra cross-evidence, not a wait condition.
- Wire decoder/parser/crypto framing changes require pinned decode fuzz per repository policy; ordinary Session/package/CLI/docs reviews do not mechanically require fuzz.

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