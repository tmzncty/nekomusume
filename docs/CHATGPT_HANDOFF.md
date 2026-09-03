# Nekomusume ChatGPT Handoff

Checked at: 2026-09-04 05:58 Asia/Shanghai
Repository HEAD reviewed: `d7553374ccb375dfced435caa933ba7fdc3d131e`
Previous reviewed coding/evidence HEAD: `85346ce19f9941ac5c41437713e0c1aee81b2102`
Previous reviewer handoff commit: `036adf9e954fd1080fe96104777d261c6a8bc6af`

## What changed

Three coding-agent commits landed after the previous review.

- `c5a253c` — **R-009 evidence-semantic repair; no Session/wire/Noise/carrier behavior change.** It adds an additive erratum for the immutable exact-`85346ce` periodic artifact, separates `capture_started` from `protocol_entered`, and makes `protocol_entered=true` depend on one exact structured `remote_exec_protocol_accepted` marker emitted only after remote binary identity/protocol acceptance. Deterministic tests reject empty/private/wrong markers.
- `347d59e` — **status/evidence reconciliation only.** It propagates the closed repeated-warm-failover and periodic orchestration boundaries through `IMPLEMENTATION_PLAN.md`, `ROADMAP.md`, `docs/status.md`, the Era-4 resilience/ledger material and reachability navigation, while preserving exact `25e0daa` positive bounded evidence and historical negatives.
- `d755337` — **HY2 diagnostic-harness hardening; no live HY2 comparison yet and no Nekomusume transport semantic change.** It retains bounded private diagnostic bundles under ignored `logs/`, records only fixed public metadata (category, bundle hash/size, timestamps, lifecycle field), bounds/redacts diagnostics, and extends deterministic tests. Exact `d755337` GitHub Actions run `33808766844` completed successfully (`Rust CI` conclusion `success`).

A/B/C from the previous rolling queue are therefore substantially consumed: R-009 is closed, status reconciliation is done, and the minimal HY2 diagnostic surface exists. No new live VPS comparative evidence was added in this delta.

Review found one new evidence-integrity blocker before the live HY2 slice.

### R-010 HIGH — `last_success_stage` can be promoted by failure text, not success evidence

Current `scripts/bench/validate-hy2-owned-lab.py` derives `client_diagnostic.last_success_stage` by searching arbitrary diagnostic text with generic regexes:

```text
quic_udp:  quic | udp | initial packet | connection established | handshake response
tls_authenticated: authenticated | authentication succeeded | connected to server | login succeeded
```

and then takes the maximum observed stage over the harness baseline.

That is not a valid success oracle. Examples such as:

```text
QUIC handshake failed
UDP timeout
not authenticated
```

can contain one of these tokens while proving failure or, at best, naming the subsystem. The current tests exercise positive-looking strings but do not prove that negative/negated/error strings cannot promote the stage. A future failed HY2 pair could therefore be truthfully classified as `client_exit` while simultaneously overclaiming `last_success_stage=quic_udp` or `tls_authenticated`.

This is an evidence-semantic defect in the new diagnostic metadata, not a HY2 runtime failure and not a Nekomusume protocol defect. Do **not** spend the next rented-VPS attempt until this field is fail-closed.

## Review verdict

**CONTINUE_WITH_REQUIRED_FIXES — R-009/status work accepted and exact-head CI green; close R-010 locally, then execute exactly one changed-hypothesis HY2/Nekomusume owned-lab attempt and keep consuming the rolling release queue without waiting between pre-authorized slices.**

The project is not globally blocked. The periodic and repeated-warm-failover current instrumentation lines remain closed and must not be retried unchanged. HY2 is still the only materially changeable live diagnostic opportunity in the current release-evidence matrix, but the new success-stage field must be made truthful first.

## Evidence boundaries

