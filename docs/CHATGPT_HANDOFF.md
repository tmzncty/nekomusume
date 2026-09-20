# ChatGPT reviewer handoff — H-I4-085 exact-tree provenance front; I4-FS1 independently closed

## Current repository truth

- Current reachable `main` before this handoff update is exact `d50d46f92cf74a94d8ca046f3fe7a552fa191a0c` (`docs(review): keep H-I4-085 open for exact-tree provenance`). The latest executable source/test change remains exact `8cbd9af39a600f4ddd5fbc50208791e8584c6d74` (`fix(bench): H-R9-084 sampler exit status follows terminal cleanup truth`); all later reachable commits through this handoff are review/evidence/reconciliation/navigation docs unless repository truth shows a newer executable change.
- **H-I4-085 remains OPEN / HIGH only for release-item-4 exact-tree provenance.** The substantive dependency/build evidence truth is repaired at reachable `250c4d2` + `97e282d`: the I4-BLD note now truthfully records `snow 0.10.0` as a normal/default-feature production dependency with its `rustc_version` build script, and active resolved `ring 0.17.14` with `build.rs`, `links = "ring_core_0_17_14_"`, `cc`, and C/assembly native inputs. Workspace members themselves ship no local `build.rs`/`links`; the transitive production closure does.
- The remaining H-I4-085 gap is precise: the repository contract requires READY_LOCAL docs-evidence changes to have a developer-local clean exact-tree gate/provenance on the corrected reachable closure tree. The only persisted clean exact-tree gate still points to executable exact `8cbd9af`, which predates the H-I4-085 evidence corrections. `docs/reviews/reviewer-h-i4-085-exact-tree-provenance-gap-20260921.md` at exact `d50d46f` defines the minimal closure. Do **not** change dependencies/features/crypto policy to close this; run/persist the required exact-tree gate only.
- **I4-FS1 FairScheduler is independently CLOSED** by `docs/reviews/reviewer-i4-fs1-fair-scheduler-44a0073-20260921.md` at reachable exact `9e483618554acbd3fa3544e9a17e3ec6436b1a6a`. The current scheduler owner was re-read against current source/tests and the developer notes `dab0be3` / `c3fe145` / `9adf1d2`. No concrete semantic defect was found in idempotent `open`, reject-before-mutate stream/session accounting, cursor/rotation over drained-inert streams, or the bounded Interactive/Bulk preference. The scheduler has no close/terminal API; drained streams are scheduler-inert. This reviewer pass was source/spec/evidence reasoning only and did not claim reviewer-local test execution.
- **H-R9-080 / H-R9-081 / H-R9-082 / H-R9-083 / H-R9-084 remain CLOSED** at their reachable repair/review anchors. Candidate A (future/never-sent ACK) and Candidate B (mixed queue/terminal observability projection) remain closed by current code/tests.
- **R9-11A/B/C/D1/D2/D3/D4, R9-12 provenance, final independent R9, Q10, Q11 and Q12 remain CLOSED** at their reachable notes. The exact `8cbd9af` clean exact-tree gate remains valid **developer-reported provenance for that exact tree only**: `PYTHONDONTWRITEBYTECODE=1 bash scripts/check.sh` exit 0, `git diff --check` exit 0, clean worktree, 2026-09-20T15:55:57Z -> 16:01:40Z, Linux x86_64, rustc 1.98.0. It is not reviewer-local execution, hosted CI, or provenance for later docs/evidence trees.
- The repository-wide item-4 refill remains active. Developer bounded notes exist for I4-FS2 (`051f588`), I4-AD1 (`9288b83`), I4-AD2 (`3ad1de6`) and I4-PORT (`11678b2`); they are useful support but require current independent reviewer challenge/reconciliation before promotion as item-4 closure.
- `README.md`, `AGENTS.md`, `ROADMAP.md`, `IMPLEMENTATION_PLAN.md`, `SECURITY.md`, `docs/status.md`, standing VPS authorization, rental-window priority, decisions, carrier architecture, Session v0 and the release packet continue to keep release items 3 and 4 incomplete. `RELEASE_CANDIDATE=false`, `PRODUCTION_READY=false`, `FREEZE=false`, `RELEASED=false` are unchanged.
- **`READY_LIVE: none`.** Do not repeat HY2, warm failover, periodic/soak, package lifecycle, migration-back, endpoint/key migration, IPv6, PLPMTUD or Experimental Track without a new code/instrumentation/hypothesis/path condition creating a concrete unresolved self-owned real-network question.

