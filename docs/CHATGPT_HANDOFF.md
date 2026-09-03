# Nekomusume ChatGPT Handoff

Checked at: 2026-09-03 23:00 Asia/Shanghai
Repository HEAD reviewed: `9fd24112763469c27b97ffdbdcccdb149259aae7`
Previous reviewed coding/evidence HEAD: `a6003ccdbfd2fb995e96c39bd6ae53c2e9f2ad5b`
Previous reviewer handoff commit: `c35017d10a20bcff23ae89eacc85c7fc65186b90`

## What changed

One coding-agent commit landed after the previous reviewer handoff:

- `9fd2411` — **structured endpoint / remote-exec harness work; no protocol/wire/Session/Noise/failover semantic change and no new VPS behavior evidence yet.** It adds a bounded `remote-endpoint-exec.py` verifier/spawner, allows the live warm-failover adapter to describe local vs SSH-style endpoints with per-endpoint binary identity, adds endpoint provenance to cycle output/schema, and adds deterministic cross-host-shaped tests. It also preserves the existing exact commit + binary SHA-256/size identity contract.

Exact `9fd2411` GitHub Actions run `33765521128` is green:

- `stable checks` — success (`bash scripts/check.sh`);
- `nightly decode fuzz smoke` — success (30-second / 8192-byte decode fuzz smoke).

The commit materially advances the previously missing cross-host execution seam, but this review found three evidence/integration defects that must be closed before the paid six-cycle VPS batch. These are harness/evidence defects, not protocol-runtime correctness defects.

### R-001 HIGH — the shell-free outer plan still cannot actually select the new structured remote endpoints

`run-live-warm-failover-cycle.py` now accepts `NEKO_FAILOVER_ENDPOINTS_JSON`, but `scripts/bench/run-repeated-warm-failover-command.py` still accepts only per-cycle `server_command`, `client_command`, and `cleanup_command`, and its `adapter_env()` still emits only the legacy `NEKO_FAILOVER_SERVER_COMMAND_JSON` / `CLIENT_COMMAND_JSON` variables.

Therefore the documented shell-free outer path used by the rolling queue cannot yet feed the new SSH endpoint descriptors into the real cycle adapter. The local preflight can still prove Python dispatch, but that does not make the real repeated cross-host batch runnable through the intended wrapper.

### R-002 HIGH — `execution="ssh"` is currently a label, not a mechanically enforced transport fact

For a structured endpoint with `execution="ssh"`, the adapter validates `transport_argv` only as a bounded argv array and then executes it. The deterministic test deliberately labels the server endpoint as `ssh` while using the local Python interpreter plus `remote-endpoint-exec.py` as the transport, and that row passes.

That is useful for local protocol-shape testing, but it proves that a future evidence row may say `execution="ssh"` even when no SSH program or cross-host transport was used. A real VPS artifact must not derive a cross-host/SSH claim from that field unless the live path mechanically verifies the transport class.

### R-003 HIGH — remote server resource/cleanup scope is currently misattributed to the local transport process

For an SSH endpoint, `execution_argv()` returns the local transport argv. `sampled_command(... role="server" ...)` then wraps that transport process with `process-resource-sampler.py --implementation nekomusume --role server`.

The sampler is Linux `/proc`-local and process-group-local. On a real remote server path it measures the local SSH transport process/group, not the remote Nekomusume server CPU/RSS/FD/socket state. The resulting `resources.server` would therefore be mislabeled if treated as remote Nekomusume resource evidence. Likewise, `server_process_reaped` derived only from the local sampler proves the transport process group ended, not independently that no remote experiment process remains.

This must be made truthful before live evidence collection. The fastest acceptable repair is to **omit/null remote server resource metrics and label their scope honestly**, while obtaining remote process/listener cleanup from an explicit remote postcheck/cleanup contract. Do not build a large remote telemetry subsystem merely to fill fields.

### R-004 MEDIUM — endpoint provenance is still optional to downstream validation

