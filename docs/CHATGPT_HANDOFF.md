# ChatGPT reviewer handoff — continue bounded RSEC process observation

## Reviewed state

- Previous reviewer-owned handoff: exact `7fe8dacc1660a1b7d645b5c64bbb2963ecea40df` (`docs(handoff): move preauth closure into bounded RSEC observation`).
- Previous reviewed developer implementation/test head: exact `81edd7ba358562e5e6013f889b76e3243dff44fe` (`fix: account endpoint rebind preauth responses`).
- Previous reviewed developer documentation/evidence head: exact `4c2e711d5910018041618cde01597f6567510cbd` (`docs: close preauth response ownership`).
- Current default-branch developer implementation/test head reviewed this cycle: exact `f74b8231ed8a088ceba20d4367993094c3ccbae5` (`test: restore timeout on failed preauth writes`).
- Current default-branch developer documentation/evidence head reviewed this cycle: exact `d9401f1125bd2ea9752ae917a2131953659eaaee` (`docs: record bounded preauth response observation`).
- Developer sequence since the previous reviewer handoff:
  - `8f0ddc3fcb2730f3edc1cbbab31f5e4ba8afb916` adds a deterministic aggregate response-accounting observation at the existing small `process_limits()` test fixture.
  - `0e4e1c9197c001faf32776b42f0d47c4c6ec941c` restores the saved TCP write timeout after both successful and failed bounded pre-auth writes and corrects the false Drop wording on `PreauthResponsePermit`.
  - `f74b8231ed8a088ceba20d4367993094c3ccbae5` adds a focused injected write-error regression proving timeout restoration and clarifies explicit complete/suppress/abandon ownership semantics.
  - `d9401f1125bd2ea9752ae917a2131953659eaaee` persists exact-`f74b823` developer-local gate provenance and indexes the bounded response observation in the release/resource evidence.
- No new VPS/WAN experiment was performed in this sequence.
- GitHub currently exposes no hosted combined-status records for exact `f74b823`. This is not a blocker and must not trigger Actions polling.

## Review verdict

### ACCEPT — TCP bounded-send timeout restoration edge is closed

The prior MEDIUM is repaired on exact `f74b823`: the TCP helper always performs the restore operation after the bounded write attempt, including the injected write-error path, before returning. A write or restore error still fails closed and leaves the charged response attempt settled through explicit abandonment. The focused regression proves that the saved timeout is restored after an injected `BrokenPipe`.

The `PreauthResponsePermit` contract wording is now truthful: completion, suppression, or abandonment must be explicit through the issuing controller; dropping alone does not mutate controller state. Do not add hidden RAII side effects.

### ACCEPT WITH BOUNDS — exact-tree developer-local closure is factual

`docs/local-preauth-bounded-response-observation-f74b823-20260910.md` records exact `f74b823`, clean-tree `PYTHONDONTWRITEBYTECODE=1 bash scripts/check.sh` exit `0`, `git diff --check` exit `0`, distinct UTC start/end, Linux/x86_64, Rust 1.98.0, and initial/final clean source tree.

This is developer-local evidence only. Reviewer did not execute that CI. Hosted CI is absent for this SHA and is not required under the local-CI-first policy.

### ACCEPT WITH BOUNDS — the new response observation is useful but only partial RSEC evidence

The exact `8f0ddc3`/`f74b823` test proves one important invariant: a charged response attempt that is explicitly abandoned remains counted, and the next max-plus-one response operation fails atomically without aggregate response counters exceeding the configured test fixture ceiling.

The scope is intentionally narrow. The test uses the small synthetic `process_limits()` fixture (`3 bytes / 1 packet` response window), not the default D019 candidate ceilings, and it covers only one response-accounting path. The retained documentation correctly says this is not adversarial-load capacity/suitability evidence. Do not promote it into proof that the default candidate limits are operationally suitable.

No new BLOCKER/HIGH correctness or security defect was found in the four reviewed commits.

## Remaining findings before the next visible output

### MEDIUM / READY_LOCAL — RSEC-001 process observation package is still incomplete

The previous handoff required a bounded process-level malformed/aborted pre-auth observation plus a subsequent valid authenticated exchange. Current developer work stopped after a deterministic model-level response-accounting test and the TCP timeout repair.

Still missing:

- a bounded executable/process observation using real local sockets or an existing process fixture;
- an exact declared malformed/truncated/early-close workload count, bytes and duration;
- secret-safe before/after FD/RSS/socket/listener observations when cheaply available;
- proof that no unintended child/listener residue remains after the rejection workload;
- a successful authenticated exchange after that workload, demonstrating that the bounded rejection path did not strand the executable;
- truthful distinction between model-level same-source saturation and process-level connections with distinct ephemeral source ports.

Do not manufacture same-source saturation by treating different TCP ephemeral ports as one source. If the public command shape cannot create repeated malformed attempts in one long-lived process, choose a minimal existing long-lived UDP/failover/periodic process fixture that can, or build a test-only local socket fixture around the existing admission API. Do not add a new production command merely for this evidence.

