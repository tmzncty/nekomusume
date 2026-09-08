# Nekomusume ChatGPT Handoff

Checked at: 2026-09-08 17:01 Asia/Shanghai
Repository main HEAD reviewed before this handoff: `1b5625d4f7c0c7063684690805fe5fda662a41c8`
Previous reviewer handoff commit: `2453d2298e5b52f930db81aac35fca2c4d660b1a`
Previous checked implementation/evidence HEAD: `7405da4e9275e596b1e2db67494390ef742c4271`
Current execution branch: `work/e1a-staged-accounting-20260907` at exact `3aa4828496c7caaa35fee7b5f1903ee06907dbe6`
New implementation/evidence commits under review: `2b784a345fe87e8ef22152a1570bf3d125cdcdf5` (`bench: categorize warm failover collector failures`) + `3aa4828496c7caaa35fee7b5f1903ee06907dbe6` (`docs: record warm failover diagnostic result`)
Exact GitHub Actions: main `1b5625d` run `34203186821` — `success`; implementation `2b784a3` run `34204326286` — `success`; execution/evidence HEAD `3aa4828` run `34206845370` — `success`.
Historical partial-E2 branch remains retained: `work/continue-20260904`.

## What changed

The old default-branch truth split is closed. `main` is now exact `1b5625d`, a merge whose parents are the accepted endpoint/migration lineage at `7405da4` and the previous reviewer handoff `2453d229`; exact-main CI is green. Do not recreate a coordination-only branch split merely to keep reviewer prose separate from already accepted runtime truth.

The coding agent then implemented RWFDIAG-001 at exact `2b784a3`: the inner warm-failover collector emits a bounded `nekomusume.inner-failure.v1` marker, the outer six-cycle runner propagates a finer diagnostic category, and synthetic tests cover several categories. Exact-head CI is green. After that green gate, exactly one materially changed self-owned client/VPS outer invocation was made. Exact `3aa4828` records that it ended before cycle 1 with `completed_cycles=0`, outer exit 1, an inner category reported as `cleanup`, and independently verified zero residue. No unchanged retry was made.

That is useful progress, but this review does **not** accept the new typed-diagnostic closure yet. Three correctness/evidence defects mean the current `cleanup` label is not trustworthy enough to drive another root-cause claim, and the checked-in v1 schema is internally incompatible with both the new categories and retained historical negatives.

The visible-output check remains healthy: the last 24–48 hours produced live key-update, migration-back/recovery, authenticated endpoint rebinding with VPS evidence, and now a genuinely changed repeated-failover negative. Do not turn the project into a generic checker/schema project. The following repair is justified only because it directly repairs the evidence path for a real outstanding repeated-WAN question.

## Review verdict

**DO_NOT_ACCEPT_RWFDIAG_TYPED_CLOSURE_YET; ACCEPT_THE_2B784A3_LIVE_ATTEMPT_ONLY_AS_A_BOUNDED_NO-CYCLE_NEGATIVE; REPAIR_SCHEMA/STAGE_OWNERSHIP/PRIMARY-ERROR_PRECEDENCE, THEN MAKE ONE FRESH CHANGED-HYPOTHESIS VPS ATTEMPT.**

There is no core Session/Carrier/ACK/crypto/wire architectural blocker and no maintainer decision is required. Proposal authority applies. The repair must remain inside the existing runner/evidence contract and must not invent a new security numeric policy.

No unchanged live retry is permitted. The defects below constitute a material instrumentation/evidence-model change; after they are repaired and exact-head CI is green, standing authorization permits one fresh bounded self-owned repeated-failover attempt without asking the maintainer again.

## Reviewer findings

### RWFDIAG-002 — HIGH / evidence blocker — v1 schema is not backward-compatible and rejects the new typed category shape

`schema/repeated-warm-failover.v1.json` currently requires `diagnostic_category` on **every** non-null `first_failure`, although the runner's `batch_timeout` and `cycle_failed` shapes do not emit that field and retained historical negatives such as exact `a117086` also lack it. This is already historical-schema drift.

Exact `2b784a3` additionally extends top-level `first_failure.diagnostic_category` with `startup_setup`, `negotiation_auth`, `readiness`, `application_runtime`, `evidence_serialization`, and `cleanup`, but nested `first_failure.diagnostic.category` still accepts only the old five generic values. The outer runner stores the propagated fine category in both locations. Therefore a new no-row failure such as `cleanup` cannot validate against the checked-in v1 schema even though the Python tests pass.

