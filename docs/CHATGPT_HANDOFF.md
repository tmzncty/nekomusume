# Nekomusume ChatGPT Handoff

Checked at: 2026-09-03 17:05 Asia/Shanghai
Repository HEAD reviewed: `61820255505b03c79a58b45bdce256aedca1b8e8`
Latest coding/evidence HEAD: `12941fabb4726c99d98cd7f225e5b236564c7bb6`
Previous reviewed implementation HEAD: `3d545859e06690c528a717015c9b7023d05ea420`
Previous reviewer handoff commit: `61820255505b03c79a58b45bdce256aedca1b8e8`

## What changed

`12941fa` retained the exact-`3d54585` HY2/Nekomusume owned-lab attempt and reconciled status/evidence. It is evidence/docs only; it does not change Nekomusume wire, Session, Noise or failover semantics.

The retained attempt is a valid negative:

- deterministic payload prepared: 1,200 bytes;
- `nekomusume-1` succeeded;
- `hy2-1` failed with generic `client_exit` and zero application bytes;
- overall status: `BLOCKED_HARNESS` at `hy2-1-failed`;
- no complete pair, no comparative median/P95, no superiority result;
- automatic cleanup stayed fail-closed because `remote_process_groups_reaped=false`; listeners were zero, remote temp/local cleanup succeeded, and later serialized postchecks observed no residue without rewriting the original artifact.

`6182025` correctly changed scheduling policy: HY2 is no longer the sole release-evidence critical path. This review tightens that direction using current code/evidence facts so the coding agent does not spend another cycle merely auditing labels.

Concrete executable-surface findings at current HEAD:

1. **Repeated cross-process warm failover/recovery is READY_LIVE.** The CLI contains a real `failover` runtime. Exact `25e0daa` already proves one self-owned cross-host controlled application-level UDP reply-cessation warm fallback with 3/3 logical records, two uncertain/replayed ranges, duplicate/lost 0 and about 434 ms failure-decision-to-first-resumed-data. `docs/era4-e-resilience.md` explicitly lists repeated cross-process failover/recovery as remaining backlog.
2. **A longer bounded periodic Session is READY_LIVE.** `periodic-*` supports one authenticated TCP Session up to 600 s, but the accepted real VPS evidence is only one approximately five-minute sample. A longer application window with cleanup reserve can answer a distinct bounded resilience question.
3. **Generic repeated TCP/UDP open/exchange/close is already substantially covered.** The repository has an accepted 14/14 alternating replacement-lifecycle sample; do not rerun generic lifecycle merely to create activity.
4. **Live key update is not READY.** CLI capabilities still classify `key-update` as `fixture`, not a live Session runtime command.
5. **Live PMTUD is not READY.** `plpmtud` remains bounded state/test evidence with no live/public probe path recorded in status.
6. **Live migration-back is not yet demonstrated.** Carrier Manager migration-back gates/tests exist, but current evidence does not establish an executable live migration-back VPS row.
7. **IPv6 remains BLOCKED_ENVIRONMENT.** No owned end-to-end IPv6 path is currently demonstrated.
8. **HY2 diagnosis is incomplete.** The benchmark script writes HY2 transport output to temporary logs and wrapper stderr to temporary files, while the retained artifact only carries the generic `client_exit` class. A future paid retry needs materially better sanitized failure attribution first.

## Review verdict

**SAFE_TO_CONTINUE — stop HY2 single-gate thrash; immediately spend the VPS window on repeated failover and longer periodic resilience, then repair HY2 observability as a side track**

The project is not globally blocked. The recent slowdown came from repeatedly serializing the entire work queue behind one HY2 benchmark row.

The coding agent may execute A -> B -> C -> D in one overall work package without waiting for another reviewer handoff, provided each dependency/gate is green and no new correctness/security blocker appears.

## Evidence boundaries

- `IMPLEMENTATION_COMPLETE=true` remains the bounded research baseline.
- `CANONICAL_CORPUS_V1_FROZEN=true` remains corpus-specific only.
- `RELEASE_CANDIDATE=false`, `PRODUCTION_READY=false`, `FREEZE=false`, `RELEASED=false` remain correct.
- Exact `12941fa` is evidence/status work; the paid run itself belongs to exact baseline `3d54585`.
- The incomplete HY2 pair is not comparative evidence.
- Exact `25e0daa` controlled warm fallback is one positive bounded cross-host result, not natural Internet packet-loss/PTO-blackhole evidence.
- Exact `25e0daa` five-minute periodic result is one positive bounded sample, not a production long-lived reliability conclusion.
- Existing 14/14 alternating replacement lifecycle evidence should not be duplicated without a new question.
- Fixture/model capabilities must not be promoted to live WAN evidence.
- Standing authorization already covers the self-owned bounded TCP/UDP work below; no per-run approval is required.

## Work Package — VPS Resilience Harvest + HY2 Diagnostic Side Track

