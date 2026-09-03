# Nekomusume ChatGPT Handoff

Checked at: 2026-09-04 04:01 Asia/Shanghai
Repository HEAD reviewed: `df4610b2915bc5be97b51133afbdf2937bfc9ae5`
Previous reviewed coding/evidence HEAD: `60cd40d612b5582337c9fa04cc28b35aed98f322`
Previous reviewer handoff commit: `4a2129ec01bcbd35fd1d6fd2952ef45390fd549a`

## What changed

Two coding/evidence commits are newly reviewed after the previous handoff, and two previously uncommitted live outcomes are now durably archived.

- `34e587d` — **periodic evidence-oracle repair; evidence tooling only, no wire/Session/Noise runtime semantic change.** It closes the prior R-008 success-predicate defect by normalizing and matching server/client workload parameters, rejecting duplicate/mismatched argv options before spawn, requiring complete declared application delivery for `passed`, parsing/enforcing `reconnects=0`, rejecting signal-interrupted samples, recomputing nearest-rank P50/P95 from interval rows, preserving client duplicate-ACK and server duplicate-data as distinct domains, and removing the synthetic `conflicts=0` claim. Deterministic regressions cover incomplete success, workload mismatch, reconnect, signals, bad percentiles and privacy/cleanup boundaries.
- `df4610b` — **evidence/status reconciliation only.** It archives the final repeated-warm-failover current-line attempt and the prior periodic live attempt without rewriting their original failure/cleanup facts, updates resilience/capability evidence references, and preserves exact hashes and boundaries.

Exact current HEAD `df4610b` has GitHub Actions run `33799316682` completed successfully (`Rust CI` conclusion `success`). Exact `34e587d` also has green run `33798025997`. This is repository CI evidence, not release/security approval.

### Accepted final repeated-failover boundary

The exact-`4a2129e` final allowed repeated warm-failover attempt made one outer live invocation, no retry, retained `0/6` rows, and stopped at cycle 1 after 1,165 ms with `invalid_cycle_evidence`. The bounded private diagnostic classification is `server exited before JSON event: start`. The lane is now **`BLOCKED_ORCHESTRATION_CURRENT_LINE`**. This is not a runtime failover failure and must not receive another automatic repeated-failover retry under the same instrumentation line.

### Accepted periodic boundary

The exact-`60cd40d` periodic live attempt made one no-retry invocation and ended pre-application at `start_timeout`: SSH server transport exit 255, `periodic_client_entered=false`, client `not_started`, no application metrics. Artifact cleanup remains failed as originally collected; a later cleanup-only observation separately found zero local/remote residue. This is a pre-application orchestration negative, not a Session reliability sample.

Important consequence: `34e587d` closes the semantic-oracle defect but does **not** change the SSH exit-255 failure hypothesis. Therefore an immediate unchanged periodic retry is prohibited. The next periodic work must first add a narrow discriminating remote-launch diagnostic seam, then one materially changed live attempt may be made.

## Review verdict

**CONTINUE_WITH_REQUIRED_FIXES — R-008 is closed and current CI is green. Repeated warm failover is closed as `BLOCKED_ORCHESTRATION_CURRENT_LINE`. Periodic remains potentially valuable but requires one narrow SSH/remote-launch diagnostic change before a final changed-hypothesis live sample. The project is not globally blocked.**

Do not return to broad repeated-failover harness rewrites. Do not spend VPS time on an unchanged periodic retry. Consume the rolling queue continuously: finish -> validate -> commit/push -> immediately continue when the next slice is dependency-satisfied.

## Evidence boundaries

- `IMPLEMENTATION_COMPLETE=true` remains the bounded research baseline.
- `CANONICAL_CORPUS_V1_FROZEN=true` remains corpus-specific only.
- `RELEASE_CANDIDATE=false`, `PRODUCTION_READY=false`, `FREEZE=false`, `RELEASED=false` remain correct.
- Exact `25e0daa` remains the accepted single controlled application-level UDP reply-cessation warm fallback: 3/3 logical records, 48 B, two uncertain/replayed, duplicate/lost 0, ~434 ms failure-decision-to-first-resumed-data. It is not natural loss/PTO-blackhole evidence.
- Exact `25e0daa` also remains the accepted approximately five-minute direct periodic sample: 60 x 32 B, 60/60 confirmed. It is one bounded sample, not a reliability rate.
- Exact `4a2129e` final repeated result is orchestration/evidence negative only; zero row means no per-cycle runtime/accounting/timing/reachability claim.
- Exact `60cd40d` periodic result is pre-application SSH/server-launch negative only; no application metrics or reliability conclusion.
- `34e587d` improves periodic evidence truthfulness but does not itself add live evidence or alter SSH behavior.
- NAT/source-endpoint change, live migration-back, live key update and live PMTUD remain `BLOCKED_IMPLEMENTATION` under current executable reality unless that reality changes for an evidence-driven reason. Do not implement them merely to fill a matrix checkbox.
- Owned end-to-end IPv6 remains `BLOCKED_ENVIRONMENT` unless a real owned path becomes available.
- HY2 remains `BLOCKED_DIAGNOSTICS`; historical negative attempts remain valid and there is still no complete fair paired comparison.
- Endpoint/user/address/private topology, private plans, identity paths, keys and raw diagnostics remain local/untracked. Only bounded non-sensitive classifications/hashes/counts belong in tracked evidence.
- Standing authorization covers bounded self-owned TCP/UDP Session/benchmark/capture/cleanup work within its limits; no per-run approval is required.

