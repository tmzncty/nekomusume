# ChatGPT reviewer handoff — repair the red exact-tree gate before periodic/identity closure

## Reviewed state

- Previous reviewer-owned handoff: exact `9036877706290d08ef3c5dbd51058096be5b1a23` (`docs(handoff): close periodic identity ordering next`).
- Previous reviewed developer implementation/test head: exact `1648dfdbf6731f55bc31598a120bd42c1de1d8a7` (`fix: secure multistream identity loading`).
- Previous reviewed developer documentation/evidence head: exact `a4dbe72d54b8f51e2734bb0be83c7b90eea8d8c7` (`docs: close multistream identity boundary`).
- Current developer implementation/test head reviewed this cycle: exact `363177c18773d52cfa9edc114ed380117e03b562` (`fix: validate periodic configuration before identity`).
- Current developer documentation/evidence head reviewed this cycle: exact `34f24f7af5e595214b870e27c43204abfb1bb7d7` (`docs: record periodic identity gate result`).
- Developer sequence since the previous reviewer handoff:
  - `363177c18773d52cfa9edc114ed380117e03b562` moves periodic-server peer-key and bind/port validation before automatic identity creation, routes periodic-client peer-key parsing through the shared exact-32-byte helper before identity creation, and adds focused deterministic no-identity-side-effect negatives.
  - `34f24f7af5e595214b870e27c43204abfb1bb7d7` truthfully retains the exact-`363177c` local validation result: focused periodic rejection and authenticated positive tests pass, `git diff --check` passes and the source tree is clean, but the full stable gate exits `101` in an existing failover process test.
- No new VPS/WAN experiment was performed in this sequence.
- GitHub exposes no combined hosted status records for exact `363177c`. This is not a blocker and must not trigger Actions polling.

## Review verdict

### ACCEPT_WITH_BOUNDS — periodic deterministic-config / persistent-identity implementation shape is correct, but closure is not green yet

The previous periodic HIGH is fixed at the code-ordering level:

- periodic-server validates the bounded periodic config, exact-32-byte `--client-key`, parsed `--bind`, and bind-port equality before `load_or_generate()`;
- periodic-client validates its address/port and exact-32-byte `--server-key` before `load_or_generate()`;
- the change does not alter Session/Carrier/ACK/Noise/wire semantics, periodic setup/ACK deadlines, key update, reconnect behavior, or authenticated DeliveryAck semantics.

Focused developer-run tests on exact `363177c` show the table-driven deterministic rejection test and an existing authenticated periodic positive passing. This is useful implementation evidence, but it is **not closure** because the authoritative stable local gate is red.

### BLOCKER / READY_LOCAL — exact-`363177c` stable local gate is red

The retained developer-local note records:

```text
PYTHONDONTWRITEBYTECODE=1 bash scripts/check.sh -> exit 101
```

The failing test is:

```text
first_udp_selection_loss_recovers_from_same_peer_duplicate_hello
```

The note labels this a known fixed-port/process-start race. Treat that as a **developer hypothesis, not an established root cause**: the retained note does not preserve the failing stderr/test transcript needed to prove the attribution.

Repository inspection does show that this test uses fixed UDP/TCP ports `40091`/`40092`, and `failover_server` emits its diagnostic `start` event only after both sockets bind. A bind/start race is therefore plausible. However, the same test already participates in the shared `TEST_PORT_LOCK`, and many other process/socket tests also use that lock; do not blindly add another mutex or assert that fixed ports alone explain the failure.

Because local CI is first-class and failed, no release/security/operator closure may claim exact `363177c` as a green tested tree. Do not paper over the failure with GitHub Actions absence, retries-until-lucky, sleeps, or a docs-only reclassification.

**Required repair contract:**

1. reproduce the failing focused test and, if needed, the full-suite-only failure on an exact clean tree;
2. retain enough sanitized diagnostics to distinguish bind collision, stale child/listener cleanup, startup synchronization, or another concrete failure class;
3. make the smallest hermetic test/process-fixture repair that addresses the observed class;
4. preserve product port policy (`40080..=40100`) and existing failover/negotiation/runtime semantics;
5. if the actual issue is test port ownership, repair test-only allocation/reservation or duplicate same-protocol port reuse rather than changing production CLI semantics;
6. if the actual issue is startup/process cleanup, fix the readiness/child lifecycle fixture rather than hiding it with arbitrary sleeps;
7. after repair, run the full exact-tree local gate once as the closure test. If it is red, continue repairing before any closure claim.

This BLOCKER is an evidence/release-gate blocker. It does **not yet establish a product runtime correctness defect**.

### MEDIUM / READY_LOCAL — periodic focused negative matrix is still incomplete

The implementation ordering is present, but the focused table does not yet cover all cases that the previous handoff explicitly required. Add the missing process negatives, each beginning with an absent identity path and proving that it remains absent after deterministic rejection:

- periodic-server bind/`--port` mismatch;
- periodic-client malformed peer-key hex;
- periodic-client malformed address;
- periodic-client address/`--port` mismatch.

The table already covers periodic-server malformed hex, 31-byte and 33-byte keys, malformed bind, plus periodic-client 31-byte and 33-byte keys. Keep at least one existing authenticated periodic positive; do not duplicate the substantial periodic runtime suite.

This MEDIUM may be folded into the same coherent test commit as the BLOCKER repair because both are owned by `crates/neko-cli/tests/probe.rs`, provided the actual root-cause repair remains clear in the diff and evidence.

### ACCEPT — current evidence note is honest about the red gate

`docs/local-periodic-identity-363177c-20260909.md` correctly records the red full gate rather than calling it green. Preserve it as historical evidence even after a replacement implementation/test SHA passes. Do not rewrite this artifact to pretend exact `363177c` later passed.

The current release packet and item-4 factual support still anchor the last retained green developer-local security gate at exact `1648dfd`; that is conservative and correct until a replacement periodic/failover test tree is green.

## Local-CI-first rule

For the BLOCKER/MEDIUM repair, commit/push one coherent implementation/test SHA first. Validate that exact developer SHA in a clean temporary worktree/clone:

```text
PYTHONDONTWRITEBYTECODE=1 bash scripts/check.sh
git diff --check
git status --porcelain   # must be empty
```

Persist minimum provenance only after the final replacement SHA is green:

- exact SHA;
- exact commands;
- distinct UTC start/end timestamps;
- all exit codes;
- host/OS/arch;
- stable Rust version;
- initial/final clean-tree state;
- sanitized failure/repair diagnostic only where needed to explain the previously red gate.

Do not wait for GitHub Actions. Hosted CI, if it happens to run, is only additional cross-evidence. Do not label developer-local CI as reviewer-executed or hosted CI.

No fuzz is required for the planned test/config-ordering repair because it does not touch wire decode/parser/crypto framing. If the repair unexpectedly crosses that boundary, use the repository's pinned fuzz toolchain contract before closure.

## Rolling queue — execute continuously in dependency order

There is one unresolved BLOCKER, so A must finish before unrelated horizontal expansion. The queue below is intentionally closure-oriented rather than an artificial collection of tiny checker tickets. Complete A -> B -> C -> D -> E continuously, then enter F -> G -> H as the rolling visible-output loop.

### A. BLOCKER / READY_LOCAL — diagnose and remove the exact-tree failover process-test failure

**Goal:** make the stable local gate deterministically green again without weakening product semantics or hiding a real failure.

**Files/concepts:** `crates/neko-cli/tests/probe.rs`, existing `TEST_PORT_LOCK`, failover process startup/readiness helpers, child/listener cleanup, fixed test ports only as evidence indicates; production `failover_server` only if diagnostics prove a product issue.

**Protected invariants:** no change to Session/Carrier/ACK/Noise/wire semantics; no widening of operator port policy; no sleeps/retries used merely to mask nondeterminism; negative evidence remains honest.

**Validation:** focused failing test, nearby negotiation/failover positives/negatives as relevant, then the full stable local gate on the final commit.

Continue immediately to B. If B is naturally part of the same small `probe.rs` repair, combine the commit and continue to C.

### B. READY_LOCAL — complete the periodic no-secret-side-effect regression matrix

**Goal:** lock the already-implemented deterministic configuration ordering across the missing periodic cases.

**Files/concepts:** `crates/neko-cli/tests/probe.rs`; no new harness framework.

**Required cases:** server bind/port mismatch; client malformed key hex; client malformed address; client address/port mismatch. All must fail with an initially absent identity still absent. Keep one authenticated periodic positive.

Continue immediately to C.

### C. READY_LOCAL — replacement exact-tree local gate and retained provenance

On the final pushed A/B implementation/test SHA, run:

```text
PYTHONDONTWRITEBYTECODE=1 bash scripts/check.sh
git diff --check
git status --porcelain
```

A red result is a failure and returns to A/B. Do not rerun unchanged merely hoping for a pass. Once green, persist the minimal provenance described above.

Continue immediately to D.

### D. READY_LOCAL / FACTUAL RECONCILIATION — re-anchor release-facing identity facts once

Only after C is green:

- update `docs/release-security-review-packet.md` to the actual C-tested implementation/test SHA;
- update `docs/reviews/release-item4-subgates-20260909.md` to the same tested-tree anchor;
- update `docs/release-engineering.md` only if its broad CLI identity statement is otherwise stale;
- preserve exact `363177c`'s red local-CI note unchanged as historical evidence;
- preserve all older green records rather than rewriting history.

Do not change release flags, item-3/item-4 completion, D019, WAN, performance, interoperability, or production claims.

Continue immediately to E.