The current live adapter emits `endpoint_provenance`, but both the JSON schema and `run-repeated-warm-failover.py` treat it as optional. For the now cross-host-capable evidence format, a retained row can therefore lose the very provenance that distinguishes local vs remote endpoint execution and still pass the batch validator.

Because the current adapter emits provenance for both legacy-local and structured endpoints, make it required for new cycle evidence rather than preserving a silent optional hole.

## Review verdict

**CONTINUE_WITH_REQUIRED_FIXES — exact-head CI is green and the cross-host seam is close, but one small evidence-integration closure is required before the six-cycle VPS batch; after that, run the VPS batch immediately and consume the rolling queue without waiting for another reviewer interval**

Do not reopen protocol semantics, do not add another generic adversarial-review layer, and do not spend another cycle polishing unrelated runner abstractions. Close R-001 through R-004 as one coherent harness/evidence slice, run the local gates, push, obtain exact-head CI, then execute the authorized real VPS batch.

## Evidence boundaries

- `IMPLEMENTATION_COMPLETE=true` remains the bounded research baseline.
- `CANONICAL_CORPUS_V1_FROZEN=true` remains corpus-specific only.
- `RELEASE_CANDIDATE=false`, `PRODUCTION_READY=false`, `FREEZE=false`, and `RELEASED=false` remain correct.
- `9fd2411` is harness/evidence infrastructure. It is **not** a cross-host behavior result by itself.
- Exact `9fd2411` has independent green stable CI and decode fuzz smoke. This is CI evidence, not WAN evidence, security audit, release approval, or production evidence.
- The deterministic `execution="ssh"` test intentionally uses a local Python transport. It proves descriptor/remote-helper behavior, not a real SSH or cross-host path.
- Exact `25e0daa` remains the accepted single controlled application-level UDP reply-cessation warm-fallback row: 3/3 logical records, 48 application bytes, two uncertain/replayed records, duplicate/lost 0, about 434 ms failure-decision-to-first-resumed-data. It is not natural degradation/PTO-blackhole evidence.
- Exact `25e0daa` remains one accepted approximately five-minute direct periodic Session: 60 x 32 B, 60/60 confirmed, no missing/duplicate/conflict. It is one bounded sample, not a reliability rate or production long-lived conclusion.
- Historical exact `1bf848d`, `07545f0`, and `c4786dc` negatives remain orchestration/pre-application evidence, not runtime failures.
- NAT/source-endpoint change, live migration-back, live key update, and live PMTUD remain implementation-blocked until executable reality changes. IPv6 remains environment-blocked.
- HY2 remains diagnostics-blocked after the retained `hy2-1 client_exit`; no valid paired comparison exists.
- Endpoint/user/address/private topology and local experiment plans remain untracked/local. Repository artifacts must not expose them.
- Standing authorization covers the bounded self-owned TCP/UDP work below within <=10 minutes / <=256 MiB / <=32 sessions. No per-run WAN approval is required.

## Rolling Work Queue

This queue is intentionally multi-hour. **Complete one coherent slice -> validate -> commit/push -> immediately consume the next pre-authorized dependency-safe slice. Do not stop merely because the reviewer has not run again.**

### A0 — Close structured cross-host evidence truthfulness and outer-plan integration

**Status:** `READY_LOCAL`, required before the next real repeated-failover VPS batch.

**Goal**

Make the new `9fd2411` remote endpoint path executable through the shell-free repeated-command wrapper and ensure a real artifact cannot mislabel local transport/resource observations as remote Nekomusume evidence.

**Likely files**

- `scripts/bench/run-repeated-warm-failover-command.py` + tests;
- `scripts/bench/run-live-warm-failover-cycle.py` + tests;
- `scripts/bench/remote-endpoint-exec.py` + tests only as needed;
- `scripts/bench/run-repeated-warm-failover.py` + tests;
- `schema/repeated-warm-failover.v1.json`;
- `scripts/check.sh` only to wire new deterministic regressions.

#### A0.1 Wire structured endpoints into the real shell-free plan

Extend the untracked JSON plan contract so each cycle can truthfully describe:

