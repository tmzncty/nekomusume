# Nekomusume ChatGPT Handoff

Checked at: 2026-09-04 01:00 Asia/Shanghai
Repository HEAD reviewed: `269c90ee80cb59be467e57bf334e7ad24b22f6da`
Previous reviewed coding/evidence HEAD: `a117086fa69553a36021137900b6052050624a8b`
Previous reviewer handoff commit: `6eea387f79de9baf3419a2fb836bdf436e921d6d`

## What changed

Two coding-agent commits landed after the previous reviewer handoff.

- `c6ab8fd` — **privacy-preserving collector-diagnostic repair; no protocol/runtime semantic change.** It closes the previous R-005 evidence-loss defect by retaining bounded private diagnostic sidecars with restrictive permissions, while the tracked batch receives only a bounded hash/length/truncation/classification object. It adds regression coverage that secret/endpoint/path material does not enter the tracked result and keeps success-row semantics unchanged. Exact-head GitHub Actions run `33778250170` completed successfully.
- `269c90e` — **real self-owned VPS typed negative archive/status update; no runtime semantic change.** It preserves the one-and-only exact-`c6ab8fd` live repeated-failover invocation: 0/6 completed cycles, cycle 1 `invalid_cycle_evidence`, collector exit 2, no valid stdout row, 3.597 s outer elapsed time, zero retries, exact staged binary SHA-256/size recorded, and separate external cleanup reporting zero experiment process/listener/temp-path residue. The bounded private diagnostic now discriminates the immediate collector failure as `missing JSON event: start`; its sanitized private diagnostic is 51 bytes with SHA-256 `6f1f1c44a571fb2f9638887e736cd70de6466fc918e54547dec9fd843ecbe686`. The committed evidence correctly does not guess whether the missing start event came from server-side event absence, client-side event absence, remote output framing, or early endpoint exit. Exact current HEAD GitHub Actions run `33781229303` completed successfully.

A1 is therefore closed and A2 produced a **useful discriminating typed negative**, not a protocol failure. The repeated-failover lane should not receive another broad harness rewrite. One final narrowly targeted startup-observability repair is justified by the new live evidence; after that, spend at most one materially changed retry on this lane before moving on to other VPS-only evidence.

### R-006 HIGH — live startup readiness is role-blind and still time-based on the remote path

The new private diagnostic proves only `missing JSON event: start`. Current `run-live-warm-failover-cycle.py` has two concrete properties that make another live retry insufficiently discriminating:

1. `one_event(..., "start", required=True)` raises the same message for client and server, so the retained collector diagnostic cannot identify which endpoint lacked the required start event.
2. The client is launched after a fixed `NEKO_FAILOVER_SERVER_STARTUP_SECONDS` sleep (default 0.2 s), not after observed server readiness. That may be adequate locally but is not a truthful synchronization contract for an SSH-launched remote server. A delayed or early-exiting remote server can therefore be conflated with the actual failover experiment.

This is an **orchestration/evidence blocker for one more paid repeated-failover attempt**, not evidence of a Nekomusume runtime correctness defect. Repair only this seam. Do not reopen endpoint abstraction, provenance, parser architecture, Session/Carrier semantics or the remote resource model.

## Review verdict

**CONTINUE_WITH_REQUIRED_FIXES — A1 accepted; A2 retained as a discriminating typed negative; close one bounded startup-observability seam, allow one final materially changed repeated-failover retry, and continuously consume the independent periodic/HY2/review-prep queue instead of waiting**

The project is not globally blocked. The time-limited VPS remains the scarce asset. If the final repeated-failover retry still produces a zero-row orchestration negative after R-006 is closed, mark this lane blocked and move on; do not keep repairing runners indefinitely.

## Evidence boundaries

