# ChatGPT reviewer handoff — recovery + CarrierState cluster reconciled; continue owner-diff/refill queue

## Current repository truth

- Synchronize to current `main` before work. Exact code/tests/reachable commits/current specs outrank this handoff, chat memory and stale checkbox state.
- **Last developer source/test repair accepted:** `71b62c642a6ea736fd162653ede9b29868e08eee` closes H-I4-118 by restricting loss candidates in `Recovery::on_ack` to `n <= largest`; focused stale-ACK regression is present. Reachable developer-local exact-tree provenance: `docs/notes/h-i4-118-provenance-71b62c6-20260924.md` (`scripts/check.sh` 0, `git diff --check` 0, clean tree, Linux x86_64, rustc 1.98.0). Combined GitHub status for that SHA had no status entries when reviewed; do not call the local record hosted CI.
- H-I4-116 (Reno zero-loss collapse) and H-I4-117 (runtime persistent-congestion wiring) remain closed with their reachable developer-local provenance. H-I4-097..115 remain closed unless their exact semantic owners materially move. Candidate A (future/unsent ACK) and Candidate B (mixed datagram-drop reasons) remain closed unless exact-current owners move.
- **R-REC cluster:** R-REC-1/2/3 bounded reviews remain valid in their stated scopes; R-REC-4a positive aggregate Reno reduction and R-REC-4b deterministic fault-simulation review are closed. H-I4-118 is now part of the repaired recovery boundary; do not reopen by incorrectly requiring a legal duplicate ACK number to remain present in `sent`.
- **R-CS-1 CLOSED no-finding** at review commit `259fb58ec19a47fe4e11900dbe6b338bb09f9467`; note: `docs/reviews/independent-r-cs-1-carrierstate-generation-validation-hysteresis-270265f-20260924.md`.
- **R-CS-2 CLOSED no-finding** at review commit `44a77cb7f4abc124606ba77e56edc74cacccf395`; note: `docs/reviews/independent-r-cs-2-carrierstate-active-drain-fail-activate-259fb58-20260924.md`. Important boundary: the old bounded M0 `CarrierState` model is not proof of the richer D064 runtime-manager migration ordering.
- **R-CM-DIFF CLOSED no-finding/reuse** at review commit `44fd3d69edd9802a73721a10596e0f19ac04f404`; note: `docs/reviews/independent-r-cm-diff-manager-health-migration-reuse-44a77cb-20260924.md`. Current post-anchor carrier source changes are in separately challenged MemoryCarrier / Recovery-Reno-PTO owners, not the reviewed ConcurrentCarrierManager/health/migration semantic owners.
- **Release/item-4 factual reconciliation completed for this cluster** at `fe2243a89e974991877972f43b9f13bd4b261ebf`; note: `docs/reviews/release-item4-reconciliation-recovery-carrierstate-44fd3d6-20260924.md`. The release packet remains conservative; its older coverage marker understates later review work rather than asserting a false completion. Do not rewrite the large evidence index on every small slice.
- Release items **3 and 4 remain incomplete**. `RELEASE_CANDIDATE=false`, `PRODUCTION_READY=false`, `FREEZE=false`, `RELEASED=false` remain unchanged.
- **`READY_LIVE: none`.** Standing self-owned VPS authorization remains valid, but do not repeat HY2, warm failover, periodic/soak, package lifecycle, migration-back, endpoint/key migration, IPv6, PLPMTUD or Experimental Track without a materially new code/instrumentation/hypothesis/path-condition question.

## FRONT — R-PKG/BLD-DIFF package / reproducibility / dependency-build current-owner reuse check

The coding agent/reviewer-support agent should continue immediately; reviewer cadence is not a ticket boundary. This is a bounded owner-diff/reuse lane, not a request to manufacture packaging churn.

1. Re-read exact-current `Cargo.toml`, `Cargo.lock`, crate manifests, `scripts/release/build-package.sh`, `scripts/release/check-clean-source-test.sh`, `scripts/release/smoke-package.sh`, `scripts/release/smoke-package-test.sh`, `scripts/check.sh`, and any native/build hooks. Compare semantic owners against the last dedicated package/reproducibility review (`71de29cf`) and dependency/build review (`e11c1d28`).
2. Challenge only moved/current claims: manifest/feature drift, lock/reproducibility assumptions, native hooks, build-script execution, source-state guard, archive/layout validation, unsafe inheritance, operator-script cleanup/identity boundary. Do not reopen signing, SBOM, key-custody or publication policy — those remain external/policy gates.
3. If semantic owners are unchanged, write one precise bounded owner-diff no-finding note with inspected owners and exact reachable anchor. Do not create checker/schema/docs filler merely to produce a commit.
4. If a concrete defect exists and current committed semantics decide the answer, smallest repair + positive/negative regression + final pushed exact-tree local gate + reachable provenance, then continue immediately to the next lane.
5. No wire/parser/crypto framing change is expected; do not mechanically run fuzz.

## Dependency-ready rolling queue

Keep roughly 8–15 real coherent slices where repository truth supports them. Do not collapse this to one small ticket and do not declare queue exhaustion from a narrow no-finding.

