# ChatGPT reviewer handoff — R-REC-4a positive-loss integration FRONT

## Current repository truth

- Synchronize to current `main` before work. Code, tests, reachable pushed commits and current specs outrank this handoff, chat memory and stale checkbox state.
- **H-I4-116 is CLOSED.** Developer source/test anchor `4265033b65031d96f106862267391685f86c6bfa` keeps `Reno::lost(0)` a no-op and adds the loss-free owner-spanning regression. Reachable `docs/notes/h-i4-116-provenance-4265033-20260924.md` records the developer-local clean exact-tree gate. Do not relabel developer-local provenance as hosted CI.
- **H-I4-117 is CLOSED** at source/test anchor `2ba5b960ebb13f1bfe6c5e4b07438385f5aab673`. `ReliableUdpRuntime::pto_probe` now applies the existing `PathRecovery::persistent_congestion()` effect after a legitimate non-empty-runtime PTO, the threshold checkpoint is executable, and the orchestration test tolerates post-collapse congestion refusal while continuing PTO/health work. Reachable `docs/notes/h-i4-117-provenance-2ba5b96-20260924.md` records `PYTHONDONTWRITEBYTECODE=1 bash scripts/check.sh` exit 0, `git diff --check` exit 0, clean tree, UTC 2026-09-24T01:54:02Z → 01:59:50Z, Linux x86_64, rustc 1.98.0 stable. Reviewer query of exact `2ba5b96` GitHub combined status returned zero status entries; that is not a hosted pass/fail.
- Reviewer bounded challenge **R-REC-4a** is recorded at `docs/reviews/reviewer-r-rec-4a-positive-loss-integration-80542a9-20260924.md` (reviewed exact source anchor `80542a94f5bb54de6754bc15f8067eea722576ad`; reviewer note commit `4d8b77de758867134dfa21a31716d15ac603a82e`). No concrete correctness defect was found in the inspected packet-threshold/Reno source path, but the dedicated owner-spanning positive-loss regression is still missing. This is a real READY_LOCAL item-4 support slice, not queue exhaustion and not a policy gate.
- R-REC-1 (`5bfa9f4a4da9efe9c1e3049c21f75e60f44fdffa`), R-REC-2 (`791e27d7a307f4360d7876448a5bc5bdb47c457b`) and R-REC-3 (`2ac22784051d0870bd6645495a6df23394cc97bb`) remain valid for their bounded scopes. R-REC-3 covers lower-level RTT/PTO/threshold-event accounting, not runtime Reno application.
- H-I4-097..115 remain closed for their challenged owners unless those owners materially move. Candidate A (future/unsent ACK) and Candidate B (mixed datagram-drop reasons) remain closed unless exact-current owners move.
- Release items **3 and 4 remain incomplete**. `RELEASE_CANDIDATE=false`, `PRODUCTION_READY=false`, `FREEZE=false`, `RELEASED=false` remain unchanged.
- **`READY_LIVE: none`.** Do not repeat HY2, warm failover, periodic/soak, package lifecycle, migration-back, endpoint/key migration, IPv6, PLPMTUD or Experimental Track merely for freshness. A live lane requires a new concrete unresolved real-network question created by code/instrumentation/hypothesis/path-condition movement.

## FRONT — R-REC-4a positive packet-threshold loss + Reno aggregate accounting

The coding agent should continue immediately; reviewer cadence is not a work-ticket boundary.

1. Re-read exact-current `neko-reliable::Recovery::on_ack`, `Reno::{acked,lost,can_send}`, `PathRecovery::{on_sent,on_ack}`, `ReliableUdpRuntime::apply_ack`, and the R-REC-4a reviewer note. Current source reasoning is coherent: `Recovery::on_ack` removes ACKed packets before computing loss; `PathRecovery::on_ack` removes each packet's `charged` ownership once, aggregates ACKed/lost charged bytes separately, then calls `reno.acked(released_acked)` once and `reno.lost(released_lost)` once. Do not redesign this merely to create churn.
2. Add one focused deterministic owner-spanning regression that makes the positive-loss accounting executable in one place. A suitable shape, if it still matches exact-current semantics after synchronization, is:
   - MSS 1200, equal 400-byte ack-eliciting packets with distinct stable FrameIds;
   - send packet numbers `0..=5`;
   - ACK only packet `5`, so the existing packet threshold declares older packets far enough behind as lost while packets inside the threshold remain outstanding;
   - assert the exact ACKed/lost packet sets, retransmit FrameIds, released ACK/loss bytes where exposed, and post-transition bytes-in-flight;
   - prove the one ACK transition causes **one aggregate** Reno reduction and tightened admission. Prefer test-module access or discriminating `can_send(...)`; do not add a production-only cwnd debug API and do not introduce new congestion-policy values.
   The exact expected sets/bytes must be derived from current code, not copied blindly from this handoff if owners moved.