- `IMPLEMENTATION_COMPLETE=true` remains the bounded research baseline.
- `CANONICAL_CORPUS_V1_FROZEN=true` remains corpus-specific only.
- `RELEASE_CANDIDATE=false`, `PRODUCTION_READY=false`, `FREEZE=false`, and `RELEASED=false` remain correct.
- `c6ab8fd` changes evidence diagnostics only. It does not add WAN/runtime success evidence.
- `269c90e` is a real VPS **negative at the orchestration/evidence-collection boundary**, not a failover runtime failure.
- Exact current HEAD `269c90e` has green independent repository CI/fuzz smoke. This is CI evidence, not WAN evidence, security audit or release approval.
- Exact `25e0daa` remains the accepted single controlled application-level UDP reply-cessation warm-fallback row: 3/3 logical records, 48 application bytes, two uncertain/replayed ranges, duplicate/lost 0, about 434 ms failure-decision-to-first-resumed-data. It is not natural degradation/PTO-blackhole evidence.
- Exact `25e0daa` also remains one accepted approximately five-minute direct periodic Session: 60 x 32 B, 60/60 confirmed, no missing/duplicate/conflict. It is one bounded sample, not a reliability rate or production long-lived conclusion.
- Exact `9fd2411`, `a117086`, and `c6ab8fd` repeated-failover attempts remain historical typed negatives at distinct orchestration/evidence stages. Do not rewrite any of them into runtime evidence.
- The `c6ab8fd` private diagnostic says only `missing JSON event: start`; deeper cause is currently unknown and must not be inferred retrospectively.
- Historical HY2 negatives remain valid; there is still no complete fair paired comparison.
- NAT/source-endpoint change, live migration-back, live key update and live PMTUD remain implementation-blocked until executable reality changes. IPv6 remains environment-blocked.
- Endpoint/user/address/private topology and raw/private diagnostics remain local/untracked. Repository artifacts retain only bounded non-sensitive metadata/hashes/classifications.
- Standing authorization covers bounded self-owned TCP/UDP failover, periodic Session, diagnostics/capture, resource observation, HY2 comparison and cleanup within <=10 minutes / <=256 MiB / <=32 sessions. No per-run WAN approval is required.

## Rolling Work Queue

This is a multi-hour capacity queue. **Complete a coherent slice -> validate -> commit/push -> immediately continue to the next dependency-satisfied pre-authorized slice.** Do not stop after one commit, one hour, CI submission or reviewer interval. CI pending blocks only the live slice that requires that exact green HEAD; use the wait for independent local slices below.

### A0 — Cross-host truthfulness/provenance integration

**Status: CLOSED** by `a117086`; exact-head CI was green.

Do not reopen unless a new concrete defect proves it necessary.

**Continue immediately:** yes.

---

### A1 — Preserve bounded private collector diagnostics

**Status: CLOSED** by `c6ab8fd`; exact-head CI green.

Accepted boundary: tracked evidence contains only bounded non-sensitive diagnostic metadata; private sidecar remains local/restrictive and bounded. `diagnostic.bytes`/SHA describe the retained private sanitized bytes, not a claim about full raw stderr.

**Continue immediately:** yes.

---

### A2 — First changed-hypothesis six-cycle VPS batch

**Status: COMPLETE AS TYPED NEGATIVE** and archived by `269c90e`.

Result: 0/6, cycle 1 `invalid_cycle_evidence`, collector exit 2, private diagnostic `missing JSON event: start`, no retry, external cleanup zero. This is a useful orchestration negative.

**Continue immediately to A3:** yes; do not rerun unchanged.

---

### A3 — Close only the role/startup-observability seam exposed by exact `c6ab8fd`

**Status:** `READY_LOCAL`; one bounded precursor to the final failover retry.

**Goal**

Make the next failure distinguish `server never reached start`, `client never reached start`, `endpoint exited before start`, and `start observed -> later collector failure`, while removing the fixed-sleep race from the remote-server startup boundary.

**Likely files**

- `scripts/bench/run-live-warm-failover-cycle.py` + deterministic tests;
- `scripts/bench/run-repeated-warm-failover.py` / schema only if one new non-sensitive fixed startup classification must be propagated.

**Required behavior**

1. Make required-event errors role/stage-specific (`missing server JSON event: start`, `missing client JSON event: start`) without embedding endpoint/user/path details.
2. Before launching the client, wait for **observed server start readiness** rather than a blind 0.2 s sleep. Use a small bounded polling window against the private server log/process state. The observed readiness condition should be the actual diagnostic `server/start` event with the expected experiment identity and bounded start fields, not merely “SSH process exists”.
3. If the remote server exits before start, fail closed immediately with a fixed non-sensitive classification and retain the bounded private server log diagnostic/hash locally. Do not launch the client in that case.
4. If the startup window expires while the remote process is still alive, classify `server_start_timeout`; preserve bounded private diagnostics and cleanup; do not proceed into application traffic.
5. Preserve shell-free argv, exact binary/commit provenance, endpoint provenance, remote resource truthfulness and remote cleanup postcheck from A0.
6. Do not redesign the general event parser. This slice is only startup synchronization + role-specific diagnostics.
7. Add deterministic tests for: delayed-but-valid remote start; remote early exit; missing server start; missing client start; malformed start event; and unchanged normal local success path.

