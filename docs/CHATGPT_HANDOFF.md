# ChatGPT reviewer handoff — close CLI/package operator truth, then continue item-4 review

## Reviewed state

- Previous reviewer-owned handoff: exact `6ec460d64fc4ff6ecac1de95786b6e256cc122ae` (`docs(handoff): close source provenance and repair clean-chain integration`).
- Developer state now reviewed through default-branch exact `34a43092e189687d1c1711b4f5fd9bb54e373e2a` (`docs: record clean package closure`).
- New developer-owned sequence since that handoff:
  - `30aa239e134a3468ec8e7509f84531d939386748` — isolates the invalid failover-server test identity in a temporary path and removes it, instead of allowing the default `crates/neko-cli/neko-server.identity` to dirty the source checkout.
  - `0a81d99ceb9bd0f950f67e42fdc9e9ee6191c314` — moves the two-build reproducibility workflow outside the repository, adds `scripts/release/reproducibility-test.sh`, compares same-tree archive hashes, and requires the repository to remain clean.
  - `34a43092e189687d1c1711b4f5fd9bb54e373e2a` — records developer-run local exact-tree validation of `0a81d99` and refreshes the release/item-4 factual anchors to that real tested tree.
- Persisted developer local-CI evidence for exact `0a81d99` records `PYTHONDONTWRITEBYTECODE=1 bash scripts/check.sh && git diff --check && test -z "$(git status --porcelain)"`, UTC start/end, exit `0`, Linux/x86_64 host, stable Rust version, clean initial/final tree, and equal package archive SHA-256. Treat this as **developer local CI**, not reviewer-executed CI and not hosted CI.
- No hosted status is required for this closure. GitHub status lookup on the current docs head exposed no status records; do not poll Actions or wait for quota.
- Work branches remain non-authoritative: `work/e1a-staged-accounting-20260907` is exact `f4404257` and main is 54 commits ahead with that branch as merge base; `work/continue-20260904` is exact `d271a99a`, diverged with main 138 commits ahead and the work branch carrying one old unique commit. Do not coordination-merge either merely to manufacture work.

## Review verdict

### ACCEPT — local-CI identity residue is fixed

Exact `30aa239` changes the concrete failing test path rather than weakening the source guard. The invalid failover-server regression now passes an explicit temporary `--identity` path and removes it. Production default identity semantics are not changed and `neko-server.identity` is not broadly ignored.

The later exact-`0a81d99` developer local gate reports an initially clean and finally clean checkout and explicitly notes repeated focused execution without recreating `crates/neko-cli/neko-server.identity`. This closes the prior `LOCAL_CI_RESIDUE` finding for the tested tree.

### ACCEPT — guarded package reproducibility workflow is repaired

Exact `0a81d99` moves the documented `dist-a` / `dist-b` outputs into an external temporary directory and adds a focused reproducibility test to `scripts/check.sh`. The test performs two same-tree package builds, compares archive SHA-256 values and requires the source repository to remain clean.

This preserves the strict clean-source invariant instead of adding an arbitrary untracked allowlist. The package builder still checks source identity before querying build metadata or creating output.

### ACCEPT_WITH_BOUNDS — clean local-CI -> reproducible package chain is proven, but current native package execution smoke is not yet retained

The persisted exact-`0a81d99` local note proves the stable gate completed, the checkout stayed clean, and the two produced archives were reproducible in that environment. That is a real local release-tool closure.

However the previous handoff's whole-chain Slice C also required running `scripts/release/smoke-package.sh` on one archive **actually produced from the repaired exact implementation tree**. Current `scripts/release/smoke-package-test.sh` intentionally constructs a synthetic `aarch64-unknown-linux-gnu` archive and checks that native execution is skipped; it is excellent adversarial archive validation, but it is not execution of the exact-current native built package. Historical local/VPS package smokes predate the `0a81d99` CLI/package workflow and cannot substitute for exact-current artifact execution.

Classify this as `MEDIUM / CURRENT_NATIVE_PACKAGE_SMOKE_NOT_RETAINED`, not a package implementation failure and not a reason to rerun VPS work. Close it once, locally, after the next CLI implementation commit so the evidence is not immediately stale.

### MEDIUM / CLI_COMMAND_INVENTORY_DRIFT — help, capability metadata and dispatch disagree