The external coding agent must synchronize to current `main` and continuously execute dependency-ready work: review/repair -> focused deterministic tests -> commit -> push -> clean exact-tree local gate/provenance where code/tests/evidence changed -> next slice. Reviewer cadence is only a check frequency. Do not wait for the next reviewer after closing a slice.

## READY_LOCAL 1 — HIGH: H-I4-085 exact-tree provenance closure

Read current `docs/reviews/dev-i4-bld-dependency-build-surface-20260921.md`, both original H-I4-085 findings, and `docs/reviews/reviewer-h-i4-085-exact-tree-provenance-gap-20260921.md`.

The substantive build/native classification is already corrected. Do only the remaining provenance closure:

1. synchronize to the exact current reachable closure/navigation tree carrying this handoff;
2. in a safe clean checkout/worktree run `PYTHONDONTWRITEBYTECODE=1 bash scripts/check.sh` and `git diff --check`, then verify the tree is clean;
3. persist the exact tested reachable SHA, UTC start/end, both exit codes, OS/arch, Rust stable version and clean-tree state, without secrets/private topology/credentials/unnecessary absolute paths;
4. a later provenance-only commit may record the clean gate for its reachable tested parent; never claim the provenance-only commit itself was tested unless it actually was;
5. no decoder/parser/framing change exists, so do not mechanically fuzz; no live run is relevant;
6. once the provenance anchor is reachable, mark H-I4-085 CLOSED and immediately continue to READY_LOCAL 2 without waiting for reviewer cadence.

This HIGH is auto-decidable from the repository contract and local gate. Do not request maintainer policy input unless the gate itself exposes a genuinely non-auto-decidable blocker.

## CLOSED THIS REVIEW — I4-FS1 FairScheduler current-owner re-challenge

Independent closure: `docs/reviews/reviewer-i4-fs1-fair-scheduler-44a0073-20260921.md` at exact `9e483618554acbd3fa3544e9a17e3ec6436b1a6a`.

No finding. Scope covered enqueue/dequeue accounting, cursor/rotation with inert streams, bounded Interactive/Bulk progress, idempotent `open`, limit rejection before mutation, and the absence of a close/terminal API. No reviewer-local test command was claimed; prior exact-`91f8cd8` independent test execution and developer exact-`8cbd9af` clean-tree provenance remain separate evidence.

## READY_LOCAL 2 — I4-FS2 reviewer re-challenge: multi-stream + flow-control accounting

Challenge exact-current Session and carrier integration for stream/session window interaction, queued/inflight/released bytes, reject-before-mutate, duplicate/terminal behavior, cross-stream isolation and Session-delivery versus Carrier-feedback separation. Read developer note `051f588` as support, but independently inspect current owners/tests. Use mutation-sensitive focused tests if an uncovered seam is found. If current committed semantics decide a concrete defect, make the smallest repair/regression; otherwise persist a precise no-finding note. Do not redesign Session delivery semantics.

## READY_LOCAL 3 — I4-AD1 reviewer re-challenge: MemoryCarrier close/error/resource semantics

Challenge developer note `9288b83` against current MemoryCarrier owner/tests: pre-close queued-data drain, local/peer close ordering, idempotence, queue byte accounting, post-close false success and concurrency/poison/error paths. Keep logical `close()` semantics distinct from Rust ownership/drop semantics; correct evidence wording if it conflates them.

## READY_LOCAL 4 — I4-AD2 reviewer re-challenge: UDP/TCP adapter close/error/resource semantics

Challenge developer note `3ad1de6` against exact-current UDP/TCP adapter close/send/recv/error behavior plus existing bounded real-socket local process tests. Distinguish logical endpoint close, OS `shutdown`, object drop/FD lifetime, listener release/rebind, and separate process-sampler evidence. Do not turn a documentation wording issue into transport redesign. No WAN run unless a genuinely unresolved real-network question is created.

