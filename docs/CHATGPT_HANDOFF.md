# Nekomusume ChatGPT Handoff

Checked at: 2026-09-03 17:59 Asia/Shanghai
Repository HEAD reviewed: `de7ab0347f6bcca53b921cf32ee4828ed9e9ba88`
Previous reviewed coding/evidence HEAD: `12941fabb4726c99d98cd7f225e5b236564c7bb6`
Previous reviewer handoff commit: `6887686df21d0624a922253c5268a28cd7c6291c`

## What changed

Two coding-agent commits landed after the previous reviewer handoff.

- `d4181b3` — **local repeated-failover evidence infrastructure; no production runtime change and no new VPS evidence.** It adds `schema/repeated-warm-failover.v1.json`, a fail-closed six-cycle aggregator/validator in `scripts/bench/run-repeated-warm-failover.py`, deterministic tests for six-pass aggregation, middle-cycle prefix retention, identity stability, cleanup failure, semantic classification, required fields and privacy, and wires the test into `scripts/check.sh`.
- `de7ab03` — **HY2 benchmark diagnostic/evidence-contract repair; no Nekomusume transport semantic change and no new paid comparison.** It makes validate-only mode side-effect free, preserves cleanup booleans truthfully, retains bounded sanitized client diagnostics for future failed samples, adds allowlisted diagnostic classes/redaction regressions, and keeps comparative results fail-closed. This correctly advances the HY2 diagnostic side track without putting HY2 back on the project-wide critical path.

Exact `de7ab03` GitHub Actions run `33740629316` completed successfully on push. This is independent repository CI evidence for the current code, not a security review or release approval.

The repeated warm-failover Primary is **not yet real VPS evidence**. The new aggregator deliberately delegates each cycle to an external `cycle command`, and no repository-tracked live cycle adapter is currently visible that converts the existing real `neko failover-server` / `neko failover-client` runtime into the required per-cycle JSON contract. The repository therefore has a good batch evidence validator but not yet a reproducible end-to-end command that another agent can invoke six times without reconstructing the accepted D064 experiment by hand.

One smaller evidence-contract ambiguity should be closed in the same local slice before spending the VPS window: the aggregator currently accepts a cycle-command process return code of `0` even when the row reports a non-zero `result.client_exit_code`; non-zero wrapper exits must match the recorded client exit. This can be a valid design if wrapper exit `0` explicitly means “evidence collection completed” while experiment success/failure lives only in the JSON row, but that meaning is not stated in the contract and is not regression-tested. Do not silently rely on it.

## Review verdict

**CONTINUE_WITH_REQUIRED_FIXES — repeated-failover batch infrastructure is accepted, HY2 diagnostics are accepted, but close the concrete live cycle-adapter/evidence-exit contract before the six-cycle VPS run**

The project is not blocked. This is a narrow evidence-orchestration closure, not a protocol/runtime correctness failure.

Do not return to HY2-only work. The highest-value path remains repeated real cross-process warm failover/recovery, followed by a distinct longer periodic Session sample. HY2 may continue as a side track only after those VPS-priority tasks are moving.

## Evidence boundaries

- `IMPLEMENTATION_COMPLETE=true` remains the bounded research baseline.
- `CANONICAL_CORPUS_V1_FROZEN=true` remains corpus-specific only.
- `RELEASE_CANDIDATE=false`, `PRODUCTION_READY=false`, `FREEZE=false`, and `RELEASED=false` remain correct.
- `d4181b3` proves deterministic aggregation/validation behavior only. It does **not** prove one, much less six, real failover cycles.
- `de7ab03` proves local diagnostic/evidence-contract behavior only. It does **not** repair or explain the historical exact-`3d54585` HY2 `client_exit`, and it does not create a comparison result.
- Exact `25e0daa` remains the accepted single controlled application-level UDP reply-cessation warm-fallback VPS row: 3/3 logical records, two uncertain/replayed ranges, duplicate/lost 0, about 434 ms failure-decision-to-first-resumed-data. It is not natural UDP loss/PTO-blackhole evidence.
- The accepted approximately five-minute periodic direct-path row remains one bounded sample, not a production long-lived reliability conclusion.
- Existing generic replacement/open-exchange-close evidence should not be repeated without a distinct research question.
- Live key update, live PMTUD, live migration-back, genuine NAT/source-endpoint change and owned end-to-end IPv6 remain unproven unless current executable surfaces can demonstrate them truthfully.
- Standing authorization covers the bounded self-owned TCP/UDP work below. No per-run WAN authorization is required.

