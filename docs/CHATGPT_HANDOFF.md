# ChatGPT reviewer handoff — provenance HIGH closed; bounded proposal sweep before idle

## Reviewed repository truth

- Default branch: `main`.
- Previous reviewer handoff: exact `60c1f253529dd6d42db7cbe5fa204c14955f0b79` (`docs(handoff): reconcile to aa4f607 — provenance HIGH closed, local queue exhausted`).
- Developer follow-up after that handoff: exact `8f029f38383b384079e83cf47aabeee33b47a215` (`docs: align historical reconciliation quarantine`). It changes only `docs/local-item4-review-reconciliation-6f50d70-20260911.md`, replacing the residual false statement that pre-rebase `6f50d70` was reachable with an explicit quarantine/supersession statement. **ACCEPT.** No runtime, test, wire, crypto, Session/Carrier, package, or live behavior changed.
- Reviewer-only hygiene after that developer follow-up: exact `a6b55e3a77f4ffbc4fe94a4fbf49f8d2d57b498f` sanitizes the local host/address and absolute workdir from `docs/reviews/independent-wire-review-4034f86-20260912.md`. The review result, tested revision, commands, tool versions, and finding boundaries are unchanged.
- Most recent developer source/test commit remains reachable exact `4034f86ceb356a3e04b8ff713b97338af3e71265` (`test: inline rejected negotiation close assertion`). Earlier reachable `dc52a5e38ea3cc1d113b00a1118a146bebbfd91a` closed the compile blocker; `4034f86` inlined the one-use close assertion. Accepted semantics remain: orderly EOF or `ConnectionReset` after rejected unsupported negotiation; emitted bytes and all other I/O errors hard-fail.
- No VPS/WAN experiment occurred in this sequence.
- Release truth remains unchanged: item 3 incomplete, item 4 incomplete, `RELEASE_CANDIDATE=false`, `PRODUCTION_READY=false`, `FREEZE=false`, `RELEASED=false`, D019 policy-blocked, and `READY_LIVE: none`.

## Reviewer actions this pass

### Historical quarantine alignment — ACCEPT / closed

Exact `8f029f3` truthfully marks pre-rebase `6f50d70` as repository-unreachable and quarantined. No accepted evidence now depends on that object.

### Wire-review provenance hygiene — ACCEPT / closed

Exact `a6b55e3` removes unnecessary local host/address/workdir detail while preserving the reachable tested revision, commands, tool versions, review result, and exclusions. This is reviewer-only hygiene and does not create a new CI wait loop.

### Stale PR #3 — CLOSED AS SUPERSEDED

PR #3 (`fix: harden preauth response send permits and deadlines`) was still open, non-mergeable, and based on old exact `01c876c`. It is now closed with an explanatory comment rather than merged/cherry-picked.

The underlying security goal was not rejected. Current `main` already contains later bounded response-I/O/permit ownership work through reachable commits including `61d69bd` (bounded pre-auth response I/O), `060f89e` (controller-bound response permits), `e10a846` (suppressed response settlement), and `81edd7b` (endpoint-rebind response accounting), with later exact-tree/review evidence. The PR body’s EOF-vs-`ConnectionReset` test blocker is also closed by the later `dc52a5e` / `4034f86` source family.

Do not revive PR #3 wholesale. If current `main` later shows a concrete regression against the present pre-auth response ownership/deadline contract, use a fresh minimal current-main repair with the exact failing call path and regression.

## Reviewer verdict

### TCP close repair — ACCEPT / closed

Do not reopen the rejected-negotiation EOF/RST seam absent a new concrete failure. Reachable exact-tree developer-local validation and hosted cross-evidence already exist for the corrected source family.

### Evidence-integrity HIGH — CLOSED

The pre-rebase/unreachable item-4 provenance chain is no longer accepted as exact-tree evidence:

- current release/item-4 navigation uses reachable exact `4034f86` plus reachable review/index commits;
- pre-rebase labels including `1db7a97`, `7275062`, `c5d0b15`, `58b5d13`, `7711162`, `1be290d`, `6f50d70`, `eb5f0e5`, `f4f7f93`, and `8d7c147` remain quarantined / not accepted as exact-tree provenance;
- reachable `289fbad` carries the accepted provenance-repair full local gate, with hosted CI as separate cross-evidence;
- `4d4820e`, `aa4f607`, and developer follow-up `8f029f3` close residual historical-note wording/timestamp contradictions without rewriting the original execution claims;
- current accepted navigation no longer promotes `BrokenPipe` into the unsupported-negotiation contract.

Do not infer that historical local commands never ran; the closed finding was shared-repository reproducibility/provenance.

### Independent/bounded review support — no new repairable defect

Accepted reachable support remains:

- `docs/reviews/reachable-item4-review-4034f86-20260912.md`: developer-performed rebuilt process/crypto/CLI/Session/Carrier bounded review, no current defect found;
- `docs/reviews/independent-wire-review-4034f86-20260912.md`: independent bounded parser/allocation review, exact-current source findings anchored to reachable `4034f86`; bounded pinned fuzz found no crash/OOM and is saturation evidence only;
- earlier reachable independent reviews: `docs/reviews/independent-release-review-3ed596a-20260911.md`, `docs/reviews/independent-resource-abuse-review-f184cf2-20260911.md`, and `docs/reviews/independent-hy2-methodology-review-e5fefc1-20260911.md`.

The current dependency/safety factual report tied to pre-rebase `8d7c147` remains explicitly quarantined as provenance. Its inventory is not a release/security approval and does not itself create a new implementation task.

## Queue status — no pre-existing READY slice

There is currently no concrete dependency-ready local coding/review-support slice and no `READY_LIVE` question. Do not manufacture 6–10 filler tickets merely to satisfy nominal queue size.

However, do **one bounded proposal sweep before entering idle**. Do not poll/wake repeatedly.

### PROPOSAL SWEEP — required once

Inspect exact-current source/tests/release evidence and propose **1–3 concrete small candidates maximum** from only these categories:

- implementation correctness gap;
- release correctness/evidence contradiction;
- bounded regression/test gap;
- stale current-main behavior whose intended answer is already fixed by an existing spec/ADR/security invariant.

Each candidate must include all of:

1. exact owner file/API/call path;
2. current behavior/claim that is actually false, ambiguous, or untested in a way that matters;
3. the already-existing repository semantic/invariant that determines the answer without policy invention;
4. the smallest implementation/evidence repair shape;
5. a bounded positive/negative regression or other deterministic proof;
6. whether it would require wire/parser fuzz;
7. why it is not duplicate/already-sufficient work.

Classify each proposal for the next reviewer as one of:

- `ACCEPT_CANDIDATE`: all seven fields above are concrete and no stop condition is implicated;
- `DEFER_POLICY_OR_ENVIRONMENT`: answer depends on D019, security/load values, signing/SBOM policy, previous frozen release, missing IPv6/live environment, benchmark-value choice, production/third-party permission, or another external decision;
- `REJECT_FILLER`: framework/checker/schema/docs cleanup without a real contradiction, already-sufficient question, unchanged WAN retry, speculative Experimental Track work, or merely “more tests would be nice”.

Do **not** implement a proposal that needs a reviewer/maintainer value judgment. If one candidate is obviously a current correctness/security/evidence defect whose answer is already fixed by current semantics, you may prepare the exact regression/repair plan, but wait for the next reviewer pass to disposition it unless an existing handoff already explicitly pre-authorizes that exact class.

If no candidate satisfies `ACCEPT_CANDIDATE`, record `PROPOSAL_SWEEP: no dependency-ready candidate` once and stop. Do not create a proposal document solely to say none exists; a concise developer checkpoint/handoff-visible note is sufficient if the agent has an existing checkpoint surface.

## Remaining gates / real stop conditions

- **item 4 independent judgment:** developer/self-review support is not an independent maintainer/security approval;
- **D019:** source-retention/no-reset policy remains a maintainer policy/value decision;
- **RSEC-001 suitability:** representative adversarial-load/capacity-suitability evidence remains unestablished; selecting meaningful pressure/capacity conditions may require maintainer judgment and may exceed ordinary bounded authorization;
- **item 3/environment:** natural-loss evidence remains incomplete; IPv6 remains environment-blocked; retained HY2/periodic/repeated-failover negatives stay frozen absent a materially new hypothesis; live PMTUD remains implementation-blocked but is not a default TODO absent an observed-problem gate;
- **release engineering policy:** signing/key custody/SBOM/publication trust and previous-frozen-release interoperability remain separate policy/dependency facts;
- **release authority:** RC, freeze, release and production readiness remain explicit decisions.

## Live/release boundary

- `IMPLEMENTATION_COMPLETE=true` only in the bounded research/governance sense;
- item 3 incomplete;
- item 4 incomplete;
- `RELEASE_CANDIDATE=false`;
- `PRODUCTION_READY=false`;
- `FREEZE=false`;
- `RELEASED=false`;
- D019 remains policy-blocked;
- `READY_LIVE: none` remains authoritative.

Do not rerun unchanged HY2, repeated warm failover, periodic/soak, package lifecycle, migration-back, endpoint migration, key update, IPv6, PLPMTUD, or Experimental Track work merely because the VPS remains rented. Standing authorization remains valid for a future materially changed dependency-ready self-owned TCP/UDP question; this handoff creates no such live question.

Resume continuous implementation only when repository truth yields a new dependency-ready defect/question, the bounded proposal sweep yields an `ACCEPT_CANDIDATE` that the reviewer accepts, or a maintainer decision unlocks one of the blocked gates above.