### Primary A — Repeated cross-process warm failover/recovery VPS batch

**Goal**

Produce bounded repeatability evidence for the existing real failover runtime, using the same truthful semantic class as the accepted D064 warm result, without changing protocol semantics and without relabelling a controlled application fault as a natural WAN blackhole.

**Use the existing accepted runtime path**

Each cycle should exercise:

```text
UDP canonical negotiation + Noise authentication
-> at least one logical record confirmed on UDP
-> controlled application-level UDP reply cessation / health-failure seam
-> pre-established warm TCP canonical negotiation + Noise + resume binding
-> three authenticated readiness proofs with live admission
-> atomic promotion
-> replay UNCERTAIN logical ranges on TCP
-> authenticated DeliveryAck confirmation
-> complete Session application accounting
```

Do not use production firewall/route/qdisc changes. Do not substitute netem and then call it real-WAN evidence.

**Batch profile**

- self-owned client + owned VPS only;
- exact current executable HEAD/binary identity recorded;
- one orchestrated lab invocation;
- **6 sequential fresh server/client process cycles**;
- concurrency 1;
- small count/payload comparable to the accepted D064 row;
- fresh or safely reused sequential unprivileged experiment ports within standing authorization;
- total lab wall clock including cleanup comfortably below 10 minutes;
- no retry of a failing cycle inside the same batch.

If no repeated runner exists, add only the smallest orchestrator/result adapter necessary. Do not redesign Session/failover behavior.

**Per-cycle evidence**

Require at least:

- cycle index, parameters, exact commit/binary identity;
- selected version/authenticated resume/readiness completion;
- UDP-confirmed records/bytes before failure;
- UNCERTAIN records/bytes at promotion;
- replayed records/bytes on TCP;
- confirmed/duplicate/lost/conflicting application delivery counts;
- failure-decision and first-resumed-data/ack timestamps when available;
- cycle exit/result;
- CPU/RSS/FD/socket/process metrics when current sampler attribution is truthful;
- cleanup state.

The batch artifact must preserve a valid successful prefix if a later cycle fails. A failed cycle is evidence, not a reason to erase cycles 1..N.

**Regression/gate**

If a new orchestrator/schema is introduced, add deterministic tests for:

- all-six success aggregation;
- middle-cycle failure preserving the preceding rows;
- missing/duplicate/lost/uncertain/replayed fields fail closed;
- cleanup failure remains failure;
- classification cannot claim natural blackhole/PTO;
- result structure contains no required secret/address material.

Run `scripts/check.sh` and `git diff --check`; fuzz only if production network-input/parser/wire behavior changes. If executable harness code changes materially, use exact-head green CI before the real VPS batch.

**Completion definition**

Retain either:

- six successful sequential warm failover/recovery cycles with bounded descriptive aggregate data; or
- a typed partial/negative batch retaining all preceding rows and the first failure.

Do not turn six successes into a general reliability rate or production claim.

### Follow-up B — Longer bounded periodic direct-path Session

**Dependency:** A complete or honestly retained as a typed partial/negative. Independent of HY2.

Use the already-implemented `periodic-*` live runtime for a second, longer bounded real-session sample.

Recommended profile:

```text
one authenticated TCP Session
application phase: about 480 s
interval: 5 s
about 96 records
payload: 32 B/record
concurrency: 1
```

Choose exact setup/application values so the **complete** experiment, including readiness/setup and cleanup, remains below the standing 10-minute limit. Do not use a full 600-second application phase if that consumes the cleanup reserve.

Record exact commit/binary identity, actual timestamps/parameters, records/bytes, confirmed/missing/duplicate/conflict counts, confirmation latency data already supported by the evidence contract, CPU/RSS/FD/socket/process metrics, exits and cleanup.

This answers only whether the current authenticated direct-path Session survives a longer bounded window than the accepted five-minute sample. It is not production long-term stability or a rate estimate.

Do not rerun the old five-minute condition unchanged.

### Follow-up C — HY2 side track: preserve a useful sanitized failure reason before another paid retry

**Dependency:** A complete; B may run before or after C. C must not block A/B.

The current script already creates temporary HY2 client logs and wrapper stderr, but the committed blocked artifact collapses the result to `client_exit`. Repair the evidence contract so a future failure is materially more informative without committing private endpoint/config/auth material.

Add a bounded allowlisted diagnostic classification grounded in observable control-flow/log states. Example classes (adjust to actual implementation evidence):

```text
forward_listener_not_ready
quic_or_path_timeout
tls_or_pin_failure
auth_failure
config_failure
transport_exited_after_ready
application_echo_failure
transport_exit_unclassified
```

At minimum preserve safely:

- transport exit code;
- last successful harness stage;
- sanitized failure class;
- whether forwarding listener became ready;
- whether application echo began;
- optional SHA-256 of the raw ephemeral diagnostic log for provenance;
- packet-direction/capture metadata only when a future run actually gathers it.

