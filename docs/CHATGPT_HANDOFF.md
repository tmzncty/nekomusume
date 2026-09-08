# Nekomusume ChatGPT Handoff

Checked at: 2026-09-08 19:02 Asia/Shanghai
Repository main HEAD reviewed before this handoff: `ef94e48d9a6f3d91b839d20a613937eac6b8d19c`
Previous reviewer handoff commit: `ef94e48d9a6f3d91b839d20a613937eac6b8d19c`
Previous checked implementation/evidence HEAD: `04cdf7ae50c0023431972525301090f1ba9eb1a7`
Current execution branch: `work/e1a-staged-accounting-20260907` at exact `9a94922794110ca0d9c0d8878f9903173d2de495`
New coding commit under review: `9a94922794110ca0d9c0d8878f9903173d2de495` (`fix: make warm failover diagnostics truthful`)
Exact GitHub Actions: main `ef94e48` run `34213318495` — `success`; implementation `9a94922` run `34216283788` — `success`.
Branch topology at review: `main` and the execution branch are intentionally diverged from merge base `1b5625d4f7c0c7063684690805fe5fda662a41c8`; the work branch is four commits ahead and two reviewer-handoff commits behind. This is not a blocker and does not justify a coordination-only merge before the current closure package is reviewable.
Historical partial-E2 branch `work/continue-20260904` remains retained and is not the active execution branch.

## What changed

The coding agent materially advanced the repeated-warm-failover diagnostic repair at exact `9a94922`. The v1 batch schema now makes `diagnostic_category` backward-compatible/optional, defines one shared diagnostic-category enum for top-level and nested category fields, and retains old failures whose diagnostic object has no category. The existing real JSON-Schema regression validates every retained `artifacts/repeated-warm-failover/*/result.json` plus a newly generated fine-category no-row batch. This closes the previous broad schema incompatibility blocker.

The inner collector also removed prose substring stage guessing. `category_for()` is now conservative (`nonzero_exit`), setup/config call sites largely construct explicit `startup_setup` errors, malformed/duplicate/cardinality/order/timing/accounting evidence largely constructs explicit `evidence_serialization`, and cleanup is attempted before the saved server-start primary error is re-raised. The changed duplicate negotiation/readiness/DeliveryAck tests now correctly expect `evidence_serialization`. Exact-head CI is green.

No VPS attempt occurred in this interval. That remains correct because review found one remaining HIGH attribution defect and two small evidence-regression gaps that should be fixed before the next live run. These are narrow repairs to the just-added diagnostic seam, not a reason to grow a general audit framework.

The 24–48 hour visible-output check remains healthy: recent accepted outputs include live key update, migration-back/recovery, authenticated endpoint rebinding and its promotion-ordering guard, plus materially changed repeated-failover negatives. The current diagnostic work remains justified only because it directly unlocks one high-value rented-VPS question.

## Review verdict

**ACCEPT_MOST_OF_EXACT_`9a94922`; RWFDIAG_SCHEMA_COMPATIBILITY_IS_CLOSED; KEEP_ONE_HIGH_STARTUP-POLL_ATTRIBUTION_DEFECT_PLUS_TWO_BOUNDED_REGRESSION_FIXES_AT_QUEUE_HEAD; DO_NOT_RUN_THE_NEXT_REPEATED-FAILOVER_VPS_ATTEMPT_UNTIL_THE_SMALL_FIX_IS_EXACT-HEAD_GREEN; THEN MAKE_EXACTLY_ONE_CHANGED-HYPOTHESIS_SELF-OWNED_VPS_ATTEMPT.**

There is no core Session/Carrier/ACK/crypto/wire architecture change, no new numeric security policy, no destructive migration, no credential requirement and no production action. Proposal authority applies. This is active implementation progress, not `STALLED_IMPLEMENTATION`.

## Reviewer findings

### RWFDIAG-002 — CLOSED for v1 compatibility / shared-category shape

