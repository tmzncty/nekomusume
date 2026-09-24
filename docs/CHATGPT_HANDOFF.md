# ChatGPT reviewer handoff — H-I4-120 BLOCKER remains open after partial rollback

## Current repository truth

- Synchronize to current `main` before work. Exact code/tests/reachable commits/current specs outrank this handoff, chat memory and stale checkbox state.
- Current reviewer anchor is `e6bf1e7dfcbdfc1211a9ac6565b37d3c45808470` (`docs(review): reject incomplete H-I4-120 closure`).
- Developer commit `9e79e6b9e2c69be9a99617f7ab0b8415864de679` **did correctly roll production `Recovery::on_ack` back to the authorized pre-H-I4-119 baseline**: `pto_count` resets whenever the newly acknowledged sent-packet set is non-empty. Keep that production rollback unless a maintainer/spec decision explicitly resolves H-I4-119.
- **H-I4-120 remains BLOCKER / core ACK-PTO governance + evidence reliability.** Original finding: `docs/reviews/reviewer-h-i4-120-ack-pto-gate-violation-20260924.md`; exact-current follow-up: `docs/reviews/reviewer-h-i4-120-partial-closure-rejected-20260924.md` (`e6bf1e7d...`). The production seam is back on the authorized baseline, but the required closure is incomplete.
- **H-I4-119 remains a MAINTAINER / CORE ACK-PTO SEMANTICS GATE.** Repository truth still does not choose between “any newly acknowledged sent packet resets `pto_count`” and “only newly acknowledged ack-eliciting packets reset.” No current `docs/decisions.md` / normative spec amendment selects either as the final project rule. Do not infer a decision from comments or commit intent.
- Exact-current `crates/neko-reliable/src/lib.rs` contains `non_ack_eliciting_ack_resets_pto_baseline`, which explicitly asserts `r.pto_count == 0` after only the non-ack-eliciting packet is acknowledged. That still pins one side of the disputed H-I4-119 semantic and violates H-I4-120 closure step 2. The corrected stale-ACK timing may remain, but the regression must be semantics-neutral while H-I4-119 is unresolved.
- The old-note erratum at `f2f33b2384d4e39042ac64162410c26a8129fbc4` correctly marks `5c2c558...` unreachable, but it incorrectly says the unauthorized semantic change was rolled back by `4f53eb817680c531f332e8da9d5e5aff828b93b3`. The actual rollback commit is `9e79e6b...`. Correct/supersede this factual error; do not silently rewrite the historical record.
- The new developer-local provenance note `docs/notes/check-gate-f2f33b2-20260924.md` is **not sufficient accepted closure evidence**. It anchors reachable `f2f33b2...`, records `scripts/check.sh` exit 0, `git diff --check` exit 0, clean tree and OS/arch, but its UTC end is approximate (`~2026-09-24T10:5x:xxZ`) and it omits the stable Rust version required by the repository evidence contract. Rerun the final reachable source/test tree and persist complete exact metadata rather than inventing missing fields.
- GitHub combined status and workflow-run lookup for `f2f33b2...` currently expose no hosted status/run entries. Do not treat developer-local provenance as hosted CI, and do not infer hosted pass/fail from absence of entries.
- Release packet commit `e7d2f43dc5d5f000b42f3bda4fa4288f39cfbbd6` correctly indexed the prior Recovery/CarrierState group and kept H-I4-119 policy-gated, but it predates the H-I4-120 rollback closure attempt. After H-I4-120 is actually closed and the owner inventory is refreshed, reconcile the packet with H-I4-120 facts/provenance; do not advance release flags.
- H-I4-116/117/118 remain CLOSED with their previously reachable developer-local exact-tree provenance. Candidate A/B and H-I4-097..115 remain closed unless their exact semantic owners move.
- Release items **3 and 4 remain incomplete**. `RELEASE_CANDIDATE=false`, `PRODUCTION_READY=false`, `FREEZE=false`, `RELEASED=false` remain unchanged.
- **`READY_LIVE: none`.** Current changes create no new unresolved real-network question. Do not repeat HY2, warm failover, periodic/soak, package lifecycle, migration-back, endpoint/key migration, IPv6, PLPMTUD or Experimental Track without a materially new code/instrumentation/hypothesis/path condition.

## FRONT — finish H-I4-120 closure

This is still a governance/evidence repair, not permission to decide H-I4-119.

