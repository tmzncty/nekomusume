# ChatGPT reviewer handoff — comparison schema repair accepted; narrow common-envelope scope before final gate

## Reviewed state

- Previous reviewer-owned handoff: exact `a97f9e052cc180407b215f53e4f22a194c22af63` (`docs(handoff): reconcile benchmark-result v1 contract`).
- Current developer-owned head reviewed this turn: exact `1222270870763f5a5f053a1ded80266e2ca669a0` (`fix: align benchmark result schemas`), parented directly on that handoff.
- This is one coherent implementation/test/docs commit. It changes the two comparison-result contracts/tests and generic comparison producer metadata; it does **not** change Session/Carrier/ACK/wire/crypto framing, package/install behavior, canonical fixtures, or perform any VPS/WAN experiment.
- GitHub exposes no hosted status records for exact `1222270`. Hosted CI is optional cross-evidence and is not a wait condition.
- No persistent clean exact-tree `scripts/check.sh` provenance for exact `1222270` is present yet. Focused tests reported by the developer are useful but do not substitute for the required final local gate.
- This ChatGPT reviewer performed GitHub source/spec/evidence review only. No reviewer-executed local CI is claimed in this turn.

## Review verdict

### `1222270` — ACCEPT_WITH_BOUNDS, not yet final closure

The original comparison-path `nekomusume.benchmark-result.v1` mismatch is substantially repaired:

- the common JSON schema now accepts the established **array** summary shape used by the comparison producers;
- failed-sample evidence such as unavailable application-byte values may remain `null` rather than being fabricated;
- owned-lab complete results can carry bounded `cleanup_evidence` while top-level unknown fields remain fail-closed;
- `bounds` is no longer universally required, and the documentation now states that it is present only when a producer has a truthful producer-enforced whole-run bound;
- the generic sequential comparator derives `git_commit` from the repository root rather than ambient caller cwd;
- generic success/failure outputs are checked against `schema/benchmark-result.v1.json` in the comparison test;
- owned-lab validator tests exercise common-schema conformance plus malformed summary/bounds/cleanup and unknown top-level negatives;
- the separately versioned `nekomusume.benchmark-blocked-harness.v1` contract remains fail-closed and is not merged into the complete-result schema.

This closes the **comparison-producer structural mismatch** at source/test level. It does not yet close the package at exact-tree local-CI level because final `scripts/check.sh` provenance has not been persisted.

Do not wait for GitHub Actions. Do not create a standalone provenance commit for exact `1222270` before fixing the new documentation-scope drift below; produce one replacement developer tree and gate that exact tree once.

## New finding — MEDIUM documentation/evidence-contract scope drift

The edited `docs/bench/result-schema-v1.md` currently states that `schema/benchmark-result.v1.json` is the common result envelope for **deterministic, netns, VPS, and later comparison experiments**. Exact-current repository truth does not support that broad claim.

Concrete evidence:

- `scripts/bench/run-isolated.py` still emits `schema: "nekomusume.bench.v0"`, with its own deterministic-recovery fixture shape and object summary;
- committed `docs/bench/latest-deterministic.json` is the corresponding `nekomusume.bench.v0` artifact;
- `scripts/bench/run-netns.sh` still emits `schema: "nekomusume.netns-bench.v0"`, with its own isolated-netns shape and object summary;
- committed `docs/bench/latest-netns.json` is the corresponding `nekomusume.netns-bench.v0` artifact;
- neither producer was changed by `1222270`.

Therefore `benchmark-result.v1` is demonstrably the aligned common envelope for the **current complete comparison paths**, but it is not currently the envelope emitted by the deterministic or netns harnesses. Calling it common to those producer families creates release/evidence navigation drift even though their existing artifacts remain truthful under their own schema identifiers.

Severity is **MEDIUM evidence/documentation correctness**, not security HIGH: no existing deterministic/netns measurement is invalidated and no live or performance conclusion changes. The problem is the newly strengthened contract claim, not the historical artifacts.

### Minimal accepted repair

Prefer the smallest truthful correction:

- narrow `docs/bench/result-schema-v1.md` to say that `benchmark-result.v1` is the common complete-result envelope for the **controlled comparison paths that actually emit that identifier**;
- explicitly state that the deterministic recovery fixture remains `nekomusume.bench.v0` and the privileged netns fixture remains `nekomusume.netns-bench.v0`, with their committed artifacts/producers separate;
- do **not** migrate or rewrite deterministic/netns producers/artifacts merely to make the prose true;
- do not invent a migration/versioning framework or reinterpret historical measurements.

A broader producer migration is **DEFER** unless an existing release/spec requirement independently demands it. This review finds no such requirement that justifies changing canonical meanings now.

## READY_LOCAL 1 — correct scope, then exact-tree close `1222270` package

This is the immediate queue head and is fully pre-authorized.

### Files/concepts

- `docs/bench/result-schema-v1.md`;
- comparison schema/tests already changed by `1222270`;
- deterministic/netns producer identifiers only as regression/navigation anchors, not migration targets.

### Protected boundaries

- keep `nekomusume.benchmark-result.v1` structural conformance for generic and owned-lab complete comparison results;
- keep blocked-harness results on their separate exact-key schema;
- preserve failed-sample truthfulness and specialized owned-lab semantic validation;
- preserve deterministic/netns v0 producer/artifact meanings;
- no WAN run, no benchmark rerun, no performance/superiority claim.

### Verification and closure

After the small documentation correction, create a coherent replacement developer SHA and run on that **clean exact tree**:

1. `bash scripts/bench/compare-hy2-test.sh`;
2. `bash scripts/bench/compare-hy2-owned-lab-test.sh`;
3. `python3 scripts/bench/validate-hy2-owned-lab-test.py`;
4. any direct JSON-schema checks already used by the focused tests;
5. `PYTHONDONTWRITEBYTECODE=1 bash scripts/check.sh`;
6. `git diff --check` and explicit clean-tree verification.

Record minimum sanitized provenance: exact SHA, commands, UTC start/end, exit codes, host OS/arch, Rust stable version, clean before/after. If local `scripts/check.sh` fails for any reason—including a newly introduced test dependency that is not available in the actual gate environment—that is a real failure: repair it and rerun rather than citing absent hosted CI.

After green closure, persist one concise provenance/factual package and update only genuinely stale release/item-4 navigation. A later provenance-only docs commit may record the tested tree without recursively claiming that the provenance commit itself was tested.

Then continue immediately to REVIEW SUPPORT 2. Do not enter watcher mode.

## REVIEW SUPPORT 2 — bounded item-4 evidence challenge

After READY_LOCAL 1, perform one bounded independent-review support pass against exact-current repository truth. This is review/support work, not permission to manufacture generic checker/harness cleanup.

Challenge at least:

- comparison `benchmark-result.v1` producer/schema conformance after the repair and the now-explicit separation from deterministic/netns v0 artifacts;
- canonical corpus/vector review: what is frozen, mechanically validated, and independently reproduced versus merely developer-authored evidence;
- package build/install/upgrade/rollback/archive claims and their tested-tree anchors;
- operator `READY -> DRAINING -> STOPPED`, process/listener cleanup and evidence boundaries;
- Session/Carrier bounded local evidence versus WAN/long-run/performance claims;
- non-policy pre-auth resource/abuse controls versus still-open adversarial-load capacity/suitability;
- HY2 blocked evidence versus a complete paired result versus an actual performance conclusion;
- D019 source-retention policy boundary;
- relationship to still-incomplete release item 3.

If this pass finds a concrete correctness/security/evidence defect with an existing semantic answer, it becomes the immediate READY_LOCAL head: smallest repair -> bounded positive/negative tests -> commit/push -> exact-tree local gate -> provenance -> factual reconciliation -> continue. Do not wait for the next hourly reviewer refresh.

If no concrete repairable defect remains, write only the bounded factual checkpoint needed for navigation and proceed to CHECKPOINT 3. Developer/agent-prepared support cannot self-promote into final independent security/release approval.

## REVIEW CHECKPOINT 3 — classify remaining release gates

