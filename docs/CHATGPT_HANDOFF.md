# Nekomusume ChatGPT Handoff

Checked at: 2026-09-04 02:57 Asia/Shanghai
Repository HEAD reviewed: `60cd40d612b5582337c9fa04cc28b35aed98f322`
Previous reviewed coding/evidence HEAD: `340f74ec76b0e6312219dc96a35c783eda790570`
Previous reviewer handoff commit: `3b2cb65c25adefabd7370fbec3fb5db345705724`

## What changed

One coding-agent commit is newly reviewed after the previous reviewed coding/evidence HEAD. The intervening `3b2cb65` commit is reviewer-handoff-only and is not counted as coding progress.

- `60cd40d` — **periodic startup/capture/privacy/semantic-adapter repair; evidence tooling only, no wire/Session/Noise/failover runtime semantic change.** It adds bounded stdout/stderr capture, waits for the real `periodic_server_ready` marker before launching the client, fails closed on early exit/start timeout/malformed readiness/log overflow, parses client/server periodic summaries and interval rows, removes raw argv/binary paths from the public endpoint record, preserves remote resource scope as `not_collected_remote`, samples the local client direct child, and retains local/remote cleanup postchecks. These changes materially close the original R-007 startup-race, discarded-output and raw-endpoint-reporting problems.

Exact-head GitHub Actions run `33790516164` completed successfully on `60cd40d`:

- `stable checks` — `bash scripts/check.sh` succeeded;
- `nightly decode fuzz smoke` — pinned cargo-fuzz decode build and 30-second / 8,192-byte smoke succeeded.

The periodic lane is therefore much closer to live use. However, the new executable oracle still has one **HIGH evidence-integrity defect** that can turn an incomplete periodic Session into a tracked `passed` result. Do not spend the longer-periodic VPS window until this narrow success-predicate/oracle defect is closed.

### R-008 HIGH — periodic success predicate is still semantically incomplete

This is directly demonstrated by the committed deterministic test, not a hypothetical concern. `test_delayed_readiness_exact_accounting_and_private_logs` currently expects:

```text
status=passed
attempted=2
confirmed=1
missing=1
```

The wrapper therefore accepts an explicitly incomplete application exchange as `passed` when the child exits 0. A live evidence collector must not delegate the truth of its success claim to the child exit code when its own parsed semantics already prove missing delivery.

The same adapter has four adjacent semantic-oracle gaps:

1. `reconnects` is required syntactically in `periodic_summary` but is never parsed, asserted or emitted in the tracked result. The current runtime explicitly reports reconnect unsupported and emits `reconnects=0`; a nonzero value must not silently pass.
2. P50/P95 fields are optional in the parser even though the real periodic runtime always emits them and C1 requires them. When present they are only parsed as integers; they are not recomputed from the parsed per-interval confirmed latencies, so stale/wrong percentile values can pass.
3. The wrapper inserts `conflicts=0` into tracked evidence even though the periodic runtime emits no `conflicts` semantic. Do not synthesize a zero-valued evidence field merely because the expected happy path has no known conflict concept; remove it unless a real runtime/derivable semantic exists.
4. The wrapper only extracts `--bytes` from the client plan. It does not bind server/client `count`, `duration`, `interval-ms` and relevant bounded timing parameters into one normalized workload contract. A child can therefore attempt fewer records than declared, or server/client argv can disagree, while the wrapper can still label the run `passed`. `signal=true` is also accepted as a successful complete sample.

There is one more semantic distinction to preserve while repairing this: server `duplicates` counts duplicate received application records, whereas client `duplicates` counts duplicate authenticated DeliveryAck observations. They are not automatically the same evidence domain. Do not require equality merely because both fields are named `duplicates`; for the normal C1 success path they should independently remain zero or be reported truthfully.

This is a lane-specific evidence blocker for C1, not a production Session/runtime bug. Do not reopen the remote executor or create another orchestration framework.

## Review verdict

