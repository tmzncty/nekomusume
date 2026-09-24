# ChatGPT reviewer handoff — H-I4-116 owner fix accepted; integration/provenance remain FRONT

## Current repository truth

- Synchronize to current `main` before work. Code, tests, reachable pushed commits and current specs outrank this handoff, chat memory and stale checkbox state.
- Latest developer-owned source/test anchor is `4acc786919a979d38ddd34af75a65e4026e54982` (`fix(reliable): H-I4-116 Reno::lost(0) no-op on loss-free ACK`). Reviewer partial-closure note: `docs/reviews/reviewer-h-i4-116-partial-4acc786-20260924.md`, added by reviewer commit `f2d0e7c5b26644665abc546e303ed432860cc4ed`.
- H-I4-116 is **PARTIALLY REPAIRED, NOT CLOSED**. The production owner fix is accepted: `Reno::lost(0)` now returns before changing `cwnd`, `ssthresh`, or `bytes_in_flight`; the direct zero-loss regression covers all three fields, and the direct non-zero-loss control proves real Reno loss reduction is still enabled. Do not redesign ACK, Reno, persistent-congestion, wire/crypto, or policy values.
- The remaining closure gap is executable integration + provenance, not another production redesign. Exact-current `PathRecovery::on_ack` still aggregates `released_acked` / `released_lost` and invokes `reno.acked(...)` followed by `reno.lost(...)`; the owner guard fixes that path, but there is not yet a focused owner-spanning test that would have failed on the pre-fix zero-loss multiplicative decrease.
- Existing tests are too weak for this regression: `congestion_window_gates_send_and_releases_on_ack` only reopens 1200 bytes, and `runtime_send_admission_refuses_when_cwnd_full_and_recovers_on_ack` only reopens 400 bytes. Both would still pass after the old erroneous half-window reduction.
- Exact `4acc786` had no visible GitHub combined-status entries and no visible workflow runs at reviewer time. No reachable repository-persisted developer-local clean exact-tree provenance for this final source tree was present. Reviewer source/control-flow review is not developer-local CI or hosted CI.
- H-I4-097..115 process/socket/thread ownership group remains closed for its challenged owners unless those owners materially move.
- Recovery challenge progress:
  - R-REC-1 ACK validity/range/future-unsent/state immutability — **NO-FINDING**, reviewer commit `5bfa9f4a4da9efe9c1e3049c21f75e60f44fdffa`.
  - R-REC-2 loss/retransmit ownership — **NO-FINDING**, reviewer commit `791e27d7a307f4360d7876448a5bc5bdb47c457b`.
  - R-REC-3 RTT/PTO/persistent-congestion boundaries — **NO-FINDING**, reviewer commit `2ac22784051d0870bd6645495a6df23394cc97bb`.
  - R-REC-4 Reno + deterministic fault accounting — **BLOCKED ONLY ON H-I4-116 integration/provenance closure**, then continue immediately.
- Candidate A (`Recovery::on_ack` future/unsent ACK) and Candidate B (`record_datagrams` mixed drop reasons) remain closed unless exact-current owner movement falsifies them.
- Release items **3 and 4 remain incomplete**. `RELEASE_CANDIDATE=false`, `PRODUCTION_READY=false`, `FREEZE=false`, `RELEASED=false` remain unchanged.
- **`READY_LIVE: none`.** Do not repeat HY2, warm failover, periodic/soak, package lifecycle, migration-back, endpoint/key migration, IPv6, PLPMTUD or Experimental Track merely for freshness. A new live lane requires a new concrete unresolved real-network question created by code/instrumentation/hypothesis/path-condition movement.

## FRONT — close H-I4-116 without redesign

The coding agent should continue immediately; reviewer cadence is not a work-ticket boundary.