After READY_LOCAL 1 and REVIEW SUPPORT 2, classify every remaining release obstacle as exactly one of:

- code/evidence defect still repairable under existing semantics;
- independent review depth still missing;
- D019 policy/value decision;
- adversarial-load/capacity-suitability evidence requiring conditions/authority not already fixed by repository truth;
- still-incomplete release item 3 / environment evidence;
- production/release decision outside current authorization.

Do **not** mark item 4, RC, freeze, production readiness, or release complete merely because local deterministic checks and partial independent reviews are green. If no dependency-ready coding work remains after this classification, the coding queue is genuinely exhausted; stop expansion instead of polling or fabricating work.

## Exact-current release/live boundary

Repository truth remains:

- `IMPLEMENTATION_COMPLETE=true` only in the repository's bounded research/governance sense;
- release item 3 remains incomplete;
- release item 4 remains incomplete and has only bounded independent review support so far;
- `RELEASE_CANDIDATE=false`;
- `PRODUCTION_READY=false`;
- `FREEZE=false`;
- `RELEASED=false`;
- D019 remains `SOURCE_RETENTION_POLICY_BLOCKED`;
- RSEC-001 still lacks representative adversarial-load/capacity-suitability evidence and the independent release/security decision required for promotion;
- current live opportunity remains `READY_LIVE: none`.

The HY2 current line remains frozen at exact `13da094`: no complete pair, no median/P95 comparison, no superiority result, and no same-class live retry without a materially new hypothesis. Standing VPS authorization remains valid but creates no new live question by itself.

## Honest rolling queue

The queue is intentionally evidence-driven rather than padded to a nominal duration:

1. **READY_LOCAL:** narrow the over-broad benchmark-result-v1 documentation scope while retaining the `1222270` comparison-contract repair; run focused + full clean exact-tree local gate on the replacement developer SHA; persist provenance/factual reconciliation.
2. **REVIEW SUPPORT:** one bounded item-4 evidence challenge against exact-current source/spec/artifacts after that closure.
3. **CHECKPOINT:** classify remaining item-3/item-4/D019/RSEC-001/independent-review/release-decision gates.
4. **CONDITIONAL READY_LOCAL:** any concrete defect found in 2–3 with an existing semantic answer is pre-authorized for smallest fix + tests + exact-tree local closure.
5. **REAL STOP/ESCALATION:** D019 policy choice; destructive/canonical-meaning migration; adversarial-load/benchmark conditions requiring maintainer value judgment or exceeding standing authorization; production impact; new credentials/server/third-party permission; core Session/Carrier/ACK/crypto/wire architecture change; or a major security issue not safely adjudicable under existing semantics.

Do not fabricate 6–12 hours of work if repository truth does not supply it. Conversely, while READY_LOCAL or a later concrete repair remains, implement/test/commit/push continuously without waiting for reviewer cadence.

## Live/VPS boundary

`READY_LIVE: none` remains authoritative. No new live question was produced by this review.

Do not repeat unchanged HY2, repeated warm failover, periodic/soak, package lifecycle, migration-back, endpoint/source migration, key update, IPv6, PLPMTUD, or Experimental Track runs merely because the VPS rental window remains open. A future materially changed, dependency-ready self-owned TCP/UDP question may use standing authorization directly, but this handoff creates none.

## Stop/escalation conditions

Stop/escalate only for:

- unresolved BLOCKER/HIGH that cannot safely be repaired under existing semantics;
- required core Session/Carrier/ACK/crypto/wire architecture change;
- destructive/canonical-meaning migration;
- action outside standing authorization;
- production impact;
- new credentials/server/third-party permission;
- benchmark/adversarial-load conditions requiring maintainer value judgment;
- D019 policy decision;
- real repository breakage;
- runtime/tool-budget exhaustion;
- genuine review/coding queue exhaustion after the explicit slices above are resolved.

Otherwise: bounded review/support -> concrete finding if any -> smallest repair -> focused tests -> commit/push -> clean exact-tree local gate -> concise provenance -> factual reconciliation -> continue.