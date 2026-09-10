# ChatGPT reviewer handoff — independent resource/HY2 reviews accepted; close repaired HY2 evidence boundary before item-4 checkpoint

## Reviewed state

- Previous reviewer-owned handoff: exact `f184cf21cd7b51c02d2b6e0f7850600f1be28784` (`docs(handoff): enter bounded independent review queue`).
- Developer sequence after that handoff, before reviewer-only evidence hygiene:
  1. `2ffde7444f4705adb0b82b07a57fee75b261ba5f` — docs-only indexing of the first bounded independent release/evidence review;
  2. `b28349d9de6b56939391c2c4991dd634b1740d4d` — docs-only independent bounded non-policy pre-auth resource/abuse review of exact `f184cf2`;
  3. `e5fefc1c90b38a8c4a1d2ebb46c4f6262c27a3a9` — docs-only indexing/sanitization of that resource review;
  4. `5dbade823da8ecdec78c0a8d7ad16c86a808a576` — docs-only independent bounded HY2 comparison-methodology review of exact `e5fefc1`;
  5. `aca2842bed699448b19b0d1ef19b3acaf3bcecdf` — implementation/test repair: make `BLOCKED_HARNESS` result validation top-level-key-exclusive and add negative regressions for noncanonical comparative fields.
- No new real VPS/WAN experiment, package mutation, protocol/wire/crypto change, Session/Carrier architecture change, or fixture reconstruction landed in this sequence.
- Reviewer-only exact `4f9aa47a0471de4442fcaa62d5044926620b955f` sanitizes unnecessary local review-host/checkout-path detail from the current-tree HY2 methodology note. It changes no reviewed code, test, release classification, or methodology finding. The historical values remain in prior Git objects; destructive history rewrite is neither authorized nor requested.
- GitHub exposes no hosted status records for exact developer implementation head `aca2842`. Hosted CI remains optional cross-evidence and is not a wait condition.
- This ChatGPT reviewer performed GitHub repository/source/evidence review only. No reviewer-executed local CI is claimed in this turn.

## Review verdict

### `2ffde744` — ACCEPT

The packet/item-4 navigation now truthfully indexes the earlier bounded independent evidence review without promoting it into item-4 completion, security approval, RC, release, or production authorization.

### `b28349d` / `e5fefc1` — ACCEPT_WITH_BOUNDS as partial item-4 resource/abuse evidence

The independent non-policy pre-auth review is useful within its declared scope. It independently reproduced:

- the machine-checked inventory of `9` responder surfaces / `7` admission sites;
- `cargo test -p neko-crypto preauth` (`24` passed on the reviewed tree);
- `cargo test -p neko-cli` (`39` passed on the reviewed tree);
- charge-before-protected-work ordering at the inventoried callsites;
- per-state/source/global response, state, queue, memory and work accounting;
- response/input permit settlement;
- expiry/release/terminalization behavior;
- the current non-policy `SECURITY.md` amplification/resource red lines.

No BLOCKER/HIGH was found **inside those audited non-policy surfaces**. That must not be widened into “no repository-wide security HIGH exists.” The review explicitly excludes adversarial-load capacity/suitability, D019 source-retention policy, cryptanalysis, penetration testing, WAN execution and production approval. Therefore:

- RSEC-001 remains open;
- D019 remains `SOURCE_RETENTION_POLICY_BLOCKED`;
- release item 4 remains incomplete;
- no public-listener or production claim follows.

The current-tree resource review provenance is sanitized. No history rewrite is requested.

### `5dbade8` — ACCEPT_WITH_BOUNDS as partial comparison-methodology review

The static/deterministic review correctly demonstrated that the owned-lab runner cannot turn an incomplete pair, failed HY2 sample, cleanup-negative artifact, or frozen current-line result into a valid comparative `summary`; retained `BLOCKED_HARNESS` artifacts contain no comparative summary and current exact `13da094` still has only a Nekomusume success followed by an HY2 failure before HY2 application bytes.

The review also found one concrete **MEDIUM evidence-integrity defect** under existing documented semantics: externally/hand-authored `nekomusume.benchmark-blocked-harness.v1` documents could carry arbitrary extra top-level comparative-looking fields because the blocked validator used `required.issubset(doc)` and rejected only the literal `summary` key. The runner itself could not generate the malformed shape and no retained artifact used it, so this was not a current performance-claim compromise or HIGH.

### `aca2842` — SOURCE REVIEW ACCEPT; exact-tree closure still required

The developer repair is the right minimum shape:

- the blocked-artifact required top-level set now explicitly includes `contract`;
- `set(doc) == required` is required before accepting a `BLOCKED_HARNESS` artifact;
- four mutation regressions cover `median_exchange_latency_ms`, `superiority`, `comparative_summary`, and `p95_latency_seconds` as forbidden noncanonical extras;
- the retained exact `13da094` blocked result already has exactly the canonical nine-key shape, so no historical artifact rewrite is required.

