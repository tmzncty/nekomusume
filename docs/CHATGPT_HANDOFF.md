# Nekomusume ChatGPT Handoff

Checked at: 2026-08-31 01:30 Asia/Shanghai (ad-hoc first review)
Repository HEAD: `1f1b6430b1cb684d840412dc2e390275ebf6c2fc`
Previous checked HEAD: none (first formal reviewer handoff)

## What changed

The GitHub default branch currently ends at three governance/coordination commits added after the older implementation baseline:

- `3a95a29` — standing VPS lab authorization;
- `7f911a0` — wire the standing authorization into `AGENTS.md`;
- `1f1b643` — update the ready-to-run agent prompt to require/read the standing authorization.

The currently visible remote branches are documentation/candidate branches plus `main`; none exposes a newer implementation history than the current default branch. Therefore GitHub is not yet demonstrably the authoritative implementation state for the recent engineering work.

This is a coordination/repository-state finding, not a claim that recent local implementation work does not exist.

## Review verdict

**needs repair — repository reconciliation gate**

Do not start a new implementation slice until the external coding agent has reconciled its actual current local authoritative history with GitHub and pushed a branch/default-branch state that preserves both:

1. the latest real implementation/test/evidence history in the coding environment; and
2. the standing-authorization/governance commits now present on GitHub.

The reviewer cannot truthfully review implementation, tests, WAN evidence, packaging, RC gates, or Era closure that is not visible in GitHub.

## Evidence boundaries

- GitHub default branch HEAD is currently `1f1b643`.
- GitHub currently contains `docs/standing-vps-lab-authorization.md`, and `AGENTS.md`/`PROMPT.md` point agents to it.
- The reviewer has not observed a remote commit containing the recently reported advanced implementation state.
- Absence from GitHub does **not** prove absence from a local coding worktree.
- Until reconciliation, do not infer implementation completion from chat, commit-message summaries, or stale ROADMAP checkboxes.

## Half-Day Work Package

### Primary — Reconcile authoritative implementation history with GitHub

**Goal**

Make GitHub the real asynchronous handoff surface before any further feature work.

**Why now**

The reviewer and coding agent cannot safely coordinate while the coding environment and GitHub appear to have different authoritative histories. Continuing implementation now risks parallel histories, lost governance commits, stale reviews, and accidental force-push overwrite.

**Preconditions**

- Read `AGENTS.md`, `SECURITY.md`, and `docs/standing-vps-lab-authorization.md` from current GitHub `main`.
- Inspect the coding environment's current local HEAD, remotes, worktrees, and pending/untracked state.
- Treat local identity/secret material as out of scope for reading/copying/committing.

**Required behavior**

1. Fetch GitHub `main` and inspect divergence from the actual local authoritative implementation branch.
2. Preserve all real local implementation/test/evidence commits.
3. Preserve GitHub governance commits `3a95a29`, `7f911a0`, and `1f1b643` (or equivalent content if integration rewrites hashes through a deliberate merge/cherry-pick workflow).
4. Do **not** force-push one history over the other.
5. Resolve conflicts narrowly; do not redesign protocol behavior during reconciliation.
6. Push the reconciled history so the relevant implementation state is visible from GitHub.
7. If default-branch update is not appropriate in one atomic operation, push a clearly named integration branch and make the intended merge path explicit.

**Validation**

- Verify the pushed GitHub commit graph contains the advanced implementation history and the standing VPS authorization governance content.
- Verify `AGENTS.md` still requires `docs/standing-vps-lab-authorization.md`.
- Verify `PROMPT.md` still points external agents to the standing authorization and does not recreate per-run WAN authorization blockers.
- Verify no identity, secret, production credential, or private topology material was committed.
- Run the repository's normal validation gate if reconciliation changes tracked files beyond pure conflict resolution; otherwise at minimum run `git diff --check` and inspect the final diff/graph.

**Completion definition**

GitHub exposes the actual current implementation/review state and contains the standing authorization governance. The next reviewer run can identify a concrete previous HEAD -> current HEAD implementation delta without relying on chat.

**Do not expand into**

- new protocol features;
- RC gate implementation;
- new WAN experiments;
- benchmark reruns;
- history cleanup/rebase for aesthetics;
- deletion of old evidence branches unless separately justified.

### Follow-up 1 — Repository fact audit after reconciliation

After the reconciled history is pushed:

- inspect current HEAD and the delta from `1f1b643`;
- classify each newly visible commit as implementation / tests / docs / research / experiment / packaging;
- reconcile stale ROADMAP/IMPLEMENTATION_PLAN status only where code/tests/evidence prove completion;
- identify correctness/security/evidence blockers before selecting further work.

This is primarily reviewer work. The coding agent should not invent a new feature package before the reviewer has had a chance to inspect the reconciled GitHub state, unless an already-recorded release-blocking correctness defect must be fixed immediately.

### Follow-up 2 — Resume the latest genuine READY release/engineering slice

Only after repository reconciliation and review:

- consume the next `docs/CHATGPT_HANDOFF.md` package;
- execute Primary first, then dependency-ordered Follow-ups;
- use the standing VPS authorization directly for in-scope bounded WAN work;
- do not recreate `count/bytes/duration/port` or ordinary `40080/40081` approval as external blockers.

### Fallback

If the local advanced implementation history has been lost or cannot be reconciled safely:

- stop before destructive Git operations;
- preserve all refs/worktrees;
- push non-destructive backup refs/branches if safe;
- record the exact commit graph and conflict in GitHub-visible coordination material;
- require maintainer decision only if there are multiple irreconcilable candidate authoritative histories.

Do not guess which history is authoritative and do not discard commits.

## Completion gates

- GitHub contains the real current implementation history.
- Standing VPS authorization remains present and referenced by agent instructions.
- No force overwrite/lost commits.
- No secrets or identity files committed.
- A reviewer can compute a trustworthy GitHub delta for the next half-day review.

## Do not expand into

- third-party targets or scanning;
- production network changes;
- high-privilege/long-duration/high-volume experiments outside standing authorization;
- new experimental carriers merely because they are interesting;
- declaring RC/production readiness from unreconciled or chat-only evidence.

## Questions requiring maintainer decision

none, unless reconciliation reveals two genuinely conflicting candidate authoritative histories that cannot be safely resolved from repository facts.
