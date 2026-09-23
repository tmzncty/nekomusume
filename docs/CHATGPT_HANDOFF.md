# ChatGPT reviewer handoff — H-I4-116 Reno zero-loss cwnd collapse is FRONT

## Current repository truth

- Synchronize to current `main` before work. Code, tests, reachable pushed commits and current specs outrank this handoff, chat memory and stale checkbox state.
- Latest developer-owned source/test anchor in this cycle remains `1f6d744c9954b9a7a5d693b43a20d137a5c5e75d` (`test(cli): H-I4-115 descendant-held pipe bound in bounded_wait_with_output`). Later commits are reviewer evidence/navigation unless a newer developer source/test commit appears.
- **H-I4-097..115 process/socket/thread ownership repair group is closed for its challenged exact source owners.** H-I4-115 has reachable developer-local exact-tree provenance plus independent bounded review; do not reopen it without material owner movement.
- Repository-wide thirteen-surface owner-diff inventory remains `docs/reviews/independent-core-surface-owner-diff-inventory-e94ac0f-20260924.md` / reviewer commit `6be17732a5effc1b6fbfaceacba130588782af95`.
- Recovery current-owner challenge progress is now:
  - R-REC-1 ACK validity/range/future-unsent/state immutability — **NO-FINDING**, reviewer commit `5bfa9f4a4da9efe9c1e3049c21f75e60f44fdffa`.
  - R-REC-2 loss/retransmit ownership — **NO-FINDING**, reviewer commit `791e27d7a307f4360d7876448a5bc5bdb47c457b`.
  - R-REC-3 RTT/PTO/persistent-congestion boundaries — **NO-FINDING**, reviewer commit `2ac22784051d0870bd6645495a6df23394cc97bb`.
  - R-REC-4 Reno + deterministic fault accounting — **BLOCKED ON H-I4-116 HIGH** below.
- Candidate A (`Recovery::on_ack` future/unsent ACK) and Candidate B (`record_datagrams` mixed drop reasons) remain closed unless exact-current source/tests materially falsify their repairs.
- Release items **3 and 4 remain incomplete**. `RELEASE_CANDIDATE=false`, `PRODUCTION_READY=false`, `FREEZE=false`, `RELEASED=false` remain unchanged.
- **`READY_LIVE: none`.** Do not repeat HY2, warm failover, periodic/soak, package lifecycle, migration-back, endpoint/key migration, IPv6, PLPMTUD or Experimental Track merely for freshness.

## FRONT — H-I4-116 HIGH: loss-free ACK collapses Reno cwnd

Finding: `docs/reviews/finding-h-i4-116-reno-zero-loss-collapse-2ac2278-20260924.md`, created at reviewer commit `e1f4f43462986a9f0c3ec9250b74030019832bdc` from exact source anchor `2ac22784051d0870bd6645495a6df23394cc97bb`.

Exact-current cross-owner defect:

- `crates/neko-carrier/src/lib.rs::PathRecovery::on_ack` always calls `self.reno.lost(released_lost)` after `acked(...)`, even when a successful ACK reports no lost charged bytes and `released_lost == 0`.
- `crates/neko-reliable/src/lib.rs::Reno::lost` unconditionally halves `cwnd` / updates `ssthresh` even for `b == 0`.
- Therefore clean ACK-only traffic can repeatedly trigger multiplicative decrease and drive the congestion window toward the 2-MSS floor despite zero loss.

### Required closure

1. Keep architecture and all congestion numeric policy values unchanged. Make zero lost congestion bytes a no-op for Reno loss reduction; prefer the narrow owner-level invariant `Reno::lost(0)` changes no congestion state unless an equivalent caller guard demonstrably covers every committed caller.
2. Add a direct deterministic regression for `Reno::lost(0)` preserving `cwnd`, `ssthresh`, and `bytes_in_flight`.
3. Add a focused integration regression at `PathRecovery` / `ReliableUdpRuntime`: an ack-eliciting packet is ACKed with no packet loss and the ACK must not cause multiplicative decrease. Do not add a production debug API only to make the test easy; use existing admission/pacing/state seams.
4. Preserve a positive-loss control proving real `released_lost > 0` still reduces Reno exactly once per aggregate loss outcome.
5. Do not change ACK architecture, RTT/PTO semantics, persistent-congestion thresholds, D019/source-retention rules, wire/parser/crypto framing, or introduce new capacity/security values.
6. Commit/push source + tests, then run the final pushed exact-tree developer-local gate in a safe clean checkout/worktree: `PYTHONDONTWRITEBYTECODE=1 bash scripts/check.sh`, `git diff --check`, clean tree. Persist exact reachable SHA, UTC start/end, exit codes, OS+arch, stable Rust, clean-tree state. No fuzz unless framing/decoder code unexpectedly changes.
7. Immediately continue the remainder of R-REC-4 after closure; reviewer cadence is not a work-ticket boundary.

