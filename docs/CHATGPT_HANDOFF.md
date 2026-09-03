# Nekomusume ChatGPT Handoff

Checked at: 2026-09-04 00:02 Asia/Shanghai
Repository HEAD reviewed: `a117086fa69553a36021137900b6052050624a8b`
Previous reviewed coding/evidence HEAD: `9fd24112763469c27b97ffdbdcccdb149259aae7`
Previous reviewer handoff commit: `aff72f8cb1ecef1f747c0888fc51596fab8fc836`

## What changed

Two coding-agent commits landed after the previous reviewer handoff.

- `c156868` — **evidence archive/status boundary only; no protocol/runtime change.** It preserves the exact `9fd2411` one-invocation cross-host attempt as a schema-valid typed negative: 0/6 completed cycles, cycle 1 `invalid_cycle_evidence`, launcher exit 1 after 9,758 ms, exact staged binary identity matched on both endpoint descriptors, and final independent residue observation was zero. The retained batch correctly does **not** claim WAN failover, runtime correctness, public reachability or production behavior. It also records that the inner collector stderr was not retained by the outer batch, so the deeper exception cannot be recovered truthfully from committed evidence.
- `a117086` — **cross-host evidence-truthfulness repair; no wire/Session/Noise/failover semantic change and no new positive VPS behavior evidence yet.** It closes the previous R-001 through R-004 integration defects: the shell-free outer plan now carries structured endpoint descriptors into `NEKO_FAILOVER_ENDPOINTS_JSON`; an endpoint labeled `execution="ssh"` must bind the transport executable to an explicitly declared SSH executable via same-file identity; remote server CPU/RSS/FD is no longer fabricated from the local SSH process and is recorded as `not_collected_remote`; remote cleanup requires a bounded remote `processes_remaining` observation in addition to listener state; and `endpoint_provenance` is required by both cycle schema and batch validation.

Exact `a117086` GitHub Actions run `33773430804` completed successfully:

- `stable checks` — success (`bash scripts/check.sh`);
- `nightly decode fuzz smoke` — success (30-second / 8192-byte decode fuzz smoke).

The previous cross-host truthfulness gate is therefore closed. The repository is again one small instrumentation step away from a high-value changed-hypothesis repeated-failover VPS run.

### R-005 HIGH — the outer batch still destroys the only discriminating collector diagnostic on nonzero exit

The exact `9fd2411` typed negative demonstrated a concrete evidence-loss problem that remains in current code:

- `run-live-warm-failover-cycle.py` returns nonzero and writes its bounded collector exception to stderr;
- `run-repeated-warm-failover.py` invokes the cycle command with `capture_output=True`;
- on nonzero exit it records only `collector returned nonzero without a valid row` and drops `completed.stderr` entirely.

This is not a protocol/runtime correctness failure, and it does not invalidate the retained `9fd2411` negative. It is an **evidence-diagnostics blocker for the next paid retry**: if the materially changed `a117086` cross-host path fails again, the project would again know only that the collector failed, wasting another VPS window without a discriminating reason.

Do not archive raw collector stderr in Git, because command/process errors can contain endpoint paths, usernames or other private lab details. Preserve it only as bounded private local diagnostics while committing only non-sensitive hashes/lengths/fixed classifications.

## Review verdict

**CONTINUE_WITH_REQUIRED_FIXES — previous cross-host HIGH findings are closed and exact-head CI is green; add one narrow privacy-preserving failure-diagnostic seam, then immediately run the changed-hypothesis six-cycle VPS batch and consume the rolling queue continuously**

Do not reopen endpoint abstraction, protocol semantics or generic runner architecture. R-005 is intentionally small. Once it is green, return to the VPS. The hourly reviewer interval is not a reason to stop after any slice below.

## Evidence boundaries

