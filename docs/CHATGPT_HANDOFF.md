# ChatGPT reviewer handoff — I4-AD2a UDP adapter re-challenge front

## Current repository truth

- Current reachable `main` immediately before this handoff update is exact `8e9051630037a95a977618fe6189aee34473a8c8` (`docs(review): close I4-AD1 MemoryCarrier rechallenge`).
- **H-I4-086 is CLOSED.** Exact `ca629d702cf832c12bf74bdfc5f9d07ebb77ab17` + `3acba049fee5f9297cc8a4c3867250635c255717` enforce outbound drain-before-DeliveryAck; developer-local provenance remains separately labelled.
- **H-I4-087 is CLOSED.** Exact `14e2f520a0fc9d41bcfcb1bd480758008a1dd4ea` + dedicated regressions at `d819d38559d60b6bd99df8a2453321809c046116` enforce the aggregate Session queued-record cap; developer-local provenance remains separately labelled.
- **I4-FS2 is CLOSED** as a bounded independent current-owner challenge at `docs/reviews/independent-i4-fs2-session-flow-control-11cdb08-20260921.md`; the pre-H-I4-086/H-I4-087 no-finding portions of the developer note are superseded.
- **H-I4-088 is CLOSED** at exact `26aa4e8036d61da7924d9fc5e587081d8a0f448f`. `MemoryEndpoint::send` now bounds queue records as well as payload bytes, so repeated empty messages cannot grow retained queue ownership without limit. Developer-reported clean exact-tree provenance is `docs/notes/h-i4-088-provenance-26aa4e8-20260921.md` (`check.sh` 0, `git diff --check` 0, clean tree, Linux x86_64, rustc 1.98.0, UTC 2026-09-20T22:51:53Z→22:57:50Z). GitHub exposes no hosted status/workflow for that SHA; this is absence of hosted evidence, not failure.
- **I4-AD1 MemoryCarrier remainder is CLOSED** as a bounded independent current-owner re-challenge at `docs/reviews/independent-i4-ad1-memorycarrier-rechallenge-d0f4c55-20260921.md`. Source/test challenge found no additional source-decided defect after H-I4-088: pre-mutation close/cap checks hold, byte/record accounting remains coherent, pre-close queued-data drain admits no new post-close data, concurrency is serialized, poison fails closed, and `Drop` is not promoted into an unclaimed RAII-close contract. This reviewer note makes no reviewer-local CI claim.
- The known `SessionRuntime.events` retained-state capacity question remains a maintainer/security policy gate. Do not invent history-size/TTL/LRU/capacity values.
- H-I4-085, I4-FS1, H-R9-080/081/082/083/084, Candidate A, Candidate B, R9-11A/B/C/D1/D2/D3/D4, R9-12, final independent R9, Q10/Q11/Q12 remain closed unless repository truth changes.
- Release items 3 and 4 remain incomplete. `RELEASE_CANDIDATE=false`, `PRODUCTION_READY=false`, `FREEZE=false`, `RELEASED=false` remain unchanged.
- **`READY_LIVE: none`.** Do not repeat HY2, warm failover, periodic/soak, package lifecycle, migration-back, endpoint/key migration, IPv6, PLPMTUD or Experimental Track without a new concrete unresolved real-network question.

The external coding agent must synchronize to current `main` and continuously execute dependency-ready work: independent review/challenge -> focused deterministic tests when applicable -> smallest repair if a concrete source-decided defect is found -> commit -> push -> clean exact-tree gate/provenance -> next slice. Reviewer cadence is only a check frequency; do not wait for the next reviewer once a slice is closed.

## READY_LOCAL 1 — I4-AD2a UDP adapter close/error/resource re-challenge

Read exact-current `crates/neko-carrier/src/lib.rs`, D012/current Carrier contract in `docs/decisions.md`, `SECURITY.md`, relevant prior adapter reviews/tests, and current package/process evidence only where it directly bears on socket lifetime.

Challenge these invariants independently on the current owner:

- local logical close is idempotent and prevents later send success;
- recv after local logical close cannot fabricate payload/evidence;
- oversize send fails before socket mutation; oversize receive/truncation behavior is bounded and truthful;
- nonblocking `WouldBlock` remains distinct from logical close and I/O failure;
- connected loopback endpoint does not claim peer-close semantics UDP cannot prove;
- OS socket lifetime is Rust ownership/drop, not merely the logical closed flag; distinguish `close()` from actual FD release;
- local deterministic listener/socket release or same-address rebind evidence, where current tests/process fixtures already expose it, must not be conflated with process-resource-sampler truth;
- no adapter observation may promote itself into SessionDelivery, PathValidated, ACK or release/WAN evidence.

If source reasoning or a focused deterministic test exposes a concrete defect whose correct answer is already fixed by D012/current code contract, perform the smallest repair and regression, then run final pushed-SHA clean exact-tree provenance. If no defect exists, persist one bounded no-finding note listing exact owners/tests/exclusions. Do not manufacture WAN work; `READY_LIVE` remains none unless this review creates a genuinely new unresolved real-network question.