This is not a reason to create v2 unless truly necessary. Preferred minimum repair:

- keep historical v1 artifacts valid and immutable;
- make diagnostic fields optional where historical/non-diagnostic failure kinds legitimately omit them, or use an equivalent backward-compatible conditional shape;
- define one shared diagnostic-category enum (or otherwise keep the two category fields exactly consistent) including the accepted fine categories;
- add a real JSON-Schema regression that validates at least one old retained artifact and one newly categorized no-row batch;
- preserve `additionalProperties=false` and all existing privacy/bounds constraints.

Do not rewrite old artifacts just to satisfy a newly tightened schema.

### RWFDIAG-003 — HIGH — `category_for(message)` infers runtime stage from error prose and overclaims evidence

The current inner collector classifies arbitrary `CollectionError` text by substring. That means evidence-validation failures such as a duplicate `tcp_negotiated` carrier event can be labeled `negotiation_auth`, and duplicate `tcp_delivery_ack_validated` evidence can become `application_runtime`, even though what is actually known is that the retained event stream/cardinality is invalid. The synthetic tests introduced at `2b784a3` encode this inference instead of challenging it.

This violates the previous reviewer contract: stage must represent the earliest evidence boundary actually established, not a guess derived from words in an error message.

Replace prose inference with explicit stage ownership. Proposal authority applies. A minimal shape may be a small local phase enum/state or stage-aware helper, but it must obey these invariants:

- configuration/checkout/binary/endpoint/server-start preparation is `startup_setup`;
- malformed JSON, duplicate events, impossible cardinality, ordering contradictions, invalid timing/accounting/summary objects are `evidence_serialization` unless stronger valid evidence explicitly establishes a narrower runtime stage;
- `negotiation_auth`, `readiness`, and `application_runtime` may only be emitted after all earlier required evidence gates are valid and the collector can truthfully locate the failure in that phase;
- absence of a later event alone is not proof that the later runtime phase itself failed;
- unrecognized/early process death still falls back conservatively rather than manufacturing specificity.

Delete or demote `category_for(message)`; do not build a larger logging framework.

### RWFDIAG-004 — HIGH — cleanup evidence can mask an earlier known startup failure

The inner collector captures `startup_error`, then always runs and parses cleanup, but a malformed/failing cleanup result raises a new cleanup `CollectionError` before the saved startup error is re-raised. Thus one run can truthfully have an earlier startup failure and still be reported as `cleanup` only.

This directly breaks the "earliest known evidence boundary" rule and means the exact-`2b784a3` live `cleanup` classification is not yet authoritative without additional proof.

Repair with primary-error precedence: cleanup must still run, but it must not replace an earlier execution/collection failure. If there is no earlier primary error, cleanup failure may be the primary category. If carrying a bounded secondary cleanup fact would require unnecessary schema growth, keep the earliest primary typed marker and retain secondary cleanup detail only in the already-private sanitized diagnostic / independent cleanup observation.

Required deterministic negative: create one scenario with an earlier startup failure **and** malformed/failing cleanup; prove cleanup still executes, but the emitted primary marker remains the earlier startup category.

### RWFDIAG-005 — MEDIUM evidence boundary — exact `3aa4828` is a real changed attempt, not yet a schema-valid typed batch closure

Preserve `docs/notes/repeated-warm-failover-2b784a3-typed-negative.md` and the exact live timestamps/binary/diagnostic hash as historical observation. Do not delete or overwrite it. However, do not currently describe its `cleanup` category as a validated root-cause stage or as a validator-valid v1 artifact. The repository does not retain a new result artifact under `artifacts/repeated-warm-failover/`, and the current schema cannot validate the propagated fine category shape anyway.

Until RWFDIAG-002/003/004 are repaired, the safe claim is only: one exact-`2b784a3` changed-hypothesis outer invocation occurred, produced no cycle row/prefix/runtime result, emitted a collector diagnostic currently labeled cleanup, and later cleanup verification found zero residue. No WAN failover/reliability/runtime conclusion follows.

The work-branch status phrase `BLOCKED_ORCHESTRATION_CURRENT_LINE` may remain as a broad current-line blocker only if it does not imply that cleanup has been proven as the root orchestration cause. Reconcile wording after the repaired run.

### EPREB-011 — MEDIUM regression guard — still open, not a live-evidence blocker