Current `crates/neko-cli/src/main.rs` has three different views of the executable surface:

- `USAGE` advertises `server|client|probe|periodic-server|periodic-client|lab|failover-server|failover-client|endpoint-rebind-server|endpoint-rebind-client|workload|keygen|capabilities`;
- `capabilities --json` reports `client`, `server`, `probe`, `health-observe`, `failover`, `multistream`, `scheduler-fairness`, and `key-update`;
- `main()` actually dispatches all of those plus `lab`, `workload`, `periodic-server`, `periodic-client`, `endpoint-rebind-server`, `endpoint-rebind-client`, `keygen`, `capabilities`, and the `failover-server` / `failover-client` aliases.

At minimum, valid canonical commands such as `health-observe`, `multistream`, `scheduler-fairness`, `key-update`, and `failover` are executable but absent from `--help`, while capability metadata is not obviously exhaustive of the executable surface. Because package smoke treats secret-free capabilities as an operator/build report, this is a concrete discoverability/operator-contract drift rather than cosmetic docs polish.

Do not invent new security policy or maturity taxonomy merely to fix it. The external Agent may use the proposal protocol: compare 1-3 small shapes, then choose the one with the least duplicated command state and clearest canonical-vs-alias contract. A single typed/static descriptor table feeding help/capabilities/dispatch is acceptable if it stays small; a focused consistency regression without a refactor is also acceptable. Legacy aliases may remain aliases, but their status must be explicit enough that help/capabilities do not contradict actual dispatch.

### LOW / RELEASE_SCOPE_WORDING_DRIFT — aarch64 absence is currently described too much like a first-RC blocker

`IMPLEMENTATION_PLAN.md` N6 already states that the first RC target, if later approved, is `x86_64-unknown-linux-gnu` only and `aarch64` remains a candidate target. The current item-4 factual review nevertheless lists missing native aarch64 execution evidence among the remaining release boundaries without restating that scope.

Do not add an aarch64 build/VPS task merely to satisfy that sentence. During the next factual reconciliation, state that native aarch64 evidence is absent/candidate evidence and is **not a first-RC blocker under the currently declared x86_64-only target scope** unless a later maintainer scope decision changes N6.

### ACCEPT — bounded DeliveryLedger follow-through remains closed

The prior reviewer pass already inspected current `insert`, transition and repaired `confirm_received` mutation ordering and found no second demonstrable mutation-before-error defect within the current Session semantics. Current code still supports that conclusion. Do not reopen this as a generic static-analysis/property-checker lane. A different interpretation of the confirmed watermark would be Session/ACK design and requires a concrete failure or architecture decision, not opportunistic cleanup.

## Current evidence/governance boundaries

- `IMPLEMENTATION_COMPLETE=true` remains bounded research-implementation status only.
- `RELEASE_CANDIDATE=false`, `PRODUCTION_READY=false`, `FREEZE=false`, and `RELEASED=false` remain unchanged.
- Release item 3 remains incomplete. Item 4 remains incomplete: developer factual support exists, but full independent release/security closure and D019 policy closure do not.
- `READY_LIVE: none` remains the current opportunity truth. Do not manufacture VPS work merely because the rental window is finite.
- HY2 exact `13da094` remains `BLOCKED_HARNESS_CURRENT_LINE_HY2` at typed `unknown / client_started`, with no complete pair or performance conclusion. No unchanged retry.
- repeated warm failover exact `f17b648` remains frozen at the primary `startup_setup` orchestration boundary. Periodic remains a pre-application/orchestration negative. No unchanged retry.
- installed-package lifecycle and distinct-version A(old) -> B(current) -> A(old) already answered their bounded operator questions. Do not rerun them to repair local package/CLI evidence.
- IPv6 remains environment-blocked when no owned IPv6 path exists.
- live PMTUD still requires its separate accepted authenticated wire/security design gate before live integration. Do not interpret the old `BLOCKED_IMPLEMENTATION` label as permission to invent wire semantics.
- D019 remains `SOURCE_RETENTION_POLICY_BLOCKED`. Do not invent TTL/LRU/history capacity, retention authority, or weakened no-reset semantics.
- Signing/key custody/publication trust also remain separate release concerns and may require maintainer credentials/value decisions; do not fabricate them as ordinary READY_LOCAL implementation.
- Standing VPS authorization remains valid for genuinely new dependency-ready self-owned bounded questions. Authorization is not the current blocker.

