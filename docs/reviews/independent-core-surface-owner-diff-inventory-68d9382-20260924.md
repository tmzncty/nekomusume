# Independent exact-current 13-surface owner-diff inventory — `68d9382`

**Source anchor:** exact reachable `68d938279478b91661f786e830e99af83db457f1`. Reviewer-only notes added after that SHA do not alter product/test owners.

This is a repository-wide release-item-4 refill/inventory pass. It does not claim reviewer-local Rust/full-gate execution, hosted CI, WAN evidence, performance results, security approval, RC/freeze/release authority, or completion of item 3/4.

## Classification rule

- **CURRENT / REUSE** — a dedicated bounded review covers the current semantic owner and later source history does not move that owner.
- **CURRENT / NEW NO-FINDING** — this run re-challenged the current owner and found no concrete defect in the stated scope.
- **MAINTAINER/POLICY** — a remaining question is explicitly outside reviewer authority.
- **READY_LOCAL** — real factual/review-support work remains and is dependency-ready.
- **READY_LIVE** — only a newly-created unresolved real-network question; none exists here.

## Thirteen required surfaces

| # | Surface | Exact-current status | Classification / consequence |
|---|---|---|---|
| 1 | `neko-reliable` UDP recovery: ACK validity/range, loss/retransmit, RTT/PTO, persistent congestion, Reno, deterministic fault simulation | H-I4-116/117/118 repairs are reachable; R-REC-1/2/3, R-REC-4a/4b, and the new `independent-r-rec-residual-68d9382-20260924.md` together cover the current owner. The only unresolved seam is H-I4-119, whose ACK/PTO reset choice is not uniquely determined by current repo semantics. | **CURRENT / NEW NO-FINDING** outside H-I4-119; **MAINTAINER / CORE ACK-PTO SEMANTICS** for H-I4-119. Do not patch that seam without a maintainer/spec decision. |
| 2 | `neko-carrier::CarrierState`: generation, validation, hysteresis, single-active, drain/fail/activate | `independent-r-cs-1-carrierstate-generation-validation-hysteresis-270265f-20260924.md` and `independent-r-cs-2-carrierstate-active-drain-fail-activate-259fb58-20260924.md` cover the latest CarrierState owner. No later CarrierState semantic movement is present. | **CURRENT / REUSE**. |
| 3 | Concurrent Carrier Manager / health / migration-back | `independent-r-cm-diff-manager-health-migration-reuse-44a77cb-20260924.md` rechecked the post-CarrierState owner history; no later manager semantic movement is present. | **CURRENT / REUSE**. |
| 4 | FairScheduler / multi-stream / session+stream flow-control accounting | The dedicated scheduler/Session repair/review chain plus `independent-r-fs-session-diff-e228ec6-20260924.md` covers the current owners. | **CURRENT / REUSE**. |
| 5 | Carrier adapters: Memory/UDP/TCP close/error/resource semantics | Dedicated adapter reviews plus `independent-r-obs-adapter-diff-12374ee-20260924.md` establish no later adapter owner movement. | **CURRENT / REUSE**. |
| 6 | `SessionRuntime` lifecycle/resource/window/DeliveryAck accounting | Current correctness owner remains covered by the SessionRuntime + FS2/H-I4-086/087 review/repair chain and the later owner-diff reuse note. Retained `SessionRuntime.events` capacity remains a value/policy decision. | **CURRENT / REUSE** for correctness; **MAINTAINER/POLICY** for retained-history capacity value. |
| 7 | `neko-observe` projection/event/counter/high-water correctness | Candidate B/mixed-drop repair history and dedicated observability reviews remain current; `independent-r-obs-adapter-diff-12374ee-20260924.md` establishes no later owner movement. | **CURRENT / REUSE**. |
| 8 | package/reproducibility/operator scripts | Dedicated package/reproducibility review plus `independent-r-pkg-bld-diff-5135d8e-20260924.md` covers current owner history. Signing/key-custody/SBOM/publication remain governance, not implementation assumptions. | **CURRENT / REUSE**; governance gates remain separate. |
| 9 | Cargo manifests/lock/features/build/native hooks/unsafe inheritance | I4-BLD plus the latest package/build owner-diff note establish no current manifest/build/native-hook/unsafe-owner movement. | **CURRENT / REUSE**. |
| 10 | cross-platform CLI/process-test semantics | The H-I4-097..115 process-owner repair chain and `independent-cli-cross-platform-process-416fd5e-20260924.md`, followed by `independent-r-cli-bnd-diff-33b6a3c-20260924.md`, cover current owners. These are not non-Linux execution claims. | **CURRENT / REUSE** with explicit evidence-platform boundary. |
| 11 | CLI exit-code / JSON / human-output contract | Dedicated CLI contract/diagnostic reviews remain applicable; the latest CLI/boundedness owner-diff note records no subsequent product-CLI semantic movement. | **CURRENT / REUSE**. |
| 12 | algorithmic resource boundedness | The current boundedness reconciliation and later CLI/boundedness owner-diff reuse cover implemented numeric-free invariants. Retained-history/capacity choices remain policy-gated; no capacity-pressure benchmark is introduced. | **CURRENT / REUSE** + **MAINTAINER/POLICY** for value selection. |
| 13 | release packet factual consistency and evidence boundary | `release-item4-reconciliation-owner-diff-reuse-a34f38f-20260924.md` predates the final H-I4-119 reclassification and the two current-owner review notes added in this run. The packet remains an evidence index and does not need a release-state change, but it needs a small current factual-index reconciliation. | **READY_LOCAL** documentation/evidence-index reconciliation only. |

## Repository-wide result

The broad implemented-core inventory no longer exposes a second unreviewed product-code surface at this source anchor. That does **not** mean release item 4 or the repository queue is exhausted:

1. release packet/evidence-index factual reconciliation remains **READY_LOCAL**;
2. a security-boundary owner diff remains useful to prove that later Recovery/test-harness repairs did not silently move crypto/trust/pre-auth/security owners;
3. H-I4-119 is a real maintainer/core-semantics gate but must not block unrelated review-support work;
4. D019, retained-history/capacity values, signing/key-custody/SBOM/publication, external security review, item-3 evidence, and release authority remain separate gates;
5. after the factual/security reconciliations, rerun owner history once more before any queue-exhaustion statement. If a coding agent moves any semantic owner in the meantime, refill from that owner immediately.

The previous inventory's requirement to create three arbitrary moved/unreviewed core lanes is therefore superseded by repository truth: there are not three such owners at this anchor. Do not manufacture filler reviews merely to hit a numeric queue target.

## Evidence boundary

This inventory is source/history/review-ledger classification only. No reviewer-local Rust/full-gate, hosted CI, cross-platform run, fuzz, WAN, adversarial-load benchmark, or performance run is claimed. Developer-local exact-tree provenance remains distinct from reviewer review notes and hosted checks. No decoder/parser/crypto-framing code changed in these reviewer-only notes.

`READY_LIVE: none`. Release items 3 and 4 remain incomplete. `RELEASE_CANDIDATE=false`, `PRODUCTION_READY=false`, `FREEZE=false`, `RELEASED=false` remain unchanged.