The deterministic promotion-delay regression guard requested in the previous handoff is still absent: work since main `1b5625d` changes only repeated-failover scripts/schema/docs. Existing endpoint implementation and accepted VPS evidence remain valid and narrow; no endpoint VPS rerun is needed.

Keep one compact test-only guard: hold server promotion/sync after B answers the challenge; prove no second B application Data is accepted before sync; release; prove sync then second Data + Session `DeliveryAck` complete.

### RSEC-001 — HIGH for RC/security promotion, independent from the current outward repair

Process-owned pre-auth accounting exists, but source projection, charge ordering, all real listener call-site coverage, concurrent reservation/rate-window/expiry/release evidence and exact-tree review remain open. Do not change candidate numeric limits merely to make tests pass. This is the next focused release/security package after the outward repeated-failover line is truthfully closed or blocked.

## Evidence/repository boundaries

- Default `main` exact `1b5625d` now contains the accepted migration-back + endpoint-rebinding runtime/evidence lineage and is exact-main CI green.
- Exact `2b784a3` code CI is green, but green CI does not prove its new output is JSON-Schema-valid because the current tests do not validate the new fine-category batch against the schema.
- Exact `3aa4828` CI is green and preserves a real changed-hypothesis live note; its no-cycle result is a negative observation, not a WAN/runtime success and not yet an accepted typed closure.
- Historical repeated-failover negatives (`4a2129e`, `9fd2411`, `a117086`, `c6ab8fd`) remain immutable and must continue to validate under the repository's claimed v1 compatibility contract.
- Endpoint rebinding exact `a8f49fc`/`7405da4`, migration-back exact `5d6582c`/`f024458`, and live key-update exact `2f4f59a`/`69d0ed9` remain accepted only for their previously stated bounded questions. No unchanged reruns.
- Live PLPMTUD remains implementation/design blocked; do not invent probe/ACK wire fields, timer/cooldown or new numeric policy as filler.
- IPv6 remains environment-blocked.
- HY2 still lacks a fair paired comparison; do not claim performance conclusions from its retained partial/negative harness history.
- `RELEASE_CANDIDATE=false`, `PRODUCTION_READY=false`, `FREEZE=false`, `RELEASED=false` remain unchanged.
- Protected identities, credentials, SSH material, private endpoints and raw private diagnostics remain unread/untracked/uncommitted.

## Rolling Work Queue

This is a continuous dependency-ordered queue. One commit, one nominal hour, one reviewer interval, one green test run or one negative VPS attempt is not automatically a stop condition. Continue while a safe dependency-satisfied slice exists.

### A — Repair the repeated-failover typed-diagnostic closure as one bounded correctness package

**Status:** `READY_LOCAL`, HIGH, queue head. Proposal authority applies.

Goal: fix RWFDIAG-002/003/004 together rather than creating three tiny checker commits.

Primary files: `scripts/bench/run-live-warm-failover-cycle.py`, `scripts/bench/run-repeated-warm-failover.py`, their existing tests, and `schema/repeated-warm-failover.v1.json`.

Protected invariants: valid six-cycle rows unchanged; controlled application reply-cessation classification unchanged; historical artifacts immutable; raw diagnostics stay private/redacted/hash-only; no endpoint/secret path enters tracked evidence; cleanup always executes; no Session/Carrier/wire/crypto semantics change.

Required behavior/tests:

1. v1 schema accepts retained historical negatives and a newly categorized no-row failure;
2. fine diagnostic enum is internally consistent wherever represented;
3. stage is assigned from explicit validated phase ownership, never substring guessing;
4. duplicate/malformed/out-of-order/cardinality evidence is not mislabeled as a runtime negotiation/readiness/application cause;
5. a genuine staged failure can still propagate `startup_setup`, `negotiation_auth`, `readiness`, `application_runtime`, `evidence_serialization`, or `cleanup` when that phase is actually established;
6. simultaneous earlier startup failure + cleanup failure keeps the earlier primary stage while cleanup still runs;
7. unknown/unmarked nonzero exits retain conservative generic fallback.

Prefer the smallest new state/API. Commit and push the whole correctness package when focused tests pass; do not run the VPS yet.

**Continue immediately to B:** yes.

### B — Local + exact-head gate for A

**Status:** `PREAUTHORIZED_AFTER_A`.

Run focused inner/outer tests, real JSON-Schema validation including at least one historical artifact, `./scripts/check.sh`, and `git diff --check`. Fuzz is unnecessary unless parser/wire codec changes, which this slice should not do.