## Local-CI-first rule

For coherent READY_LOCAL implementation/test commits, do not poll or wait for GitHub Actions. Verify the **exact pushed implementation SHA** in a clean temporary worktree/clone.

Minimum default gate:

```text
PYTHONDONTWRITEBYTECODE=1 bash scripts/check.sh
git diff --check
git status --porcelain   # must be empty
```

When persisting exact-tree/release evidence, retain only minimal non-secret provenance: exact SHA, command(s), UTC start/end, exit, host/OS/arch, stable Rust version and initial/final clean-tree state; archive/binary hashes when the package itself is the subject. A small sanitized log hash/path is optional. Label developer local CI, reviewer-executed checks and hosted CI separately.

Package/CLI metadata work does not require fuzz. If a later slice changes wire decoder/parser/crypto framing, use the pinned toolchain from `scripts/fuzz-toolchain.sh`, `cargo fuzz build decode`, then the 30-second / 8192-byte decode smoke required by repository policy.

## Rolling queue — execute continuously in dependency order

There is no open correctness/security BLOCKER/HIGH. Finish A -> B -> C without waiting for another reviewer cycle. D is a bounded item-4 review/repair lane; E selects the next real output if D is clean. F is conditional live work only if repository truth opens a new question. G is the policy-blocked parallel lane. Do not enter watcher/polling mode merely because `READY_LIVE` is empty.

### A. MEDIUM / READY_LOCAL — unify the truthful CLI command surface

**Goal:** make operator-visible help, secret-free capability metadata, and actual dispatch describe a coherent command contract.

**Why now:** this is a current executable/package operator defect, and the exact-current package smoke in B should exercise the repaired CLI rather than freeze evidence on the already-stale surface.

**Files/concepts:** primarily `crates/neko-cli/src/main.rs` plus focused CLI tests. Touch release docs only if the chosen command-inventory semantics need one sentence of explanation.

**Protected invariants:**

- no Session/Carrier/ACK/crypto/wire behavior change;
- no new numeric security policy;
- no previously hidden command becomes production-approved merely because it becomes discoverable;
- `capabilities --json` remains secret-free and machine-readable;
- legacy failover aliases may remain, but canonical commands and aliases must not silently contradict each other.

**Proposal protocol:** if more than one small shape is reasonable, compare 1-3 shapes. Prefer the least duplicated source of truth and the smallest new API/state. A compact command descriptor feeding both help and capabilities is acceptable; a smaller consistency repair is also acceptable if regression tests prevent drift.

**Minimum tests:** positive `--help` coverage for canonical executable commands; machine-readable capabilities coverage for the intended canonical inventory/scope; alias behavior where retained; unknown command still fails closed. Avoid a giant generic CLI framework.

**Validation/commit:** exact pushed implementation commit; developer local `scripts/check.sh`, `git diff --check`, final clean tree. Persist minimal local-CI provenance only if it will anchor B/C. Commit + push, then continue immediately to B.

### B. READY_LOCAL / EXACT-CURRENT NATIVE PACKAGE CLOSURE — execute one produced package locally

**Goal:** close the remaining exact-current local package evidence gap on the exact A implementation tree.

In a clean temporary worktree of that pushed SHA:

