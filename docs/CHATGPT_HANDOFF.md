# Nekomusume ChatGPT Handoff

Checked at: 2026-09-04 02:02 Asia/Shanghai
Repository HEAD reviewed: `340f74ec76b0e6312219dc96a35c783eda790570`
Previous reviewed coding/evidence HEAD: `269c90ee80cb59be467e57bf334e7ad24b22f6da`
Previous reviewer handoff commit: `4f7e24e0061a88f02ba42ce0ad6358168c039677`

## What changed

Three coding-agent commits are newly reviewed after the previous reviewed coding/evidence HEAD. The `4f7e24e` commit between them is reviewer-handoff-only and is not counted as coding progress.

- `67453d5` — **initial shell-free periodic Session wrapper; local evidence/tooling only.** It introduced a JSON argv plan, validate/dry-run modes, bounded timeout/cleanup, and deterministic dispatch tests. The initial form was local-only and therefore did not yet satisfy the cross-host C0 contract.
- `177f373` — **A3 startup-observability repair; evidence/orchestration only, no wire/Session/Noise/failover semantic change.** It makes required `start` failures role-specific, removes the blind fixed startup sleep, and waits on one validated server `start` event before launching the client. It fails closed on early server exit, malformed/missing start, or bounded startup timeout and adds deterministic delayed-start/early-exit/missing-client-start/malformed-start regressions. The real `failover-server` emits its `server/start` diagnostic only after UDP and TCP listeners are successfully bound/configured and the Session runtime stream has been opened, so this observed readiness is materially stronger than “SSH process exists”. R-006 is closed.
- `340f74e` — **periodic cross-host endpoint/provenance upgrade; evidence/tooling only.** It replaces the local-only periodic plan with structured local/SSH endpoint semantics, uses `remote-endpoint-exec.py` for remote binary hash/size/commit verification immediately before direct spawn, keeps the local client process distinct from SSH transport, labels remote server resources `not_collected_remote`, requires separate remote/local cleanup checks, and adds NAT/path-independent shell-free literal/cleanup/provenance regressions. Exact-head GitHub Actions run `33785234307` completed successfully: `stable checks` and `nightly decode fuzz smoke` both passed.

The coding agent therefore consumed two independent queued lanes without waiting: A3 is closed and the transport/provenance portion of C0 is substantially closed. The final repeated-failover retry is now dependency-ready on exact current green HEAD and should use the rented VPS immediately.

### R-007 HIGH — periodic wrapper is transport-truthful but not yet application-evidence-truthful

Do **not** spend the longer-periodic VPS window yet. Current `scripts/bench/run-periodic-command.py` still cannot support the C1 evidence claim for three concrete reasons:

1. It launches the periodic client immediately after spawning the server transport and does not wait for the real `periodic_server_ready` line. On an SSH server path this can turn remote startup latency into a client `connect failed` negative.
2. It redirects both server and client stdout/stderr to `DEVNULL`. The real periodic runtime already emits `periodic_server_ready`, authentication lines, per-record interval rows, `periodic_server_summary`, and the client `periodic_summary` containing attempted/confirmed/missing/duplicates, P50/P95 confirmation latency, elapsed time and application bytes. The wrapper currently discards exactly the semantics C1 is supposed to preserve.
3. Its public report copies raw endpoint `binary.path` and complete argv vectors. A real plan may contain addresses, identity paths and key arguments. The private plan may retain them, but a tracked evidence artifact must not copy private endpoint/topology/path/key material into Git merely because the wrapper knows it.

This is a **lane-specific evidence blocker for C1**, not a production runtime defect and not a blocker for A4. Repair only the minimum periodic observation/sanitization seam after the final repeated-failover attempt. Do not create another generic orchestration framework.

## Review verdict

**CONTINUE_WITH_REQUIRED_FIXES — A3 accepted and exact current CI green; run the one final materially changed repeated-failover VPS batch now. Periodic C0 transport/provenance work is accepted, but close R-007 before the longer periodic VPS sample.**

The project is not globally blocked. Do not wait for another reviewer interval after each slice. Consume the rolling queue continuously until a listed stop condition is reached.

## Evidence boundaries