1. Add one focused loss-free owner-spanning regression using existing seams. Preferred `path_recovery_tests` shape:

   ```text
   r = PathRecovery(MSS=1200)
   send one 1200-byte ack-eliciting packet
   ACK exactly that packet, with no induced loss
   assert bytes_in_flight() == 0
   assert can_send(12_000)
   ```

   `12_000` is intentionally discriminating: the correct loss-free path retains/grows the initial 10-MSS window, while the pre-fix `acked(1200)` then `lost(0)` path collapsed to roughly half and would reject this admission. An equivalent `ReliableUdpRuntime` threshold test is acceptable if it proves the same invariant without adding a production debug getter.

2. Preserve/prove the positive aggregate-loss path. Prefer a narrow `PathRecovery` case where packet-threshold loss yields `released_lost > 0` and public admission/accounting evidence shows one Reno reduction for that aggregate outcome. Do not create a second congestion algorithm or new policy number just to count callbacks. The direct `Reno::lost(400)` test is a useful owner-level positive control but does not replace the cross-owner loss-free regression.

3. Push the final source/test SHA and run the first-class developer-local exact-tree gate in a safe clean checkout/worktree:
   - `PYTHONDONTWRITEBYTECODE=1 bash scripts/check.sh`
   - `git diff --check`
   - verify clean tree.

4. Persist reachable provenance for that exact pushed SHA: UTC start/end, command exit codes, OS/arch, stable Rust version, clean-tree state. Do not record secrets, private topology, credentials, or unnecessary absolute paths. No decoder/parser/crypto-framing owner moved in H-I4-116, so do **not** mechanically run fuzz.

5. Immediately continue the R-REC-4 remainder after closure. Do not stop at a green H-I4-116 commit and wait for the next reviewer.

## Dependency-ready rolling queue

Keep the queue deep while real work exists; unchanged REUSE surfaces are not duplicated merely to inflate ticket count.

1. **H-I4-116 integration + exact-tree provenance** — FRONT described above.
2. **R-REC-4 remainder — Reno positive-loss/persistent-congestion integration + deterministic fault simulation accounting.** Challenge exact-current `neko-reliable` / `PathRecovery` / runtime behavior after H-I4-116; a bounded no-finding note is valid if invariants survive source reasoning and focused deterministic tests.
3. **R-CS-1 — `CarrierState` generation / validation / hysteresis.** Challenge stale/old/future generation rejection, state atomicity on rejection, validation-vs-packet-feedback separation, dwell/success reset/accumulation, and hysteresis gate behavior against exact-current owner/spec. Do not invent new hysteresis values.
4. **R-CS-2 — `CarrierState` single-active / drain / fail / activate.** Challenge active-owner uniqueness, degradation/drain/fail transition legality, active clearing, failed-generation terminality where applicable, and active-epoch progression without architecture redesign.
5. **R-CM-DIFF — Concurrent Carrier Manager / health / migration-back owner-diff reuse check.** Reuse prior independent challenge if owners are unchanged; if a relevant owner moved, issue a bounded current-owner challenge rather than mechanically reopening the whole subsystem.
6. **R-RPKT-1 — release packet factual/evidence reconciliation.** After the H-I4-116 group and several coherent review slices, reconcile H-I4-097..116, R-REC current-owner reviews, evidence classes, and release-item-4 boundaries. Never imply one SHA ran every historical test.
7. **R-PKG/BLD-DIFF — package/reproducibility/dependency/build reuse check.** Reopen only if manifests/features/native hooks/release scripts/unsafe inheritance moved; signing/SBOM/key-custody/publication stay policy/external gates.
8. **R-OBS/ADAPTER-DIFF — observability + Memory/UDP/TCP adapter reuse check.** Candidate B stays closed unless semantic owners move. Do not churn stable evidence for an unchanged owner.
9. **R-FS/SESSION-DIFF — FairScheduler / multi-stream / SessionRuntime / flow-control owner-diff spot challenge.** Reuse prior bounded reviews when exact owners are unchanged; if source moved, challenge accounting/lifecycle/resource invariants at the moved seam.
10. **R-CLI-DIFF — CLI exit-code/JSON/human output + process portability owner-diff check.** Reuse prior closure when product/process owners are unchanged; Linux evidence remains distinct from Windows/macOS/BSD execution proof.
11. **REFILL — repository-wide 13-surface current-owner inventory.** Item 4 is still incomplete, so any implemented core owner lacking a reachable dedicated bounded challenge is legitimate review-support work. Queue exhaustion is legal only after this broad inventory shows no unreviewed current core owner, no concrete defect, no READY review-support lane, and no READY live question.
12. **CONDITIONAL LIVE only on changed question.** Standing VPS authorization remains valid, but current classification is `READY_LIVE: none`; create a live slice only if a new code/instrumentation/hypothesis/path condition produces a concrete unresolved real-network question within authorization.

