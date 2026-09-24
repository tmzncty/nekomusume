# Independent exact-current 13-surface owner-diff inventory — `4511e4f`

**Product/test source anchor:** exact reachable `4511e4f147eb9511907bb4ec52cad687f113bb7a`.

**Reviewer notes after that source anchor do not move product/test owners.**

This is a repository-wide release-item-4 refill pass after H-I4-120 closure. It does not claim reviewer-local Rust/full-gate execution, hosted CI, WAN evidence, performance results, security approval, RC/freeze/release authority, or completion of release items 3/4.

## Delta from the prior `68d9382` inventory

Repository history from `68d938279478b91661f786e830e99af83db457f1` through `4511e4f...` moves only the `neko-reliable` source/test surface plus reviewer/evidence documentation. The unauthorized H-I4-119 semantic implementation was subsequently rolled back; surviving product-code semantics return to the pre-gate baseline while focused tests remain. No CarrierState/manager, scheduler, SessionRuntime, adapter, observability, package/build, CLI, crypto, pre-auth, or operator-script owner moved in that interval.

H-I4-120 is now independently closed by `reviewer-h-i4-120-closure-4511e4f-20260924.md`; the moved Recovery delta is independently re-challenged with bounded no-finding in `independent-r-post-h120-recovery-4511e4f-20260924.md`. H-I4-119 remains policy-gated.

## Thirteen required surfaces

| # | Surface | Exact-current classification |
|---|---|---|
| 1 | `neko-reliable` UDP recovery: ACK validity/range, loss/retransmit, RTT/PTO, persistent congestion, Reno, deterministic fault simulation | **CURRENT / NEW NO-FINDING** outside H-I4-119. H-I4-120 rollback closed; post-rollback owner delta re-challenged. **MAINTAINER / CORE ACK-PTO SEMANTICS** remains for H-I4-119 only. |
| 2 | `neko-carrier::CarrierState`: generation, validation, hysteresis, single-active, drain/fail/activate | **CURRENT / REUSE**. No owner movement after the dedicated R-CS-1/R-CS-2 reviews. |
| 3 | Concurrent Carrier Manager / health / migration-back | **CURRENT / REUSE**. No manager owner movement. |
| 4 | FairScheduler / multi-stream / Session + stream flow-control accounting | **CURRENT / REUSE**. No scheduler/flow-accounting owner movement. |
| 5 | Memory/UDP/TCP carrier adapter close/error/resource semantics | **CURRENT / REUSE**. No adapter owner movement. |
| 6 | `SessionRuntime` lifecycle/resource/window/DeliveryAck accounting | **CURRENT / REUSE** for correctness. Retained `SessionRuntime.events` capacity remains **MAINTAINER/POLICY** rather than an invented numeric repair. |
| 7 | `neko-observe` projection/event/counter/high-water correctness | **CURRENT / REUSE**. Candidate-B/mixed-drop repair and bounded observability reviews remain current. |
| 8 | package/reproducibility/operator scripts | **CURRENT / REUSE**. Signing/key-custody/SBOM/publication remain separate governance gates. |
| 9 | dependency/build manifests/lock/features/build/native hooks/unsafe inheritance | **CURRENT / REUSE**. No build/dependency owner movement. |
| 10 | cross-platform CLI/process-test semantics | **CURRENT / REUSE** with evidence-platform boundary. No CLI/process owner movement; prior review does not claim non-Linux execution. |
| 11 | CLI exit-code / JSON / human-output contract | **CURRENT / REUSE**. No product CLI semantic movement. |
| 12 | algorithmic resource boundedness | **CURRENT / REUSE** for implemented numeric-free invariants; retained-history/capacity values remain **MAINTAINER/POLICY**. |
| 13 | release packet factual consistency / evidence boundary | **READY_LOCAL**. The packet still predates the completed H-I4-120 rollback/provenance closure and post-rollback Recovery review, and needs a bounded factual-index reconciliation without changing release flags. |

## Security/pre-auth owner reuse

`independent-security-boundary-diff-5577a2f-20260924.md` and `independent-r-preauth-diff-68d9382-20260924.md` remain reusable: after their product/test source anchor, the only product/test owner movement through `4511e4f...` is in `neko-reliable`. No crypto, trust/authz, pre-auth admission, Session security, or wire/parser owner moved. D019, RSEC-001/adversarial-load suitability, persistent restart/rollback replay safety, signing/key-custody/SBOM/publication, external security review and release authority remain gates rather than READY_LOCAL code changes.

## Repository-wide consequence

There is currently no honest basis to manufacture multiple new code-review lanes simply to hit a queue-depth target. Real dependency-ready work remains:

1. **R-RPKT-CURRENT** — reconcile the release/security packet factual index with H-I4-116/117/118, H-I4-119 classification, H-I4-120 rollback/closure/provenance, post-H120 Recovery review, CarrierState/manager and current security/pre-auth owner-diff reuse. Preserve the Session-delivery evidence boundary and all release flags.
2. **R-RPKT-CHECK** — independently verify the packet edit against reachable anchors and evidence-class boundaries; no self-attestation or claim that one SHA ran all historical tests.
3. **R-ITEM4-EXTERNAL/POLICY MAP** — after packet facts are current, refresh the residual map: H-I4-119, D019, retained-history capacity, RSEC-001/adversarial-load/security approval, restart/rollback replay safety, signing/key-custody/SBOM/publication, independent external review, item-3 evidence, environment limits and release authority.
4. **R-FINAL-OWNER-HISTORY** — rerun exact-current 13-surface owner history after the packet/reconciliation commits. Any actual product/test semantic movement refills READY_LOCAL immediately.
5. **CONDITIONAL LIVE** — only if new code/instrumentation/hypothesis/path condition creates a concrete unresolved real-network question within standing authorization. Current classification remains `READY_LIVE: none`.
6. **QUEUE-EXHAUSTION CHECK** — only after broad owner inventory, factual packet, external/policy map and live classification are all current. A remaining policy/external gate is not permission to claim release readiness.

## Evidence boundary

This inventory is source/history/review-ledger classification only. No reviewer-local Rust/full-gate, hosted CI, cross-platform run, fuzz, WAN, adversarial-load benchmark, performance run, security approval or release action is claimed. Developer-local exact-tree provenance remains a distinct evidence class.

Release items 3 and 4 remain incomplete. `RELEASE_CANDIDATE=false`, `PRODUCTION_READY=false`, `FREEZE=false`, `RELEASED=false`; `READY_LIVE: none` remains unchanged.