## Rolling Work Queue

This is a multi-hour capacity queue, not a one-hour ticket. **Every slice below is pre-authorized to continue into the next dependency-satisfied slice without waiting for another reviewer interval.** Pause only on a real stop condition.

### A — Add the minimum periodic remote-launch diagnostic discrimination

**Status:** `READY_LOCAL`.

**Goal:** change the hypothesis behind the exact-`60cd40d` SSH exit-255/start-timeout negative without reopening orchestration architecture.

**Likely files:** `scripts/bench/run-periodic-command.py`, its deterministic tests, and existing small evidence schema/validator only if necessary.

**Required behavior:**

1. Preserve bounded private server stdout/stderr capture and raw privacy boundaries.
2. On pre-application remote failure, emit only a small tracked diagnostic object such as:
   - fixed `classification` from a closed enum;
   - transport/server exit code when available;
   - bounded private diagnostic `sha256`, byte count and truncation flag;
   - readiness state / whether remote executor protocol was entered if mechanically observable.
3. Distinguish at least the evidence classes the current wrapper can prove without parsing secrets: e.g. `ssh_transport_exit`, `remote_exec_protocol_reject`, `remote_binary_identity_reject`, `server_runtime_exit_before_ready`, `start_timeout_no_terminal_evidence`, `log_overflow`.
4. Do not regex raw host/user/path/key text into tracked output. Classification must be based on bounded structural/exit evidence; private raw text remains untracked.
5. Do not change Nekomusume Session/wire/Noise semantics and do not add a generic remote orchestration framework.
6. Preserve all `34e587d` workload/success/percentile/privacy invariants.

**Required tests:** one deterministic case per supported classification, unknown/private text remains private, truncated logs fail closed, early-exit/start-timeout paths remain bounded, and all prior periodic regressions stay green.

**Validation/commit:** targeted tests -> full local gate -> `git diff --check` -> commit/push.

**Continue immediately to B:** yes.

---

### B — Exact-head CI gate plus changed-hypothesis periodic preflight

**Dependency:** A pushed.

- Wait for exact A-head stable/fuzz workflow as required by current CI policy before live periodic evidence.
- While CI is pending, continue H/I read-only release/security preparation; do not idle.
- Once exact-head CI is green, use validate/dry-run plus the same structured remote executor to confirm the plan reaches the newly discriminating remote-launch path without application traffic.
- Dry-run/preflight is not live evidence and must remain labeled as such.

If deterministic/preflight evidence exposes a correctness bug, repair locally and repeat the gate. If it only shows environment-specific private values, keep them private.

**Continue immediately to C when exact-head CI is green:** yes.

---

### C — One final materially changed periodic VPS attempt

**Status after A/B:** `READY_LIVE_CHANGED_HYPOTHESIS`.

**Goal:** obtain one scientifically distinct longer periodic Session or close the current periodic instrumentation line with a discriminating pre-application negative.

**Profile:**

```text
self-owned client + owned VPS only
application phase: target ~480 s
interval: 5 s
count: target ~96
payload: 32 B/record
concurrency: 1
single outer invocation; no retry
total setup + application + cleanup < 10 min
```

Shorten only as needed to preserve the standing 10-minute bound; do not split one soak into multiple nominal runs.

**Success evidence:** exact commit + executable hash/size, normalized workload, attempted==declared count, confirmed==attempted, missing=0, reconnects=0, validated P50/P95 from interval rows, separate client duplicate-ACK/server duplicate-data counters, application bytes, exits, no signal interruption, local resource scope if truthfully sampled, remote resources `not_collected_remote` unless genuinely sampled, and verified local/remote cleanup.