- `IMPLEMENTATION_COMPLETE=true` remains the bounded research baseline.
- `CANONICAL_CORPUS_V1_FROZEN=true` remains corpus-specific only.
- `RELEASE_CANDIDATE=false`, `PRODUCTION_READY=false`, `FREEZE=false`, `RELEASED=false` remain unchanged.
- Exact `25e0daa` remains the accepted bounded positive controlled warm fallback and one approximately five-minute periodic direct-path sample; neither is a natural-loss/reliability-rate/production proof.
- Repeated warm failover current line remains `BLOCKED_ORCHESTRATION_CURRENT_LINE`; no unchanged retry.
- Periodic current line remains `BLOCKED_ORCHESTRATION_CURRENT_LINE_PERIODIC`; exact `85346ce` is a pre-application `ssh_transport_exit` negative with R-009 additive erratum; no unchanged retry.
- `c5a253c` closes R-009 for future evidence; it does not retroactively change the immutable artifact.
- `d755337` adds bounded diagnostic retention/classification infrastructure only. It does not prove QUIC, TLS/authentication, application exchange, fair comparison, or any HY2 success/failure stage beyond what a future live artifact actually records.
- R-010 means the current `last_success_stage` field is not yet safe as a release-evidence claim for stages above the harness-known baseline.
- NAT/source-endpoint change, live migration-back, live key update and live PMTUD remain implementation-blocked under current executable reality; do not implement them merely to fill a matrix checkbox.
- Owned end-to-end IPv6 remains environment-blocked unless a real owned path appears.
- HY2 fair paired comparative samples, medians/P95 and superiority evidence remain absent.
- Endpoint/user/address/private topology, credentials, key paths, keys, raw private diagnostics and protected identity material stay local/untracked. `logs/` is ignored; only bounded sanitized classifications/hashes/counts belong in tracked evidence.
- Standing authorization covers the bounded self-owned work below. No per-run WAN approval is required.

## Rolling Work Queue

This is a multi-hour capacity queue. **Every dependency-satisfied slice is pre-authorized to begin immediately after the previous coherent slice is validated, committed and pushed. Do not stop after one commit or one nominal hour.** Pause only for a real stop condition in `AGENTS.md`, standing authorization, a new BLOCKER/HIGH correctness-security-evidence finding, an exact-head CI dependency for a live run, actual runtime/tool-budget termination, or true queue exhaustion.

### A — Make HY2 lifecycle-stage evidence fail closed

**Status:** `READY_LOCAL` — highest priority due R-010.

**Goal:** keep useful diagnostic category/private-bundle evidence while ensuring `last_success_stage` never advances merely because an error message mentions QUIC/UDP/TLS/authentication.

**Likely files**

- `scripts/bench/validate-hy2-owned-lab.py`;
- `scripts/bench/compare-hy2-owned-lab-test.sh`;
- schema/validator docs only if the public diagnostic contract changes.

**Required behavior**

1. Generic subsystem/error keywords may classify `category`; they must **not** by themselves prove a successful lifecycle stage.
2. `last_success_stage` may always include the harness-known baseline (`server_bound` or `client_started`) because the orchestrator itself proves that transition.
3. Promotion to `quic_udp` or `tls_authenticated` requires a mechanically positive discriminator, not mere keyword presence. Acceptable approaches include:
   - an exact structured marker emitted by repository-controlled instrumentation after the positive transition; or
   - trustworthy bounded capture/structured state already collected by the harness and explicitly mapped to that transition.
4. If no such positive discriminator is currently available for pinned HY2, leave `last_success_stage` at the harness-known baseline. It is better to report less than to infer success from prose logs.
5. Do not weaken TLS/authentication, change HY2/Nekomusume application workload semantics, or add production protocol behavior.
6. Keep private diagnostic bundles bounded, redacted, mode 0600 and untracked.

**Adversarial tests required**

At minimum prove that none of these can promote a stage above baseline:

```text
QUIC handshake failed
UDP timeout
no QUIC response
not authenticated
authentication failed
TLS handshake failed
connected to server: false
```

Also prove that an exact positive structured marker, if implemented, promotes only the intended stage and that duplicate/malformed/contradictory markers fail closed or do not promote.

**Validation / commit**

targeted diagnostic tests -> full local gate -> `git diff --check` -> commit/push.

**Continue immediately to B:** yes.

---

### B — Rehearse the exact first-pair HY2 path locally and freeze the live-attempt contract

**Status:** `READY_LOCAL` after A.

**Goal:** before touching the VPS, prove the live path will consume the new fail-closed stage semantics and cannot create a partial comparative summary.

Required deterministic/local rehearsal:

- first Nekomusume sample succeeds under the existing fair lifecycle contract;
- first HY2 sample fails at each representative diagnostic class without producing median/P95 or a complete-pair claim;
- last-success stage never exceeds mechanically proven evidence;
- diagnostic bundle hash/size/timestamps match the local ignored bundle;
- no raw address/credential/key/path/private string enters tracked JSON;
- cleanup remains fail closed;
- exact payload bytes/hash and pinned HY2 v2.9.3 identity remain unchanged;
- one substantive live attempt remains the next action; do not add another generic harness layer.

Run the full local gate and push. Wait for exact B-head GitHub CI before C. While CI is pending, consume E/F/G/H independent local work rather than idle.

**Continue immediately to C after exact-head CI green:** yes.

---

### C — One changed-hypothesis HY2/Nekomusume owned-lab attempt

**Dependency:** B exact-head CI green.

Run exactly one bounded live attempt under standing authorization:

- self-owned client + owned VPS only;
- pinned HY2 v2.9.3 and exact Nekomusume binary/commit identity;
- deterministic 1200-byte payload;
- concurrency 1;
- fresh unprivileged experiment ports;
- same route/time-window/MTU/security/load class as the comparison contract;
- total live invocation comfortably below 10 minutes;
- bounded capture only around experiment ports/window when it adds a discriminating fact;
- no firewall/route/qdisc/DNS/provider/production-service change;
- complete cleanup verification.

Execution rule:

- if the first required Neko/HY2 pair fails, stop that live invocation, retain the valid prefix and the typed new diagnostic, and **do not automatically retry again**;
- only if the first pair succeeds may the existing contract continue to the complete five paired samples;
- only a complete success set may produce median/P95/resource comparison;
- one batch cannot justify a superiority claim.

For a failed HY2 first pair, the tracked evidence may state only mechanically supported facts: failure stage/category, baseline/positive lifecycle stage actually proved, bounded bundle hash/size/time, packet-direction facts only if capture proves them, exit/cleanup. Do not promote a stage from arbitrary log wording.

**Commit/push:** minimal sanitized evidence + validators + full gate.

**Continue immediately to D:** yes.

---

### D — Reconcile the HY2 result into the release matrix and close/reclassify the line

**Status:** `READY_LOCAL` after C, regardless of C success or honest block.

Update exact evidence only in the established status/plan/ledger/navigation files.

- If C first-pair fails: retain the typed diagnostic and classify the current HY2 line at the proven cause/stage; no comparative summary; no automatic unchanged retry.
- If all required pairs succeed: link raw complete samples and bounded summary; keep claims limited to this owned route/time window and batch.
- Preserve all historical HY2 negatives separately.
- Do not convert diagnostic uncertainty or orchestration inability into PASS.
- Bounded release-evidence matrix item 3 remains unchecked unless **all** declared requirements are honestly satisfied.
- Governance flags remain unchanged.

**Continue immediately to E:** yes.

---

### E — Build/refresh the release-evidence closure ledger and identify `OPEN_READY` work

**Status:** `READY_LOCAL`; can be consumed while B-head CI is pending if it does not conflict.

Maintain one machine-navigable blocker/coverage map for every declared release-evidence row:

- required claim;
- best exact positive evidence;
- retained negatives;
- classification from a closed set such as `ALREADY_SUFFICIENT_FOR_BOUNDED_QUESTION`, `BLOCKED_ORCHESTRATION_CURRENT_LINE`, `BLOCKED_DIAGNOSTICS`, `BLOCKED_IMPLEMENTATION_ARCHITECTURE`, `BLOCKED_ENVIRONMENT`, `OPEN_READY`;
- exact evidence needed to change classification;
- whether doing so needs new architecture, environment, authorization or maintainer value judgment.

Do not silently narrow release scope. If no `OPEN_READY` VPS row remains after C/D, stop generic rental-time repeats and continue local release/security work below.

**Continue immediately to F:** yes.

---

### F — Prepare the independent release/security review packet

**Status:** `READY_LOCAL`; may run while live-run CI is pending.

Preparation only; this does **not** complete independent review item 4.

Create/update a machine-navigable packet linking exact evidence for:

- canonical corpus/frozen identity/executable oracles;
- negotiation compatibility and transcript binding;
- Noise/trust/authz boundaries;
- wire/parser/fuzz evidence;
- pre-auth/resource limits;
- Session delivery ACK vs packet/carrier feedback separation;
- failover/resume positive and negative evidence with boundaries;
- package install/upgrade/rollback;
- operator readiness/shutdown/cleanup;
- VPS matrix classifications;
- HY2 methodology/result boundary;
- unresolved release/security findings.