Do not commit free-form raw logs containing endpoint addresses, generated auth, cert/key paths or private topology. Prefer an allowlisted parser/classifier plus hash. Unknown failures must remain `transport_exit_unclassified`, not fabricated certainty.

Add deterministic regressions for transport exit before listener readiness, listener-ready then application failure, recognizable TLS/auth/config/timeout text, unknown exit, and secret/address-shaped redaction. Blocked artifacts must still contain no comparative summary.

Run full local gate and exact-head CI after this change.

**Retry rule:** C completion alone does not force an immediate HY2 paid retry. A later retry is permitted only if the new instrumentation gives a genuinely new diagnostic variable/hypothesis. HY2 is no longer the project-wide critical path.

### Follow-up D — Reconcile resilience status and name the next smallest release unlock

**Dependency:** A/B complete; C may be complete or in progress.

Update only from actual evidence:

- `docs/era4-e-resilience.md`: repeated cross-process failover/recovery becomes positive only if A really supports it;
- `docs/status.md`: add A/B exact evidence and boundaries;
- `IMPLEMENTATION_PLAN.md` / `ROADMAP.md`: keep the full release-evidence matrix unchecked unless its declared rows are actually satisfied; controlled application reply cessation remains distinct from natural UDP degradation.

Then record a code-backed blocker matrix for:

```text
NAT/source-endpoint change
live migration-back
live key update
live PMTUD
IPv6
HY2 comparison
```

Use exact statuses such as `READY_LIVE`, `BLOCKED_IMPLEMENTATION`, `BLOCKED_ENVIRONMENT`, `BLOCKED_DIAGNOSTICS`.

Current evidence already indicates key-update is fixture-only, PLPMTUD has no live probe, migration-back is not yet demonstrated live, and IPv6 is environment-blocked. Confirm current code before finalizing those labels.

If one remaining row is unexpectedly `READY_LIVE`, execute one bounded VPS row immediately in the same package. If none is live, choose the **smallest direct implementation/instrumentation seam** that unlocks the highest-value row as the next coding slice. Do not open unrelated experimental features.

### Optional stretch E — Independent release/security review preparation

**Dependency:** A/D complete; lower priority than VPS-only evidence.

Prepare a compact internal pre-review map covering:

- resource/abuse limits and evidence locations;
- version/compatibility policy;
- package install/upgrade/rollback/readiness evidence;
- canonical corpus/freeze references;
- operator lifecycle/cleanup evidence;
- every remaining matrix row labelled positive / negative / blocked implementation / blocked environment / blocked diagnostics.

This is preparation only, not an independent security review or RC approval.

## Fallback

If A reveals a genuine Nekomusume runtime correctness defect rather than an orchestration/evidence defect:

1. retain the first failing cycle/minimal reproducer;
2. stop additional resilience claims;
3. repair correctness first with deterministic tests;
4. run parser/fuzz gates if network-input/wire behavior changes;
5. rerun A only after material implementation change.

If A is blocked only by a harness defect, retain the partial result, repair that exact harness issue, and continue B while the repair proceeds. Do not freeze the whole project.

If B fails, preserve the negative and continue C/D; do not mechanically repeat it.

If future HY2 diagnostics prove a provider/path block requiring production firewall/route changes or anything outside standing authorization, leave HY2 blocked and continue other release work.

## Completion gates

- the project does not return to HY2-only scheduling;
- one repeated cross-process warm failover/recovery VPS batch is executed or retained as a typed partial negative;
- one longer bounded periodic Session sample is executed or retained as a typed negative;
- no unchanged WAN failure is rerun;
- HY2 next-failure evidence can preserve a safe useful reason beyond bare `client_exit` before another retry;
- remaining release rows are each backed by live evidence or an exact implementation/environment/diagnostic blocker with the smallest unlock named;
- all experiments stay within standing authorization and record cleanup;
- `RELEASE_CANDIDATE=false`, `PRODUCTION_READY=false`, `FREEZE=false`, `RELEASED=false` remain unchanged.

## Do not expand into

- another immediate opaque HY2 retry;
- repeating already accepted generic lifecycle evidence without a new question;
- changing Nekomusume wire/Session/Noise semantics for benchmark convenience;
- calling controlled reply cessation natural Internet blackhole/PTO evidence;
- claiming production stability from bounded 5-8 minute samples;
- promoting fixture-only key-update/PLPMTUD/manager behavior to live WAN evidence;
- third-party targets, scanning or production firewall/route/qdisc/DNS/proxy/tunnel changes;
- >10-minute single experiments or mechanically split long soaks;
- enabled FEC, 0-RTT, striping/aggregation or exotic carriers without an observed-problem gate.

## Questions requiring maintainer decision

none.

Standing authorization covers A, B and any genuinely READY bounded self-owned TCP/UDP row discovered in D. HY2 remains a diagnostic side track until its next failure can be attributed materially better than `client_exit`.