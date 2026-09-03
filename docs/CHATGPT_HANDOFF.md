# Nekomusume ChatGPT Handoff

Checked at: 2026-09-03 20:00 Asia/Shanghai
Repository HEAD reviewed: `1bf848d10e605121c6d11bd70d9c3a9a3b2d86f6`
Previous reviewed coding/evidence HEAD: `de7ab0347f6bcca53b921cf32ee4828ed9e9ba88`
Previous reviewer handoff commit: `90c5b4bef9e6cc22c1461cb38b3c8804415d6811`

## What changed

Three coding-agent commits landed after the previous reviewer handoff:

- `9df07d2` — **live repeated-failover cycle adapter + runtime diagnostic evidence; no wire-format or Session architecture change.** It adds `scripts/bench/run-live-warm-failover-cycle.py`, connects the six-cycle runner to the real `failover-server` / `failover-client` path, and adds runtime `failover_timing` / `failover_accounting` diagnostics needed to turn one live cycle into one machine-checkable row.
- `67569e3` — **strict evidence-parser / adapter hardening; no transport semantic change.** It rejects malformed/invalid JSON-looking event rows, duplicate singleton JSON/carrier evidence, role/experiment identity mismatches, inconsistent start parameters, invalid event cardinality/order and contradictory terminal evidence. The deterministic combined attack fixture produces `ADVERSARIAL_REJECTED`.
- `1bf848d` — **additional invariant hardening after the strict follow-up; no transport semantic change.** It extends rejection to JSON list/scalar/malformed-list lookalikes, exact non-negative integer typing (including bool rejection), negative/reversed/internally inconsistent failover timing, contradictory/negative/bool accounting, and preserves the legal zero-latency boundary. It also tightens the pass predicate so timing/accounting validity is not decorative.

The exact current HEAD has independent GitHub Actions run `33751892793`, completed successfully:

- `stable checks` — `bash scripts/check.sh` succeeded;
- `nightly decode fuzz smoke` — pinned cargo-fuzz decode build and 30-second / 8,192-byte smoke succeeded.

This closes the previous parser/semantic-oracle concerns. The coding-agent's extra local adversarial review found useful timing/accounting boundary cases, but a separate pre-push child review is **not** a standing gate going forward. The normal gate remains targeted/adversarial tests -> full local gate -> push -> exact-head GitHub CI. Do not recursively add another review layer without a new concrete failure model.

### New reviewer finding — R-001 HIGH: exact evidence provenance is not yet bound to the executed binary

The live adapter currently computes `binary_sha256` / `binary_bytes` from `NEKO_FAILOVER_BINARY` and records `NEKO_FAILOVER_GIT_COMMIT`, but `NEKO_FAILOVER_SERVER_COMMAND_JSON` and `NEKO_FAILOVER_CLIENT_COMMAND_JSON` are only checked for required CLI tokens. The adapter does **not** mechanically prove that those command arrays actually execute the same binary whose SHA-256 is recorded, and it does not bind the declared commit to the checkout containing the adapter.

That creates a realistic evidence-attribution failure: an accidental old/alternate `neko` executable could be used by the server/client commands while the artifact truthfully hashes a different file and labels the run with the new commit. Six successful cycles would then be valid behavior observations but falsely attributed to the exact HEAD/binary named in the release evidence.

This is an evidence-integrity blocker for the paid six-cycle batch, not a protocol/runtime correctness failure. It is narrow and should be closed once, then the VPS run should proceed immediately.

## Review verdict

**CONTINUE_WITH_REQUIRED_FIXES — strict parser/invariant work accepted and exact-head CI green; close one provenance-binding defect, then execute the six-cycle VPS batch and longer periodic sample without another reviewer round-trip**

The project is not globally blocked. Do not return to HY2-only work and do not start another generic independent adversarial review before push. The highest-value path remains real repeated cross-process warm failover/recovery followed by the distinct longer periodic Session sample.

## Evidence boundaries

