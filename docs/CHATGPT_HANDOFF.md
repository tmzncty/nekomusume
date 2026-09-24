# ChatGPT reviewer handoff — rootless middlebox/reachability queue active

## Current repository truth

- Synchronize to current `main` before work. Exact code/tests/reachable commits/current specs outrank this handoff, chat memory and stale checkbox state.
- **Latest developer-owned source/test anchor:** reachable `62d59141dfe30ea1412e9e9d330c5c21c697f7f9` (`test(carrier): R-MBOX-ROOTLESS-LOSS ...`). It is one commit ahead of the previous reviewer handoff and materially moves `crates/neko-carrier/src/lib.rs`, `crates/neko-cli/src/main.rs`, and `crates/neko-cli/tests/probe.rs`. The older handoff sentence claiming the current product/test anchor was `4511e4f...` and all later movement was docs-only is superseded.
- Reviewer bounded source/control-flow review is reachable at `docs/reviews/independent-r-mbox-rootless-loss-62d5914-20260924.md` (review commit `f7ba0d646737a07e649419d2e4311df603bed0c3`). No concrete correctness/security defect was found in that bounded pass, but **R-MBOX-ROOTLESS-LOSS is not evidence-closed yet** because no reachable developer-local exact-tree gate/provenance for `62d5914...` is present.
- Evidence boundary from that review: historical `FaultPolicy::one_way` suppresses alternating sends and must not be described as proof of a directional network topology. Actual rootless reply-cessation/hard-loss path behavior is represented by the process seams `--cease-udp-replies-after` and the new opt-in `--drop-all-udp`. Do not inflate the alternating-send unit test into a real-network claim.
- GitHub combined status for `62d5914...` exposes no status entries and no PR-triggered workflow run was visible during review. This is not hosted CI evidence.
- **H-I4-120 is CLOSED.** `9e79e6b...` restored the authorized pre-gate production `Recovery::on_ack` baseline; `4511e4f...` made the disputed non-ack-eliciting ACK regression semantics-neutral. Reachable developer-local exact-tree provenance remains in `docs/notes/check-gate-4511e4f-20260924.md`.
- **H-I4-119 remains MAINTAINER / CORE ACK-PTO SEMANTICS.** Do not choose between any-newly-acked reset and ack-eliciting-only reset. Current rollback baseline is the last authorized implementation state, not a final semantic decision.
- Release/security packet is current through the latest prior grouped reconciliation and independently checked; the new R-MBOX source/test movement is research/test-support work and does not change release/security/governance flags.
- Residual item-4 policy/external map remains current: H-I4-119, D019, retained-history capacity, RSEC-001/adversarial-load suitability, restart/rollback replay safety, signing/key-custody/SBOM/publication, independent external review, and RC/freeze/release/production authority remain policy/external/dependency classes.
- Thirteen-surface owner history remains the baseline inventory, but the new CLI/carrier test-support delta means exact-current owner reuse must be checked rather than mechanically inherited. `docs/reviews/independent-r-mbox-rootless-loss-62d5914-20260924.md` is the current bounded delta review for this slice.
- PR #4 (`research: Define reproducible middlebox and reachability simulation plan`) remains merged as reachable `f7bf1f0d2b55ab69834c119cfc6310c9a2dce0be`; it creates real dependency-ready local research/test-support work and does not freeze architecture or create release/security approval.
- Release items **3 and 4 remain incomplete**. `RELEASE_CANDIDATE=false`, `PRODUCTION_READY=false`, `FREEZE=false`, `RELEASED=false` remain unchanged.
- **`READY_LIVE: none`.** Continue deterministic local/rootless work. Do not repeat HY2, warm failover, periodic/soak, package lifecycle, migration-back, endpoint/key migration, IPv6, PLPMTUD or Experimental Track merely because a VPS is rented.

## FRONT — close R-MBOX-ROOTLESS-LOSS evidence, then continue immediately

The implementation/test slice exists at `62d5914...`; reviewer source review is no-finding but the mandatory developer-local exact-tree evidence is missing.

1. Re-read exact-current `docs/research/middlebox-reachability-and-network-behavior.md`, `docs/carrier-architecture.md`, current `crates/neko-carrier/src/lib.rs`, `crates/neko-cli/src/main.rs`, `crates/neko-cli/tests/probe.rs`, standing authorization, and this handoff.
2. If a newer developer source/test commit already exists, review/repair that current tree instead of mechanically gating stale `62d5914...`.
3. Otherwise use reachable pushed source SHA `62d59141dfe30ea1412e9e9d330c5c21c697f7f9` as the final source tree and run in a safe clean checkout/worktree:

```text
PYTHONDONTWRITEBYTECODE=1 bash scripts/check.sh
git diff --check
```

Verify a clean tree and persist a reachable exact-tree provenance note containing exact SHA, exact UTC start/end, both exit codes, OS/arch and stable Rust version. Do not fabricate metadata. This delta does not touch decoder/parser/crypto framing, so do not run fuzz mechanically.
4. If the exact-tree gate exposes a concrete current-semantics defect, make the smallest repair + focused positive/negative regression, push the repaired source SHA, rerun the exact-tree gate on that final pushed SHA, persist provenance, and immediately refill the affected owner review. A failing gate outranks this reviewer's source-only no-finding.
5. If the gate is clean, **do not add code/docs churn merely to make another slice**. R-MBOX-ROOTLESS-LOSS is then closure-complete for its bounded local question. Continue in the same agent run directly to R-MBOX-REORDER-DELAY below; reviewer cadence is not a work-ticket boundary.
6. Preserve the interpretation boundary: `FaultPolicy::one_way` alternating-send suppression is deterministic fault injection, not proof of directional topology. Use the process/local-socket seam for directional reply cessation/hard loss claims.

## Dependency-ordered rolling queue

1. **R-MBOX-ROOTLESS-LOSS closure — FRONT.** Exact-tree local gate/provenance for `62d5914...` (or a repaired successor). No source churn if green.
2. **R-MBOX-REORDER-DELAY.** Extend/reuse the same deterministic local seam for bounded reorder + delay. Challenge transition/order semantics without inventing production timer, performance, readiness, hysteresis or congestion policy. Prefer existing `FaultInjectCarrier`/loopback/failover owners; do not create a parallel network framework.
3. **R-MBOX-MTU.** Model a local oversized-datagram/MTU-drop boundary without changing system interface MTU, PLPMTUD policy or wire framing. Keep packet-size/path evidence separate from Session delivery.
4. **R-MBOX-CLEAN-RECOVERY.** Controlled loss followed by a clean path; prove recovery/validation according to existing Carrier semantics and no duplicate Session delivery. Do not invent a new migration/readiness threshold.
5. **R-MBOX-REPEATED-FAILOVER.** Bounded repeated failover/recovery transitions; assert exactly one active carrier, valid generation ownership, no duplicate delivery and diagnostic transition order. Reuse current CarrierState/manager semantics and existing process ownership helpers.
6. **R-MBOX-RESOURCE.** Bounded repeated-transition resource fixture reusing existing observability/resource helpers. Challenge ownership convergence and monotonic leaks in live endpoints/session entries/in-flight recovery work. This is **not** a capacity-pressure benchmark; do not invent RSS/FD/security/capacity numbers.
7. **R-MBOX-TRANSITION-MATRIX.** Reconcile injected impairment -> exact Carrier/Session event/state expectations and record tested vs unresolved boundaries. Evidence/spec support only; no new architecture ADR merely for bookkeeping.
8. **Grouped release/item-4 factual reconciliation** after 3–4 coherent R-MBOX slices or immediately after any important repair. Preserve evidence-class boundaries and all policy/external gates.
9. **Moved-owner refill.** Because `62d5914...` moved `neko-cli/src/main.rs`, `neko-carrier/src/lib.rs` test-support ownership and process tests, check whether any prior CLI/process/carrier bounded review claim materially depends on the changed lines. If yes, add a focused delta challenge; if not, record exact scoped reuse. Any later semantic movement in Recovery, CarrierState/manager, Session/flow-control, adapters, observability, package/build, CLI/process/resource or security/pre-auth owners invalidates corresponding reuse and creates fresh bounded review work.
10. **Repository-wide 13-surface refill.** Keep at least the unreviewed/moved implemented owners live as review-support while item 4 remains incomplete. A no-finding narrow seam never means queue exhausted.
11. **CONDITIONAL LIVE.** Only if new code/instrumentation/hypothesis/path condition creates a concrete unresolved real-network question within standing VPS authorization. Current classification remains `READY_LIVE: none`.
12. **FINAL repository-wide reconciliation before any `queue exhausted` statement.** Queue exhaustion is currently false. It becomes legal only after the real R-MBOX queue is completed/blocked and the broad 13-surface inventory again shows no unreviewed/moved implemented owner, no defect, no READY review-support, no READY live question, and every residual is genuinely policy/external/environment/release-authority gated.

The queue contains real current work; do not inflate it with checker/schema/framework/docs filler. If the agent completes coherent slices rapidly with low defect rate, refill from actual moved owners, the transition matrix and remaining item-4 surfaces rather than waiting for the next reviewer.

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