### MEDIUM / READY_LOCAL — deterministic evidence should touch real D019 default candidate boundaries, not only the tiny synthetic fixture

`ProcessPreauthLimits::default()` still matches the D019 candidate table: source/global states `8/1024`, queue `4/256`, response source/window `2 KiB/4 packets` and `256 KiB/512 packets`, input/work ceilings, 1 s admission/idle window, 5 s lifetime and 100 ms response-send ceiling.

Existing small-fixture tests are valuable for atomicity and failure shape, but the new observation should not be described as exercising the ADR candidate boundary merely because `process_limits()` is internally configured to `3/1`.

Add only compact missing deterministic evidence at actual default limits. Prefer representative boundaries rather than cloning every existing test:

- default per-source state ceiling and first refusal;
- one global state/queue boundary if it can be exercised cheaply and deterministically;
- default response packet/byte accounting with first refusal and retained charging after failure/suppression/abandonment;
- reuse existing generic input/work/overflow tests when they already prove the invariant instead of duplicating them.

Test loop counts are verification parameters, not new capacity policy. Do not change candidate numbers.

### MEDIUM / FACTUAL RECONCILIATION — current evidence anchors are no longer fully aligned

`docs/release-security-review-packet.md` now indexes developer local gates through exact `f74b823`, while its row named `Exact-current item-4 factual review support` still links `docs/reviews/release-item4-subgates-20260909.md`, whose title/scope and exact-tree section remain anchored at `81edd7b`.

`docs/reviews/resource-abuse-evidence-2026-09-04.md` also still opens by describing engineering controls through exact `81edd7b` even though later text separately records the exact-`f74b823` response observation and timeout semantics.

Do not create a standalone docs-only churn commit now. Reconcile all three files once the bounded process observation and its final exact-tree gate exist, so they can share one truthful tested-tree anchor and one precise RSEC evidence boundary.

### LOW / NON-BLOCKING — avoid permanent public API solely for test introspection

`ProcessPreauthAdmission::response_accounting()` was introduced only for the new same-file test in the reviewed diff. If no real runtime/diagnostic consumer needs it, prefer test-local/private or `pub(crate)` access when next touching this area rather than expanding a library API just to assert internal counters. Do not detour from the RSEC observation solely for this cleanup.

## Local-CI-first rule

For every implementation/test slice below, push the coherent developer SHA first, then validate that exact SHA in a clean temporary worktree/clone:

```text
PYTHONDONTWRITEBYTECODE=1 bash scripts/check.sh
git diff --check
git status --porcelain   # empty
```

Persist minimum provenance only after the final replacement SHA is green: exact SHA, exact commands, distinct UTC start/end timestamps, exits, host/OS/arch, stable Rust version and initial/final clean-tree state. If red, preserve the concrete failure and repair it before closure; do not unchanged-rerun until lucky.

No fuzz is required for the local resource/process observation unless implementation actually changes wire decoder/parser/crypto framing. Hosted CI is optional cross-evidence and must not become a wait condition.

## Rolling queue — execute continuously in dependency order

The previous response-permit HIGH and TCP timeout MEDIUM are closed. Continue the RSEC output package without entering a watcher.

### A. READY_LOCAL — finish compact default-limit model evidence

Against exact-current `ProcessPreauthAdmission`, add the smallest missing deterministic coverage described above using the actual D019 default candidate values. Reuse existing generic atomicity/overflow tests; do not duplicate the whole table and do not change policy values.

Keep Delivery/PathValidated/ACK evidence outside this model. Continue immediately to B.

### B. READY_LOCAL / OUTPUT DESIGN — choose the minimal process-level malformed/churn shape

Run a short autonomous proposal cycle over current CLI/process fixtures and pick one of 1-3 concrete shapes. The chosen shape must support a bounded series of pre-auth rejection attempts and then a valid authenticated exchange without inventing a production daemon or new protocol surface.

Preferred order:

1. reuse an existing long-lived loopback command/process fixture that already continues after malformed UDP/TCP pre-auth input;
2. minimally extend an existing process test fixture;
3. use a test-only socket harness around `ListenerAdmission` only if the command architecture cannot demonstrate the property truthfully.

State why the selected source-tuple behavior is what it is. Continue immediately to C.

### C. READY_LOCAL — execute and assert one bounded process recovery/resource observation

Use a deliberately small local workload, normally well below standing WAN limits because this is local evidence. Record exact attempt count, bytes and duration.

Minimum assertions:

- clean listener/process baseline;
- bounded malformed/truncated/early-close or otherwise pre-auth-rejected attempts;
- no successful authentication/delivery evidence from those attempts;
- secret-safe FD/RSS/socket/listener observations where available without a new framework;
- no unintended child/listener residue;
- one subsequent valid authenticated exchange succeeds;
- final cleanup is verified.

This is leak/pathological-growth/recovery evidence, not a stress benchmark, maximum-capacity measurement or public-listener suitability result. Continue immediately to D.

### D. READY_LOCAL — final exact-tree gate and concise provenance

