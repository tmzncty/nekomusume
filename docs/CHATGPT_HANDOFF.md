# ChatGPT reviewer handoff — H-I4-117 runtime persistent-congestion wiring FRONT

## Current repository truth

- Synchronize to current `main` before work. Code, tests, reachable pushed commits and current specs outrank this handoff, chat memory and stale checkbox state.
- H-I4-116 is **CLOSED**. Developer source/test anchor `4265033b65031d96f106862267391685f86c6bfa` keeps `Reno::lost(0)` a no-op and adds the loss-free owner-spanning regression. Reachable `docs/notes/h-i4-116-provenance-4265033-20260924.md` records the developer-local clean exact-tree `scripts/check.sh`, `git diff --check`, clean tree, Linux x86_64 and stable Rust provenance. Exact `4265033` had no visible hosted combined-status entries/workflow runs at reviewer time; do not relabel developer-local provenance as hosted CI.
- New reviewer finding **H-I4-117 HIGH correctness / release-evidence reliability** is recorded at `docs/reviews/reviewer-h-i4-117-runtime-persistent-congestion-8b2f2f3-20260924.md` (reviewed exact source anchor `8b2f2f382eca62217110c7e604f1d38bf97efd5c`).
- H-I4-117 is an integration gap in already-committed semantics, not a request to choose a new congestion policy: `Recovery::on_pto` counts the committed three-PTO threshold; `Reno` has the committed persistent-congestion collapse; `PathRecovery::persistent_congestion()` explicitly applies that collapse; but exact-current `ReliableUdpRuntime::pto_probe()` increments PTO through `PathRecovery::on_pto(4)` and never invokes the persistent-congestion effect. A real runtime can therefore cross the committed persistent-congestion threshold while Reno keeps the pre-collapse window.
- The old `pacing_and_send_decision_are_deterministic_and_accounted` test does not prove its own “persistent congestion collapses the window” comment: it never drives PTO count to threshold and performs no assertion after `r.persistent_congestion()`. Historical commit `4dc3121861d0a066940bc8bc98438f7cbd6cd39d` claimed the collapse in its message, so H-I4-117 also closes an evidence gap.
- R-REC-3 (`2ac22784051d0870bd6645495a6df23394cc97bb`) remains a valid bounded no-finding for its **narrow owner**: RTT/PTO arithmetic, `Recovery::on_pto`, threshold-event counting and quiesce history. It did not review the `PathRecovery`/`ReliableUdpRuntime` application of Reno persistent congestion and is not contradicted by H-I4-117.
- H-I4-097..115 process/socket/thread ownership findings remain closed for their challenged owners unless those owners materially move. Candidate A (future/unsent ACK) and Candidate B (mixed datagram-drop reasons) remain closed unless exact-current owners move.
- Release items **3 and 4 remain incomplete**. `RELEASE_CANDIDATE=false`, `PRODUCTION_READY=false`, `FREEZE=false`, `RELEASED=false` remain unchanged.
- **`READY_LIVE: none`.** Do not repeat HY2, warm failover, periodic/soak, package lifecycle, migration-back, endpoint/key migration, IPv6, PLPMTUD or Experimental Track merely for freshness. A live lane requires a new concrete unresolved real-network question created by code/instrumentation/hypothesis/path-condition movement.

## FRONT — H-I4-117 smallest current-semantics repair

The coding agent should continue immediately; reviewer cadence is not a work-ticket boundary.

1. Wire the already-existing `PathRecovery::persistent_congestion()` effect into the real `ReliableUdpRuntime::pto_probe()` transition after a legitimate non-empty-runtime PTO increments the underlying count. Do **not** change `PERSISTENT_CONGESTION_PTO_THRESHOLD`, invent a second threshold, or redesign Reno/PTO/ACK/wire/crypto semantics.
2. Add a deterministic runtime regression that would fail on the reviewed tree. Suggested shape using public behavior only:
   - create a runtime with MSS 1200 and admit one small ack-eliciting packet so PTO is legitimate;
   - after PTO 1 and PTO 2, prove a discriminating `can_send(...)` admission still reflects the non-collapsed window;
   - after PTO 3, prove admission tightens to the existing `2 * MSS` persistent-congestion window;
   - ACK/reset the streak and prove a later sub-threshold PTO does not immediately re-trigger collapse.
   Choose bytes that discriminate the existing window states; do not add a production-only cwnd debug API merely for the test.
