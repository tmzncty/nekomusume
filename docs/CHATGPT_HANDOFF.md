# ChatGPT reviewer handoff — H-I4-118 remains FRONT after developer test drift

## Current repository truth

- Synchronize to current `main` before work. Code, tests, reachable pushed commits and current specs outrank this handoff, chat memory and stale checkbox state.
- **Reviewer-observed developer HEAD before this handoff:** `1021b74d685d5de77e2fd06d4b326a2dbce6129c`, one commit ahead of the prior handoff `4a72d0c761c5c3ca4217cec477b2abe0e6fd0968`.
- That developer commit is **test-only** in `crates/neko-carrier/src/lib.rs`: it adds `packet_threshold_loss_triggers_one_aggregate_reno_reduction`, an additional legitimate positive-loss / Reno-reduction control. Reviewer source inspection accepts the test shape as supplemental coverage. It does **not** modify `crates/neko-reliable::Recovery::on_ack`, does **not** repair H-I4-118, and does not justify returning to already-closed R-REC-4a work while a HIGH remains open. Exact `1021b74` had no GitHub combined-status entries when reviewed; no developer-local exact-tree provenance for that SHA was found. A later final H-I4-118 exact-tree gate may cover this unchanged test, but do not claim `1021b74` itself had a local/hosted pass without evidence.
- **H-I4-118 remains OPEN / HIGH correctness + release-item-4 evidence reliability.** Exact current `Recovery::on_ack` still rejects only `largest > largest_sent`, then applies time-threshold loss to every remaining old-enough packet without constraining the candidate packet number to the ACK frontier. Thus a legal duplicate/delayed ACK for an already-retired older packet can still time-threshold **newer** in-flight packet numbers and fabricate lost/retransmit/congestion evidence. Finding: `docs/reviews/reviewer-h-i4-118-stale-ack-time-threshold-20260924.md` (`bbf167c9130d913baec42786b404882efc7ceaca`). The semantic owner is unchanged from source anchor `8f48444feb9a8e4fac01c6333ab39873ad46c452`; current developer HEAD only added a downstream positive test.
- Do **not** reopen Candidate A by requiring ACK numbers to remain present in `sent`: H-I4-118 uses an ACK number that was genuinely sent and previously retired. Preserve the existing `largest <= largest_sent` fail-closed high-water check.
- **H-I4-116 is CLOSED** at source/test anchor `4265033b65031d96f106862267391685f86c6bfa`; reachable developer-local provenance: `docs/notes/h-i4-116-provenance-4265033-20260924.md`.
- **H-I4-117 is CLOSED** at source/test anchor `2ba5b960ebb13f1bfe6c5e4b07438385f5aab673`; reachable developer-local provenance: `docs/notes/h-i4-117-provenance-2ba5b96-20260924.md`. Keep developer-local provenance distinct from hosted CI.
- **R-REC-4a is already CLOSED** at `4497752b9a5fc7348d322df9c81dd3659d6d5f3b`: `aggregate_loss_single_reno_reduction` proves exact multi-packet threshold loss (`0..=2`) produces one aggregate Reno reduction; provenance: `docs/notes/r-rec-4a-provenance-4497752-20260924.md`. `1021b74` is extra coverage, not a reason to keep spending slices on R-REC-4a.
- **R-REC-4b is CLOSED** as bounded no-finding at `4497752`: deterministic blackhole accounting retains the expected `delivered = 0` failure shape; review: `docs/reviews/independent-r-rec-4b-fault-simulation-4497752-20260924.md`.
- R-REC-1 (`5bfa9f4a4da9efe9c1e3049c21f75e60f44fdffa`) and R-REC-3 (`2ac22784051d0870bd6645495a6df23394cc97bb`) remain valid for their bounded scopes. R-REC-2's frame-copy/retransmit-ownership no-finding remains useful only for that ownership invariant; it is not coverage of H-I4-118 packet-number ordering.
- H-I4-097..115 remain closed for their challenged owners unless those owners materially move. Candidate A (future/unsent ACK) and Candidate B (mixed datagram-drop reasons) remain closed unless exact-current owners move.
- Release items **3 and 4 remain incomplete**. `RELEASE_CANDIDATE=false`, `PRODUCTION_READY=false`, `FREEZE=false`, `RELEASED=false` remain unchanged.
- **`READY_LIVE: none`.** Do not repeat HY2, warm failover, periodic/soak, package lifecycle, migration-back, endpoint/key migration, IPv6, PLPMTUD or Experimental Track merely for freshness. H-I4-118 is local deterministic correctness work, not a new live question.