This repairs the demonstrated MEDIUM at source/test level. **Do not yet write “closed by a green exact-tree gate”**: no persistent developer-local `scripts/check.sh` provenance for exact `aca2842` is present at this review point, and GitHub has no hosted status for it. Focused review/test existence is not a substitute for the requested exact-tree gate.

No fuzz is required for this repair: it changes the Python benchmark evidence validator/test, not wire decode/parser/crypto framing.

## Exact-current release/live boundary

Repository truth remains unchanged:

- `IMPLEMENTATION_COMPLETE=true` only in the repository's bounded research/governance sense;
- release item 3 remains incomplete;
- release item 4 remains incomplete, now with bounded independent evidence-index, non-policy resource/abuse, and comparison-methodology review material;
- `RELEASE_CANDIDATE=false`;
- `PRODUCTION_READY=false`;
- `FREEZE=false`;
- `RELEASED=false`;
- D019 remains `SOURCE_RETENTION_POLICY_BLOCKED`;
- RSEC-001 still lacks representative adversarial-load/capacity-suitability evidence and the independent release/security decision required for promotion;
- current live opportunity remains `READY_LIVE: none`.

The HY2 current line remains frozen at exact `13da094`: no complete pair, no median/P95 comparison, no superiority result, and no same-class live retry without a materially new hypothesis. Standing VPS authorization remains valid but creates no new live question by itself.

## READY_LOCAL 1 — close the repaired HY2 methodology/evidence package

This is the immediate queue head. It is a closure package for a demonstrated defect, not another broad benchmark project.

### 1A. Exact implementation-tree verification

Use a safe temporary worktree/clean checkout of exact developer implementation commit `aca2842bed699448b19b0d1ef19b3acaf3bcecdf`; do not disturb the active shared workspace.

Run at minimum:

- `python3 scripts/bench/validate-hy2-owned-lab-test.py`;
- `bash scripts/bench/compare-hy2-owned-lab-test.sh`;
- `bash scripts/bench/compare-hy2-test.sh`;
- `python3 scripts/bench/validate-hy2-owned-lab.py validate-result` on each retained `artifacts/hy2-owned-lab/*/result.json`;
- `PYTHONDONTWRITEBYTECODE=1 bash scripts/check.sh`;
- `git diff --check`;
- confirm the exact checkout is clean before and after.

Persist minimum provenance for exact `aca2842`: exact SHA, commands, UTC start/end, exit codes, sanitized host OS/arch, Rust stable version, clean-tree state. Do not store local addresses, private topology, credentials, or absolute private checkout paths.

If any gate is red, repair the real failure first and rerun. Do not substitute absent GitHub Actions for local closure.

### 1B. Release-facing factual reconciliation

After exact `aca2842` is green, update only the release-facing facts that actually changed:

- `docs/release-security-review-packet.md` — index the independent HY2 methodology review and the subsequent bounded validator repair;
- `docs/reviews/release-item4-subgates-20260909.md` — record that the review found a MEDIUM in blocked-artifact key exclusivity and that exact `aca2842` repairs it after the exact-tree gate is green;
- `docs/status.md` only if a current status sentence is actually stale;
- retain the historical reviewed-tree anchor of `docs/reviews/independent-hy2-methodology-review-e5fefc1-20260911.md`; do not rewrite it to pretend the review originally observed the later repair.

Required claims:

- incomplete/failed/cleanup-negative BLOCKED artifacts cannot carry noncanonical top-level comparative fields after the repair;
- no retained artifact needed rewriting;
- no live HY2 run occurred;
- no complete pair or performance conclusion exists;
- item 4 remains incomplete;
- all release flags remain false.

For the final coherent docs reconciliation SHA, run the focused repository policy checks (`check-era4-closure`, `check-plan-sync`, `check-status-evidence`, `check-release-boundaries`, `check-markdown-links`) and then one clean exact-tree `scripts/check.sh + git diff --check` closure. A later provenance-only commit may record that final docs-tree result; do not recurse into testing the provenance-only commit itself.

After green closure, continue immediately to the next review slice. Do not enter watcher mode.

## REVIEW SUPPORT 2 — bounded complete-result schema/validator applicability check

This is a focused release-evidence contract question discovered while reviewing the HY2 validator; it is **not yet declared a defect**.

Observed exact-current facts that require reconciliation:

- the HY2 complete-result branch of `validate-hy2-owned-lab.py` intentionally accepts a required subset plus runner metadata;
- `schema/benchmark-result.v1.json` declares `additionalProperties: false` for `nekomusume.benchmark-result.v1` and lists a different common envelope;
- the owned-lab runner emits fields such as `cleanup_evidence` in its complete result.

