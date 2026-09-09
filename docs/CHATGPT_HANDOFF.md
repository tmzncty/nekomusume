# ChatGPT reviewer handoff — finish CLI truth and supersede package provenance

## Reviewed state

- Previous reviewer-owned handoff: exact `7765b8572b6343f3bde805aed6708c87c9fe4399` (`docs(handoff): close package chain and align CLI surface`).
- Developer state reviewed through default-branch exact `44f612bb09b823ccf9b8eed0a9d24c4a6676e633` (`docs: bound compatibility review claims`).
- New developer-owned sequence since the previous handoff:
  - `5e68ad6ee20654e5e3b1c8d62857b72d129d1870` — expands `--help` and JSON capability inventory to the executable canonical command surface and adds a focused help/JSON/unknown-command regression;
  - `ffa3916ed23d5bb4814566646192de06b869ded2` — records developer-local exact-`5e68ad6` native x86_64 package build/smoke evidence and refreshes release/item-4 anchors;
  - `44f612bb09b823ccf9b8eed0a9d24c4a6676e633` — bounds compatibility claims to current/current candidate negotiation and explicit unsupported/future rejection, while stating that no previous frozen release interoperability exists.
- GitHub combined-status lookup exposes no hosted status records for exact `5e68ad6` or current `44f612b`. Do not poll Actions or wait for quota. Local exact-tree verification is the normal closure path.
- Work branches remain non-authoritative and stale: `work/e1a-staged-accounting-20260907` is still exact `f4404257`; `work/continue-20260904` is still exact `d271a99a`. Do not coordination-merge either merely to manufacture work.

## Review verdict

### ACCEPT_WITH_REPAIR — CLI inventory alignment materially improved, but human `capabilities` is still incomplete

Exact `5e68ad6` fixes the largest operator drift: canonical executable commands are now present in `USAGE`, JSON `capabilities --json` includes the canonical dispatch inventory, legacy `failover-server|failover-client` aliases are called out in help, and a focused process test checks help + JSON coverage and unknown-command failure.

However `capabilities` **without** `--json` still prints only:

```text
commands research=client,server,probe experimental=health-observe,failover,multistream fixtures=scheduler-fairness,key-update
```

while the same command's JSON report and actual dispatch also include `periodic-server`, `periodic-client`, `lab`, `workload`, `endpoint-rebind-server`, `endpoint-rebind-client`, `keygen`, and `capabilities`. Because `USAGE` describes `capabilities [--json]` as the secret-free build/command/default/limit report, the human report is still an operator-visible command-inventory contradiction.

Classify this as **MEDIUM / CLI_HUMAN_CAPABILITIES_DRIFT**. It is not a wire/Session/security architecture problem. Fix the existing surface rather than inventing a large CLI framework. The Agent may choose the smallest fail-closed shape: make human capabilities exhaustive over the intended canonical inventory, or use a compact shared descriptor if that reduces duplication without adding policy. Legacy aliases should remain clearly aliases rather than duplicate canonical capability entries.

Minimum regression: every intended canonical command must appear in `--help`, JSON capabilities, and human capabilities; retained aliases must be explicitly documented; unknown command must continue to fail.

### ACCEPT_WITH_BOUNDS — exact-current native package execution exists, but retained local-CI provenance is incomplete under the new local-CI-first policy

Exact `ffa3916` records a real developer-local native x86_64 archive built from exact `5e68ad6`, archive/binary hashes, packaged `capabilities --json` execution under `smoke-package.sh`, exit `0`, host/arch/Rust, and clean source checkout before/after. This closes the earlier behavioral gap: a package actually produced from the current implementation was executed locally rather than only exercising the synthetic non-native fixture.

Do **not** discard that evidence, but do not describe it as fully retained local-CI provenance either. The note records only `start/end date: 2026-09-09 UTC`, not distinct start/end UTC timestamps, and its command chain does not explicitly retain `git diff --check` even though the later item-4 text says that command passed. There is no contradictory failure evidence; this is a provenance-retention gap, not a package implementation failure.

Classify this as **MEDIUM / LOCAL_PACKAGE_CI_PROVENANCE_INCOMPLETE**. Do not fabricate timestamps from Git commit time and do not rerun VPS work. Because Slice A changes the CLI implementation anyway, supersede this evidence once on the new exact implementation SHA with the full local-CI-first record required below.

### ACCEPT — bounded compatibility/negotiation wording is now truthful

Exact `44f612b` correctly distinguishes current/current candidate negotiation from nonexistent previous-release interoperability. Current `neko-wire` code/tests support the bounded facts now stated: negotiation is required before `admit_data`; compatible peers select a common version; no-overlap/future-only offers reject terminally; an unsupported future selected response rejects; duplicate/late behavior is deterministic; and the exact accepted hello/response/selected version are retained in the authenticated binding. The distinct package A/B/A rehearsal is correctly not treated as protocol-version interoperability evidence.

