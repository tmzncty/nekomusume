# ChatGPT reviewer handoff — bounded independent review accepted as partial item-4 evidence; no coding queue inflation

## Reviewed state

- Previous reviewer-owned handoff: exact `3ed596ac62a3e3bda0fcea494432e0533285cb83` (`docs(handoff): accept release facts and force final proposal sweep`).
- Developer/current sequence after that handoff:
  1. `a24370bde84b86dced1e0af88283679afbd414c2` — docs-only bounded independent release/security evidence-index review of exact `3ed596a`;
  2. `013f7526c2bff83a6b9c71489c664cf16a7cbcab` — docs-only Era-4 ledger provenance scoping, converting the stale generic `closure.handoff_sha256` into explicitly historical initial-classification provenance.
- No runtime implementation, tests, package mutation, fixture reconstruction, or real VPS/WAN experiment landed in this developer sequence.
- Reviewer follow-up commit `011eca9e49a3504c998c588ab03414b21f6fecad` sanitizes unnecessary local review-host/private-workdir details from the current-tree review note. This is reviewer-owned evidence hygiene only; it does not change the reviewed tree, release facts, classification, or any protocol/runtime behavior.
- GitHub exposes no hosted status records for the current docs head. Hosted CI remains optional cross-evidence and is not a wait condition.
- This ChatGPT reviewer performed GitHub repository/source/evidence review only. No reviewer-executed local CI is claimed in this turn.

## Review verdict

### `a24370b` — ACCEPT_WITH_BOUNDS as partial independent-review evidence

The bounded review is useful and reproducible within its stated scope. It independently reproduced or spot-checked:

- release flags and cross-document boundary consistency;
- zero Era-4 `OPEN_READY` rows on the reviewed exact `3ed596a` tree;
- docs-only delta since the last fully gated classification tree `78111e8`;
- canonical corpus validation (`42` vectors / `10` domains / `freeze=true`);
- the `neko-crypto` 37-test matrix;
- release boundary/link/status checks;
- package smoke / clean-source regressions;
- the retained five-record package anchor hash;
- targeted Unix identity open/create behavior.

Its own scope explicitly excludes exhaustive source audit, cryptanalysis, adversarial-load assessment, WAN/VPS execution, penetration testing, and production approval. Therefore it is **not** sufficient to mark release item 4 complete and does not close RSEC-001 or D019. It found no BLOCKER/HIGH inside the surfaces it actually reviewed; that statement must not be promoted to “no security HIGH remains repository-wide.”

The review artifact is evidence support, not a release/security approval. `RELEASE_CANDIDATE=false`, `PRODUCTION_READY=false`, `FREEZE=false`, and `RELEASED=false` remain unchanged.

### `013f752` — ACCEPT

The Era-4 ledger change correctly scopes the stale handoff digest as historical initial-classification provenance:

- the rolling classifications continue to be cross-checked against `IMPLEMENTATION_PLAN.md` and `docs/status.md`;
- `open_ready_rows`, `already_sufficient_rows`, and `blocked_dependency_rows` are unchanged;
- no live row is promoted and no dependency is reclassified;
- `scripts/check-era4-closure.py` does not depend on the removed generic `handoff_sha256` field.

This is evidence-navigation repair only, not a new release fact.

## Reviewer evidence-hygiene finding — current tree repaired, history intentionally not rewritten

The new independent review note originally persisted a local RFC1918 host identifier and an absolute local checkout path. That is unnecessary provenance and conflicts with the repository rule against storing endpoint/private-topology detail when a sanitized host description is sufficient.

Severity: **MEDIUM evidence/security hygiene**, not a credential leak and not a release correctness blocker.

Reviewer action already taken in exact `011eca9`:

- current-tree note now says the review host and checkout path are intentionally sanitized;
- `rustc 1.98.0` and all actual commands/results remain preserved;
- no substantive review claim changed.

The original values remain reachable in Git history at `a24370b`. Fully purging historical Git objects would require destructive history rewriting, which is outside normal reviewer authority and must not be performed automatically. No history rewrite is requested or implied by this handoff.

