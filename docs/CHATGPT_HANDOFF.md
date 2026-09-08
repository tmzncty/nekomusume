# Nekomusume ChatGPT Handoff

Checked at: 2026-09-08 17:58 Asia/Shanghai
Repository main HEAD reviewed before this handoff: `921ded1b7ed0730601a74cd99cddf401b4e7b842`
Previous reviewer handoff commit: `921ded1b7ed0730601a74cd99cddf401b4e7b842`
Previous checked implementation/evidence HEAD: `3aa4828496c7caaa35fee7b5f1903ee06907dbe6`
Current execution branch: `work/e1a-staged-accounting-20260907` at exact `04cdf7ae50c0023431972525301090f1ba9eb1a7`
New coding commit under review: `04cdf7ae50c0023431972525301090f1ba9eb1a7` (`test: delay endpoint promotion sync`)
Exact GitHub Actions: main `921ded1` run `34208538004` — `success`; implementation `04cdf7a` run `34211087426` — `success`.
Branch topology at review: the execution branch is ahead of current main by the `2b784a3` / `3aa4828` / `04cdf7a` implementation/evidence lineage and is one reviewer-handoff commit behind; this is not a blocker and does not justify a coordination-only merge.
Historical partial-E2 branch `work/continue-20260904` remains retained and is not the active execution branch.

## What changed

The coding agent completed the independent EPREB-011 regression guard at exact `04cdf7a`. The endpoint-rebind server now accepts a bounded test-only `--test-promotion-delay-ms` value (`0..=1000` ms). When nonzero, after receiving and authenticating B's exact challenge response but **before** `validate_and_promote`, the server holds promotion/sync, reads the candidate B socket during the hold, and fails if any authenticated `ProcessMessage::Data` arrives. The existing real-socket process test enables a 100 ms hold and requires the `endpoint_promotion_delay_held` event before the normal promotion/sync/post-rebind DeliveryAck closure. Exact-head CI is green.

Review accepts this as the requested deterministic ordering regression. It adds no new runtime capability and no new WAN evidence; the already accepted exact-`a8f49fc` endpoint-rebinding VPS observation remains unchanged and must not be rerun merely because this guard landed. The hook is bounded, opt-in, test-named, and does not change wire, Session delivery, crypto or numeric security policy.

The queue-head repeated-warm-failover repair did **not** move in this coding slice. Exact `04cdf7a` still carries the same three HIGH evidence-model defects previously identified in exact `2b784a3` / `3aa4828`: schema incompatibility, prose-derived stage inference, and cleanup masking an earlier primary failure. This is only the first reviewer cycle after that HIGH package was assigned, so do **not** label it `STALLED_IMPLEMENTATION` yet. It remains the next mandatory slice; do not spend the next slice on HY2, release polish, or another endpoint/WAN rerun.

No new VPS run was made in this interval. That is correct: the repeated-failover live opportunity is gated on the local correctness repair plus exact-head green. Standing authorization remains valid once that gate is met.

The 24–48 hour visible-output check remains healthy: the repository has recent accepted live key-update, migration-back/recovery, authenticated endpoint rebinding, and a materially changed repeated-failover negative. The current diagnostic repair is justified only because it directly unlocks a still-open real-WAN question; do not generalize it into a new audit/checker framework.

## Review verdict

**ACCEPT_EP_REB_DELAY_GUARD; KEEP_RWFDIAG_REPAIR_AS_HIGH_QUEUE_HEAD; DO_NOT_ACCEPT_THE_CURRENT `cleanup` LABEL AS ROOT-CAUSE OR SCHEMA-VALID CLOSURE; AFTER_LOCAL_REPAIR_AND_EXACT_GREEN, MAKE EXACTLY_ONE_FRESH_CHANGED-HYPOTHESIS SELF-OWNED VPS ATTEMPT.**

There is no core Session/Carrier/ACK/crypto/wire architectural blocker and no maintainer decision is required. Proposal authority applies to the repair below. No new numeric security policy, destructive migration, credentials or production change are needed.

## Reviewer findings

### EPREB-011 — CLOSED / accepted regression guard

