# ChatGPT reviewer handoff — rootless middlebox/reachability research queue active

## Current repository truth

- Synchronize to current `main` before work. Exact code/tests/reachable commits/current specs outrank this handoff, chat memory and stale checkbox state.
- **Exact current product/test anchor:** reachable `4511e4f147eb9511907bb4ec52cad687f113bb7a`. Subsequent movement through the latest reviewer notes and merged PR #4 is documentation/research/review navigation only; no Rust source/test/Cargo/build/package/wire owner moved.
- **H-I4-120 is CLOSED.** `9e79e6b...` restored the authorized pre-gate production `Recovery::on_ack` baseline; `4511e4f...` made the disputed non-ack-eliciting ACK regression semantics-neutral. Reachable developer-local exact-tree provenance is `docs/notes/check-gate-4511e4f-20260924.md` (`scripts/check.sh` exit 0, `git diff --check` exit 0, clean tree, exact UTC start/end, Linux x86_64, stable rustc 1.98.0). GitHub combined status for that SHA exposes `statuses=[]`; do not call this hosted CI.
- **H-I4-119 remains MAINTAINER / CORE ACK-PTO SEMANTICS.** Do not choose between any-newly-acked reset and ack-eliciting-only reset. Current rollback baseline is the last authorized implementation state, not a final semantic decision.
- Release/security packet is current and independently checked: `docs/reviews/independent-r-rpkt-check-cc89045-20260924.md` is bounded no-finding.
- Residual item-4 policy/external map is current: `docs/reviews/independent-r-item4-external-policy-map-d9e1296-20260924.md`. It keeps H-I4-119, D019, retained-history capacity, RSEC-001/adversarial-load suitability, restart/rollback replay safety, signing/key-custody/SBOM/publication, independent external review, and RC/freeze/release/production authority in their proper policy/external/dependency classes.
- Thirteen-surface owner history is current in `docs/reviews/independent-r-final-owner-history-f7bf1f0-20260924.md`; all 13 release-item-4 surfaces retain dedicated reachable bounded review/evidence outside explicit policy seams.
- PR #4 (`research: Define reproducible middlebox and reachability simulation plan`) was independently reviewed and squash-merged as reachable `f7bf1f0d2b55ab69834c119cfc6310c9a2dce0be`. Fresh open-PR query after merge returned none. It adds research/spec material only and does not freeze architecture or create release/security approval.
- **PR #4 also creates real dependency-ready local research/test-support work.** `docs/reviews/independent-r-mbox-gap-inventory-bb8c620-20260924.md` records the exact-current gap inventory. Current repo search finds no reusable netem/rootless impairment harness, repeated-failover integration fixture, or repeated-transition resource fixture implementing the new matrix; existing `crates/neko-cli/tests/probe.rs` already provides loopback peers, warm/cold failover diagnostics, bounded child ownership and Linux resource snapshots that should be reused instead of creating a parallel framework.
- Release items **3 and 4 remain incomplete**. `RELEASE_CANDIDATE=false`, `PRODUCTION_READY=false`, `FREEZE=false`, `RELEASED=false` remain unchanged.
- **`READY_LIVE: none`.** The new research queue is deterministic local/rootless work. Do not repeat HY2, warm failover, periodic/soak, package lifecycle, migration-back, endpoint/key migration, IPv6, PLPMTUD or Experimental Track merely because a VPS is rented.

## FRONT — R-MBOX-ROOTLESS-LOSS

Implement/review the first dependency-ready slice from the merged middlebox/reachability plan **without changing production semantics** unless the test independently proves a concrete defect.

1. Re-read exact-current `docs/research/middlebox-reachability-and-network-behavior.md`, `docs/carrier-architecture.md`, current failover owners in `crates/neko-cli/tests/probe.rs`, applicable CarrierState/manager reviews, and standing authorization.
2. Reuse the existing loopback/failover test seams. Add the smallest rootless local user-space impairment seam necessary to deterministically model:
   - UDP reply cessation / one-way silence; and
   - hard UDP loss.
   Do **not** create a parallel generic network framework if existing fake-peer/local-socket seams can express the invariant.
3. Challenge current committed behavior: Carrier owns path health/failure/promotion; Session retains protocol/delivery semantics. Assert bounded transition/timeline evidence and no duplicate delivery. Do not select new readiness/hysteresis/PTO/security/capacity values.
4. No host route/firewall/DNS/proxy/tunnel/qdisc mutation. Do not make privileged `tc`/netem/netns execution a prerequisite for this slice. Rootless/local-socket execution is the required first path.
5. If the fixture exposes a concrete current-semantics defect, stop expansion, make the smallest repair + positive/negative regression, then refill the affected core-surface review immediately. If it exposes a policy/core semantic choice, classify it rather than deciding it.
6. For any source/test change, commit/push and run the final pushed developer source SHA through `PYTHONDONTWRITEBYTECODE=1 bash scripts/check.sh`, `git diff --check`, verify clean tree, and persist reachable exact-tree provenance with required metadata. No decoder/parser/crypto-framing change is implied; do not run fuzz mechanically.
7. Continue immediately to the remaining queue below; reviewer cadence is not a work-ticket boundary.

## Dependency-ordered rolling queue

