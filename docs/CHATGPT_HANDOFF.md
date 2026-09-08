# ChatGPT reviewer handoff — reconcile lifecycle evidence provenance before further release claims

## Reviewed state

- Previous reviewer handoff: exact `ff1e8f8803715adc2015e9e45a9414e3a15a50c7` (`docs(handoff): repair lifecycle emission before VPS retry`).
- Current default `main` reviewed here: exact `dc90c5f265a93f113560ba3c97ab436638bdc307` (`docs: review exact-current release subgates`).
- New developer-owned commits since the previous handoff:
  - `7cebe6b34008fcd6aae096ca7cd3a579f9aa6f3b` — emit signal-driven `DRAINING` and lock TCP/UDP lifecycle ordering;
  - `e84770e487180e253cd98af52c4f303a081c01bc` — initially retain the changed-hypothesis installed-package lifecycle run as a TCP-readiness orchestration negative;
  - `c4ee406e73156345bdc5648a2ea68f5295bf5016` — rewrite that same experiment record as a complete TCP+UDP lifecycle success;
  - `dc90c5f265a93f113560ba3c97ab436638bdc307` — add developer-prepared release item-4 factual review support.
- Exact-head Rust CI is green for implementation exact `7cebe6b` (`34279192488`), evidence/doc exact `c4ee406` (`34279747083`), and current `dc90c5f` (`34283071038`). Green CI validates repository gates; it does not resolve contradictory experiment provenance by itself.
- No work branch is ahead of `main`. `work/e1a-staged-accounting-20260907` remains stale at `f4404257520e9a014ac4e785b0ab9a97f8aaf794`; `work/continue-20260904` remains older at `d271a99a2ab26abbcb146c411ba0fde697395abe`. Do not coordination-merge either.

## Review verdict

### ACCEPT — `7cebe6b` closes the local lifecycle-observability defect

The implementation repair is small and correctly scoped. Signal shutdown now transitions `READY -> DRAINING`, emits the actual lifecycle state, then transitions to and emits `STOPPED`. Both the UDP in-data shutdown path and the shared post-listener shutdown path reuse the helper. The process regression now requires ordered `READY < DRAINING < STOPPED`, exactly one `DRAINING`, exactly one `STOPPED`, and retains the existing same-address TCP/UDP rebind proof.

Keep the boundary: this is local implementation/process-test evidence. It changes no Session/Carrier/ACK/crypto/wire semantics and does not by itself prove installed-package VPS lifecycle behavior, daemon/service-manager behavior, production readiness, security approval, RC, or release.

### HIGH / EVIDENCE_PROVENANCE_CONFLICT — the exact-`7cebe6b` VPS lifecycle result is internally contradictory

The repository currently contains two consecutive, mutually incompatible descriptions of the **same experiment id / exact tested tree**:

1. exact `e84770e` says the changed-hypothesis run wrote only `start`, `package_smoke`, `installed_hash`, and `tcp-preclean`, then exited nonzero before `tcp-ready`; it classifies the run as `BLOCKED_ORCHESTRATION_CURRENT_LINE_PACKAGE_LIFECYCLE` and explicitly says no remote READY/SIGTERM/DRAINING/STOPPED/rebind/authenticated-exchange claim can be made;
2. exact `c4ee406`, two minutes later, rewrites the same evidence document and same experiment as a complete TCP and UDP success with a full phase record through cleanup.

Git history preserves both descriptions, but the current tree provides no provenance explanation for why the first retained negative was wrong, whether a second invocation occurred, or what inspectable source justifies replacing the prefix/nonzero account with a complete success account. Standing evidence rules require negative results to remain valid and prohibit pass-chasing/overwriting without a truthful changed-hypothesis/new-run boundary.

Therefore **do not currently treat the exact-`7cebe6b` VPS lifecycle success as accepted operator evidence**. The local `7cebe6b` implementation/tests remain accepted. The VPS lifecycle row is temporarily `EVIDENCE_PROVENANCE_CONFLICT` until reconciled from already-existing evidence. Do not rerun the experiment merely to obtain a cleaner answer.

This is an evidence/governance defect, not evidence of a runtime lifecycle failure.

### HIGH / EXACT_TREE_PROVENANCE — the new item-4 factual review targets a GitHub-unreachable SHA

`docs/reviews/release-item4-subgates-20260909.md` claims to be a factual review of exact `bb008d0302c17dcf34dac7c5b18259fd34ec495f` and claims `scripts/check.sh` / `git diff --check` passed on that exact tree. GitHub cannot resolve that commit, it is not current `main`, and it is not an ancestor between `c4ee406` and `dc90c5f`.

A local/unpushed or mistyped SHA cannot serve as the repository's exact-tree review provenance. The developer-prepared review remains useful as an outline, but its exact-tree claims are not accepted until rewritten against a real reachable commit and rerun/verified on that tree. Do not call this independent review or security approval.

