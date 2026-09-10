# ChatGPT reviewer handoff — HY2 blocked-artifact repair closed; reconcile common benchmark-result v1 contract

## Reviewed state

- Previous reviewer-owned handoff: exact `580da6a6ce3736983aac60e111302bc39f8ec205` (`docs(handoff): close HY2 validator repair before item-4 checkpoint`).
- Current developer-owned head reviewed this turn: exact `6def1c6182c86e415dcbb974a959fd8ed3b00aac` (`docs: close HY2 blocked schema review`), parented directly on that handoff.
- `6def1c` is docs/evidence only. It adds developer-local exact-tree provenance for implementation/test commit `aca2842bed699448b19b0d1ef19b3acaf3bcecdf`, indexes the bounded independent HY2 methodology review + repair into release/item-4 navigation, and changes no runtime implementation, Session/Carrier/wire/crypto architecture, package behavior, fixture reconstruction, or VPS/WAN evidence.
- GitHub exposes no hosted status records for exact `aca2842`; hosted CI remains optional cross-evidence and is not a wait condition.
- This ChatGPT reviewer performed GitHub repository/source/evidence review only. No reviewer-executed local CI is claimed in this turn.

## Review verdict

### `6def1c` — ACCEPT_WITH_BOUNDS

The previously demonstrated blocked-artifact MEDIUM is now closed at implementation/test + developer-local exact-tree gate level:

- exact `aca2842` requires the canonical nine top-level keys for `nekomusume.benchmark-blocked-harness.v1` and rejects noncanonical comparative-looking extras;
- retained blocked artifacts validate without rewrite;
- focused HY2 validator/comparison tests pass;
- every retained `artifacts/hy2-owned-lab/*/result.json` validates;
- `PYTHONDONTWRITEBYTECODE=1 bash scripts/check.sh` and `git diff --check` both exit `0` on a clean detached checkout of exact `aca2842`;
- recorded full-gate UTC interval is `2026-09-10T22:13:24Z -> 22:15:20Z`, Linux x86_64, Rust `1.98.0`, clean before/after.

This is **developer-local CI provenance**, not reviewer-executed CI, GitHub-hosted CI, a security audit, release approval, or production evidence. No live HY2 run occurred and no complete pair/performance/superiority result exists.

`6def1c` itself is the subsequent docs/evidence commit and is not the tested implementation tree. The previous handoff asked for one final exact-tree docs closure after factual reconciliation; no persistent `scripts/check.sh` provenance for exact `6def1c` is present. Treat that as a low-priority closure hygiene gap, not a reason to stall. Fold the next coherent docs/code package into one final exact-tree gate rather than creating a standalone watcher/provenance cycle for `6def1c`.

## New finding — MEDIUM evidence-contract drift in `nekomusume.benchmark-result.v1`

The previous handoff asked whether `schema/benchmark-result.v1.json` and the HY2 owned-lab Python validator are intentionally distinct contracts. Exact-current repository truth answers that question: **the common JSON schema is claimed as the common result envelope, while current comparison producers emit the same exact schema identifier but do not conform to it.** This is a real current contract mismatch, not merely similar naming.

Evidence:

- `docs/bench/result-schema-v1.md` explicitly says `schema/benchmark-result.v1.json` is the **common result envelope** for deterministic, netns, VPS, and later comparison experiments and says it requires source commit, mode/transport/scope, bounds, samples, summary, and cleanup status.
- `scripts/bench/compare-hy2.sh` emits `schema: "nekomusume.benchmark-result.v1"`.
- `scripts/bench/compare-hy2-owned-lab.sh` also emits `schema: "nekomusume.benchmark-result.v1"` and validates that complete result through the specialized Python validator.
- `schema/benchmark-result.v1.json` currently requires `bounds`, declares `summary` to be an object, and has top-level `additionalProperties: false` without a `cleanup_evidence` property.
- both current comparison producers emit `summary` as an **array** produced by `group_by(... ) | map(...)`;
- the generic `compare-hy2.sh` complete result does **not** emit the required top-level `bounds` object;
- the owned-lab complete result emits top-level `cleanup_evidence`, which the common JSON schema rejects because it is not a declared property;
- generic failed samples can retain `application_bytes: null`, while the common schema currently types present `application_bytes` only as an integer.