## Exact-current release boundary

Repository truth remains:

- `IMPLEMENTATION_COMPLETE=true` only in the bounded research/governance sense;
- release item 3 remains incomplete;
- release item 4 remains incomplete, although one bounded independent evidence-index review now exists;
- `RELEASE_CANDIDATE=false`;
- `PRODUCTION_READY=false`;
- `FREEZE=false`;
- `RELEASED=false`;
- D019 remains `SOURCE_RETENTION_POLICY_BLOCKED`;
- RSEC-001 still requires representative adversarial-load/suitability evidence and an independent release/security decision before public-listener/release promotion;
- current live opportunity remains `READY_LIVE: none`.

Standing VPS authorization remains valid. It is not a reason to repeat HY2, repeated warm failover, periodic/soak, package lifecycle, migration-back, endpoint migration, key update, IPv6, or PMTUD without a materially new dependency-satisfied question.

## Stage transition — coding expansion remains exhausted; independent review is now the real queue

The final proposal sweep did not produce a concrete new implementation defect. The next work is therefore **independent release/security review**, not filler coding and not watcher polling.

The external coding agent remains a support/repair agent during this stage. It may reproduce deterministic evidence, prepare narrowly scoped factual notes, or immediately repair a concrete finding that the reviewer identifies under existing semantics. It must not self-promote its own support work into “independent release approval,” and it must not manufacture new checker/parser/harness/framework work merely to stay busy.

### READY_LOCAL 1 — reconcile the first independent-review artifact into release-facing navigation

Goal: make release-facing docs acknowledge that one bounded independent evidence-index review has occurred without claiming item-4 completion.

Files/concepts:

- `docs/release-security-review-packet.md`;
- `docs/reviews/release-item4-subgates-20260909.md` only if necessary for factual indexing;
- `docs/status.md` only if an actual status statement needs precision;
- `docs/reviews/independent-release-review-3ed596a-20260911.md` as the source artifact.

Required boundary:

- describe the review as bounded/partial;
- preserve its exclusions (no exhaustive audit, cryptanalysis, adversarial load, WAN, penetration test, production approval);
- do not mark item 4 complete;
- do not flip any release flag;
- do not convert “no HIGH found in reviewed surfaces” into a repository-wide security verdict;
- keep the current-tree sanitized host/workdir wording; do not reintroduce private endpoint/topology detail.

Validation/closure:

- `python3 scripts/check-era4-closure.py`;
- `bash scripts/check-plan-sync.sh`;
- `bash scripts/check-status-evidence.sh`;
- `bash scripts/check-release-boundaries.sh`;
- `bash scripts/check-markdown-links.sh`;
- then one clean exact-tree `PYTHONDONTWRITEBYTECODE=1 bash scripts/check.sh`, `git diff --check`, and clean-tree check on the final pushed docs SHA;
- persist one concise local-CI provenance note for that final exact tree. Do not create separate provenance churn for `013f752` or reviewer-only `011eca9`; fold them into this final docs-tree gate.

After green closure, continue directly. Do not wait for the next reviewer turn.

### REVIEW SUPPORT 2 — independent resource/abuse review, non-policy portion only

This is a genuine release-item-4 review slice, not a new implementation project.

Primary surfaces:

- `SECURITY.md` resource/anti-amplification red lines;
- `docs/reviews/resource-abuse-evidence-2026-09-04.md`;
- `docs/adr/m1-g0-preauth-resource-budget.md`;
- `crates/neko-crypto/src/lib.rs::{PreauthBudget, ProcessPreauthAdmission}`;
- `crates/neko-cli/src/preauth.rs::{ListenerAdmission, AdmissionTicket, response/input permits}`;
- the inventoried executable responder/admission call paths and their existing deterministic/process tests.

Reviewer question:

> Do the exact-current non-policy controls actually enforce charge-before-expensive-work, bounded response amplification, bounded source/global state/queue/memory/work accounting, permit settlement, expiry/release cleanup, and fail-closed terminalization on every currently advertised responder path?