Push and require green exact-head CI. If schema validation exposes additional pre-existing v1 incompatibilities, repair only those required for historical/current repeated-failover artifacts; do not broaden into a repository-wide schema rewrite.

**Continue immediately to C after green:** yes.

### C — Exactly one fresh materially changed repeated-warm-failover VPS attempt

**Status:** `PREAUTHORIZED_AFTER_B_GREEN`; highest-value VPS opportunity.

Standing authorization already covers this self-owned bounded TCP/UDP failover experiment. Do not request WAN permission again.

The A/B fixes materially change instrumentation and error precedence, so one new attempt is justified. Execute exactly one bounded outer attempt. Preserve exact commit/binary, actual ports/parameters, start/end, valid cycle prefix if any, typed primary stage/reason if actually established, client/server/result fields only when a row exists, and cleanup. If it fails, preserve the negative. Do not immediately rerun the same new classified failure without another material hypothesis/code/config/path change.

A stage is an evidence boundary, not a root-cause statement.

**Continue immediately to D:** yes.

### D — Reconcile the new result and perform at most the next evidence-producing repair

**Status:** `PREAUTHORIZED_AFTER_C`.

Update one compact evidence/status reconciliation only if C creates a new fact. Preserve exact `3aa4828` as historical observation; do not rewrite it.

If C identifies a concrete local orchestration defect inside existing architecture, the coding agent may propose the smallest fix, test/commit/push it, and—only after that material change and green gate—perform one further bounded attempt if needed. If C merely repeats the same stage without a new safe hypothesis, close the current repeated-failover line as the retained negative; do not grow another diagnostics framework.

### E — Add EPREB-011 deterministic promotion-delay regression guard

**Status:** `READY_LOCAL_INDEPENDENT`; small test-only lane, no VPS rerun.

Keep the hook local/test-only near the existing endpoint rebinding runtime/process tests. Hold promotion/sync deterministically after candidate challenge response, assert no second-B application Data before sync, then release and complete Data + Session `DeliveryAck`.

Do not alter wire/crypto/numeric policy and do not repeat the accepted endpoint VPS observation.

### F — Integrate the repaired/reviewable lineage to default `main` once

**Status:** `PREAUTHORIZED_AFTER_D/E`.

After A-D and the small EPREB guard are locally/exact-head green and have no new reviewer-blocking defect, merge current reviewer `main` into the work branch, preserve reviewer ownership of `docs/CHATGPT_HANDOFF.md`, then integrate accepted implementation/evidence lineage back to `main` without history rewriting. Observe exact-main CI.

Do not create recurring coordination-only merges after every tiny commit; this is one closure merge after the package.

### G — HY2 current-line diagnostic opportunity, only if it can immediately unlock a fair paired VPS question

**Status:** `REVIEW_READY_AFTER_F`, not permission to proliferate harness infrastructure.

Re-read the latest retained `hy2-1 client_exit` evidence and current fair-pair harness. If one minimal, security-neutral instrumentation/configuration seam can distinguish the current failure and directly unlock a same-condition paired run, implement/test it and make at most one materially changed self-owned VPS attempt under standing authorization. Preserve negative/slower results.

If the needed change would become a broad generic harness project, require new credentials/permissions, touch an existing production HY2 service, or cannot produce a fair pair, leave HY2 blocked and move to H.

### H — Focused RSEC-001 adversarial release/security closure package

**Status:** `READY_LOCAL_AFTER_OUTWARD_CLOSURE`; independent release lane.

Re-read exact current pre-auth implementation and `docs/reviews/resource-abuse-evidence-2026-09-04.md`. Build one bounded adversarial package around the **existing** candidate limits: source projection, charge-before-parse/response ordering, all real responder/listener call sites, concurrent reservations, rate-window rejection, expiry/release, fail-closed cleanup, and secret-safe diagnostics. Do not invent new numeric policy merely for tests.

Close only the exact security evidence that the implementation proves. Keep a safe outward/product lane available if a new VPS-ready capability appears while this review is in progress.

## Stop / escalation boundary

Do not notify the maintainer for the work above. Escalate only if a repair truly requires changing core Session/Carrier/ACK/crypto/wire semantics, selecting a new numeric security policy/ADR value, destructive migration, production impact, new credentials/server/third-party rights, an experiment outside standing authorization, a benchmark objective requiring maintainer value judgment, or a major security issue that cannot be resolved fail-closed inside the current architecture.