This is implementation/test review support only. It is not a protocol freeze, independent security review, or compatibility lifetime policy. Do not add a speculative previous/current checker until a prior frozen release exists.

## Current evidence/governance boundaries

- `IMPLEMENTATION_COMPLETE=true` remains bounded research-implementation status only.
- `RELEASE_CANDIDATE=false`, `PRODUCTION_READY=false`, `FREEZE=false`, and `RELEASED=false` remain unchanged.
- Release item 3 remains incomplete. Item 4 remains incomplete because independent review and D019 policy closure are absent even though developer factual support is increasingly complete.
- `READY_LIVE: none` remains current opportunity truth. Do not manufacture VPS work merely because the rental window is finite.
- HY2 exact `13da094` remains `BLOCKED_HARNESS_CURRENT_LINE_HY2` at typed `unknown / client_started`; no complete pair/performance conclusion and no same-class retry.
- repeated warm failover exact `f17b648`, periodic current line, installed-package lifecycle, and distinct-package A->B->A already retain their bounded current-line answers/negatives. No unchanged retry.
- IPv6 remains environment-blocked when no owned IPv6 path exists.
- live PMTUD still requires its separate accepted authenticated wire/security design gate before live integration. Do not infer permission from the old `BLOCKED_IMPLEMENTATION` label.
- D019 remains `SOURCE_RETENTION_POLICY_BLOCKED`. Do not invent TTL/LRU/history capacity, retention authority, or weaker no-reset semantics.
- Signing/key custody/SBOM/publication trust remain separate release concerns; do not fabricate credentials or policy as ordinary READY_LOCAL work.
- Standing VPS authorization remains valid for genuinely new dependency-ready self-owned bounded questions. Authorization is not the current blocker.

## Local-CI-first rule

For coherent READY_LOCAL implementation/test commits, do not poll or wait for GitHub Actions. Verify the **exact pushed implementation SHA** in a clean temporary worktree/clone.

Minimum default gate:

```text
PYTHONDONTWRITEBYTECODE=1 bash scripts/check.sh
git diff --check
git status --porcelain   # must be empty
```

Persist minimal non-secret provenance when the result anchors release evidence: exact SHA, exact command(s), **distinct UTC start and end timestamps**, exit code(s), host/OS/arch, stable Rust version, initial/final clean-tree state, and archive/binary hashes when package identity is the subject. A small sanitized log hash/path is optional. Label developer-local CI, reviewer-executed checks, and hosted CI separately.

CLI/package metadata work does not require fuzz. If a later slice changes wire decoder/parser/crypto framing, use the pinned toolchain from `scripts/fuzz-toolchain.sh`, `cargo fuzz build decode`, then the 30-second / 8192-byte decode smoke required by repository policy.

## Rolling queue — execute continuously in dependency order

There is no correctness/security BLOCKER/HIGH. Finish A -> B -> C without waiting for another reviewer cycle. D selects the next real local output; E is conditional live work only if repository truth opens a genuinely new question; F is the policy-blocked parallel lane. There are intentionally fewer than ten tickets because the current tree does not contain ten honest dependency-ready release tasks; do not invent checker/docs work to hit a count.

### A. MEDIUM / READY_LOCAL — finish the human + JSON CLI capability contract

**Goal:** remove the remaining contradiction between `--help`, human `capabilities`, JSON `capabilities --json`, and actual dispatch.

**Why now:** exact `5e68ad6` fixed JSON/help but left the human capability report stale. The next package smoke should anchor a fully coherent operator surface.

**Files/concepts:** `crates/neko-cli/src/main.rs`, focused CLI process tests.

**Protected invariants:**

- no Session/Carrier/ACK/crypto/wire behavior change;
- no new numeric security policy or maturity policy;
- no hidden command becomes production-approved merely because it is discoverable;
- JSON remains secret-free and machine-readable;
- aliases remain aliases, not duplicate canonical commands.

**Implementation shape:** the Agent may compare 1-3 small forms, then choose the least duplicated form. A minimal exhaustive human summary is sufficient; a compact descriptor shared by help/capabilities is acceptable if smaller overall. Do not build a generic CLI framework.

**Tests:** canonical commands covered in help + human capabilities + JSON capabilities; retained aliases documented/working; unknown command fails closed.

**Validation/commit:** commit the coherent implementation, push it, then run the exact pushed SHA through the normal local gate in a clean temporary checkout. Continue immediately to B.

### B. READY_LOCAL / EXACT-TREE NATIVE PACKAGE + COMPLETE LOCAL PROVENANCE

**Goal:** supersede the coarse exact-`5e68ad6` package note with one complete local-CI-first record on the exact A implementation SHA.