3. Preserve H-I4-116's loss-free control: `released_lost == 0` must not reduce `cwnd`/`ssthresh`/bytes-in-flight. Preserve packet/frame ownership, RTT/PTO, Session-delivery separation, D019 and all current wire/crypto semantics.
4. Run focused tests while iterating. On the **final pushed source/test SHA**, run the developer-local clean exact-tree gate in a safe clean checkout/worktree:
   - `PYTHONDONTWRITEBYTECODE=1 bash scripts/check.sh`
   - `git diff --check`
   - verify clean tree.
   Persist reachable provenance with exact SHA, UTC start/end, command exit codes, OS/arch, stable Rust version and clean-tree state. Do not record secrets/private topology/credentials/unnecessary absolute paths.
5. No decoder/parser/crypto-framing change is expected; do **not** mechanically run fuzz.
6. After R-REC-4a closure, continue immediately into R-REC-4b. Do not wait for the next reviewer merely because the focused regression and full gate are green.

## Dependency-ready rolling queue

Keep the deep queue intact; unchanged REUSE surfaces are not duplicated merely to inflate ticket count.

1. **R-REC-4a — positive packet-threshold loss + Reno aggregate accounting regression/provenance.** FRONT above.
2. **R-REC-4b — deterministic fault simulation accounting.** Challenge `simulate_fault_profile` / `simulate_delivery`: no-loss, drop-every, finite burst, reorder and blackhole behavior; exact sent/delivered/retransmitted/round accounting; round bound; invalid-limit/overflow/fail-closed behavior; and correspondence to release/item-4 claims. Blackhole is a negative/failure profile, so do not silently reinterpret `delivered=0` as success. No capacity-pressure benchmark and no new policy values.
3. **R-CS-1 — `CarrierState` generation / validation / hysteresis.** Challenge stale/old/future generation rejection, rejection atomicity, validation-vs-packet-feedback separation, dwell/success accumulation/reset, and hysteresis gates against exact-current owner/spec. Do not invent new hysteresis values.
4. **R-CS-2 — `CarrierState` single-active / drain / fail / activate.** Challenge active-owner uniqueness, degradation/drain/fail legality, active clearing, terminal/fresh-generation behavior where committed, and active-epoch progression without architecture redesign.
5. **R-CM-DIFF — Concurrent Carrier Manager / health / migration-back owner-diff reuse check.** Reuse prior independent challenge if owners are unchanged; if a relevant owner moved, issue a bounded current-owner challenge rather than mechanically reopening the subsystem.
6. **R-RPKT-1 — release packet factual/evidence reconciliation.** After the R-REC-4 and CarrierState cluster (or after 3–4 coherent slices), reconcile H-I4-116/117 closure, R-REC no-finding/test anchors, provenance classes, release-packet wording and release-item-4 boundaries. Never imply one SHA ran every historical test.
7. **R-PKG/BLD-DIFF — package/reproducibility/operator + dependency/build reuse check.** Reopen only if manifests/features/native hooks/release scripts/unsafe inheritance moved. Signing/SBOM/key-custody/publication remain policy/external gates.
8. **R-OBS/ADAPTER-DIFF — observability + Memory/UDP/TCP adapter current-owner reuse check.** Candidate B stays closed while semantic owners are unchanged; reopen only moved owners.
9. **R-FS/SESSION-DIFF — FairScheduler / multi-stream / SessionRuntime / flow-control current-owner spot challenge.** Reuse prior bounded reviews for unchanged owners; if source moved, challenge the moved accounting/lifecycle/resource seam.
10. **R-CLI/BND-DIFF — CLI exit-code/JSON/human-output + cross-platform process + algorithmic boundedness reconciliation.** Keep Linux evidence distinct from other-OS execution; do not invent timeout/capacity/security values.
11. **REFILL — repository-wide 13-surface current-owner inventory.** Item 4 remains incomplete: any implemented core owner lacking a reachable dedicated bounded challenge is valid review-support work. Queue exhaustion is legal only after the broad inventory shows no unreviewed current core owner, no concrete defect, no READY review-support lane and no READY live question.
12. **CONDITIONAL LIVE only on a changed question.** Standing VPS authorization remains valid, but current classification is `READY_LIVE: none`.