## FRONT — H-I4-118 stale duplicate ACK must not time-threshold newer packets

The coding agent should continue immediately; reviewer cadence is not a work-ticket boundary. Do not spend another slice expanding R-REC-4a while this HIGH is open.

1. Re-read exact-current `crates/neko-reliable/src/lib.rs::Recovery::on_ack`, its `time_threshold_and_reno_pacing` / future-ACK tests, downstream `PathRecovery::on_ack`, the existing `ack_retires_bytes_in_flight_exactly_once_and_emits_no_session_evidence` duplicate-ACK control, and `docs/reviews/reviewer-h-i4-118-stale-ack-time-threshold-20260924.md`.
2. Preserve the existing fail-closed check `ack.largest() <= largest_sent`; **do not** require the ACKed packet number to still exist in `sent`. Duplicate/delayed ACKs for genuinely sent and retired packets are legal inputs to this owner.
3. Apply the smallest current-semantics repair so packet/time-threshold loss candidates cannot be packet numbers newer than the ACK's packet-number frontier. If owners are unchanged, an `n <= largest` (equivalently, remaining outstanding candidates strictly older than `largest`) guard around loss eligibility is the expected narrow shape. Do not redesign ACK architecture, retransmit ownership, Reno, Session delivery, crypto or wire semantics.
4. Add a focused deterministic regression that establishes non-zero RTT/loss delay, sends+ACKs an older packet to retire it, sends newer packet numbers and leaves them outstanding, advances virtual `now_us` beyond the loss delay, then delivers a duplicate stale ACK for the retired older packet. Prove: ACK accepted; newly ACKed packets/bytes are zero; lost packets/bytes and retransmit work are zero; RTT and PTO count do not move merely due to staleness; newer packets remain in flight.
5. Keep the positive controls: an actually later ACK may time-threshold an older outstanding packet after the delay; packet-threshold loss remains unchanged; one legitimate positive loss transition still drives one aggregate Reno reduction. `1021b74` may remain as supplemental positive control, but it does not replace the stale-ACK negative.
6. If adding a `PathRecovery` integration regression, prove stale ACK cannot release false lost bytes or trigger Reno reduction, while legitimate loss still does. Do not introduce congestion-policy values.
7. Run focused tests while iterating. On the **final pushed source/test SHA**, run the developer-local clean exact-tree gate in a safe clean checkout/worktree:
   - `PYTHONDONTWRITEBYTECODE=1 bash scripts/check.sh`
   - `git diff --check`
   - verify clean tree.
   Persist reachable provenance with exact SHA, UTC start/end, command exit codes, OS/arch, stable Rust version and clean-tree state. The final exact-tree gate may cover unchanged `1021b74` supplemental test; state that boundary accurately rather than inventing a separate historical pass.
8. No decoder/parser/crypto-framing change is expected; do **not** mechanically run fuzz.
9. After H-I4-118 closure, continue immediately into R-CS-1 and the deep queue below. Do not wait for the next reviewer merely because the focused regression and full gate are green.

## Dependency-ready rolling queue

Keep the deep queue intact; unchanged REUSE surfaces are not duplicated merely to inflate ticket count.