**CONTINUE_WITH_REQUIRED_FIXES — `60cd40d` closes the original R-007 startup/capture/privacy defects and exact-head CI is green. The final repeated-failover VPS attempt remains READY_LIVE_NOW and should run immediately. Periodic C1 remains blocked only on the narrow R-008 success/oracle repair.**

The project is not globally blocked. The rolling queue below remains pre-authorized: complete a coherent slice, validate, commit/push, and immediately continue to the next dependency-satisfied slice. Do not wait for another reviewer interval unless a listed stop condition is actually reached.

## Evidence boundaries

- `IMPLEMENTATION_COMPLETE=true` remains the bounded research baseline.
- `CANONICAL_CORPUS_V1_FROZEN=true` remains corpus-specific only.
- `RELEASE_CANDIDATE=false`, `PRODUCTION_READY=false`, `FREEZE=false`, and `RELEASED=false` remain correct.
- `60cd40d` is evidence/orchestration tooling. It does **not** add a new WAN success, periodic success, failover success, security approval or performance result.
- Exact `60cd40d` has independent green `stable checks` and 30-second decode fuzz-smoke CI. This is repository CI evidence only.
- Exact `25e0daa` remains the accepted single controlled application-level UDP reply-cessation warm-fallback row: 3/3 logical records, 48 application bytes, two uncertain/replayed ranges, duplicate/lost 0, about 434 ms failure-decision-to-first-resumed-data. It is not natural degradation/PTO-blackhole evidence.
- Exact `25e0daa` also remains one accepted approximately five-minute direct periodic Session: 60 x 32 B, 60/60 confirmed, no missing/duplicate/conflict. It is one bounded sample only.
- Exact `9fd2411`, `a117086`, `c6ab8fd` and other archived repeated-failover attempts remain orchestration/evidence negatives unless a retained row proves otherwise. Do not rewrite them as runtime failures or successes.
- NAT/source-endpoint change, live migration-back, live key update and live PMTUD remain implementation-blocked unless executable reality changes. IPv6 remains environment-blocked.
- Historical HY2 negatives remain valid; there is still no complete fair paired comparison.
- Endpoint/user/address/private topology, private plans, identity paths, keys and raw diagnostics remain local/untracked. Tracked artifacts may retain bounded hashes/classifications/ownership classes but not raw private strings.
- Standing authorization covers the bounded self-owned TCP/UDP work below within <=10 minutes / <=256 MiB / <=32 sessions. No per-run WAN approval is required.

## Rolling Work Queue

This is a multi-hour capacity queue, not a one-hour ticket. **Finish -> validate -> commit/push -> continue immediately** when the next slice is dependency-satisfied. A lane-specific blocker does not idle independent lanes.

### A4 — Execute the one final materially changed repeated warm-failover VPS batch

**Status:** `READY_LIVE_NOW`.

**Dependency:** satisfied. A3 startup readiness is closed and exact current HEAD `60cd40d` has green CI. `60cd40d` changed only periodic evidence tooling, so it does not invalidate the failover runtime/harness contract.

**Goal**

Use the paid VPS window now for the capped final repeated warm-failover attempt with observed server-start synchronization and role-specific startup diagnostics. Do not insert another generic local review or runner rewrite first.

**Profile**

- self-owned client + self-owned VPS only;
- exact executed commit + staged executable SHA-256/size;
- one outer aggregator invocation;
- exactly 6 sequential fresh cycles, concurrency 1;
- controlled application-level UDP reply cessation only;
- preferred 3 logical records x 16 B when the current runtime contract remains unchanged;
- fresh unprivileged ports in the existing bounded range;
- no retry of a failed cycle inside the batch;
- total setup + cycles + cleanup <10 minutes, prefer <=540 s;
- no production firewall/route/qdisc/DNS/proxy/tunnel/service changes;
- private plans/logs/endpoints remain untracked.

**Required evidence**

