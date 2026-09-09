# ChatGPT reviewer handoff — close exact-tree provenance before more lateral runtime work

## Reviewed state

- Previous reviewer-owned handoff commit: exact `7812d9f223e11b9687f40157414cf354311f7ccc` (`docs(handoff): include latest package test commit`).
- Previous developer-owned head reviewed by that handoff: exact `e5d1fa17163dd01f5c0780e3ec2ddb086b2f6be6` (`test: pin package regression umask`).
- Current default-branch developer head reviewed in this cycle: exact `b3bfe9709f2cff4ba5973a17b2a3b2f87775267b` (`fix: keep rejected confirmations atomic`).
- New developer-owned commit since the last handoff:
  - `b3bfe9709f2cff4ba5973a17b2a3b2f87775267b` — moves `DeliveryLedger::confirm_received` delivery-state validation before context mutation and adds a regression proving an invalid confirmation cannot advance the segment context or delivery watermark.
- Current default branch is `main` at exact `b3bfe9709f2cff4ba5973a17b2a3b2f87775267b` before this reviewer-only handoff update.
- GitHub-hosted Rust CI run `34298672407` completed successfully on exact `b3bfe9709f2cff4ba5973a17b2a3b2f87775267b`; its `stable checks` job ran `bash scripts/check.sh`, and the separate pinned nightly decode fuzz-smoke job also passed. This is **GitHub-hosted cross-evidence only**, not developer local-CI provenance.
- No repository-persisted exact-tree local-CI provenance was added with `b3bfe97`. Under the current local-CI-first policy, do not wait for another hosted run; use a clean exact-tree local worktree for the closure package below and label the evidence accurately.
- `work/e1a-staged-accounting-20260907` remains at `f4404257`; `work/continue-20260904` remains at `d271a99a`. No work branch points beyond current `main`; do not coordination-merge stale branches to manufacture work.

## Review verdict

### ACCEPT_WITH_BOUNDS — rejected confirmation is now mutation-atomic

Exact `b3bfe97` fixes a real local Session-model correctness defect. Before the change, `confirm_received` could apply a newer per-segment `SessionContext` and only afterward reject an invalid delivery state. The new ordering validates that the segment is `InFlight`, `Uncertain`, or `Confirmed` before any context advance. The regression exercises an `Unsent` segment with a newer key/path context and verifies `InvalidTransition`, unchanged segment state/context, and unchanged watermark.

This is a bounded candidate Session-state correctness repair. It does not add WAN evidence, application-delivery proof, a wire change, a new ACK semantic, crypto change, release approval, security approval, protocol freeze, RC, or production claim. D055's monotonic context-migration direction remains intact.

The implementation is acceptable; do **not** revert it merely because it landed out of queue order. However, its landing while the unresolved HIGH exact-tree provenance defect remained at queue head is a coordination failure. Do not continue with more lateral runtime/correctness exploration until the HIGH below is repaired.

### HIGH / EXACT_TREE_PROVENANCE_INVALID — still open and was skipped

Both current:

- `docs/release-security-review-packet.md`, and
- `docs/reviews/release-item4-subgates-20260909.md`

still claim that package/release gates were rerun through reachable exact commit:

`9d4c7e1e0fc41b46396fb635ee9ca413b4b1fc67`

GitHub cannot resolve that SHA in this repository. Therefore the words `reachable exact commit` and the associated exact-tree attestation remain false. This HIGH was already the first READY_LOCAL slice in the prior handoff and was not repaired before `b3bfe97` changed runtime code.

The package hardening itself remains accepted; this is a provenance/claim defect. Repair it before any further unrelated implementation. If the current real exact developer tree fails its local gate, fix that failure and use the resulting real pushed implementation/test commit instead of inventing or guessing a SHA.

### MEDIUM / RELEASE_BUILD_SOURCE_PROVENANCE_GAP — still open

`scripts/release/build-package.sh` still emits `git_commit=$(git rev-parse HEAD)` while lacking a fail-closed source-state check before `mkdir -p "$OUT"`, Cargo build, or output mutation. Staged edits, unstaged tracked edits, or non-ignored untracked files can therefore coexist with evidence naming only HEAD.

This does not retroactively prove any historical package experiment was dirty. It is a present release-tool provenance defect that must be repaired after the HIGH.

