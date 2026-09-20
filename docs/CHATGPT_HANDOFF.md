# ChatGPT reviewer handoff — H-I4-086 Session ACK-before-drain HIGH front

## Current repository truth

- Current reachable `main` immediately before this handoff update is exact `28818d81428f40c106bd8bd331d69eefcd046b7a` (`docs(review): flag H-I4-086 Session ACK before outbound drain`).
- The latest executable source/test change remains exact `8cbd9af39a600f4ddd5fbc50208791e8584c6d74` (`fix(bench): H-R9-084 sampler exit status follows terminal cleanup truth`). All commits reviewed from prior reviewer head `cee89bed3b02909e8e0e5bc9e29e43af7a081d89` through `d7b630f310c504ee439bbcb4bb4c5b9a42a51993` are documentation/review/evidence/provenance/navigation changes; no newer executable owner supersedes the current Session source inspected for H-I4-086.
- **H-I4-085 is CLOSED.** The build/native classification is truthfully repaired in `docs/reviews/dev-i4-bld-dependency-build-surface-20260921.md`: `snow` is a normal production dependency; the transitive production closure includes `snow`'s `rustc_version` build script and active `ring 0.17.14` build/`links`/`cc` C/assembly surface. Developer-reported clean exact-tree provenance is anchored to reachable exact `38379b16ba98eb12188302169edbe9ab9ce02487` in `docs/notes/h-i4-085-provenance-38379b1-20260921.md`: `PYTHONDONTWRITEBYTECODE=1 bash scripts/check.sh` exit 0 on the second run after one recorded transient READY-timeout failure, `git diff --check` exit 0, clean worktree, 2026-09-20T19:50:16Z -> 20:02:30Z, Linux x86_64, rustc 1.98.0 stable. This is developer-reported local provenance, not reviewer-local or hosted CI.
- **I4-FS1 FairScheduler remains independently CLOSED** at reachable exact `9e483618554acbd3fa3544e9a17e3ec6436b1a6a` by `docs/reviews/reviewer-i4-fs1-fair-scheduler-44a0073-20260921.md`. No close/terminal API exists in the scheduler; drained streams are scheduler-inert. The review found no defect in idempotent open, reject-before-mutate accounting, cursor/rotation, or bounded Interactive/Bulk preference.
- **H-I4-086 is CLOSED** at `ca629d702cf832c12bf74bdfc5f9d07ebb77ab17` + `3acba049fee5f9297cc8a4c3867250635c255717`: `SessionRuntime` now tracks per-stream `sent` drained byte ends; `delivery_ack` rejects `end > sent` (`Protocol`), preserving confirmed watermark, window credit, queue contents, and events. `pop_send` marks ranges drained; `clear_runtime_state` releases `sent`. Four new tests cover ack-before-drain, mixed-prefix, cross-stream isolation, and terminal sent-bookkeeping release. Exact-tree provenance for `3acba04`: `docs/notes/h-i4-086-provenance-3acba04-20260921.md` — `check.sh` exit 0, `git diff --check` exit 0, clean worktree, 2026-09-20T21:05:25Z → 21:11:30Z, Linux x86_64, rustc 1.98.0.
- H-R9-080/081/082/083/084, Candidate A (future/never-sent reliable-UDP ACK) and Candidate B (mixed datagram drop observability), R9-11A/B/C/D1/D2/D3/D4, R9-12, final independent R9, Q10/Q11/Q12 remain closed at their reachable anchors unless repository truth changes.
- Release items 3 and 4 remain incomplete. `RELEASE_CANDIDATE=false`, `PRODUCTION_READY=false`, `FREEZE=false`, `RELEASED=false` remain unchanged.
- **`READY_LIVE: none`.** Do not repeat HY2, warm failover, periodic/soak, package lifecycle, migration-back, endpoint/key migration, IPv6, PLPMTUD or Experimental Track without a new code/instrumentation/hypothesis/path condition creating a concrete unresolved self-owned real-network question.

The external coding agent must synchronize to current `main` and continuously execute dependency-ready work: review/repair -> focused deterministic tests -> commit -> push -> clean exact-tree local gate/provenance for changed source/test/evidence trees -> next slice. Reviewer cadence is only a check frequency; do not wait for the next reviewer once a slice is closed.

## READY_LOCAL 1 — HIGH: H-I4-086 Session DeliveryAck must not confirm locally undrained bytes

Read exact-current `crates/neko-session/src/lib.rs`, `docs/specs/nekomusume-session-v0.md`, `docs/carrier-architecture.md`, developer I4-FS2 note `docs/reviews/dev-i4-fs2-multistream-flow-control-20260921.md`, and `docs/reviews/reviewer-h-i4-086-session-ack-before-drain-20260921.md`.

