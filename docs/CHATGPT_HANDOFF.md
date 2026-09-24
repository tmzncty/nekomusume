# ChatGPT reviewer handoff — recovery/CarrierState + four owner-diff reuse slices reconciled; continue refill queue

## Current repository truth

- Synchronize to current `main` before work. Exact code/tests/reachable commits/current specs outrank this handoff, chat memory and stale checkbox state.
- **Last developer source/test repair accepted:** `71b62c642a6ea736fd162653ede9b29868e08eee` closes H-I4-118 by restricting loss candidates in `Recovery::on_ack` to `n <= largest`; focused stale-ACK regression is present. Reachable developer-local exact-tree provenance: `docs/notes/h-i4-118-provenance-71b62c6-20260924.md` (`scripts/check.sh` 0, `git diff --check` 0, clean tree, Linux x86_64, rustc 1.98.0). Combined GitHub status for that SHA had no status entries when reviewed; do not call the local record hosted CI.
- H-I4-116 (Reno zero-loss collapse) and H-I4-117 (runtime persistent-congestion wiring) remain closed with reachable developer-local provenance. H-I4-097..115 remain closed unless their exact semantic owners materially move. Candidate A (future/unsent ACK) and Candidate B (mixed datagram-drop reasons) remain closed unless exact-current owners move.
- **Recovery cluster:** R-REC-1/2/3 bounded reviews remain valid in their stated scopes; R-REC-4a positive aggregate Reno reduction and R-REC-4b deterministic fault-simulation review are closed. H-I4-118 is part of the repaired recovery boundary; do not reopen by incorrectly requiring a legal duplicate ACK number to remain present in `sent`.
- **CarrierState / manager cluster:** R-CS-1 and R-CS-2 are closed bounded no-finding; R-CM-DIFF is closed bounded no-finding/reuse. The old bounded M0 `CarrierState` model is not proof of richer D064 runtime-manager migration ordering.
- **R-PKG/BLD-DIFF CLOSED no-finding/reuse** at review commit `12374ee99f5bf5bdd5096e8d5e24a88d9046d188`; note `docs/reviews/independent-r-pkg-bld-diff-5135d8e-20260924.md`. Root/lock/crate manifests, release scripts, `scripts/check.sh` and repository build-hook surface are unchanged from the dedicated package/dependency anchors; no signing/SBOM/key-custody/publication policy was reopened.
- **R-OBS/ADAPTER-DIFF CLOSED no-finding/reuse** at `e228ec61a25a54e77de9b5ba38a4fda4393c1191`; note `docs/reviews/independent-r-obs-adapter-diff-12374ee-20260924.md`. Observability owner is unchanged from stable-v1 re-close; Memory/UDP/TCP adapter semantic sections remain unchanged from their dedicated 2026-09-21 review chain. Candidate B remains closed.
- **R-FS/SESSION-DIFF CLOSED no-finding/reuse** at `33b6a3c907ba0d37913def5cc98fd1402900bfec`; note `docs/reviews/independent-r-fs-session-diff-e228ec6-20260924.md`. `neko-session` source is unchanged from FS2 closure and exact-current `FairScheduler` matches the reviewed semantic owner; later multistream edits are separately reviewed process/deadline hardening.
- **R-CLI/BND-DIFF CLOSED no-finding/reuse** at `a34f38f0a6cda7086195a45562307035a476ad69`; note `docs/reviews/independent-r-cli-bnd-diff-33b6a3c-20260924.md`. CLI process/output semantic owners did not move after the current 2026-09-24 process/cross-platform reviews; Linux evidence remains distinct from other-OS execution. Existing `SessionRuntime.events` retained-history cap remains `POLICY_BLOCKED_RESOURCE_BOUND`; no capacity value was invented.
- **Grouped release/item-4 reconciliation completed** at `d4e2e42b7a16e6946771c44bc436be9017c6993a`; note `docs/reviews/release-item4-reconciliation-owner-diff-reuse-a34f38f-20260924.md`. The release packet remains conservative and does not need a large rewrite after every small review note.
- Release items **3 and 4 remain incomplete**. `RELEASE_CANDIDATE=false`, `PRODUCTION_READY=false`, `FREEZE=false`, `RELEASED=false` remain unchanged.
- **`READY_LIVE: none`.** Standing self-owned VPS authorization remains valid, but do not repeat HY2, warm failover, periodic/soak, package lifecycle, migration-back, endpoint/key migration, IPv6, PLPMTUD or Experimental Track without a materially new code/instrumentation/hypothesis/path-condition question.