### ACCEPTED CONTINUING BOUNDARIES

- RSEC non-policy engineering controls may remain bounded-reviewed, while full D019 remains `SOURCE_RETENTION_POLICY_BLOCKED`; do not invent TTL/LRU/history capacity or weaken no-reset semantics.
- HY2 exact `13da094` remains frozen `BLOCKED_HARNESS_CURRENT_LINE_HY2` at typed `unknown / client_started`; no complete pair or performance result; no same-class retry.
- repeated warm failover exact `f17b648` remains frozen at `startup_setup`; no same-class retry without a material setup hypothesis.
- IPv6 remains environment-blocked when no owned IPv6 path exists.
- live PMTUD still requires its separately accepted authenticated wire/security design gate before live integration; do not infer READY implementation from stale roadmap wording.
- release flags remain `RELEASE_CANDIDATE=false`, `PRODUCTION_READY=false`, `FREEZE=false`, `RELEASED=false`.

## Design / proposal protocol

The coding agent should resolve the two HIGH provenance defects without waiting for administrator approval. This is repository/evidence engineering, not a core architecture or security-policy decision.

For the lifecycle evidence conflict, use the smallest truthful shape:

- first determine whether there was **one invocation with a misread/incompletely copied local phase log**, or **more than one invocation**;
- inspect already-existing local phase/result material from the run if still available; do not fabricate missing facts and do not run the VPS scenario again just to repair documentation;
- if a complete sanitized phase record genuinely exists, record a non-secret provenance anchor such as the local retained log/result SHA-256, line/record count, command exit status, and a short chronology explaining exactly why `e84770e` described an incomplete prefix and why `c4ee406` corrected it;
- if there were two invocations, identify them as separate attempts and verify that the second was actually permitted by a material changed hypothesis; if it was an unchanged pass-chasing retry, preserve that governance defect rather than hiding it;
- if existing evidence cannot establish the complete success account, downgrade current docs back to the strongest supported boundary (`EVIDENCE_PROVENANCE_CONFLICT` or the retained negative) and keep local lifecycle tests as the only accepted closure.

Do not create a general evidence database/framework for this repair.

For the item-4 review SHA defect, use an actually pushed/reachable tree. A review document may say “facts reviewed on exact `<reachable commit>`; review text committed afterward” to avoid impossible self-reference.

## Rolling queue

The queue is deliberately short until the evidence conflict is resolved. Do not spend VPS rental time on another same-class lifecycle run while repository truth is contradictory.

### A. HIGH / READY_LOCAL — reconcile exact-`7cebe6b` lifecycle experiment provenance

**Goal:** make one truthful, inspectable account of the changed-hypothesis installed-package lifecycle run without rewriting history or rerunning for a pass.

**Why now:** current main simultaneously inherits a retained negative and a later success rewrite for the same experiment/tested tree. Release/operator claims cannot build on contradictory provenance.

**Files:** primarily `docs/package-operator-lifecycle-7cebe6b-20260909.md`, plus minimal `IMPLEMENTATION_PLAN.md`, `docs/release-engineering.md`, `docs/release-security-review-packet.md`, `docs/status.md` if their current success claims must be downgraded or qualified. Prefer a small explicit erratum/chronology rather than deleting history.

**Protected invariants:**

- no new VPS invocation for this repair;
- no raw endpoint/topology/secret material;
- exact `e84770e` negative remains historical fact about what that commit claimed;
- do not silently relabel one invocation as two or vice versa;
- no runtime failure claim follows from an evidence conflict;
- local exact-`7cebe6b` lifecycle implementation/tests stay accepted.

**Tests/gates:** governance/evidence checks, `scripts/check.sh`, `git diff --check`, commit/push, exact-head CI green.

**Commit/push:** yes. Continue immediately to B.

### B. HIGH / READY_LOCAL — repair release item-4 exact-tree provenance

**Goal:** remove the unreachable `bb008d0...` as an exact-tree attestation and prepare the factual review against a real GitHub-reachable tree after A.

**Files:** `docs/reviews/release-item4-subgates-20260909.md`, `docs/release-security-review-packet.md`, minimum status/release docs if needed.

**Behavior:**

1. choose the real pushed tree containing the reconciled A evidence and the accepted `7cebe6b` implementation;
2. run `scripts/check.sh` and `git diff --check` on that exact tree;
3. record that reachable SHA explicitly;
4. keep wording “developer-prepared factual support”, not independent review/security approval;
5. do not assert successful VPS lifecycle evidence unless A actually supports it;
6. preserve D019/HY2/repeated-failover/PMTUD/release boundaries.

**Commit/push:** yes. Continue immediately to C after exact-head CI green.

### C. READY_LOCAL / REVIEW CLOSURE — reviewer-facing exact-current release subgate packet