- `IMPLEMENTATION_COMPLETE=true` remains the bounded research baseline.
- `CANONICAL_CORPUS_V1_FROZEN=true` remains corpus-specific only.
- `RELEASE_CANDIDATE=false`, `PRODUCTION_READY=false`, `FREEZE=false`, and `RELEASED=false` remain correct.
- `177f373` and `340f74e` are harness/evidence infrastructure. They do not themselves add new WAN/runtime success evidence.
- Exact `340f74e` has independent green `stable checks` and 30-second decode fuzz-smoke CI. This is CI evidence, not WAN evidence, security audit or release approval.
- Exact `25e0daa` remains the accepted single controlled application-level UDP reply-cessation warm-fallback row: 3/3 logical records, 48 application bytes, two uncertain/replayed ranges, duplicate/lost 0, about 434 ms failure-decision-to-first-resumed-data. It is not natural degradation/PTO-blackhole evidence.
- Exact `25e0daa` also remains one accepted approximately five-minute direct periodic Session: 60 x 32 B, 60/60 confirmed, no missing/duplicate/conflict. It is one bounded sample only.
- Exact `9fd2411`, `a117086`, `c6ab8fd`, and the later archived A2 negative remain historical orchestration/evidence negatives. Do not rewrite them into runtime failures or successes.
- NAT/source-endpoint change, live migration-back, live key update and live PMTUD remain implementation-blocked unless executable reality changes. IPv6 remains environment-blocked.
- Historical HY2 negatives remain valid; there is still no complete fair paired comparison.
- Endpoint/user/address/private topology, private plans, identity paths, keys and raw diagnostics remain local/untracked. Tracked artifacts may retain bounded hashes/classifications/ownership classes but not raw secrets/topology/path strings.
- Standing authorization covers the bounded self-owned TCP/UDP work below within <=10 minutes / <=256 MiB / <=32 sessions. No per-run WAN approval is required.

## Rolling Work Queue

This is a multi-hour capacity queue. **Complete a coherent slice -> validate -> commit/push -> immediately continue to the next dependency-satisfied pre-authorized slice.** Do not stop after one commit, one hour, CI submission or reviewer interval. A lane-specific blocker does not idle independent lanes.

### A4 — One final materially changed repeated warm-failover VPS attempt

**Status:** `READY_LIVE_NOW`.

**Dependency:** satisfied. A3 is closed by `177f373`; current exact HEAD `340f74e` has green CI.

**Goal**

Spend exactly one final paid-window retry on the repeated warm-failover lane using observed server-start synchronization and role-specific startup diagnostics. This is the last automatic retry for the current instrumentation line.

**Profile**

- self-owned client + self-owned VPS only;
- exact executed current commit + staged executable SHA-256/size;
- one outer aggregator invocation;
- exactly 6 sequential fresh cycles, concurrency 1;
- controlled application-level UDP reply cessation only;
- preferred 3 logical records x 16 B if runtime contract remains unchanged;
- fresh unprivileged ports inside the existing bounded range;
- no cycle retry inside the batch;
- total setup + cycles + cleanup comfortably <10 minutes, prefer <=540 s;
- no production firewall/route/qdisc/DNS/proxy/tunnel/service changes;
- private plans/logs/endpoint details remain untracked.

**Required evidence**

For every valid row, machine-check the already accepted contract: canonical negotiation, Noise authentication, one UDP delivery confirmation before controlled failure, warm TCP negotiation/authentication/resume, three authenticated readiness proofs before promotion, no observed TCP application data before promotion, UNCERTAIN -> replay -> DeliveryAck accounting, timing/accounting/exits, exact provenance and cleanup. Remote resources remain `not_collected_remote` unless genuinely sampled on the remote process.

**Failure boundary**

- retain valid prefix + first typed failure + bounded non-sensitive diagnostic metadata/hash;
- if this still yields a 0-row orchestration/evidence failure after A3, mark the lane `BLOCKED_ORCHESTRATION_CURRENT_LINE` and stop spending automatic VPS retries on repeated failover;
- do not add another broad runner rewrite;
- a true runtime correctness failure gets a deterministic reproducer and correctness repair before any future live attempt.

**Validation/commit:** archive only minimal non-sensitive evidence; targeted validator/schema checks; `scripts/check.sh`; `git diff --check`; push.

**Continue immediately to B:** yes.

---

### B — Classify and close the final repeated-failover attempt

**Dependency:** A4 complete or typed negative.

- `6/6`: validate all rows/provenance/cleanup and compute only bounded descriptive summaries; no reliability-rate or natural-WAN claim.
- partial/runtime failure: classify strictly from evidence and preserve a deterministic reproducer if correctness-related.
- 0-row orchestration failure: mark this repeated-failover instrumentation line blocked and stop automatic retries.
- preserve all historical negatives and exact hashes.

**Validation:** evidence/schema checks + `scripts/check.sh` + `git diff --check`.

