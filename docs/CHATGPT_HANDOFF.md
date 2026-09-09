# ChatGPT reviewer handoff — close clean-source integration, then return to milestone work

## Reviewed state

- Previous reviewer-owned handoff: exact `e7f15fa7e00e6c5b1b47de3abf8aabf1bbca9cd2` (`docs(handoff): prioritize exact-tree provenance closure`).
- Previous developer implementation head reviewed there: exact `b3bfe9709f2cff4ba5973a17b2a3b2f87775267b` (`fix: keep rejected confirmations atomic`).
- Current developer-owned/default-branch head reviewed in this cycle before this reviewer update: exact `cb0b9970e1fd9a4541fd71363b494e7446751817` (`docs: close package source provenance`).
- New developer-owned sequence since the prior handoff:
  - `ad82ca5a98bf06b82c85e4c35fd43605e3e9ab1d` — replaces the nonexistent release tested-tree SHA with real exact `b3bfe97`, retains developer local-CI provenance, and labels hosted CI separately.
  - `188d5a5d7c0f87c5554b6673ad56284dfc7d7138` — adds a fail-closed package source-tree guard, ignores repository-root `/dist/` as generated output, and adds isolated staged/unstaged/untracked/ignored-output regressions to `scripts/check.sh`.
  - `bea2e124a5cbb58d873cae0215ad879f85a56879` — records developer local stable validation of exact `188d5a5`.
  - `cb0b9970e1fd9a4541fd71363b494e7446751817` — refreshes release/item-4 factual wording through exact `188d5a5` and aligns provisional Session-v0 prose with rejected-`confirm_received` mutation atomicity.
- GitHub-hosted Rust CI run `34303134340` succeeded on exact `cb0b997`; its stable job ran `bash scripts/check.sh` and its decode fuzz-smoke job also passed. This is **hosted cross-evidence only**. Do not make subsequent READY_LOCAL work wait for hosted Actions.
- Work branches remain stale: `work/e1a-staged-accounting-20260907` is still exact `f4404257`; `work/continue-20260904` is still exact `d271a99a`. Neither is ahead of main and neither should be coordination-merged to manufacture work.

## Review verdict

### ACCEPT — prior HIGH exact-tree provenance defect is closed

The false `9d4c7e1...` attestation is gone. Exact `ad82ca5` first anchored release/item-4 factual support to real reachable exact `b3bfe97` and retained a developer-run local-CI note with command, UTC interval, exit status, host/arch, stable Rust version and source-tree state. Later package work then moved the package/release tested-tree anchor forward again to real exact `188d5a5`.

The distinction remains correct: developer local CI, GitHub-hosted CI, historical VPS/operator evidence, developer-prepared factual review support, and actual independent security/release review are different evidence classes. No release/security/production flag is promoted.

### ACCEPT_WITH_BOUNDS — package dirty-source provenance guard is implemented and locally exercised

Exact `188d5a5` calls `scripts/release/check-clean-source.sh` before package metadata queries or output mutation. It rejects unstaged tracked changes, staged changes and non-ignored untracked files, while allowing ignored build output. `/dist/` is now explicitly generated output. The focused temporary-repository regression is part of `scripts/check.sh`.

Exact `bea2e124` records developer local validation of the exact `188d5a5` implementation tree (`PYTHONDONTWRITEBYTECODE=1 bash scripts/check.sh && git diff --check`, exit 0). Exact `cb0b997` then updates the release packet/item-4 factual review and release-engineering prose to describe the guard.

This closes the original source-provenance defect: a successful package evidence JSON naming commit X must no longer be produced while tracked or non-ignored source state differs from X. It does **not** by itself make the whole local-CI -> package workflow residue-free; two integration defects below remain.

### ACCEPT — `confirm_received` provisional spec alignment is now truthful

The provisional Session-v0 entry point now states the already-implemented candidate invariant from exact `b3bfe97`: a rejected `confirm_received` does not advance segment context, delivery state, or confirmed watermark. The evidence boundary still says transport-delivery acceptance is not application delivery/effect.