**Goal:** leave a concise, internally consistent packet that a later independent reviewer can inspect without reconstructing contradictory history.

**Review scope:** package provenance and rollback boundary; lifecycle local tests versus VPS evidence; canonical corpus scope; negotiation/Noise/trust/parser facts; HY2 methodology/current negative; RSEC engineering controls versus D019 policy; remaining item-3/item-4 release evidence.

**Do not grow:** no generic checker/parser/harness infrastructure unless a concrete false pass is first demonstrated.

**Commit/push:** only for real factual delta. Continue immediately to D if dependency-ready.

### D. CONDITIONAL OUTPUT / VPS — genuine distinct-version package rehearsal if a real prior package is safely available

**Goal:** exercise `A(old) -> B(current) -> A(old)` only with genuinely different binary/package hashes, immutable experimental release directories, external temporary identity/state, bounded authenticated workload, and cleanup.

**Why this has value:** historical N5 established distinct-version rollback on older trees; current runtime/package has materially changed since then. A real old/current/old rehearsal would add operator evidence not reconstructable from loopback.

**Gate:** use only an already-known-good historical commit/package that repository tooling can build or retrieve safely. Skip if unavailable or if state compatibility would require inventing a migration policy. Never install the same tree twice and call it upgrade.

**Standing boundary:** self-owned VPS only, dedicated experimental path, <=10 minutes, tiny traffic, no production service/path/state, opaque endpoint labels, cleanup required.

**Negative rule:** preserve failures; no unchanged retry/pass-chasing.

**Commit/push:** only if a real rehearsal executes. Continue to E.

### E. CONDITIONAL VPS / RELEASE SUPPORT — native current-tree package/build gate only if it answers a new question

**Goal:** after release-relevant lifecycle/package changes, optionally run one native x86_64 VPS-side current-tree package/check or package smoke if the toolchain already exists and the result would add provenance not already present.

**Skip if:** this would only duplicate the local/CI/package-VPS facts already recorded, require installing a large new toolchain solely for the check, or interfere with a higher-value WAN/operator question.

**Boundary:** this is build/operator provenance, not performance/security/release approval.

### F. NEXT OUTPUT SELECTION — choose a genuinely dependency-ready real-network/operator question

After A–E, re-read exact current state. Prefer one standing-authorized VPS-only question whose hypothesis is concrete and whose evidence cannot be recreated later.

**Do not select:** unchanged HY2, unchanged repeated warm failover, another package-lifecycle retry without a new material hypothesis, IPv6 without environment, live PMTUD before wire/security design acceptance, natural-loss “simulation” mislabeled as natural, FEC/0-RTT/striping/multipath/exotic carriers without an observed-problem gate, new security numeric policy, third-party targets, or production mutation.

If no real READY output exists, keep the queue short instead of manufacturing audit work.

### G. POLICY-BLOCKED PARALLEL LANE — D019 source retention

**Status:** `SOURCE_RETENTION_POLICY_BLOCKED`.

Current bounded engineering controls remain useful, but terminal source-retention semantics require a maintainer/security-policy decision. Do not invent TTL/LRU/history capacity or silently weaken D019. This does not block A–F.

## VPS / evidence priority

- Do **not** rerun the exact-`7cebe6b` lifecycle scenario to cure the documentation contradiction. First reconcile already-existing evidence.
- If A proves the complete success account, freeze that question as answered; do not rerun it unchanged.
- If A cannot prove success, keep the truthful negative/conflict and require a genuinely new material hypothesis before any future lifecycle attempt.
- A real distinct-version package rehearsal is currently the highest-value conditional VPS opportunity after provenance is clean.
- HY2 and repeated-warm-failover same-class attempts remain frozen.
- VPS/load evidence never substitutes for deterministic security accounting or D019 policy.

## Visible-output check

The last 24–48 hours did produce real output: current package reproducibility/install smoke, signal lifecycle implementation with ordered process tests, and multiple bounded real-VPS/operator attempts. The problem this hour is not lack of output but **evidence truth becoming internally inconsistent**. A/B are direct blockers to trustworthy release/operator claims; after they close, return to outward operator evidence rather than growing audit infrastructure.

## Stagnation check

This is **not** `STALLED_IMPLEMENTATION`: `7cebe6b` is a real implementation/test commit, subsequent evidence/review commits landed, and exact-head CI is green. The queue is being corrected because two new provenance defects were discovered, not because coding stopped.

## Maintainer/admin boundary

No administrator action is required for A–F when performed within existing architecture and standing authorization. Do not rewrite published Git history autonomously. D019 remains the known maintainer/security-policy checkpoint. Live PMTUD remains a separate wire/security design-stage boundary. If lifecycle evidence reconciliation proves that an unchanged second VPS invocation was intentionally used to chase a PASS in conflict with standing evidence policy, preserve that fact and stop that subline; do not ask the administrator merely to retroactively bless it.