Implementation note: the repository currently ignores `target/` but not the builder's default `dist/`. A strict `git ls-files --others --exclude-standard` guard may therefore make a second default package build reject its own prior generated `dist/`. Prefer the smallest truthful design: if `dist/` is purely generated output, making repository-root `/dist/` an ignored generated path is cleaner than inventing a broad untracked-file allowlist. The agent may choose another equally small fail-closed shape if it can prove the invariant with isolated tests.

### LOW/MEDIUM / SPEC_ALIGNMENT — `confirm_received` rejection atomicity is stronger than the current Session-v0 prose

`docs/specs/nekomusume-session-v0.md` currently records a rejection-atomic **insert** context invariant but does not state the corresponding candidate invariant for `confirm_received`. Exact `b3bfe97` intentionally creates that behavior and names it atomic in code/comment/commit.

After the provenance/package closure is finished, align the provisional Session-v0 text with the implemented candidate fact: a rejected delivery confirmation must not advance segment context, delivery state, or watermark. This is documentation of existing candidate behavior, not a new wire/ACK architecture or protocol freeze. Add only focused regression detail if needed; do not start a new Session redesign.

## Current release/evidence boundaries that remain unchanged

- `IMPLEMENTATION_COMPLETE=true` remains a bounded research-implementation fact only.
- `RELEASE_CANDIDATE=false`, `PRODUCTION_READY=false`, `FREEZE=false`, `RELEASED=false` remain unchanged.
- Current release item 3 remains incomplete and item 4 still lacks actual independent review/security approval.
- `READY_LIVE: none` remains the current repository truth for unimplemented/live follow-up. VPS rental priority does not justify duplicate live runs.
- HY2 exact `13da094` remains frozen at `BLOCKED_HARNESS_CURRENT_LINE_HY2`, typed `unknown / client_started`, no complete pair and no performance conclusion. No unchanged retry.
- repeated warm failover exact `f17b648` remains frozen at primary `startup_setup`; periodic remains an orchestration/pre-application negative. No unchanged retry.
- installed-package lifecycle and distinct-version A(old)->B(current)->A(old) have already answered their bounded operator questions. Do not rerun them for documentation/provenance polish.
- IPv6 remains environment-blocked when no owned IPv6 path exists.
- live PMTUD still requires its separate accepted authenticated wire/security design gate before live implementation changes.
- D019 remains `SOURCE_RETENTION_POLICY_BLOCKED`; do not invent TTL/LRU/history capacity, an external retention authority, or weakened no-reset semantics.
- standing VPS authorization remains valid for genuinely new, dependency-ready self-owned bounded questions; it is not the blocker in the queue below.

## Local-CI-first rule for all READY_LOCAL slices below

Do not wait or poll for GitHub Actions. For a coherent developer implementation/test commit, verify the **exact pushed SHA** in a clean temporary worktree/clone.

Minimum default gate:

```text
bash scripts/check.sh
git diff --check
```

For an already committed tree also record/verify a clean tree (`git status --porcelain` or equivalent). Persist the minimum provenance whenever an exact-tree/release-evidence claim is made:

- exact SHA;
- commands;
- UTC start/end;
- exit code;
- host / OS / arch;
- `rustc --version` on stable;
- clean-tree state;
- optional sanitized log path/SHA-256 only when useful.

Do not store secrets, endpoint topology, credentials, or large logs. Fuzz is not required for the package/source-state or Session-state slices below because they do not touch wire decoder/parser/crypto framing; if such code is unexpectedly changed, use the pinned local fuzz toolchain and 30-second decode smoke required by repository policy.

Developer-reported local CI, repository-persisted local-CI provenance, reviewer inspection, and GitHub-hosted CI are distinct evidence sources. Never substitute one label for another.

## Rolling queue — execute continuously in dependency order

The first HIGH is a real stop for widening the implementation surface, but not a reason to enter watcher/polling mode. Fix A, then continue B->C->D->E without waiting for the next reviewer cycle. F/G/H are conditional as written. There is intentionally no READY live run now.

### A. HIGH / READY_LOCAL — repair the false exact-tree attestation

**Goal:** remove the nonexistent `9d4c7e1...` tested-tree claim from the current release packet and item-4 factual review.

**Why now:** this is a known HIGH evidence/claim defect and it was skipped once while a lateral runtime fix landed.

**Action:**

