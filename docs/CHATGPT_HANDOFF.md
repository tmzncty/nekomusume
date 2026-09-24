# ChatGPT reviewer handoff — H-I4-120 BLOCKER; H-I4-119 ACK/PTO gate violated on current main

## Current repository truth

- Synchronize to current `main` before work. Exact code/tests/reachable commits/current specs outrank this handoff, chat memory and stale checkbox state.
- **Exact-current source/test commit `8d0a6e3a2824cd8338fff5a0cdb995107f0e114e` moved `crates/neko-reliable/src/lib.rs` after the last authorized reviewer source anchor `68d938279478b91661f786e830e99af83db457f1`.** It is not a test-only move: its diff changes production `Recovery::on_ack` PTO-reset semantics.
- **H-I4-120 is BLOCKER / core ACK-PTO semantics + evidence reliability.** Reviewer finding: `docs/reviews/reviewer-h-i4-120-ack-pto-gate-violation-20260924.md` (`2065b36c5cc4f870ffa399b4e563babc60f7f3a6`). Reachable `8d0a6e3...` implements the stronger H-I4-119 rule (`pto_count` resets only when a newly acknowledged packet was ack-eliciting) even though the current repository gate explicitly forbids auto-patching that seam until maintainer/spec chooses the intended rule.
- **H-I4-119 remains a MAINTAINER / CORE ACK-PTO SEMANTICS GATE.** The repository still does not uniquely decide between “any newly acknowledged sent packet resets” and “only newly acknowledged ack-eliciting packets reset.” No `docs/decisions.md` / normative spec amendment selecting the latter was found. Reconciliation remains `docs/reviews/reviewer-h-i4-119-pto-reset-semantics-gate-20260924.md` (`49166d0b9d7faa67a89f472d57dee28e5554b633`).
- **Current provenance is not acceptable shared exact-tree evidence.** `docs/notes/check-gate-5c2c558-20260924.md` (reachable provenance commit `4f53eb817680c531f332e8da9d5e5aff828b93b3`) anchors `5c2c558fab6521bfb1ebb3a89c0f49ae95604c47`, which is not GitHub-resolvable from this repository, and describes the change as test-only. The reachable `8d0a6e3...` diff includes the gated production semantic change. Do not use the existing provenance as accepted current-tree evidence; supersede/errata it rather than silently rewriting history.
- **H-I4-116/117/118 remain CLOSED** with their previously reachable developer-local exact-tree provenance; Candidate A/B and H-I4-097..115 remain closed unless their exact semantic owners move.
- Recovery residual, pre-auth reuse, CarrierState R-CS-1/R-CS-2, manager/health/migration-back, package/build, observe/adapters, FairScheduler/Session, CLI/process/boundedness and security-boundary reuse notes remain valid only for their stated earlier owner anchors. Because `neko-reliable` moved at `8d0a6e3...`, the prior repository-wide 13-surface inventory is no longer exact-current for surface 1 and must be refreshed after the BLOCKER is closed.
- Release items **3 and 4 remain incomplete**. `RELEASE_CANDIDATE=false`, `PRODUCTION_READY=false`, `FREEZE=false`, `RELEASED=false` remain unchanged.
- **`READY_LIVE: none`.** The current source movement creates no new real-network question. Do not repeat HY2, warm failover, periodic/soak, package lifecycle, migration-back, endpoint/key migration, IPv6, PLPMTUD or Experimental Track absent a materially new unresolved path condition.

## FRONT — H-I4-120 closure contract

This is a governance rollback, not a maintainer semantic choice. Until a maintainer/spec decision is committed, the permitted repository state is the last authorized pre-H-I4-119 behavior.

1. Re-read exact-current `8d0a6e3...`, the H-I4-119 gate note, this finding and applicable Recovery tests.
2. **Restore the production PTO-reset seam to the pre-gate baseline** present before the unauthorized semantic change: `pto_count` resets when the newly-acked sent-packet set is non-empty. Do not describe this rollback as permanently selecting that rule; it only restores the authorized baseline while H-I4-119 remains unresolved.
3. Remove or rewrite the policy-selecting `non_ack_eliciting_ack_does_not_reset_pto` assertion. The stale-ACK timing improvement may remain only as a semantics-neutral regression (for example, proving the older ack-eliciting packet remains outstanding under the corrected timings) and must not decide the reset rule.
4. Do not change PTO thresholds, persistent-congestion policy, D019, Session/Carrier architecture, wire/crypto semantics or any release/governance flag.
5. Add an explicit erratum/superseding note for `docs/notes/check-gate-5c2c558-20260924.md`: its recorded SHA is unreachable and its “test-only” scope claim is not valid for the reachable current change. Do not mutate the historical note in place to hide the mismatch.
6. On the final **reachable pushed** source/test SHA, run `PYTHONDONTWRITEBYTECODE=1 bash scripts/check.sh`, `git diff --check`, verify clean tree, and persist SHA / UTC start+end / exit codes / OS-arch / stable Rust version. Hosted CI is separate cross-evidence and not a wait condition. No decoder/framing change exists, so do not run fuzz mechanically.
7. Commit/push closure, then immediately continue the queue below without waiting for reviewer cadence.