For every valid row, machine-check canonical negotiation, Noise authentication, at least one UDP delivery confirmation before controlled failure, warm TCP negotiation/authentication/resume, exactly three authenticated readiness proofs before promotion, no observed TCP application data before promotion, UNCERTAIN -> replay -> DeliveryAck accounting, timing/accounting/exits, exact provenance and cleanup. Remote resources remain `not_collected_remote` unless genuinely sampled remotely.

**Failure boundary**

- retain valid prefix + first typed failure + bounded non-sensitive diagnostic metadata/hash;
- if this still yields a 0-row orchestration/evidence failure after the startup-readiness repair, mark the lane `BLOCKED_ORCHESTRATION_CURRENT_LINE` and stop automatic repeated-failover retries;
- do not start another broad harness rewrite;
- a true runtime correctness failure gets a deterministic reproducer and correctness repair before any future live retry.

**Validation/commit:** minimal non-sensitive evidence only; targeted schema/validator checks; `scripts/check.sh`; `git diff --check`; push.

**Continue immediately to B:** yes.

---

### B — Classify and close the final repeated-failover attempt

**Dependency:** A4 complete or typed negative.

- `6/6`: validate all rows/provenance/cleanup and compute only bounded descriptive summaries; no reliability-rate or natural-WAN claim.
- partial/runtime failure: classify from actual evidence; preserve a deterministic reproducer only if correctness-related.
- 0-row orchestration failure: mark the current repeated-failover instrumentation line blocked and stop automatic retries.
- preserve every historical negative and exact artifact hash.

**Validation:** evidence/schema checks + `scripts/check.sh` + `git diff --check`.

**Continue immediately to C0.6:** yes unless A4 exposes a cross-cutting runtime correctness blocker.

---

### C0.6 — Close R-008: bind periodic `passed` to the complete declared workload and real summary semantics

**Status:** `READY_LOCAL`.

**Goal**

Keep the useful `60cd40d` readiness/capture/privacy work, but make `status=passed` mean the declared periodic experiment actually completed and every tracked semantic is machine-enforced.

**Likely files**

- `scripts/bench/run-periodic-command.py`;
- `scripts/bench/run-periodic-command-test.py`;
- schema/validator only if a small machine-readable periodic result contract already exists or is necessary; do not introduce a new framework.

**Required behavior**

1. Normalize the bounded periodic workload from the private plan before execution. At minimum bind the server/client `transport/port/bytes/count/duration/interval-ms` values that define the application experiment; include setup/ACK timeout values in provenance if they materially affect the run. Server and client parameters that are required to agree must be checked before spawn.
2. Keep raw argv private. Emit only the normalized non-sensitive numeric/workload values needed to interpret the result.
3. A complete successful C1-style result must fail closed unless:
   - `attempted == declared_count`;
   - `confirmed == attempted`;
   - `missing == 0`;
   - `reconnects == 0` under the current reconnect-unsupported runtime;
   - client and server complete without signal interruption for the normal sample;
   - application bytes equal `attempted * declared_bytes`;
   - exits and cleanup satisfy the existing success boundary.
   Do not rely on child exit 0 to override contradictory parsed evidence.
4. Parse and emit `reconnects`. A nonzero reconnect count cannot pass under the current runtime contract.
5. Require the real runtime's P50 and P95 fields. Recompute the same nearest-rank P50/P95 from the confirmed non-null `periodic_interval` latencies and require exact equality to the emitted summary. A stale or omitted percentile must fail closed.
6. Remove the synthetic tracked `conflicts=0` field unless a real runtime or mechanically derived conflict semantic is added for an independent reason. Do not add a production API merely to keep that field.
7. Preserve semantic domains: client duplicate-ACK observations and server duplicate-data observations are separately truthful counters; do not impose fake equality solely on the shared word `duplicates`. For the ordinary complete C1 sample, record both and require the intended no-duplicate boundary only if the experiment contract actually selects that as a success condition.
8. Preserve the `60cd40d` improvements: observed `periodic_server_ready`, bounded private capture, private endpoint/argv/path handling, exact endpoint provenance, remote resources `not_collected_remote`, local client process sampling, and local/remote cleanup checks.

