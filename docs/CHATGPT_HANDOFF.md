# Nekomusume ChatGPT Handoff

Checked at: 2026-09-03 21:02 Asia/Shanghai
Repository HEAD reviewed: `07545f049790a088bfa655aff4995ab9d6e8fc29`
Previous reviewed coding/evidence HEAD: `1bf848d10e605121c6d11bd70d9c3a9a3b2d86f6`
Previous reviewer handoff commit: `c4786dc8570dc176fc47251f955979dff7de4b58`

## What changed

One coding-agent commit landed after the previous reviewer handoff:

- `07545f0` — **live-evidence provenance binding + extra invariant regressions; no protocol/wire/Session/Noise/failover semantic change and no new VPS evidence yet.** It mechanically binds both real server/client command executables to the exact file named by `NEKO_FAILOVER_BINARY` using same-file identity, binds `NEKO_FAILOVER_GIT_COMMIT` to the actual checkout HEAD containing the live adapter, preserves the recorded executable SHA-256/size as the binary identity, and adds deterministic rejection of decoy server/client executables and wrong checkout commit declarations. It also adds useful non-blocking accounting regressions for wrong uncertain/replayed counts.

Exact `07545f0` GitHub Actions run `33755759414` completed successfully:

- `Rust CI` concluded `success` on exact HEAD `07545f049790a088bfa655aff4995ab9d6e8fc29`;
- the repository stable gate and workflow-required fuzz smoke therefore passed at the exact provenance-repair HEAD.

The previous R-001 evidence-attribution blocker is closed. The adapter now proves that the direct server/client commands execute the same underlying file whose hash/size are recorded, and that the declared commit equals the adapter checkout HEAD. This does **not** prove a reproducible build from that commit by itself; the artifact truthfully retains both commit and executable hash for attribution.

No new correctness/security blocker is visible from this delta. Do not add another generic pre-push/adversarial review layer. The current highest-value work is now real VPS evidence under standing authorization.

## Review verdict

**SAFE_TO_CONTINUE — provenance blocker closed and exact-head CI green; execute the six-cycle real warm-failover batch now, then the distinct longer periodic Session, without another reviewer round-trip**

The project is not blocked. The time-limited VPS is the scarce asset now; further local polish before the authorized evidence runs would reduce evidence value per rental day unless a real deterministic failure appears during execution.

## Evidence boundaries

- `IMPLEMENTATION_COMPLETE=true` remains the bounded research baseline.
- `CANONICAL_CORPUS_V1_FROZEN=true` remains corpus-specific only.
- `RELEASE_CANDIDATE=false`, `PRODUCTION_READY=false`, `FREEZE=false`, and `RELEASED=false` remain correct.
- `9df07d2` / `67569e3` / `1bf848d` / `07545f0` add live evidence collection, strict parsing/invariants and provenance enforcement. They do **not** themselves add real VPS/WAN behavior evidence.
- Exact `07545f0` has independent exact-head CI success. This is repository CI evidence, not an independent security audit or release approval.
- Exact `25e0daa` remains the accepted single controlled application-level UDP reply-cessation warm-fallback VPS row: 3/3 logical records, two uncertain/replayed ranges, duplicate/lost 0, about 434 ms failure-decision-to-first-resumed-data. It is not natural UDP loss/PTO-blackhole evidence.
- The accepted approximately five-minute periodic direct-path row remains one bounded sample, not a production long-lived reliability conclusion.
- Existing generic replacement/open-exchange-close evidence should not be repeated without a distinct research question.
- Live key update, live PMTUD, live migration-back, genuine NAT/source-endpoint change and owned end-to-end IPv6 remain unproven unless current executable surfaces can demonstrate them truthfully.
- Historical HY2 negative evidence remains valid. HY2 stays a diagnostic side track and must not block the main VPS harvest.
- Standing authorization explicitly covers bounded self-owned TCP/UDP failover, Session, benchmark, capture, resource observation and cleanup within <=10 minutes / <=256 MiB / <=32 sessions. No per-run WAN authorization is required.

## Work Package — Six-Cycle VPS Failover -> Longer Periodic Session -> Matrix Reconciliation -> Next VPS Row

### Primary A — Execute the six-cycle repeated real warm-failover VPS batch immediately

**Dependency:** already satisfied. `07545f0` is pushed and exact-head CI is green.

**Goal**

Obtain bounded repeatability evidence from the real `failover-server` / `failover-client` runtime using the now provenance-bound live cycle adapter and the fail-closed six-cycle aggregator.