- `IMPLEMENTATION_COMPLETE=true` remains the bounded research baseline.
- `CANONICAL_CORPUS_V1_FROZEN=true` remains corpus-specific only.
- `RELEASE_CANDIDATE=false`, `PRODUCTION_READY=false`, `FREEZE=false`, and `RELEASED=false` remain correct.
- `c156868` preserves a typed negative; it does not add runtime success evidence.
- `a117086` repairs evidence collection/integration only. It does not itself prove cross-host failover behavior.
- Exact `a117086` has independent green stable CI and decode fuzz smoke. This is CI evidence, not WAN evidence, a security audit, release approval or production evidence.
- Exact `25e0daa` remains the accepted single controlled application-level UDP reply-cessation warm-fallback row: 3/3 logical records, 48 application bytes, two uncertain/replayed ranges, duplicate/lost 0, about 434 ms failure-decision-to-first-resumed-data. It is not natural degradation/PTO-blackhole evidence.
- Exact `25e0daa` remains one accepted approximately five-minute direct periodic Session: 60 x 32 B, 60/60 confirmed, no missing/duplicate/conflict. It is one bounded sample, not a reliability rate or production long-lived conclusion.
- Exact `9fd2411` remains 0/6 at the orchestration/evidence-collection boundary. Its exact inner collector exception is unknowable from retained evidence; do not retrospectively guess it from the later `a117086` repairs.
- Historical HY2 negatives remain valid; there is still no complete fair paired comparison.
- NAT/source-endpoint change, live migration-back, live key update and live PMTUD remain implementation-blocked until executable reality changes. IPv6 remains environment-blocked.
- Endpoint/user/address/private topology and raw private diagnostics remain local/untracked. Repository artifacts may retain only bounded non-sensitive metadata, hashes and classifications.
- Standing authorization covers bounded self-owned TCP/UDP failover, periodic Session, diagnostics, bounded capture, resource observation and cleanup within <=10 minutes / <=256 MiB / <=32 sessions. No per-run WAN approval is required.

## Rolling Work Queue

This queue is intentionally much larger than one reviewer interval. **Complete a coherent slice -> run its required gates -> commit/push -> immediately continue to the next dependency-satisfied pre-authorized slice.** Do not stop after one commit, one hour or CI submission merely because the reviewer has not run again.

### A0 — Close the cross-host endpoint truthfulness gate

**Status: CLOSED by `a117086`; exact-head CI green.**

The shell-free structured endpoint path, verified SSH executable class, truthful remote-resource absence, bounded remote cleanup postcheck and required endpoint provenance are accepted as evidence infrastructure. Do not polish this layer further unless a new concrete failure demonstrates a defect.

**Continue immediately to A1:** yes.

---

### A1 — Preserve bounded private collector diagnostics without leaking them into committed evidence

**Status:** `READY_LOCAL`; narrow required precursor to the next live retry.

**Why now**

The retained exact-`9fd2411` failure proves that the current outer batch discards the only deeper collector diagnostic. A second live attempt without changing this instrumentation would risk another non-discriminating negative.

**Likely files**

- `scripts/bench/run-repeated-warm-failover.py` + deterministic tests;
- `scripts/bench/run-repeated-warm-failover-command.py` + tests only if the private diagnostic destination must be passed through the shell-free wrapper;
- schema only if new **non-sensitive** failure metadata is added to the tracked batch.

**Required contract**

1. On a nonzero cycle-collector exit, do not place raw stderr in the tracked batch JSON.
2. Retain the raw child stderr only in an explicitly local/untracked private diagnostic path, bounded in size and created with restrictive permissions. A per-cycle sidecar is acceptable. Cap retained stderr (for example <=64 KiB/cycle); overflow must be explicit rather than unbounded.
3. The tracked `first_failure` may retain only non-sensitive deterministic metadata such as:
   - collector exit code;
   - stderr byte count;
   - stderr SHA-256;
   - a fixed broad stage/classification (`collector_nonzero`, `collector_timeout`, `invalid_stdout`, etc.) that does not embed raw error text.
4. If no private diagnostic destination is configured, still fail closed and retain at least hash/length/exit metadata without raw text. Do not make diagnostics required for normal successful runs.
5. Success-path row/schema semantics must remain unchanged.
6. Add regression tests proving:
   - a synthetic secret/endpoint string written to child stderr never appears in tracked batch JSON;
   - the private sidecar contains the diagnostic only in the test-private path and is size-bounded;
   - SHA-256/byte count match the retained private bytes;
   - malformed stdout + stderr and nonzero/no-stdout cases remain typed negatives;
   - success rows remain unchanged;
   - no shell reparsing is introduced.

Do not redesign the collector exception hierarchy merely to complete A1. The immediate purpose is to stop losing the private discriminating diagnostic.

**Validation:** targeted repeated-runner/command-wrapper regressions, `bash scripts/check.sh`, `git diff --check`. Protocol fuzz need not be rerun solely for this evidence-harness change unless CI does so automatically.

