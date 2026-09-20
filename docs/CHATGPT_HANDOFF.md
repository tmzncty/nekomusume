# ChatGPT reviewer handoff — H-I4-088 MemoryCarrier empty-message resource bound HIGH front

## Current repository truth

- Current reachable `main` immediately before this handoff update is exact `7b935a0a528bca315eaf7ed4224d488e4c6abd86` (`docs(review): flag H-I4-088 MemoryCarrier empty-message bound`).
- **H-I4-086 is CLOSED.** Exact `ca629d702cf832c12bf74bdfc5f9d07ebb77ab17` plus `3acba049fee5f9297cc8a4c3867250635c255717` enforce Session outbound drain-before-DeliveryAck; developer-reported provenance is retained in `docs/notes/h-i4-086-provenance-3acba04-20260921.md`.
- **H-I4-087 is CLOSED.** Exact `14e2f520a0fc9d41bcfcb1bd480758008a1dd4ea` + dedicated regressions at `d819d38559d60b6bd99df8a2453321809c046116` enforce the aggregate Session queued-record cap. Developer-reported clean exact-tree provenance is retained in `docs/notes/h-i4-087-provenance-d819d38-20260921.md`.
- **I4-FS2 is CLOSED as a bounded independent current-owner challenge** at `docs/reviews/independent-i4-fs2-session-flow-control-11cdb08-20260921.md`. The pre-H-I4-086/H-I4-087 no-finding portions of the developer FS2 note are superseded. No additional concrete source-decided flow-control/accounting defect was found after those repairs.
- **H-I4-088 is CLOSED** at `26aa4e8036d61da7924d9fc5e587081d8a0f448f`: `MemoryEndpoint::send` now rejects `BufferFull` when the peer queue record count reaches `max_queue_bytes` — zero-length sends cannot grow the queue unbounded while still obeying the byte cap exactly. `recv` releases the record slot so admission resumes. Exact-tree provenance: `docs/notes/h-i4-088-provenance-26aa4e8-20260921.md` — `check.sh` exit 0, `git diff --check` exit 0, clean worktree, 2026-09-20T22:51:53Z → 22:57:50Z, Linux x86_64, rustc 1.98.0.
- The known `SessionRuntime.events` retained-state capacity question remains the existing maintainer/security policy gate; do not invent a history-size/TTL/LRU/capacity value.
- H-I4-085 and I4-FS1 remain closed. H-R9-080/081/082/083/084, Candidate A, Candidate B, R9-11A/B/C/D1/D2/D3/D4, R9-12, final independent R9, Q10/Q11/Q12 remain closed unless repository truth changes.
- Release items 3 and 4 remain incomplete. `RELEASE_CANDIDATE=false`, `PRODUCTION_READY=false`, `FREEZE=false`, `RELEASED=false` remain unchanged.
- **`READY_LIVE: none`.** Do not repeat HY2, warm failover, periodic/soak, package lifecycle, migration-back, endpoint/key migration, IPv6, PLPMTUD or Experimental Track without a new concrete unresolved real-network question.

The external coding agent must synchronize to current `main` and continuously execute dependency-ready work: implementation/review -> focused deterministic tests -> commit -> push -> clean exact-tree local gate/provenance -> next slice. Reviewer cadence is only a check frequency; do not wait for the next reviewer once a slice is closed.

## READY_LOCAL 1 — HIGH: H-I4-088 MemoryCarrier empty-message queue bound

Read exact-current `crates/neko-carrier/src/lib.rs`, D011/current Carrier contract, `SECURITY.md`, developer note `docs/reviews/dev-i4-ad1-memorycarrier-close-resource-20260921.md`, and `docs/reviews/reviewer-h-i4-088-memorycarrier-empty-message-unbounded-20260921.md`.

Current source-decided counterexample:

1. create `MemoryPair` with valid nonzero `max_message_bytes` / `max_queue_bytes`;
2. repeatedly call `send(&[])` on one open endpoint;
3. every call passes the byte-cap check because `message.len()==0` and `queue_bytes` stays unchanged;
4. each call nevertheless appends another queue element, so MemoryCarrier-owned queue memory/record count can grow without bound.

Required bounded repair/regressions:

- preserve currently supported empty-message FIFO semantics unless an independently approved contract change says otherwise;
- add an internal record-count ceiling derived from already committed nonzero limits, or an equivalent source-local bound, without inventing a new public capacity/security policy value;
- retain exact payload-byte accounting for nonempty data and the existing `max_queue_bytes` cap;
- rejection must occur before queue/accounting mutation;
- focused regression: with small existing limits, fill the derived record bound using empty messages, prove the next empty send returns `BufferFull`, receive one empty record, then prove one new empty send succeeds;
- retain close/peer-close ordering, pre-close queued-data drain and no evidence-domain promotion.

No decoder/parser/crypto-framing change is required; do not mechanically run fuzz. After source/test repair, run focused deterministic tests plus final pushed SHA clean exact-tree gate (`PYTHONDONTWRITEBYTECODE=1 bash scripts/check.sh`, `git diff --check`, clean tree) and persist provenance. This HIGH is currently auto-decidable; escalate only if closure unexpectedly requires a new public policy value or architecture change.

## READY_LOCAL 2 — I4-AD1 MemoryCarrier close/error/resource re-challenge remainder

