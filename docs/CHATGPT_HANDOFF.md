# ChatGPT reviewer handoff — H-I4-119 policy gate; packet factual lane FRONT

## Current repository truth

- Synchronize to current `main` before work. Exact code/tests/reachable commits/current specs outrank this handoff, chat memory and stale checkbox state.
- Product/test source anchor for the latest reviewer refill remains `68d938279478b91661f786e830e99af83db457f1`; reviewer-only commits after that anchor do not move product/test owners.
- **H-I4-119 remains a MAINTAINER / CORE ACK-PTO SEMANTICS GATE, not an automatic READY_LOCAL correctness repair.** Exact-current `Recovery::on_ack` resets `pto_count` whenever an ACK newly retires at least one sent packet. Repository docs do not uniquely require the stronger “only newly acknowledged ack-eliciting packets reset” rule. Do not auto-patch this seam until maintainer/spec chooses the intended rule and its relation to simplified health/persistent-congestion evidence. Reconciliation: `docs/reviews/reviewer-h-i4-119-pto-reset-semantics-gate-20260924.md` (`49166d0b9d7faa67a89f472d57dee28e5554b633`).
- **H-I4-116/117/118 remain CLOSED** with reachable developer-local exact-tree provenance. H-I4-097..115 and Candidate A/B remain closed unless their exact semantic owners move.
- **Recovery residual refill CLOSED NO-FINDING** outside H-I4-119: `docs/reviews/independent-r-rec-residual-68d9382-20260924.md` (`4aee288de0cd10143bb34c1a6bb547ccde095636`).
- **Pre-auth owner-diff CLOSED REUSE**: `docs/reviews/independent-r-preauth-diff-68d9382-20260924.md` (`a370aff5d0e6a2c4c72b1a1ec37fea938dc8f122`). D019 and RSEC-001 remain separate gates.
- **Exact-current 13-surface inventory refreshed**: `docs/reviews/independent-core-surface-owner-diff-inventory-68d9382-20260924.md` (`3001156724537c11eb427cb6edd9ddeb11be4a3c`). Every implemented core surface has a current dedicated challenge or exact owner-diff reuse classification at the source anchor; no second materially moved/unreviewed production-code owner was identified. Do not manufacture fake moved-owner lanes to meet a numeric queue target.
- **Grouped release/item-4 factual reconciliation**: `docs/reviews/release-item4-reconciliation-recovery-policy-preauth-3001156-20260924.md` (`28f954850ea882ff508d72624703eb7a5808016c`).
- **R-SECURITY-BOUNDARY-DIFF CLOSED NO-FINDING / REUSE** at `docs/reviews/independent-security-boundary-diff-5577a2f-20260924.md` (`e02edfe43de2ef1cde41b8bb1f0a4640f37a882e`). Exact owner/history inspection found no later semantic movement in the current crypto/pre-auth/security engineering owners that invalidates the dedicated independent reviews. This is source/history review support, not cryptanalysis, adversarial-load evidence or security approval. D019, RSEC-001, `SessionRuntime.events` retained-history capacity, persistent restart/rollback replay safety, signing/key-custody/SBOM/publication and release/security authority remain explicit gates.
- CarrierState R-CS-1/R-CS-2 and manager/health/migration-back remain current no-findings; package/build, observe/adapters, FairScheduler/Session, CLI/process/boundedness owner-diff reuse notes remain current while owners do not move.
- Release items **3 and 4 remain incomplete**. `RELEASE_CANDIDATE=false`, `PRODUCTION_READY=false`, `FREEZE=false`, `RELEASED=false` remain unchanged.
- **`READY_LIVE: none`.** Standing self-owned VPS authorization remains valid, but do not repeat HY2, warm failover, periodic/soak, package lifecycle, migration-back, endpoint/key migration, IPv6, PLPMTUD or Experimental Track absent a materially new unresolved real-network question.

## MAINTAINER / SPEC GATE — H-I4-119

Do not let this gate idle the agent; it blocks only the disputed PTO-reset semantic.

Before any implementation change, maintainer/spec must explicitly choose/document one of the intended rules:

1. **Any newly acknowledged sent packet resets `pto_count`:** retain current behavior; close H-I4-119 as no code defect after suitable semantic documentation/tests if useful.
2. **Only newly acknowledged ack-eliciting packets reset `pto_count`:** explicitly adopt that project-specific rule; only then does the earlier narrow guard become dependency-ready.