**Validation:** targeted live-cycle/repeated-runner regressions, `bash scripts/check.sh`, `git diff --check`; protocol fuzz only if production input/parser semantics change (they should not).

**Commit/push condition:** one coherent harness/evidence commit, normal push; require exact-head CI green before A4.

**While CI waits:** immediately work C0, F or H below. Do not idle.

**Continue immediately to A4 when CI green:** yes.

---

### A4 — One final materially changed repeated warm-failover VPS attempt

**Dependency:** A3 pushed + exact-head CI green.

This is the **last automatically pre-authorized repeated-failover retry in the current instrumentation line**. It is materially changed by observed server-start synchronization and role-specific failure evidence.

**Profile**

- self-owned client + VPS only;
- exact executed commit + staged executable SHA-256/size;
- one outer aggregator invocation, 6 sequential fresh cycles, concurrency 1;
- controlled application-level UDP reply cessation only;
- preferred 3 logical records x 16 B if runtime contract unchanged;
- fresh unprivileged ports in existing range;
- no cycle retry inside the batch;
- total setup + cycles + cleanup <10 minutes (prefer <=540 s);
- no production network changes;
- private plan/log diagnostics remain untracked.

**Success evidence:** same accepted negotiation/authentication/readiness/UNCERTAIN->replay->DeliveryAck/accounting/timing/exit/provenance/cleanup contract as the previous queue.

**Failure boundary**

- retain valid prefix + first typed failure + non-sensitive diagnostic hash/classification;
- if the result is again 0-row orchestration/evidence failure after A3, **stop this lane** as `BLOCKED_ORCHESTRATION` and move to C/F/H; do not perform another generic runner repair automatically;
- only a newly discriminating concrete root cause plus a very small repair may justify future reconsideration by a later reviewer;
- runtime correctness failure gets a deterministic reproducer and correctness repair before any future live rerun.

**Commit/push condition:** archive minimal non-sensitive evidence + hashes; validate + `git diff --check`; push.

**Continue immediately to B:** yes.

---

### B — Classify/close the final repeated-failover attempt

**Dependency:** A4 complete or typed negative.

- 6/6: validate rows/provenance/cleanup and compute only bounded descriptive timing/failure summaries; no reliability-rate claim.
- partial/runtime failure: classify only from evidence and preserve a reproducer if correctness-related.
- 0-row orchestration failure: mark repeated-failover live matrix row blocked for this instrumentation line and stop spending VPS windows on it.
- preserve all historical negatives and their exact hashes.

**Validation:** targeted evidence/schema checks + `scripts/check.sh` + `git diff --check`.

**Continue immediately to C0:** yes unless a cross-cutting correctness blocker appears.

---

### C0 — Build the minimum shell-free periodic cross-host wrapper

**Status:** `READY_LOCAL`, independent during A3/A4 CI waits.

The accepted ~5-minute periodic direct row is real; later orchestration invoked zero periodic clients. Build only the minimum wrapper around existing real `periodic-server` / `periodic-client` behavior.

Required:

- shell-free argv;
- local/untracked endpoint plan;
- exact executable SHA-256/size + checkout commit attribution;
- deterministic dry-run proving the real periodic client dispatch path is entered, explicitly `live_evidence=false`;
- malformed plan/argv fail closed;
- structured local/SSH endpoint semantics reused from A0 where applicable, without another generic orchestration framework;
- remote server resource metrics either genuinely remote or explicit `not_collected_remote`; never measure SSH as Nekomusume;
- verified remote/local cleanup.

**Validation:** targeted tests + full local gate + `git diff --check`; push. Require exact-head green CI before C1.

**Continue immediately to C1 when CI green:** yes. F/H remain pre-authorized during CI.

---