A bounded reviewer inspection of current `DeliveryLedger` mutators found no second demonstrable mutation-before-error defect that can be repaired without inventing new semantics. `insert` performs rejection checks before context/state commit; `transition` validates before mutation; `confirm_received` now validates the delivery state and context before mutation. Do not manufacture a generic static-analysis task from this. Any future question about whether the current watermark means max-confirmed-end versus contiguous-confirmed-prefix would be Session/ACK semantic design, not a safe opportunistic cleanup; defer unless a concrete failure/evidence requires it.

### MEDIUM / LOCAL_CI_RESIDUE — the authoritative local gate still dirties its exact-tree checkout

The developer local-CI note for exact `188d5a5` truthfully records that `scripts/check.sh` leaves:

`crates/neko-cli/neko-server.identity`

as an untracked generated identity. This is not a secret leak into Git because it was not committed, but it means the exact-tree local gate does not finish with a clean checkout under the current local-CI-first policy. It also conflicts operationally with the new package guard: a package build launched in the same checkout after `scripts/check.sh` will reject that non-ignored residue.

Do **not** solve this by broadly ignoring `neko-server.identity`. Identity/key residue inside the source checkout is exactly the sort of state a strict package source guard should continue to notice. Prefer test isolation: identify the test/CLI path that relies on the default identity filename, pass an explicit temporary identity path, and guarantee cleanup even on failure where practicable. Another equally small fail-closed test-only shape is acceptable.

### MEDIUM / RELEASE_REPRO_RECIPE_REGRESSION — the documented two-build reproducibility recipe now self-poisons

`docs/release-engineering.md` currently tells operators to build first with `OUT="$PWD/dist-a"` and then with `OUT="$PWD/dist-b"`. The new source guard scans all non-ignored untracked repository files before every build, but only repository-root `/dist/` is ignored. Therefore after the first successful `dist-a` build, the second invocation sees non-ignored generated `dist-a/` and should fail closed before building `dist-b`.

This is a real integration regression between the correct source guard and the documented reproducibility workflow. Do not weaken the source guard with a broad arbitrary untracked allowlist. Prefer a small truthful fix such as moving the documented reproducibility outputs outside the source tree (for example separate `mktemp -d` directories) or another narrowly generated-output design that preserves the invariant that non-ignored repository state must equal HEAD. Add only the minimum focused regression/validation needed to prove the chosen workflow.

These two MEDIUM findings belong to one coherent closure package: **local gate leaves clean tree -> clean tree can immediately enter package builder -> reproducibility recipe still works under the guard**.

## Current evidence/governance boundaries

- `IMPLEMENTATION_COMPLETE=true` remains bounded research-implementation status only.
- `RELEASE_CANDIDATE=false`, `PRODUCTION_READY=false`, `FREEZE=false`, `RELEASED=false` remain unchanged.
- Release item 3 remains incomplete; item 4 remains incomplete because actual independent security/release review and D019 policy closure are absent.
- `READY_LIVE: none` remains the current release-evidence opportunity truth. The VPS rental window does not justify duplicate live work.
- HY2 exact `13da094` remains frozen at `BLOCKED_HARNESS_CURRENT_LINE_HY2`, typed `unknown / client_started`, with no complete pair or performance conclusion. No unchanged retry.
- repeated warm failover exact `f17b648` remains frozen at primary `startup_setup`; periodic remains a pre-application/orchestration negative. No unchanged retry.
- installed-package lifecycle and distinct-version A(old)->B(current)->A(old) already answered their bounded operator questions. Do not rerun them to repair local package/docs provenance.
- IPv6 remains environment-blocked when no owned IPv6 path exists.
- live PMTUD still requires its separate authenticated wire/security design gate before live integration; do not treat the old `BLOCKED_IMPLEMENTATION` label as permission to invent wire semantics.
- D019 remains `SOURCE_RETENTION_POLICY_BLOCKED`. Do not invent TTL/LRU/history capacity, retention authority, or weakened no-reset semantics.
- Standing VPS authorization remains valid for genuinely new dependency-ready self-owned bounded questions; authorization is not the current blocker.

## Local-CI-first rule

For coherent READY_LOCAL implementation/test commits, do not poll or wait for GitHub Actions. Verify the **exact pushed implementation SHA** in a clean temporary worktree/clone.

