# ChatGPT reviewer handoff — H-I4-120 closed; packet reconciliation FRONT

## Current repository truth

- Synchronize to current `main` before work. Exact code/tests/reachable commits/current specs outrank this handoff, chat memory and stale checkbox state.
- **Exact current product/test anchor:** reachable `4511e4f147eb9511907bb4ec52cad687f113bb7a`. Commits after it through the latest reviewer inventory are documentation/provenance/review navigation only and do not move product/test owners.
- **H-I4-120 is CLOSED.** Reviewer closure: `docs/reviews/reviewer-h-i4-120-closure-4511e4f-20260924.md`. Developer rollback `9e79e6b...` restored the authorized pre-gate production `Recovery::on_ack` baseline; `4511e4f...` made the non-ack-eliciting ACK regression semantics-neutral and corrected the erratum. Reachable exact-tree developer-local provenance is `docs/notes/check-gate-4511e4f-20260924.md` (`scripts/check.sh` exit 0, `git diff --check` exit 0, clean tree, UTC start/end, Linux x86_64, stable rustc 1.98.0).
- GitHub combined status for `4511e4f...` exposes no hosted status entries. Do not relabel developer-local provenance as hosted CI.
- **H-I4-119 remains a MAINTAINER / CORE ACK-PTO SEMANTICS GATE.** Repository truth still does not normatively choose between “any newly acknowledged sent packet resets `pto_count`” and “only newly acknowledged ack-eliciting packets reset.” The current baseline is the last authorized implementation state pending a maintainer/spec decision, not reviewer authority to choose the final rule.
- **Post-H-I4-120 Recovery delta is independently re-challenged with bounded NO-FINDING** in `docs/reviews/independent-r-post-h120-recovery-4511e4f-20260924.md`, explicitly excluding H-I4-119. Existing R-REC-1/2/3, R-REC-4a/4b, H-I4-116/117/118 and `independent-r-rec-residual-68d9382-20260924.md` remain reusable outside that policy seam.
- **Post-H-I4-120 13-surface inventory:** `docs/reviews/independent-core-surface-owner-diff-inventory-4511e4f-20260924.md`. Repository history after the prior `68d9382` inventory moved only `neko-reliable` source/tests plus review/evidence docs; unauthorized production semantics were rolled back. Surfaces 2–12 remain current/reusable; surface 13 (release packet factual consistency/evidence boundary) is the immediate READY_LOCAL lane.
- `independent-security-boundary-diff-5577a2f-20260924.md` and `independent-r-preauth-diff-68d9382-20260924.md` remain reusable because no crypto/trust/pre-auth/Session-security/wire owner moved after their source anchor. D019, RSEC-001/adversarial-load suitability, retained-history capacity, restart/rollback replay safety, signing/key-custody/SBOM/publication, independent security review and release authority remain separate gates.
- Release items **3 and 4 remain incomplete**. `RELEASE_CANDIDATE=false`, `PRODUCTION_READY=false`, `FREEZE=false`, `RELEASED=false` remain unchanged.
- **`READY_LIVE: none`.** Current source/test movement creates no new unresolved real-network question. Do not repeat HY2, warm failover, periodic/soak, package lifecycle, migration-back, endpoint/key migration, IPv6, PLPMTUD or Experimental Track without a materially new code/instrumentation/hypothesis/path condition.

## FRONT — R-RPKT-CURRENT release packet factual reconciliation

This is evidence-index maintenance, not a release-state transition and not permission to choose H-I4-119.

1. Re-read exact-current `docs/release-security-review-packet.md`, `docs/status.md`, the current Session v0 spec, the H-I4-116/117/118 closure/provenance chain, H-I4-119 gate note, H-I4-120 finding + closure/provenance, `independent-r-post-h120-recovery-4511e4f-20260924.md`, CarrierState/manager reviews, current pre-auth/security owner-diff notes, and `independent-core-surface-owner-diff-inventory-4511e4f-20260924.md`.
2. Update only factual evidence-index claims that are stale. At minimum index H-I4-116/117/118, classify H-I4-119 as unresolved maintainer/core ACK-PTO semantics, index H-I4-120 rollback/closure + exact-tree provenance, and point to the current Recovery/CarrierState/manager/security/pre-auth/inventory notes where appropriate.
3. Preserve evidence classes. Never imply one SHA executed all historical tests; developer-local exact-tree provenance, reviewer source/control-flow review, hosted CI, live WAN and performance conclusions remain distinct.
4. Preserve the corrected Session-delivery boundary: bounded independent SessionRuntime/FairScheduler review exists, but there is **no complete Session release/security validation or protocol-freeze approval**.
5. Do not advance release item 3/4 checkboxes or `RELEASE_CANDIDATE`, `PRODUCTION_READY`, `FREEZE`, `RELEASED`.
6. No code/decoder/parser/crypto-framing change is expected; do not run fuzz mechanically.
7. Commit/push the factual packet edit, then immediately continue R-RPKT-CHECK and the rolling queue below without waiting for reviewer cadence.

## Dependency-ready rolling queue