This is **MEDIUM evidence/schema correctness**, not a security HIGH: the owned-lab specialized Python validator still fail-closes its actual evidence semantics, the current retained HY2 line is `BLOCKED_HARNESS`, and there is no current complete HY2 pair/performance claim whose numerical conclusion is invalidated by this mismatch. But an artifact that labels itself `nekomusume.benchmark-result.v1` must not be structurally incompatible with the repository's documented schema of that same identifier.

The separate `nekomusume.benchmark-blocked-harness.v1` JSON schema and the newly repaired blocked validator are not reopened by this finding.

## READY_LOCAL 1 — reconcile the common `benchmark-result.v1` contract

This demonstrated MEDIUM is the immediate queue head. Do not wait for the next reviewer cycle.

### Goal

Make current complete comparison producers and the documented/schema contract mutually consistent without weakening evidence semantics, inventing a schema framework, or performing a live benchmark.

### Bounded proposal/implementation authority

Before editing, inspect exact consumers/tests of:

- `schema/benchmark-result.v1.json`;
- `docs/bench/result-schema-v1.md`;
- `scripts/bench/compare-hy2.sh` + test;
- `scripts/bench/compare-hy2-owned-lab.sh` + validator/tests;
- any committed complete-result artifacts using `nekomusume.benchmark-result.v1`.

Then compare at most 1–3 **minimal** reconciliation shapes. The following is pre-authorized as `ACCEPT_WITH_BOUNDS` if consumer truth supports it:

1. keep the existing `nekomusume.benchmark-result.v1` identifier;
2. make the JSON schema faithfully accept the intended current complete-result envelope(s), including the established array summary shape and owned-lab cleanup evidence, with bounded types/keys rather than a broad `additionalProperties: true` escape hatch;
3. make the generic comparison producer emit any truly required common-envelope field that can be derived truthfully under existing semantics (notably `bounds`) rather than silently weakening a documented requirement;
4. preserve failure truthfulness: nullable/absent evidence on failed samples must not be rewritten into invented success evidence merely to satisfy schema;
5. preserve the stronger specialized owned-lab Python semantic validator; JSON-schema conformity is an additional structural contract, not a replacement for lifecycle/resource/cleanup checks.

If a field such as generic `maximum_duration_ms` cannot be assigned a truthful meaning from the existing harness without redefining benchmark semantics, do **not** guess. Prefer a smaller documented/common-contract reconciliation or present the specific versioned alternative in the 1–3 option comparison. A schema identifier split, reinterpretation of committed complete artifacts, or destructive canonical migration is **not** pre-authorized merely to make tests green.

### Required regressions

At minimum prove the repaired contract against representative outputs from both current complete comparison producers:

- generic local `compare-hy2.sh` success output conforms structurally;
- generic failed-sample output remains truthful and conforms if failed results are intended to remain in v1;
- owned-lab complete-result assembly shape conforms structurally, including cleanup evidence;
- malformed/unknown top-level evidence fields remain fail-closed where the contract claims key exclusivity;
- malformed summary/bounds/cleanup evidence does not become accepted merely because of the reconciliation;
- existing specialized owned-lab semantic-validator tests remain green;
- `BLOCKED_HARNESS` exact-key regressions remain green and use their separate schema.

Reuse existing repository tooling if present. Do not add a heavyweight schema-validation framework solely for this slice. A small targeted contract test is acceptable if needed to keep the schema and producer shapes from drifting again.

### Closure

After the coherent repair commit:

- run focused benchmark/schema tests;
- run `PYTHONDONTWRITEBYTECODE=1 bash scripts/check.sh`;
- run `git diff --check`;
- verify the exact developer SHA is clean before/after;
- persist minimum sanitized developer-local provenance for the **final implementation/test tree**;
- then update only genuinely stale benchmark/release/item-4 navigation and run one final docs-tree closure; a later provenance-only commit may record that result without recursively testing itself.

No fuzz is required unless the repair unexpectedly touches wire decoder/parser/crypto framing, which is outside this slice and should normally not happen.

After green closure, continue immediately to REVIEW SUPPORT 2. Do not enter watcher mode.

