# ChatGPT reviewer handoff — release facts accepted; run one final exact-current proposal sweep

## Reviewed state

- Previous reviewer-owned handoff: exact `9a42cc62ce64b44eb5d417d39621498249855ea6` (`docs(handoff): enter release review packet closure`).
- Current default `main` before this reviewer update: exact `d77be326cd038707160383b89dbff1b5611139ed` (`docs: record release review facts gate`).
- Two developer-owned commits landed after the previous handoff:
  1. `bf7b1d3964c510d08d93bfb922d004a2539760e7` — docs-only release-facing factual reconciliation across `docs/release-security-review-packet.md`, `docs/reviews/release-item4-subgates-20260909.md`, and the Session row in `docs/status.md`;
  2. `d77be326cd038707160383b89dbff1b5611139ed` — provenance-only note recording the clean exact-tree local gate for `bf7b1d3`.
- No runtime implementation, tests, package mutation, fixture reconstruction, or real VPS/WAN experiment landed in this sequence.
- GitHub exposes no hosted status records for exact `bf7b1d3`. Hosted CI remains optional cross-evidence and is not a wait condition.
- This reviewer performed GitHub repository/source/evidence review only. No reviewer-executed local CI is claimed.

## Review verdict — release-facing factual reconciliation ACCEPT

The prior MEDIUM release-navigation drift is closed.

The packet and item-4 factual support now distinguish:

- developer-reviewed factual coverage through exact classification tree `78111e8ab55b37fefe2a6f3aa1e8c10bb84f2c09`;
- provenance-only commit `3ee4fd0b93c97cf0fc7727783bf94cc656711ddc`, which records the clean gate for that classification tree;
- the individual tested-tree anchors retained by the underlying local review notes.

The updated text does not pretend that one SHA ran every historical test and does not promote local engineering evidence into independent audit, RC, protocol freeze, release, public-listener approval, or production readiness.

The Session status row is also now appropriately bounded: the declared ledger question was reviewed, while complete Session/release validation remains unestablished.

Exact `bf7b1d3` has a persisted developer-local gate in `docs/local-release-review-facts-bf7b1d3-20260911.md`:

- `python3 scripts/check-era4-closure.py`: passed (`0` open-ready, `16` already-sufficient, `0` dependency-blocked);
- `bash scripts/check-plan-sync.sh`: passed;
- `bash scripts/check-status-evidence.sh`: passed;
- `bash scripts/check-release-boundaries.sh`: passed;
- `bash scripts/check-markdown-links.sh`: passed;
- `PYTHONDONTWRITEBYTECODE=1 bash scripts/check.sh`: exit `0`;
- `git diff --check`: exit `0`;
- UTC `2026-09-10T19:10:12Z -> 19:12:07Z`;
- Linux `6.8.0-137-generic` x86_64;
- Rust `1.98.0`;
- initial/final source tree clean.

This is developer-local validation of exact `bf7b1d3`; exact `d77be32` is only the later provenance-text commit and must not be described as the tested tree.

No new correctness/security BLOCKER or HIGH was found in the two-commit docs package.

## Exact-current release boundary remains unchanged

The repository is now genuinely at the end of the old local A-T closure queue:

- rolling Era-4 `OPEN_READY` rows: `0`;
- release item 3 remains incomplete;
- independent release/security item 4 remains incomplete;
- `RELEASE_CANDIDATE=false`;
- `PRODUCTION_READY=false`;
- `FREEZE=false`;
- `RELEASED=false`;
- D019 source retention remains `SOURCE_RETENTION_POLICY_BLOCKED`;
- current live opportunity remains `READY_LIVE: none`.

Standing VPS authorization remains valid, but it does not manufacture a live question. Do not repeat HY2, repeated warm failover, periodic/soak, package lifecycle, migration-back, endpoint migration, key update, IPv6, or PMTUD without a materially new dependency-satisfied question.

Known remaining release boundaries are not automatically coding tickets:

- D019 retention requires a policy decision: **DEFER / maintainer decision**;
- signing/key-custody/SBOM workflow would require release-policy choices: **DEFER**, do not invent policy;
- previous/current interoperability has no prior frozen release: **NOT APPLICABLE / DEFER**;
- native aarch64 evidence is not a blocker for the currently declared x86_64-only first-RC scope unless N6 changes;
- sustained/public/general reachability, natural-loss evidence and production/service-manager hardening are not presently `READY_LIVE` or a dependency-free local coding slice;
- Experimental Track work remains non-TODO absent observed-problem evidence.

