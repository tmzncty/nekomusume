# ChatGPT reviewer handoff — process ownership group closed; exact-current item-4 refill

## Current repository truth

- Synchronize to current `main` before work. Code, tests, reachable pushed commits and current specs outrank this handoff, chat memory and stale checkbox state.
- Latest developer-owned source/test anchor reviewed in this cycle remains `1f6d744c9954b9a7a5d693b43a20d137a5c5e75d` (`test(cli): H-I4-115 descendant-held pipe bound in bounded_wait_with_output`). No later developer-owned implementation/source commit was present when this handoff was refreshed; later reviewer commits are evidence/navigation only.
- **H-I4-097..115 process/socket/thread ownership repair group is closed for its challenged exact source owners.** H-I4-115 exact-tree developer-local provenance is `docs/notes/h-i4-115-provenance-1f6d744-20260924.md`; earlier findings keep their own reachable source/provenance anchors. Do not collapse the chain into an imaginary SHA that ran every historical gate.
- Current independent remaining-owner sweep: `docs/reviews/independent-cli-process-ownership-fcd7a08-20260924.md` — bounded **NO FINDING** for remaining test-harness process/socket/thread ownership. It explicitly does not claim every authenticated product socket lifetime is globally wall-clock bounded.
- Current cross-platform reconciliation: `docs/reviews/independent-cli-cross-platform-process-416fd5e-20260924.md` — **NO correctness finding inside the currently evidenced Linux release/package scope; no Windows/macOS/BSD execution claim.** The old `8e11de0` portability note is historical exact-tree evidence only. Current package/hosted paths are Linux-scoped; `/proc/self/fd` benchmark FD accounting and Unix signal/process regressions must not be promoted to generic cross-platform proof.
- Current algorithmic/resource reconciliation: `docs/reviews/independent-i4-bnd-current-reconciliation-934c878-20260924.md` — **NO NEW dependency-ready boundedness finding.** Compare from exact `5b7b22b` shows no changes to `crates/neko-session/src/lib.rs`, `crates/neko-carrier/src/lib.rs`, or `crates/neko-reliable/src/lib.rs`; post-anchor code changes are CLI process-test hardening. `SessionRuntime.events` retained-history capacity remains `POLICY_BLOCKED_RESOURCE_BOUND`; do not invent a history/capacity number.
- Process-group release/item-4 reconciliation: `docs/reviews/release-item4-process-boundary-reconciliation-59c7f30-20260924.md`. It preserves evidence classes and release flags.
- Exact owner-diff reuse is already established for two queued lanes:
  - `d96aabe..f5c3e9b`: no changes to `crates/neko-cli/src/preauth.rs`, `crates/neko-cli/src/main.rs` or `crates/neko-crypto/src/lib.rs`; prior `docs/reviews/independent-preauth-rejection-accounting-d96aabe-20260922.md` remains reusable for unchanged owners. D019/RSEC policy boundaries remain open.
  - `b4af007..f5c3e9b`: no changes to `crates/neko-cli/src/main.rs`, manifests/package scripts or product CLI owners; changes are `probe.rs`/`multistream.rs` tests plus docs. Existing CLI diagnostic/output and package/build reviews remain reusable where their named owners are unchanged. Do not manufacture duplicate notes merely because HEAD advanced.
- Candidate A (`Recovery::on_ack` future/unsent ACK) and Candidate B (`record_datagrams` mixed drop reasons) remain closed unless materially changed or falsified by exact-current source/tests.
- Release items **3 and 4 remain incomplete**. `RELEASE_CANDIDATE=false`, `PRODUCTION_READY=false`, `FREEZE=false`, `RELEASED=false` remain unchanged.
- **`READY_LIVE: none`.** Do not repeat HY2, warm failover, periodic/soak, package lifecycle, migration-back, endpoint/key migration, IPv6, PLPMTUD or Experimental Track merely for freshness.

## FRONT — repository-wide exact-current owner-diff refill, then challenge changed/uncovered owners

The long CLI process seam is no longer the queue front. Do not keep rediscovering `.output()` variants after the exact-current no-finding sweep unless source materially changes or a concrete counterexample appears.

Build a current 13-surface owner-diff map against the **latest dedicated independent review anchor for each surface**, not merely the oldest 2026-09-13 sweep. For each surface classify:

1. exact-current owner unchanged since latest adequate independent challenge -> **REUSE**, retain prior review/provenance and move on without duplicate doc churn;
2. owner changed only in tests/evidence helpers -> inspect the changed harness seam and reuse production review only if semantics are unchanged;
3. material owner change after latest dedicated challenge -> **READY_LOCAL independent bounded challenge**;
4. known policy/external/environment/release-authority gate -> classify explicitly and continue other independent work;
5. concrete correctness/security/evidence defect whose answer is already fixed by committed semantics -> repair immediately, add focused positive/negative regression, exact-tree local gate/provenance, then continue.

A no-finding challenge is valid item-4 support, but only when it covers a real changed/uncovered owner or a materially different invariant. Do not create checker/schema/framework/docs filler just to count slices.

## Dependency-ready rolling queue after owner-diff classification

Keep multiple lanes ready; execute dependency-ready work continuously without waiting for the next reviewer cadence.