## FRONT — R-REC-REFILL repaired Recovery residual challenge

Continue immediately; reviewer cadence is not a ticket boundary. This is a bounded residual challenge of the **repaired exact-current Recovery/Reno/PTO integration**, not a request to build another recovery framework or re-prove already closed slices mechanically.

1. Re-read exact-current `crates/neko-reliable/src/lib.rs`, the `PathRecovery` / `ReliableUdpRuntime` recovery bridge in `crates/neko-carrier/src/lib.rs`, their focused deterministic tests, and the current R-REC/H-I4-116/117/118 review/provenance notes.
2. Challenge seams that actually moved through H-I4-116/117/118: zero-loss Reno no-op, positive aggregate loss reduction, ACK/loss eligibility ordering, RTT/PTO reset preservation, persistent-congestion threshold application, frame/retransmit ownership, deterministic fault-simulation invariants, and the health bridge. Do not duplicate R-REC-1/2/3/4a/4b unless a moved owner invalidates the old proof.
3. In particular, try to construct a focused deterministic counterexample where a legal duplicate/stale ACK, zero-loss ACK, positive loss, or ACK-after-PTO sequence mutates state outside its evidence domain: packet/frame ownership, RTT/PTO counters, Reno bytes/cwnd, retransmit work, or manager health.
4. If no concrete defect is found, write a precise bounded no-finding note listing exact owners/tests and exclusions. No code churn for its own sake.
5. If a correctness defect exists and current committed semantics decide the answer, it becomes FRONT immediately: smallest repair + positive/negative regression + final pushed exact-tree developer-local gate/provenance, then continue the rolling queue.
6. Do not change ACK architecture, D019, congestion/security values or wire/crypto semantics. No decoder/parser/crypto framing change is expected; do not mechanically run fuzz.

## Dependency-ready rolling queue

Keep roughly 8–15 real coherent slices where repository truth supports them. Do not collapse this to one ticket and do not declare queue exhaustion from a narrow no-finding.

1. **R-REC-REFILL — repaired Recovery residual challenge.** FRONT above.
2. **R-PREAUTH-DIFF — pre-auth rejection/accounting owner diff.** Reuse the existing bounded review if owners are unchanged; challenge only moved responder/inventory/accounting/resource seams. D019 source-retention/no-reset remains maintainer/security policy.
3. **R-CORE-INV — repository-wide 13-surface current-owner inventory.** Explicitly record current dedicated review/reuse anchor for all thirteen standing surfaces, including whether any semantic owner moved after its anchor.
4. **R-CORE-MOVED-1 — first dependency-ready moved/unreviewed implemented core owner from the inventory.** No-finding bounded review is valid item-4 support; concrete defect converts to repair.
5. **R-CORE-MOVED-2 — second independent moved/unreviewed core owner if inventory exposes one.** Keep independent from lane 4 where possible.
6. **R-RELEASE-PACKET-DIFF — grouped release-packet/status evidence-index update after the next coherent refill group.** Reconcile source anchors/provenance classes; never imply one SHA executed all historical tests.
7. **R-SECURITY-BOUNDARY-DIFF — current SECURITY/spec/release-boundary factual cross-check.** Only moved claims; do not turn policy absences into invented implementation requirements.
8. **R-CLI/PROC-RESIDUAL — only if the inventory shows a current CLI/process semantic owner moved after the 2026-09-24 process reviews.** Otherwise record reuse, not another framework.
9. **R-OBS/SESSION-RESIDUAL — only if later recovery/preauth work moves observability or SessionRuntime semantic owners.** Candidate B stays closed until its owner moves.
10. **CONDITIONAL LIVE only on a changed question.** If new code/instrumentation/hypothesis/path condition creates a concrete unresolved real-network question under standing authorization, queue it; otherwise `READY_LIVE: none` remains correct.
11. **FINAL broad factual reconciliation only after the 13-surface inventory and moved-owner challenges materially close.** Item 4 stays unchecked until remaining independent-review/policy/external boundaries are truthfully resolved; no automatic RC/freeze/release transition.