- one server endpoint and one client endpoint;
- `local` or remote/SSH execution as applicable;
- underlying binary path/hash/size/commit identity;
- shell-free endpoint argv;
- bounded transport argv for the remote endpoint;
- cleanup/postcheck argv.

The outer dispatcher must pass `NEKO_FAILOVER_ENDPOINTS_JSON` to the live adapter. Do not keep two contradictory live sources of truth for one cycle. Legacy local-only command variables may remain for deterministic/backward local tests, but the paid cross-host path must consume the structured descriptors through the same shell-free wrapper.

Add a deterministic outer-plan test that reaches the actual live adapter dispatch with one local client + one remote-shaped server descriptor. It must prove the endpoint descriptor survives outer plan -> dispatch -> adapter without shell reparsing.

#### A0.2 Make transport classification truthful

Do not let an arbitrary command array create an evidence claim of `execution="ssh"`.

Use the smallest truthful contract. Acceptable approaches include:

- an untracked plan field naming the intended SSH executable and a same-file/resolved-executable check before an endpoint may be labeled `ssh`; or
- rename the generic execution class to a non-claiming transport label and add a separately verified `ssh` classification only on the live plan path.

Do **not** record host/user/address in the committed result. The evidence only needs to know that the configured live transport class was actually the verified SSH program; endpoint ownership/address stays in local secret configuration and experiment notes may retain only sanitized/hash metadata.

The deterministic local Python transport test should remain, but it must no longer be able to produce a row that semantically claims verified SSH execution.

#### A0.3 Fix remote server resource/cleanup scope without building a telemetry project

For a remote server endpoint:

- do not emit local SSH CPU/RSS/FD/socket observations as `resources.server` for Nekomusume;
- either set remote server resource metrics to `null` / explicit `not_collected_remote`, or collect real remote Nekomusume metrics only if an already-small, bounded remote sampler integration is straightforward;
- label any local transport-process resource observation separately if retained, and never mix it into Nekomusume server performance summaries;
- derive remote process/listener cleanup from an explicit remote cleanup/postcheck result, not from reaping the local SSH process group alone;
- keep local client resource sampling at its truthful local Nekomusume process scope.

The minimal preferred path is truthful absence for remote server resource metrics plus a bounded remote process/listener postcheck. Missing remote CPU/RSS/FD is better evidence than measuring SSH and calling it Nekomusume.

#### A0.4 Require endpoint provenance in every current cycle row

Now that the adapter emits provenance for local and structured endpoints:

- make `endpoint_provenance` required in the cycle schema;
- make `validate_cycle()` require it, not optional;
- require exactly server then client roles with binary identity matching the cycle identity;
- retain no transport argv, host, address, username, secret or private topology in the row.

#### A0.5 Regression / gates

Add deterministic tests proving at minimum:

- outer shell-free plan reaches structured endpoint adapter dispatch;
- local Python fake-transport cannot be labeled verified SSH;
- real-SSH-class path fails closed if the declared SSH executable does not match the transport executable;
- remote endpoint result cannot report local transport resources as remote Nekomusume server resources;
- remote cleanup/process observation is required for a `verified` cleanup status;
- deleting `endpoint_provenance` makes batch validation fail;
- all existing malformed argv/JSON, provenance, event-order, accounting, timing, privacy, cleanup and shell-free tests remain green.

Run targeted Python regressions, `bash scripts/check.sh`, `git diff --check`; protocol fuzz need not be rerun solely for evidence-harness changes unless the repository workflow does so automatically.

**Commit/push condition:** one coherent harness/evidence repair commit, normal push, exact-head CI green.

**Continue immediately to A1:** yes when exact-head CI is green. While CI is pending, C0/G/I preparation below is pre-authorized if independent.

---

### A1 — Execute one changed-hypothesis six-cycle real warm-failover VPS batch immediately

**Dependency:** A0 pushed and exact-head CI green.

**Status after dependency:** `READY_LIVE`.

**Goal**

Obtain bounded repeatability evidence from the real `failover-server` / `failover-client` runtime using the shell-free structured local-client + owned-VPS-server path.

**Execution profile**