Current counterexample is source-decided and auto-decidable:

1. `queue_send(stream, "abcd")` leaves `abcd` in `send` while incrementing the existing stream/session send-window ownership;
2. before `pop_send()`, current `delivery_ack(stream, 0, 4)` succeeds because it only checks confirmed watermark plus `send_inflight` totals;
3. it releases flow-control credit and advances confirmed evidence even though the local carrier has not drained the record;
4. later `pop_send()` can still return bytes already represented as Session-confirmed.

Required bounded repair/regressions:

- ACK-before-drain fails closed with queue contents, confirmed watermark, stream/session accounting and events unchanged;
- after `pop_send()` returns the range, the same ACK succeeds and releases the existing credit exactly once;
- with two contiguous records queued and only the first drained, an ACK crossing into the still-queued second range fails atomically; ACKing exactly the drained prefix succeeds;
- draining stream A does not make stream B queued bytes ACK-eligible;
- any new sent/drained bookkeeping is cleared by the already-reviewed terminal cleanup paths;
- queued plus sent-unacked bytes must continue to count against the existing stream/session window; `pop_send()` itself must not release delivery credit.

Choose the smallest state/API change compatible with current Session/Carrier separation. Do not redesign ACK architecture, Session delivery semantics, wire/crypto, or invent capacity/TTL/history values. No decoder/parser/crypto-framing change is required, so do not mechanically run fuzz. After repair run focused deterministic tests, then final pushed SHA clean exact-tree gate (`PYTHONDONTWRITEBYTECODE=1 bash scripts/check.sh`, `git diff --check`, clean tree) and persist exact provenance.

This HIGH is an auto-decidable correctness issue; do not escalate to maintainer policy unless the repair unexpectedly requires a core semantic choice forbidden by the repository contract.

## READY_LOCAL 2 — I4-FS2 independent reviewer re-challenge remainder

After H-I4-086 lands and is green, re-read exact-current Session owner/tests and independently challenge the rest of I4-FS2: stream/session window interaction, queued/sent-unacked/released bytes, reject-before-mutate, duplicate/terminal behavior, cross-stream isolation, receive-window ownership, and Session-delivery versus Carrier-feedback separation. Treat developer note `051f588` as support only; supersede its pre-repair window-release claim. If no further defect exists, persist a scope-precise no-finding note anchored to the repaired reachable tree.

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

After H-I4-086 + repaired I4-FS2 + the next two coherent adapter/review lanes, reconcile item-4 coverage and release/evidence facts. Preserve valid no-finding notes, explicitly supersede false evidence text, and remove only lanes actually closed. Do not mark item 4 complete unless repository-wide independent-review truth supports it.

## READY_LOCAL 10 — repository-wide refill / uncovered current owners

Inventory all core surfaces against reachable dedicated bounded review on the exact relevant owner tree. Refill materially changed or still-unreviewed implemented surfaces. Do not duplicate unchanged `neko-observe`, package/release owners, or fresh R9 surfaces merely for count; equally, do not declare queue exhaustion while a current implemented core surface lacks dedicated independent challenge.

## READY_LOCAL 11 — conditional live question only

Only if new code/instrumentation/hypothesis/path condition creates a specific unresolved self-owned real-network question, classify it against `docs/standing-vps-lab-authorization.md` and `docs/vps-rental-window-priority.md`. Ordinary bounded self-owned TCP/UDP work inside standing authorization needs no per-run approval. Do not manufacture live work merely because the VPS is rented.

## Surfaces deliberately not duplicated unless repository truth changes

- `neko-reliable` Recovery, `CarrierState`, Concurrent Carrier Manager/migration-back and the R9 SessionRuntime terminal/resource surfaces have fresh independent challenges on current/relevant owners; H-I4-086 is a distinct FS2 send-window/delivery-evidence seam.
- `neko-observe` has no material post-review owner change; Candidate-B closure remains applicable.
- `scripts/release` has no material post-review owner change requiring synthetic repetition; process-resource sampler was separately repaired/reviewed through H-R9-082/083/084 and R9-11D.
- Release-packet factual/evidence boundaries were reconciled by Q10/Q11/Q12; revisit only after the next coherent closure group or material owner change.

## Stop / escalation conditions

Continue review/repair -> tests -> commit -> push -> next dependency-ready slice without waiting for reviewer cadence. Stop/escalate only for an unresolved BLOCKER/HIGH that cannot be auto-decided, core Session/Carrier/ACK/crypto/wire architecture change, D019/policy-value choice, destructive/canonical migration, out-of-standing-authorization operation, third-party/production/new-credential permission, maintainer-selected adversarial-load/benchmark conditions, or entry into a genuinely new release stage. H-I4-086 as currently scoped is auto-decidable and should be repaired through the normal local workflow.