Do **not** wait for another reviewer handoff and do not add another generic local adversarial review before this run.

**Execution profile**

- self-owned client + self-owned VPS only;
- exact checkout HEAD `07545f049790a088bfa655aff4995ab9d6e8fc29` unless GitHub advances before the run; if it advances, use the exact current coding HEAD and record it truthfully rather than pretending the run used `07545f0`;
- build/stage one exact executable and record SHA-256/size;
- use the repository-tracked `run-live-warm-failover-cycle.py` as the cycle command for `run-repeated-warm-failover.py`;
- one outer aggregator invocation;
- exactly 6 sequential fresh server/client cycles;
- concurrency 1;
- same exact executable identity and stable application parameters across all cycles;
- controlled application-level UDP reply cessation only, matching the accepted semantic class; do not relabel it natural packet loss/PTO blackhole;
- preferred workload remains 3 logical records x 16 B if the current live CLI still supports that exact truthful contract;
- unprivileged ports within the existing 40080-40100 bounded range, with cleanup verified before reuse/change;
- no retry of a failed cycle inside the batch;
- the complete six-cycle experiment including cleanup must stay comfortably below the standing 10-minute bound;
- no production firewall/route/qdisc/DNS/proxy/tunnel/service changes.

**Required evidence**

Retain either all six valid rows or the valid prefix through the first failure. Each successful row must machine-prove, from real runtime output rather than constants:

- canonical negotiation and authenticated Session identity;
- at least one UDP delivery confirmation before the controlled failure decision;
- warm TCP negotiation/authentication/resume validation;
- exactly three authenticated readiness proofs before promotion;
- no TCP application data before promotion where the current evidence contract observes that invariant;
- UNCERTAIN -> replay -> DeliveryAck accounting;
- confirmed / duplicate / lost / conflict counts and bytes;
- failure-decision / first-resumed-data / first-resumed-ack timing when exposed;
- client/server exit state;
- CPU/RSS/FD/socket/process observations at the sampler's truthful scope;
- cleanup status;
- exact commit + executable hash/size provenance now enforced by `07545f0`.

**Outcome boundary**

- 6/6 passes = one bounded repeatability batch for a controlled application-level failure seam only;
- a middle-cycle failure = valid partial/negative evidence; preserve the prefix and do not retry unchanged;
- no reliability-rate, natural-WAN-degradation, public-reachability or production claim from six successes.

If the run exposes a genuine Nekomusume runtime correctness defect, retain the failing evidence and stop only the failover claim path for correctness repair. If it exposes only orchestration/evidence failure, retain it and proceed to B while repairing the exact harness defect separately.

### Follow-up B — Execute one scientifically distinct longer periodic direct-path Session

**Dependency:** A complete or honestly retained as a typed partial/negative. This run is independent of HY2 and normally independent of a failover-only orchestration defect.

Use the existing real `periodic-*` runtime for a longer bounded sample than the accepted approximately five-minute row.

Recommended profile, as a **separate experiment** from A rather than a continuation used to evade the 10-minute limit:

```text
application phase: about 480 s
interval: 5 s
about 96 records
payload: 32 B/record
concurrency: 1
```

Keep total setup + application + cleanup below 10 minutes. Record:

- exact commit/binary SHA-256/size;
- actual setup/application start/end timestamps;
- record/byte counts;
- confirmed/missing/duplicate/conflict counts;
- supported confirmation-latency median/P95 or raw distribution summary;
- CPU/RSS/FD/socket/process observations at truthful scope;
- client/server exits;
- cleanup state.

This is one longer bounded sample only. Do not call it production long-lived stability or infer a reliability rate.

Do not repeat the prior ~5-minute condition unchanged.

### Follow-up C — Reconcile the release-evidence matrix from A/B

**Dependency:** A/B complete or honestly retained as typed negatives.

Update only from actual artifacts and exact execution identities:

- `docs/era4-e-resilience.md` — repeated cross-process failover/recovery may become positive only to the exact extent A supports;
- `docs/status.md` — add exact A/B evidence, hashes and narrow boundaries;
- `IMPLEMENTATION_PLAN.md` / `ROADMAP.md` — keep the full bounded release-evidence matrix incomplete while declared rows remain open;
- preserve the distinction between controlled application-level reply cessation and natural UDP degradation/PTO blackhole;
- preserve all historical negative HY2/failover evidence rather than rewriting history.

Do not change RC/production/global-freeze/release flags.