Exact `9a94922` fixes the former internal schema contradiction:

- `first_failure.diagnostic_category` is optional, so historical `batch_timeout`, `cycle_failed` and retained typed negatives without that field remain valid;
- one `$defs.diagnosticCategory` enum is referenced by both top-level and nested category fields when present;
- nested `diagnostic.category` is optional, preserving historical artifacts such as `4a2129e` / `c6ab8fd` whose diagnostic object predates the category field;
- the test suite performs real Draft 2020-12 validation across the retained repeated-warm-failover artifact corpus and a fresh fine-category no-row batch.

Do not rewrite historical artifacts.

### RWFDIAG-002A — MEDIUM / unnecessary SHA-256 validation weakening

The same schema patch relaxes tracked `diagnostic.sha256` from exact 64 lowercase hex to `16..64`. Current writer code always stores the full `hashlib.sha256(...).hexdigest()`, and retained diagnostic-bearing artifacts also carry full 64-hex values; only human-readable `detail` text uses a 16-character display prefix.

Restore the schema field to exact `^[0-9a-f]{64}$`. Keep the **category field optionality** required for historical compatibility; do not conflate that compatibility fix with hash weakening.

### RWFDIAG-003 — HIGH / narrowed — server-start polling still rewrites evidence failures as startup

The broad prose inference is gone, which is accepted. However, `server_start_readiness()` currently wraps **every** `CollectionError` raised while polling the server log as:

`CollectionError("malformed server JSON event: start", "startup_setup")`.

That catches errors already explicitly owned by `event_objects()`, `validate_event_stream()` and `one_event()` as `evidence_serialization`. The current test consequently changed `malformed_json` to expect `startup_setup`.

This is not truthful stage ownership. Worse, the same malformed line can be observed either during the polling read or only during the later full-log parse depending on scheduling, which can make the diagnostic category timing-dependent even though the evidence defect is identical.

Required minimum repair:

1. do not blanket-convert parser/identity/cardinality `evidence_serialization` errors to `startup_setup` inside `server_start_readiness()`;
2. actual process-start facts remain `startup_setup`: unavailable/invalid config, spawn/start timeout, process exit before a valid start event, or invalid required fields of the start contract;
3. malformed JSON/event shape, duplicate event/cardinality, identity contradiction and later evidence contradictions remain `evidence_serialization` regardless of when the polling loop first observes them;
4. change the malformed-JSON regression back to deterministic `evidence_serialization` and add/retain one start-exit case proving `startup_setup`.

While touching this seam, make pre-spawn `requires()` failures and invalid startup-timeout configuration explicitly `startup_setup` instead of relying on generic `nonzero_exit`. This is a tiny ownership cleanup, not a new phase framework.

### RWFDIAG-004 — CODE DIRECTION ACCEPTED / MEDIUM regression gap

The implementation now preserves the saved server-start `primary_error`, executes cleanup, captures a separate `cleanup_error`, and re-raises the earlier primary before a cleanup error. This is the correct precedence direction.

The current regression does not prove it through the real inner collector. `run-repeated-warm-failover-test.py` synthesizes one subprocess result containing a `startup_setup` marker plus the text `cleanup=malformed`; no inner cleanup command is actually executed in that test. The inner fake harness separately tests `early_exit` and `malformed_cleanup`, but not the required combined case.

Add one deterministic **inner-collector** scenario: server exits before a valid start, cleanup is definitely invoked and returns malformed/failing bounded evidence, a test-only side effect proves cleanup ran, stdout remains empty, and the emitted primary marker remains `startup_setup`. Do not add a secondary tracked schema field merely for this test.

### RWFDIAG-003B — claim boundary for fine runtime stages

The new schema may continue to accept `negotiation_auth`, `readiness` and `application_runtime`, but the collector must emit those only from explicit owned runtime evidence points. Do not add string matching merely to make every enum value appear in tests. A next live failure may truthfully remain `nonzero_exit` if no stronger stage is established.