## Work Package — Live Cycle Adapter Closure -> Six-Cycle VPS Batch -> Longer Periodic Sample -> Matrix Reconciliation

### Primary A — Turn the six-cycle validator into a reproducible live failover experiment path

**Goal**

Close the remaining gap between `run-repeated-warm-failover.py` and the already-existing real failover runtime so the next command can produce six fresh, truthful VPS cycles without hand-reconstructing JSON or protocol state.

This is the smallest direct unlock for the highest-value READY VPS row. Do not redesign Session, Noise, failover semantics, readiness semantics or wire format.

**Likely files / existing surfaces**

- `scripts/bench/run-repeated-warm-failover.py`;
- `scripts/bench/run-repeated-warm-failover-test.py`;
- `schema/repeated-warm-failover.v1.json` only if the adapter contract genuinely requires an evidence field;
- add one small repository-tracked live cycle adapter under `scripts/bench/`, preferably Python or shell consistent with the existing experiment harnesses;
- reuse the current `crates/neko-cli/src/main.rs` `failover-server` / `failover-client` path and the exact accepted D064 evidence procedure rather than inventing a second failover implementation;
- reuse current process-resource sampler / bounded cleanup helpers when they fit.

#### A1. Add a concrete live cycle adapter

The adapter must perform exactly one fresh controlled warm-failover cycle and print exactly one per-cycle JSON object accepted by `validate_cycle`.

For each invocation, it must:

1. consume `NEKO_FAILOVER_CYCLE_INDEX` and derive only experiment-local identifiers/temporary paths from it;
2. start a fresh bounded `failover-server` and fresh `failover-client` process pair using the same current binary identity;
3. exercise the same **controlled application-level UDP reply-cessation** semantic class as the accepted D064 row;
4. obtain at least one authenticated logical delivery confirmation on UDP before the failure decision;
5. prove warm TCP completed canonical negotiation, fresh Noise authentication, resume validation and exactly three authenticated readiness proofs before promotion;
6. prove no TCP application data was sent before promotion;
7. preserve the real UNCERTAIN -> replay -> DeliveryAck accounting from runtime output, rather than calculating success from expected constants;
8. capture the actual failure-decision / first-resumed-data / first-resumed-ack timestamps when the current diagnostic stream exposes them;
9. collect truthful client/server exit status, bounded CPU/RSS/FD/socket/process evidence where the current sampler attribution is valid, and cleanup observations;
10. print only the schema-safe result to stdout. Logs/diagnostics go elsewhere and must not leak endpoint addresses, identities, credentials or payload contents into committed artifacts.

The adapter may use local secret/config environment supplied by the coding environment. Do not commit target addresses, identity material or credentials.

A cycle that cannot prove all required semantics must print a truthful failed row or fail in a way the outer aggregator retains as `invalid_cycle_evidence`; it must never synthesize a passing row.

#### A2. Make cycle-command exit semantics explicit and fail-closed

Resolve the current ambiguity around this code path:

```text
cycle command process exit
vs
row.result.client_exit_code / server_exit_code
vs
row.result.status
```

Two acceptable contracts are:

- **collector contract:** wrapper exit `0` means only “one valid evidence row was collected”; experiment success/failure is solely `row.result.status`, while underlying client/server exit codes are fields inside the row; or
- **experiment contract:** wrapper exit mirrors a designated underlying experiment exit and must match the row.

Choose one explicitly, document it in the runner docstring/schema note, and regression-test it. Do not leave “zero happens to bypass mismatch checking” as an implicit semantic.

Whichever contract is selected, reject contradictory states such as a wrapper claiming successful collection but emitting malformed/incomplete evidence, or a non-zero wrapper exit inconsistent with the documented row semantics.

#### A3. Add deterministic adapter/runner regressions

Without VPS access, cover at minimum:

- six valid adapter rows aggregate to one six-cycle pass;
- a middle-cycle live-adapter fixture failure preserves the valid prefix and stops further cycles;
- child/wrapper exit semantics follow the chosen explicit contract;
- a mismatch/contradiction is rejected;
- missing readiness proof, resume proof, UDP pre-failure confirmation, timing, accounting or cleanup fails closed;
- TCP application-before-promotion evidence, if observable in the adapter contract, cannot pass;
- identity/parameters remain stable across all six cycles;
- cleanup failure cannot be promoted to a pass;
- result contract requires no endpoint/secret material;
- controlled reply cessation cannot be relabelled natural blackhole/PTO.

