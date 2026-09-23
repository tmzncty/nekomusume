# ChatGPT reviewer handoff — 13-surface owner diff complete; recovery/CarrierState are current review work

## Current repository truth

- Synchronize to current `main` before work. Code, tests, reachable pushed commits and current specs outrank this handoff, chat memory and stale checkbox state.
- Latest developer-owned source/test anchor in this cycle remains `1f6d744c9954b9a7a5d693b43a20d137a5c5e75d` (`test(cli): H-I4-115 descendant-held pipe bound in bounded_wait_with_output`). Later commits through the owner-diff inventory are reviewer evidence/navigation only.
- **H-I4-097..115 process/socket/thread ownership repair group is closed for its challenged exact source owners.** Current no-finding process-owner sweep, cross-platform reconciliation and algorithmic/resource reconciliation remain reusable unless their named owners move.
- Repository-wide thirteen-surface owner-diff inventory: `docs/reviews/independent-core-surface-owner-diff-inventory-e94ac0f-20260924.md` (created at reviewer commit `6be17732a5effc1b6fbfaceacba130588782af95`).
- Inventory result: surfaces 3–12 are currently **REUSE** for their named semantic owners, with `SessionRuntime.events` retained-history capacity still **MAINTAINER/POLICY**; surfaces 1 (`neko-reliable` recovery) and 2 (`CarrierState`) are **READY_LOCAL** because their direct broad/dedicated challenges predate material owner evolution; surface 13 is **READY_LOCAL** evidence-index maintenance after those current-owner challenges stabilize.
- Candidate A (`Recovery::on_ack` future/unsent ACK) and Candidate B (`record_datagrams` mixed drop reasons) remain closed unless exact-current source/tests materially falsify their repairs. Do not reopen them merely because the broad surface is being re-challenged.
- Release items **3 and 4 remain incomplete**. `RELEASE_CANDIDATE=false`, `PRODUCTION_READY=false`, `FREEZE=false`, `RELEASED=false` remain unchanged.
- **`READY_LIVE: none`.** Do not repeat HY2, warm failover, periodic/soak, package lifecycle, migration-back, endpoint/key migration, IPv6, PLPMTUD or Experimental Track merely for freshness.

## FRONT — R-REC-1 exact-current ACK validity/range/state-immutability challenge

Read exact-current `crates/neko-reliable/src/lib.rs`, its focused tests, applicable Session/recovery spec text, the older reliable-UDP independent review chain, and all reachable repairs after those anchors. Challenge only the current committed semantics; do not redesign ACK architecture.

Required bounded questions:

1. peer ACK/range validity, including future/never-sent ACK and ACK ranges that mix already-purged/known history with impossible packet numbers;
2. invalid ACK must fail closed without changing sent/outstanding state, retransmit ownership, RTT/PTO state, Reno bytes-in-flight/cwnd, persistent-congestion state, or reservation/watermark bookkeeping;
3. valid late/duplicate ACK behavior must remain distinct from impossible future ACK — do not require every valid packet number to remain present in `sent`;
4. quiesce/lifetime-history changes through `62d03a6` must not accidentally make stale/future ACK material mutate fresh recovery ownership.

If source reasoning plus focused deterministic regression finds a concrete defect whose correct result is already fixed by current semantics, make the smallest repair, add positive/negative regression, commit/push, run the final reachable exact-tree developer-local gate and provenance, then continue immediately. If no defect, create one narrowly scoped independent no-finding review anchored to the exact-current owner and move immediately to R-REC-2.

No decoder/parser/crypto framing change is expected; do not run fuzz mechanically.

## Dependency-ready rolling queue

Do not wait for the next reviewer cadence. Keep these lanes moving in order when dependency-ready; unchanged REUSE surfaces are deliberately not duplicated just to inflate queue length.