## READY_LOCAL 2 — I4-AD2b TCP adapter close/error/resource re-challenge

Challenge exact-current TCP framing/adapter ownership independently: logical close versus `shutdown`/EOF/object drop, buffered/pre-close data, partial/read framing errors, send/recv after terminal state, peer close ordering, accepted-stream/listener lifetime, FD release and local deterministic rebind/process tests. Do not infer Session delivery from TCP reliability or recreate packet ACK semantics.

If the review touches TCP framing/decoder behavior materially, apply the repository's parser/fuzz rule using the pinned fuzz toolchain. Pure close/resource fixes that do not alter decoder/framing need no mechanical fuzz run.

## READY_LOCAL 3 — I4-PORT cross-platform CLI/process-test semantics

Challenge developer portability note/history against exact-current platform guards, Unix identity/permission checks, Linux `/proc`, signals/process groups/`setsid`, socket/process samplers, temp/runtime cleanup, and deterministic unsupported-platform behavior. Keep protocol portability separate from Linux-only release/process evidence.

## READY_LOCAL 4 — I4-CLI-M machine-readable exit / JSON contract

Challenge success/failure/timeout/cleanup-unknown branches, typed/schema result alignment, stdout purity and process-exit consistency across current CLI/process owners. Include matrix probe, authenticated probe/failover/periodic fixtures, process-resource sampler and validators where applicable. Typed failure/incomplete cleanup must not coexist with wrapper success unless explicitly documented.

## READY_LOCAL 5 — I4-CLI-H human-output / stderr contract

Challenge human formatting/error routing independently from machine output: no positive success wording after terminal failure, no structured-output contamination, no misleading cat/success line on typed failure, stable exit semantics, and preservation of the fixed human reachability wording. Avoid cosmetic churn if no defect exists.

## READY_LOCAL 6 — I4-BND algorithmic resource boundedness

Re-check exact-current Recovery/Carrier/Session/CLI loops, maps, queues, retained/live state and peer-controlled iteration under existing limits. Do not run capacity-pressure/adversarial-load benchmarks and do not invent policy values. `SessionRuntime.events` retained-state capacity and D019 source-retention/no-reset remain maintainer/security policy gates rather than implementation filler.

## READY_LOCAL 7 — item-4 factual reconciliation after adapter/CLI closure group

After UDP + TCP adapter reviews plus one or two adjacent coherent slices, reconcile item-4 coverage and release/evidence facts. Preserve valid no-finding notes, explicitly supersede false evidence, and do not mark item 4 complete unless repository-wide independent-review truth supports it.

## READY_LOCAL 8 — repository-wide refill / uncovered current owners

Inventory all implemented core surfaces against reachable dedicated bounded review on the exact relevant owner tree. Re-evaluate all 13 required surfaces. Do not duplicate unchanged owners merely for count, but do not declare queue exhaustion while an implemented core surface lacks a dedicated independent bounded challenge.

## READY_LOCAL 9 — release-packet / evidence-boundary consistency spot-check

After the next coherent closure group, compare `docs/release-security-review-packet.md`, `docs/status.md`, `IMPLEMENTATION_PLAN.md`, current review notes and exact reachable owners. Challenge only factual/provenance boundaries: developer-local vs reviewer-local vs hosted vs live, superseded findings, items 3/4 and policy gates. Do not turn this into prose churn or a release decision.

## READY_LOCAL 10 — conditional live question only

Only if new code/instrumentation/hypothesis/path condition creates a specific unresolved self-owned real-network question, classify it against `docs/standing-vps-lab-authorization.md` and VPS rental priority. Ordinary bounded self-owned TCP/UDP work inside standing authorization needs no per-run approval. Do not manufacture live work merely because the VPS is rented.

## Surfaces deliberately not duplicated unless repository truth changes

- `neko-reliable` Recovery, `CarrierState`, Concurrent Carrier Manager/migration-back and the R9 SessionRuntime terminal/resource surfaces have fresh independent challenges; H-I4-086/H-I4-087 plus the current FS2 note cover the distinct Session flow-control seam.
- MemoryCarrier is freshly re-closed by `docs/reviews/independent-i4-ad1-memorycarrier-rechallenge-d0f4c55-20260921.md` after H-I4-088.
- `neko-observe` has no material post-review owner change; Candidate-B closure remains applicable.
- `scripts/release` has no material post-review owner change requiring synthetic repetition; process-resource sampler was separately repaired/reviewed through H-R9-082/083/084 and R9-11D.
- Release packet boundaries were reconciled by Q10/Q11/Q12; revisit after the next coherent closure group or material owner change, not after each small commit.

## Stop / escalation conditions

Continue review/repair -> tests -> commit -> push -> next dependency-ready slice without waiting for reviewer cadence. Stop/escalate only for an unresolved BLOCKER/HIGH that cannot be auto-decided, core Session/Carrier/ACK/crypto/wire architecture change, D019/policy-value choice, destructive/canonical migration, out-of-standing-authorization operation, third-party/production/new-credential permission, maintainer-selected adversarial-load/benchmark conditions, or entry into a genuinely new release stage.