#### A4. Full gate and exact-head CI

Run at minimum:

- targeted repeated-warm-failover tests;
- adapter-specific tests;
- `cargo fmt --all -- --check`;
- `cargo check --workspace --locked`;
- `cargo test --workspace --all-targets --locked --no-fail-fast`;
- `cargo clippy --workspace --all-targets --locked -- -D warnings`;
- `bash scripts/check.sh`;
- `git diff --check`.

No fuzz claim is required unless production parser/network-input/wire behavior changes.

Push the repair and require exact-head GitHub CI green before the paid VPS sequence. Do not spend the VPS window on an unreviewable local-only adapter revision.

**Completion definition**

A repository-tracked command can be pointed at the owned lab and produce one truthful cycle row from the current real failover runtime; the outer six-cycle runner can invoke it sequentially; exit semantics are explicit/tested; full local gate passes; exact-head CI is green.

### Follow-up B — Execute the six-cycle repeated warm-failover VPS batch immediately

**Dependency:** A complete and exact-head CI green.

Do not wait for another reviewer handoff. Use standing authorization directly.

**Profile**

- self-owned client + self-owned VPS only;
- one outer orchestrator invocation;
- exactly 6 sequential fresh server/client cycles;
- concurrency 1;
- same exact binary SHA-256/size and stable parameters across all rows;
- small workload comparable to the accepted D064 semantics (3 logical records x 16 B is acceptable if still supported truthfully);
- controlled application-level UDP reply cessation only;
- unprivileged experiment ports;
- complete batch including cleanup comfortably below 10 minutes;
- no retry of a failed cycle inside this batch;
- no production firewall/route/qdisc/DNS/proxy/tunnel/service modification.

**Retain**

- exact commit/binary identity and actual parameters;
- six rows or valid prefix through first failure;
- per-cycle negotiation/auth/resume/readiness evidence;
- UDP-confirmed / uncertain / replayed / confirmed / duplicate / lost / conflict accounting;
- recovery timings;
- client/server exits;
- resource/process/socket evidence at truthful scope;
- cleanup state;
- compact artifact hashes/index.

If all six pass, report only bounded descriptive repeatability for this controlled seam. Do not turn six successes into a general reliability rate, natural-WAN failover conclusion or production claim.

If a cycle fails, retain the partial batch exactly and proceed to C unless the failure is a genuine Nekomusume correctness blocker that would make further current-runtime evidence invalid.

### Follow-up C — One scientifically distinct longer periodic direct-path Session

**Dependency:** B complete or honestly retained as a typed partial/negative. Independent of HY2.

Use the existing real `periodic-*` runtime for a longer bounded sample than the accepted ~5-minute row.

Recommended target, adjusted so setup + application + cleanup remain safely below the standing 10-minute ceiling:

```text
application phase: about 480 s
interval: 5 s
about 96 records
payload: 32 B/record
concurrency: 1
```

Record exact commit/binary identity, actual setup/application timestamps, records/bytes, confirmation/missing/duplicate/conflict counts, supported confirmation-latency statistics, CPU/RSS/FD/socket/process evidence, exits and cleanup.

This proves one longer bounded sample only. Do not call it production long-lived stability.

Do not repeat the old five-minute condition unchanged.

### Follow-up D — Reconcile release-evidence state from B/C and select the next smallest unlock

**Dependency:** B/C complete.

Update only from actual evidence:

- `docs/era4-e-resilience.md` — repeated cross-process failover/recovery may become positive only to the exact extent B supports;
- `docs/status.md` — add exact B/C evidence/boundaries;
- `IMPLEMENTATION_PLAN.md` and `ROADMAP.md` — preserve the full matrix as incomplete while declared rows remain open; keep controlled application reply cessation distinct from natural degradation/PTO blackhole;
- preserve all historical negative HY2 and failover evidence.

Then re-audit these rows against current executable code, not labels:

```text
NAT/source-endpoint change
live migration-back
live key update
live PMTUD
IPv6
HY2 comparison
```

Use one of `READY_LIVE`, `BLOCKED_IMPLEMENTATION`, `BLOCKED_ENVIRONMENT`, `BLOCKED_DIAGNOSTICS`, or `ALREADY_SUFFICIENT_FOR_CURRENT_BOUNDARY` with an exact evidence/code reason.