## READY_LOCAL — one final exact-current proposal sweep, then either implement or stop coding expansion

The release-facts package is green. The external coding agent must **not** wait for another reviewer turn. Immediately perform the pre-authorized exact-current proposal sweep that was queued by the prior handoff.

Re-read current code/spec/status and propose at most 1–3 candidate local outputs. A candidate is admissible only if it has all of:

- a named owner file/API/call path;
- a demonstrated contradiction, correctness/security defect, or missing advertised behavior in the current tree;
- an existing semantic/invariant basis for the fix, with no new core policy invention;
- a bounded positive/negative regression or factual check;
- a clear stop condition;
- no core Session/Carrier/ACK/crypto/wire architecture choice;
- no D019 retention choice, production mutation, new credentials/server/third-party permission, or maintainer benchmark-value judgment.

Pre-adjudication remains:

- **ACCEPT** — concrete current correctness/security/operator defect with an unambiguous repair under existing semantics. Choose the smallest safe accepted item and immediately implement/test/commit/push it.
- **ACCEPT_WITH_BOUNDS** — demonstrable factual/evidence contradiction. Repair only the contradiction, preserve evidence boundaries, then continue.
- **DEFER** — requires a new core semantic, security-capacity, release-policy, D019, benchmark-value, production, or external-environment decision.
- **REJECT** — generic checker/parser/harness normalization, another broad Session/ledger/pre-auth sweep without a demonstrated defect, documentation beautification, Experimental Track feature creation, or unchanged WAN reruns.

Do not convert this proposal sweep into an open-ended static-analysis campaign. One bounded pass is enough. Existing closed lanes stay closed unless the current exact tree supplies a concrete reproducer or contradiction.

### If one proposal is ACCEPT / ACCEPT_WITH_BOUNDS

Continue in the same coding session:

1. implement the smallest coherent slice;
2. add focused positive/negative regression evidence;
3. commit and push the implementation/test tree;
4. run the required clean exact-tree local gate on the final developer SHA: `PYTHONDONTWRITEBYTECODE=1 bash scripts/check.sh`, `git diff --check`, clean tree;
5. run the pinned decode fuzz smoke only if the change actually touches wire decoder/parser/crypto framing or the reviewer explicitly requires it;
6. persist one concise provenance note with exact SHA, commands, UTC start/end, host/OS/arch, Rust stable version and clean-tree state;
7. reconcile only facts genuinely changed by that slice;
8. if the next proposal is independently ACCEPT and dependency-ready, continue immediately. Do not wait for reviewer acknowledgement.

### If no proposal is ACCEPT / ACCEPT_WITH_BOUNDS

This is **real coding-queue exhaustion**, not a watcher condition.

Stop coding expansion and leave the repository at the next genuine stage: independent maintainer/security review of release item 4 using `docs/release-security-review-packet.md` and its linked evidence. Do not create filler work and do not poll every 5/30 minutes waiting for a reviewer handoff.

Independent review is a true stage boundary but does **not** flip any release flag. New concrete findings from that review, if any, become the next dependency-ready local queue.

## Rolling queue

The honest queue is intentionally short because the rolling local overlay already has zero `OPEN_READY` rows. Do not fabricate 6–12 hours of work.

1. **READY_LOCAL:** execute the one exact-current 1–3 proposal sweep above now.
2. **CONDITIONAL READY_LOCAL:** if a proposal is ACCEPT / ACCEPT_WITH_BOUNDS, implement -> focused tests -> commit/push -> exact-tree local gate -> concise provenance -> factual reconciliation, then continue to any other independently accepted proposal.
3. **REAL QUEUE EXHAUSTION:** if no proposal qualifies, stop coding expansion and enter independent release/security review stage.
4. **POST-REVIEW CONDITIONAL:** if independent review later produces a concrete local correctness/security/evidence finding, that finding becomes queue head; repair HIGH/BLOCKER before any broader work.

## Stop/escalation conditions

Stop/escalate only for an unresolved BLOCKER/HIGH that cannot safely be repaired under existing semantics, a required core Session/Carrier/ACK/crypto/wire architecture change, destructive/canonical-meaning migration, action outside standing authorization, production impact, new credentials/server/third-party permission, benchmark conditions requiring maintainer value judgment, D019 policy decision, real repository breakage, runtime/tool-budget exhaustion, genuine queue exhaustion, or the independent-review stage boundary above.

Otherwise: coherent slice -> focused checks -> commit/push -> clean exact-tree local gate -> concise provenance -> immediately continue to the next explicitly pre-authorized dependency-ready slice.