**Required deterministic regressions**

- the currently accepted fake `attempted=2, confirmed=1, missing=1, exit=0` case must become a failure;
- attempted fewer than declared count with exit 0 fails;
- server/client count/bytes/duration/interval disagreement fails before live spawn;
- `reconnects=1` fails;
- missing percentile fields fail;
- wrong P50 or P95 relative to interval latencies fails;
- signal-interrupted incomplete sample cannot become `passed`;
- no tracked `conflicts` field is synthesized;
- private argv/address/identity/key strings remain absent from tracked output;
- delayed readiness, remote early exit, log cap, malformed summary, cleanup failure and local/SSH happy paths remain covered.

**Validation/commit:** targeted periodic tests + full local gate + `git diff --check`; push normally. Require exact-head CI green before C1.

**While CI is pending:** E read-only classification or H evidence-index work may proceed; do not idle and do not run C1 before the exact repair HEAD is green.

**Continue immediately to C1 once exact-head CI is green:** yes.

---

### C1 — Run one scientifically distinct longer periodic Session

**Dependency:** C0.6 pushed and exact-head CI green.

**Recommended bounded profile**

```text
application phase: about 480 s
interval: 5 s
count: about 96
payload: 32 B/record
concurrency: 1
```

Shorten the application phase only as needed so setup + application + cleanup remain <10 minutes. Do not split one intended soak into nominally separate runs to evade the bound.

**Required tracked evidence**

- exact commit + executable SHA-256/size;
- normalized actual duration/count/interval/payload and application bytes;
- attempted/confirmed/missing, client duplicate-ACK observations, server duplicate-data observations, reconnects;
- P50/P95 confirmation latency validated against interval rows;
- server authenticated/received/confirmed consistency appropriate to the actual semantics;
- local client CPU/RSS/FD/process scope if truthfully collected; remote server resources remain `not_collected_remote` unless actually sampled;
- exits, no normal-run signal interruption, and verified local/remote cleanup;
- no raw endpoint/private plan/log material.

One success is one bounded sample, not a reliability rate or production-long-lived claim. One failure is retained; no unchanged retry.

**Commit/push:** minimal evidence artifact/summary + hashes; validate; push.

**Continue immediately to D:** yes.

---

### D — Reconcile release-matrix/resilience status from A4/C1 facts

Update only from real evidence:

- `docs/era4-e-resilience.md`;
- `docs/status.md`;
- `IMPLEMENTATION_PLAN.md`;
- `ROADMAP.md`.

Rules:

- repeated controlled failover becomes positive only if A4 produced the required valid rows;
- natural UDP degradation/PTO blackhole remains unchecked;
- longer periodic evidence remains bounded to its exact sample/duration;
- all historical negatives remain visible;
- release-evidence item 3 stays unchecked while declared matrix gaps remain;
- no RC/global freeze/production/release flag changes.

**Validation:** status/plan sync checks + `scripts/check.sh` + `git diff --check`.

**Continue immediately to E:** yes.

---

### E — Re-audit blocked VPS rows; execute only a genuinely ready one

Classify current executable reality for genuine NAT/source-endpoint change, migration-back, live key update, live PMTUD and IPv6.

Use one of:

- `READY_LIVE` — real current CLI/runtime seam already proves the property; run once under standing authorization.
- `SMALL_LOCAL_UNLOCK` — only a small non-architectural observation/adapter seam is missing; implement/test/push/CI, then run once.
- `BLOCKED_IMPLEMENTATION_ARCHITECTURE` — requires new rebinding/migration/crypto/PMTUD runtime architecture; record blocked and do not invent it to fill the matrix.
- `BLOCKED_ENVIRONMENT` — environment lacks the path, e.g. owned end-to-end IPv6.
- `ALREADY_SUFFICIENT_FOR_CURRENT_BOUNDARY` — existing evidence answers the bounded question; do not repeat.

