# ChatGPT reviewer handoff — item-4 checkpoint accepted; multistream rejection flake is READY_LOCAL

## Reviewed state

- Previous reviewer-owned handoff: exact `8cfda2240c06aebd18f052c26d07ab655b9abf66` (`docs(handoff): accept benchmark scope fix and force exact-tree closure`).
- Previous reviewed developer tree in that handoff: exact `287e2223d40b7b43dfa2a14ba1992e3bf845a0f2` (`docs: bound benchmark result schema scope`).
- Current developer-owned head reviewed this turn: exact `e96b9c1be84dbafb78010dca30cb7e3d5f5ada1a` (`docs: index independent item-4 checkpoint`).
- New developer-owned commits since the previous reviewer handoff:
  1. `bc2bd7ef14e91897bd43f87c734e15fde448942f` — docs/provenance closure for exact `287e222` plus factual release/index reconciliation;
  2. `26aa2e130e8455cd494e58f0fd352b389c1fdedb` — bounded independent item-4 evidence challenge at exact `bc2bd7e`;
  3. `e96b9c1be84dbafb78010dca30cb7e3d5f5ada1a` — index that checkpoint and sanitize unnecessary local host/workdir metadata from the current tree.
- These three commits do not change runtime implementation, Session/Carrier/ACK semantics, wire/crypto framing, package/install behavior, canonical fixtures, or VPS/WAN evidence.

## Review verdict

### `bc2bd7e` — ACCEPT

The comparison-schema package is now exact-tree locally closed. Persisted developer-local provenance records a clean detached checkout of exact `287e222` with all required focused comparison/schema tests, `PYTHONDONTWRITEBYTECODE=1 bash scripts/check.sh`, `git diff --check`, and clean-tree verification passing. The note records UTC `2026-09-11T00:14:15Z -> 00:16:13Z`, Linux x86_64, Rust `1.98.0`, and distinguishes this from reviewer-executed or GitHub-hosted CI.

The structural/common-envelope repair and documentation scope correction remain bounded exactly as requested: `nekomusume.benchmark-result.v1` covers the controlled complete-comparison paths that emit it; the blocked-result contract remains separately versioned and fail-closed; deterministic recovery and privileged netns keep their separate v0 result contracts; no historical artifact, live benchmark, or performance/superiority claim is manufactured.

### `26aa2e1` / `e96b9c1` — ACCEPT_WITH_BOUNDS

The bounded item-4 checkpoint is useful independent-review support. It challenges the current comparison schema/producer contract, v0 separation, canonical-corpus scope, package/archive/lifecycle evidence, Session/Carrier and non-policy pre-auth controls, HY2 blocked/complete/performance boundaries, and release-governance checks. Within that bounded scope it reports no current repairable correctness/security/evidence contradiction.

This checkpoint is **not** a full security audit, cryptanalysis, adversarial-load/capacity suitability result, WAN/VPS validation, penetration test, D019 decision, final independent release/security approval, RC, freeze, release, or production authorization. Item 4 therefore remains incomplete.

Exact `26aa2e1` initially recorded an unnecessary RFC1918 review-host address and absolute checkout path. Exact `e96b9c1` sanitizes those fields in the current tree and release-facing navigation. The historical commit still contains them; deleting that Git history would be a destructive rewrite and is not authorized by this handoff. They are not credentials or endpoint secrets, and no history rewrite should be attempted automatically.

## Cross-evidence correction: exact `287e222` did have hosted checks

The prior reviewer handoff said GitHub exposed no hosted status records for exact `287e222`. The legacy commit-status endpoint is empty, but GitHub **check-runs** show a Rust CI workflow on that exact SHA:

- nightly decode fuzz smoke: **success**;
- stable checks (`bash scripts/check.sh`): **failure**.

The stable job failed in `crates/neko-cli/tests/multistream.rs`, test `executable_rejects_unsupported_only_negotiation_before_noise_or_data`, because the test called `socket.read(...).unwrap()` and the peer close surfaced as Linux `ConnectionReset` instead of `Ok(0)` EOF. Later exact `bc2bd7e` and exact-current `e96b9c1` both have hosted `stable checks` and nightly fuzz **success**, while code/scripts are unchanged across the docs-only sequence. Developer-local exact `287e222` full gate is also green.

Therefore the hosted red is **not** a release blocker and hosted CI remains optional cross-evidence, not a wait condition. However it exposes a concrete nondeterministic process-test contract that can recur and should be repaired rather than dismissed.

## READY_LOCAL 1 — make unsupported-negotiation rejection terminal-close tolerant

Severity: **LOW/MEDIUM test reliability / evidence determinism**, not a runtime security finding.

### Exact problem

In `crates/neko-cli/tests/multistream.rs`, `executable_rejects_unsupported_only_negotiation_before_noise_or_data` sends a syntactically valid negotiation hello containing only an unsupported version, then requires the next `read()` to return exactly `Ok(0)`. The semantic contract is only that the server rejects before Noise or Session data. On Linux, a rejecting process can validly make the same terminal close observable as `ConnectionReset`; the exact `287e222` hosted run demonstrated that path. Treating that transport-close representation as test failure makes stable CI nondeterministic without strengthening the protocol assertion.

### Pre-authorized smallest repair

Keep the runtime semantics unchanged. Change only the negative process test (or a tiny test-only helper local to this file) so that after the unsupported-only hello:

- `Ok(0)` is accepted as a terminal close;
- `Err(e)` with `e.kind() == std::io::ErrorKind::ConnectionReset` is also accepted as the same terminal rejection observation;
- `Ok(n > 0)` remains a hard failure because the peer emitted bytes after the unsupported negotiation and may have entered a later protocol stage;
- any other error remains a hard failure unless current repository/runtime contracts independently justify it;
- the existing server process assertion must still require the uniform `neko: handshake rejected` failure and no success/record output.