## Dependency-ready rolling queue

Do not wait for the next reviewer cadence. Keep these lanes moving when dependency-ready; unchanged REUSE surfaces are not duplicated solely to inflate queue length.

1. **H-I4-116 — Reno zero-loss multiplicative-decrease repair + focused regressions + exact-tree provenance** (FRONT).
2. **R-REC-4 remainder — Reno positive-loss / persistent-congestion integration + deterministic fault simulation accounting.** Re-read exact-current source after H-I4-116; bounded no-finding is valid if the remaining invariants survive challenge.
3. **R-CS-1 — CarrierState generation/validation/hysteresis.** Challenge stale/wrong-generation transition atomicity and validation/hysteresis separation against exact-current owner/spec.
4. **R-CS-2 — CarrierState single-active/drain/fail/activate ordering.** Challenge single-active and terminal transition semantics without architecture redesign.
5. **R-RPKT-1 — release packet factual/evidence indexing.** Reconcile H-I4-097..116 closure/evidence classes and 2026-09-24 current-owner reviews; never imply one all-tests SHA.
6. **R-PKG/BLD spot reuse check after any source repair.** Only reopen package/build/dependency claims if H-I4-116 or later repair actually moves those owners/manifests/hooks.
7. **R-OBS/ADAPTER reuse check after any source repair.** Only reopen observability/adapters if semantic owners move; Candidate B remains closed otherwise.
8. **REFILL — repository-wide 13-surface owner diff after the repair/review group.** Queue exhaustion is legal only after the broad inventory shows no unreviewed current core owner, concrete defect, READY review-support lane, or READY_LIVE question and all remaining items are genuine policy/external/release-authority gates.

## Current REUSE map

- **Concurrent Carrier Manager / health / migration-back:** reuse `5e73aead` health/promotion/switch challenge plus `a2a1e6cf` retained replay/Carrier-generation/migration-back challenge unless owners move.
- **FairScheduler + multi-stream/flow control:** reuse `docs/reviews/reviewer-i4-fs1-fair-scheduler-44a0073-20260921.md` and I4-FS2 closure after H-I4-086/087 (`cd182ade` / `982ee6da`).
- **Carrier adapters:** reuse I4-AD1 MemoryCarrier (`8e905163`), I4-AD2a UDP (`5f24cbff`), I4-AD2b TCP (`e8fbc65e`).
- **SessionRuntime:** reuse lifecycle/terminal review chain plus I4-FS2 DeliveryAck/queue-cap coverage. Retained-history capacity remains maintainer/security policy.
- **Observability:** owner unchanged since `8e11de0`; reuse independent stable-v1 re-close `18f7c58f`. Candidate B stays closed.
- **Package/reproducibility/operator:** reuse `71de29cf`; signing/SBOM/key-custody/publication remain external/policy.
- **Dependency/build:** reuse I4-BLD `e11c1d28` unless manifests/features/native hooks/unsafe inheritance move.
- **Cross-platform CLI/process tests:** reuse `docs/reviews/independent-cli-cross-platform-process-416fd5e-20260924.md`; Linux-scope evidence is not Windows/macOS/BSD execution proof.
- **CLI exit/JSON/human output:** reuse I4-CLI-H `536d59a5` and current machine/human contract chain unless product output owners move.
- **Algorithmic/resource boundedness:** reuse `docs/reviews/independent-i4-bnd-current-reconciliation-934c878-20260924.md`; do not invent capacity values.
- **Pre-auth rejection/accounting:** reuse `docs/reviews/independent-preauth-rejection-accounting-d96aabe-20260922.md`. D019/RSEC policy stays separate.

## Evidence discipline

- Developer-reported local CI, repository-persisted developer-local provenance, reviewer source/control-flow review, reviewer-local generic OS checks, hosted CI, live WAN evidence and performance conclusions are distinct evidence classes.
- Accepted exact-tree provenance must anchor a GitHub-resolvable pushed SHA; never publish local-only/unreachable SHA as shared evidence.
- Ordinary READY_LOCAL source/test repair must end with the final pushed SHA's developer-local clean exact-tree gate and persisted provenance described above. GitHub Actions are extra cross-evidence, not a waiting condition.
- Fuzz only for material wire decoder/parser/crypto framing changes, using pinned `scripts/fuzz-toolchain.sh` and the required `decode` build/run commands.
- Standing VPS authorization remains in force but **READY_LIVE: none**; no repeat live evidence without a new concrete unresolved real-network question.
- Never decide D019, TTL/LRU/history/capacity/security values, signing/key-custody/SBOM/publication policy, core Session/Carrier/ACK/crypto/wire architecture, destructive/canonical migration, RC/freeze/release/production authority.