## Current REUSE map

- **Recovery R-REC-1:** future/unsent ACK + range/state immutability no-finding `5bfa9f4a4da9efe9c1e3049c21f75e60f44fdffa`.
- **Recovery R-REC-2:** loss/retransmit ownership no-finding `791e27d7a307f4360d7876448a5bc5bdb47c457b`.
- **Recovery R-REC-3:** RTT/PTO/threshold-event-counting no-finding `2ac22784051d0870bd6645495a6df23394cc97bb`; scope is `neko-reliable::Recovery`, not runtime Reno application.
- **Concurrent Carrier Manager / health / migration-back:** reuse `5e73aead` and `a2a1e6cf` unless owners move.
- **FairScheduler + multi-stream/flow control:** reuse `docs/reviews/reviewer-i4-fs1-fair-scheduler-44a0073-20260921.md` plus I4-FS2 closure after H-I4-086/087 (`cd182ade` / `982ee6da`) unless owners move.
- **Carrier adapters:** reuse I4-AD1 MemoryCarrier (`8e905163`), I4-AD2a UDP (`5f24cbff`), I4-AD2b TCP (`e8fbc65e`).
- **SessionRuntime:** reuse the lifecycle/terminal review chain plus I4-FS2 DeliveryAck/queue-cap coverage. Retained-history capacity remains maintainer/security policy.
- **Observability:** reuse independent stable-v1 re-close `18f7c58f` while owner unchanged; Candidate B remains closed.
- **Package/reproducibility/operator:** reuse `71de29cf`; signing/SBOM/key-custody/publication remain external/policy.
- **Dependency/build:** reuse I4-BLD `e11c1d28` unless manifests/features/native hooks/unsafe inheritance move.
- **Cross-platform CLI/process:** reuse `docs/reviews/independent-cli-cross-platform-process-416fd5e-20260924.md`; do not turn Linux-only evidence into other-OS proof.
- **CLI output contract:** reuse I4-CLI-H `536d59a5` and current machine/human contract chain unless product output owners move.
- **Algorithmic/resource boundedness:** reuse `docs/reviews/independent-i4-bnd-current-reconciliation-934c878-20260924.md`; no new capacity values.
- **Pre-auth rejection/accounting:** reuse `docs/reviews/independent-preauth-rejection-accounting-d96aabe-20260922.md`; D019/RSEC policy remains separate.

## Evidence discipline and stop conditions

- Developer-reported local CI, repository-persisted developer-local provenance, reviewer source/control-flow review, reviewer-local generic OS checks, hosted CI, live WAN evidence and performance conclusions are distinct evidence classes.
- Accepted exact-tree provenance must anchor a GitHub-resolvable pushed SHA; never publish local-only/unreachable SHA as shared evidence.
- Ordinary READY_LOCAL source/test repair ends with the final pushed SHA's developer-local clean exact-tree gate and persisted provenance. GitHub Actions are extra cross-evidence, not a waiting condition.
- Fuzz only for material wire decoder/parser/crypto-framing changes using pinned `scripts/fuzz-toolchain.sh` and the required decode build/run commands.
- Never decide D019 source-retention/no-reset policy; TTL/LRU/history/capacity/security values; signing/key-custody/SBOM/publication policy; previous frozen release policy; core Session/Carrier/ACK/crypto/wire architecture; destructive/canonical migration; RC/freeze/release/production authority.
- A correctness/security/evidence BLOCKER/HIGH stays at the front until closed. No such new blocker was found in the R-REC-4a source review; the current FRONT is a dedicated item-4 positive-control test/evidence closure, after which the agent must continue the queue.
