# Nekomusume ChatGPT Handoff

Checked at: 2026-08-31 07:57 Asia/Shanghai
Repository HEAD: `e794f3437acf9300ccb2e128c3656460b82f1a53`
Previous checked HEAD: `e794f3437acf9300ccb2e128c3656460b82f1a53`

## What changed

No external-agent implementation/test/experiment commits are visible on GitHub since the previous reviewer handoff.

The default branch reviewed for this cycle is still the reviewer coordination commit:

- `e794f34` — create `docs/CHATGPT_HANDOFF.md` and establish the repository-reconciliation gate.

The immediately preceding GitHub-visible changes remain governance/coordination only:

- `3a95a29` — add standing VPS lab authorization;
- `7f911a0` — wire that authorization into `AGENTS.md`;
- `1f1b643` — require the authorization in the ready-to-run agent prompt.

The currently visible remote branches remain documentation/candidate branches plus `main`; none exposes the recently reported advanced implementation history. GitHub therefore still cannot be treated as a complete review surface for those local-only claims.

The required governance files were re-read in this cycle. `AGENTS.md` correctly treats standing VPS authorization as a persistent permission source; `SECURITY.md` still forbids production promotion without security/benchmark closure; `ROADMAP.md`, `IMPLEMENTATION_PLAN.md`, `README.md`, `docs/carrier-architecture.md`, and `docs/specs/nekomusume-session-v0.md` still describe an older/provisional implementation state. Until the advanced history is reconciled into GitHub, that apparent documentation drift cannot be safely repaired from chat-only claims.

GitHub cannot expose an unpushed local worktree, so no claim is made here about local dirty/clean state beyond what is represented by pushed commits and files.

## Review verdict

**needs repair — repository reconciliation gate remains the only justified Primary slice**

There is no new correctness/security/evidence regression visible since the last handoff because there is no new external-agent implementation delta to review. The blocker is coordination integrity: GitHub still does not expose the authoritative implementation/test/evidence history that the reviewer is expected to inspect.

Do not start a new protocol/release/WAN slice before reconciliation. Otherwise two agents would be operating against different repository realities, defeating the GitHub-only handoff design.

## Evidence boundaries

- Reviewed GitHub default-branch HEAD: `e794f3437acf9300ccb2e128c3656460b82f1a53`.
- No newer external-agent commit is visible on `main` or the currently listed remote branches.
- `docs/standing-vps-lab-authorization.md` is present and remains authoritative for in-scope bounded self-owned VPS experiments.
- `AGENTS.md` explicitly says in-scope `40080/40081`, bounded TCP/UDP/WAN experiments and missing per-run parameters are not approval blockers.
- `SECURITY.md` still describes Nekomusume as a research prototype, not an audited production transport.
- `README.md`, `ROADMAP.md`, `IMPLEMENTATION_PLAN.md`, `docs/carrier-architecture.md`, and the v0 normative entry point visibly lag the advanced implementation state previously reported outside GitHub; do not repair them by trusting chat summaries.
- Absence of advanced commits on GitHub does **not** prove the local implementation is absent.
- Code existing locally, pushed implementation, tested behavior, real WAN evidence, and performance conclusions remain distinct evidence levels.

## Half-Day Work Package

### Primary — Reconcile authoritative implementation history with GitHub

**Goal**

Make GitHub the actual authoritative asynchronous handoff surface by reconciling the coding environment's real current implementation history with the current GitHub governance/reviewer history.

**Why now**

This is a prerequisite for meaningful review. Until it is done, the reviewer cannot independently verify Era/Milestone closure, release engineering, WAN evidence, RC gates, tests, or claim boundaries.

**Preconditions**

- Fetch current GitHub `main` including `e794f34`.
- Read `AGENTS.md`, `SECURITY.md`, `docs/standing-vps-lab-authorization.md`, and this handoff.
- Inspect the coding environment's actual local HEAD, refs, worktrees, remote-tracking refs, and tracked/untracked state without reading/copying/committing identity/secret material.

**Required behavior**

1. Identify the actual local authoritative implementation tip and its commit graph.
2. Compare it against GitHub `main`.
3. Preserve the full real implementation/test/evidence history.
4. Preserve the GitHub governance/reviewer content introduced by `3a95a29`, `7f911a0`, `1f1b643`, and the handoff file introduced at `e794f34` (hashes may change only through a deliberate merge/cherry-pick integration; content must survive).
5. Do **not** force-push one history over the other and do not drop commits for aesthetic history cleanup.
6. Resolve conflicts narrowly. Reconciliation is not permission to redesign protocol behavior, rewrite evidence, or update claims from memory.
7. Push the reconciled history to GitHub. Prefer a normal merge/integration path; if updating `main` atomically is unsafe, push a clearly named integration branch and make the intended path to `main` obvious in repository-visible history/metadata.
8. Leave `docs/CHATGPT_HANDOFF.md` read-only from the coding-agent side; do not edit this file during reconciliation.