1. **H-I4-118 — stale ACK time-threshold ordering repair/tests/provenance.** FRONT above. HIGH; close before broadening the changed recovery surface.
2. **R-CS-1 — `CarrierState` generation / validation / hysteresis.** Challenge stale/old/future generation rejection, rejection atomicity, validation-vs-packet-feedback separation, dwell/success accumulation/reset, and hysteresis gates against exact-current owner/spec. Explicitly inspect whether dwell evidence is intentionally carrier-global or path-local before classifying unrelated-event accumulation; do not invent a hysteresis semantic or value when current committed docs do not decide it.
3. **R-CS-2 — `CarrierState` single-active / drain / fail / activate.** Challenge active-owner uniqueness, degradation/drain/fail legality, active clearing, terminal/fresh-generation behavior where committed, and active-epoch progression without architecture redesign.
4. **R-CM-DIFF — Concurrent Carrier Manager / health / migration-back owner-diff reuse check.** Reuse prior independent challenge if owners are unchanged; if a relevant owner moved, issue a bounded current-owner challenge rather than mechanically reopening the subsystem.
5. **R-RPKT-1 — release packet factual/evidence reconciliation.** H-I4-116/117 plus R-REC-4 and H-I4-118 make this the next factual reconciliation after the CarrierState cluster or another 3–4 coherent slices. Reconcile exact source anchors, provenance classes, independent-review boundaries and release-item-4 wording. Never imply one SHA ran every historical test. Include the factual boundary that `1021b74` was supplemental test-only coverage unless/until a later exact-tree provenance covers it.
6. **R-PKG/BLD-DIFF — package/reproducibility/operator + dependency/build reuse check.** Reopen only if manifests/features/native hooks/release scripts/unsafe inheritance moved. Signing/SBOM/key-custody/publication remain policy/external gates.
7. **R-OBS/ADAPTER-DIFF — observability + Memory/UDP/TCP adapter current-owner reuse check.** Candidate B stays closed while semantic owners are unchanged; reopen only moved owners.
8. **R-FS/SESSION-DIFF — FairScheduler / multi-stream / SessionRuntime / flow-control current-owner spot challenge.** Reuse prior bounded reviews for unchanged owners; if source moved, challenge the moved accounting/lifecycle/resource seam.
9. **R-CLI/BND-DIFF — CLI exit-code/JSON/human-output + cross-platform process + algorithmic boundedness reconciliation.** Keep Linux evidence distinct from other-OS execution; do not invent timeout/capacity/security values.
10. **R-REC-REFILL — recovery current-owner residual challenge after H-I4-118.** Re-check ACK/loss/RTT/PTO/Reno/fault-simulation review boundaries at the repaired source anchor, with emphasis on whether the fix moved frame ownership or congestion accounting. Prefer a precise no-finding note if unchanged; do not manufacture another recovery checker.
11. **REFILL — repository-wide 13-surface current-owner inventory.** Item 4 remains incomplete: any implemented core owner lacking a reachable dedicated bounded challenge is valid review-support work. Queue exhaustion is legal only after the broad inventory shows no unreviewed current core owner, no concrete defect, no READY review-support lane and no READY live question.
12. **CONDITIONAL LIVE only on a changed question.** Standing VPS authorization remains valid, but current classification is `READY_LIVE: none`.

## Current REUSE map

- **Recovery R-REC-1:** future/unsent ACK + range/state immutability no-finding `5bfa9f4a4da9efe9c1e3049c21f75e60f44fdffa`. H-I4-118 is not a future-ACK regression.
- **Recovery R-REC-2:** frame-copy/retransmit ownership no-finding `791e27d7a307f4360d7876448a5bc5bdb47c457b`; reuse only for its challenged ownership invariant, not as proof that every loss-eligibility predicate is correct.
- **Recovery R-REC-3:** RTT/PTO/threshold-event-counting no-finding `2ac22784051d0870bd6645495a6df23394cc97bb`; scope is lower-level Recovery accounting, not every runtime Reno application seam.
- **Recovery R-REC-4a/4b:** positive aggregate Reno reduction source/test `4497752` plus deterministic fault-simulation no-finding at the same source anchor. `1021b74` is supplemental downstream test coverage only.
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
- A correctness/security/evidence BLOCKER/HIGH stays at the front until closed. **H-I4-118 remains the current HIGH and must stay ahead of the broader CarrierState/reconciliation queue.**