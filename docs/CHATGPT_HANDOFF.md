# ChatGPT reviewer handoff — H-I4-087 aggregate Session queued-record cap HIGH front

## Current repository truth

- Current reachable `main` immediately before this handoff update is exact `4ba6ea6a2a9eb766ff5b0170b33b94842d9c3030` (`docs(review): flag H-I4-087 aggregate Session queue-record cap`).
- Developer source/test repair for H-I4-086 is exact `ca629d702cf832c12bf74bdfc5f9d07ebb77ab17` plus companion test-ordering/cleanup exact `3acba049fee5f9297cc8a4c3867250635c255717`. Later `8efb82b`/`6b24a16` are navigation updates; they do not supersede the executable owner.
- **H-I4-086 is CLOSED.** `SessionRuntime` now tracks per-stream drained/sent ends; `delivery_ack` rejects `end > sent`, so queued-but-undrained bytes cannot release Session/stream send-window credit or advance confirmation evidence. Regressions cover ACK-before-drain, a mixed drained/queued prefix, cross-stream isolation, resumed-session drain-before-ACK ordering, and terminal cleanup of the new sent bookkeeping.
- Developer-reported clean exact-tree provenance for H-I4-086 is anchored to reachable exact `3acba049fee5f9297cc8a4c3867250635c255717` in `docs/notes/h-i4-086-provenance-3acba04-20260921.md`: `PYTHONDONTWRITEBYTECODE=1 bash scripts/check.sh` exit 0, `git diff --check` exit 0, clean worktree, UTC 2026-09-20T21:05:25Z -> 21:11:30Z, Linux x86_64, rustc 1.98.0 stable. This is developer-reported local provenance, not reviewer-local or hosted CI. No hosted status/workflow run was visible for exact `3acba04`; absence of hosted evidence is not a failure.
- **H-I4-087 is CLOSED** at `d819d38559d60b6bd99df8a2453321809c046116`: `SessionRuntime::queue_send` and `SessionRuntime::receive` now both reject before mutation when the aggregate queued-record count (`send.len() + recv.len()`) would exceed `max_queue_records` — the configured cap is a single runtime-level bound, not two independent directional caps. Three new tests cover send-then-receive/receive-then-send fill orders and positive slot-release. Exact-tree provenance: `docs/notes/h-i4-087-provenance-d819d38-20260921.md` — `check.sh` exit 0 (re-run after transient `known-fd` bind race), `git diff --check` exit 0, clean worktree, 2026-09-20T22:01:25Z → 22:12:25Z, Linux x86_64, rustc 1.98.0.
- H-I4-085 and I4-FS1 remain closed at their reachable anchors. H-R9-080/081/082/083/084, Candidate A, Candidate B, R9-11A/B/C/D1/D2/D3/D4, R9-12, final independent R9, Q10/Q11/Q12 remain closed unless repository truth changes.
- Release items 3 and 4 remain incomplete. `RELEASE_CANDIDATE=false`, `PRODUCTION_READY=false`, `FREEZE=false`, `RELEASED=false` remain unchanged.
- **`READY_LIVE: none`.** Do not repeat HY2, warm failover, periodic/soak, package lifecycle, migration-back, endpoint/key migration, IPv6, PLPMTUD or Experimental Track without a new code/instrumentation/hypothesis/path condition creating a concrete unresolved self-owned real-network question.

The external coding agent must synchronize to current `main` and continuously execute dependency-ready work: implementation/review -> focused deterministic tests -> commit -> push -> clean exact-tree local gate/provenance for changed source/test/evidence trees -> next slice. Reviewer cadence is only a check frequency; do not wait for the next reviewer once a slice is closed.

## READY_LOCAL 1 — HIGH: H-I4-087 aggregate queued-record cap

Read exact-current `crates/neko-session/src/lib.rs`, `docs/specs/nekomusume-session-v0.md`, `SECURITY.md`, developer I4-FS2 note `docs/reviews/dev-i4-fs2-multistream-flow-control-20260921.md`, and `docs/reviews/reviewer-h-i4-087-session-aggregate-queue-record-cap-20260921.md`.

Current source-decided counterexample:

1. choose limits with `max_queue_records = 1` and byte/window limits large enough not to interfere;
2. open one stream;
3. `queue_send(stream, b"a", ...)` succeeds because it checks only `send.len()`;
4. `receive(InboundRecord { stream, offset: 0, data: b"b" }, ...)` also succeeds because it checks only `recv.len()`;
5. `queued_records()` is now 2 although the configured runtime cap is 1. The reverse admission order has the same defect.

Required bounded repair/regressions:

- both `queue_send` and `receive` must reject **before mutation** when the aggregate queued-record count would exceed `max_queue_records`;
- add both direction-order negatives: send-full then receive, and receive-full then send;
- verify rejected admission leaves queues, `queued_bytes`, relevant send/receive window accounting, offsets/watermarks, and event state consistent with the already-documented rejection contract;
- add a positive slot-release control: after `pop_send` or `pop_receive` removes one queued record, the opposite direction can consume the freed aggregate slot if all other limits permit it;
- preserve current byte-window semantics: queued + sent-unacked outbound bytes still consume send credit; `pop_send` does not release delivery credit; `pop_receive` continues releasing receive-window credit;
- do not invent separate send/recv record caps or any new policy value.

No decoder/parser/crypto-framing change is required, so do not mechanically run fuzz. After repair run focused deterministic tests, then final pushed SHA clean exact-tree gate (`PYTHONDONTWRITEBYTECODE=1 bash scripts/check.sh`, `git diff --check`, clean tree) and persist exact provenance. This HIGH is auto-decidable and should not be escalated unless the repair unexpectedly requires a forbidden architecture/policy choice.