If a row is `READY_LIVE`, execute one bounded VPS row immediately under standing authorization. If none is ready, name and implement only the **smallest direct runtime/instrumentation seam** that unlocks the highest-value row. Do not open an unrelated feature track.

### Follow-up E — HY2 diagnostic side track after the main VPS harvest

**Dependency:** A/B complete; C may run before E. HY2 must not block B/C/D.

`de7ab03` is accepted as useful local progress: future non-zero HY2 client samples can retain bounded sanitized diagnostic categories such as TLS/auth/config/path/readiness, and validate-only mode no longer creates evidence files.

Before another paid HY2 retry:

1. confirm the live harness actually passes the generated HY2 client stderr/log path into the new `--client-diagnostics` input on the failure path;
2. retain the last successful harness stage plus sanitized category/summary and optional raw-log SHA-256;
3. when a run is made, gather bounded packet-direction metadata only if it answers a concrete hypothesis;
4. require a materially new hypothesis/instrumentation variable compared with exact `3d54585`;
5. preserve an unknown case as unclassified rather than guessing.

Do **not** spend another VPS invocation merely to see whether the generic `client_exit` happens again. A future paid retry is justified only when the new diagnostic path is known to survive into the retained blocked artifact and can distinguish the next failure.

### Optional stretch F — Independent release/security review preparation

**Dependency:** B/D complete; lower priority than VPS-only evidence.

Prepare a compact pre-review map of resource/abuse limits, compatibility policy, package rollback/readiness, canonical corpus/freeze, operator lifecycle/cleanup, and release-matrix positive/negative/blocked rows. This is preparation only, not an independent review or RC decision.

## Fallback

If A reveals that producing the required per-cycle evidence needs a real Nekomusume runtime semantic change rather than a thin adapter/instrumentation bridge:

- do not fake the row;
- record the exact missing observable/runtime seam as `BLOCKED_IMPLEMENTATION`;
- continue C (longer periodic sample) immediately because it is independent and VPS-ready;
- then implement the smallest failover instrumentation seam if it does not change architecture/security boundaries.

If B exposes a genuine runtime correctness defect:

1. retain the failing cycle and valid prefix;
2. stop further failover claims;
3. repair correctness first with deterministic regression;
4. run fuzz/parser gates if relevant;
5. rerun B only after material implementation change.

If B is blocked by orchestration/evidence only, retain the partial batch, repair the exact harness issue, and continue C while the repair proceeds. Do not freeze the whole project.

If C fails, retain it and continue D/E; no unchanged retry.

If any row requires new credentials, another server, third-party access, production network modification or anything outside standing authorization, stop only that row and continue other READY work.

## Completion gates

- `d4181b3` batch validator is connected to one reproducible live cycle adapter rather than hand-authored JSON;
- wrapper/experiment exit semantics are explicit and regression-tested;
- exact adapter HEAD passes local gate and independent GitHub CI;
- one six-cycle real cross-process warm-failover batch is executed or retained as a typed partial negative;
- one longer bounded periodic direct-path sample is executed or retained as a typed negative;
- no unchanged WAN failure is rerun;
- HY2 stays non-blocking and its next paid retry, if any, can retain materially better diagnostics than bare `client_exit`;
- remaining matrix rows are backed by live evidence or exact implementation/environment/diagnostic blockers with the smallest unlock named;
- all experiments stay within standing authorization and preserve cleanup evidence;
- `RELEASE_CANDIDATE=false`, `PRODUCTION_READY=false`, `FREEZE=false`, and `RELEASED=false` remain unchanged.

## Do not expand into

- another immediate opaque HY2 retry;
- hand-authoring six JSON rows without a repository-tracked real adapter;
- changing Session/Noise/wire/failover semantics merely to satisfy the evidence harness;
- calling controlled application reply cessation natural Internet loss/PTO evidence;
- claiming production stability from bounded repeatability or one 8-minute sample;
- promoting fixture-only key-update/PLPMTUD/manager behavior to live WAN evidence;
- repeated generic lifecycle runs with no new question;
- third-party targets, scanning or production firewall/route/qdisc/DNS/proxy/tunnel changes;
- >10-minute single experiments or mechanically split long soaks;
- enabled FEC, 0-RTT, striping/aggregation or exotic carriers without an observed-problem gate.

## Questions requiring maintainer decision

none.

Standing authorization covers B, C and any genuinely READY bounded self-owned TCP/UDP row discovered in D. No maintainer interaction is required for the local adapter/evidence closure in A.