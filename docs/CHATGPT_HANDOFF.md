# Nekomusume ChatGPT Handoff

Checked at: 2026-09-08 20:00 Asia/Shanghai
Repository main HEAD reviewed before this handoff: `a73f44ef0aae61f6b2aea68d5be324cb4dd0556e`
Previous reviewer handoff commit: `a73f44ef0aae61f6b2aea68d5be324cb4dd0556e`
Previous checked implementation/evidence HEAD: `9a94922794110ca0d9c0d8878f9903173d2de495`
Current execution branch: `work/e1a-staged-accounting-20260907` at exact `99996226815a58a99270b16c1bd62cd1773e3bed`
New implementation commit under review: `f17b648690fed63cce6f52b6f24c4a79edc51695` (`fix: preserve warm failover primary stage`)
New evidence/reconciliation commit under review: `99996226815a58a99270b16c1bd62cd1773e3bed` (`docs: retain post-fix failover boundary`)
Exact GitHub Actions: main `a73f44e` run `34219166076` — `success`; implementation `f17b648` run `34220263467` — `success`; current execution `9999622` run `34222644026` — `success`.
Branch topology at review remains intentionally diverged from merge base `1b5625d4f7c0c7063684690805fe5fda662a41c8`; the work branch now contains the implementation/evidence closure lineage while main contains the reviewer-only handoff lineage. This truth split is now old enough to close after the small local contract cleanup below; do not create multiple coordination-only merges.

## What changed

Exact `f17b648` closes the material parts of the prior startup-attribution repair. Startup polling now preserves parser/identity/cardinality errors already owned as `evidence_serialization`; malformed JSON again has a deterministic evidence-serialization regression; a real inner combined scenario proves that server exit before a valid start remains the primary `startup_setup` failure even when cleanup actually runs and returns malformed evidence; and tracked diagnostic SHA-256 validation is restored to exact 64 lowercase hexadecimal characters. Exact-head CI is green.

After that gate, the coding agent made exactly one reviewer-authorized post-fix self-owned repeated-warm-failover attempt and retained it at exact `9999622`. The experiment binary was exact `f17b648`, CI run `34220263467` was green, the outer invocation ran from `2026-09-08T11:45:31Z` to `11:45:37Z`, and the bounded plan remained at most six sequential cycles with one local client / one owned VPS server per cycle, three 16-byte records and controlled application-level UDP reply cessation. It retained zero valid cycles and stopped before cycle 1 with `invalid_cycle_evidence`, primary `diagnostic_category=startup_setup`, sanitized diagnostic SHA-256 `5867711ad01f5cc505e982f9b5956b4b05d08758c2071f4fe6d5a6edaec4d101`, 160 bytes, `truncated=false`. Separate post-run cleanup observation found zero listeners/processes and the deployment was removed.

Crucially, this post-fix result repeats exact `9a94922`'s same category/hash/size. That is an evidence boundary, not a root-cause diagnosis and not runtime-failover evidence. It consumes the changed-hypothesis retry authorized by the previous handoff. The repeated-warm-failover WAN line is therefore **frozen against another same-class retry** until a concrete new setup hypothesis plus a material code/configuration/path change exists. Do not keep growing diagnostics merely to chase a PASS.

The 24–48 hour visible-output check remains healthy: accepted live key-update, scripted migration-back/recovery, authenticated source-endpoint rebinding, deterministic promotion-ordering protection and now a truthful repeated-failover negative all exist. The diagnostic work has reached diminishing returns and must now be closed rather than recursively expanded.

## Review verdict

**ACCEPT_EXACT_`f17b648`_CORE_ATTRIBUTION_REPAIR; ACCEPT_EXACT_`9999622`_AS_ONE_BOUNDED_POST-FIX_NEGATIVE_WITH_ITS_NARROW_BOUNDARY; FREEZE_SAME-CLASS_REPEATED-FAILOVER_WAN_RETRIES; REQUIRE_ONE_SMALL_LOCAL_CONTRACT/REGRESSION_CLEANUP_BEFORE_INTEGRATION; THEN_INTEGRATE_AND_MOVE_TO_THE_NEXT_OUTWARD_OR_RELEASE-CLOSURE_LANE.**