At exact `04cdf7a`, the server-side delay occurs after B's authenticated challenge response is validated but before carrier/source-binding promotion. During the hold, an authenticated B `Data` record causes fail-closed termination; only after the hold does the server promote and send the existing authenticated promotion-sync response. The client runtime itself still blocks on that sync before constructing/sending the second B application record. The process test runs this path with a 100 ms hold and exact-head CI is green.

Boundary: this is a local deterministic ordering guard. It does not expand the accepted endpoint claim beyond one bounded same-Session source-port/source-binding change under the existing fixed crypto record context, and it does not justify another endpoint VPS observation.

### RWFDIAG-002 — HIGH / still open — v1 schema is internally incompatible with historical and new negatives

Current exact `04cdf7a` still has all of the following:

- `$defs.failure.required` requires `diagnostic_category` on every non-null failure;
- the outer runner's `batch_timeout` and `cycle_failed` shapes legitimately omit it;
- retained historical v1 negatives also exist without that field;
- top-level `first_failure.diagnostic_category` accepts the new fine categories, but nested `first_failure.diagnostic.category` still accepts only the old generic five.

Preferred minimum shape: keep `diagnostic_category` optional/backward-compatible in v1; define one shared diagnostic-category enum in `$defs` and reference it from both locations; preserve `additionalProperties=false`; do not rewrite historical artifacts. Add actual JSON-Schema validation of at least one retained historical artifact and one newly categorized no-row batch.

### RWFDIAG-003 — HIGH / still open — inner stage ownership is inferred from prose

`run-live-warm-failover-cycle.py` still uses `category_for(message)` substring matching. The current synthetic tests therefore encode false specificity: duplicate `udp/tcp_negotiated` evidence can become `negotiation_auth`, duplicate readiness evidence can become `readiness`, and duplicate DeliveryAck evidence can become `application_runtime`, even though those cases establish only malformed/duplicate/cardinality **evidence**.

Do not replace this with a bigger logging framework. Use explicit ownership at the smallest existing seam. A good minimal implementation shape is:

1. setup/config/checkout/binary/endpoint/server-start failures are explicitly constructed/raised as `startup_setup`;
2. malformed JSON, duplicate events, cardinality, ordering, timing/accounting/summary contradictions are explicitly `evidence_serialization`;
3. `negotiation_auth`, `readiness`, and `application_runtime` are emitted only from explicit validated runtime evidence/failure points whose earlier gates are already known valid — never merely because an error string contains `tcp`, `auth`, `readiness`, `delivery`, or `timeout`;
4. an unmarked early/nonzero process death falls back conservatively (`nonzero_exit`/generic outer category) rather than manufacturing a phase.

The agent may choose a tiny phase enum/state, stage-aware helper, or explicit `CollectionError(..., category=...)` at owned call sites. Choose the form with the least new state/API and easiest fail-closed tests.

### RWFDIAG-004 — HIGH / still open — cleanup can mask an earlier startup failure

Current collector saves `startup_error`, always runs cleanup, then parses cleanup **before** re-raising `startup_error`. A malformed cleanup result raises `CollectionError("cleanup command did not return bounded evidence")` first and discards the earlier known startup boundary.

Repair with explicit primary-error precedence while still always executing cleanup:

- retain the earliest execution/collection primary error;
- run and parse cleanup regardless;
- if an earlier primary exists, emit/raise that primary category even when cleanup also fails;
- only use `cleanup` as the primary category when no earlier primary failure exists;
- keep any secondary cleanup fact in the existing private/sanitized diagnostic or independent cleanup observation unless adding a tracked secondary field is strictly necessary.

Required deterministic negative: earlier startup failure + malformed/failing cleanup; prove cleanup executes but the emitted primary marker remains `startup_setup`.

### RWFDIAG-005 — MEDIUM / historical live-boundary preservation

