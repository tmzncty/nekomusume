# ChatGPT reviewer handoff — H-I4-085 active build/native-surface truth front

## Current repository truth

- Current reachable `main` before this handoff update is `a7ea3819f5bb33f4b33f0ab12d5babc08bf71edd` (`docs(review): keep H-I4-085 open for active build/native surface`). The latest executable source/test change remains exact `8cbd9af39a600f4ddd5fbc50208791e8584c6d74` (`fix(bench): H-R9-084 sampler exit status follows terminal cleanup truth`); later reachable commits through this handoff are review/evidence/reconciliation/navigation docs unless repository truth shows a newer executable change.
- **H-I4-085 is CLOSED** at `250c4d2` + `97e282d`: the I4-BLD note now truthfully records the active build/native surface — `snow 0.10.0` ships `build.rs` (`rustc_version` build-dep) and its default features resolve `ring 0.17.14` which declares `links = "ring_core_0_17_14_"`, a `cc` build-dep, and C/assembly native inputs. `cargo tree -e normal,build` and `-e features,normal,build` evidence is persisted; workspace members ship no `build.rs`/`links`, transitive production closure does.
- H-I4-085 is an **evidence-truth** finding, not a claim that `snow`/`ring` is unsafe and not an instruction to replace crypto/build dependencies. Do not change dependencies/features merely to make old prose true. Use a dependency view that preserves the normal path into active transitive packages while exposing their build edges/features/native hooks.
- New developer/review commits inspected after the original finding: `c4925b6` (correct direct `snow` classification), `803bee3` (locked Cargo-tree evidence but incomplete build/native classification), and `9adf1d2` (I4-FS1 wording repair: drained/inert rather than nonexistent scheduler close/terminal semantics). `9adf1d2` is useful later-lane evidence but does not close H-I4-085.
- **H-R9-080 / H-R9-081 / H-R9-082 / H-R9-083 / H-R9-084 remain CLOSED** at their reachable repair/review anchors. Candidate A (future/never-sent ACK) and Candidate B (mixed queue/terminal observability projection) remain closed by current code/tests.
- **R9-11A/B/C/D1/D2/D3/D4, R9-12 provenance, final independent R9, Q10, Q11 and Q12 remain CLOSED** at their reachable notes. The exact `8cbd9af` clean exact-tree gate remains persisted as developer-reported provenance: `PYTHONDONTWRITEBYTECODE=1 bash scripts/check.sh` exit 0, `git diff --check` exit 0, clean worktree, 2026-09-20T15:55:57Z -> 16:01:40Z, Linux x86_64, rustc 1.98.0. It is not reviewer-local execution or hosted CI.
- The repository-wide item-4 refill was persisted at `bc7f9479fbe8c277f0b5319065b84dd55cfe9a85`. Developer bounded notes now exist for I4-FS1 (`dab0be3`, `c3fe145`, wording correction `9adf1d2`), I4-FS2 (`051f588`), I4-AD1 (`9288b83`), I4-AD2 (`3ad1de6`), I4-BLD (`e11c1d2` plus `c4925b6`/`803bee3`) and I4-PORT (`11678b2`). **I4-BLD remains reopened by H-I4-085 and is not accepted closed.** The other developer notes remain useful support but still require current reviewer challenge/reconciliation before promotion as independent item-4 closure.
- `README.md`, `AGENTS.md`, `ROADMAP.md`, `IMPLEMENTATION_PLAN.md`, `SECURITY.md`, `docs/status.md`, standing VPS authorization, rental-window priority, decisions, carrier architecture, Session v0 and release packet continue to keep release items 3 and 4 incomplete. `RELEASE_CANDIDATE=false`, `PRODUCTION_READY=false`, `FREEZE=false`, `RELEASED=false` are unchanged.
- **`READY_LIVE: none`.** Do not repeat HY2, warm failover, periodic/soak, package lifecycle, migration-back, endpoint/key migration, IPv6, PLPMTUD or Experimental Track without a new code/instrumentation/hypothesis/path condition creating a concrete unresolved self-owned real-network question.