There is no new core Session/Carrier/ACK/crypto/wire architecture change, no new numeric security policy, no destructive migration, no new credential requirement and no production action. Proposal authority applies. This is active implementation/evidence progress, not `STALLED_IMPLEMENTATION`.

## Reviewer findings

### RWFDIAG-003 — CLOSED for the live defect that blocked the post-fix attempt

The prior timing-dependent startup-poll rewrite is fixed for explicitly owned `evidence_serialization` errors. `event_objects`, stream identity checks and duplicate/cardinality checks keep their evidence category even when observed during startup polling. Actual server exit before a valid start and invalid start-contract fields remain startup/setup facts. The malformed-JSON regression is back to `evidence_serialization`.

This is sufficient to accept the single exact-`f17b648` post-fix live attempt for the narrow `startup_setup` evidence boundary it actually emitted. Do not promote that label to a root cause.

### RWFDIAG-004 — CLOSED

The inner collector now has a real combined regression: the server exits before a valid start, cleanup is definitely invoked, cleanup returns malformed bounded evidence, a test-only marker proves cleanup ran, stdout remains empty, and the emitted primary remains `startup_setup`. Earlier primary failure therefore wins over later cleanup failure while cleanup still executes.

### RWFDIAG-002A — CLOSED

Tracked `first_failure.diagnostic.sha256` is again exactly `^[0-9a-f]{64}$`. Historical optional-category compatibility remains separate from that hash contract.

### RWFDIAG-005 — MEDIUM / two pre-spawn ownership holes remain

The collector still has two small local attribution holes that contradict the intended explicit setup ownership, although they do **not** invalidate the exact `f17b648` negative just retained:

1. `requires()` raises uncategorized `CollectionError` for a missing required command token/value. The outer fallback converts uncategorized failures to conservative `nonzero_exit`; these are actually pre-spawn command-contract failures and should be explicit `startup_setup`.
2. `float(NEKO_FAILOVER_SERVER_STARTUP_TIMEOUT_SECONDS)` is parsed outside a typed conversion guard. A non-numeric value reaches the generic `ValueError` catch and becomes `nonzero_exit`, even though invalid startup-timeout configuration is also a setup/configuration failure.

Minimum repair: give both `requires()` failure forms `startup_setup`; wrap startup-timeout conversion so malformed/non-finite/out-of-range configuration deterministically yields `startup_setup`; add small focused regressions for missing required token/value and malformed timeout. Do not add new diagnostic categories or a general phase framework.

This repair is **local-only**. It does not create a new WAN hypothesis and therefore must not be followed by another repeated-failover VPS retry.

### RWFDIAG-006 — MEDIUM / historical schema corpus regression coverage was narrowed unnecessarily

Exact `9a94922` had a real Draft 2020-12 regression over every retained `artifacts/repeated-warm-failover/*/result.json`. Exact `f17b648` narrows that test to only `a117086-typed-negative/result.json` while changing the SHA pattern.

The reviewer re-read the current retained corpus: `4a2129e` and `c6ab8fd` carry full 64-hex diagnostic hashes with no fine category; `9fd2411` and `a117086` carry no diagnostic object/category. Their shapes remain compatible with the intended current schema, so this is a **coverage regression, not evidence corruption**. Restore the corpus-wide glob validation and keep the fresh fine-category generated-batch validation. Historical artifacts stay immutable.

### RWFDIAG-007 — MEDIUM / status text has a stale next-seam sentence

Current work-branch `docs/status.md` correctly classifies repeated warm failover as `BLOCKED_ORCHESTRATION_CURRENT_LINE` at exact `f17b648`, but the immediately following paragraph still says the next seam is “local sanitized inner-collector failure categorization”. That categorization seam now exists and produced the post-fix negative. `ROADMAP.md` also retains an older sentence grouping repeated warm failover under `BLOCKED_DIAGNOSTICS` before describing the newer current-line boundary.

During the closure reconciliation, remove those stale statements without rewriting historical evidence. The current truth is: same-class repeated-failover WAN retry is closed absent a new material setup hypothesis; the retained negative is not a runtime failover result; release evidence item 3 remains open.

### RSEC-001 — HIGH for RC/security promotion, independent from the frozen failover line