### RSEC-001 — HIGH for RC/security promotion, independent from this outward closure

Pre-auth accounting remains a real release/security package: source projection, charge-before-parse/response ordering, every real responder/listener call site, concurrent reservations, rate-window rejection, expiry/release, fail-closed cleanup and exact-tree review remain open. Existing candidate numeric limits must not be changed merely to make tests pass. Keep this after the current rented-VPS outward closure unless a newly discovered security defect forces it earlier.

## Evidence / claim boundaries

- Default `main` before this handoff is exact `ef94e48`; exact-main Rust CI run `34213318495` is green.
- Execution exact `9a94922` has green exact-head Rust CI run `34216283788`. It is locally/CI validated diagnostic/schema code, **not** new WAN evidence.
- Exact `9a94922` closes the former broad v1 compatibility and prose-guessing defects, but the startup-poll category rewrite above prevents treating its fine-stage contract as ready for a new live attribution run yet.
- Exact `3aa4828` remains immutable historical changed-hypothesis negative evidence. Its then-emitted `cleanup` label is still only the collector label recorded by that older tree, not a retroactively corrected root-cause claim.
- Historical repeated-failover negatives `4a2129e`, `9fd2411`, `a117086` and `c6ab8fd` remain immutable and validate under the current intended v1 compatibility boundary.
- Endpoint rebinding exact `a8f49fc` / `7405da4`, migration-back exact `5d6582c` / `f024458`, and live key-update exact `2f4f59a` / `69d0ed9` remain accepted only for their previously stated bounded questions. No unchanged reruns.
- Live PLPMTUD remains implementation/design blocked; do not invent probe/ACK wire fields, timer/cooldown or numeric policy as filler.
- IPv6 remains environment-blocked.
- HY2 still has no fair paired comparison and no performance conclusion.
- `RELEASE_CANDIDATE=false`, `PRODUCTION_READY=false`, `FREEZE=false`, `RELEASED=false` remain unchanged.
- Protected identities, credentials, SSH material, private endpoints and raw private diagnostics remain unread/untracked/uncommitted.

## Rolling Work Queue

This is a continuous dependency-ordered queue. A commit, nominal hour, reviewer interval, green test or negative VPS attempt is not a stop condition while a safe dependency-satisfied slice remains.

### A — Finish repeated-failover diagnostic truthfulness as one small repair

**Status:** `READY_LOCAL`, HIGH queue head. Proposal authority applies.

Goal: close the narrowed RWFDIAG-003 attribution defect plus RWFDIAG-002A/004 regression gaps without adding a new diagnostic framework.

Primary files: `scripts/bench/run-live-warm-failover-cycle.py`, `scripts/bench/run-live-warm-failover-cycle-test.py`, `schema/repeated-warm-failover.v1.json`; touch outer tests only if needed for the existing contract.

Protected invariants: valid six-cycle row semantics unchanged; controlled application reply-cessation classification unchanged; historical artifacts immutable; raw diagnostics private/redacted/hash-only; cleanup always executes; no Session/Carrier/ACK/crypto/wire behavior changes.

Required behavior/tests:

1. polling-time malformed/duplicate/identity evidence remains `evidence_serialization`, never opportunistically becomes `startup_setup`;
2. server exit before a valid start and invalid pre-spawn/start-timeout configuration are explicitly `startup_setup`;
3. remove remaining prose-derived stage ownership; generic unknown nonzero remains conservative;
4. startup failure + malformed cleanup executes the real inner cleanup seam and still emits startup as primary;
5. restore tracked diagnostic SHA-256 schema to exactly 64 lowercase hex while retaining historical category optionality;
6. historical artifact schema regression and fresh fine-category batch schema regression remain green.

Commit and push the complete small repair. Do not run the VPS before B.

**Continue immediately to B:** yes.

### B — Local + exact-head gate for A