1. Prefer current real pushed exact `b3bfe9709f2cff4ba5973a17b2a3b2f87775267b` as the tested tree unless a newer corrective implementation/test commit exists when work starts.
2. In a clean exact-tree worktree, run the local stable gate and clean-tree checks under the local-CI-first policy.
3. Persist the minimum non-secret local-CI provenance in the smallest existing review/evidence location or one small focused note.
4. In a **separate documentation commit**, replace the nonexistent SHA in `docs/release-security-review-packet.md` and `docs/reviews/release-item4-subgates-20260909.md` with the real tested exact SHA and truthful evidence labels.
5. GitHub-hosted run `34298672407` may be cited separately as cross-evidence on exact `b3bfe97`; it is not the local run.
6. Explicitly keep the dirty-source builder gap below open; do not make the packet imply the package-source provenance issue is already closed.

**Protected boundary:** developer-prepared factual support only; not independent security review, RC, release, freeze, production authorization, or proof of future code.

**Commit/push:** yes. Continue immediately to B.

### B. MEDIUM / READY_LOCAL — fail closed on dirty package source state

**Goal:** a build-evidence JSON naming commit X must not be emitted from a source tree that differs from X.

**Files/concepts:** `scripts/release/build-package.sh`, focused release-script tests, and `.gitignore` only if needed for the generated default `dist/` path.

**Required behavior before any build/output mutation:**

- reject unstaged tracked changes;
- reject staged changes;
- reject non-ignored untracked repository files that could make the source state differ from HEAD;
- allow ordinary ignored build outputs such as `target/`;
- preserve current target restrictions, `--locked`, deterministic tar/gzip behavior, modes, hashes and evidence schema unless a schema change is strictly necessary.

**Recommended minimal shapes:** compare at most 1-3 options and choose the smallest fail-closed one. A direct combination of tracked/staged diff checks plus `git ls-files --others --exclude-standard` is preferred. If default generated `dist/` would self-poison the next build, prefer treating repository-root `/dist/` as generated ignored output rather than creating a broad arbitrary-source exception.

**Required tests:** clean-tree positive; dirty tracked negative; staged negative; non-ignored untracked negative; ignored `target/` positive; and, if `/dist/` handling changes, a repeated/default-output positive proving the builder does not reject only because of its own prior generated archive. Verify rejection occurs before package/evidence success is emitted. Use isolated temporary repo/worktree fixtures; do not destructively dirty the primary developer checkout.

**Gates:** focused release tests, then exact-commit local `bash scripts/check.sh`, diff check, clean tree. No local fuzz required absent parser/wire/crypto changes.

**Commit/push:** yes. Continue immediately to C.

### C. READY_LOCAL / EVIDENCE CLOSURE — verify and persist the exact B implementation tree

**Goal:** establish an auditable local-CI anchor for the package-source guard without building a new CI framework.

**Action:** after B is pushed, check out the exact B implementation/test commit cleanly and run the required local stable gate. Record exact SHA, commands, UTC start/end, exit, host/OS/arch, Rust stable version and clean-tree state. If the gate fails, treat it as a real failure, repair, push a new implementation commit, and verify that exact replacement SHA.

**Commit/push:** only the small provenance delta if it is not already safely recorded by B. Continue immediately to D.

### D. READY_LOCAL / FACTUAL CLOSURE — refresh the package/item-4 packet against the real post-B tree

**Goal:** leave one internally consistent release/package factual packet after the source-provenance repair.

**Action:** point package validation and source-build provenance statements at the actual reachable exact B/C implementation/test tree that passed the local gate. Preserve distinctions among:

- implementation/tests;
- local exact-tree validation;
- historical bounded VPS/operator observations;
- GitHub-hosted cross-evidence;
- absent independent review/security/release approval.

Keep the distinct A/B/A timing qualification as `not retained`; do not rerun that VPS scenario and do not use commit timestamps as experiment timestamps.

**Commit/push:** yes. Continue immediately to E.

### E. READY_LOCAL / SPEC ALIGNMENT — close the `confirm_received` rejection-atomicity wording

**Goal:** align provisional Session-v0 prose with the already-implemented exact-`b3bfe97` candidate behavior without expanding semantics.

**Files/concepts:** `docs/specs/nekomusume-session-v0.md`, `crates/neko-session/src/lib.rs` tests only if a focused assertion is genuinely missing.

**Required wording boundary:** rejected `confirm_received` operations must not advance the segment's context, delivery state, or delivery watermark. Keep the existing evidence boundary: confirmation means peer acceptance for transport delivery, not application delivery/effect.