## REVIEW SUPPORT 2 — bounded item-4 evidence challenge

After READY_LOCAL 1, continue the independent-review support queue rather than inventing implementation backlog. Challenge current release item-4 evidence in one bounded pass, prioritizing places where a machine-readable or release-facing claim can disagree with code/artifacts.

Cover at least:

- common benchmark-result schema/producer conformity after the repair;
- canonical corpus/vector review and what has actually been independently reproduced;
- package build/install/upgrade/rollback/archive validation boundaries;
- operator `READY -> DRAINING -> STOPPED` and cleanup evidence boundaries;
- Session/Carrier bounded local evidence versus unverified WAN/long-run claims;
- non-policy pre-auth resource/abuse controls versus still-open adversarial-load capacity/suitability;
- HY2 methodology and the distinction between blocked evidence, a complete pair, and a performance conclusion;
- D019 policy boundary;
- relationship to still-incomplete release item 3.

If this pass finds a concrete correctness/security/evidence defect with an existing semantic answer, it becomes immediate READY_LOCAL head: smallest repair -> bounded positive/negative tests -> commit/push -> exact-tree local gate -> provenance -> factual reconciliation -> continue. Do not turn the review into generic checker/harness normalization.

If no concrete repairable defect remains, write only the bounded factual checkpoint needed for navigation and classify remaining gates honestly. Developer/agent-prepared support cannot self-promote into an independent final security/release approval.

## REVIEW CHECKPOINT 3 — classify remaining release gates

After READY_LOCAL 1 and REVIEW SUPPORT 2, explicitly classify what remains as one of:

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
- release item 4 remains incomplete, with bounded independent evidence-index, non-policy resource/abuse, and HY2 methodology review support only;
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

1. **READY_LOCAL:** repair the demonstrated common `benchmark-result.v1` schema/producer mismatch, add bounded regressions, exact-tree gate, provenance, factual reconciliation.
2. **REVIEW SUPPORT:** one bounded item-4 evidence challenge after the schema repair; concrete defects immediately become repair heads.
3. **CHECKPOINT:** classify remaining item-3/item-4/D019/RSEC-001/independent-review/release-decision gates.
4. **CONDITIONAL READY_LOCAL:** any concrete defect found in 2–3 with an existing semantic answer is pre-authorized for the smallest fix + tests + exact-tree local closure.
5. **REAL STOP/ESCALATION:** D019 policy choice; schema/canonical-result migration that requires redefining committed meaning; adversarial-load/benchmark conditions requiring maintainer value judgment or exceeding standing authorization; destructive history rewrite; production impact; new credentials/server/third-party permission; core Session/Carrier/ACK/crypto/wire architecture change; or a major security issue not safely adjudicable under existing semantics.

Do not fabricate 6–12 hours of work if repository truth does not supply it. Conversely, while the benchmark-result contract MEDIUM or a later concrete repair remains, keep implementing/testing/committing/pushing continuously without waiting for hourly reviewer refresh.

## Live/VPS boundary

`READY_LIVE: none` remains authoritative. No new live question was produced by this review.

Do not repeat unchanged HY2, repeated warm failover, periodic/soak, package lifecycle, migration-back, endpoint/source migration, key update, IPv6, PLPMTUD, or Experimental Track runs merely because the VPS rental window remains open. A future materially changed, dependency-ready self-owned TCP/UDP question may use standing authorization directly, but this handoff creates none.

## Stop/escalation conditions

Stop/escalate only for:

- unresolved BLOCKER/HIGH that cannot safely be repaired under existing semantics;
- required core Session/Carrier/ACK/crypto/wire architecture change;
- destructive/canonical-meaning migration, including a benchmark schema version split that would reinterpret committed complete-result artifacts without an already documented migration rule;
- action outside standing authorization;
- production impact;
- new credentials/server/third-party permission;
- benchmark/adversarial-load conditions requiring maintainer value judgment;
- D019 policy decision;
- real repository breakage;
- runtime/tool-budget exhaustion;
- genuine review/coding queue exhaustion after the explicit slices above are resolved.

Otherwise: bounded review/support -> concrete finding if any -> smallest repair -> focused tests -> commit/push -> clean exact-tree local gate -> concise provenance -> factual reconciliation -> continue.