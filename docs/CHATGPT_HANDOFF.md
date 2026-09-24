# ChatGPT reviewer handoff — H-I4-118 stale-ACK time-threshold loss FRONT

## Current repository truth

- Synchronize to current `main` before work. Code, tests, reachable pushed commits and current specs outrank this handoff, chat memory and stale checkbox state.
- **H-I4-118 is OPEN / HIGH correctness.** Reviewer exact-source anchor `8f48444feb9a8e4fac01c6333ab39873ad46c452`; finding: `docs/reviews/reviewer-h-i4-118-stale-ack-time-threshold-20260924.md` (reviewer finding commit `bbf167c9130d913baec42786b404882efc7ceaca`). `Recovery::on_ack` correctly rejects never-sent future ACKs but its time-threshold loss arm lacks the packet-number ordering guard applied by the intended loss model: a legal delayed/duplicate ACK for an older, already-retired packet can time-threshold **newer** in-flight packet numbers once they are old enough, fabricating `lost_packets`/retransmit work and downstream congestion evidence. Keep this at FRONT until source/tests/final exact-tree provenance close it.
- **H-I4-116 is CLOSED.** Developer source/test anchor `4265033b65031d96f106862267391685f86c6bfa` keeps `Reno::lost(0)` a no-op and adds the loss-free owner-spanning regression. Reachable `docs/notes/h-i4-116-provenance-4265033-20260924.md` records the developer-local clean exact-tree gate. Do not relabel developer-local provenance as hosted CI.
- **H-I4-117 is CLOSED** at source/test anchor `2ba5b960ebb13f1bfe6c5e4b07438385f5aab673`. `ReliableUdpRuntime::pto_probe` now applies the existing `PathRecovery::persistent_congestion()` effect after a legitimate non-empty-runtime PTO, the threshold checkpoint is executable, and the orchestration test tolerates post-collapse congestion refusal while continuing PTO/health work. Reachable `docs/notes/h-i4-117-provenance-2ba5b96-20260924.md` records `PYTHONDONTWRITEBYTECODE=1 bash scripts/check.sh` exit 0, `git diff --check` exit 0, clean tree, UTC 2026-09-24T01:54:02Z → 01:59:50Z, Linux x86_64, rustc 1.98.0 stable. Reviewer query of exact `2ba5b96` GitHub combined status returned zero status entries; that is not a hosted pass/fail.
- **R-REC-4a is CLOSED** at `4497752`: `aggregate_loss_single_reno_reduction` proves one ACK retiring packet 5 while threshold-declaring 0..=2 lost produces exactly one aggregate Reno reduction. Exact-tree provenance: `docs/notes/r-rec-4a-provenance-4497752-20260924.md`. **R-REC-4b is CLOSED** as no-finding at `4497752`: blackhole `delivered = 0` is the expected failure shape; deterministic simulation accounting remains exact and bounded. Review: `docs/reviews/independent-r-rec-4b-fault-simulation-4497752-20260924.md`.
- R-REC-1 (`5bfa9f4a4da9efe9c1e3049c21f75e60f44fdffa`) and R-REC-3 (`2ac22784051d0870bd6645495a6df23394cc97bb`) remain valid for their bounded scopes. R-REC-2's frame-copy/retransmit-ownership reasoning remains useful, but **do not treat its no-finding as coverage of time-threshold packet-number ordering**; H-I4-118 is a new eligibility seam outside that challenged ownership invariant.
- H-I4-097..115 remain closed for their challenged owners unless those owners materially move. Candidate A (future/unsent ACK) and Candidate B (mixed datagram-drop reasons) remain closed unless exact-current owners move. H-I4-118 does **not** reopen Candidate A: the stale ACK number in H-I4-118 was genuinely sent and previously acknowledged, so requiring all ACK numbers to remain in `sent` would be the wrong fix.
- Release items **3 and 4 remain incomplete**. `RELEASE_CANDIDATE=false`, `PRODUCTION_READY=false`, `FREEZE=false`, `RELEASED=false` remain unchanged.
- **`READY_LIVE: none`.** Do not repeat HY2, warm failover, periodic/soak, package lifecycle, migration-back, endpoint/key migration, IPv6, PLPMTUD or Experimental Track merely for freshness. H-I4-118 is local deterministic correctness work, not a new live question.

## FRONT — H-I4-118 stale duplicate ACK must not time-threshold newer packets

The coding agent should continue immediately; reviewer cadence is not a work-ticket boundary.