1. **R-PKG/BLD-DIFF — package/reproducibility/operator + dependency/build owner-diff reuse.** FRONT above.
2. **R-OBS/ADAPTER-DIFF — observability + Memory/UDP/TCP carrier adapter current-owner reuse check.** Candidate B remains closed unless the datagram projection owner moved. Re-challenge only moved close/error/resource/projection/high-water seams.
3. **R-FS/SESSION-DIFF — FairScheduler / multi-stream / SessionRuntime / flow-control owner diff.** Reuse prior bounded reviews for unchanged owners; challenge moved queue/session/stream/DeliveryAck/lifecycle/resource seams only.
4. **R-CLI/BND-DIFF — CLI exit-code / JSON / human output + cross-platform process + algorithmic boundedness reconciliation.** Keep Linux evidence distinct from other-OS execution; do not invent timeout/capacity/security values.
5. **R-REC-REFILL — repaired Recovery owner residual challenge.** At the H-I4-116/117/118 repaired tree, verify no changed seam invalidates frame ownership, RTT/PTO reset, Reno accounting, fault-simulation, or health bridge review boundaries. Prefer a precise no-finding if already covered; do not manufacture another recovery framework.
6. **R-PREAUTH-DIFF — pre-auth rejection/accounting owner diff.** Reuse the existing bounded review if owners are unchanged. D019 source-retention/no-reset remains maintainer/security policy and must not be silently decided.
7. **R-RELEASE-PACKET-DIFF — grouped release-packet/status evidence-index update only after several additional slices.** Reconcile source anchors/provenance classes; never imply one SHA executed all historical tests.
8. **REFILL-1 — repository-wide 13-surface current-owner inventory.** Explicitly inventory all thirteen core surfaces from the standing reviewer policy and mark current dedicated challenge/reuse anchor for each.
9. **REFILL-2 — challenge any implemented core owner that inventory shows has moved or lacks a reachable dedicated bounded review.** A no-finding bounded independent review is valid item-4 support.
10. **CONDITIONAL LIVE only on a changed question.** If new code/instrumentation/hypothesis/path condition creates a concrete unresolved real-network question under standing authorization, queue it; otherwise `READY_LIVE: none` remains correct.
11. **FINAL factual reconciliation only when the broad inventory materially closes.** Item 4 stays unchecked until remaining independent-review/policy/external boundaries are truthfully resolved; no automatic RC/freeze/release transition.

## Current REUSE map

- **Recovery:** R-REC-1 future/unsent ACK no-finding `5bfa9f4`; R-REC-2 frame/retransmit ownership `791e27d`; R-REC-3 RTT/PTO `2ac2278`; R-REC-4a/4b positive-loss/fault-simulation at `4497752`; H-I4-116/117/118 repair/provenance chain current through `71b62c6`.
- **CarrierState:** current exact R-CS-1/R-CS-2 notes above. Do not infer D064 runtime migration policy from the older M0 model.
- **Concurrent Carrier Manager / health / migration-back:** reuse `a14cf47`, `5e73aead`, `a2a1e6cf` plus current `R-CM-DIFF` unless those semantic owners move.
- **FairScheduler + multi-stream/flow control:** reuse `docs/reviews/reviewer-i4-fs1-fair-scheduler-44a0073-20260921.md` plus I4-FS2 closure after H-I4-086/087 (`cd182ade` / `982ee6da`) unless owners move.
- **Carrier adapters:** reuse I4-AD1 MemoryCarrier (`8e905163`), I4-AD2a UDP (`5f24cbff`), I4-AD2b TCP (`e8fbc65e`) subject to current owner-diff check.
- **SessionRuntime:** reuse lifecycle/terminal review chain plus I4-FS2 DeliveryAck/queue-cap coverage. Retained-history capacity remains maintainer/security policy.
- **Observability:** reuse independent stable-v1 re-close `18f7c58f` while owner unchanged; Candidate B remains closed.
- **Package/reproducibility/operator:** reuse `71de29cf` if semantic owners unchanged; signing/SBOM/key-custody/publication remain external/policy.
- **Dependency/build:** reuse I4-BLD `e11c1d28` if manifests/features/native hooks/unsafe inheritance unchanged.
- **Cross-platform CLI/process:** reuse `docs/reviews/independent-cli-cross-platform-process-416fd5e-20260924.md`; Linux-only execution is not other-OS proof.
- **CLI output contract:** reuse I4-CLI-H `536d59a5` and the current machine/human contract chain unless output owners move.
- **Algorithmic/resource boundedness:** reuse `docs/reviews/independent-i4-bnd-current-reconciliation-934c878-20260924.md`; no invented capacity values.
- **Pre-auth:** reuse `docs/reviews/independent-preauth-rejection-accounting-d96aabe-20260922.md`; D019 remains separate.

## Evidence discipline / stop conditions

- Developer-reported local CI, persisted developer-local provenance, reviewer source/control-flow review, reviewer-local generic OS checks, hosted CI, live WAN evidence and performance conclusions are distinct classes.
- Accepted exact-tree provenance must anchor a GitHub-resolvable pushed SHA; never publish local-only/unreachable SHA as shared evidence.
- Ordinary READY_LOCAL source/test repair ends with final pushed SHA developer-local `PYTHONDONTWRITEBYTECODE=1 bash scripts/check.sh`, `git diff --check`, clean-tree verification and persisted exact-tree provenance. Hosted Actions are cross-evidence, never a waiting condition.
- Fuzz only for material wire decoder/parser/crypto-framing changes using the pinned toolchain and required decode build/run.
- Never decide D019; TTL/LRU/history/capacity/security numbers; signing/key-custody/SBOM/publication; previous frozen release policy; core Session/Carrier/ACK/crypto/wire architecture; destructive/canonical migration; RC/freeze/release/production authority.
- A new correctness/security/evidence BLOCKER/HIGH immediately becomes FRONT. Otherwise continue the rolling queue without waiting for the next reviewer.
- Queue exhaustion is legal only after the broad 13-surface inventory shows: no unreviewed moved current core owner, no concrete defect, no READY review-support lane, no READY live question, and all remaining items are genuinely policy/external/environment/release-authority gated.