1. require the normal local gate and final clean tree (reuse A's exact-tree run if it is the same commit and already recorded truthfully);
2. run the repaired external-temp two-build reproducibility workflow;
3. require matching archive hashes under the existing same-source/toolchain/target/environment contract;
4. run `scripts/release/smoke-package.sh` on one **actual produced native x86_64 archive**, so its packaged binary/capabilities path executes rather than taking the synthetic non-native `execution skipped` branch;
5. record archive SHA-256, packaged binary SHA-256/capability success as available, exit status, target, and final clean source tree.

This is developer local release-tool evidence only. It is not VPS/WAN, signing, security approval, RC, release or production authorization. Do not add another package checker just to record it, and do not rerun historical VPS package experiments.

**Commit/push:** a small evidence note is appropriate if needed by the release packet. Continue immediately to C.

### C. READY_LOCAL / FACTUAL RECONCILIATION — close the package/CLI lane on the real tested tree

Refresh only stale facts in:

- `docs/release-engineering.md`;
- `docs/release-security-review-packet.md`;
- `docs/reviews/release-item4-subgates-20260909.md`;
- `docs/status.md` / `IMPLEMENTATION_PLAN.md` only if their current wording is stale.

Anchor CLI/package facts to the real exact A/B implementation/test SHA, never to a later self-referential docs commit. Preserve the distinction between implementation/tests, developer local CI, historical bounded VPS evidence, hosted cross-evidence, and absent independent release/security approval.

Also repair the aarch64 scope wording: current N6 makes x86_64 the first-RC target; missing native aarch64 execution remains candidate evidence, not a first-RC blocker unless that scope changes.

After this, declare the package checker/source/reproducibility/native-smoke lane locally closed unless a **new concrete defect** exists. Stop package-checker growth.

**Commit/push:** yes. Continue immediately to D.

### D. BOUNDED ITEM-4 REVIEW / FIX-ONLY-IF-CONCRETE — compatibility and negotiation policy

**Goal:** perform one bounded factual review of the release item-4 compatibility/negotiation subgate on the exact current tree, rather than creating another generic checker.

Inspect current version negotiation, canonical-vector mapping and relevant executable negative tests. Confirm only repository-supported facts such as current/current candidate acceptance and fail-closed unsupported/future-version behavior. There is no prior frozen release, so previous-release interoperability must not be fabricated as a first-release fact.

- If a **specific current implementation/test contradiction** is found, repair it with focused tests and the normal exact-tree local gate, then update the factual review.
- If no concrete defect is found, do not add a speculative checker/TODO; at most tighten the factual review wording and continue immediately to E.
- Do not freeze the global protocol, change negotiation wire bytes, or invent a compatibility lifetime policy in this lane.

### E. LOCAL OUTPUT SELECTION — choose the next real milestone/release output, not a watcher

After A-D, re-read exact-current `docs/status.md`, `IMPLEMENTATION_PLAN.md`, `ROADMAP.md`, the release packet and the Era-4 opportunity ledger.

If no ready defect is already named, the coding Agent should propose 1-3 **specific** local options grounded in current code/evidence, each with observed gap/output, invariant, files/API owner, risk, minimum positive/negative test, and whether it changes architecture/security policy. Classify each `ACCEPT`, `ACCEPT_WITH_BOUNDS`, `DEFER`, or `REJECT`, then autonomously implement the smallest accepted option if it needs no maintainer decision.

Prefer a runtime/operator/release capability or closure over another checker/doc parser. Do not reopen DeliveryLedger auditing, package archive variants, FEC/0-RTT/striping/multipath, or other Experimental Track items without an observed problem.

If all 1-3 defensible options require a maintainer value/policy decision, record the exact blocker instead of polling; continue any independent safe lane that still exists.

### F. CONDITIONAL VPS OUTPUT — only if a genuinely new live question opens

If, after E, the exact-current ledger exposes a **new concrete unresolved real-network question** whose dependencies are satisfied and whose code/config/instrumentation/path/hypothesis materially differs from a retained negative, execute it boundedly under standing authorization and preserve negative results.

If `READY_LIVE: none` remains true, run nothing merely to use rental time. In particular, do not unchanged-rerun HY2, repeated failover, periodic, installed-package lifecycle, distinct A/B/A, generic soak, IPv6 without a real owned path, or live PMTUD before its authenticated wire/security design gate.

### G. POLICY-BLOCKED PARALLEL LANE — D019 source retention

**Status:** `SOURCE_RETENTION_POLICY_BLOCKED`.

Engineering controls remain useful and the non-policy controls have bounded factual review support, but terminal source-retention/no-reset semantics require maintainer/security-policy judgment. Do not invent TTL/LRU/history capacity, external retention authority, or weaker lifetime semantics. This does not block A-F.

## Stop / escalation conditions

Default behavior is continuous: implement -> exact-tree local validate -> commit -> push -> next READY slice. Do not wait for hourly review after each commit.

Stop only for a real unresolved BLOCKER/HIGH, a core Session/Carrier/ACK/crypto/wire architecture choice, new security numeric policy, destructive/canonical migration, production impact, action outside standing authorization, new credential/server/third-party permission, benchmark-value judgment, actual repository/tool-budget breakage, D019 policy decision, or a genuinely new phase requiring maintainer scope. Otherwise propose/implement/review/repair continuously.