### C1 — Run one scientifically distinct longer periodic Session

**Dependency:** C0 exact-head CI green.

Recommended bounded profile:

```text
application phase: ~480 s
interval: 5 s
~96 records
payload: 32 B/record
concurrency: 1
```

Shorten application phase if needed so setup + application + cleanup stay <10 minutes.

Record exact identity, actual duration, records/bytes, confirmed/missing/duplicate/conflict, truthful latency raw/median/P95 if available, local/remote resource scope, exits and cleanup. One success is one bounded sample only; one failure is retained with no unchanged retry.

**Commit/push condition:** minimal evidence artifact/summary + hashes.

**Continue immediately to D:** yes.

---

### D — Reconcile release-matrix/resilience status from A4/C1 facts

Update only from real evidence:

- `docs/era4-e-resilience.md`;
- `docs/status.md`;
- `IMPLEMENTATION_PLAN.md`;
- `ROADMAP.md`.

Rules:

- controlled repeated failover becomes positive only if A4 actually produces the required valid rows;
- natural UDP degradation/PTO blackhole remains unchecked;
- longer periodic evidence remains bounded to exact duration/sample;
- historical `9fd2411`, `a117086`, `c6ab8fd` negatives remain visible;
- release-evidence item 3 stays unchecked while declared matrix gaps remain;
- no RC/global freeze/production/release flag changes.

**Validation:** status/plan boundary checks + `scripts/check.sh` + `git diff --check`.

**Continue immediately to E:** yes.

---

### E — Audit implementation-blocked VPS rows and unlock only what is architecturally ready

Priority: genuine NAT/source-endpoint change -> migration-back -> live key update -> live PMTUD, unless repository facts show another row is closer.

Classify each as `READY_LIVE`, `SMALL_LOCAL_UNLOCK`, `BLOCKED_IMPLEMENTATION_ARCHITECTURE`, `BLOCKED_ENVIRONMENT`, or `ALREADY_SUFFICIENT_FOR_CURRENT_BOUNDARY`.

- `READY_LIVE`: execute one bounded owned-endpoint experiment immediately.
- `SMALL_LOCAL_UNLOCK`: implement only the smallest seam using accepted Session/Carrier identity/anti-replay semantics, test/gate/push/CI, then run once.
- new migration/rebinding/crypto architecture or production network manipulation: mark blocked and continue; do not invent architecture to fill a matrix.

**Continue immediately to F:** yes.

---

### F — Upgrade HY2 failure diagnostics locally; no unchanged retry

**Status:** `READY_LOCAL`, safe fallback during CI or blocked failover lane.

Use the retained HY2 `client_exit` negatives to build one bounded discriminating diagnostic contract:

- prove real HY2 stderr/log path feeds a private/sanitized diagnostic bundle;
- track only fixed classifications + non-sensitive hash/length/timestamps in Git;
- retain last successful lifecycle stage (server bind, client start, QUIC/UDP observed, TLS/auth stage where exposed) without credentials/topology;
- preserve fair bind/connect, TLS pin, fresh-client lifecycle and symmetric measurement contract already repaired;
- no broad HY2 harness rewrite and no security weakening.

**Validation:** deterministic failure-path tests + full local gate + `git diff --check`; push and require exact-head CI green before G.

**Continue immediately to G:** yes.

---

### G — One materially changed fair HY2/Nekomusume paired attempt

**Dependency:** F exact-head CI green and diagnostics materially differ from the retained HY2 client-exit attempts.

- same owned client/VPS + pinned HY2 v2.9.3;
- same payload/security/load/lifecycle contract;
- up to 5 pairs only if both sides satisfy fair lifecycle requirements;
- concurrency 1, fresh unprivileged ports, total <10 min;
- bounded capture/diagnostics on experiment ports only;
- no production network changes.

Success: retain complete raw pair rows, median/P95/failures, exact application bytes/hash and symmetric resource scope; no superiority claim.

Failure: retain typed discriminating diagnostic, no comparative summary and no unchanged retry.

**Continue immediately to H:** yes.

---

### H — Prepare the independent release/security review evidence map

**Status:** `READY_LOCAL`, fully independent fallback.

Create a compact reviewer map linking rather than duplicating:

- resource/abuse limits + pre-auth admission;
- Noise/replay/nonce boundaries;
- compatibility policy + canonical corpus freeze scope;
- Session delivery vs packet ACK separation;
- package install/upgrade/rollback + binary provenance;
- lifecycle/readiness/shutdown/cleanup;
- exact positive/negative/blocked release-matrix rows;
- HY2 methodology/status;
- remaining environment/implementation blockers.

State explicitly: **review preparation, not an independent security audit**.

**Continue immediately to I:** yes.

---

### I — Native VPS/package/resource opportunity only when it answers a live release question

Do only if it will not contaminate WAN/performance samples and current package/resource lineage materially changed:

- exact-head native release build/package reproducibility;
- bounded install/smoke/upgrade/rollback/readiness/cleanup in dedicated experiment path;
- low-concurrency leak/pathological-growth observation tied to a current real-session question.

If no new question exists, mark not applicable. Do not run generic microbench/fuzz merely to occupy the VPS.

**Continue immediately to J:** yes.

---

### J — Release-matrix closure audit and next-phase gate

Audit bounded release-evidence item 3 from repository facts:

- IPv4;
- IPv6 environment status;
- controlled vs natural UDP degradation/fallback;
- periodic/longer bounded Session;
- NAT/source-endpoint change;
- repeated cross-process failover/recovery;
- HY2 comparison;
- package/operator/resource evidence;
- exact-head CI/evidence provenance.

Close item 3 only if its declared acceptance boundary is genuinely satisfied. Otherwise keep it unchecked and name only real remaining blockers. Do not weaken criteria to reach RC.

Governance remains `RELEASE_CANDIDATE=false`, `FREEZE=false`, `PRODUCTION_READY=false`, `RELEASED=false`. Independent release/security review and a separate RC decision remain later gates.

**Stop after J only if** remaining work genuinely requires maintainer value judgment, new credentials/server/third-party access, action outside standing authorization, a major architecture choice, or independent external review unavailable to the coding agent. Otherwise continue newly discovered dependency-safe work.

## Queue-wide continuation rules

- Complete coherent slice -> validate -> commit/push -> immediately consume next pre-authorized dependency-safe slice.
- Do not stop after an arbitrary hour, one commit, CI submission or reviewer interval.
- CI pending blocks only slices explicitly requiring exact-head green CI; use waits for C0/F/H work.
- Runtime/tool-budget forced stop is legitimate; record exact checkpoint and resume next wake.
- Any VPS failure remains evidence; preserve it and continue independent READY lanes.
- No unchanged WAN/HY2 reruns.
- **Repeated-failover tooling is now capped:** after A3, allow only A4 automatically. If A4 is again zero-row orchestration failure, move on rather than opening another generic runner project.
- `docs/CHATGPT_HANDOFF.md` remains reviewer-owned; coding agent reads only.

## Completion gates for this rolling queue

- A3 turns `missing start` into a role-specific, observed-readiness startup contract without exposing private details.
- A4 produces either valid repeated controlled-failover evidence or a final discriminating blocker for this instrumentation line.
- C0/C1 add one distinct longer periodic bounded sample or retain an honest typed blocker/negative.
- D reconciles status without erasing historical negatives or promoting controlled faults to natural WAN behavior.
- E classifies implementation-blocked rows from executable reality and runs only genuinely ready ones.
- F/G either produce the first fair complete HY2 pair set or a materially more discriminating negative without security weakening.
- H leaves an auditable independent-review preparation map without pretending the review already happened.
- Item 3 remains open unless its declared matrix boundary is truly satisfied.
- Governance flags remain unchanged absent later reviewed decisions.

## Do not expand into

- protocol byte changes merely to make experiments pass;
- new Session/Carrier/ACK/crypto architecture without ADR-level evidence/decision;
- raw stderr, endpoint address/user/private topology or credentials in Git;
- production route/firewall/qdisc/DNS/proxy/tunnel changes;
- third-party targets/scanning;
- >10 minute single runs, >256 MiB application traffic, >32 concurrent sessions, or long-lived experimental daemons without new authorization;
- enabled FEC, 0-RTT, striping/aggregation or exotic carriers without an observed-problem gate;
- rewriting historical negative evidence into success;
- RC/security/production claims from bounded self-owned lab evidence.

## Questions requiring maintainer decision

none.