- self-owned client + self-owned VPS only;
- exact executed checkout HEAD and exact staged executable SHA-256/size recorded;
- one outer aggregator invocation;
- exactly 6 sequential fresh server/client cycles, concurrency 1;
- controlled application-level UDP reply cessation only;
- preferred workload 3 logical records x 16 B if the current live CLI contract remains unchanged;
- fresh/unprivileged ports within the existing bounded CLI range;
- no retry of a failed cycle inside the same batch;
- complete setup + six cycles + cleanup comfortably below the standing 10-minute limit (prefer <=540 s outer budget);
- no production firewall/route/qdisc/DNS/proxy/tunnel/service changes;
- no endpoint/private-topology plan committed.

**Required per-cycle evidence**

- canonical negotiation + authenticated Session identity;
- UDP confirmation before controlled failure;
- warm TCP negotiation/authentication/resume validation;
- exactly three authenticated readiness proofs before promotion;
- no TCP application data before promotion where the current contract observes it;
- `UNCERTAIN -> replay -> DeliveryAck` accounting;
- confirmed/uncertain/replayed/duplicate/lost/conflict records/bytes;
- recovery timing where exposed;
- client/server exit state;
- local client resource metrics at truthful scope; remote server metrics only if genuinely collected remotely, otherwise explicit absence;
- verified remote process/listener cleanup plus local cleanup;
- exact commit + binary identity + required endpoint provenance.

**Outcome boundary**

- 6/6 passes = one bounded repeatability batch for the controlled application-fault seam only;
- partial/failed cycle = valid typed negative/partial evidence, retain valid prefix and first failure;
- never promote this to natural packet-loss/PTO-blackhole, public reachability, reliability-rate or production evidence.

**Commit/push condition:** archive minimal non-sensitive result/evidence summary/hashes, run evidence validators / `git diff --check`, commit/push.

**Continue immediately to B:** yes.

---

### B — Validate, classify and close the repeated-failover batch

**Dependency:** A1 produced a complete batch or typed partial/negative.

1. Validate every row and batch identity/provenance/cleanup invariant.
2. Record batch SHA-256, exact executed commit/binary, actual parameters and cleanup scope.
3. For 6/6, compute only bounded descriptive summaries directly supported (e.g. median/P95 recovery timing + failures=0 when all timing is valid); no reliability rate.
4. On failure, preserve valid prefix and first failure; classify `runtime correctness` vs `orchestration/evidence` vs `environment/path` vs `cleanup`.
5. Runtime correctness -> deterministic regression + correctness repair before another failover run.
6. Orchestration/evidence-only failure -> repair exactly that seam; one materially changed retry later is permitted after gate/CI, but do not block independent C/G/I lanes.

**Validation:** targeted runner/schema tests + `scripts/check.sh` + `git diff --check`.

**Commit/push condition:** coherent evidence/repair commit.

**Continue immediately to C0:** yes if no cross-cutting correctness blocker exists.

---

### C0 — Close the known periodic zero-client orchestration gap locally

The accepted ~5-minute direct periodic row is real, but exact `c4786dc` later invoked zero periodic clients. Before another long VPS run, make the minimum shell-free exact-head-attributed wrapper around the existing real `periodic-server` / `periodic-client` path.

Required properties:

- shell-free argv;
- untracked endpoint plan;
- exact executable SHA-256/size + checkout commit attribution;
- deterministic dry-run proving the real client dispatch path is entered;
- malformed plan/argv fail closed;
- dry-run always `live_evidence=false`;
- truthful resource/cleanup scope; do not repeat the remote-resource misattribution fixed in A0.

Prefer a small wrapper around the already-successful direct/manual mechanism over another generic orchestration framework.

**Validation:** targeted tests + full local gate + `git diff --check`; push and require exact-head green CI before C1.

**Continue immediately to C1:** yes when CI green. While CI is pending, F/G/I are pre-authorized.

---

### C1 — Run one scientifically distinct longer periodic direct-path Session

**Dependency:** C0 exact-head CI green.

Recommended profile:

```text
application phase: about 480 s
interval: 5 s
about 96 records
payload: 32 B/record
concurrency: 1
```