If a maintainer has actually selected an H-I4-119 rule outside repository truth, do **not** infer it from chat or commit intent: first encode the explicit decision in the appropriate decision/spec surface, then re-evaluate this closure contract.

## Dependency-ready rolling queue — continue after H-I4-120 closure

Do not expand implementation on top of the unresolved core-semantics violation. Once the rollback/provenance closure is reachable, continue continuously through real work:

1. **R-POST-H120-RECOVERY — exact-current Recovery re-challenge.** Challenge only the owner delta introduced by the rollback/timing-regression closure: ACK validity/high-water, stale/duplicate ACK ordering, loss eligibility, RTT/PTO state, Reno/persistent-congestion integration and deterministic fault simulation. H-I4-119 itself remains excluded as policy-gated.
2. **R-POST-DIFF-REFILL — repository-wide 13-surface owner inventory refresh.** The prior `68d9382` inventory is stale for `neko-reliable`. Recompute exact-current moved-owner classification after closure; any other real semantic owner movement becomes READY_LOCAL.
3. **R-RPKT-CURRENT — release packet factual/evidence-index maintenance.** Re-read exact-current `docs/release-security-review-packet.md`, `docs/status.md`, implementation-plan flags and reachable review/provenance notes. Index H-I4-116/117/118, H-I4-119 policy classification, H-I4-120 rollback/evidence correction, CarrierState/manager reviews, Recovery/pre-auth/security reuse and the refreshed owner inventory. Preserve historical evidence classes; never imply one SHA ran all historical tests. Reconcile the older Session-delivery wording only to “no complete Session release/security validation or protocol-freeze approval” (or equivalent), not broader assurance. Do not advance release flags.
4. **R-MOVED-OWNER-1 — conditional.** First real moved/unreviewed implemented core owner found by the refreshed inventory. Exact source/tests/spec -> invariant -> falsification attempt -> smallest repair only if current semantics decide it. No filler if none exists.
5. **R-MOVED-OWNER-2 — conditional.** Second independent real moved/unreviewed owner if present; a precise bounded no-finding is valid item-4 support.
6. **R-RELEASE-RECONCILE — grouped factual reconciliation.** After 3–4 coherent review/repair slices or any important repair group, reconcile item-4 facts/evidence boundaries again. Do not rewrite every tiny commit into the packet.
7. **R-ITEM4-EXTERNAL/POLICY MAP — after local lanes close.** Classify residual gaps as H-I4-119/core semantics, D019, retained-history capacity, RSEC-001/adversarial-load/security approval, persistent restart/rollback replay safety, signing/key-custody/SBOM/publication, environment limits, independent external review, item-3 evidence or release authority. This is navigation, not permission to choose values.
8. **CONDITIONAL LIVE — only if a new concrete unresolved path condition appears** and is within standing authorization. Current classification is `READY_LIVE: none`.
9. **FINAL repository-wide reconciliation before any `queue exhausted` statement.** Queue exhaustion is legal only if there is no unreviewed moved core owner, no concrete defect, no READY review-support lane, no READY live question, and every remaining item is genuinely policy/external/environment/release-authority gated.

Nine entries are the real dependency graph now. Do not manufacture checker/schema/framework/docs churn merely to reach a numeric target; conversely, expand immediately if the refreshed inventory finds multiple real moved owners.

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

and verify a clean tree. Persist a reachable exact-tree provenance record with SHA, UTC start/end, exit codes, OS/arch and stable Rust version. Hosted CI is separate cross-evidence and never a wait condition. Wire decoder/parser/crypto-framing changes additionally use the pinned fuzz toolchain and required decode build/run; do not run fuzz mechanically for unrelated code.

If no defect is found, record a scope-precise independent bounded no-finding note naming owners inspected, commands/tests if actually run, exclusions and exact reachable anchor. Do not change code merely to manufacture review churn.

## Evidence discipline / stop conditions

- Developer-reported local CI, persisted developer-local provenance, reviewer source/control-flow review, reviewer-local generic OS checks, hosted CI, live WAN evidence and performance conclusions are distinct classes.
- Accepted exact-tree provenance must anchor a GitHub-resolvable pushed SHA; never publish local-only/unreachable SHA as shared evidence.
- Never decide D019; TTL/LRU/history/capacity/security values; signing/key-custody/SBOM/publication; previous frozen release policy; core Session/Carrier/ACK/crypto/wire architecture; destructive/canonical migration; RC/freeze/release/production authority.
- A correctness/security/evidence BLOCKER/HIGH becomes FRONT when current semantics determine a repair. If it requires a core semantic/policy choice, keep it as maintainer/spec gate and continue only unrelated READY work; do not let an unauthorized implementation silently convert a gate into a decision.
- Normal progression does not require administrator notification. Notify only for unresolved BLOCKER/HIGH requiring maintainer choice, core architecture/destructive migration, policy/value decisions, authorization expansion/new credentials/third-party/production actions, adversarial-load benchmark conditions requiring maintainer choice, or a genuine release-phase transition.