Pre-auth admission implementation exists, but independent exact-tree review and adversarial concurrency/rate/expiry/release evidence remain open. The current security review requires source projection, charge-before-parse/response ordering, every real listener/responder call site, concurrent reservations, rate-window rejection, expiry/release, fail-closed cleanup and secret-safe counters. Candidate numeric limits must not be changed merely to make tests pass.

RSEC-001 blocks release/security/public-listener promotion, not bounded authenticated research probes. Once the current outward/evidence lineage is integrated, it becomes a primary local closure lane unless the HY2 fair-pair question can be unlocked with one small security-neutral seam.

## Evidence / claim boundaries

- Default `main` before this handoff is exact `a73f44e`; exact-main Rust CI run `34219166076` is green.
- Exact implementation `f17b648` has green exact-head Rust CI run `34220263467`. Current work/evidence HEAD `9999622` has green exact-head Rust CI run `34222644026`.
- Exact `9999622` retains one post-fix repeated-warm-failover negative only. `completed_cycles=0`; therefore there is no retained runtime negotiation/auth/readiness/failover/application-accounting/timing/per-cycle resource conclusion from that invocation.
- The repeated `startup_setup` category/hash/size relative to exact `9a94922` closes this current WAN line against unchanged or category-only reruns. A stage label is not a causal diagnosis.
- Exact `3aa4828`, `9a94922`, `2b784a3`, `4a2129e`, `9fd2411`, `a117086` and `c6ab8fd` remain immutable historical evidence; do not rewrite old artifacts to match newer categories.
- Endpoint rebinding exact `a8f49fc` / `7405da4`, migration-back exact `5d6582c` / `f024458`, and live key-update exact `2f4f59a` / `69d0ed9` remain accepted only for their previously stated bounded questions. No unchanged reruns.
- Live PLPMTUD remains implementation/design blocked; do not invent probe/ACK wire fields, timers/cooldowns or new numeric policy as filler.
- IPv6 remains environment-blocked.
- HY2 still has no fair paired comparison and no performance conclusion. Existing Hysteria service must not be touched as production infrastructure; only temporary owned experimental instances inside standing authorization are eligible.
- `RELEASE_CANDIDATE=false`, `PRODUCTION_READY=false`, `FREEZE=false`, `RELEASED=false` remain unchanged.
- Protected identities, credentials, SSH material, private endpoints and raw private diagnostics remain unread/untracked/uncommitted.

## Rolling Work Queue

This is a continuous dependency-ordered queue. A commit, nominal hour, reviewer interval, green test or negative VPS attempt is not a stop condition while a safe dependency-satisfied slice remains.

### A — Close the remaining local repeated-failover contract gaps

**Status:** `READY_LOCAL`, MEDIUM, bounded final cleanup. Proposal authority applies.

Goal: finish RWFDIAG-005/006 without adding any new diagnostic machinery.

Primary files: `scripts/bench/run-live-warm-failover-cycle.py`, `scripts/bench/run-live-warm-failover-cycle-test.py`, `scripts/bench/run-repeated-warm-failover-test.py`.

Protected invariants: existing valid row semantics unchanged; historical artifacts immutable; no Session/Carrier/ACK/crypto/wire change; no new category enum; no raw private diagnostic retention.

Required behavior/tests:

1. missing required command token/value is explicit `startup_setup`;
2. malformed/non-finite/out-of-range startup-timeout configuration is explicit `startup_setup`;
3. restore real Draft 2020-12 validation over **all** retained repeated-warm-failover `result.json` artifacts plus the generated fine-category negative;
4. keep malformed/duplicate/identity evidence as `evidence_serialization` and keep the real primary-vs-cleanup combined regression.

Commit and push. **Do not run repeated-failover VPS after this slice.** It is not a new setup hypothesis.

**Continue immediately to B:** yes.

### B — Exact-head gate + current-state reconciliation

**Status:** `PREAUTHORIZED_AFTER_A`.

Run focused inner/outer failover tests, JSON-Schema tests, `./scripts/check.sh`, and `git diff --check`; fuzz is unnecessary unless parser/wire codec code changes. Push and require green exact-head CI.

Then make one compact developer-owned status/evidence reconciliation removing only stale current-state statements: repeated warm failover is `BLOCKED_ORCHESTRATION_CURRENT_LINE`, the current same-class live line is frozen without a material setup hypothesis, and “next seam is inner categorization” is no longer current. Do not rewrite old artifacts or inflate any WAN claim.