External coding-agent behavior:

- reproduce focused deterministic/process evidence and collect exact code pointers;
- do not invent TTL/LRU/history capacity or weaken D019;
- do not run a new WAN/adversarial load merely because the VPS exists;
- if a concrete correctness/security defect is demonstrated, it becomes immediate queue head: implement the smallest repair under existing semantics, add bounded positive/negative regressions, run exact-tree local gates, commit/push, and stop broader review until the defect is closed;
- if no defect is found, record a bounded support note only. RSEC-001 still remains open for adversarial-load/suitability and release decision unless an independently justified later stage closes it.

### REVIEW SUPPORT 3 — comparison-methodology boundary review

Primary surfaces:

- current HY2 owned-lab runner/validator scripts;
- retained exact `13da094` / prior negative artifacts and their immutable cleanup/status fields;
- release packet comparison claims.

Reviewer question:

> Can any incomplete pair, failed HY2 sample, cleanup-negative artifact, or current-line frozen result accidentally produce or imply median/P95/superiority/comparative success that the retained evidence does not support?

No live rerun is authorized by this queue entry. This is static/deterministic review support only because the current live ledger says `READY_LIVE: none` and same-class HY2 retry is frozen without a material new hypothesis.

If a concrete claim-suppression/validator defect exists, repair/test/gate it immediately. Otherwise record the bounded review and continue.

### REVIEW CHECKPOINT 4 — item-4 partial-closure assessment

After the two bounded review-support slices above, reconcile what item 4 actually has versus what remains:

- canonical vectors;
- compatibility policy/current-current behavior;
- package rollback and operator lifecycle;
- comparison methodology;
- resource/abuse controls;
- independent review depth;
- D019 policy boundary;
- adversarial-load/suitability boundary.

Do not mark item 4 complete merely because all deterministic checks pass. If the only remaining gates require D019 policy, adversarial-load condition/suitability value judgment, release signing/key-custody/SBOM policy, unavailable environment, or a genuinely independent human/maintainer security decision, classify them explicitly and stop coding expansion.

## Honest rolling queue

The local implementation overlay still has zero `OPEN_READY` rows, so the queue is intentionally short and review-oriented:

1. **READY_LOCAL:** release-facing indexing of the first bounded independent review + one exact-tree docs gate/provenance closure.
2. **REVIEW SUPPORT:** non-policy pre-auth/resource-abuse evidence reproduction and exact-current challenge.
3. **REVIEW SUPPORT:** HY2/comparison methodology claim-boundary challenge without live retry.
4. **CHECKPOINT:** item-4 partial-closure assessment and explicit remaining-gate classification.
5. **CONDITIONAL READY_LOCAL:** any concrete correctness/security/evidence defect found by 2–4 becomes immediate repair head and blocks broader expansion until closed.
6. **REAL STOP/ESCALATION:** D019 policy choice, destructive history rewrite/purge, production impact, new credentials/server/third-party permission, benchmark/adversarial-load conditions requiring maintainer value judgment, core Session/Carrier/ACK/crypto/wire architecture change, or a major security issue not safely adjudicable under existing semantics.

Do not fabricate 6–12 hours of implementation work. Do not enter 5/30-minute watcher loops. During this independent-review stage, a quiet coding queue is legitimate; only concrete findings create new implementation slices.

## Live/VPS boundary

`READY_LIVE: none` remains authoritative for currently unresolved live capability rows. Standing authorization still permits bounded self-owned TCP/UDP experiments when a real dependency-ready question appears, but no such new question is created by this review.

Do not repeat unchanged HY2, repeated warm failover, periodic/soak, package lifecycle, migration-back, endpoint/source migration, key update, IPv6, PLPMTUD, or Experimental Track runs merely to consume the rental window.

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
- genuine review/coding queue exhaustion.

Otherwise: bounded review/support slice -> concrete finding if any -> smallest repair -> focused tests -> commit/push -> clean exact-tree local gate -> concise provenance -> factual reconciliation -> continue to the next explicitly pre-authorized review slice.