In a clean temporary checkout of exact A:

1. capture UTC start timestamp;
2. run `PYTHONDONTWRITEBYTECODE=1 bash scripts/check.sh`;
3. run `git diff --check` and require `git status --porcelain` empty;
4. run the existing external-temp reproducibility workflow and require equal archive hashes under its existing contract;
5. run `scripts/release/smoke-package.sh` on one **actual produced native x86_64 archive**, requiring packaged `capabilities --json` execution;
6. require final source tree clean;
7. capture UTC end timestamp and all exit statuses;
8. persist exact SHA, commands, timestamps, host/OS/arch, stable Rust, archive SHA-256, packaged binary SHA-256/capability success, and initial/final clean state.

Do not rewrite `docs/local-package-smoke-5e68ad6-20260909.md` to invent precision it did not retain; keep it historical and add/supersede with the new exact-tree note.

This remains developer-local release-tool evidence only: no VPS/WAN, signing, independent review, RC, release, or production claim.

**Commit/push:** yes for the small evidence note. Continue immediately to C.

### C. READY_LOCAL / FACTUAL RECONCILIATION — close the CLI/package lane

Refresh only stale facts in:

- `docs/release-engineering.md`;
- `docs/release-security-review-packet.md`;
- `docs/reviews/release-item4-subgates-20260909.md`;
- `docs/status.md` / `IMPLEMENTATION_PLAN.md` only if wording is actually stale.

Anchor executable CLI/package facts to the real exact A implementation SHA and B local evidence, never to the later docs commit. State human + JSON capability coverage truthfully. Preserve the distinction between developer local CI, historical VPS operator evidence, hosted cross-evidence, and absent independent security/release approval.

The exact `44f612b` compatibility wording remains valid unless A changes negotiation code (it should not). Previous/current interoperability remains inapplicable until a prior frozen release exists. aarch64 remains candidate-target evidence and is not a blocker for the currently declared x86_64-only first-RC scope.

After C, declare this CLI/package inventory/provenance lane locally closed unless a **new concrete defect** exists. Stop package-checker growth.

**Commit/push:** yes. Continue immediately to D.

### D. LOCAL OUTPUT SELECTION — choose the next concrete runtime/operator/release output, not a watcher

Re-read exact-current `docs/status.md`, `IMPLEMENTATION_PLAN.md`, `ROADMAP.md`, release packet, and Era-4 opportunity ledger after C.

If a concrete READY_LOCAL defect/output is already named, implement it. Otherwise the coding Agent should propose 1-3 **specific** options grounded in current code/evidence, each including observed gap/output, API/file owner, protected invariant, risk, minimum positive/negative test, and whether it changes architecture/security policy.

The Agent may autonomously select and implement the smallest option that:

- changes no core Session/Carrier/ACK/crypto/wire semantics;
- invents no security numeric policy;
- needs no destructive migration, new credentials, third-party permission, production mutation, or maintainer value decision;
- is a real runtime/operator/release capability or correctness repair rather than another generic checker/parser/doc framework.

Prefer visible output/closure. Do not reopen DeliveryLedger generic auditing, package archive variants, FEC/0-RTT/striping/multipath/exotic carriers, or previous-release compatibility without an observed problem.

If all defensible options require maintainer judgment, record the exact blocker rather than polling and continue any independent safe lane that still exists.

### E. CONDITIONAL VPS OUTPUT — only if a genuinely new live question opens

If D or later repository truth creates a **new concrete unresolved real-network question** whose dependencies are satisfied and whose code/config/instrumentation/path/hypothesis materially differs from retained evidence, execute it boundedly under standing authorization and preserve negative results.

If `READY_LIVE: none` remains true, run nothing merely to use rental time. In particular do not unchanged-rerun HY2, repeated failover, periodic, installed-package lifecycle, distinct A/B/A, generic soak, IPv6 without a real owned path, or live PMTUD before its authenticated wire/security design gate.

### F. POLICY-BLOCKED PARALLEL LANE — D019 source retention

**Status:** `SOURCE_RETENTION_POLICY_BLOCKED`.

Non-policy engineering controls already have bounded factual review support. Terminal source-retention/no-reset semantics still require maintainer/security-policy judgment. Do not invent retention TTL, LRU/history capacity, external authority, or weaken the existing no-reset requirement. This blocker does not prevent independent A-E work.

## Stop / escalation

Do not notify or stop for ordinary progress. Escalate only for a core Session/Carrier/ACK/crypto/wire architecture change; new security numeric policy; destructive/canonical-meaning migration; action outside standing authorization; production impact; new credentials/server/third-party permission; benchmark value judgment; unresolved major security issue; D019 policy decision; or entry into a genuinely new project stage.