**Status:** `PREAUTHORIZED_AFTER_A`.

Run the focused inner/outer repeated-failover tests, real JSON-Schema regressions, `./scripts/check.sh`, and `git diff --check`. Fuzz is unnecessary unless parser/wire codec code changes, which this slice should not do.

Push and require green exact-head CI. Do not add checker/doc-only work after green.

**Continue immediately to C after green:** yes.

### C — Exactly one fresh materially changed repeated-warm-failover VPS attempt

**Status:** `PREAUTHORIZED_AFTER_B_GREEN`; highest-value VPS opportunity.

Standing authorization already covers this bounded self-owned TCP/UDP failover experiment. Do not request WAN permission again. A/B materially changes diagnostic ownership and the invalid category race, so exactly one new attempt is justified.

Preserve exact commit/binary, actual bounded parameters/ports, start/end, valid cycle prefix if any, client/server results when a row exists, typed diagnostic boundary only when explicitly established, and cleanup. A stage label is an evidence boundary, not a causal root-cause claim. Preserve a negative result. Do not rerun the same classified failure without another material hypothesis/code/config/path change.

**Continue immediately to D:** yes.

### D — Reconcile C and make at most one evidence-producing repair

**Status:** `PREAUTHORIZED_AFTER_C`.

If C adds a new fact, write one compact evidence/status reconciliation; do not rewrite exact `3aa4828` or older artifacts.

If C exposes one concrete orchestration defect inside the existing architecture, the coding agent may propose the smallest repair, test/commit/push it, and only after that material change plus exact-head green make at most one further bounded attempt. If C repeats the same boundary without a new safe hypothesis, freeze the negative and close this repeated-failover line. Do not grow additional diagnostic infrastructure merely to chase PASS.

### E — Integrate accepted closure lineage to default main once

**Status:** `PREAUTHORIZED_AFTER_D`.

After A-D are reviewable/green with no new HIGH blocker, merge current reviewer `main` into the execution branch while preserving reviewer ownership of `docs/CHATGPT_HANDOFF.md`, then integrate the accepted implementation/evidence lineage to default `main` without history rewriting. Observe exact-main CI.

Do not create coordination-only merges after tiny commits; one closure integration is enough.

### F — HY2 direct-unlock opportunity only

**Status:** `REVIEW_READY_AFTER_E`.

Re-read the latest retained `hy2-1 client_exit` evidence and fair-pair harness. If one minimal security-neutral instrumentation/configuration seam can directly unlock a same-condition paired owned-VPS run, implement/test it and make at most one materially changed attempt under standing authorization. Preserve negative/slower results.

If this would become another generic harness project, require new credentials/permissions, touch an existing production HY2 service, or still cannot produce a fair pair, keep HY2 blocked and continue to G.

### G — Focused RSEC-001 adversarial release/security closure

**Status:** `READY_LOCAL_AFTER_OUTWARD_CLOSURE`.

Re-read exact current pre-auth implementation plus `docs/reviews/resource-abuse-evidence-2026-09-04.md`. Build one bounded adversarial closure package around the **existing** candidate limits: source projection, charge-before-parse/response ordering, every real responder/listener call site, concurrent reservations, rate-window rejection, expiry/release, fail-closed cleanup and secret-safe diagnostics. Do not invent new numeric policy.

Close only what the exact tree proves. If a new safe VPS-ready outward capability becomes available while this security package is underway, retain a parallel/follow-up exit rather than letting audit infrastructure monopolize the project.

## Stop / escalation boundary

Do not notify the maintainer for A-G. Escalate only if a repair truly requires changing core Session/Carrier/ACK/crypto/wire semantics, selecting a new numeric security policy/ADR value, destructive migration, production impact, new credentials/server/third-party rights, an experiment outside standing authorization, a benchmark objective requiring maintainer value judgment, or a major security issue that cannot be resolved fail-closed inside the current architecture.