## Current REUSE map

- **Concurrent Carrier Manager / health / migration-back:** reuse `5e73aead` health/promotion/switch challenge plus `a2a1e6cf` retained replay/Carrier-generation/migration-back challenge unless owners move.
- **FairScheduler + multi-stream/flow control:** reuse `docs/reviews/reviewer-i4-fs1-fair-scheduler-44a0073-20260921.md` and I4-FS2 closure after H-I4-086/087 (`cd182ade` / `982ee6da`) unless owners move.
- **Carrier adapters:** reuse I4-AD1 MemoryCarrier (`8e905163`), I4-AD2a UDP (`5f24cbff`), I4-AD2b TCP (`e8fbc65e`).
- **SessionRuntime:** reuse lifecycle/terminal review chain plus I4-FS2 DeliveryAck/queue-cap coverage. Retained-history capacity remains maintainer/security policy.
- **Observability:** reuse independent stable-v1 re-close `18f7c58f` while owner unchanged; Candidate B remains closed.
- **Package/reproducibility/operator:** reuse `71de29cf`; signing/SBOM/key-custody/publication remain external/policy.
- **Dependency/build:** reuse I4-BLD `e11c1d28` unless manifests/features/native hooks/unsafe inheritance move.
- **Cross-platform CLI/process:** reuse `docs/reviews/independent-cli-cross-platform-process-416fd5e-20260924.md`; do not turn Linux-only evidence into other-OS execution claims.
- **CLI output contract:** reuse I4-CLI-H `536d59a5` and current machine/human contract chain unless product output owners move.
- **Algorithmic/resource boundedness:** reuse `docs/reviews/independent-i4-bnd-current-reconciliation-934c878-20260924.md`; do not invent capacity values.
- **Pre-auth rejection/accounting:** reuse `docs/reviews/independent-preauth-rejection-accounting-d96aabe-20260922.md`. D019/RSEC policy remains separate.

## Evidence discipline and stop conditions

- Developer-reported local CI, repository-persisted developer-local provenance, reviewer source/control-flow review, reviewer-local generic OS checks, hosted CI, live WAN evidence and performance conclusions are distinct evidence classes.
- Accepted exact-tree provenance must anchor a GitHub-resolvable pushed SHA; never publish local-only/unreachable SHA as shared evidence.
- Ordinary READY_LOCAL source/test repair ends with the final pushed SHA's developer-local clean exact-tree gate and persisted provenance. GitHub Actions are extra cross-evidence, not a waiting condition.
- Fuzz only for material wire decoder/parser/crypto-framing changes, using pinned `scripts/fuzz-toolchain.sh` with the required decode build/run commands.
- Never decide D019 source-retention/no-reset policy; TTL/LRU/history/capacity/security values; signing/key-custody/SBOM/publication policy; previous frozen release policy; core Session/Carrier/ACK/crypto/wire architecture; destructive/canonical migration; RC/freeze/release/production authority.
- A correctness/security/evidence BLOCKER/HIGH stays at the front until closed; independent unaffected READY local review work remains available once the blocker repair is underway/completed.