**Commit/push condition:** one small coherent evidence-diagnostics commit, normal push.

**Continue immediately:** while exact-head CI is pending, C0/F/H below are pre-authorized. Once exact-head CI is green, go directly to A2 without waiting for reviewer acknowledgment.

---

### A2 — Execute one materially changed six-cycle real warm-failover VPS batch

**Dependency:** A1 pushed and exact-head CI green.

**Status after dependency:** `READY_LIVE`.

**Changed hypothesis**

This is not an unchanged retry of exact `9fd2411`: `a117086` materially repairs structured cross-host endpoint execution/provenance/resource/cleanup truthfulness, and A1 adds discriminating private failure diagnostics.

**Execution profile**

- self-owned client + self-owned VPS only;
- exact executed checkout HEAD and exact staged executable SHA-256/size recorded;
- one outer aggregator invocation only;
- exactly 6 sequential fresh server/client cycles, concurrency 1;
- controlled application-level UDP reply cessation only;
- preferred 3 logical records x 16 B if current CLI contract remains unchanged;
- fresh/unprivileged ports inside the existing bounded CLI range;
- no retry of a failed cycle inside this batch;
- complete setup + cycles + cleanup comfortably below 10 minutes (prefer <=540 s outer budget);
- no production firewall/route/qdisc/DNS/proxy/tunnel/service changes;
- local endpoint plan and raw diagnostic sidecars remain untracked/private.

**Required successful-row evidence**

- canonical negotiation and authenticated Session identity;
- UDP delivery confirmation before controlled failure;
- warm TCP negotiation/authentication/resume validation;
- exactly three authenticated readiness proofs before promotion;
- no TCP application data before promotion where the existing contract observes it;
- `UNCERTAIN -> replay -> DeliveryAck` accounting;
- confirmed/uncertain/replayed/duplicate/lost/conflict records/bytes;
- recovery timing where exposed;
- client/server exit state;
- local client resources at truthful local scope;
- remote server resources only if genuinely collected remotely, otherwise explicit `not_collected_remote`;
- verified remote process/listener cleanup and local cleanup;
- exact commit/binary identity and required endpoint provenance.

**Outcome boundary**

- 6/6 passes = one bounded repeatability batch for this controlled application-fault seam only;
- partial/failed cycle = valid typed partial/negative; retain valid prefix + first failure + non-sensitive diagnostic hash metadata;
- never promote this to natural packet loss/PTO blackhole, public reachability, reliability rate or production evidence.

**Commit/push condition:** archive minimal non-sensitive batch/result summary + hashes; run evidence validators and `git diff --check`; commit/push.

**Continue immediately to B:** yes, regardless of pass/fail unless a real runtime correctness blocker is discovered.

---

### B — Classify and close the repeated-failover batch

**Dependency:** A2 produced a complete batch or typed partial/negative.

1. Validate every row and batch identity/provenance/cleanup invariant.
2. Record batch SHA-256, exact executed commit/binary, actual parameters and cleanup scope.
3. For 6/6, compute only bounded descriptive summaries directly supported (e.g. median/P95 recovery timing, failures=0); do not infer a reliability rate.
4. On failure, use the A1 private diagnostic only for local diagnosis; commit only a sanitized classification/hash/size and preserve the private/raw boundary.
5. Classify failure as one of `runtime_correctness`, `orchestration_evidence`, `environment_path`, `cleanup`, or `unknown_after_bounded_diagnostics` only when evidence supports it.
6. Runtime correctness -> deterministic reproducer + correctness repair before another failover run.
7. Orchestration/evidence-only failure -> repair exactly that seam; one later materially changed retry is permitted, but do not block independent C/F/H/I work.

**Validation:** targeted runner/schema/evidence checks + `scripts/check.sh` + `git diff --check`.

**Commit/push condition:** coherent evidence/repair commit.

**Continue immediately to C0:** yes if no cross-cutting correctness blocker exists.

---

### C0 — Close the periodic zero-client orchestration gap with the smallest shell-free wrapper

**Status:** `READY_LOCAL`, independent fallback during A1/A2 CI or after B.

The accepted ~5-minute direct periodic row is real, but exact `c4786dc` later invoked zero periodic clients. Build only the minimum exact-head-attributed wrapper around the existing real `periodic-server` / `periodic-client` path.

Required properties:

- shell-free argv;
- local/untracked endpoint plan;
- exact executable SHA-256/size + checkout commit attribution;
- deterministic dry-run proving real periodic client dispatch is entered;
- malformed plan/argv fail closed;
- dry-run explicitly `live_evidence=false`;
- truthful local/remote resource and cleanup scope; reuse the A0 principle instead of measuring SSH as Nekomusume.

Prefer a small wrapper around the already-successful direct/manual mechanism, not another generic orchestration framework.

**Validation:** targeted tests + full local gate + `git diff --check`; push and require exact-head green CI before C1.

**Continue immediately to C1:** yes when CI green. F/H are pre-authorized while waiting.

---

### C1 — Run one scientifically distinct longer periodic direct-path Session

**Dependency:** C0 exact-head CI green.

Recommended bounded profile:

```text
application phase: about 480 s
interval: 5 s
about 96 records
payload: 32 B/record
concurrency: 1
```

Keep complete setup + application + cleanup below 10 minutes; shorten application phase rather than exceed authorization.

Record exact identity, actual duration, records/bytes, confirmed/missing/duplicate/conflict, truthful latency raw/median/P95 if measured, local/remote resource scope, client/server exits and cleanup. One success remains one bounded sample; one failure is retained with no unchanged retry.

**Commit/push condition:** minimal non-sensitive evidence artifact/summary + hashes.

**Continue immediately to D:** yes.

---

### D — Reconcile release-matrix and resilience status after A2/C1

Update only from actual evidence:

- `docs/era4-e-resilience.md`;
- `docs/status.md`;
- `IMPLEMENTATION_PLAN.md`;
- `ROADMAP.md`.

Rules:

- repeated cross-process failover/recovery becomes positive only to the exact controlled-seam/repeatability extent proved;
- natural UDP degradation/PTO blackhole remains unchecked unless separately observed;
- longer periodic evidence remains bounded to exact duration/sample;
- preserve exact `9fd2411` and all other historical negatives/hashes;
- keep release-evidence item 3 unchecked while declared matrix gaps remain;
- do not change RC/global freeze/production/release flags.

**Validation:** status/plan/release-boundary checks + `scripts/check.sh` + `git diff --check`.

**Continue immediately to E:** yes.

---

### E — Audit the highest-value implementation-blocked release rows and unlock only within accepted architecture

**Priority order:** genuine NAT/source-endpoint change, then migration-back, live key update, live PMTUD — unless repository facts show another row is materially closer to live execution.

Classify each as:

- `READY_LIVE`;
- `SMALL_LOCAL_UNLOCK`;
- `BLOCKED_IMPLEMENTATION_ARCHITECTURE`;
- `BLOCKED_ENVIRONMENT`;
- `ALREADY_SUFFICIENT_FOR_CURRENT_BOUNDARY`.

If a row is `READY_LIVE`, execute one bounded owned-endpoint experiment immediately. If `SMALL_LOCAL_UNLOCK`, implement only the smallest seam using already-accepted Session/Carrier identity/anti-replay semantics, test/gate/push/CI, then run once. If it requires a new migration identity/rebinding design or production network manipulation, mark blocked and continue. Do not invent architecture to fill a release matrix.

**Continue immediately to F:** yes unless a cross-cutting blocker appears.

---

### F — Upgrade HY2 failure diagnostics locally; no unchanged `client_exit` retry

**Status:** `READY_LOCAL`, safe fallback while another exact-head CI is pending.

Turn the retained HY2 `client_exit` into a discriminating diagnostic contract:

- prove real HY2 stderr/log path feeds a sanitizer/private diagnostic bundle;
- retain last successful harness stage;
- classify config/TLS-auth/client-process/network-path only when evidence supports it;
- allow raw private log retention locally with committed SHA-256/size only;
- preserve cleanup truthfully;
- deterministic leak tests for secret/endpoint/private-topology strings;
- prepare bounded temporary-port capture metadata if packet direction is required;
- do not loosen TLS/auth/security equivalence.

Do not reopen generic harness fairness work already accepted.

**Validation:** targeted tests + full local gate + exact-head CI.

**Continue immediately to G:** yes when CI green; H is pre-authorized while waiting.

---

### G — One materially changed HY2/Nekomusume owned-lab attempt

**Dependency:** F exact-head CI green and instrumentation/hypothesis materially differs from exact `3d54585`.