The external coding agent must synchronize to current `main` and continuously execute dependency-ready work: review/repair -> focused deterministic tests -> commit -> push -> clean exact-tree local gate/provenance where code/tests/evidence changed -> next slice. Reviewer cadence is only a check frequency. Do not wait for the next reviewer after closing a slice.

## READY_LOCAL 1 — FRONT HIGH: H-I4-085 active transitive build/native-surface truth

Read exact-current workspace/crate manifests, `Cargo.lock`, `docs/reviews/dev-i4-bld-dependency-build-surface-20260921.md`, the original finding `docs/reviews/reviewer-h-i4-085-dependency-evidence-truth-20260921.md`, and the continuation finding `docs/reviews/reviewer-h-i4-085-build-hook-reopen-20260921.md`.

Required closure:

1. run exact-current locked dependency inspection with a view that preserves normal dependency paths while exposing active transitive build edges/features, e.g. `cargo metadata --locked --format-version 1` without `--no-deps`, `cargo tree --locked -p neko-crypto -e normal,build`, `cargo tree --locked -p neko-crypto -e features,normal,build`, plus a dev-edge view or an equivalent exact resolved-classification method;
2. correct/supersede the remaining false "no build/native hooks" claim and truthfully distinguish direct normal, transitive normal, dev-only, active default/features, build-script/build-dependency and native/`links` surfaces;
3. explicitly establish current `snow 0.10.0` normal/default-feature use, `snow` build script + `rustc_version`, and active resolved `ring 0.17.14` build script + `links` + `cc`/native surface from exact-version tooling/upstream manifests rather than lockfile guesswork;
4. keep workspace-member facts separate from transitive dependency facts: it is acceptable to say no workspace member has a local `build.rs` if verified, but not that the active production closure has no build/native hooks;
5. do not change dependencies/default features/crypto selection/build policy merely to make old prose true; if a separate concrete correctness/security/build defect appears and current policy already decides the answer, make only the smallest repair/regression;
6. final pushed closure SHA gets the normal clean exact-tree developer-local gate/provenance: `PYTHONDONTWRITEBYTECODE=1 bash scripts/check.sh`, `git diff --check`, clean tree, UTC interval, OS/arch, Rust stable, exact reachable SHA.

No decoder/parser/framing change is implicated, so do not mechanically fuzz. No live run is relevant. After truthful closure and gate/provenance, continue immediately to the next lane.

## READY_LOCAL 2 — I4-FS1 reviewer re-challenge: FairScheduler

Independently challenge the current `FairScheduler` owner/tests rather than merely repeating developer notes `dab0be3` / `c3fe145` / `9adf1d2`. Verify enqueue/dequeue accounting, cursor/rotation under empty streams, bounded interactive burst versus bulk progress, idempotent `open`, limit rejection before mutation, and the corrected fact that scheduler-inert drained streams are not a separate close/terminal API. If no semantic defect exists, preserve a precise no-finding note; if a concrete defect appears, convert to smallest repair/regression. Do not invent scheduler policy.

## READY_LOCAL 3 — I4-FS2 reviewer re-challenge: multi-stream + flow-control accounting

Challenge exact-current Session and carrier integration for stream/session window interaction, queued/inflight/released bytes, reject-before-mutate, duplicate/terminal behavior, cross-stream isolation and Session-delivery versus Carrier-feedback separation. Use mutation-sensitive focused tests if an uncovered seam is found. Do not redesign Session delivery semantics.

## READY_LOCAL 4 — I4-AD1 reviewer re-challenge: MemoryCarrier close/error/resource semantics

Challenge pre-close queued-data drain, local/peer close ordering, idempotence, queue byte accounting, post-close false success and concurrency/poison/error paths. Keep logical `close()` semantics distinct from Rust ownership/drop semantics; correct evidence wording if it conflates them.

## READY_LOCAL 5 — I4-AD2 reviewer re-challenge: UDP/TCP adapter close/error/resource semantics

Challenge exact-current UDP/TCP adapter close/send/recv/error behavior plus bounded real-socket local process tests. Distinguish logical endpoint close, OS `shutdown`, object drop/FD lifetime, listener release/rebind, and separate process-sampler evidence. Do not turn a documentation wording issue into transport redesign. No WAN run unless a genuinely unresolved real-network question is created.

