# Repository-wide release-item-4 inventory / refill — 2026-09-21

**Exact executable source/test anchor:** `8cbd9af39a600f4ddd5fbc50208791e8584c6d74`.

This inventory follows the final independent R9 review plus Q10/Q11/Q12 reconciliation. It is deliberately repository-wide: a clean R9 sweep is not queue exhaustion while release item 4 remains open. Existing dedicated bounded reviews remain valid where their owners did not materially change; lanes are refilled only where current/relevant owners changed after the prior independent review or where the current exact tree has not had a dedicated bounded challenge of that surface.

## 13-surface inventory

1. **`neko-reliable` recovery — covered current/relevant tree.** R9 independently re-challenged future/unsent ACK fail-closed behavior, bounded/canonical ranges, RTT/loss/PTO/frame ownership, replay suppression, quiesce/lifetime boundaries and terminal ownership. No immediate duplicate lane.
2. **`CarrierState` — covered current/relevant tree.** Final R9 review plus R9-11 carrier work re-challenged generation, validation, hysteresis, single-active, drain/fail/activate and packet-feedback separation.
3. **Concurrent Carrier Manager / health / migration-back — covered current/relevant tree.** R9-11C and final R9 review cover generation/readiness, uncertain retention, replay, validation/health/hold and migration-back.
4. **FairScheduler / multi-stream / flow-control — REFILL.** The dedicated scheduler review is from the 2026-09-13 deep sweep, while `crates/neko-carrier/src/lib.rs` received substantial later R8/R9 work. R9 reviews concentrated on recovery/readiness/replay/migration and did not independently re-challenge the whole current scheduler + multi-stream + session/stream accounting surface. Refill as two bounded lanes rather than assuming unchanged semantics from file-level review.
5. **Carrier adapters — REFILL.** The dedicated Memory/UDP/TCP adapter review is from the 2026-09-13 sweep; the carrier owner subsequently changed heavily. Current R9 review did not substitute for a close/error/resource-semantics challenge of every adapter. Refill Memory separately from socket adapters to keep failure/resource ownership tests bounded.
6. **`SessionRuntime` lifecycle/resource/window/DeliveryAck — covered current/relevant tree.** H-R9-080 and final R9 independently exercised populated terminal ownership, send/receive/window/dedup/deadline cleanup, lifetime survivors and post-terminal evidence rejection.
7. **`neko-observe` projection/counters/high-water — covered unchanged owner.** GitHub path history shows the latest `crates/neko-observe/src/lib.rs` change remains exact `8e11de0` on 2026-09-13, the same deep-sweep repair/review line. Candidate-B mixed drop projection was separately closed. Do not manufacture another observability lane absent a new owner change or concrete challenge.
8. **Package/reproducibility/operator scripts — covered unchanged release-script owners; process sampler separately refreshed.** GitHub path history shows no `scripts/release` commits after the 2026-09-13 independent package/release review. The later process-resource sampler/result-truth owners were separately challenged and repaired through H-R9-082/083/084 and R9-11D. No generic package rerun lane.
9. **Dependency/build surface — REFILL.** The earlier dependency review predates later lock/dependency changes, including `neko-reliable` integration into carrier/CLI. Current lock/manifests/features/native/unsafe inheritance deserve one exact-current bounded challenge. Do not add dependencies or redesign features merely to exercise this lane.
10. **Cross-platform CLI/process-test semantics — REFILL.** `crates/neko-cli/src/main.rs` changed repeatedly through R9 after the earlier portability review, and the current process-evidence stack includes Linux `/proc`, process-group/setsid and signal semantics. Re-challenge platform gating, deterministic skips/fail-closed behavior, process cleanup and portable command assumptions; do not claim unsupported platforms.
11. **CLI exit-code / JSON / human-output contract — REFILL.** The current CLI gained many R9 ACK/replay/result-truth branches after the earlier command/portability reviews. Re-challenge machine-readable exit/JSON truth and human stdout/stderr separately so typed failure cannot coexist with false success and human formatting cannot corrupt JSON contracts.
12. **Algorithmic resource boundedness — REFILL.** Current Session/Carrier/CLI ownership changed after the old boundedness sweep. Re-check iteration/allocation/state-growth properties using existing semantics only. Do not run capacity-pressure/adversarial-load benchmarks and do not invent TTL/LRU/history/capacity/security values. `SessionRuntime.events` policy-bound capacity remains a maintainer/security policy gate, not an agent-selected repair.
13. **Release-packet factual consistency/evidence boundary — covered current tree.** Q10/Q11/Q12 are fresh bounded reconciliations. Item 3 and item 4 remain open; release flags remain false; `READY_LIVE: none` remains current.

## Dependency-ready refill queue

The following are legitimate item-4 support lanes on current/relevant owners. Each begins as independent bounded review; a concrete defect with already-decided committed semantics converts immediately to the smallest repair + regression + exact-tree gate/provenance.

1. **I4-FS1 FairScheduler queue/fairness/current-owner review** — enqueue/dequeue/remove/empty transitions, fairness rotation, terminal/closed-owner exclusion, deterministic queue accounting.
2. **I4-FS2 multi-stream + session/stream flow-control accounting** — stream/session limit interaction, bytes queued/inflight/released, reject-before-mutate, duplicate/terminal paths and cross-stream isolation.
3. **I4-AD1 MemoryCarrier close/error/resource semantics** — send/recv/close ordering, idempotence, peer/owner release, post-close behavior and false-success negatives.
4. **I4-AD2 UDP/TCP carrier close/error/resource semantics** — listener/socket ownership, connect/accept/send/recv errors, shutdown/rebind/resource release, bounded cleanup; keep this distinct from benchmark process-sampler evidence.
5. **I4-BLD dependency/build surface** — workspace/crate manifests, lock consistency, feature/default-feature reachability, build/native hooks and inherited unsafe assumptions after R8/R9 dependency integration.
6. **I4-PORT cross-platform CLI/process-test semantics** — platform guards, `/proc`/signals/process-groups/tempfiles, deterministic unsupported behavior, no platform-specific false pass.
7. **I4-CLI-M machine-readable exit/JSON contract** — success/failure/timeout/cleanup-unknown branches, schema/typed-result alignment, stdout purity and process exit consistency.
8. **I4-CLI-H human-output/stderr contract** — human formatting, error routing, no positive success after terminal failure, and no accidental structured-output contamination.
9. **I4-BND algorithmic resource boundedness** — current Recovery/Carrier/Session/CLI loops/maps/queues/state-retention mechanics under existing limits; no pressure benchmark and no new policy numbers.
10. **Conditional live only** — remains closed unless one of the above produces a new code/instrumentation/hypothesis/path condition with a specific unresolved self-owned real-network question.

This is a real refill, not filler: surfaces 4/5/9/10/11/12 have either materially changed owners after the older dedicated review or were not fully re-challenged by the narrow R9 closure. Surfaces 7/8 are explicitly *not* refilled because owner history shows the existing review remains applicable, preventing duplicate work.

`READY_LIVE: none`; release items 3/4 remain incomplete; D019/policy/release-authority gates remain untouched.
