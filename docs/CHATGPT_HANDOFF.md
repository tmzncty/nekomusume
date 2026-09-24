# ChatGPT reviewer handoff — H-I4-119 FRONT after four owner-diff reuse slices + Recovery refill

## Current repository truth

- Synchronize to current `main` before work. Exact code/tests/reachable commits/current specs outrank this handoff, chat memory and stale checkbox state.
- **H-I4-119 OPEN / FRONT:** exact-current `Recovery::on_ack` resets `pto_count` whenever any packet is newly ACKed, including a committed `ack_eliciting=false` packet. `PathRecovery` explicitly supports/tests non-ack-eliciting sent packets and charges them zero Reno bytes because they cannot elicit an ACK by themselves. A newly ACKed non-ack-eliciting packet can therefore erase the PTO streak of a still-unresolved ack-eliciting packet and suppress/postpone the committed persistent-congestion/health evidence. Reviewer finding: `docs/reviews/reviewer-h-i4-119-non-ack-eliciting-ack-pto-reset-20260924.md`, finding commit `a98600e23c3f398b7bafb5524d1be91a0b97c408`; challenged source anchor `5d9d16a238d1f368f61c163a6d3d439f56e4f31a`.
- Important H-I4-119 boundary: current `ReliableUdpRuntime::{on_packet_sent,on_retransmit_sent}` constructs only `ack_eliciting=true` `SentPacket`s, so do not overstate this as an already-demonstrated canonical runtime ACK-only send failure. It is a correctness defect in the reusable implemented `Recovery` / `PathRecovery` semantic surface, which is in item-4 scope.
- **Last accepted developer source/test repair before H-I4-119:** `71b62c642a6ea736fd162653ede9b29868e08eee` closes H-I4-118; reachable developer-local exact-tree provenance remains `docs/notes/h-i4-118-provenance-71b62c6-20260924.md`. H-I4-116/117 are also closed with reachable provenance. H-I4-097..115 remain closed unless their exact owners materially move. Candidate A/B remain closed unless their owners move.
- **R-PKG/BLD-DIFF CLOSED** at `12374ee99f5bf5bdd5096e8d5e24a88d9046d188`; **R-OBS/ADAPTER-DIFF CLOSED** at `e228ec61a25a54e77de9b5ba38a4fda4393c1191`; **R-FS/SESSION-DIFF CLOSED** at `33b6a3c907ba0d37913def5cc98fd1402900bfec`; **R-CLI/BND-DIFF CLOSED** at `a34f38f0a6cda7086195a45562307035a476ad69`. All four are bounded source/owner-diff no-findings, not current exact-tree execution records.
- **Grouped release/item-4 reconciliation** for those four slices is reachable at `d4e2e42b7a16e6946771c44bc436be9017c6993a`; note `docs/reviews/release-item4-reconciliation-owner-diff-reuse-a34f38f-20260924.md`.
- Release items **3 and 4 remain incomplete**. `RELEASE_CANDIDATE=false`, `PRODUCTION_READY=false`, `FREEZE=false`, `RELEASED=false` remain unchanged.
- **`READY_LIVE: none`.** Standing self-owned VPS authorization remains valid, but do not repeat old live evidence absent a materially new code/instrumentation/hypothesis/path-condition question.

## FRONT — H-I4-119 non-ack-eliciting ACK must not reset unrelated PTO progress

This is a narrow current-semantics repair; no maintainer policy choice is required.

1. Re-read exact-current `Recovery::{on_sent,on_ack,on_pto}`, `SentPacket::ack_eliciting`, `PathRecovery::{on_sent,on_ack,on_pto,fresh_health_sample,persistent_congestion}`, and the exact-current non-ack-eliciting accounting tests.
2. Add a focused deterministic regression with **both** an unresolved ack-eliciting packet and a later non-ack-eliciting packet. Fire PTO(s), then ACK only the non-ack-eliciting packet. Required result: the real ack-eliciting packet remains in flight, the PTO streak is unchanged, no unrelated Reno/loss/retransmit state mutates, and the ACK is otherwise legal.
3. Add/retain the positive control: newly ACKing at least one ack-eliciting packet resets the PTO streak exactly as committed.
4. Smallest repair: while processing newly ACKed packets, remember whether any retired packet was `ack_eliciting=true`; reset `pto_count` only for that condition (or an equivalent guard covering all current callers). Do not require retired ACK numbers to remain in `sent` afterward.
5. Preserve H-I4-118 loss frontier (`largest <= largest_sent`, loss candidates `n <= largest`), H-I4-116 zero-loss Reno no-op, H-I4-117 persistent-congestion threshold/effect, RTT sampling, frame ownership, retransmit plaintext ownership, health freshness, and Session/Carrier evidence separation.
6. After focused tests, run on the **final pushed developer source SHA**: `PYTHONDONTWRITEBYTECODE=1 bash scripts/check.sh`, `git diff --check`, verify clean tree, and persist reachable exact-tree provenance with UTC start/end, exits, OS/arch and stable Rust version. Hosted CI is separate cross-evidence and never a wait condition.
7. No decoder/parser/crypto framing change is expected; do not mechanically run fuzz.
8. Once closed, continue immediately to the residual Recovery refill rather than waiting for the next reviewer.