## READY_LOCAL 6 — I4-PORT reviewer re-challenge: cross-platform CLI/process-test semantics

Challenge developer note `11678b2` against current owners: platform guards, Unix identity checks, Linux `/proc`, signal/process-group/setsid assumptions, temp/runtime cleanup, and deterministic unsupported-platform behavior. Do not claim support the repository does not provide. Keep Linux benchmark/process-evidence scope distinct from protocol portability.

## READY_LOCAL 7 — I4-CLI-M machine-readable exit / JSON contract

Challenge success/failure/timeout/cleanup-unknown branches, typed/schema result alignment, stdout purity and process-exit consistency across current CLI/process owners. Include matrix probe, authenticated probe/failover/periodic fixture surfaces, process-resource sampler and validators where applicable. Typed failure or incomplete cleanup must never coexist with wrapper success unless the documented command contract explicitly separates those concepts. Do not silently redefine versioned JSON field meaning; reconcile schema/docs/source before treating ambiguous fields as evidence.

## READY_LOCAL 8 — I4-CLI-H human-output / stderr contract

Challenge human formatting/error routing independently from machine-readable output: no positive success wording after terminal failure, no structured-output contamination, no misleading cat/success line on typed failure, and stable exit semantics. Avoid cosmetic churn when no defect exists.

## READY_LOCAL 9 — I4-BND algorithmic resource boundedness

Re-check current Recovery/Carrier/Session/CLI loops, maps, queues, retained/live state and peer-controlled iteration under existing limits. Do not run capacity-pressure/adversarial-load benchmarks and do not invent TTL/LRU/history/capacity/security values. `SessionRuntime.events` retained-state capacity remains a maintainer/security policy gate; classify it, do not choose a cap.

## READY_LOCAL 10 — item-4 factual reconciliation after each 3–4 coherent lanes / important repair group

Reconcile item-4 coverage and current release/evidence facts after H-I4-085 plus a coherent review group. Preserve valid no-finding notes, explicitly supersede false evidence text, and remove only lanes actually closed. Do not mark item 4 complete unless repository-wide independent-review truth supports it.

## READY_LOCAL 11 — repository-wide refill / remaining uncovered current owners

After the above lanes, inventory all core surfaces against reachable dedicated bounded review on the exact relevant owner tree. Refill any materially changed or still-unreviewed implemented surface. Do not duplicate unchanged `neko-observe`, release/package owners, or fresh R9 surfaces merely for count; equally, do not declare queue exhaustion while a current implemented core surface lacks dedicated independent challenge.

## READY_LOCAL 12 — conditional live question only

Only if a new code/instrumentation/hypothesis/path condition creates a specific unresolved self-owned real-network question, classify it against `docs/standing-vps-lab-authorization.md` and `docs/vps-rental-window-priority.md`. Ordinary bounded self-owned TCP/UDP work inside standing authorization needs no per-run approval. Do not manufacture live work merely because the VPS is rented.

## Surfaces deliberately not duplicated unless repository truth changes

- `neko-reliable` Recovery, `CarrierState`, Concurrent Carrier Manager/migration-back and `SessionRuntime` have fresh R9 independent challenges on current/relevant owners.
- `neko-observe` owner history shows no post-`8e11de0` source change; its dedicated deep-sweep review plus Candidate-B closure remain applicable.
- `scripts/release` has no material post-review owner change requiring a synthetic repeat; process-resource sampler was separately repaired/reviewed through H-R9-082/083/084 and R9-11D.
- Release-packet factual/evidence boundaries were freshly reconciled by Q10/Q11/Q12.

## Stop / escalation conditions

Continue review/repair -> tests -> commit -> push -> next dependency-ready slice without waiting for reviewer cadence. Stop/escalate only for an unresolved BLOCKER/HIGH that cannot be auto-decided, core Session/Carrier/ACK/crypto/wire architecture change, D019/policy-value choice, destructive/canonical migration, out-of-standing-authorization operation, third-party/production/new-credential permission, maintainer-selected adversarial-load/benchmark conditions, or entry into a genuinely new release stage. H-I4-085 is currently auto-decidable from repository/dependency facts and therefore should be repaired without maintainer interruption.