After A-C are in one coherent final implementation/test SHA, run the exact-tree local gate. Persist one concise note with exact SHA, commands, UTC interval, host/OS/arch, Rust, clean-tree state, workload parameters and sanitized observations. If the gate is red, repair the concrete failure before any documentation closure.

Continue immediately to E.

### E. BOUNDED RELEASE/SECURITY RECONCILIATION — narrow RSEC-001 only as far as evidence supports

Update together, only after the final green SHA exists:

- `docs/reviews/resource-abuse-evidence-2026-09-04.md`;
- `docs/release-security-review-packet.md`;
- `docs/reviews/release-item4-subgates-20260909.md` if its exact-tree factual support advances.

Allowed claim: exact bounded local model/process observations exist for the exercised candidate values/workload, cleanup/recovery result and exact tested tree.

Forbidden claims: RSEC-001 fully closed, candidate limits production-suitable, D019 resolved, independent security review complete, public listener approved, RC/release/production ready.

Continue immediately to F.

### F. REVIEW-CLOSURE / READY_LOCAL — close this RSEC engineering observation lane

Perform one bounded adjacent review of the exact implementation/test/evidence package for claim inflation, source-domain confusion, leaked secret/topology data, stale tested-tree anchors and accidental policy invention. Fix only concrete defects found.

If no BLOCKER/HIGH remains, explicitly mark this bounded engineering observation package complete and stop extending the resource-test harness without a new demonstrated defect. Continue immediately to G.

### G. LOCAL OUTPUT SELECTION — choose the next real release/runtime/operator output

Re-read exact-current plan/status/release packet and produce 1-3 dependency-ready proposals. Prefer, in order:

1. a remaining demonstrated correctness/security defect with no policy invention;
2. an advertised runtime/operator behavior that lacks direct executable evidence;
3. another named release-evidence question answerable locally without capacity/security claim inflation.

For each proposal state the observed contradiction/missing behavior, owner file/API, protected invariant, minimum positive/negative evidence and stop condition. Autonomously choose the smallest safe proposal and implement it; do not wait for the next reviewer hour.

Do not choose D019 TTL/LRU/history policy, signing/key-custody policy, SBOM publication policy, previous-release interoperability without a frozen prior release, experimental carriers, or another generic checker merely to fill the queue.

Continue immediately to H.

### H. READY_LOCAL / ROLLING VISIBLE OUTPUT — implement, exact-tree close, repeat

Implement the chosen G output, run the exact-tree local gate, reconcile only changed facts, then repeat G -> H while concrete safe work remains. If a proposal cycle genuinely finds no dependency-ready safe local output, state that truth explicitly instead of entering a polling watcher or manufacturing documentation work.

### I. CONDITIONAL VPS OUTPUT — only if exact-current truth creates a new live question

Repository truth still says `READY_LIVE: none`. Standing VPS authorization remains valid but is not a reason to duplicate old evidence.

Only execute a VPS run if a new implementation/instrumentation change creates a named unresolved real-network question with satisfied dependencies. Otherwise do not unchanged-rerun HY2, repeated warm failover, periodic/soak, package lifecycle, distinct A -> B -> A, already-answered endpoint migration/key update, IPv6 without a real owned IPv6 path, or live PMTUD before its separate authenticated wire/security design gate.

### J. POLICY-BLOCKED PARALLEL LANE — D019 source retention

**Status:** `SOURCE_RETENTION_POLICY_BLOCKED`.

Bounded non-policy engineering and local RSEC evidence may continue. Terminal source-retention/no-reset semantics still require maintainer/security-policy judgment. Do not invent TTL, LRU/history capacity, external authority or weaker reset semantics.

## Coordination notes

- Open PR #3 is stale relative to current main and may be used only as historical/research reference. Do not merge/cherry-pick it wholesale into current main; the response-permit ownership/deadline implementation has already evolved on main.
- `IMPLEMENTATION_COMPLETE=true` remains bounded research-implementation status only.
- `RELEASE_CANDIDATE=false`, `PRODUCTION_READY=false`, `FREEZE=false`, and `RELEASED=false` remain authoritative.
- Release item 3 remains incomplete and current opportunity classification remains `READY_LIVE: none`.
- Item 4 remains independent-review incomplete; developer-prepared factual support is not independent audit/security approval.
- RSEC-001 remains open for promotion suitability/independent review even after the bounded local observation package is complete.

## Stop conditions

Stop continuous coding only for a real condition:

- unresolved BLOCKER/HIGH correctness/security/evidence finding outside the already-authorized repair path;
- core Session/Carrier/ACK/crypto/wire architecture change;
- destructive/canonical-meaning migration;
- action outside standing authorization;
- production impact;
- new credentials/server/third-party permission;
- benchmark conditions requiring maintainer value judgment;
- repository/tool/runtime breakage preventing safe progress;
- actual runtime/tool-budget exhaustion;
- or a genuinely exhausted rolling queue after the required proposal cycle finds no concrete safe work.

Do not stop because GitHub Actions does not run, because this handoff becomes older than a developer commit, or because one coherent slice finishes before the next reviewer hour.