1. Re-read exact-current `e6bf1e7d...`, the original H-I4-120 finding, the partial-closure rejection note, the H-I4-119 gate note, and exact-current Recovery tests.
2. **Keep production `Recovery::on_ack` on the restored pre-gate baseline** from `9e79e6b...`. Do not add an ack-eliciting-only guard unless a maintainer/spec decision first lands in repository truth.
3. Rewrite `non_ack_eliciting_ack_resets_pto_baseline` so the corrected timing remains tested **without asserting either disputed `pto_count` outcome**. Preserve semantics-neutral facts such as: packet 1 is legally newly acknowledged; packet 0 remains outstanding under the corrected timing; no stale/time-threshold loss or retransmit occurs merely because packet 1 was acknowledged. Do not rename/comment a policy-selecting assertion and claim it is neutral.
4. Correct or supersede the `check-gate-5c2c558-20260924.md` erratum so it names `9e79e6b...` as the rollback commit. Preserve the historical unreachable-SHA record; do not mutate history to hide the mismatch.
5. On the final **reachable pushed source/test SHA**, run `PYTHONDONTWRITEBYTECODE=1 bash scripts/check.sh`, `git diff --check`, verify clean tree, and persist: exact SHA, exact UTC start, exact UTC end, both exit codes, OS/arch, and stable Rust version. The incomplete `f2f33b2` note is historical partial evidence, not sufficient closure provenance.
6. No decoder/parser/crypto-framing change is involved; do not run decode fuzz mechanically.
7. Commit/push closure, then immediately continue the dependency-ready queue below without waiting for reviewer cadence.

If a maintainer has actually selected an H-I4-119 rule outside repository truth, first encode the explicit decision in the appropriate decision/spec surface, then re-evaluate this contract. Do not infer it from chat or commit intent.

## Dependency-ready rolling queue — continue immediately after H-I4-120 closure

1. **R-POST-H120-RECOVERY — exact-current Recovery owner-delta re-challenge.** Challenge only the closure delta and currently moved owner: ACK validity/high-water, stale/duplicate ACK ordering, loss eligibility, RTT/PTO state, Reno/persistent-congestion integration and deterministic fault simulation. Explicitly exclude the unresolved H-I4-119 rule itself.
2. **R-POST-DIFF-REFILL — repository-wide 13-surface owner inventory refresh.** The prior `68d9382` inventory is stale for `neko-reliable`; recompute exact-current owner movement after the final closure. Any other real semantic owner movement becomes READY_LOCAL.
3. **R-RPKT-CURRENT — release packet factual/evidence-index reconciliation.** Index H-I4-116/117/118, H-I4-119 policy classification, H-I4-120 rollback + corrected provenance, CarrierState/manager reviews, Recovery/pre-auth/security reuse and the refreshed owner inventory. Preserve historical evidence classes; never imply one SHA ran all historical tests. Keep the corrected Session-delivery boundary (“no complete Session release/security validation or protocol-freeze approval” or equivalent). Do not change release flags.
4. **R-MOVED-OWNER-1 — conditional.** First real moved/unreviewed implemented core owner from the refreshed inventory. Exact source/tests/spec -> invariant -> falsification attempt -> smallest repair only when current committed semantics decide the answer.
5. **R-MOVED-OWNER-2 — conditional.** Second independent real moved/unreviewed owner if present. A scope-precise bounded no-finding is valid item-4 support.
6. **R-RELEASE-RECONCILE — grouped factual reconciliation.** After 3–4 coherent slices or any important repair group, reconcile item-4 facts/evidence boundaries. Do not rewrite the packet after every tiny commit.
7. **R-ITEM4-EXTERNAL/POLICY MAP — after local lanes close.** Classify residual gaps as H-I4-119/core semantics, D019, retained-history capacity, RSEC-001/adversarial-load/security approval, persistent restart/rollback replay safety, signing/key-custody/SBOM/publication, environment limits, independent external review, item-3 evidence or release authority. This is navigation, not permission to choose values.
8. **CONDITIONAL LIVE — only for a new concrete unresolved path condition** within standing authorization. Current classification remains `READY_LIVE: none`.
9. **FINAL repository-wide reconciliation before any `queue exhausted` statement.** Queue exhaustion is legal only if broad owner inventory shows no unreviewed core surface, no concrete defect, no READY review-support lane, no READY live question, and every residual item is genuinely policy/external/environment/release-authority gated.

Do not manufacture checker/schema/framework/docs churn to hit a numeric target; expand the queue immediately when the refreshed inventory finds real independent work.

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
- A correctness/security/evidence BLOCKER/HIGH becomes FRONT when current semantics determine a repair. If it requires a core semantic/policy choice, keep it as maintainer/spec gate and continue only unrelated READY work; do not let an unauthorized implementation silently convert a gate into a decision.
- Normal progression does not require administrator notification. Notify only for unresolved BLOCKER/HIGH requiring maintainer choice, core architecture/destructive migration, policy/value decisions, authorization expansion/new credentials/third-party/production actions, adversarial-load benchmark conditions requiring maintainer choice, or a genuine release-phase transition.