Do **not** change the runtime merely to force FIN instead of RST, do not weaken the unsupported-version fail-closed behavior, and do not create a repository-wide socket-test framework for this one seam.

### Regression / validation

1. Run the single failing test repeatedly enough to exercise process-close timing variability (a bounded loop is sufficient; do not turn this into a stress framework).
2. Run the full `neko-cli` multistream test target.
3. On the final implementation/test SHA, run `PYTHONDONTWRITEBYTECODE=1 bash scripts/check.sh`.
4. Run `git diff --check` and verify clean exact tree after commit.
5. Persist concise developer-local provenance: exact SHA, commands, UTC start/end, exit codes, OS/arch, Rust stable version, clean before/after. No fuzz is required for a test-only close-semantics repair; if runtime/parser/wire code is changed unexpectedly, reassess fuzz requirements.
6. Commit and push. Do not wait for GitHub Actions. If hosted checks happen to run, they are additional cross-evidence only.

After this slice is green, continue immediately to REVIEW SUPPORT 2 without waiting for reviewer cadence.

## REVIEW SUPPORT 2 — final targeted item-4 contradiction sweep

The broad bounded checkpoint is already complete, so do **not** repeat another generic all-repository review or manufacture checker/harness cleanup. Perform one short exact-current proposal sweep aimed only at concrete contradictions that were not already exhausted by the checkpoint. A candidate is ACCEPT/ACCEPT_WITH_BOUNDS only if it names a specific owner file/API/call path, a presently false or ambiguous repository claim, an existing semantic answer, and a bounded positive/negative regression.

Prioritize at most these remaining technical seams:

1. process-test / operator-close determinism adjacent to the just-fixed unsupported-negotiation path;
2. any current release-facing claim that still confuses developer-local exact-tree evidence, hosted CI, independent-review support, live WAN evidence, or performance conclusions;
3. any concrete item-4 evidence defect revealed by the current exact tree that is repairable without policy invention.

Explicitly REJECT as filler: another broad ledger sweep, generic checker/parser framework, schema normalization without a real producer/consumer contradiction, new benchmark infrastructure without a current question, or repeated docs-only reclassification with no changed fact.

If one concrete defect is found, immediately do smallest repair -> bounded regressions -> commit/push -> clean exact-tree local gate -> concise provenance, then continue. If none is found, proceed directly to CHECKPOINT 3.

## CHECKPOINT 3 — coding queue exhaustion / remaining release obstacles

After READY_LOCAL 1 and the narrow sweep, classify remaining obstacles without inventing coding work:

- **independent review depth:** item 4 still needs genuine independent security/release judgment beyond bounded support notes;
- **D019 policy/value decision:** source-retention/no-reset policy remains maintainer-blocked; do not invent TTL/LRU/history policy;
- **RSEC-001 suitability:** representative adversarial-load/capacity-suitability evidence is still not established; conditions/authority requiring maintainer judgment are not pre-authorized as pressure testing;
- **release item 3 / environment evidence:** item 3 and natural-loss evidence remain incomplete; IPv6 remains environment-blocked; current HY2/periodic/repeated-failover lines remain frozen at their retained negatives unless a materially new hypothesis appears;
- **release/production authority:** RC, production readiness, freeze and release remain separate explicit decisions.

If no new concrete repairable defect remains, the **coding queue is genuinely exhausted**. Stop expanding implementation/checker/harness work rather than entering a watcher loop. The next real work is independent review, policy, environment evidence, or an explicit release/security decision.

## Exact-current release/live boundary

Current repository truth remains:

- `IMPLEMENTATION_COMPLETE=true` only in the repository's bounded research/governance sense;
- release item 3 remains incomplete;
- release item 4 remains incomplete despite several bounded independent-review support notes;
- `RELEASE_CANDIDATE=false`;
- `PRODUCTION_READY=false`;
- `FREEZE=false`;
- `RELEASED=false`;
- D019 remains `SOURCE_RETENTION_POLICY_BLOCKED`;
- RSEC-001 still lacks representative adversarial-load/capacity-suitability evidence and final independent release/security decision;
- `READY_LIVE: none` remains authoritative.

Do not rerun unchanged HY2, repeated warm failover, periodic/soak, package lifecycle, migration-back, endpoint/source migration, key update, IPv6, PLPMTUD, or Experimental Track work merely because the VPS remains rented. Standing authorization remains valid for a future materially changed dependency-ready self-owned TCP/UDP question, but this handoff creates no such live question.

## Honest rolling queue

1. **READY_LOCAL:** fix the unsupported-negotiation process-test EOF/RST nondeterminism; focused repeated regression + full exact-tree local gate + provenance.
2. **REVIEW SUPPORT:** one narrow contradiction sweep adjacent to that fix and current release-evidence boundaries; immediately repair any concrete existing-semantics defect.
3. **CHECKPOINT:** classify remaining item-3/item-4/D019/RSEC-001/environment/release-authority obstacles.
4. **CONDITIONAL READY_LOCAL:** any concrete defect found in step 2 with an existing semantic answer is pre-authorized for smallest-fix closure.
5. **REAL STOP:** if no concrete defect remains, coding queue exhaustion is real; do not poll. Escalate only D019 policy, destructive history/canonical migration, core Session/Carrier/ACK/crypto/wire architecture change, adversarial-load/benchmark conditions needing maintainer judgment, out-of-authorization/production/third-party actions, new credentials/server permissions, major security issue not safely adjudicable under existing semantics, repository breakage, or actual runtime/tool-budget exhaustion.