**Continue immediately to C:** yes.

### C — Integrate accepted closure lineage to default main once

**Status:** `PREAUTHORIZED_AFTER_B_GREEN`.

Merge current reviewer `main` into the execution branch while preserving reviewer ownership/content of `docs/CHATGPT_HANDOFF.md`, resolve only genuine conflicts, run the normal gate, and integrate the accepted implementation/evidence lineage to default `main` without history rewriting. Observe exact-main CI.

This should close the long-lived main/work truth split. Do not create additional coordination-only merges after tiny follow-up commits.

**Continue immediately to D after green:** yes.

### D — HY2 direct-unlock proposal or explicit skip

**Status:** `REVIEW_READY_AFTER_C`; outward/VPS-priority lane.

Re-read the latest retained fair-pair artifact/harness at the exact current tree and identify why `hy2-1` ends at `client_exit` without discriminating evidence. The agent must propose 1–3 minimal shapes and choose the smallest security-neutral seam that can distinguish harness/setup from actual HY2 client/runtime failure while preserving lifecycle/resource-accounting symmetry with Nekomusume.

Allowed implementation shape: bounded typed exit/stderr classification or an equivalent existing-harness observation that directly unlocks one same-condition pair. Do **not** build a generic diagnostic framework, alter benchmark workload/security levels to favor either implementation, touch an existing production Hysteria service, or require new credentials/permissions.

If no small seam can truthfully unlock a fair pair, record that as a developer note only if useful and **skip directly to F** rather than manufacturing work.

**Continue immediately to E only if D materially changes the hypothesis and exact-head CI is green.**

### E — At most one materially changed self-owned HY2 fair-pair attempt

**Status:** `PREAUTHORIZED_ONLY_IF_D_GREEN_AND_TRUTHFUL`.

Standing authorization already covers a bounded Nekomusume-vs-HY2 comparison on owned client/VPS endpoints. Run at most one attempt with the existing same-condition workload/security/lifecycle/resource contract. Preserve a negative or slower Nekomusume result exactly. No superiority claim follows from a one-off sample or incomplete pair.

If the attempt fails in the same class without a new hypothesis, freeze it; no mechanical retry. If it produces a complete comparable pair, reconcile only the actually supported statistics/claim boundary.

**Continue immediately to F:** yes.

### F — Focused RSEC-001 adversarial release/security closure

**Status:** `READY_LOCAL_AFTER_C` and also the fallback if D is not a direct unlock.

Re-read exact current `ProcessPreauthAdmission`, `ListenerAdmission`, every real TCP/UDP responder/listener call site and `docs/reviews/resource-abuse-evidence-2026-09-04.md`. Build one bounded closure package around **existing candidate limits**:

- source projection / tuple ownership;
- charge input before parse/auth work;
- charge exact response bytes before response send;
- every real responder/listener integration point;
- concurrent reservation success/rejection and atomic rollback;
- global/source rate-window rejection;
- idle/lifetime expiry and release;
- cleanup after errors/timeouts;
- redacted observable counters sufficient to prove accounting without secrets.

Prefer deterministic/process adversarial tests over VPS load. Do not invent or tune numeric limits. If a discovered bug can be repaired inside existing ownership/state-machine semantics, propose/implement/test it directly; only a required new numeric security policy or core architectural change escalates.

### G — Exact-tree RSEC reconciliation and next milestone decision input

**Status:** `PREAUTHORIZED_AFTER_F`.

Run full local gates and exact-head CI, then update the resource-abuse review/status only for what the exact tree proves. If RSEC-001 closes, mark the security evidence package closed but **do not** set RC/freeze/production/release flags. Re-read the remaining bounded release matrix and select the next real dependency-satisfied output lane; do not default back to checker/docs growth.

## Stop / escalation boundary

Do not notify the maintainer for A-G. Escalate only if a repair truly requires changing core Session/Carrier/ACK/crypto/wire semantics, selecting a new numeric security policy/ADR value, destructive migration, production impact, new credentials/server/third-party rights, an experiment outside standing authorization, a benchmark objective requiring maintainer value judgment, or a major security issue that cannot be resolved fail-closed inside the current architecture.