1. Re-read exact-current `crates/neko-reliable/src/lib.rs::Recovery::on_ack`, its `time_threshold_and_reno_pacing` / future-ACK tests, downstream `PathRecovery::on_ack`, and `docs/reviews/reviewer-h-i4-118-stale-ack-time-threshold-20260924.md`.
2. Preserve the existing fail-closed check `ack.largest() <= largest_sent`; **do not** require the ACKed packet number to still exist in `sent`. Duplicate/delayed ACKs for genuinely sent and retired packets are legal inputs to this owner.
3. Apply the smallest current-semantics repair so packet/time-threshold loss candidates cannot be packet numbers newer than the ACK's packet-number frontier. An `n <= largest` (equivalently, for remaining outstanding entries, strictly older-than-largest) guard around loss eligibility is the expected shape if exact-current owners are unchanged. Do not redesign ACK architecture, retransmit ownership, Reno, Session delivery, or wire/crypto semantics.
4. Add a focused deterministic regression:
   - establish a non-zero RTT/loss delay;
   - send and ACK an older packet so it is retired;
   - send newer packet numbers and leave them outstanding;
   - advance virtual `now_us` beyond the loss delay;
   - deliver a duplicate stale ACK for the retired older packet;
   - prove the ACK is accepted but newly ACKs nothing, loses nothing, schedules no retransmit, does not perturb RTT/PTO state merely due to staleness, and leaves newer packets in flight.
5. Keep/add the positive control: an actually later acknowledged packet may still time-threshold an older outstanding packet after the delay, and packet-threshold loss remains unchanged. If using `PathRecovery` for an integration regression, prove the stale ACK cannot release false lost bytes or trigger Reno reduction, while legitimate loss still does.
6. Run focused tests while iterating. On the **final pushed source/test SHA**, run the developer-local clean exact-tree gate in a safe clean checkout/worktree:
   - `PYTHONDONTWRITEBYTECODE=1 bash scripts/check.sh`
   - `git diff --check`
   - verify clean tree.
   Persist reachable provenance with exact SHA, UTC start/end, command exit codes, OS/arch, stable Rust version and clean-tree state. Do not record secrets/private topology/credentials/unnecessary absolute paths.
7. No decoder/parser/crypto-framing change is expected; do **not** mechanically run fuzz.
8. After H-I4-118 closure, continue immediately into R-CS-1 and the deep queue below. Do not wait for the next reviewer merely because the focused regression and full gate are green.

## Dependency-ready rolling queue

Keep the deep queue intact; unchanged REUSE surfaces are not duplicated merely to inflate ticket count.

1. **H-I4-118 — stale ACK time-threshold ordering repair/tests/provenance.** FRONT above. HIGH; close before broadening the changed recovery surface.
2. **R-CS-1 — `CarrierState` generation / validation / hysteresis.** Challenge stale/old/future generation rejection, rejection atomicity, validation-vs-packet-feedback separation, dwell/success accumulation/reset, and hysteresis gates against exact-current owner/spec. Explicitly inspect whether dwell evidence is intentionally carrier-global or path-local before classifying unrelated-event accumulation; do not invent a hysteresis semantic or value when current committed docs do not decide it.
3. **R-CS-2 — `CarrierState` single-active / drain / fail / activate.** Challenge active-owner uniqueness, degradation/drain/fail legality, active clearing, terminal/fresh-generation behavior where committed, and active-epoch progression without architecture redesign.
4. **R-CM-DIFF — Concurrent Carrier Manager / health / migration-back owner-diff reuse check.** Reuse prior independent challenge if owners are unchanged; if a relevant owner moved, issue a bounded current-owner challenge rather than mechanically reopening the subsystem.
5. **R-RPKT-1 — release packet factual/evidence reconciliation.** H-I4-116/117 plus R-REC-4 and H-I4-118 make this the next factual reconciliation after the CarrierState cluster or another 3–4 coherent slices. Reconcile exact source anchors, provenance classes, independent-review boundaries and release-item-4 wording. Never imply one SHA ran every historical test.
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
- **Recovery R-REC-4a/4b:** positive aggregate Reno reduction source/test `4497752` plus deterministic fault-simulation no-finding at the same source anchor.
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
- A correctness/security/evidence BLOCKER/HIGH stays at the front until closed. **H-I4-118 is the current HIGH and must stay ahead of the broader CarrierState/reconciliation queue.**