**Continue immediately to C0.5:** yes unless A4 reveals a cross-cutting runtime correctness blocker.

---

### C0.5 — Close periodic startup, semantic-output and privacy evidence gaps

**Status:** `READY_LOCAL`; R-007 repair. `340f74e` already closes cross-host execution/provenance/cleanup structure, so do not replace it with another framework.

**Goal**

Make one real cross-host periodic run able to prove its Session semantics without racing remote startup or leaking private plan material.

**Likely files**

- `scripts/bench/run-periodic-command.py` and deterministic tests;
- a small periodic-result validator/schema only if needed to make tracked evidence machine-checkable;
- reuse `remote-endpoint-exec.py`; do not fork another remote executor.

**Required behavior**

1. Capture server/client output into bounded private files/pipes rather than `DEVNULL`; cap retained bytes and fail closed on overflow/malformed required summary evidence.
2. Before launching the client, observe the real remote/local `periodic_server_ready transport=tcp port=... reconnect=unsupported` marker with bounded timeout and process-exit detection. This marker is emitted only after the TCP listener is bound/nonblocking in the real runtime.
3. Parse and validate the final client `periodic_summary`: attempted, confirmed, missing, duplicates, P50/P95 confirmation latency, reconnects, elapsed_ms, application_bytes, cleanup, signal. Cross-check them against the private plan's bounded count/bytes/duration/interval contract.
4. Parse/validate server authentication + `periodic_server_summary` enough to prove authenticated=true and received/confirmed/duplicates are consistent with the client result. Do not require fake equality for fields whose semantics genuinely differ; document exact cross-check rules.
5. A successful tracked result must contain only bounded non-sensitive semantics/provenance: commit, executable SHA-256/size, endpoint execution class/ownership class, actual bounded workload parameters, parsed Session summary, resource scope and cleanup. Do **not** copy raw argv, endpoint address, SSH user/host, identity path, key arguments or private log text into tracked evidence.
6. Private logs/plans may contain endpoint material but remain restrictive/untracked. Tracked failure evidence contains only fixed classification + bounded hash/length/stage metadata.
7. Preserve exact remote binary verification, shell-free dispatch, `not_collected_remote` resource truthfulness and local/remote cleanup checks from `340f74e`.
8. Add deterministic tests for delayed server-ready, server exit before ready, ready timeout, malformed/missing summary, inconsistent attempted/confirmed/application bytes, duplicate summary, private argv/address/key non-leakage, remote cleanup failure, and unchanged local/SSH happy paths.

**Validation/commit:** targeted tests + full local gate + `git diff --check`; push; require exact-head CI green before C1.

**Continue immediately to C1 when exact-head CI green:** yes.

---

### C1 — Run one scientifically distinct longer periodic Session

**Dependency:** C0.5 pushed + exact-head CI green.

**Recommended bounded profile**

```text
application phase: about 480 s
interval: 5 s
count: about 96
payload: 32 B/record
concurrency: 1
```

Shorten the application phase if necessary so setup + run + cleanup remain <10 minutes. Do not exceed the standing bound by splitting a single intended soak into nominally separate runs.

**Required tracked evidence**

- exact commit + executable SHA-256/size;
- actual bounded duration/count/interval/payload and application bytes;
- attempted/confirmed/missing/duplicates/reconnects;
- truthful P50/P95 confirmation latency from the real periodic client summary;
- server authenticated/received/confirmed/duplicate consistency checks;
- local client CPU/RSS/FD/process scope if the existing sampler can truthfully provide it; remote server resource scope is `not_collected_remote` unless genuinely collected remotely;
- exits and verified cleanup;
- no raw endpoint/private plan/log material.

One success is one bounded sample, not a reliability rate or production-long-lived proof. One failure is retained; no unchanged retry.

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

- repeated controlled failover becomes positive only if A4 produces the required valid rows;
- natural UDP degradation/PTO blackhole remains unchecked;
- longer periodic evidence remains bounded to exact duration/sample;
- all historical negatives remain visible;
- release-evidence item 3 stays unchecked while declared matrix gaps remain;
- no RC/global freeze/production/release flag changes.

**Validation:** status/plan sync checks + `scripts/check.sh` + `git diff --check`.

**Continue immediately to E:** yes.

---

### E — Re-audit implementation-blocked VPS rows and execute only an actually ready one

Classify current facts for: genuine NAT/source-endpoint change, migration-back, live key update, live PMTUD and IPv6.

Use one of:

- `READY_LIVE` — existing real CLI/runtime seam already proves the intended property; execute one bounded owned-endpoint run.
- `SMALL_LOCAL_UNLOCK` — only a small non-architectural observation/adapter seam is missing; implement/test/push/CI, then run once.
- `BLOCKED_IMPLEMENTATION_ARCHITECTURE` — would require new rebinding/migration/crypto/PMTUD runtime architecture; record blocked and do not invent it for the matrix.
- `BLOCKED_ENVIRONMENT` — environment lacks the path, e.g. owned end-to-end IPv6.
- `ALREADY_SUFFICIENT_FOR_CURRENT_BOUNDARY` — existing evidence answers the bounded question; do not repeat.

Do not modify production routing/firewall or invent NAT by relabeling a same-path run.

**Continue immediately to F:** yes.

---

### F — Upgrade HY2 failure diagnostics locally; no unchanged retry

**Status:** `READY_LOCAL` after higher-value A/C lanes, or usable if E has no READY VPS row.

Use retained HY2 `client_exit` negatives to add one bounded discriminating private diagnostic contract:

- real HY2 stderr/log path feeds a bounded private/sanitized diagnostic bundle;
- tracked artifact keeps only fixed lifecycle stage/classification + non-sensitive hash/length/timestamps;
- preserve repaired bind/connect separation, pinned disposable TLS certificate, fresh-client lifecycle and fair measurement contract;
- no broad harness rewrite; no security weakening; no production service/config changes.

**Validation:** deterministic failure-path tests + full local gate + `git diff --check`; push + exact-head green CI before G.

**Continue immediately to G:** yes.

---

### G — One materially changed fair HY2/Nekomusume paired attempt

**Dependency:** F exact-head CI green and the new diagnostic contract materially changes what a failure can teach us.

- same self-owned client/VPS, pinned HY2 v2.9.3;
- same exact deterministic payload/security/load/lifecycle contract;
- up to 5 pairs only if both implementations satisfy the fair fresh-client/session measurement contract;
- complete successful pair set -> raw rows + bounded median/P95/failures, no superiority claim;
- any required pair failure -> typed diagnostic artifact, no comparative summary;
- retain bounded capture metadata only if needed and privacy-safe;
- cleanup experiment-owned processes/listeners/temp paths.

No unchanged HY2 retry after failure.

**Continue immediately to H:** yes.

---

### H — Prepare the independent release/security review evidence packet without claiming review

**Status:** `READY_LOCAL`; do after time-sensitive VPS work or during a true environment block.

Create/update one reviewer-oriented evidence index that points to existing repository facts for:

- canonical corpus/freeze scope;
- version/negotiation compatibility policy;
- parser/resource/pre-auth/abuse bounds;
- package install/upgrade/rollback;
- lifecycle/readiness/cleanup;
- current release-evidence matrix with positive and negative rows;
- HY2 comparison methodology/status;
- unresolved security/reachability/environment/implementation blockers.

The artifact is an **evidence packet/index only**, not an independent security review, not approval, and not RC authorization. Do not duplicate normative specs or erase negative evidence.

**Validation:** links/commit/evidence identity checks + `scripts/check.sh` + `git diff --check`.

**Continue immediately:** only to other already-listed READY work; do not self-complete item 4 or set RC flags.

## Completion gates for this queue refresh

- A3 remains closed; no fixed-sleep startup contract returns.
- A4 is executed once now or honestly classified; no repeated-failover retry loop continues automatically after another 0-row orchestration failure.
- Periodic C1 is not run until R-007 is closed and exact-head CI is green.
- Any periodic tracked artifact proves real periodic Session summary semantics and does not copy raw private argv/address/path/key material.
- VPS evidence remains exact-identity, bounded, cleanup-verified and scoped to self-owned endpoints.
- Historical negatives remain immutable evidence, not failures to hide.
- `RELEASE_CANDIDATE=false`, `PRODUCTION_READY=false`, `FREEZE=false`, `RELEASED=false` remain unchanged until their separate gates.

## Do not expand into

- another generic failover/periodic orchestration framework;
- automatic retries after the capped A4/C1/G live attempts;
- natural-WAN/PTO claims from the controlled application-level UDP reply-cessation seam;
- production route/firewall/qdisc/DNS/proxy/tunnel changes;
- third-party targets or scanning;
- new NAT/rebinding/migration/key-update/PMTUD architecture merely to fill release rows;
- 0-RTT, enabled FEC, striping/aggregation or exotic carriers without an observed-problem gate;
- RC/security/production approval from bounded research evidence.

## Questions requiring maintainer decision

none.