Keep complete setup + application + cleanup below 10 minutes; shorten application phase rather than exceed authorization.

Record exact identity, actual duration, records/bytes, confirmed/missing/duplicate/conflict, truthful latency raw/median/P95 if measured, local/remote resource scope honestly, client/server exits and cleanup. One success remains one bounded sample; one failure is retained with no unchanged retry.

**Commit/push condition:** minimal non-sensitive evidence artifact/summary + hashes.

**Continue immediately to D:** yes.

---

### D — Reconcile release-matrix and resilience status after A1/C1

Update as applicable:

- `docs/era4-e-resilience.md`;
- `docs/status.md`;
- `IMPLEMENTATION_PLAN.md`;
- `ROADMAP.md`.

Rules:

- repeated cross-process failover/recovery becomes positive only to the exact controlled-seam/repeatability extent proved;
- natural UDP degradation/PTO blackhole remains unchecked unless separately observed;
- longer periodic evidence remains bounded to exact duration/sample;
- preserve historical negative artifacts/hashes;
- keep release-evidence item 3 unchecked while declared matrix gaps remain;
- do not change RC/global freeze/production/release flags.

**Validation:** status/plan/release-boundary checks + `scripts/check.sh` + `git diff --check`.

**Continue immediately to E:** yes.

---

### E — Audit the highest-value implementation-blocked release row and unlock only within accepted architecture

**Priority target:** genuine owned-environment NAT/source-endpoint change. Migration-back/key-update/PMTUD are secondary unless repository facts show one is materially closer to live execution.

Classify each relevant lane as:

- `READY_LIVE`;
- `SMALL_LOCAL_UNLOCK`;
- `BLOCKED_IMPLEMENTATION_ARCHITECTURE`;
- `BLOCKED_ENVIRONMENT`;
- `ALREADY_SUFFICIENT_FOR_CURRENT_BOUNDARY`.

If NAT/source-endpoint is `READY_LIVE`, execute one bounded owned-endpoint row immediately. If `SMALL_LOCAL_UNLOCK`, implement only the smallest seam using already-accepted Session/Carrier identity/anti-replay semantics, test/gate/push/CI, then run once. If it requires a new migration identity/rebinding design or production network manipulation, mark blocked and continue. Do not invent architecture merely to fill the matrix.

**Continue immediately to F:** yes unless a cross-cutting blocker appears.

---

### F — Upgrade HY2 failure diagnostics locally; no unchanged `client_exit` retry

Turn the retained HY2 `client_exit` into a discriminating diagnostic contract:

- prove real HY2 stderr/log path feeds a sanitizer/diagnostic bundle;
- retain last successful harness stage;
- classify config/TLS-auth/client-process/network-path only when evidence supports it;
- optional raw-log SHA-256 without committing secrets/endpoints;
- preserve cleanup truthfully;
- deterministic leak tests for secret/endpoint/private-topology strings;
- prepare bounded temporary-port capture metadata if packet direction is required;
- do not loosen TLS/auth/security equivalence.

**Validation:** targeted tests + full local gate + exact-head CI.

**Continue immediately to G:** yes when CI green; H is pre-authorized while waiting.

---

### G — One materially changed HY2/Nekomusume owned-lab attempt

**Dependency:** F exact-head CI green and instrumentation/hypothesis materially differs from exact `3d54585`.

- same owned client/VPS and pinned HY2 v2.9.3;
- same application payload/security/load contract;
- 5 paired samples only if both sides satisfy fair lifecycle contract;
- concurrency 1, fresh unprivileged ports, total <=10 min;
- bounded diagnostics/capture on experiment ports only;
- no production network changes.

Success: retain raw complete pairs, median/P95/failures, exact application bytes/hash and symmetric resource scope; no superiority claim.

Failure: retain typed discriminating diagnostic; no comparative summary and no unchanged retry.

**Continue immediately to H:** yes.

---

### H — Prepare the independent release/security review evidence map

