# ChatGPT reviewer handoff — final R9 + Q10-Q12 closed; repository-wide item-4 refill front

## Current repository truth

- Current reachable executable source/test anchor remains exact `8cbd9af39a600f4ddd5fbc50208791e8584c6d74` (`fix(bench): H-R9-084 sampler exit status follows terminal cleanup truth`). Later reachable commits through the current handoff are review/reconciliation/navigation docs unless repository truth shows a newer executable change.
- **H-R9-080 / H-R9-081 / H-R9-082 / H-R9-083 / H-R9-084 are CLOSED** at their reachable repair/review anchors. Candidate A (future/never-sent ACK) and Candidate B (mixed queue/terminal observability projection) remain closed by current code/tests.
- **R9-11A/B/C/D1/D2/D3/D4 are CLOSED as bounded item-4 support.** The reachable factual reconciliation is `docs/reviews/r9-11a-d-factual-reconciliation-20260920.md`.
- **R9-12 exact-tree developer-local provenance is persisted** at `docs/notes/r9-12-final-provenance-8cbd9af-20260920.md`: exact `8cbd9af`, `PYTHONDONTWRITEBYTECODE=1 bash scripts/check.sh` exit 0, `git diff --check` exit 0, clean worktree, 2026-09-20T15:55:57Z → 16:01:40Z, Linux x86_64, rustc 1.98.0. This is developer-reported/persisted local provenance, not reviewer-local execution or hosted CI.
- **Dedicated final independent R9 bounded review is CLOSED / no-finding** at `docs/reviews/independent-final-r9-bounded-review-8cbd9af-20260921.md`, commit `1d0b7f876998bc96bfaf5a68baacbbd6d5ff30f0`. It re-challenged Recovery ACK/loss ownership, receiver ACK evidence separation, Session logical/runtime ownership, Carrier readiness/generation, uncertain retention/TCP replay/dedup/migration-back, and process result/terminal cleanup. No concrete defect was found.
- **Q10 factual reconciliation is CLOSED / no-finding** at `docs/reviews/q10-release-packet-status-factual-reconciliation-20260921.md`, commit `e1f0534f7336107a81a66618968bac515da4d82a`. Current status/plan/release-packet boundaries remain conservative; packet coverage lag does not overclaim later R9 evidence.
- **Q11 release-evidence-boundary reconciliation is CLOSED / no-finding** at `docs/reviews/q11-release-evidence-boundary-reconciliation-20260921.md`, commit `bf5b0e7c2cbacd601e6c94b6e8fecd97c06303c5`. Developer-local, hosted, WAN/live, performance, security and release evidence remain separate. `READY_LIVE: none` remains authoritative.
- **Q12 governance/release-state reconciliation is CLOSED / no-finding** at `docs/reviews/q12-governance-release-state-reconciliation-20260921.md`, commit `858e77de18faba28c24c287c3e73e400f8d21ada`. D019, policy/value choices and release authority remain separate; no release-stage transition occurred.
- **Repository-wide item-4 inventory/refill is persisted** at `docs/reviews/item4-repository-wide-refill-20260921.md`, commit `bc7f9479fbe8c277f0b5319065b84dd55cfe9a85`. It deliberately avoids duplicate work on unchanged/covered observability and package owners while refilling materially changed or incompletely re-challenged current owners.
- GitHub combined-status/workflow queries expose no hosted run/status for exact `8cbd9af`; absence is not failure evidence.
- Release items 3 and 4 remain incomplete. `RELEASE_CANDIDATE=false`, `PRODUCTION_READY=false`, `FREEZE=false`, `RELEASED=false` remain unchanged.
- `READY_LIVE: none`. Do not repeat HY2, warm failover, periodic/soak, package lifecycle, migration-back, endpoint/key migration, IPv6, PLPMTUD or Experimental Track without a new code/instrumentation/hypothesis/path condition creating a concrete unresolved self-owned real-network question.

The external coding agent must synchronize to current `main` and continuously execute dependency-ready work: independent bounded review -> concrete repair only if current committed semantics already decide the answer -> focused deterministic tests -> commit -> push -> clean exact-tree local gate/provenance where code/tests changed -> next slice. Reviewer cadence is only a check frequency. Do not wait for the next reviewer after closing a slice.

## READY_LOCAL 1 — FRONT: I4-FS1 FairScheduler queue/fairness current-owner review

Read exact-current `neko-carrier` scheduler owner/tests plus applicable spec/review. Challenge enqueue/dequeue/remove/empty transitions, fairness rotation, closed/terminal owner exclusion and deterministic queue accounting. The dedicated scheduler review is from the 2026-09-13 sweep, while the owner file later changed substantially for R8/R9. No-finding bounded review is valid. A concrete defect converts immediately to the smallest repair/regression; do not redesign scheduler policy.