**Files / areas likely involved**

- Git refs / merge history;
- `AGENTS.md`;
- `PROMPT.md`;
- `docs/standing-vps-lab-authorization.md`;
- any files with genuine merge conflicts between the local advanced chain and current GitHub governance commits.

Do not touch unrelated crates/specs merely to make the merge look tidy.

**Tests / validation**

- Confirm the pushed graph contains both advanced implementation history and standing-authorization/reviewer governance content.
- Confirm `AGENTS.md` still requires `docs/standing-vps-lab-authorization.md`.
- Confirm `PROMPT.md` does not recreate per-run WAN approval blockers.
- Confirm `docs/CHATGPT_HANDOFF.md` remains present and coding-agent read-only by convention.
- Confirm no identity file, secret, production credential, or private topology was committed.
- Run the repository's normal local verification gate if conflict resolution changes tracked code/spec behavior; for a pure history merge with no behavioral conflict, at minimum inspect the final diff/graph and run `git diff --check` plus any repository-prescribed cheap governance/link checks that apply.

**Completion definition**

GitHub exposes the actual current implementation/test/evidence chain and the standing VPS authorization/reviewer coordination content, with no lost history or secret material. A subsequent reviewer run can compute a trustworthy pushed delta and review engineering claims without relying on chat.

**Do not expand into**

- N1/N2/N4/RC implementation;
- new protocol features;
- new WAN experiments or benchmark reruns;
- status/checkbox rewrites based only on chat summaries;
- force-push/rebase solely for aesthetics;
- deletion of old candidate/evidence branches.

### Follow-up 1 — Make reconciliation evidence explicit

After the reconciled history is pushed, but before starting a fresh engineering slice:

1. record/verify the resulting authoritative GitHub HEAD and merge ancestry;
2. ensure the advanced commits are reachable from the intended review branch/default branch;
3. ensure governance files survived unchanged in meaning;
4. run the appropriate repository gates for any conflict-resolved tracked files;
5. push those results normally.

Do not edit `docs/CHATGPT_HANDOFF.md`; the reviewer will perform the next fact audit.

### Follow-up 2 — Wait for reviewer fact audit, then consume the next package

Once reconciliation is visible on GitHub, the next reviewer cycle will:

- classify newly visible commits as implementation / tests / docs / research / experiment / packaging;
- inspect the relevant code/tests/evidence rather than commit messages alone;
- check correctness/security/spec/evidence drift;
- reconcile stale status only when pushed facts justify it;
- issue the next genuine release/engineering work package.

Until that review lands, do not invent a new major feature package. A known release-blocking correctness/security defect may still be fixed immediately if it is independently evident from the reconciled repository.

### Fallback — Preserve histories if reconciliation is ambiguous

If the local advanced history and GitHub history cannot be safely reconciled in one pass:

- stop before destructive Git operations;
- preserve all refs/worktrees;
- create non-destructive backup refs/branches if needed;
- push a clearly named integration/backup branch when safe so the state is GitHub-visible;
- document the exact commit graph outside `docs/CHATGPT_HANDOFF.md` only if a normal Git branch/commit graph is insufficient;
- request maintainer decision only if repository facts reveal two genuinely competing authoritative histories and neither can be selected safely.

A merge conflict alone is not a maintainer decision; resolve ordinary textual conflicts using current security/governance precedence and implementation facts.

## Completion gates

- Actual advanced implementation history is GitHub-visible.
- Standing VPS authorization remains present and referenced by agent instructions.
- Reviewer handoff file remains present and coding-agent read-only by convention.
- No force overwrite or lost commits.
- No secret/identity/private-topology material committed.
- Any conflict-resolved tracked code/spec passes the relevant repository validation.
- A future reviewer can compute an honest previous-HEAD -> current-HEAD delta entirely from GitHub.

## Do not expand into

- third-party targets, scanning, or unauthorized network activity;
- production route/firewall/DNS/proxy/tunnel/qdisc changes;
- high-privilege, long-duration, high-volume, or high-concurrency experiments outside standing authorization;
- new exotic/experimental carriers merely because they are interesting;
- RC/production declarations based on local-only or chat-only evidence;
- speculative updates to stale roadmap/status files before the reconciled implementation is reviewable.

## Questions requiring maintainer decision

none.