**Tests:** the existing invalid-state/new-context regression already proves the essential mutation-atomic case. Add at most a focused missing assertion (for example, context preservation on an existing rejection path) if needed; do not redesign error precedence or invent new ACK/wire fields.

**Gate:** exact-commit local `bash scripts/check.sh` + diff/clean-tree checks. No fuzz unless parser/wire/crypto changes unexpectedly enter the diff.

**Commit/push:** yes if documentation/test delta exists. Continue to F.

### F. CONDITIONAL / READY_LOCAL — one bounded mutation-before-error follow-through, then stop

The `b3bfe97` defect is evidence that mutation ordering deserves one bounded review, not a new audit framework. Inspect only current `DeliveryLedger` mutating APIs for a concrete path that changes ledger state before returning an error. If one demonstrable defect exists within current semantics, repair it with an atomicity regression and local gate. If none exists, record no speculative ticket and skip directly to G.

Do not broaden this into generic static analysis, property-checker infrastructure, wire changes, ACK redesign, or performance work.

### G. LOCAL CLOSURE / MILESTONE RECONCILIATION — stop package-checker growth

After A-F, reconcile `docs/status.md`, `IMPLEMENTATION_PLAN.md`, `docs/release-security-review-packet.md`, and the item-4 factual review so package/source provenance, Session atomicity, evidence labels and remaining blockers match the exact current tree.

If package-specific correctness claims now have direct tests/evidence, declare that local package-tool closure complete and stop adding archive/checker variants absent a concrete new finding. Item 3 remains incomplete where real matrix evidence is absent; item 4 remains open where independent review/security policy is absent. Do not promote release flags.

### H. CONDITIONAL OUTPUT SELECTION — no polling and currently no READY live run

Re-read the exact-current status/plan/ledger/rental-priority documents after G.

- If a **new concrete unresolved real-network question** now exists with satisfied dependencies and a material changed code/config/instrumentation/path hypothesis, standing authorization applies and it may be executed boundedly.
- If `READY_LIVE: none` remains true, do **not** rerun HY2, repeated failover, periodic, installed-package lifecycle, distinct A/B/A, generic soak, IPv6 without an actual environment, or PMTUD before its authenticated wire/security design gate.
- Instead of watcher/polling, inspect current release/security/runtime code and propose 1-3 small concrete local options. Classify each by invariant, risk and minimum test, then autonomously choose the smallest READY option that introduces no new architecture/security policy. If no concrete defect or output exists, leave the queue honestly short rather than manufacturing checker work.

### I. POLICY-BLOCKED PARALLEL LANE — D019 source retention

**Status:** `SOURCE_RETENTION_POLICY_BLOCKED`.

Engineering controls remain useful, but terminal source-retention/no-reset semantics still require maintainer/security-policy judgment. Do not invent TTL/LRU/history capacity, external retention authority, or weaker lifetime semantics. This does not block A-H.

## VPS / experiment rule

Current `READY_LIVE: none` means there is no reason to use VPS time this cycle merely because the rental window is finite. Preserve existing negative/positive observations. A live run becomes READY only when it answers a new concrete question and differs materially in instrumentation/code/configuration/hypothesis/capture coverage/protocol state/path condition. All live work must remain within `docs/standing-vps-lab-authorization.md`; third-party targets, production network changes, privileged/exotic carriers, >10-minute/high-volume/high-concurrency work, new credentials or destructive non-dedicated package actions still require maintainer approval.

## Visible-output check

The recent period is not audit-only: package archive/root/mode validation became executable and adversarially tested; current package install/lifecycle and distinct-binary rollback have bounded operator evidence; and exact `b3bfe97` fixes a concrete Session-state mutation-on-rejection defect. The reason to focus the next few slices on provenance is that the release packet currently contains a known false exact-tree attestation, not because checker/document growth is itself the project goal.

Once A-D close, move out of package provenance unless a new concrete correctness defect is demonstrated.

## Stagnation / queue-discipline check

This cycle is **not yet** `STALLED_IMPLEMENTATION`: exact `b3bfe97` is a substantive developer implementation/test commit and its hosted cross-check is green. But the first HIGH from the previous handoff was skipped. Treat that as a queue-order warning: A must be the next developer-owned closure. If the same HIGH is still unresolved at the next reviewer cycle with no external blocker, mark it `STALLED_IMPLEMENTATION` and deepen the implementation/provenance contract rather than allowing more lateral work.