Minimum default gate:

```text
bash scripts/check.sh
git diff --check
git status --porcelain   # must be empty after the gate once Slice A below is fixed
```

When an exact-tree/release evidence claim is persisted, retain only minimal non-secret provenance: exact SHA, commands, UTC start/end, exit, host/OS/arch, stable Rust version and initial/final clean-tree state. A small sanitized log hash/path is optional. Hosted CI is extra cross-evidence, never a substitute label for local CI.

Package/docs-only work does not require fuzz. If a later slice unexpectedly changes wire decoder/parser/crypto framing, use the pinned toolchain from `scripts/fuzz-toolchain.sh`, build `decode`, and run the 30-second / 8192-byte fuzz smoke required by repository policy.

## Rolling queue — execute continuously in dependency order

There is no correctness/security BLOCKER/HIGH now. The two concrete MEDIUM integration findings are first because they prevent a truthful clean local-CI -> package closure. Finish A -> B -> C -> D without waiting for another reviewer cycle. E is a bounded review/skip gate; F/G are output-selection/conditional lanes. Do not enter watcher mode merely because `READY_LIVE` is empty.

### A. MEDIUM / READY_LOCAL — make the stable local gate residue-free

**Goal:** exact-tree `scripts/check.sh` must finish without creating a non-ignored identity/key file or other source-checkout residue.

**Why now:** local exact-tree CI is the primary validation path, and the new package builder intentionally rejects the residue currently left by that path.

**Files/concepts:** the CLI integration test(s) or command path producing `crates/neko-cli/neko-server.identity`; test temp-path helpers/cleanup; avoid changing production identity semantics unless required.

**Implementation bounds:**

- identify the concrete producer of the default identity file;
- prefer explicit temporary `--identity` in tests and deterministic cleanup;
- do not globally ignore `neko-server.identity` merely to make `git status` quiet;
- do not delete arbitrary user/runtime identities from production paths;
- no Session/Carrier/ACK/crypto/wire change.

**Required validation:** focused producer regression, then exact pushed implementation commit in a clean worktree: `bash scripts/check.sh`, `git diff --check`, and final empty `git status --porcelain`. If a failure leaves a temp identity outside the repository temp area, fix the test isolation rather than adding an ignore rule.

**Commit/push:** yes. Continue immediately to B.

### B. MEDIUM / READY_LOCAL — repair the guarded reproducibility workflow

**Goal:** the documented two-build package reproducibility procedure must work while the source guard remains strict.

**Why now:** the current `dist-a` -> `dist-b` recipe is inconsistent with the new non-ignored-untracked rejection.

**Preferred shapes:** compare at most 1-3 small options. Prefer outputs outside the repository (for example two external temporary directories) or another narrow generated-output contract. Avoid broad `dist-*`/arbitrary path exceptions that could hide source state unless their scope is rigorously justified.

**Protected invariant:** every package build starts from repository state equal to HEAD apart from deliberately ignored generated paths; evidence `git_commit` remains truthful.

**Validation:** demonstrate two sequential same-tree builds under the documented/fixed workflow, compare archive hashes as before, and verify the repository remains clean. This may be a bounded manual exact-tree release validation if embedding two full release builds in every `scripts/check.sh` run would be wasteful. Do not build a new CI framework.

**Commit/push:** yes for script/docs/test changes. Continue immediately to C.

### C. READY_LOCAL / EXACT-TREE PACKAGE CLOSURE — prove the whole local chain

**Goal:** prove on the exact A/B implementation tree that local validation and package construction compose correctly.

In a clean temporary worktree of the exact pushed implementation SHA:

1. run `bash scripts/check.sh`;
2. require final clean `git status --porcelain`;
3. run the repaired same-tree two-build package reproducibility workflow;
4. record matching/nonmatching hashes truthfully (matching is expected only under the existing same-source/toolchain/environment contract);
5. run package smoke on one produced archive;
6. confirm source checkout remains clean except paths explicitly outside the checkout or already ignored by the existing generated-output contract.