1. **R-REC-1 — ACK validity/range/future-unsent/state immutability** (FRONT above).
2. **R-REC-2 — loss/retransmit ownership.** Packet/time-threshold loss, retransmission ownership, ACK-range purge/history and late/duplicate interactions.
3. **R-REC-3 — RTT/PTO/persistent congestion.** RTT sample eligibility, PTO deadline/count transition, quiesce interaction and persistent-congestion boundaries; no new timing policy values.
4. **R-REC-4 — Reno + deterministic fault simulation accounting.** bytes-in-flight/cwnd transitions and deterministic fault-model invariants; no capacity-pressure benchmark.
5. **R-CS-1 — CarrierState generation/validation/hysteresis.** Re-read exact-current state owner and reject stale/wrong-generation transitions without changing architecture.
6. **R-CS-2 — CarrierState single-active/drain/fail/activate terminal ordering.** Challenge transition atomicity, single-active and terminality against current specs/ADRs.
7. **R-RPKT-1 — release packet factual/evidence indexing.** Once the current-owner challenges above settle, index still-current H-I4-097..115 closure and 2026-09-24 reconciliations; preserve historical provenance/evidence classes and do not imply one all-tests SHA.
8. **REFILL — repository-wide owner diff after any repair group.** Re-open only surfaces whose semantic owners actually moved. If none moved and all current lanes close, inventory the entire thirteen-surface set again before any queue-exhaustion claim.

Seven substantive lanes plus the conditional refill are the real repository-backed queue at this anchor. There is no justification to manufacture duplicate notes for unchanged surfaces solely to reach 8–15.

## Current REUSE map

- **Concurrent Carrier Manager / health / migration-back:** reuse `5e73aead` health/promotion/switch challenge plus `a2a1e6cf` retained replay/Carrier-generation/migration-back challenge; later carrier empty-record repair is adapter-local.
- **FairScheduler + multi-stream/flow control:** reuse `docs/reviews/reviewer-i4-fs1-fair-scheduler-44a0073-20260921.md` and I4-FS2 closure after H-I4-086/087 (`cd182ade` / `982ee6da`).
- **Carrier adapters:** reuse I4-AD1 MemoryCarrier (`8e905163`), I4-AD2a UDP (`5f24cbff`), I4-AD2b TCP (`e8fbc65e`).
- **SessionRuntime:** reuse lifecycle/terminal review chain plus I4-FS2 coverage of DeliveryAck sent/drained boundaries and aggregate queue cap. Retained-history capacity remains a maintainer/security value gate.
- **Observability:** `neko-observe` owner unchanged since `8e11de0`; reuse independent stable-v1 re-close `18f7c58f`. Candidate B stays closed.
- **Package/reproducibility/operator:** reuse `71de29cf` spot re-challenge; signing/SBOM/key-custody/publication remain external/policy.
- **Dependency/build:** reuse I4-BLD `e11c1d28`; recent process hardening did not change manifests/package scripts/product build owners.
- **Cross-platform CLI/process tests:** reuse `docs/reviews/independent-cli-cross-platform-process-416fd5e-20260924.md`; it is Linux-scope evidence and not Windows/macOS/BSD execution proof.
- **CLI exit/JSON/human output:** reuse existing machine/human contract reviews, including I4-CLI-H `536d59a5`; recent changes are test-harness process ownership, not product CLI output semantics.
- **Algorithmic/resource boundedness:** reuse `docs/reviews/independent-i4-bnd-current-reconciliation-934c878-20260924.md`; do not invent capacity values.
- **Pre-auth rejection/accounting:** owner unchanged across the recorded recent diff; reuse `docs/reviews/independent-preauth-rejection-accounting-d96aabe-20260922.md`. D019/RSEC policy remains separate.

## Evidence discipline

- Developer-reported local CI, repository-persisted developer-local provenance, reviewer source/control-flow review, reviewer-local generic OS checks, hosted CI, live WAN evidence and performance conclusions are distinct evidence classes.
- Accepted exact-tree provenance must anchor a GitHub-resolvable pushed SHA. Never publish local-only/unreachable SHA as shared exact-tree evidence.
- For any new ordinary READY_LOCAL code/test/docs-evidence source repair, final pushed developer SHA must run in a safe clean checkout/worktree with `PYTHONDONTWRITEBYTECODE=1 bash scripts/check.sh`, `git diff --check`, clean-tree confirmation and persisted SHA/UTC times/exit codes/OS+arch/stable Rust state without secrets/private topology.
- Fuzz only if wire decoder/parser/crypto framing materially changes, with pinned `scripts/fuzz-toolchain.sh` and required `decode` build/run commands.
- Standing VPS authorization remains in force but **READY_LIVE: none**; no repeat live evidence without a new concrete unresolved network question.
- Never decide D019, TTL/LRU/history/capacity/security values, signing/key-custody/SBOM/publication policy, core Session/Carrier/ACK/crypto/wire architecture, destructive/canonical migration, RC/freeze/release/production authority.