Prefer links/index generation over duplicated normative prose. Never label the packet an audit or security approval.

**Continue immediately to G:** yes.

---

### G — Audit resource/abuse-limit evidence; close only small deterministic gaps

**Status:** `READY_LOCAL`.

Map implementation/tests against `SECURITY.md` for:

- per-connection/global memory, CPU/rate bounds;
- UDP amplification/pre-auth admission;
- malformed lengths/counts/offsets;
- unknown version/type/frame handling;
- duplicate/old packet numbers and replay;
- fail-closed resource behavior;
- secret-safe logging and no open-proxy behavior.

Do not duplicate existing tests. If one **small deterministic negative-test gap** exists, add it and run gates. If a substantive design/resource-bound gap appears, record a release/security blocker rather than inventing arbitrary production limits.

**Continue immediately to H if no new BLOCKER/HIGH:** yes.

---

### H — Package/operator evidence link-integrity audit

**Status:** `READY_LOCAL`.

N5/package lifecycle already has positive evidence; do not repeat it merely for freshness. Verify an independent reviewer can trace:

- reproducible package/build identity;
- install -> readiness/smoke -> upgrade -> rollback;
- retained external-state boundary without reading protected identity material;
- listener/process cleanup;
- exact commit/artifact hashes;
- x86_64 first-RC target boundary.

If evidence is sufficient, produce only an index/closure note. A single bounded package rehearsal is pre-authorized **only** if the audit first identifies one narrow executable assertion that genuinely lacks evidence and the run remains inside standing authorization without touching protected identity material.

**Continue immediately to I:** yes.

---

### I — RC-scope decision dossier if the matrix has no remaining READY path

**Status:** `READY_LOCAL_PREPARATION`; any release-scope change itself is **not** pre-authorized.

If E shows the bounded release matrix cannot complete because every remaining requirement is only orchestration-, implementation-, architecture- or environment-blocked, prepare a concise maintainer decision dossier without choosing among:

1. keep current release-evidence scope and remain pre-RC;
2. define a narrower first-RC claim, explicitly deferring/removing named claims;
3. authorize the implementation/environment work needed to satisfy the current matrix.

For each option state evidence consequences, compatibility/security risk, work required and which existing artifacts remain valid.

Do not alter `IMPLEMENTATION_PLAN.md` release scope, `RELEASE_CANDIDATE`, production readiness, global freeze or release flags until maintainer choice.

**Stop after I only if this is genuinely the first remaining gate requiring maintainer value judgment.**

## Completion gates for this rolling queue

- R-010 is closed with adversarial evidence that failure/error text cannot promote a success stage.
- Exact repair-head local gates and GitHub CI are green before any live HY2 attempt.
- At most one materially changed HY2 live attempt occurs for this diagnostic hypothesis; unchanged retry is prohibited.
- Any failed first pair produces no comparative median/P95 and preserves a typed bounded diagnostic with truthful lifecycle evidence.
- Any successful complete pair set remains bounded to one owned route/time window and carries no superiority claim.
- Status/plan/ledger preserve exact positives, historical negatives and blocked classifications without retroactive rewriting.
- Release/security packet and resource/operator audits are navigable and do not self-certify independent review.
- `RELEASE_CANDIDATE=false`, `PRODUCTION_READY=false`, `FREEZE=false`, `RELEASED=false` remain unchanged unless a later explicit reviewed decision changes them.
- Protected `neko-server.identity` / credentials / private topology / raw private logs remain unread/untracked/uncommitted.

## Do not expand into

- another generic repeated-warm-failover or periodic retry on the closed current lines;
- repeated HY2 retries after the one C attempt without a materially new hypothesis;
- firewall/route/qdisc/DNS/provider/production-service changes;
- third-party targets or scanning;
- bare-insecure TLS or weakened auth/integrity for benchmark convenience;
- NAT/migration/key-update/PMTUD implementation merely to fill the release matrix;
- IPv6 claims without a real owned path;
- 0-RTT, enabled FEC, striping/aggregation or exotic carriers without an observed-problem gate;
- RC/production/release/global-freeze state changes before their explicit reviewed gates.

## Questions requiring maintainer decision

none at this review. If I becomes the first remaining gate, prepare the dossier and then request the maintainer value decision rather than choosing release scope autonomously.