Persist minimal local provenance only if useful for the release packet: exact SHA, commands, UTC interval, exit, host/OS/arch, stable Rust, archive hashes, final clean state. This is local release-tool evidence, not VPS, signing, security review, RC or production approval.

**Commit/push:** small evidence note if needed. Continue immediately to D.

### D. READY_LOCAL / FACTUAL RECONCILIATION — move package/release truth to the repaired exact tree

Refresh only what actually changed in:

- `docs/release-engineering.md`;
- `docs/release-security-review-packet.md`;
- `docs/reviews/release-item4-subgates-20260909.md`;
- `docs/status.md` / `IMPLEMENTATION_PLAN.md` only where their current package/local-CI wording is stale.

Anchor the package/local-CI facts to the real exact implementation/test SHA from A/B/C, not the later self-referential docs commit. Preserve the distinctions among implementation/tests, developer local validation, GitHub-hosted cross-evidence, historical bounded VPS observations, and absent independent review/security/release approval.

Declare the local package-source/tooling closure complete after this if no concrete defect remains. **Stop package-checker growth**; do not add more archive/checker variants without a specific observed correctness issue.

**Commit/push:** yes. Continue to E.

### E. BOUNDED REVIEW / SKIP-IF-CLEAN — close the one-time DeliveryLedger follow-through

The reviewer has already inspected the current `DeliveryLedger` mutation ordering and found no second demonstrable mutation-before-error defect within current semantics. Agent may perform one bounded confirmatory read/test pass only.

- If a concrete current API returns `Err` after a visible ledger mutation, repair it with a focused atomicity regression and exact-tree local gate.
- If none exists, **do not commit a speculative ticket**; record no generic checker and continue immediately to F.
- Do not reinterpret watermark semantics, redesign ACKs, add wire fields, or turn this into static-analysis/property-checker infrastructure.

### F. LOCAL MILESTONE RECONCILIATION — identify the next real output after package closure

Re-read exact-current `docs/status.md`, `IMPLEMENTATION_PLAN.md`, `ROADMAP.md`, the release packet, and the Era-4 opportunity ledger after A-E.

If package/local-CI closure is complete, keep it closed. Then inspect current release/security/runtime code for 1-3 **specific** small local options. For each proposal state:

- concrete observed gap or output;
- invariant protected;
- files/API ownership;
- risk;
- minimum positive/negative test;
- whether it changes architecture/security numeric policy.

Classify each `ACCEPT`, `ACCEPT_WITH_BOUNDS`, `DEFER`, or `REJECT`, then autonomously implement the smallest accepted option if it needs no maintainer decision. Do not wait for reviewer merely because the old queue ended.

Good options must close a real release correctness/runtime/operator gap or produce a user/system-visible capability; docs/checker-only proposals without a direct blocker should be deferred.

### G. CONDITIONAL VPS OUTPUT — only for a newly opened real-network question

After F, if the exact-current ledger exposes a **new concrete unresolved real-network question** with dependencies satisfied and a material change in code/config/instrumentation/path/hypothesis, execute it boundedly under standing authorization and preserve negative results.

If `READY_LIVE: none` remains true, run **nothing** merely to use rental time. Specifically do not unchanged-rerun HY2, repeated failover, periodic, installed-package lifecycle, distinct A/B/A, generic soak, IPv6 without environment, or live PMTUD before its wire/security design gate.

### H. POLICY-BLOCKED PARALLEL LANE — D019 source retention

**Status:** `SOURCE_RETENTION_POLICY_BLOCKED`.

Engineering controls remain useful, but terminal source-retention/no-reset semantics require maintainer/security-policy judgment. Do not invent TTL/LRU/history capacity, external retention authority, or weaker lifetime semantics. This does not block A-G.

## Stop / escalation conditions

Continue implement -> local exact-tree validate -> commit -> push -> next READY slice without waiting for hourly review. Stop only for a real unresolved BLOCKER/HIGH, core Session/Carrier/ACK/crypto/wire architecture choice, new security numeric policy, destructive/canonical migration, production impact, action beyond standing authorization, new credential/server/third-party permission, benchmark-value judgment, actual repository/tool-budget breakage, D019 policy choice, or genuinely exhausted safe queue.

No maintainer action is required for A-G as currently bounded.