- `IMPLEMENTATION_COMPLETE=true` remains the bounded research baseline.
- `CANONICAL_CORPUS_V1_FROZEN=true` remains corpus-specific only.
- `RELEASE_CANDIDATE=false`, `PRODUCTION_READY=false`, `FREEZE=false`, and `RELEASED=false` remain correct.
- `9df07d2` / `67569e3` / `1bf848d` add live evidence collection, diagnostics and fail-closed validation. They do **not** themselves add new VPS/WAN evidence.
- Exact `1bf848d` has green independent stable CI and nightly decode fuzz smoke; this is repository CI evidence, not release/security approval.
- Exact `25e0daa` remains the accepted single controlled application-level UDP reply-cessation warm-fallback VPS row: 3/3 logical records, two uncertain/replayed ranges, duplicate/lost 0, about 434 ms failure-decision-to-first-resumed-data. It is not natural UDP loss/PTO-blackhole evidence.
- The accepted approximately five-minute periodic direct-path row remains one bounded sample, not a production long-lived reliability conclusion.
- Existing generic replacement/open-exchange-close evidence should not be repeated without a distinct question.
- Live key update, live PMTUD, live migration-back, genuine NAT/source-endpoint change and owned end-to-end IPv6 remain unproven unless current executable surfaces can demonstrate them truthfully.
- Historical HY2 negative evidence remains valid; HY2 stays a diagnostic side track and must not block the main VPS harvest.
- Standing authorization covers the bounded self-owned TCP/UDP work below. No per-run WAN authorization is required.

## Work Package — Provenance Binding -> Exact-Head Gate -> Six-Cycle VPS Batch -> Longer Periodic Sample -> Matrix Reconciliation

### Primary A — Bind every live cycle artifact to the binary and checkout actually executed

**Goal**

Make the live adapter's `git_commit`, `binary_sha256` and `binary_bytes` mechanically attributable to the executable used by both real failover processes before spending the rented VPS window.

This is evidence infrastructure only. Do not change wire bytes, Session delivery semantics, Noise, readiness, failover promotion or Carrier Manager behavior.

**Likely files**

- `scripts/bench/run-live-warm-failover-cycle.py`;
- `scripts/bench/run-live-warm-failover-cycle-test.py`;
- `scripts/bench/run-repeated-warm-failover.py` / its tests only if a small provenance field/contract propagation is genuinely needed;
- schema only if a new machine-checked provenance field is required.

#### A1. Bind server/client command execution to `NEKO_FAILOVER_BINARY`

Before starting either process, mechanically prove that both server/client command arrays execute the same file identified by `NEKO_FAILOVER_BINARY`.

Preferred direct-command contract:

- normalize/resolve the executable path used by each command;
- require both to refer to the same underlying file as `NEKO_FAILOVER_BINARY` (an `os.path.samefile`-style check is acceptable so a harmless symlink to the same inode is not falsely rejected);
- reject a command pointing at a different/old/decoy binary even when it contains all required CLI tokens;
- keep wrappers out of the paid path unless the adapter has an explicit, mechanically checkable way to identify the actual child binary. Do not weaken provenance just to support arbitrary wrappers.

The cleanup command is not the experiment binary and does not need this identity rule.

#### A2. Bind the declared commit to the checkout containing the live adapter

The adapter runs from the repository worktree used to prepare the exact-head experiment. Before emitting a row:

- mechanically obtain the repository HEAD for the checkout containing the adapter;
- require it to equal `NEKO_FAILOVER_GIT_COMMIT`;
- fail closed if HEAD cannot be resolved or differs;
- do not infer a commit from a branch name.

The recorded binary SHA-256 remains the actual executable identity. Do not claim that commit equality alone proves reproducible build identity; the evidence row keeps both commit and binary hash for that reason.

A tracked-dirty-tree check is optional rather than required if the adapter can still prove exact HEAD plus exact binary hash. If implemented, do not inspect or commit protected identity files.

#### A3. Regression tests

Add deterministic tests proving at minimum:

- success when server/client commands execute the exact declared binary;
- success through a symlink only when it resolves to the same underlying binary, if symlink support is retained;
- rejection with empty stdout / collector nonzero when server command uses a decoy binary;
- same for client command;
- rejection when `NEKO_FAILOVER_GIT_COMMIT` differs from the adapter checkout HEAD;
- existing malformed/duplicate/identity/order/timing/accounting/adversarial/privacy tests remain green;
- the collector contract remains: exit 0 = one valid evidence row collected, not necessarily experiment success.