1. **R-RPKT-CURRENT — FRONT.** Perform the bounded factual/evidence-index reconciliation above.
2. **R-RPKT-CHECK — independent packet verification.** Re-read the edited packet against reachable anchors. Challenge factual SHA reachability, evidence-class wording, Session-delivery/security boundaries, policy gates and release flags. If no mismatch: write a scope-precise bounded no-finding. If a concrete factual mismatch remains: smallest docs repair, then re-check.
3. **R-ITEM4-EXTERNAL/POLICY MAP.** Refresh residual item-4 gaps after packet truth is current: H-I4-119/core semantics; D019; retained `SessionRuntime.events` capacity; RSEC-001/adversarial-load/security approval; persistent restart/rollback replay safety; signing/key-custody/SBOM/publication; environment constraints; independent external review; item-3 evidence; RC/freeze/release/production authority. Classify; do not choose values or policies.
4. **R-FINAL-OWNER-HISTORY.** Rerun exact-current thirteen-surface owner history after packet/reconciliation commits. Any actual source/test semantic movement creates a new READY_LOCAL challenge immediately; documentation-only movement does not invalidate unchanged semantic-owner reviews.
5. **R-RECOVERY-CONDITIONAL.** If any coding agent changes `neko-reliable`, immediately re-challenge ACK high-water/future/stale ordering, loss eligibility, RTT/PTO, Reno, persistent congestion, retransmit/frame-copy ownership and deterministic fault simulation, while continuing to exclude H-I4-119 until explicitly decided.
6. **R-CARRIER-CONDITIONAL.** If CarrierState or manager owners move, refill generation/validation/hysteresis/single-active/drain/fail/activate and manager health/migration-back lanes before broader expansion.
7. **R-SESSION/FLOW-CONDITIONAL.** If FairScheduler, multi-stream, SessionRuntime, flow-control or DeliveryAck owners move, refill their dedicated independent challenge rather than relying on stale reuse.
8. **R-OBS/ADAPTER/BUILD/CLI-CONDITIONAL.** If observability, carrier adapters, package/repro/operator scripts, Cargo/build/dependency hooks, CLI output/process or resource-boundedness owners move, refill only the actually moved owners; do not manufacture duplicate reviews.
9. **CONDITIONAL LIVE.** Only if new code/instrumentation/hypothesis/path condition creates a concrete unresolved real-network question within `docs/standing-vps-lab-authorization.md`. Current classification remains `READY_LIVE: none`.
10. **FINAL repository-wide reconciliation before any `queue exhausted` statement.** Queue exhaustion is legal only if broad owner inventory shows no unreviewed/moved implemented core surface, no concrete defect, no READY review-support lane, no READY live question, and every residual item is genuinely policy/external/environment/release-authority gated.

The repository currently does **not** have eight independent unreviewed product-code owners merely because a target queue depth was requested. Do not invent checker/schema/framework/docs churn to hit a number. Keep the conditional refill lanes above armed and expand immediately when repository truth creates real work.

## Standing 13-surface inventory requirement

Every meaningful refill must verify whether each surface still has a dedicated reachable independent bounded review or a materially moved owner:

1. `neko-reliable` UDP recovery — ACK range/future/stale ACK/loss/retransmit/RTT/PTO/persistent congestion/Reno/fault simulation;
2. `neko-carrier` `CarrierState` — generation/validation/hysteresis/single-active/drain/fail/activate;
3. Concurrent Carrier Manager / health / migration-back;
4. FairScheduler / multi-stream / Session + stream flow-control accounting;
5. Memory/UDP/TCP carrier adapter close/error/resource semantics;
6. `SessionRuntime` lifecycle/resource/window/DeliveryAck accounting;
7. `neko-observe` projection/event/counter/high-water correctness;
8. package/reproducibility/operator scripts;
9. dependency/build manifests/lock/features/build/native hooks/unsafe inheritance;
10. cross-platform CLI/process-test semantics;
11. CLI exit-code / JSON / human-output contract;
12. algorithmic resource boundedness without capacity-pressure benchmark or invented policy values;
13. release packet factual consistency/evidence boundary.

A narrow no-finding never means repository-wide queue exhaustion.

## Review -> repair / provenance contract

For every bounded slice: read exact-current owner source/tests + applicable spec/ADR/status claim; state the invariant; try to falsify it with source reasoning and focused deterministic tests. If a concrete defect exists and current committed semantics decide the answer, make the smallest repair + positive/negative regression + commit/push, then run the final pushed developer source SHA through:

```text
PYTHONDONTWRITEBYTECODE=1 bash scripts/check.sh
git diff --check
```

and verify a clean tree. Persist a reachable exact-tree provenance record with exact SHA, exact UTC start/end, exit codes, OS/arch and stable Rust version. Hosted CI is separate cross-evidence and never a wait condition. Wire decoder/parser/crypto-framing changes additionally use the pinned fuzz toolchain and required decode build/run; do not run fuzz mechanically for unrelated code.

If no defect is found, record a scope-precise independent bounded no-finding note naming owners inspected, commands/tests if actually run, exclusions and exact reachable anchor. Do not change code merely to manufacture review churn.

## Evidence discipline / stop conditions

- Developer-reported local CI, persisted developer-local provenance, reviewer source/control-flow review, reviewer-local generic OS checks, hosted CI, live WAN evidence and performance conclusions are distinct classes.
- Accepted exact-tree provenance must anchor a GitHub-resolvable pushed SHA and contain the required exact metadata; never publish local-only/unreachable SHA as shared evidence and never fabricate missing timestamps/toolchain details.
- Never decide D019; TTL/LRU/history/capacity/security values; signing/key-custody/SBOM/publication; previous frozen release policy; core Session/Carrier/ACK/crypto/wire architecture; destructive/canonical migration; RC/freeze/release/production authority.
- A correctness/security/evidence BLOCKER/HIGH becomes FRONT when current semantics determine a repair. If it requires a core semantic/policy choice, keep it as maintainer/spec gate and continue unrelated READY work; do not let an unauthorized implementation silently convert a gate into a decision.
- Normal progression does not require administrator notification. Notify only for unresolved BLOCKER/HIGH requiring maintainer choice, core architecture/destructive migration, policy/value decisions, authorization expansion/new credentials/third-party/production actions, adversarial-load benchmark conditions requiring maintainer choice, or a genuine release-phase transition.