**Failure evidence:** retain one typed result plus the new bounded diagnostic classification/hash/count. If the run fails pre-application in the same now-discriminated class, mark `BLOCKED_ORCHESTRATION_CURRENT_LINE_PERIODIC` and stop automatic periodic retries. If it exposes a real runtime correctness failure, preserve a deterministic reproducer and make correctness repair the next blocking slice.

One success remains one bounded sample; no reliability-rate or production-long-lived claim.

**Commit/push:** minimal sanitized evidence + validator checks + full gate.

**Continue immediately to D:** yes.

---

### D — Reconcile the release matrix after final A/C facts

**Status:** `READY_LOCAL` after C, regardless of success/typed negative.

Update from exact facts only:

- `docs/era4-e-resilience.md`;
- `docs/status.md`;
- `IMPLEMENTATION_PLAN.md`;
- `ROADMAP.md`;
- capability/evidence indexes already used by the repository.

Required reconciliation:

1. Record exact `4a2129e` as the final current-line repeated-failover orchestration negative and remove stale wording that still calls repeated failover merely `BLOCKED_DIAGNOSTICS` or suggests another automatic retry.
2. Record the final periodic outcome from C. Preserve exact `60cd40d` as historical pre-application negative; do not overwrite it.
3. Keep natural UDP degradation/PTO-blackhole unchecked unless new evidence actually proves it.
4. Keep bounded periodic success scoped to exact sample duration; no general long-lived conclusion.
5. Keep NAT/migration/key-update/PMTUD/IPv6/HY2 truthful to actual classifications.
6. Release-evidence item 3 remains unchecked while declared gaps remain.
7. No RC/global freeze/production/release flag changes.

**Validation:** status/plan sync checks + `scripts/check.sh` + `git diff --check`.

**Continue immediately to E:** yes.

---

### E — Add one minimal HY2 diagnostic discriminator; no live retry yet

**Status:** `READY_LOCAL` after higher-value periodic lane, or while C is blocked on CI.

**Goal:** turn retained HY2 `client_exit` into a changed hypothesis without broad harness redesign.

Use existing pinned HY2 v2.9.3 and fair-comparison contract. Add only bounded private diagnostics sufficient to classify the next failure stage, for example:

- HY2 client process/transport exit classification;
- whether client started, QUIC/UDP send was observed, server temporary listener was ready, and server response direction was observed when existing bounded capture/metadata can prove it;
- bounded private stderr/log hash + byte count + truncation flag;
- TLS/auth/config vs path/packet-direction classification only when evidence supports it.

Do not track raw addresses, credentials, certificate secrets or private logs. Do not change firewall/route/qdisc/DNS/provider policy. Do not weaken TLS/authentication.

**Tests:** deterministic stage-classification tests; failure rows retain valid prefix; cleanup remains fail-closed; malformed/private diagnostics never leak tracked strings.

**Validation/commit:** targeted tests + full gate + push.

**Continue immediately to F after exact-head CI green:** yes.

---

### F — One changed-hypothesis HY2/Nekomusume paired attempt

**Dependency:** E exact-head CI green.

Run exactly one bounded self-owned attempt using the existing comparison contract and new diagnostic seam:

- 5 paired samples only if the first required pair succeeds and the harness contract permits continuation;
- deterministic 1200-byte payload, concurrency 1;
- same client/VPS/route/time window/MTU/security/load class;
- pinned HY2 artifact already recorded;
- no production-service/network-policy modifications;
- bounded capture only around experiment ports/window when needed;
- complete cleanup verification.

If all required pairs succeed, retain raw rows and bounded median/P95/failure summaries only under the fair lifecycle/resource contract; no superiority claim.

If the first required HY2 pair fails, stop that live invocation, retain the typed stage diagnostic, and do **not** automatically retry again. Mark the current HY2 line blocked at the proven class.

**Continue immediately to G:** yes.

---

### G — Release-evidence closure ledger and blocker map

**Status:** `READY_LOCAL` after F or an honest F block.

Create/update one review-oriented release-evidence ledger derived from existing repository facts. For every declared item-3 row, record:

- required claim;
- best exact positive evidence;
- retained negatives;
- current classification (`ALREADY_SUFFICIENT_FOR_BOUNDED_QUESTION`, `BLOCKED_ORCHESTRATION_CURRENT_LINE`, `BLOCKED_DIAGNOSTICS`, `BLOCKED_IMPLEMENTATION_ARCHITECTURE`, `BLOCKED_ENVIRONMENT`, `OPEN_READY`);
- what evidence would be needed to change classification;
- whether doing so requires a new architecture/value decision.