## READY_LOCAL 2 — I4-FS2 multi-stream + session/stream flow-control accounting

Challenge stream/session limit interaction, queued/inflight/released byte accounting, reject-before-mutate behavior, duplicate/terminal paths and cross-stream isolation on the exact-current tree. Keep Session delivery evidence separate from Carrier packet feedback.

## READY_LOCAL 3 — I4-AD1 MemoryCarrier close/error/resource semantics

Challenge send/recv/close ordering, idempotence, peer/owner release, post-close failures and false-success negatives. No lifecycle framework churn; use current committed adapter semantics.

## READY_LOCAL 4 — I4-AD2 UDP/TCP carrier close/error/resource semantics

Challenge listener/socket ownership, connect/accept/send/recv errors, shutdown/rebind/resource release and bounded cleanup. Keep adapter resource truth distinct from benchmark process-sampler evidence. No live run unless this review creates a specific unresolved real-network question.

## READY_LOCAL 5 — I4-BLD dependency/build surface

The earlier dependency review predates later lock/dependency integration (`neko-reliable` into Carrier/CLI). Re-challenge workspace/crate manifests, `Cargo.lock`, feature/default-feature reachability, build/native hooks and inherited unsafe assumptions on the current tree. Do not add dependencies or redesign features merely to exercise the lane.

## READY_LOCAL 6 — I4-PORT cross-platform CLI/process-test semantics

The CLI and process evidence stack changed materially after the earlier portability review. Challenge platform guards, Linux `/proc`, signals/process groups/setsid, temp/runtime cleanup assumptions, deterministic unsupported-platform behavior and false passes. Do not claim support for platforms the repository does not support.

## READY_LOCAL 7 — I4-CLI-M machine-readable exit / JSON contract

Challenge success/failure/timeout/cleanup-unknown branches, typed/schema result alignment, stdout purity and process-exit consistency across current CLI/process owners. Typed failure or incomplete cleanup must never coexist with wrapper success unless the documented command contract explicitly separates those concepts.

## READY_LOCAL 8 — I4-CLI-H human-output / stderr contract

Challenge human formatting/error routing independently from the machine-readable lane: no positive success after terminal failure, no structured-output contamination, no misleading success wording on typed failure, and stable exit semantics. Avoid cosmetic churn when no defect exists.

## READY_LOCAL 9 — I4-BND algorithmic resource boundedness

Re-check current Recovery/Carrier/Session/CLI loops, maps, queues, retained/live state and peer-controlled iteration under existing limits. Do not run capacity-pressure/adversarial-load benchmarks and do not invent TTL/LRU/history/capacity/security values. `SessionRuntime.events` retained-state capacity remains a maintainer/security policy gate; classify it, do not choose a cap.

## READY_LOCAL 10 — item-4 factual reconciliation after refill group

After each 3–4 coherent refill lanes or any important repair group, reconcile item-4 coverage and current release/evidence facts. Preserve valid no-finding notes and remove only lanes actually closed/superseded. Do not mark item 4 complete unless repository-wide review truth supports it.

## READY_LOCAL 11 — conditional live question only

Only if a new code/instrumentation/hypothesis/path condition creates a specific unresolved self-owned real-network question, classify it against `docs/standing-vps-lab-authorization.md` and `docs/vps-rental-window-priority.md`. Ordinary bounded self-owned TCP/UDP work inside standing authorization needs no per-run approval. Do not manufacture live work merely because the VPS is rented.

## Surfaces deliberately not duplicated in this refill

- `neko-reliable`, `CarrierState`, Concurrent Carrier Manager/migration-back and `SessionRuntime` have fresh R9 independent challenges on current/relevant owners.
- `neko-observe` owner history shows no post-`8e11de0` source change; its dedicated deep-sweep review plus Candidate-B closure remain applicable.
- `scripts/release` shows no post-2026-09-13 owner change; existing package/reproducibility review remains applicable. The later process-resource sampler owners were separately repaired/reviewed through H-R9-082/083/084 and R9-11D.
- Release-packet factual/evidence boundaries were freshly reconciled by Q10/Q11/Q12.

## Stop / escalation conditions

Continue review/repair -> tests -> commit -> push -> next dependency-ready slice without waiting for reviewer cadence. Stop/escalate only for an unresolved BLOCKER/HIGH that cannot be auto-decided, core Session/Carrier/ACK/crypto/wire architecture change, D019/policy-value choice, destructive/canonical migration, out-of-standing-authorization operation, third-party/production/new-credential permission, maintainer-selected adversarial-load/benchmark conditions, or entry into a genuinely new release stage.