Preserve exact `3aa4828` and `docs/notes/repeated-warm-failover-2b784a3-typed-negative.md` unchanged as a historical changed-hypothesis observation. Safe claim remains: one exact-`2b784a3` changed outer invocation produced no cycle row/prefix/runtime result, the collector emitted a diagnostic then labelled `cleanup`, and later independent cleanup verification found zero residue. Until RWFDIAG-002/003/004 close, that label is not an authoritative root-cause stage and the run is not an accepted schema-valid typed closure.

### RSEC-001 — HIGH for RC/security promotion, independent from the current outward closure

Pre-auth accounting remains a real release/security package: source projection, charge-before-parse/response ordering, all real responder/listener call sites, concurrent reservation/rate-window/expiry/release evidence and exact-tree review remain open. Do not invent or change candidate numeric limits merely to make tests pass. This remains after the current outward repeated-failover package unless a new independent blocker changes ordering.

## Evidence / claim boundaries

- Default `main` before this handoff is exact `921ded1`; its CI is green. It contains the already accepted migration-back + endpoint-rebinding lineage plus the prior reviewer handoff.
- Execution exact `04cdf7a` has green exact-head CI and closes only the local EPREB-011 regression guard. It does not add or alter endpoint WAN evidence.
- Exact `2b784a3` code exists and was locally/CI validated, but its fine-category evidence contract remains rejected pending RWFDIAG-002/003/004.
- Exact `3aa4828` is a real self-owned changed-hypothesis negative observation, not a completed WAN failover cycle, reliability result, or accepted root-cause classification.
- Historical repeated-failover negatives (`4a2129e`, `9fd2411`, `a117086`, `c6ab8fd`) remain immutable and must continue to validate under the claimed v1 compatibility boundary.
- Endpoint rebinding exact `a8f49fc`/`7405da4`, migration-back exact `5d6582c`/`f024458`, and live key-update exact `2f4f59a`/`69d0ed9` remain accepted only for their previously stated bounded questions. No unchanged reruns.
- Live PLPMTUD remains implementation/design blocked; do not invent probe/ACK wire fields, timer/cooldown or new numeric policy as filler.
- IPv6 remains environment-blocked.
- HY2 still has no fair paired comparison; no performance conclusion exists.
- `RELEASE_CANDIDATE=false`, `PRODUCTION_READY=false`, `FREEZE=false`, `RELEASED=false` remain unchanged.
- Protected identities, credentials, SSH material, private endpoints and raw private diagnostics remain unread/untracked/uncommitted.

## Rolling Work Queue

This is a continuous dependency-ordered queue. A commit, nominal hour, reviewer interval, green test, or negative VPS attempt is not a stop condition while a safe dependency-satisfied slice remains.

### A — Repair repeated-failover diagnostic correctness as one bounded package

**Status:** `READY_LOCAL`, HIGH, queue head. Proposal authority applies.

Goal: close RWFDIAG-002/003/004 together. Do not split them into separate schema/checker-only commits unless an intermediate checkpoint is genuinely needed for safety.

Primary files: `scripts/bench/run-live-warm-failover-cycle.py`, `scripts/bench/run-repeated-warm-failover.py`, `scripts/bench/run-live-warm-failover-cycle-test.py`, the existing outer-runner tests, and `schema/repeated-warm-failover.v1.json`.

Protected invariants: valid six-cycle row semantics unchanged; controlled application reply-cessation classification unchanged; historical artifacts immutable; raw diagnostics remain private/redacted/hash-only; cleanup always runs; no Session/Carrier/ACK/crypto/wire behavior changes.

Required tests/behavior:

1. one retained historical v1 negative validates against the current schema;
2. one new fine-category no-row batch validates against the same v1 schema;
3. top-level and nested category fields share the same accepted enum when present;
4. duplicate negotiation/readiness/DeliveryAck evidence classifies as `evidence_serialization`, not a runtime stage;
5. a genuine explicitly established startup/negotiation/readiness/application/cleanup failure can still propagate the corresponding category;
6. earlier startup failure + cleanup failure preserves startup as primary while proving cleanup executed;
7. unknown/unmarked nonzero exit remains conservative rather than being guessed from prose.

Commit and push the complete repair after focused tests pass. Do not run the VPS before B.

**Continue immediately to B:** yes.

### B — Local and exact-head gate for A