The ledger must not silently narrow the release scope and must not treat inability to collect a row as PASS.

At this point, if no `OPEN_READY` VPS row remains, stop spending VPS time on generic repetition. Move to H/I and prepare the maintainer scope decision packet in J.

**Continue immediately to H:** yes.

---

### H — Independent-review preparation packet (not an independent review)

**Status:** `READY_LOCAL`, may run while CI blocks live work.

Prepare a bounded, machine-navigable packet for a future independent release/security reviewer. Reuse current evidence rather than re-running it.

Include links/identities for:

- canonical corpus/freeze identity and executable oracles;
- negotiation compatibility policy;
- Noise/trust/authz transcript binding;
- parser/fuzz evidence;
- pre-auth/resource limits;
- Session vs Carrier ACK boundaries;
- failover/resume evidence and exact limitations;
- package install/upgrade/rollback evidence;
- operator readiness/cleanup evidence;
- VPS positive/negative matrix;
- HY2 methodology/results boundary;
- current unresolved findings/blockers.

Do not label this packet a security audit and do not mark item 4 complete.

**Validation:** link/path consistency + existing doc/status gates.

**Continue immediately to I:** yes.

---

### I — Resource/abuse-limit evidence audit and targeted deterministic gaps

**Status:** `READY_LOCAL`.

Audit the current implementation/tests against the explicit `SECURITY.md` requirements: per-connection/global memory, CPU/rate bounds, UDP amplification, malformed lengths/offsets/unknown versions/old packet numbers, pre-auth admission and fail-closed resource behavior.

Rules:

- first map existing code/tests/evidence; do not duplicate already-covered checks;
- if a small deterministic missing negative test is found, add it and run the required gate;
- if a substantive design/resource-bound gap is found, record it as a release/security blocker and do not invent arbitrary production limits without a decision record;
- no public listener or stress/capacity test beyond standing authorization.

**Continue immediately to J:** yes if no BLOCKER/HIGH requires reviewer/maintainer decision.

---

### J — RC-scope decision dossier; terminal maintainer gate if matrix has no READY path

**Status:** `READY_LOCAL_PREPARATION`; any scope change itself is **not** pre-authorized.

If G shows item 3 cannot complete because the remaining declared requirements are only blocked by architecture/environment/current-line orchestration, prepare a concise decision dossier with exact options, without choosing for the maintainer:

1. keep the current release-evidence scope and remain pre-RC until IPv6/NAT/live migration/key-update/PMTUD/etc. become genuinely available where required;
2. define a narrower first-RC claim (for example x86_64 Linux / owned IPv4 / explicitly bounded evidence) and state exactly which claims are removed/deferred;
3. authorize new implementation/environment work needed to satisfy the existing matrix.

For each option give evidence consequences, compatibility/security risk, work required and what existing artifacts remain valid. Do not change `IMPLEMENTATION_PLAN.md` release scope, RC flags or production claims until the maintainer chooses.

**Stop condition:** this dossier is a real maintainer value-judgment gate. Once reached with no other independent READY work, notify the maintainer rather than silently selecting a scope.

## Completion gates for this rolling queue

- R-008 periodic semantic oracle remains closed.
- Repeated failover receives no further automatic retry under the closed current instrumentation line.
- Periodic gets at most one materially changed live retry after a real diagnostic change and green exact-head CI.
- HY2 gets at most one materially changed diagnostic retry after local discrimination and green exact-head CI.
- No tracked private endpoint/key/argv/log material leaks.
- Every live negative remains a negative; later cleanup does not rewrite artifact cleanup fields.
- `IMPLEMENTATION_PLAN.md`, `ROADMAP.md`, `docs/status.md` and evidence indexes agree on final exact facts.
- No experimental feature is implemented merely to fill a release checkbox.
- RC/production/global-freeze/release flags remain unchanged absent a later reviewed maintainer decision.

## Do not expand into

- another broad repeated-failover harness rewrite;
- unchanged WAN retries;
- production firewall/route/qdisc/DNS/proxy/tunnel changes;
- third-party targets/scanning;
- automatic NAT/migration/key-update/PMTUD implementation just to satisfy the matrix;
- 0-RTT, enabled FEC, striping/aggregation or exotic carriers without an observed-problem gate;
- weakening HY2/Nekomusume security for benchmark convenience;
- self-declaring an independent security review;
- changing RC/release scope without the maintainer decision in J.

## Questions requiring maintainer decision

none now. A maintainer decision becomes required only if/when slice J is reached and the remaining release-evidence matrix has no dependency-safe READY path under the current declared scope.