## READY_LOCAL 2 — I4-FS2 independent reviewer re-challenge remainder

After H-I4-087 lands and is green, re-read exact-current Session owner/tests and independently challenge the rest of I4-FS2: stream/session byte-window interaction, aggregate queue accounting, queued/sent-unacked/released ownership, reject-before-mutate, duplicate/terminal behavior, cross-stream isolation, receive-window ownership, and Session-delivery versus Carrier-feedback separation. Supersede the pre-H-I4-086/H-I4-087 parts of developer note `dev-i4-fs2-multistream-flow-control-20260921.md`. If no further defect exists, persist a scope-precise no-finding note anchored to the repaired reachable tree.

## READY_LOCAL 3 — I4-AD1 MemoryCarrier close/error/resource re-challenge

Challenge developer note `9288b83` against exact-current MemoryCarrier owner/tests: pre-close queued-data drain, local/peer close ordering, idempotence, queue byte accounting, post-close false success, and concurrency/poison/error paths. Keep logical `close()` semantics distinct from Rust ownership/drop semantics. Repair only concrete source-decided defects; otherwise write bounded no-finding evidence.

## READY_LOCAL 4 — I4-AD2 UDP/TCP adapter close/error/resource re-challenge

Challenge developer note `3ad1de6` against exact-current UDP/TCP adapter close/send/recv/error behavior plus existing bounded real-socket local process tests. Distinguish logical endpoint close, OS shutdown, object drop/FD lifetime, listener release/rebind, and separate process-sampler evidence. No WAN run unless a genuinely new unresolved real-network question is created.

## READY_LOCAL 5 — I4-PORT cross-platform CLI/process-test semantics

Challenge developer note `11678b2` against platform guards, Unix identity checks, Linux `/proc`, signal/process-group/setsid assumptions, temp/runtime cleanup, and deterministic unsupported-platform behavior. Do not claim platform support the repository does not provide; keep Linux benchmark/process evidence distinct from protocol portability.

## READY_LOCAL 6 — I4-CLI-M machine-readable exit / JSON contract

Challenge success/failure/timeout/cleanup-unknown branches, typed/schema result alignment, stdout purity and process-exit consistency across current CLI/process owners. Include matrix probe, authenticated probe/failover/periodic fixture surfaces, process-resource sampler and validators where applicable. Typed failure or incomplete cleanup must never coexist with wrapper success unless the documented command contract explicitly separates them. Do not silently redefine versioned JSON field meaning.

## READY_LOCAL 7 — I4-CLI-H human-output / stderr contract

Challenge human formatting/error routing independently from machine-readable output: no positive success wording after terminal failure, no structured-output contamination, no misleading cat/success line on typed failure, and stable exit semantics. Avoid cosmetic churn when no defect exists.

## READY_LOCAL 8 — I4-BND algorithmic resource boundedness

Re-check current Recovery/Carrier/Session/CLI loops, maps, queues, retained/live state and peer-controlled iteration under existing limits. Do not run capacity-pressure/adversarial-load benchmarks and do not invent TTL/LRU/history/capacity/security values. `SessionRuntime.events` retained-state capacity and D019 source-retention/no-reset remain maintainer/security policy gates; classify them, do not choose values.

## READY_LOCAL 9 — item-4 factual reconciliation after coherent closure group

After H-I4-087 + repaired I4-FS2 + the next two coherent adapter/review lanes, reconcile item-4 coverage and release/evidence facts. Preserve valid no-finding notes, explicitly supersede false evidence text, and remove only lanes actually closed. Do not mark item 4 complete unless repository-wide independent-review truth supports it.

## READY_LOCAL 10 — repository-wide refill / uncovered current owners

Inventory all core surfaces against reachable dedicated bounded review on the exact relevant owner tree. Refill materially changed or still-unreviewed implemented surfaces. Do not duplicate unchanged `neko-observe`, package/release owners, or fresh R9 surfaces merely for count; equally, do not declare queue exhaustion while a current implemented core surface lacks dedicated independent challenge.

## READY_LOCAL 11 — conditional live question only

Only if new code/instrumentation/hypothesis/path condition creates a specific unresolved self-owned real-network question, classify it against `docs/standing-vps-lab-authorization.md` and `docs/vps-rental-window-priority.md`. Ordinary bounded self-owned TCP/UDP work inside standing authorization needs no per-run approval. Do not manufacture live work merely because the VPS is rented.

## Surfaces deliberately not duplicated unless repository truth changes

- `neko-reliable` Recovery, `CarrierState`, Concurrent Carrier Manager/migration-back and the R9 SessionRuntime terminal/resource surfaces have fresh independent challenges on current/relevant owners; H-I4-086/H-I4-087 are distinct I4-FS2 flow-control/resource-accounting seams.
- `neko-observe` has no material post-review owner change; Candidate-B closure remains applicable.
- `scripts/release` has no material post-review owner change requiring synthetic repetition; process-resource sampler was separately repaired/reviewed through H-R9-082/083/084 and R9-11D.
- Release-packet factual/evidence boundaries were reconciled by Q10/Q11/Q12; revisit only after the next coherent closure group or material owner change.

## Stop / escalation conditions

Continue implementation/review -> tests -> commit -> push -> next dependency-ready slice without waiting for reviewer cadence. Stop/escalate only for an unresolved BLOCKER/HIGH that cannot be auto-decided, core Session/Carrier/ACK/crypto/wire architecture change, D019/policy-value choice, destructive/canonical migration, out-of-standing-authorization operation, third-party/production/new-credential permission, maintainer-selected adversarial-load/benchmark conditions, or entry into a genuinely new release stage. H-I4-087 as currently scoped is auto-decidable and should be repaired through the normal local workflow.