Question:

> Is `schema/benchmark-result.v1.json` actually an authoritative validator contract for HY2 owned-lab complete results, or is the Python owned-lab validator the intentionally specialized authority and the common schema only documents another result surface?

Agent behavior:

1. Trace exact consumers/checkers and docs; do not assume the JSON schema applies merely because the schema string is similar.
2. If the two contracts are intentionally distinct and no current claim says otherwise, record the boundary and stop; do not invent a schema framework.
3. If repository docs/checkers claim that the common JSON schema governs owned-lab complete results, propose 1–3 minimal reconciliation shapes, select the smallest fail-closed option under existing semantics, add bounded positive/negative tests, implement, exact-tree gate, commit/push, and then continue.
4. Do not broaden this into benchmark redesign or a live HY2 retry.

Any demonstrated evidence-integrity defect becomes queue head until closed. Otherwise this slice ends with a bounded factual note or no commit if no navigation change is needed.

## REVIEW CHECKPOINT 3 — item-4 partial-closure assessment

After READY_LOCAL 1 and REVIEW SUPPORT 2, assess item 4 against exact-current evidence. Reconcile at least:

- canonical corpus/vector review;
- compatibility/current-current behavior and the absence of a prior frozen release;
- package build/install/upgrade/rollback/archive validation;
- operator READY/DRAINING/STOPPED and cleanup behavior;
- Session/Carrier bounded local evidence;
- non-policy resource/abuse controls;
- comparison methodology and the repaired blocked-artifact validator;
- independent review depth actually performed;
- D019 policy boundary;
- RSEC-001 adversarial-load/capacity-suitability boundary;
- relationship to still-incomplete release item 3.

Do **not** mark item 4 complete merely because deterministic/local checks are green. Developer/agent-prepared review support cannot self-promote into a genuinely independent release/security approval. If the remaining gates are policy/value/environment/independent-human decisions rather than code defects, classify them explicitly and stop coding expansion.

## Honest rolling queue

The local Era-4 implementation overlay had zero `OPEN_READY` rows before these independent reviews, so the queue remains intentionally short and evidence-driven:

1. **READY_LOCAL:** exact-tree gate + provenance for `aca2842`, then factual release/index reconciliation and one final docs-tree gate.
2. **REVIEW SUPPORT:** determine whether the complete-result JSON-schema/Python-validator relationship is a real current contract mismatch; repair only if demonstrated.
3. **CHECKPOINT:** item-4 partial-closure assessment with explicit remaining-gate classification.
4. **CONDITIONAL READY_LOCAL:** any concrete correctness/security/evidence defect found by 2–3 becomes immediate repair head: smallest fix -> bounded tests -> commit/push -> exact-tree local gate -> provenance -> factual reconciliation.
5. **REAL STOP/ESCALATION:** D019 policy choice; adversarial-load/benchmark conditions requiring maintainer value judgment or exceeding standing authorization; destructive Git-history rewrite; production impact; new credentials/server/third-party permission; core Session/Carrier/ACK/crypto/wire architecture change; or a major security issue not safely adjudicable under existing semantics.

Do not fabricate 6–12 hours of implementation work after genuine queue exhaustion. Do not create checker/parser/harness churn merely to keep the agent busy. Conversely, while an explicit READY_LOCAL or demonstrated conditional repair remains, do not wait for the next hourly reviewer update.

## Live/VPS boundary

`READY_LIVE: none` remains authoritative for unresolved live capability rows. No new live question was produced by these reviews.

Do not repeat unchanged HY2, repeated warm failover, periodic/soak, package lifecycle, migration-back, endpoint/source migration, key update, IPv6, PLPMTUD, or Experimental Track runs merely because the VPS rental window remains open. A future materially changed, dependency-ready self-owned TCP/UDP question may use standing authorization directly, but this handoff creates none.

## Stop/escalation conditions

Stop/escalate only for:

- unresolved BLOCKER/HIGH that cannot safely be repaired under existing semantics;
- required core Session/Carrier/ACK/crypto/wire architecture change;
- destructive/canonical-meaning migration or Git-history rewrite;
- action outside standing authorization;
- production impact;
- new credentials/server/third-party permission;
- benchmark/adversarial-load conditions requiring maintainer value judgment;
- D019 policy decision;
- real repository breakage;
- runtime/tool-budget exhaustion;
- genuine review/coding queue exhaustion after the explicit slices above are resolved.

Otherwise: bounded review/support -> concrete finding if any -> smallest repair -> focused tests -> commit/push -> clean exact-tree local gate -> concise provenance -> factual reconciliation -> continue to the next pre-authorized slice.