## Current REUSE map

- **Recovery:** R-REC-1 future/unsent ACK no-finding `5bfa9f4`; R-REC-2 frame/retransmit ownership `791e27d`; R-REC-3 RTT/PTO `2ac2278`; R-REC-4a/4b positive-loss/fault-simulation at `4497752`; H-I4-116/117/118 repair/provenance chain current through `71b62c6`. FRONT is a residual changed-seam challenge, not blanket re-review.
- **CarrierState:** R-CS-1/R-CS-2 current 2026-09-24 no-finding notes; do not infer D064 runtime migration policy from the older M0 model.
- **Concurrent Carrier Manager / health / migration-back:** reuse `a14cf47`, `5e73aead`, `a2a1e6cf` plus R-CM-DIFF unless semantic owners move.
- **FairScheduler + multi-stream/flow control:** reuse `reviewer-i4-fs1-fair-scheduler-44a0073-20260921.md`, I4-FS2 closure after H-I4-086/087, plus current `R-FS/SESSION-DIFF`.
- **Carrier adapters:** reuse I4-AD1 MemoryCarrier, I4-AD2a UDP, I4-AD2b TCP plus current `R-OBS/ADAPTER-DIFF`.
- **SessionRuntime:** reuse lifecycle/terminal review chain + I4-FS2 + current R-FS/SESSION-DIFF. Retained-history capacity remains maintainer/security policy.
- **Observability:** stable-v1 re-close `18f7c58f` + current R-OBS/ADAPTER-DIFF; Candidate B remains closed.
- **Package/reproducibility/operator:** `71de29cf` + current R-PKG/BLD-DIFF. Signing/SBOM/key-custody/publication remain external/policy.
- **Dependency/build:** I4-BLD `e11c1d28` + current R-PKG/BLD-DIFF while manifests/features/native hooks/unsafe inheritance stay unchanged.
- **Cross-platform CLI/process:** current `independent-cli-cross-platform-process-416fd5e-20260924.md` + R-CLI/BND-DIFF; Linux-only execution is not other-OS proof.
- **CLI output contract:** reuse I4-CLI-H / machine-human contract chain + R-CLI/BND-DIFF unless output owners move.
- **Algorithmic/resource boundedness:** `independent-i4-bnd-current-reconciliation-934c878-20260924.md` + R-CLI/BND-DIFF; no invented capacity values.
- **Pre-auth:** reuse `independent-preauth-rejection-accounting-d96aabe-20260922.md` if semantic owners are unchanged; D019 remains separate.
- **Release/evidence:** recovery/CarrierState reconciliation `fe2243a8` plus current four-slice reconciliation `d4e2e42`; packet remains evidence index, not approval.

## Evidence discipline / stop conditions

- Developer-reported local CI, persisted developer-local provenance, reviewer source/control-flow review, reviewer-local generic OS checks, hosted CI, live WAN evidence and performance conclusions are distinct classes.
- Accepted exact-tree provenance must anchor a GitHub-resolvable pushed SHA; never publish local-only/unreachable SHA as shared evidence.
- Ordinary READY_LOCAL source/test repair ends with final pushed SHA developer-local `PYTHONDONTWRITEBYTECODE=1 bash scripts/check.sh`, `git diff --check`, clean-tree verification and persisted exact-tree provenance. Hosted Actions are cross-evidence, never a waiting condition.
- Fuzz only for material wire decoder/parser/crypto-framing changes using the pinned toolchain and required decode build/run.
- Never decide D019; TTL/LRU/history/capacity/security numbers; signing/key-custody/SBOM/publication; previous frozen release policy; core Session/Carrier/ACK/crypto/wire architecture; destructive/canonical migration; RC/freeze/release/production authority.
- A new correctness/security/evidence BLOCKER/HIGH immediately becomes FRONT. Otherwise continue the rolling queue without waiting for the next reviewer.
- Queue exhaustion is legal only after the broad 13-surface inventory shows: no unreviewed moved current core owner, no concrete defect, no READY review-support lane, no READY live question, and all remaining items are genuinely policy/external/environment/release-authority gated.
