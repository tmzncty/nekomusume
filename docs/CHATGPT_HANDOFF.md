# ChatGPT reviewer handoff — benchmark scope fix accepted; exact-tree closure is queue head

## Reviewed state

- Previous reviewer-owned handoff: exact `3e4677121019cc125bb69345db15c4bea8a3e93a` (`docs(handoff): close comparison schema and bound common-envelope scope`).
- Current developer-owned head reviewed this turn: exact `287e2223d40b7b43dfa2a14ba1992e3bf845a0f2` (`docs: bound benchmark result schema scope`), parented directly on that handoff.
- New developer work since the previous review is one docs-only commit touching `docs/bench/result-schema-v1.md`. It does not change runtime code, Session/Carrier/ACK semantics, wire/crypto framing, package/install behavior, canonical fixtures, or any VPS/WAN evidence.
- GitHub exposes no hosted status records for exact `287e222`. Hosted CI is optional cross-evidence and is not a wait condition.
- No persistent clean exact-tree `scripts/check.sh` provenance for exact `287e222` is present yet. This reviewer performed GitHub source/spec/evidence review only and does not claim reviewer-executed local CI.

## Review verdict

### `287e222` — ACCEPT

The MEDIUM documentation/evidence-contract scope drift identified in the previous handoff is closed at source/documentation level.

The result-schema document now truthfully says that `schema/benchmark-result.v1.json` is the common complete-result envelope for the controlled comparison paths that actually emit `nekomusume.benchmark-result.v1`. It also explicitly preserves the separate existing result contracts:

- deterministic recovery remains `nekomusume.bench.v0` from `scripts/bench/run-isolated.py` / `docs/bench/latest-deterministic.json`;
- privileged netns remains `nekomusume.netns-bench.v0` from `scripts/bench/run-netns.sh` / `docs/bench/latest-netns.json`;
- neither producer nor historical artifact is migrated or reinterpreted merely to satisfy prose.

That is the bounded repair requested by the previous reviewer. No new BLOCKER/HIGH is identified in this review. No fuzz is required for this docs-only correction.

The broader comparison-schema package (`1222270` structural/schema repair + `287e222` scope correction) is **not yet exact-tree locally closed**, because the final developer tree has not yet received persisted clean exact-tree stable-gate provenance. Do not wait for GitHub Actions and do not go into polling/watcher mode.

## READY_LOCAL 1 — exact-tree close the comparison-schema package

This is the immediate queue head and is fully pre-authorized. Unless a real gate failure requires a repair commit, exact `287e222` is the implementation/docs tree to validate.

### Protected boundaries

- preserve the current `nekomusume.benchmark-result.v1` structural contract for generic and owned-lab **complete comparison** results;
- preserve `nekomusume.benchmark-blocked-harness.v1` as a separate exact-key fail-closed blocked-result contract;
- preserve deterministic recovery `nekomusume.bench.v0` and privileged netns `nekomusume.netns-bench.v0` as separate producer/artifact contracts;
- preserve nullable failed-sample evidence where the measurement does not exist rather than fabricating success values;
- preserve specialized owned-lab semantic validation in addition to common JSON-schema structural validation;
- no live rerun, no performance/superiority claim, no historical artifact rewrite.

### Required local closure

On a safe temporary worktree/clone or clean checkout of the exact final developer SHA, run:

1. `bash scripts/bench/compare-hy2-test.sh`;
2. `bash scripts/bench/compare-hy2-owned-lab-test.sh`;
3. `python3 scripts/bench/validate-hy2-owned-lab-test.py`;
4. the direct JSON-schema checks already exercised by those focused tests;
5. `PYTHONDONTWRITEBYTECODE=1 bash scripts/check.sh`;
6. `git diff --check` plus explicit clean-tree verification.

A missing `jsonschema` dependency or any other local gate failure is a real failure. Repair the actual failure and rerun on the replacement exact SHA; absence of hosted CI is not an excuse or blocker.

Persist one concise sanitized provenance package recording exact SHA, commands, UTC start/end, exit codes, OS/arch, Rust stable version, and clean before/after. Do not record credentials, private endpoint topology, or unnecessary host/workdir details. If a later docs-only commit records the provenance, state explicitly that the earlier exact tree was tested; do not recursively claim the provenance commit itself was tested.

After green closure, update only genuinely stale release/item-4 factual navigation to record that the comparison-result schema package is structurally aligned and exact-tree locally gated. Do not promote item 4, RC, freeze, production readiness, or release.

Then continue immediately to REVIEW SUPPORT 2. Do not wait for the next reviewer cadence.

## REVIEW SUPPORT 2 — bounded item-4 evidence challenge

After READY_LOCAL 1, perform one bounded independent-review support pass against exact-current repository truth. This is a targeted challenge pass, not permission to manufacture generic checker/harness cleanup.

Challenge these surfaces and their claim boundaries:

1. **Benchmark evidence contracts** — confirm generic and owned-lab complete comparison producers remain conformant with `benchmark-result.v1`, blocked artifacts remain on their separate schema, and deterministic/netns v0 surfaces are not accidentally treated as v1. Distinguish a blocked HY2 line from a complete paired result and from an actual performance conclusion.
2. **Canonical corpus/vectors** — distinguish corpus-scoped freeze, mechanical validation and developer-authored review from genuinely independent reproduction. Do not turn a frozen corpus into a whole-protocol/release freeze claim.
3. **Package/release engineering** — challenge build cleanliness, archive shape/modes, installed binary identity, package smoke, distinct-version A→B→A evidence, and their exact tested-tree anchors. Do not infer signing, SBOM, publication trust, arbitrary state-schema compatibility, or service-manager hardening unless separately evidenced.
4. **Operator lifecycle/cleanup** — challenge the recorded `READY -> DRAINING -> STOPPED`, listener release/rebind, process cleanup, and negative-path evidence without expanding a bounded observation into sustained/public production reliability.
5. **Session/Carrier evidence** — keep deterministic/loopback correctness, local failover/resume/dedup and bounded WAN observations distinct from long-run reliability, general reachability, interoperability, or performance claims.
6. **Pre-auth resource controls** — distinguish charge-before-work, bounded state/queue/memory/response permits and cleanup from still-open representative adversarial-load/capacity-suitability evidence. Do not self-resolve D019 source-retention policy.
7. **Release gates** — explicitly trace the relationship between still-incomplete item 3, partial item-4 independent review support, D019, RSEC-001, and the final release/security decision.

If this pass finds a concrete correctness/security/evidence defect whose semantic answer is already determined by current repository contracts, it immediately becomes the new READY_LOCAL head: smallest repair -> bounded positive/negative regression -> commit/push -> clean exact-tree local gate -> concise provenance -> factual reconciliation -> continue. Do not wait for an hourly reviewer refresh.

If no concrete repairable defect remains, write only the bounded factual checkpoint needed for navigation and continue to CHECKPOINT 3. Developer/agent-prepared support cannot self-promote into final independent security/release approval.

## REVIEW CHECKPOINT 3 — classify the remaining release obstacles

After READY_LOCAL 1 and REVIEW SUPPORT 2, classify each remaining release obstacle into exactly one bucket:

- concrete code/evidence defect repairable under existing semantics;
- independent review depth still missing;
- D019 policy/value decision;
- adversarial-load/capacity-suitability evidence whose conditions or authority are not already fixed by repository truth;
- still-incomplete release item 3 / environment evidence;
- production/release decision outside current authorization.

Any concrete repairable defect is pre-authorized for immediate smallest-fix closure. If no such dependency-ready coding work remains, the coding queue is genuinely exhausted: stop expanding implementation/checker/harness work rather than entering watcher mode or fabricating backlog. The next real work is independent review, policy, environment evidence, or release decision according to the classification.

## Exact-current release/live boundary

Repository truth remains unchanged by `287e222`:

- `IMPLEMENTATION_COMPLETE=true` only in the repository's bounded research/governance sense;
- release item 3 remains incomplete;
- release item 4 remains incomplete and has bounded independent-review support only;
- `RELEASE_CANDIDATE=false`;
- `PRODUCTION_READY=false`;
- `FREEZE=false`;
- `RELEASED=false`;
- D019 remains `SOURCE_RETENTION_POLICY_BLOCKED`;
- RSEC-001 still lacks representative adversarial-load/capacity-suitability evidence and the independent release/security decision required for promotion;
- current live opportunity remains `READY_LIVE: none`.

The HY2 current line remains frozen at the retained `13da094` negative evidence: no complete pair, no median/P95 comparison, no superiority conclusion, and no same-class retry without a materially changed hypothesis/instrumentation/protocol state. Standing VPS authorization remains valid but does not manufacture a new live question.

## Honest rolling queue

The current queue is intentionally short because repository truth does not justify padding it to an arbitrary duration:

1. **READY_LOCAL:** clean exact-tree focused + stable local gate for `287e222` (or the replacement SHA if a real failure is repaired), then one concise provenance/factual reconciliation package.
2. **REVIEW SUPPORT:** bounded item-4 evidence challenge across the explicitly named surfaces above.
3. **CHECKPOINT:** classify every remaining item-3/item-4/D019/RSEC-001/independent-review/release-decision obstacle.
4. **CONDITIONAL READY_LOCAL:** any concrete defect found in 2–3 with an existing semantic answer is pre-authorized for smallest repair + regressions + exact-tree local closure.
5. **REAL STOP/ESCALATION:** D019 policy choice; destructive/canonical-meaning migration; core Session/Carrier/ACK/crypto/wire architecture change; benchmark/adversarial-load conditions requiring maintainer value judgment; action outside standing authorization; production impact; new credentials/server/third-party permission; major security issue not safely adjudicable under existing semantics; real repository breakage; runtime/tool-budget exhaustion; or genuine queue exhaustion after the explicit slices are resolved.

While READY_LOCAL or a later concrete repair remains, external coding work should continue implementation/test/commit/push without waiting for reviewer cadence. If only policy/review/environment/release-decision buckets remain, stop coding expansion rather than poll.

## Live/VPS boundary

`READY_LIVE: none` remains authoritative. No new real-network question was produced by this review.

Do not repeat unchanged HY2, repeated warm failover, periodic/soak, package lifecycle, migration-back, endpoint/source migration, key update, IPv6, PLPMTUD, or Experimental Track runs merely because the VPS rental window remains open. A future materially changed, dependency-ready self-owned TCP/UDP question may use standing authorization directly, but this handoff creates none.