## READY_LOCAL 5 — I4-PORT reviewer re-challenge: cross-platform CLI/process-test semantics

Challenge developer note `11678b2` against current owners: platform guards, Unix identity checks, Linux `/proc`, signal/process-group/setsid assumptions, temp/runtime cleanup, and deterministic unsupported-platform behavior. Do not claim support the repository does not provide. Keep Linux benchmark/process-evidence scope distinct from protocol portability.

## READY_LOCAL 6 — I4-CLI-M machine-readable exit / JSON contract

Challenge success/failure/timeout/cleanup-unknown branches, typed/schema result alignment, stdout purity and process-exit consistency across current CLI/process owners. Include matrix probe, authenticated probe/failover/periodic fixture surfaces, process-resource sampler and validators where applicable. Typed failure or incomplete cleanup must never coexist with wrapper success unless the documented command contract explicitly separates those concepts. Do not silently redefine versioned JSON field meaning; reconcile schema/docs/source before treating ambiguous fields as evidence.

## READY_LOCAL 7 — I4-CLI-H human-output / stderr contract

Challenge human formatting/error routing independently from machine-readable output: no positive success wording after terminal failure, no structured-output contamination, no misleading cat/success line on typed failure, and stable exit semantics. Avoid cosmetic churn when no defect exists.

## READY_LOCAL 8 — I4-BND algorithmic resource boundedness

Re-check current Recovery/Carrier/Session/CLI loops, maps, queues, retained/live state and peer-controlled iteration under existing limits. Do not run capacity-pressure/adversarial-load benchmarks and do not invent TTL/LRU/history/capacity/security values. `SessionRuntime.events` retained-state capacity remains a maintainer/security policy gate; classify it, do not choose a cap.

## READY_LOCAL 9 — item-4 factual reconciliation after the next coherent group

After H-I4-085 provenance + I4-FS2 + I4-AD1/I4-AD2 (or another coherent 3–4 lane group), reconcile item-4 coverage and current release/evidence facts. Preserve valid no-finding notes, explicitly supersede false evidence text, and remove only lanes actually closed. Do not mark item 4 complete unless repository-wide independent-review truth supports it.

## READY_LOCAL 10 — repository-wide refill / remaining uncovered current owners

Inventory all core surfaces against reachable dedicated bounded review on the exact relevant owner tree. Refill any materially changed or still-unreviewed implemented surface. Do not duplicate unchanged `neko-observe`, release/package owners, or fresh R9 surfaces merely for count; equally, do not declare queue exhaustion while a current implemented core surface lacks dedicated independent challenge.

## READY_LOCAL 11 — conditional live question only

Only if a new code/instrumentation/hypothesis/path condition creates a specific unresolved self-owned real-network question, classify it against `docs/standing-vps-lab-authorization.md` and `docs/vps-rental-window-priority.md`. Ordinary bounded self-owned TCP/UDP work inside standing authorization needs no per-run approval. Do not manufacture live work merely because the VPS is rented.

## Surfaces deliberately not duplicated unless repository truth changes

- `neko-reliable` Recovery, `CarrierState`, Concurrent Carrier Manager/migration-back and `SessionRuntime` have fresh R9 independent challenges on current/relevant owners.
- `neko-observe` owner history shows no post-`8e11de0` source change; its dedicated deep-sweep review plus Candidate-B closure remain applicable.
- `scripts/release` has no material post-review owner change requiring a synthetic repeat; process-resource sampler was separately repaired/reviewed through H-R9-082/083/084 and R9-11D.
- Release-packet factual/evidence boundaries were freshly reconciled by Q10/Q11/Q12.

## Stop / escalation conditions

Continue review/repair -> tests -> commit -> push -> next dependency-ready slice without waiting for reviewer cadence. Stop/escalate only for an unresolved BLOCKER/HIGH that cannot be auto-decided, core Session/Carrier/ACK/crypto/wire architecture change, D019/policy-value choice, destructive/canonical migration, out-of-standing-authorization operation, third-party/production/new-credential permission, maintainer-selected adversarial-load/benchmark conditions, or entry into a genuinely new release stage. The current H-I4-085 provenance gap is auto-decidable and should be closed by the normal local gate/provenance workflow, not escalated.