Fully pre-authorized local work for CI waits / blocked VPS lanes. Create a compact reviewer map linking, rather than duplicating:

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
- bounded install/smoke/upgrade/rollback/readiness/cleanup in the dedicated experiment path if current package lineage changed since the accepted evidence;
- low-concurrency leak/pathological-growth resource observation only if a current real-session question needs it;
- no repeated generic microbench/fuzz just to occupy the VPS.

If no new package/resource question exists, mark this slice not applicable and continue; do not manufacture evidence.

**Continue immediately to J:** yes.

---

### J — Release-matrix closure audit and next-phase gate

After consuming all currently READY evidence lanes or retaining them as typed blockers/negatives, audit item 3 from repository facts:

- IPv4 evidence;
- IPv6 environment status;
- controlled vs natural UDP degradation/fallback;
- periodic/longer bounded Session;
- NAT/source-endpoint change;
- repeated cross-process failover/recovery;
- HY2 comparison;
- package/operator/resource evidence;
- exact-head CI/evidence provenance.

Close bounded release-evidence item 3 only if its declared acceptance boundary is genuinely satisfied. Otherwise keep it unchecked and name only real remaining blockers. Do not weaken acceptance criteria to reach RC.

`RELEASE_CANDIDATE=false`, `FREEZE=false`, `PRODUCTION_READY=false`, and `RELEASED=false` remain unchanged. Independent release/security review and a separate RC decision remain later gates.

**Stop after J only if:** remaining work genuinely requires maintainer value judgment, new credentials/server/third-party access, action outside standing authorization, a major architecture choice, or independent external review unavailable to the coding agent. Otherwise continue newly discovered dependency-safe work.

## Queue-wide continuation rules

- Complete coherent slice -> validate -> commit/push -> immediately consume next pre-authorized dependency-safe slice.
- Do not stop after an arbitrary hour, one commit, CI submission, or reviewer interval.
- CI pending blocks only slices explicitly requiring exact-head green CI; use that wait for independent E/F/H work when safe.
- Runtime/tool-budget forced stop is legitimate; record exact checkpoint and resume next wake.
- Any VPS failure remains evidence; preserve it and continue independent READY lanes.
- No unchanged WAN/HY2 reruns.
- Do not spend another full local cycle polishing cross-host tooling after A0 passes its explicit regressions. Return to VPS.

## Completion gates for this rolling queue

- exact `9fd2411` structured endpoint work remains green in CI;
- A0 closes outer-plan integration, verified transport labeling, remote resource/cleanup scope and mandatory endpoint provenance;
- one materially changed six-cycle real warm-failover batch is executed or retained as typed partial/negative;
- repeated-failover evidence is reconciled without claim inflation;
- the periodic zero-client orchestration gap is closed before another long run, and one distinct longer bounded sample is obtained or retained as typed negative;
- status docs reflect only exact evidence;
- NAT/source-endpoint and other live-blocked rows are classified from executable reality;
- HY2 diagnostics materially improve before one future retry;
- independent review preparation exists without being called an audit;
- release-evidence item 3 remains open unless explicit criteria are truly met;
- governance flags remain unchanged.

## Do not expand into

- protocol/wire/Session/Noise redesign merely to obtain nicer evidence;
- weakening authentication, integrity, readiness, provenance or resource limits;
- production firewall/route/qdisc/DNS/proxy/tunnel changes;
- third-party targets or scanning;
- committing endpoint credentials/private topology/local experiment plans;
- reporting local SSH transport CPU/RSS/FD as remote Nekomusume server metrics;
- treating a generic transport argv as proof of SSH/cross-host execution;
- repeated unchanged WAN/HY2 attempts;
- reliability-rate, public-reachability, production-readiness or superiority claims from bounded samples;
- treating fixture-only key-update/PLPMTUD/manager behavior as live evidence;
- speculative FEC/0-RTT/striping/exotic-carrier work without an observed-problem gate.

## Questions requiring maintainer decision

none.

Standing authorization covers A1, C1, any immediately `READY_LIVE` E row, G, and bounded I operations that stay within the existing self-owned TCP/UDP/package experiment contract. No additional maintainer approval is required.