3. Strengthen or replace the old PathRecovery checkpoint so its persistent-congestion claim is executable: below-threshold negative control + threshold positive control + a post-collapse `can_send` assertion. Preserve the existing exact threshold and all ACK/RTT/PTO/frame ownership behavior.
4. Keep PTO probe selection, stable FrameId/plaintext ownership, retransmit accounting, health freshness, Reno ordinary-loss behavior and teardown semantics unchanged. This repair does not touch decoder/parser/crypto framing; do **not** mechanically run fuzz.
5. Push the final source/test SHA and run the developer-local clean exact-tree gate in a safe clean checkout/worktree:
   - `PYTHONDONTWRITEBYTECODE=1 bash scripts/check.sh`
   - `git diff --check`
   - verify clean tree.
   Persist reachable provenance with exact SHA, UTC start/end, command exit codes, OS/arch, stable Rust version and clean-tree state. No secrets/private topology/credentials/unnecessary absolute paths.
6. Immediately continue R-REC-4 after H-I4-117 closure; do not stop at a green repair and wait for the next reviewer.

## Dependency-ready rolling queue

Keep a real deep queue; unchanged REUSE surfaces are not duplicated merely to inflate ticket count.

1. **H-I4-117 repair/tests/exact-tree provenance** — FRONT above.
2. **R-REC-4a — real packet-threshold loss + Reno aggregate accounting.** Build a deterministic PathRecovery/runtime case with actual `released_lost > 0`; prove one aggregate loss release, bytes-in-flight reconciliation, one Reno reduction and tightened admission. This is the positive-loss integration control missing from the current H-I4-116 regression. No new congestion values.
3. **R-REC-4b — deterministic fault simulation accounting.** Challenge `simulate_fault_profile`/`simulate_delivery` drop-every, burst, reorder and blackhole bounds, retransmit counts, round bound, overflow/fail-closed behavior, and correspondence to the claims actually made by release/item-4 evidence. No capacity-pressure benchmark.
4. **R-CS-1 — `CarrierState` generation / validation / hysteresis.** Challenge stale/old/future generation rejection, rejection atomicity, validation-vs-packet-feedback separation, dwell/success accumulation/reset, and hysteresis gates against exact-current owner/spec. Do not invent new hysteresis values.
5. **R-CS-2 — `CarrierState` single-active / drain / fail / activate.** Challenge active-owner uniqueness, degradation/drain/fail legality, active clearing, terminal/fresh-generation behavior where committed, and active-epoch progression without architecture redesign.
6. **R-CM-DIFF — Concurrent Carrier Manager / health / migration-back owner-diff reuse check.** Reuse prior independent challenge if owners are unchanged; if a relevant owner moved, issue a bounded current-owner challenge rather than reopening the subsystem mechanically.
7. **R-RPKT-1 — release packet factual/evidence reconciliation.** After H-I4-116/117 and the R-REC-4 group, reconcile finding closure, source anchors, provenance classes, no-finding scope and release-item-4 boundaries. Never imply one SHA ran every historical test.
8. **R-PKG/BLD-DIFF — package/reproducibility/operator + dependency/build reuse check.** Reopen only if manifests/features/native hooks/release scripts/unsafe inheritance moved. Signing/SBOM/key-custody/publication remain policy/external gates.
9. **R-OBS/ADAPTER-DIFF — observability + Memory/UDP/TCP adapter current-owner reuse check.** Candidate B stays closed while semantic owners are unchanged.
10. **R-FS/SESSION-DIFF — FairScheduler / multi-stream / SessionRuntime / flow-control current-owner spot challenge.** Reuse prior bounded reviews for unchanged owners; if source moved, challenge the moved accounting/lifecycle/resource seam.
11. **R-CLI/BND-DIFF — CLI exit-code/JSON/human-output + cross-platform process + algorithmic boundedness reconciliation.** Keep Linux evidence distinct from other-OS execution; do not invent timeout/capacity/security values.
12. **REFILL — repository-wide 13-surface current-owner inventory.** Item 4 remains incomplete: any implemented core owner lacking a reachable dedicated bounded challenge is valid review-support work. Queue exhaustion is legal only after the broad inventory shows no unreviewed current core owner, no defect, no READY review-support lane and no READY live question.
13. **CONDITIONAL LIVE only on a changed question.** Standing VPS authorization remains valid, but current classification is `READY_LIVE: none`.

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
- A correctness/security/evidence BLOCKER/HIGH stays at the front until closed; independent unaffected READY local review work remains available once the blocker repair is underway/completed.