## Dependency-ready rolling queue

Keep a real deep queue; H-I4-119 does not erase independent later review work.

1. **H-I4-119 repair/tests/provenance.** FRONT above.
2. **R-REC-REFILL continuation.** Rechallenge only residual repaired Recovery seams not consumed by H-I4-119: duplicate/stale ACK ownership, positive/zero loss Reno accounting, RTT/PTO reset, persistent congestion, frame copy/retransmit lifetime, deterministic fault simulation and health bridge. Do not rebuild the recovery framework.
3. **R-PREAUTH-DIFF.** Pre-auth rejection/accounting owner diff; reuse current review if unchanged. D019 source-retention/no-reset stays a maintainer/security policy gate.
4. **R-CORE-INV.** Repository-wide current-owner inventory across all thirteen standing surfaces, with dedicated challenge/reuse anchor and moved-owner status for each.
5. **R-CORE-MOVED-1.** First dependency-ready implemented current owner revealed by inventory as moved/unreviewed.
6. **R-CORE-MOVED-2.** Second independent moved/unreviewed current owner if available; concrete defect -> repair, bounded no-finding is valid item-4 support.
7. **R-RELEASE-PACKET-DIFF.** Grouped packet/status evidence-index update after the next coherent refill group. Never imply one SHA executed all historical tests.
8. **R-SECURITY-BOUNDARY-DIFF.** Exact-current SECURITY/spec/release-boundary factual cross-check only where claims moved; do not invent policy requirements.
9. **R-CLI/PROC-RESIDUAL only if semantic owner moved** after the 2026-09-24 process reviews; otherwise reuse.
10. **R-OBS/SESSION-RESIDUAL only if semantic owner moved** after current owner-diff closures; Candidate B remains closed otherwise.
11. **CONDITIONAL LIVE only on a changed real-network question** inside standing authorization; otherwise `READY_LIVE: none`.
12. **FINAL broad factual reconciliation only after inventory + moved-owner challenges materially close.** Item 4 remains unchecked until remaining independent-review/policy/external boundaries are truthfully resolved; no automatic RC/freeze/release transition.

## Current REUSE map

- **Recovery:** R-REC-1/2/3 remain scoped no-findings; R-REC-4a/4b positive-loss/fault-simulation closed; H-I4-116/117/118 closed through `71b62c6`; H-I4-119 now challenges the non-ack-eliciting PTO-reset seam.
- **CarrierState:** current R-CS-1/R-CS-2 no-finding notes; do not infer D064 runtime migration policy from old M0 CarrierState.
- **Concurrent Carrier Manager / health / migration-back:** current R-CM-DIFF plus prior dedicated anchors unless semantic owners move.
- **FairScheduler / multistream / SessionRuntime:** current `R-FS/SESSION-DIFF` + prior I4-FS1/FS2/lifecycle chain; retained-history capacity remains policy-blocked.
- **Carrier adapters + observability:** current `R-OBS/ADAPTER-DIFF`; Candidate B closed while projection owner unchanged.
- **Package/reproducibility/operator + dependency/build:** current `R-PKG/BLD-DIFF`; signing/SBOM/key-custody/publication remain external/policy.
- **CLI output/process + algorithmic boundedness:** current `R-CLI/BND-DIFF`; Linux evidence is not other-OS proof; no invented capacity values.
- **Pre-auth:** reuse `independent-preauth-rejection-accounting-d96aabe-20260922.md` if owners unchanged; D019 is separate.
- **Release/evidence:** recovery/CarrierState reconciliation `fe2243a8` + four-slice reuse reconciliation `d4e2e42`; packet remains evidence index, not approval.

## Evidence discipline / stop conditions

- Developer-reported local CI, persisted developer-local provenance, reviewer source/control-flow review, reviewer-local generic OS checks, hosted CI, live WAN evidence and performance conclusions are distinct classes.
- Accepted exact-tree provenance must anchor a GitHub-resolvable pushed SHA; never publish local-only/unreachable SHA as shared evidence.
- Ordinary READY_LOCAL source/test repair ends with final pushed SHA developer-local `PYTHONDONTWRITEBYTECODE=1 bash scripts/check.sh`, `git diff --check`, clean-tree verification and persisted exact-tree provenance.
- Fuzz only for material wire decoder/parser/crypto-framing changes using the pinned toolchain and required decode build/run.
- Never decide D019; TTL/LRU/history/capacity/security values; signing/key-custody/SBOM/publication; previous frozen release policy; core Session/Carrier/ACK/crypto/wire architecture; destructive/canonical migration; RC/freeze/release/production authority.
- A new correctness/security/evidence BLOCKER/HIGH immediately becomes FRONT. Otherwise continue the rolling queue without waiting for the next reviewer.
- Queue exhaustion is legal only after the broad 13-surface inventory shows no unreviewed moved core owner, no concrete defect, no READY review-support lane, no READY live question, and all remaining work is genuinely policy/external/environment/release-authority gated.