After H-I4-088 is repaired and green, finish the independent MemoryCarrier challenge against the repaired owner: pre-close queued-data drain, local/peer close ordering, idempotence, exact queue byte/record accounting, post-close send/recv false success, concurrency/poison/error paths, and logical `close()` versus Rust ownership/drop semantics. If no additional defect exists, persist one bounded no-finding closure note superseding the developer resource no-finding where H-I4-088 falsified it.

## READY_LOCAL 3 — I4-AD2a UDP adapter close/error/resource re-challenge

Independently challenge exact-current UDP adapter semantics: local logical close, OS socket lifetime/drop, send/recv after close, oversize/errors, nonblocking behavior, listener/socket release and same-address rebind evidence where current local tests exist. Distinguish adapter semantics from process-resource-sampler evidence. No WAN run unless a new unresolved real-network question appears.

## READY_LOCAL 4 — I4-AD2b TCP adapter close/error/resource re-challenge

Challenge exact-current TCP framing/adapter ownership independently: logical close versus `shutdown`/EOF/object drop, buffered/pre-close data, partial/read framing errors, send/recv after terminal state, peer close ordering, accepted-stream/listener lifetime, FD release and local deterministic rebind/process tests. Do not infer Session delivery from TCP reliability or recreate packet ACK semantics.

## READY_LOCAL 5 — I4-PORT cross-platform CLI/process-test semantics

Challenge developer note `11678b2` against current platform guards, Unix identity/permission checks, Linux `/proc`, signals/process groups/`setsid`, socket/process samplers, temp/runtime cleanup, and deterministic unsupported-platform behavior. Keep protocol portability separate from Linux-only release/process evidence.

## READY_LOCAL 6 — I4-CLI-M machine-readable exit / JSON contract

Challenge success/failure/timeout/cleanup-unknown branches, typed/schema result alignment, stdout purity and process-exit consistency across current CLI/process owners. Include matrix probe, authenticated probe/failover/periodic fixtures, process-resource sampler and validators where applicable. Typed failure/incomplete cleanup must not coexist with wrapper success unless explicitly documented.

## READY_LOCAL 7 — I4-CLI-H human-output / stderr contract

Challenge human formatting/error routing independently from machine output: no positive success wording after terminal failure, no structured-output contamination, no misleading cat/success line on typed failure, stable exit semantics, and preservation of the fixed human reachability wording. Avoid cosmetic churn if no defect exists.

## READY_LOCAL 8 — I4-BND algorithmic resource boundedness

Re-check current Recovery/Carrier/Session/CLI loops, maps, queues, retained/live state and peer-controlled iteration under existing limits. Do not run capacity-pressure/adversarial-load benchmarks and do not invent policy values. `SessionRuntime.events` retained-state capacity and D019 source-retention/no-reset remain maintainer/security policy gates.

## READY_LOCAL 9 — item-4 factual reconciliation after coherent closure group

After H-I4-088 + MemoryCarrier remainder + UDP + TCP adapter reviews (or an equivalent 3–4 coherent slice group), reconcile item-4 coverage and release/evidence facts. Preserve valid no-finding notes, explicitly supersede false evidence, and do not mark item 4 complete unless repository-wide independent-review truth supports it.

## READY_LOCAL 10 — repository-wide refill / uncovered current owners

Inventory all implemented core surfaces against reachable dedicated bounded review on the exact relevant owner tree. Re-evaluate the full 13-surface reviewer inventory. Do not duplicate unchanged owners merely for count, but do not declare queue exhaustion while an implemented core surface lacks dedicated independent challenge.

## READY_LOCAL 11 — release-packet / evidence-boundary consistency spot-check

After the next coherent closure group, compare `docs/release-security-review-packet.md`, `docs/status.md`, `IMPLEMENTATION_PLAN.md`, current review notes and exact reachable owners. Challenge only factual/provenance boundaries (developer-local vs reviewer-local vs hosted vs live, superseded findings, items 3/4, policy gates); do not turn this into prose churn or a release decision.

## READY_LOCAL 12 — conditional live question only

Only if new code/instrumentation/hypothesis/path condition creates a specific unresolved self-owned real-network question, classify it against standing authorization and VPS rental priority. Ordinary bounded self-owned TCP/UDP work inside standing authorization needs no per-run approval. Do not manufacture live work merely because the VPS is rented.

## Surfaces deliberately not duplicated unless repository truth changes

- `neko-reliable` Recovery, `CarrierState`, Concurrent Carrier Manager/migration-back and the R9 SessionRuntime terminal/resource surfaces have fresh independent challenges; H-I4-086/H-I4-087 plus the current FS2 note cover the distinct Session flow-control seam.
- `neko-observe` has no material post-review owner change; Candidate-B closure remains applicable.
- `scripts/release` has no material post-review owner change requiring synthetic repetition; process-resource sampler was separately repaired/reviewed through H-R9-082/083/084 and R9-11D.
- Release packet boundaries were reconciled by Q10/Q11/Q12; revisit after the next coherent closure group or material owner change, not after each small commit.

## Stop / escalation conditions

Continue repair/review -> tests -> commit -> push -> next dependency-ready slice without waiting for reviewer cadence. Stop/escalate only for an unresolved BLOCKER/HIGH that cannot be auto-decided, core Session/Carrier/ACK/crypto/wire architecture change, D019/policy-value choice, destructive/canonical migration, out-of-standing-authorization operation, third-party/production/new-credential permission, maintainer-selected adversarial-load/benchmark conditions, or entry into a genuinely new release stage. H-I4-088 as currently scoped is auto-decidable through the normal local workflow.