The current fixture uses a fake executable path that is not the same as `sys.executable`; update the fixture honestly rather than adding a test-only bypass to the production provenance check.

**Non-blocking parser note**

Do not open another broad parser rewrite. If, while touching the fixture, it is trivial to assert exact per-record sequence identities (`udp_delivery_ack_validated` for the first record, two distinct TCP delivery-ack sequences, readiness sequence `{1,2,3}`), that is useful hardening, but it is not a reason to delay the provenance fix or reopen already accepted parser semantics.

#### A4. Gate and push

Run targeted adapter/runner tests, the normal full local gate and `git diff --check`. Fuzz is required only if production network-input/wire/parser behavior changes; evidence-parser-only tests already have the current fuzz evidence and should not manufacture a new protocol-fuzz claim.

Then **push normally**. Do not insert another generic pre-push child-agent review. Wait only for exact new-HEAD GitHub CI (`stable checks` + nightly decode fuzz smoke if triggered by the workflow). If exact-head CI is green, proceed directly to B without waiting for a new reviewer handoff.

### Follow-up B — Execute the six-cycle repeated warm-failover VPS batch immediately

**Dependency:** A complete, pushed, exact-head CI green.

Use standing authorization directly. No additional administrator/reviewer approval is required.

**Profile**

- self-owned client + self-owned VPS only;
- one outer `run-repeated-warm-failover.py` invocation;
- exactly 6 sequential fresh server/client cycles;
- live cycle command = the repository-tracked provenance-bound adapter from A;
- concurrency 1;
- same exact commit, executable SHA-256/size and parameters across all rows;
- small workload comparable to the accepted D064 semantics (`3` logical records x `16 B` is preferred if still supported truthfully);
- controlled application-level UDP reply cessation only;
- unprivileged experiment ports within the existing CLI/standing-authorization range;
- full batch including cleanup comfortably below 10 minutes;
- no retry of a failing cycle inside the batch;
- no production firewall/route/qdisc/DNS/proxy/tunnel/service modification.

**Retain**

- exact commit/binary provenance now mechanically bound by A;
- six rows or valid prefix through first failure;
- negotiation/auth/resume/readiness evidence;
- UDP-confirmed / uncertain / replayed / confirmed / duplicate / lost / conflict accounting;
- recovery timings;
- client/server exits;
- resource/process/socket evidence at its truthful scope;
- cleanup state;
- compact artifact hashes/index.

If all six pass, report only bounded descriptive repeatability for this controlled seam. Six successes are not a general reliability rate and are not natural-WAN/PTO-blackhole evidence.

If one cycle fails, preserve the prefix exactly. Continue to C unless the failure demonstrates a genuine Nekomusume runtime correctness defect that invalidates further current-runtime evidence.

### Follow-up C — One scientifically distinct longer periodic direct-path Session

**Dependency:** B complete or honestly retained as typed partial/negative. This is independent of HY2.

Use the existing real `periodic-*` runtime for a longer bounded sample than the accepted approximately five-minute row.

Recommended profile, adjusted so setup + application + cleanup remain safely below the standing 10-minute ceiling:

```text
application phase: about 480 s
interval: 5 s
about 96 records
payload: 32 B/record
concurrency: 1
```

Record exact commit/binary identity, actual setup/application timestamps, records/bytes, confirmation/missing/duplicate/conflict counts, supported confirmation-latency statistics, CPU/RSS/FD/socket/process evidence, exits and cleanup.

This proves one longer bounded sample only. Do not call it production long-lived stability. Do not repeat the old five-minute condition unchanged.

### Follow-up D — Reconcile the release-evidence matrix and immediately choose the next VPS opportunity

**Dependency:** B/C complete.

Update only from actual evidence:

- `docs/era4-e-resilience.md` — repeated cross-process failover/recovery becomes positive only to the exact extent B supports;
- `docs/status.md` — add exact B/C evidence and boundaries;
- `IMPLEMENTATION_PLAN.md` / `ROADMAP.md` — keep the full release matrix incomplete while declared rows remain open; keep controlled application reply cessation distinct from natural degradation/PTO blackhole;
- preserve all historical negative HY2/failover evidence.