No production route/firewall/qdisc changes and no fake NAT via relabeling a same-path run.

**Continue immediately to F:** yes.

---

### F — Upgrade HY2 failure diagnostics locally; no unchanged retry

**Status:** `READY_LOCAL` after higher-value A/C VPS lanes, or while a live lane is genuinely blocked.

Use the retained HY2 `client_exit` negatives to add one bounded discriminating private diagnostic contract:

- real HY2 stderr/log path feeds bounded private diagnostics;
- tracked artifact keeps only fixed lifecycle stage/classification plus non-sensitive hash/length/timestamps;
- preserve repaired bind/connect separation, pinned disposable TLS certificate, fresh-client lifecycle and fair measurement contract;
- no broad harness rewrite, no security weakening, no production service/config changes.

**Validation:** deterministic failure-path tests + full local gate + `git diff --check`; push + exact-head green CI before G.

**Continue immediately to G:** yes.

---

### G — One materially changed fair HY2/Nekomusume paired attempt

**Dependency:** F exact-head CI green and the diagnostic change materially increases what a failure can teach.

- same self-owned client/VPS and pinned HY2 v2.9.3;
- same deterministic payload/security/load/fresh-client lifecycle contract;
- up to 5 pairs only under the already reviewed fair contract;
- complete successful pair set -> raw rows + bounded median/P95/failures, no superiority claim;
- any required pair failure -> typed diagnostic artifact and no comparative summary;
- bounded capture metadata only when necessary and privacy-safe;
- cleanup experiment-owned processes/listeners/temp paths.

No unchanged HY2 retry after failure.

**Continue immediately to H:** yes.

---

### H — Prepare the independent release/security review evidence packet without claiming review

**Status:** `READY_LOCAL`; do after the time-sensitive VPS work or during a real CI/environment wait.

Create/update one reviewer-oriented evidence index pointing to existing facts for:

- canonical corpus/freeze scope;
- version/negotiation compatibility policy;
- parser/resource/pre-auth/abuse bounds;
- package install/upgrade/rollback;
- lifecycle/readiness/cleanup;
- current release-evidence matrix with positive and negative rows;
- HY2 methodology/status;
- unresolved security/reachability/environment/implementation blockers.

This is an evidence packet/index only, **not** an independent security review, approval or RC authorization. Do not duplicate normative specs or erase negative evidence.

**Validation:** link/commit/evidence-identity checks + `scripts/check.sh` + `git diff --check`.

**Continue immediately:** only to other already-listed READY work. Do not self-complete release-plan item 4 or set RC flags.

## Completion gates for this queue refresh

- A4 is executed once now or honestly classified; no repeated-failover retry loop continues after another capped 0-row orchestration failure.
- R-008 is closed before C1; an incomplete/missing/reconnected/interrupted periodic sample cannot be labeled `passed`.
- C1 uses a normalized declared workload contract, real runtime summary semantics and recomputed percentile checks.
- Periodic tracked evidence contains no synthetic `conflicts=0` or raw private argv/address/path/key material.
- VPS evidence remains exact-identity, bounded, cleanup-verified and scoped to self-owned endpoints.
- Historical negatives remain immutable evidence rather than failures to hide.
- `RELEASE_CANDIDATE=false`, `PRODUCTION_READY=false`, `FREEZE=false`, `RELEASED=false` remain unchanged until their separate gates.

## Do not expand into

- another generic failover/periodic orchestration framework;
- automatic retries after the capped A4/C1/G live attempts;
- natural-WAN/PTO claims from controlled application-level UDP reply cessation;
- production route/firewall/qdisc/DNS/proxy/tunnel changes;
- third-party targets or scanning;
- new NAT/rebinding/migration/key-update/PMTUD architecture merely to fill release rows;
- 0-RTT, enabled FEC, striping/aggregation or exotic carriers without an observed-problem gate;
- RC/security/production approval from bounded research evidence.

## Questions requiring maintainer decision

none.