### Follow-up D — Immediately select the next highest-value VPS-only row from executable reality

**Dependency:** C complete.

Classify each remaining row using current code/evidence, not labels:

```text
NAT/source-endpoint change
live migration-back
live key update
live PMTUD
IPv6
HY2 comparison
```

Use exactly one of:

- `READY_LIVE`
- `BLOCKED_IMPLEMENTATION`
- `BLOCKED_ENVIRONMENT`
- `BLOCKED_DIAGNOSTICS`
- `ALREADY_SUFFICIENT_FOR_CURRENT_BOUNDARY`

For every classification give an exact runtime/evidence reason.

If any row is `READY_LIVE`, execute one bounded row immediately under standing authorization in the same overall package. Prefer VPS-only evidence over local polish.

If none is `READY_LIVE`, implement only the **smallest direct runtime/instrumentation seam** that unlocks the highest-value row, with targeted tests + full gate + normal push/exact-head CI. Do not open an unrelated feature track.

### Follow-up E — HY2 diagnostic side track after the main VPS harvest

**Dependency:** A complete; B/C/D may proceed before E. HY2 must not block them.

`de7ab03` added sanitized diagnostic categories. Before any new paid HY2 retry, prove the live failure path actually feeds HY2 client stderr/log into that diagnostic input and that a blocked artifact retains:

- last successful harness stage;
- sanitized failure category/summary;
- optional raw-log SHA-256 when useful;
- cleanup truthfully;
- no endpoint/credential/private-topology leakage.

A future HY2 retry must have a materially new diagnostic hypothesis/instrumentation variable compared with exact `3d54585`. Do not spend another VPS invocation just to get another generic `client_exit`.

### Optional stretch F — Independent release/security review preparation

**Dependency:** A/C substantially complete; lower priority than VPS-only evidence.

Prepare a compact map of resource/abuse limits, compatibility policy, package install/upgrade/rollback/readiness, canonical corpus/freeze, operator lifecycle/cleanup, and release-matrix positive/negative/blocked rows. This is preparation only, not an independent security review or RC decision.

## Fallback

If A exposes a provenance issue despite `07545f0`, retain the failed collector state and repair only the exact attribution defect. Do not weaken same-file/checkout-HEAD binding.

If A exposes a genuine failover runtime correctness defect:

1. retain the valid prefix/failing cycle;
2. stop further failover claims;
3. repair correctness with a deterministic regression;
4. run parser/fuzz gates only if the defect touches external input/wire behavior;
5. rerun A only after a material implementation change;
6. continue B if the defect is isolated to failover and direct periodic Session remains valid.

If A is blocked only by orchestration/evidence, retain the partial batch, repair that exact harness issue, and continue B meanwhile. Do not freeze the whole project.

If B fails, retain it and continue C/D/E; no unchanged retry.

If any candidate row requires new credentials, another server, third-party access, production-network modification or anything outside standing authorization, stop only that row and continue other READY work.

## Completion gates

- R-001 provenance binding remains closed at `07545f0` or later exact coding HEAD;
- exact current provenance-repair HEAD has green GitHub CI;
- no extra generic pre-push review layer is inserted without a concrete new failure model;
- one six-cycle real cross-process warm-failover batch is executed or retained as a typed partial negative;
- one longer bounded periodic direct-path sample is executed or retained as a typed negative;
- no unchanged WAN/HY2 failure is rerun;
- HY2 remains non-blocking;
- release/status documents reflect only evidence actually obtained;
- at least one next VPS opportunity is either executed truthfully or reduced to a concrete blocked classification + smallest unlock seam;
- `RELEASE_CANDIDATE=false`, `PRODUCTION_READY=false`, `FREEZE=false`, and `RELEASED=false` remain unchanged.

## Do not expand into

- protocol/wire/Session/Noise changes merely for evidence convenience;
- weakening authentication/integrity/readiness/provenance checks;
- another generic independent pre-push review without a concrete new failure model;
- production-network changes, third-party targets or scanning;
- repeated unchanged WAN/HY2 attempts;
- publishing superiority or reliability-rate claims from one bounded batch;
- treating fixture-only key-update/PLPMTUD/manager behavior as live WAN evidence;
- speculative FEC/0-RTT/striping/exotic-carrier work without an observed-problem gate.

## Questions requiring maintainer decision

none.

Standing authorization covers A, B and any immediately READY D row that stays within the existing bounded self-owned TCP/UDP experiment contract. No additional maintainer approval is required for those runs.