Do not infer the choice from Reno byte charging or receiver ACK-obligation behavior; those are distinct existing meanings of `ack_eliciting`. Do not change PTO thresholds, D019, wire/crypto/Session/Carrier architecture, or health policy as a side effect.

## Dependency-ready rolling queue — continue immediately

Recovery, pre-auth, current-owner inventory and security-boundary diff are complete at the anchors above. Continue only real repository work; do not wait for the next reviewer cadence and do not invent filler.

1. **FRONT — R-RPKT-CURRENT: release packet factual/evidence-index maintenance.** Re-read exact-current `docs/release-security-review-packet.md`, `docs/status.md`, implementation-plan flags and latest reachable review/provenance notes. Add/index the latest H-I4-116/117/118 repair/provenance chain, H-I4-119 policy reclassification, current CarrierState/manager reviews, Recovery residual review, pre-auth/security owner-diff reuse, and current-owner inventory where appropriate. Preserve historical evidence classes; never imply one SHA ran all historical tests. The older Session-delivery row currently says “no ... independent review” while later packet rows already index bounded independent SessionRuntime/FairScheduler review: update that wording only to the factual boundary “no complete Session release/security validation or protocol-freeze approval” (or equivalent), without inflating assurance. Do not advance release flags or call H-I4-119 resolved.
2. **R-POST-DIFF-REFILL — repository-wide history refresh.** Immediately after packet maintenance, or sooner if any developer source/test commit lands, rerun the thirteen-surface owner inventory. Any newly moved semantic owner becomes READY_LOCAL and must be challenged under the normal review→repair contract.
3. **R-MOVED-OWNER-1 — conditional.** First real moved/unreviewed implemented core owner found by R-POST-DIFF-REFILL. Read exact source/tests/spec, state an invariant, attempt to falsify, smallest repair only if current semantics decide it. No filler lane if none exists.
4. **R-MOVED-OWNER-2 — conditional.** Second independent real moved/unreviewed owner if present. A precise no-finding is valid item-4 support.
5. **R-RELEASE-RECONCILE — grouped factual reconciliation.** After 3–4 coherent new review/repair slices or any important repair group, reconcile item-4 facts, evidence boundaries and flags again. Do not rewrite every tiny commit into the packet.
6. **R-ITEM4-EXTERNAL/POLICY MAP — only after local lanes close.** Classify what remains as H-I4-119/core semantics, D019, retained-history capacity, RSEC-001/adversarial-load/security approval, persistent restart/rollback replay safety, signing/key-custody/SBOM/publication, environment limits, independent external review, item-3 evidence, or release authority. This is navigation, not permission to choose those values.
7. **CONDITIONAL LIVE — only if a new concrete unresolved path condition appears** and it is within `docs/standing-vps-lab-authorization.md`. Current classification is `READY_LIVE: none`.
8. **FINAL repository-wide reconciliation before any `queue exhausted` statement.** Queue exhaustion is legal only if there is no unreviewed moved core owner, no concrete defect, no READY review-support lane, no READY live question, and every remaining item is genuinely policy/external/environment/release-authority gated.

Eight entries are the real dependency graph at this anchor; several are conditional by design. Do not manufacture checker/schema/framework/docs churn merely to reach 8–15 active slices. Conversely, if a new developer commit moves multiple owners, expand the queue immediately and keep the coding agent working continuously through dependency-ready slices.

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

A narrow no-finding never means repository-wide queue exhaustion. If an owner moves, refill immediately from repository truth.

## Review -> repair / provenance contract

For every bounded slice: read exact-current owner source/tests + applicable spec/ADR/status claim; state the invariant; try to falsify it with source reasoning and focused deterministic tests. If a concrete defect exists and current committed semantics already decide the answer, make the smallest repair + positive/negative regression + commit/push, then run the final pushed developer source SHA through:

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
- A correctness/security/evidence BLOCKER/HIGH becomes FRONT only when current semantics determine a repair. If it requires a core semantic/policy choice, classify it as maintainer/spec gate and continue unrelated READY work.
- Normal progression does not require administrator notification. Notify only for unresolved BLOCKER/HIGH requiring maintainer choice, core architecture/destructive migration, policy/value decisions, authorization expansion/new credentials/third-party/production actions, adversarial-load benchmark conditions requiring maintainer choice, or a genuine release-phase transition.