**Status:** `PREAUTHORIZED_AFTER_A`.

Run focused inner/outer tests, real JSON-Schema validation of historical + new negative samples, `./scripts/check.sh`, and `git diff --check`. Fuzz is unnecessary unless parser/wire codec changes, which this package should not do.

Push and require green exact-head CI. If the schema regression exposes another pre-existing repeated-failover v1 incompatibility, repair only what is needed for the retained/current repeated-failover corpus; do not broaden into a repository-wide schema rewrite.

**Continue immediately to C after green:** yes.

### C — Exactly one fresh materially changed repeated-warm-failover VPS attempt

**Status:** `PREAUTHORIZED_AFTER_B_GREEN`; highest-value VPS opportunity.

Standing authorization already covers this bounded self-owned TCP/UDP failover experiment. Do not request WAN permission again.

The A/B changes materially alter instrumentation/category ownership/error precedence, so exactly one new attempt is justified. Preserve exact commit/binary, actual ports/parameters, start/end, valid cycle prefix if any, typed primary boundary only when actually established, row fields only when a row exists, and cleanup. Preserve a negative result. Do not rerun the same classified failure without another material hypothesis/code/config/path change.

A diagnostic stage is an evidence boundary, not a root-cause claim.

**Continue immediately to D:** yes.

### D — Reconcile C and perform at most the next evidence-producing repair

**Status:** `PREAUTHORIZED_AFTER_C`.

If C creates a new fact, update one compact evidence/status reconciliation. Do not rewrite exact `3aa4828`.

If C exposes one concrete orchestration defect inside existing architecture, the coding agent may propose the smallest repair, test/commit/push it, and only after that material change + green exact-head gate perform at most one further bounded attempt. If C repeats the same stage without a new safe hypothesis, freeze the negative and close the current repeated-failover line; do not grow more diagnostic infrastructure.

### E — Integrate the accepted closure lineage to default main once

**Status:** `PREAUTHORIZED_AFTER_D`.

EPREB-011 is already closed at `04cdf7a`, so no additional endpoint work is required before integration. After A-D are reviewable/green with no new HIGH blocker, merge current reviewer `main` into the execution branch while preserving reviewer ownership of `docs/CHATGPT_HANDOFF.md`, then integrate the accepted implementation/evidence lineage to default `main` without history rewriting. Observe exact-main CI.

Do not create coordination-only merges after tiny commits. One closure integration is enough.

### F — HY2 direct-unlock opportunity only

**Status:** `REVIEW_READY_AFTER_E`.

Re-read the latest retained `hy2-1 client_exit` evidence and fair-pair harness. If one minimal security-neutral instrumentation/configuration seam can directly unlock a same-condition paired owned-VPS run, implement/test it and make at most one materially changed attempt under standing authorization. Preserve negative/slower results.

If the work would become a generic harness project, require new credentials/permissions, touch an existing production HY2 service, or still cannot produce a fair pair, leave HY2 blocked and continue to G.

### G — Focused RSEC-001 adversarial release/security closure

**Status:** `READY_LOCAL_AFTER_OUTWARD_CLOSURE`.

Re-read exact current pre-auth implementation plus `docs/reviews/resource-abuse-evidence-2026-09-04.md`. Build one bounded adversarial package around the **existing** candidate limits: source projection, charge-before-parse/response ordering, every real responder/listener call site, concurrent reservations, rate-window rejection, expiry/release, fail-closed cleanup, and secret-safe diagnostics. Do not invent new numeric policy.

Close only evidence the exact tree proves. If a new safe VPS-ready outward capability appears while this security package is underway, preserve a parallel/follow-up exit rather than letting audit infrastructure monopolize the project.

## Stop / escalation boundary

Do not notify the maintainer for A-G. Escalate only if a repair truly requires changing core Session/Carrier/ACK/crypto/wire semantics, selecting a new numeric security policy/ADR value, destructive migration, production impact, new credentials/server/third-party rights, an experiment outside standing authorization, a benchmark objective requiring maintainer value judgment, or a major security issue that cannot be resolved fail-closed inside the current architecture.