1. **R-MBOX-ROOTLESS-LOSS — FRONT.** Rootless one-way UDP silence + hard-loss impairment seam using existing failover owners.
2. **R-MBOX-REORDER-DELAY.** Extend/reuse the same seam for deterministic bounded reorder + delay; challenge transition/order semantics without inventing timer or performance policy.
3. **R-MBOX-MTU.** Model a local oversized-datagram/MTU-drop boundary without changing system interface MTU, PLPMTUD policy or wire framing.
4. **R-MBOX-CLEAN-RECOVERY.** Controlled loss followed by clean path; prove recovery/validation according to existing semantics and no duplicate Session delivery.
5. **R-MBOX-REPEATED-FAILOVER.** Bounded repeated failover/recovery transitions; assert exactly one active carrier, valid generation ownership, no duplicate delivery and diagnostic transition order. No new hysteresis/readiness thresholds.
6. **R-MBOX-RESOURCE.** Bounded repeated-transition resource fixture reusing existing observability/resource helpers where possible; challenge ownership convergence and monotonic leaks in live endpoints/session entries/in-flight recovery work. This is **not** a capacity-pressure benchmark; do not invent RSS/FD/security limits.
7. **R-MBOX-TRANSITION-MATRIX.** Reconcile each injected impairment against exact Carrier/Session events/state expectations and record tested vs unresolved boundaries; no new architecture ADR.
8. **Grouped release/item-4 factual reconciliation** after 3–4 coherent R-MBOX slices or any important repair. Preserve evidence-class boundaries and existing policy/external gates.
9. **Conditional core-owner refill.** Any semantic movement in Recovery, CarrierState/manager, Session/flow-control, adapters, observability, package/build, CLI/process/resource or security/pre-auth owners immediately invalidates the corresponding reuse and creates a fresh bounded challenge.
10. **CONDITIONAL LIVE.** Only if new code/instrumentation/hypothesis/path condition creates a concrete unresolved real-network question within standing VPS authorization. Current classification is `READY_LIVE: none`.
11. **FINAL repository-wide reconciliation before any `queue exhausted` statement.** The newly merged R-MBOX queue means queue exhaustion is currently false. It becomes legal only after this real queue is completed/blocked and the broad 13-surface inventory again shows no unreviewed/moved implemented owner, no defect, no READY review-support, no READY live question, and every residual is genuinely policy/external/environment/release-authority gated.

Seven coherent R-MBOX slices are real current work; do not inflate them with checker/schema/framework/docs filler merely to hit an arbitrary number. If the agent completes them rapidly with low defect rate, refill from actual moved owners and the transition matrix rather than waiting for the next reviewer.

## Standing 13-surface inventory requirement

Every meaningful refill must verify whether each surface still has a dedicated reachable independent bounded review or a materially moved owner:

1. `neko-reliable` UDP recovery — ACK range/future/stale ACK/loss/retransmit/RTT/PTO/persistent congestion/Reno/fault simulation;
2. `neko-carrier` `CarrierState` — generation/validation/hysteresis/single-active/drain/fail/activate;
3. Concurrent Carrier Manager / health / migration-back;
4. FairScheduler / multi-stream / Session + stream flow-control accounting;
5. Memory/UDP/TCP carrier adapter close/error/resource semantics;
6. `SessionRuntime` lifecycle/resource/window/DeliveryAck accounting;
7. `neko-observe` projection/event/counter/high-water correctness;
8. package/reproducibility/operator scripts;
9. dependency/build manifests/lock/features/build/native hooks/unsafe inheritance;
10. cross-platform CLI/process-test semantics;
11. CLI exit-code / JSON / human-output contract;
12. algorithmic resource boundedness without capacity-pressure benchmark or invented policy values;
13. release packet factual consistency/evidence boundary.

A narrow no-finding never means repository-wide queue exhaustion.

## Review -> repair / provenance contract

For every bounded slice: read exact-current owner source/tests + applicable spec/ADR/status claim; state the invariant; try to falsify it with source reasoning and focused deterministic tests. If a concrete defect exists and current committed semantics decide the answer, make the smallest repair + positive/negative regression + commit/push, then run the final pushed developer source SHA through:

```text
PYTHONDONTWRITEBYTECODE=1 bash scripts/check.sh
git diff --check
```

and verify a clean tree. Persist a reachable exact-tree provenance record with exact SHA, exact UTC start/end, exit codes, OS/arch and stable Rust version. Hosted CI is separate cross-evidence and never a wait condition. Wire decoder/parser/crypto-framing changes additionally use the pinned fuzz toolchain and required decode build/run; do not run fuzz mechanically for unrelated code.

If no defect is found, record a scope-precise independent bounded no-finding note naming owners inspected, commands/tests if actually run, exclusions and exact reachable anchor. Do not change code merely to manufacture review churn.

## Evidence discipline / stop conditions

- Developer-reported local CI, persisted developer-local provenance, reviewer source/control-flow review, reviewer-local generic OS checks, hosted CI, live WAN evidence and performance conclusions are distinct classes.
- Accepted exact-tree provenance must anchor a GitHub-resolvable pushed SHA and contain required exact metadata; never publish local-only/unreachable SHA as shared evidence and never fabricate missing timestamps/toolchain details.
- Never decide H-I4-119/core ACK-PTO semantics; D019; TTL/LRU/history/capacity/security values; signing/key-custody/SBOM/publication; previous frozen release policy; core Session/Carrier/ACK/crypto/wire architecture; destructive/canonical migration; RC/freeze/release/production authority.
- Do not automatically mutate host/production route/firewall/DNS/proxy/tunnel/qdisc or rely on privileged network setup. The current R-MBOX queue is deliberately rootless first.
- A correctness/security/evidence BLOCKER/HIGH becomes FRONT when current semantics determine a repair. If it requires a core semantic/policy choice, keep it as maintainer/spec gate and continue unrelated READY work; do not let an unauthorized implementation silently convert a gate into a decision.
- Normal progression does not require administrator notification. Notify only for unresolved BLOCKER/HIGH requiring maintainer choice, core architecture/destructive migration, policy/value decisions, authorization expansion/new credentials/third-party/production actions, adversarial-load benchmark conditions requiring maintainer choice, or a genuine release-phase transition.