Then classify each remaining row from executable reality:

```text
NAT/source-endpoint change
live migration-back
live key update
live PMTUD
IPv6
HY2 comparison
```

Use `READY_LIVE`, `BLOCKED_IMPLEMENTATION`, `BLOCKED_ENVIRONMENT`, `BLOCKED_DIAGNOSTICS`, or `ALREADY_SUFFICIENT_FOR_CURRENT_BOUNDARY` with an exact code/evidence reason.

If any row is `READY_LIVE`, execute one bounded VPS row immediately under standing authorization in the same overall package. If none is ready, implement only the smallest direct runtime/instrumentation seam that unlocks the highest-value row. Do not open an unrelated feature track.

### Follow-up E — HY2 diagnostic side track after the main VPS harvest

**Dependency:** B complete; C/D may proceed before E. HY2 must not block B/C/D.

`de7ab03` already added useful sanitized diagnostic categories. Before another paid HY2 retry, prove the live harness actually feeds the generated HY2 client stderr/log into that diagnostic path and that a blocked artifact retains the last successful stage plus sanitized category/summary (and optional raw-log SHA-256). Require a materially new diagnosis variable compared with exact `3d54585`; do not spend another VPS invocation just to obtain another generic `client_exit`.

### Optional stretch F — Independent release/security review preparation

**Dependency:** B/D substantially complete; lower priority than VPS-only evidence.

Prepare a compact reviewer map of resource/abuse limits, compatibility policy, package rollback/readiness, canonical corpus/freeze, operator lifecycle/cleanup, and release-matrix positive/negative/blocked rows. This is preparation only, not an independent security review and not an RC decision.

## Fallback

If A reveals that the real paid command genuinely requires a wrapper, do not bypass executable identity. Add the smallest explicit wrapper-to-child provenance contract that mechanically proves the child binary path/hash, test it, and continue B after exact-head CI.

If B exposes a genuine runtime correctness defect:

1. retain the failing cycle and valid prefix;
2. stop further failover claims;
3. repair correctness first with deterministic regression;
4. run parser/fuzz gates if the defect touches external input/wire behavior;
5. rerun B only after a material implementation change;
6. continue C if the defect is isolated to failover and does not invalidate direct periodic Session evidence.

If B is blocked only by orchestration/evidence, retain the partial batch, repair that exact issue and continue C while the repair proceeds. Do not freeze the whole project.

If C fails, retain it and continue D/E; no unchanged retry.

If any row requires new credentials, another server, third-party access, production network modification or anything outside standing authorization, stop only that row and continue other READY work.

## Completion gates

- the current strict parser/invariant work remains accepted and green;
- live server/client command execution is mechanically bound to the artifact's recorded binary identity;
- declared commit is mechanically bound to the adapter checkout HEAD;
- the provenance repair is pushed and exact-head GitHub CI is green;
- no extra generic pre-push review layer is inserted without a new concrete failure model;
- one six-cycle real cross-process warm-failover batch is executed or retained as a typed partial negative;
- one longer bounded periodic direct-path sample is executed or retained as a typed negative;
- no unchanged WAN failure is rerun;
- HY2 remains non-blocking;
- release/status documents reflect only evidence actually obtained;
- `RELEASE_CANDIDATE=false`, `PRODUCTION_READY=false`, `FREEZE=false`, and `RELEASED=false` remain unchanged.

## Do not expand into

- protocol/wire/Session/Noise changes merely for evidence convenience;
- weakening authentication/integrity/readiness/provenance checks;
- another generic independent pre-push review without a concrete new failure model;
- production network changes, third-party targets or scanning;
- repeated unchanged WAN/HY2 attempts;
- publishing superiority or reliability-rate claims from one bounded batch;
- treating fixture-only key-update/PLPMTUD/manager behavior as live WAN evidence;
- speculative FEC/0-RTT/striping/exotic-carrier work without an observed-problem gate.

## Questions requiring maintainer decision

none.

Standing authorization covers B, C and any immediately READY D follow-up that remains within the existing self-owned bounded TCP/UDP experiment contract. A maintainer decision is required only for a genuinely new credential/server/third-party requirement, production-network change, out-of-authorization experiment or major architecture choice.