### E. BOUNDED IDENTITY-CONSUMER CLOSURE — finish this audit lane and stop expanding it

Inspect only actual long-term identity consumers reachable from advertised CLI commands:

- automatic-generation commands perform deterministic local config validation before `load_or_generate()` where possible;
- read-existing-only multistream remains on the descriptor-bound secure reader and does not auto-generate;
- operator-supplied authenticated peer keys use exact length before secret/network side effects;
- no second raw long-term identity reader remains.

If aligned, explicitly close the identity/config lane. Do **not** expand into parent-directory policy, hard-link policy, secret-memory frameworks, generic filesystem hardening, or another checker suite without a newly demonstrated defect.

Continue immediately to F.

### F. LOCAL OUTPUT SELECTION / PROPOSAL — choose the next real visible gap

Re-read exact-current `docs/status.md`, `IMPLEMENTATION_PLAN.md`, release packet, command help/capabilities, operator/runtime code, and tests. Produce 1–3 concrete dependency-ready proposals grounded in an observed executable behavior/evidence contradiction, then autonomously choose the smallest fail-closed option that does not hit a stop condition.

For each proposal record:

- the actual user/operator/runtime/release behavior that is missing or contradictory;
- file/API ownership;
- protected invariant/evidence boundary;
- minimum positive/negative tests;
- why it creates more value than another checker/doc-only refinement.

Prefer a real advertised runtime/operator behavior lacking direct executable coverage, then release correctness, then a bounded lifecycle/recovery output lacking truthful exact-current local evidence. Do not select speculative FEC/0-RTT/concurrent striping/multipath/exotic carriers, previous-release interoperability without a prior release, broad audits, or evidence-schema work without an observed defect.

Continue immediately to G after selecting a safe proposal.

### G. READY_LOCAL / VISIBLE OUTPUT — implement the selected F option

Implement one coherent output with focused tests, preserving core architecture and established security/resource policy. Commit/push and continue immediately to H.

### H. READY_LOCAL / ROLLING CLOSURE — exact-tree gate, factual closure, repeat

Run the exact-tree local gate on G. Add the pinned decode fuzz smoke only if wire decoder/parser/crypto framing changed. Reconcile docs only where actual behavior/evidence changed. Then repeat F -> G -> H while concrete safe local work remains; do not enter watcher mode merely because the reviewer interval has not arrived.

If a genuine proposal cycle finds no dependency-ready safe local output, state that truth explicitly rather than manufacturing checker work. A later reviewer can re-evaluate new repository facts.

### I. CONDITIONAL VPS OUTPUT — only if exact-current truth opens a genuinely new live question

Current repository truth remains `READY_LIVE: none`. Standing authorization remains valid, but the VPS rental window does not justify duplicate evidence.

Only execute a live task if a new exact-current implementation/instrumentation change creates a named unresolved real-network question with satisfied dependencies. Otherwise do not unchanged-rerun HY2, repeated warm failover, periodic, package lifecycle, distinct A -> B -> A, already-answered migration-back/key-update, IPv6 without a real owned IPv6 path, or live PMTUD before its separate authenticated wire/security design gate.

### J. POLICY-BLOCKED PARALLEL LANE — D019 source retention

**Status:** `SOURCE_RETENTION_POLICY_BLOCKED`.

Bounded non-policy pre-auth engineering may continue, but terminal source-retention/no-reset semantics require maintainer/security-policy judgment. Do not invent TTL, LRU/history capacity, external authority, or weaker reset semantics. This does not block A-I.

## Stop conditions

Stop continuous coding only for a real condition:

- unresolved BLOCKER/HIGH correctness/security/evidence finding;
- core Session/Carrier/ACK/crypto/wire architecture change;
- destructive/canonical-meaning migration;
- action outside standing authorization;
- production impact;
- new credentials/server/third-party permission;
- benchmark conditions requiring maintainer value judgment;
- repository/tool/runtime breakage preventing safe progress;
- actual runtime/tool-budget exhaustion;
- or a genuinely exhausted rolling queue after the required proposal cycle finds no concrete safe work.

Do not stop because the handoff becomes older than a developer commit, because GitHub Actions does not run, or because one slice finishes before the next reviewer hour.

## Governance boundaries

- `IMPLEMENTATION_COMPLETE=true` remains bounded research-implementation status only.
- `RELEASE_CANDIDATE=false`, `PRODUCTION_READY=false`, `FREEZE=false`, and `RELEASED=false` remain authoritative.
- Release item 3 remains incomplete; opportunity classification remains `READY_LIVE: none`.
- Item 4 remains independent-review incomplete; developer-prepared factual support is not independent audit/security approval.
- Canonical corpus freeze remains corpus-specific and does not freeze the global protocol.
- Standing VPS authorization remains valid, but no dependency-ready live row currently exists.
- D019 remains a maintainer/security-policy checkpoint and must not be invented by the coding agent.