1. **13-surface owner-diff inventory/refill.** Map the latest independent review anchor and current owner-change status for every required surface. This is the navigation step that determines which following lanes are true review work rather than stale reruns.
2. **Reliable UDP recovery current spot challenge if changed/uncovered.** ACK range/future-unsent ACK, loss/retransmit, RTT/PTO, persistent congestion, Reno and fault simulation. Candidate A stays closed absent contrary current evidence.
3. **CarrierState + concurrent Carrier Manager/health/migration-back current spot challenge if changed/uncovered.** Preserve single-active/multi-ready and committed hysteresis/generation semantics; do not redesign architecture.
4. **FairScheduler + multi-stream + session/stream flow-control current spot challenge if changed/uncovered.** Challenge aggregate accounting and error atomicity, not new policy values.
5. **Carrier adapters Memory/UDP/TCP close/error/resource semantics current spot challenge if changed/uncovered.** Avoid repeating already closed empty-message/terminal-ownership slices unless owner code moved.
6. **SessionRuntime lifecycle/window/DeliveryAck current spot challenge if changed/uncovered.** Keep `events` retained-history capacity separate as a maintainer/security value gate; do not use it to block other runtime review.
7. **Observability projection/counter/high-water current spot challenge if changed/uncovered.** Candidate B remains closed unless current source/tests falsify its repair.
8. **CLI exit-code/JSON/human-output + diagnostic evidence boundary reuse/rechallenge.** Product owner source is unchanged since `b4af007`; only challenge newly changed test/harness claims or a new exact-current contradiction. Diagnostics never become authentication, Session Delivery, Path or ACK evidence.
9. **Package/reproducibility/dependency/build reuse/rechallenge.** Current product/manifests/package-script owners are unchanged in the recent process-test interval. Reuse prior reviews unless owner diff finds a material change; do not invent signing/SBOM/key-custody/publication policy.
10. **Pre-auth rejection/resource accounting reuse/rechallenge.** `d96aabe..f5c3e9b` leaves preauth/main/crypto owners unchanged. Reuse prior no-finding; re-open only on source change/new counterexample. D019 and RSEC-001 release/security promotion remain separate gates.
11. **Release packet / item-4 factual indexing.** The packet currently does not index H-I4-097..115 or the 2026-09-24 current-owner reconciliations. Add links only as evidence-index maintenance after the owner-diff map identifies which notes remain current; do not rewrite historical provenance, imply one all-tests SHA, or advance release flags.
12. **Conditional live.** Only if new code/instrumentation/hypothesis/path condition creates a specific unresolved real-network question inside standing authorization. Otherwise keep `READY_LIVE: none`.

If the broad inventory finds that every implemented surface has a reachable dedicated review still valid on its exact-current owner, no concrete defect exists, no READY review-support remains, and all outstanding work is genuinely policy/external/environment/release-authority gated, then and only then may queue exhaustion be considered. Do not infer that from one seam.

## Required 13-surface inventory

For every repository-wide refill, explicitly classify all of these:

1. `neko-reliable` UDP recovery — ACK range, future/unsent ACK, loss/retransmit, RTT/PTO, persistent congestion, Reno, fault simulation;
2. `neko-carrier::CarrierState` — generation, validation, hysteresis, single-active, drain/fail/activate;
3. Concurrent Carrier Manager / health / migration-back;
4. FairScheduler / multi-stream / session+stream flow-control accounting;
5. Memory/UDP/TCP carrier adapter close/error/resource semantics;
6. `SessionRuntime` lifecycle/resource/window/DeliveryAck accounting;
7. `neko-observe` projection/event/counter/high-water correctness;
8. package/reproducibility/operator scripts;
9. Cargo manifests/lock/features/build/native hooks/unsafe inheritance;
10. cross-platform CLI/process-test semantics;
11. CLI exit-code / JSON / human-output contract;
12. algorithmic resource boundedness without capacity-pressure benchmarks or new policy values;
13. release-packet factual consistency and evidence boundary.

## Evidence discipline

- Developer-reported local CI, repository-persisted developer-local provenance, reviewer source/control-flow review, reviewer-local generic OS checks, hosted CI, live WAN evidence and performance conclusions are distinct evidence classes.
- Accepted exact-tree provenance must anchor a GitHub-resolvable pushed SHA. Never publish a local-only/unreachable SHA as shared exact-tree evidence.
- The 2026-09-24 `independent-cli-process-ownership`, cross-platform and boundedness notes are reviewer source/owner-diff reviews; they are **not** reviewer-local Rust/full-gate execution records.
- H-I4-115 exact `1f6d744...` has reachable developer-local exact-tree provenance. Do not infer hosted or reviewer-local execution unless separately observed.
- Exact `46d624e...` H-I4-113/114 has reachable developer-local provenance and separately recorded hosted Rust CI; neither is reviewer-local execution.
- Hosted Actions are cross-evidence, not a wait condition and not a replacement for developer-local clean exact-tree validation.
- For any new ordinary READY_LOCAL code/test/docs-evidence source change, run the final pushed developer SHA in a safe clean checkout/worktree with `PYTHONDONTWRITEBYTECODE=1 bash scripts/check.sh`, `git diff --check`, confirm clean tree, and persist SHA/UTC times/exit codes/OS+arch/stable Rust/clean state without secrets or private topology.
- Fuzz only when wire decoder/parser/crypto framing changes materially, using the pinned `scripts/fuzz-toolchain.sh` toolchain and the required `decode` build/run commands.
- Standing VPS authorization permits bounded self-owned TCP/UDP lab work, but authoritative classification remains `READY_LIVE: none`; no repeat live run without a new concrete question.
- Never decide D019, TTL/LRU/history/capacity/security values, signing/key-custody/SBOM/publication policy, core Session/Carrier/ACK/crypto/wire architecture, destructive/canonical migration, RC/freeze/release/production authority.