- same owned client/VPS and pinned HY2 v2.9.3;
- same application payload/security/load contract;
- 5 paired samples only if both sides satisfy the fair lifecycle contract;
- concurrency 1, fresh unprivileged ports, total <=10 min;
- bounded diagnostics/capture on experiment ports only;
- no production network changes.

Success: retain raw complete pairs, median/P95/failures, exact application bytes/hash and symmetric resource scope; no superiority claim.

Failure: retain typed discriminating diagnostic; no comparative summary and no unchanged retry.

**Continue immediately to H:** yes.

---

### H — Prepare the independent release/security review evidence map

**Status:** `READY_LOCAL`, fully pre-authorized fallback for CI waits or blocked VPS lanes.

Create a compact reviewer map linking rather than duplicating:

- resource/abuse limits and pre-auth admission;
- Noise/replay/nonce boundaries;
- compatibility policy + canonical corpus freeze scope;
- Session delivery vs packet ACK separation;
- package install/upgrade/rollback + binary provenance;
- operator lifecycle/readiness/shutdown/cleanup;
- exact positive/negative/blocked release-matrix rows;
- HY2 methodology/status;
- remaining environment/implementation blockers.

State explicitly: review preparation, **not** an independent security audit.

**Continue immediately to I:** yes.

---

### I — Native VPS/package/resource opportunity while higher-priority lanes wait

Only when it does not interfere with WAN/performance samples and answers a current release question:

- exact-head/native release build or package reproducibility after meaningful package/build changes;
- bounded install/smoke/upgrade/rollback/readiness/cleanup in the dedicated experiment path if package lineage changed since accepted evidence;
- low-concurrency leak/pathological-growth resource observation only when a current real-session question needs it;
- no repeated generic microbench/fuzz just to occupy the VPS.

If no new package/resource question exists, mark this slice not applicable and continue. Do not manufacture evidence.

**Continue immediately to J:** yes.

---

### J — Release-matrix closure audit and next-phase gate

After consuming all currently READY evidence lanes or retaining them as typed blockers/negatives, audit bounded release-evidence item 3 from repository facts:

- IPv4 evidence;
- IPv6 environment status;
- controlled vs natural UDP degradation/fallback;
- periodic/longer bounded Session;
- NAT/source-endpoint change;
- repeated cross-process failover/recovery;
- HY2 comparison;
- package/operator/resource evidence;
- exact-head CI/evidence provenance.

Close item 3 only if its declared acceptance boundary is genuinely satisfied. Otherwise keep it unchecked and name only real remaining blockers. Do not weaken acceptance criteria to reach RC.

`RELEASE_CANDIDATE=false`, `FREEZE=false`, `PRODUCTION_READY=false`, and `RELEASED=false` remain unchanged. Independent release/security review and a separate RC decision remain later gates.

**Stop after J only if:** remaining work genuinely requires maintainer value judgment, new credentials/server/third-party access, action outside standing authorization, a major architecture choice, or independent external review unavailable to the coding agent. Otherwise continue newly discovered dependency-safe work.

## Queue-wide continuation rules

- Complete coherent slice -> validate -> commit/push -> immediately consume the next pre-authorized dependency-safe slice.
- Do not stop after an arbitrary hour, one commit, CI submission or reviewer interval.
- CI pending blocks only slices explicitly requiring exact-head green CI; use the wait for independent C0/F/H work when safe.
- Runtime/tool-budget forced stop is legitimate; record exact checkpoint and resume next wake.
- Any VPS failure remains evidence; preserve it and continue independent READY lanes.
- No unchanged WAN/HY2 reruns.
- Do not spend another full local cycle polishing cross-host endpoint tooling after A1. Return to VPS.
- `docs/CHATGPT_HANDOFF.md` remains reviewer-owned; coding agent reads only.

## Completion gates for this rolling queue

- A1 prevents another paid failure from destroying the only collector diagnostic while keeping raw/private material out of Git.
- A2 produces either a valid six-cycle controlled-failover batch or a discriminating typed partial/negative with cleanup and exact provenance.
- C0/C1 either add one distinct longer periodic bounded sample or retain an honest typed blocker/negative.
- D reconciles status without erasing historical evidence or promoting controlled faults to natural WAN behavior.
- E classifies the implementation-blocked rows from executable reality and runs only genuinely ready ones.
- F/G either produce the first fair complete HY2 pair set or a materially more discriminating negative without security weakening.
- H leaves an auditable evidence map for later independent review without pretending that review already happened.
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
