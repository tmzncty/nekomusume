# Nekomusume ChatGPT Handoff

Checked at: 2026-09-03 22:00 Asia/Shanghai
Repository HEAD reviewed: `a6003ccdbfd2fb995e96c39bd6ae53c2e9f2ad5b`
Previous reviewed coding/evidence HEAD: `07545f049790a088bfa655aff4995ab9d6e8fc29`
Previous reviewer handoff commit: `b7ff87bfbe2592dac9564c34c232e06a2733ea27`

## What changed

Two coding-agent commits landed after the previous reviewer handoff.

- `52c96bd` — **status/evidence reconciliation only; no protocol/runtime behavior change and no new VPS behavior evidence.** It reconciles `IMPLEMENTATION_PLAN.md`, `ROADMAP.md`, `docs/status.md`, and `docs/era4-e-resilience.md` with the retained exact-head negatives: exact `1bf848d` produced a zero-cycle repeated-failover collector negative, exact `07545f0` exited before entering the Python runner, and exact `c4786dc` started one periodic orchestrator but invoked zero clients and transferred zero application bytes. It correctly classifies those as orchestration/pre-application negatives rather than transport failures, preserves separate post-exit cleanup observations, and keeps the bounded release-evidence matrix open.
- `a6003cc` — **repeated-failover orchestration repair; no wire/Session/Noise/failover semantic change and no new VPS evidence yet.** It adds `scripts/bench/run-repeated-warm-failover-command.py` plus deterministic tests, converts the failed outer launch boundary to shell-free argv construction, validates a bounded local plan, dispatches the existing real cycle adapter through the Python runner, proves six-cycle dispatch in deterministic preflight, fails closed on malformed plan/argv input, and wires the new regression into `scripts/check.sh`. `run-repeated-warm-failover.py` now accepts the explicit `--` command separator without reparsing shell text.

Exact `a6003cc` GitHub Actions run `33762011758` completed successfully:

- `stable checks` — success;
- `nightly decode fuzz smoke` — success.

The exact `07545f0` live attempt failed at the outer shell command boundary before the Python batch runner. `a6003cc` materially changes that failure hypothesis: command construction and dispatch are now argv-native and deterministic. This is sufficient to justify **one** changed-hypothesis real VPS repeated-failover batch under standing authorization. The local preflight is not live-network evidence and does not prove SSH/remote launch success; that is now exactly what the next VPS run must test.

No new protocol correctness or security blocker is visible in this delta. The current bottleneck is evidence acquisition, not architecture. The previous handoff was also too short for the demonstrated agent cadence; this handoff therefore becomes a rolling multi-hour queue rather than a one-ticket stop-and-wait package.

## Review verdict

**SAFE_TO_CONTINUE — shell-boundary diagnostic blocker closed at exact green HEAD; repeated real warm failover is now `READY_LIVE`; consume the rolling queue continuously without waiting after each slice**

The external coding agent should not insert another generic local review or harness-polish round before the changed-hypothesis VPS batch. Complete a coherent slice, run its required gates, commit/push, then move directly to the next explicitly pre-authorized slice whenever its dependency is satisfied.

## Evidence boundaries

- `IMPLEMENTATION_COMPLETE=true` remains the bounded research baseline.
- `CANONICAL_CORPUS_V1_FROZEN=true` remains corpus-specific only.
- `RELEASE_CANDIDATE=false`, `PRODUCTION_READY=false`, `FREEZE=false`, and `RELEASED=false` remain correct.
- `52c96bd` is documentation/status reconciliation, not runtime evidence.
- `a6003cc` is local orchestration/test infrastructure, not VPS/WAN behavior evidence.
- Exact `a6003cc` has independent exact-head stable CI and decode fuzz smoke success. This is CI evidence, not a security audit, release approval, or WAN evidence.
- Exact `25e0daa` remains the accepted single controlled application-level UDP reply-cessation warm-fallback row: 3/3 logical records, 48 application bytes, two uncertain/replayed records, duplicate/lost 0, approximately 434 ms failure-decision-to-first-resumed-data. It is not natural UDP degradation/PTO-blackhole evidence.
- Exact `25e0daa` also remains one accepted approximately five-minute direct periodic Session: 60 x 32 B, 60/60 confirmed, no missing/duplicate/conflict. It is one bounded sample, not a reliability rate or production long-lived conclusion.
- Exact `1bf848d`, `07545f0`, and `c4786dc` negatives are orchestration/pre-application evidence, not proof of failover/periodic runtime failure.
- The repeated-failover row changes from `BLOCKED_DIAGNOSTICS` at the `52c96bd` checkpoint to **`READY_LIVE` at `a6003cc`** because the known shell/argv boundary is repaired and exact-head CI is green.
- NAT/source-endpoint change, live migration-back, live key update, and live PMTUD remain implementation-blocked until current executable reality proves otherwise. IPv6 remains environment-blocked.
- HY2 remains diagnostics-blocked after the latest `hy2-1 client_exit`; no valid paired comparison exists.
- A local plan for the new shell-free runner may contain owned endpoint/user/address material. Keep it local/secret and out of Git; repository evidence must retain only non-sensitive metadata/hashes/classifications.
- Standing authorization covers bounded self-owned TCP/UDP failover, periodic Session, diagnostics, resource observation, bounded capture, benchmark and cleanup within <=10 minutes / <=256 MiB / <=32 sessions. No per-run WAN approval is required.

## Rolling Work Queue

The queue below is deliberately larger than one reviewer interval. **Do not stop after A, B, or one commit merely because the reviewer has not run again.** Each slice states whether the next slice is pre-authorized.

### A — Run one changed-hypothesis six-cycle real warm-failover VPS batch now

**Status:** `READY_LIVE`.

**Why now**

The known exact-`07545f0` launch failure occurred before the Python runner because of a shell command boundary. `a6003cc` removes that boundary with shell-free argv construction, deterministic runner entry and exact-head green CI. Further local polishing would have lower evidence value than the real run.

**Use**

- `scripts/bench/run-repeated-warm-failover-command.py`;
- `scripts/bench/run-repeated-warm-failover.py`;
- existing `scripts/bench/run-live-warm-failover-cycle.py`;
- one local untracked plan containing actual owned endpoint details.

**Execution profile**

- self-owned client + self-owned VPS only;
- exact checkout HEAD `a6003ccdbfd2fb995e96c39bd6ae53c2e9f2ad5b` unless a later coding HEAD is intentionally used before launch; whichever is executed must be recorded exactly;
- build/stage one exact executable; retain SHA-256 and size;
- exactly 6 sequential fresh server/client cycles, concurrency 1;
- controlled application-level UDP reply cessation only;
- preferred workload: 3 logical records x 16 B if the current live CLI contract remains unchanged;
- unprivileged experiment ports in the existing allowed range;
- no retry of a failed cycle inside the same batch;
- set the outer batch budget conservatively (prefer <=540 s) so total setup + six cycles + cleanup remains below the standing 10-minute limit;
- no production firewall/route/qdisc/DNS/proxy/tunnel/service changes;
- do not commit the plan file or endpoint/private topology material.

**Required evidence**

For each collected successful row, prove from real output rather than constants:

- canonical negotiation + authenticated Session identity;
- UDP confirmation before controlled failure;
- warm TCP negotiation/authentication/resume validation;
- exactly three authenticated readiness proofs before promotion;
- no TCP application data before promotion where the current contract observes it;
- `UNCERTAIN -> replay -> DeliveryAck` accounting;
- confirmed/uncertain/replayed/duplicate/lost/conflict records and bytes;
- recovery timing when exposed;
- client/server exit state;
- CPU/RSS/FD/socket/process observations at their truthful scope;
- cleanup status;
- exact commit + binary SHA-256/size provenance.

**Outcome boundary**

- 6/6 = one bounded repeatability batch for the controlled application-fault seam only;
- partial/failed cycle = valid negative/partial evidence; retain it and do not rerun unchanged;
- never promote this to natural packet-loss/PTO-blackhole, public reachability, reliability-rate or production evidence.

**Commit/push condition:** archive only the minimal non-sensitive result/evidence summary/hashes needed by repository policy, update no status claim beyond the actual outcome, run applicable evidence validators / `git diff --check`, then commit and push.

**Continue immediately to B:** yes. Do not wait for reviewer acknowledgment.

---

### B — Validate, classify and close the repeated-failover batch without evidence inflation

**Dependency:** A produced either a complete batch or a typed partial/negative artifact.

**Goal**

Make A reviewable and decide whether repeated cross-process failover/recovery is positive, negative, or still orchestration-blocked at the exact executed identity.

**Required work**

1. Validate the batch against the tracked schema/runner invariants.
2. Record batch SHA-256, exact executed commit/binary identity, actual parameters and cleanup boundary.
3. If 6/6 passes, compute only bounded descriptive batch summaries that are directly supported (e.g. median/P95 recovery timing across the six rows if all required timing values are valid, plus failures=0). Do not infer a reliability rate.
4. If a cycle fails:
   - preserve the valid prefix and first failure;
   - distinguish `runtime correctness`, `orchestration/evidence`, `environment/path`, and `cleanup` failure;
   - no unchanged retry.
5. If failure is **runtime correctness**, add a deterministic regression and repair correctness before another failover run.
6. If failure is only **orchestration/evidence**, repair exactly that defect; one later changed-hypothesis retry is allowed after local gate + exact-head CI, but do not block independent periodic/HY2-diagnostic work meanwhile.

**Files/concepts:** repeated-failover scripts/schema, a small evidence artifact/doc if needed, `docs/era4-e-resilience.md` only after the actual outcome is known.

**Validation:** targeted runner/validator tests, `scripts/check.sh`, `git diff --check`; protocol fuzz only if production external-input/wire behavior changed.

**Commit/push condition:** one coherent evidence/repair commit, normal push.

**Continue immediately to C:** yes if no cross-cutting correctness blocker exists. If A found a failover-only correctness bug, C may still proceed because it is a direct-path Session lane, provided the defect does not affect generic Session/crypto correctness.

---

### C — Close the known periodic orchestration gap locally before spending another long VPS window

**Why**

The accepted ~5-minute direct periodic row is real, but the later exact-`c4786dc` periodic orchestrator invoked zero clients and transferred zero application bytes. A longer periodic run should not be launched through an already-known broken orchestration path. Fix only the minimum command/provenance seam first.

**Goal**

Create or repair the smallest shell-free, exact-head-attributed way to invoke the existing real `periodic-server` / `periodic-client` path for one owned-lab run. Reuse existing runtime semantics; do not invent a second periodic protocol/harness framework.

**Required properties**

- shell-free argv for the experiment launch path;
- local/secret endpoint plan remains untracked;
- exact executable SHA-256/size and checkout commit attribution;
- finite setup/application/ACK budgets already supported by the runtime;
- deterministic dry-run/preflight proving server/client argv dispatch and that the client is actually invoked;
- malformed plan/argv fail closed;
- no fake `live_evidence=true` from dry-run;
- truthful resource/cleanup fields; unknown remains null/unknown rather than fabricated pass.

If the already-successful direct/manual mechanism can satisfy these properties with a very small wrapper, prefer that over a new generic orchestration subsystem.

**Validation:** targeted preflight/adversarial tests + full local gate + `git diff --check`; push and obtain exact-head green CI before D. While CI is pending, continue independent F/G preparation if safe.

**Commit/push condition:** minimal local unlock commit, not a protocol change.

**Continue immediately to D:** yes once exact-head CI for the substantive orchestration change is green.

---

### D — Run one scientifically distinct longer periodic direct-path Session

**Dependency:** C exact-head CI green.

**Goal**

Use the rental window for one longer bounded direct Session observation distinct from the accepted ~5-minute sample.

**Recommended profile**

```text
application phase: about 480 s
interval: 5 s
about 96 records
payload: 32 B/record
concurrency: 1
```

Keep complete setup + application + cleanup below 10 minutes. If overhead cannot fit honestly, shorten the application phase rather than exceed authorization.

**Record**

- exact commit/binary SHA-256/size;
- start/end timestamps and actual duration;
- record/byte counts;
- confirmed/missing/duplicate/conflict counts;
- confirmation-latency raw/median/P95 only if the runtime measures that semantic truthfully;
- CPU/RSS/FD/socket/process observations at truthful scope;
- client/server exits;
- cleanup state.

One success remains one bounded sample, not production long-lived stability or a reliability rate. One failure is retained; no unchanged retry.

**Commit/push condition:** minimal non-sensitive evidence artifact/summary + hashes, normal push.

**Continue immediately to E:** yes.

---

### E — Reconcile release-matrix and resilience status after A/D

**Dependency:** A/B complete; D complete or typed negative. If C/D are blocked, reconcile that blocker truthfully rather than waiting indefinitely.

**Goal**

Make repository status reflect the new exact evidence without rewriting history.

**Update as applicable**

- `docs/era4-e-resilience.md`;
- `docs/status.md`;
- `IMPLEMENTATION_PLAN.md`;
- `ROADMAP.md`.

**Rules**

- repeated cross-process failover/recovery may become positive only to the exact controlled-seam/repeatability extent A proves;
- natural UDP degradation/PTO blackhole remains unchecked unless separately observed;
- longer periodic evidence remains bounded to its exact duration/sample;
- preserve all historical negative rows and hashes;
- keep item 3 release-evidence matrix unchecked while declared missing rows remain open;
- do not change RC/global freeze/production/release flags.

**Validation:** status/plan/release-boundary checks + full `scripts/check.sh` + `git diff --check`.

**Commit/push condition:** one reconciliation commit.

**Continue immediately to F:** yes.

---

### F — Audit the highest-value implementation-blocked release row and unlock only if existing architecture already supports it

**Priority target:** genuine owned-environment NAT/source-endpoint change, because it is explicitly named in the release-evidence matrix. Migration-back/key-update/PMTUD are secondary unless repository facts show one is much closer to live execution.

**Goal**

Determine whether a truthful authenticated source-endpoint/path-change run can be produced using existing Session/Carrier identity and anti-replay semantics **without changing core architecture or production networking**.

**Inspect**

- current `neko-cli` live socket paths;
- `neko-carrier` Path/manager/migration state;
- D064 and migration ADRs;
- Session identity/resume guards;
- current tests and observability events.

**Classification**

For NAT/source endpoint and, secondarily, migration-back/key-update/PMTUD, assign one of:

- `READY_LIVE`;
- `SMALL_LOCAL_UNLOCK`;
- `BLOCKED_IMPLEMENTATION_ARCHITECTURE`;
- `BLOCKED_ENVIRONMENT`;
- `ALREADY_SUFFICIENT_FOR_CURRENT_BOUNDARY`.

**Execution rule**

- If NAT/source-endpoint change is `READY_LIVE`, execute one bounded owned-endpoint row immediately under standing authorization.
- If it is `SMALL_LOCAL_UNLOCK`, implement only the smallest runtime/instrumentation seam that uses already-accepted Session/Carrier semantics, add deterministic tests, full gate, push/CI, then execute one bounded row.
- If it requires deciding new migration identity/rebinding semantics or weakening Session/Carrier boundaries, **do not invent the architecture**. Mark that lane `BLOCKED_IMPLEMENTATION_ARCHITECTURE` and continue to G.

No production route/firewall/qdisc/NAT manipulation is authorized merely to manufacture the row.

**Commit/push condition:** classification note plus implementation/evidence only if safely within existing architecture.

**Continue immediately to G:** yes unless a genuine cross-cutting correctness/security blocker appears.

---

### G — Upgrade HY2 failure diagnostics locally; do not rerun the same `client_exit`

**Goal**

Turn the current nondiscriminating HY2 `client_exit` into a diagnostic contract capable of justifying one materially changed future attempt.

**Required work**

- prove the real HY2 client stderr/log path feeds the sanitizer/diagnostic bundle rather than being discarded;
- retain last successful harness stage;
- produce a bounded sanitized category/summary that distinguishes at least configuration/TLS-auth/client-process/network-path classes when evidence supports it;
- optional raw-log SHA-256 may be retained without committing secrets/endpoints;
- preserve cleanup truthfully;
- add deterministic tests that secret/endpoint/private topology strings cannot leak through the diagnostic summary;
- do not loosen TLS/auth/security equivalence for benchmark convenience.

If packet-direction evidence is necessary, prepare a bounded capture metadata path limited to the temporary HY2 UDP port; raw pcap need not be committed.

**Validation:** targeted harness/diagnostic tests + full local gate + exact-head CI.

**Commit/push condition:** one diagnostics-only commit.

**Continue immediately to H:** yes once exact-head CI is green. While CI is pending, I is pre-authorized.

---

### H — One materially changed HY2/Nekomusume owned-lab attempt after diagnostic closure

**Dependency:** G exact-head CI green and the failure hypothesis/instrumentation is materially different from exact `3d54585`.

**Goal**

Obtain either the first valid paired batch or a discriminating typed failure that makes another unchanged retry unnecessary.

**Bounded profile**

- same owned client/VPS, pinned HY2 v2.9.3 artifact already recorded;
- same application payload/security/load contract;
- 5 paired samples only if both sides satisfy the fair lifecycle contract;
- concurrency 1;
- fresh unprivileged ports;
- total experiment <=10 min;
- bounded capture/diagnostic metadata on temporary ports if needed;
- no production network changes.

**If successful:** retain raw paired rows; compute median/P95/failures only for complete valid pairs; report exact application bytes/hash and symmetric resource scope; no superiority claim.

**If failed:** retain typed diagnostic with packet/log direction evidence where available; no comparative summary; no unchanged retry.

**Commit/push condition:** minimal evidence artifact/summary, normal push.

**Continue immediately to I:** yes.

---

### I — Prepare the independent release/security review evidence map

**Priority:** lower than READY VPS evidence, but fully pre-authorized local work and useful while waiting on CI or when VPS lanes are blocked.

Create a compact reviewer-oriented map, not a security approval, covering:

- resource and abuse limits / pre-auth admission;
- crypto/Noise selection and replay/nonce boundaries;
- version compatibility policy and canonical corpus freeze scope;
- Session delivery vs packet ACK separation;
- package install/upgrade/rollback and binary provenance;
- operator lifecycle/readiness/shutdown/cleanup;
- positive/negative/blocked release-matrix rows with exact commit/artifact identities;
- HY2 comparison methodology and current absence/presence of valid pairs;
- outstanding environment/implementation blockers.

Prefer links to existing authoritative docs/artifacts over duplicating claims. Explicitly state that this is **review preparation**, not an independent security review.

**Validation:** link/status consistency checks + `git diff --check`.

**Commit/push condition:** one review-preparation doc commit.

**Continue immediately to J:** yes.

---

### J — Release-matrix closure audit and next-phase gate

**Dependency:** consume all currently READY evidence lanes above or retain them as typed blockers/negatives.

**Goal**

Decide from repository facts whether bounded release-evidence item 3 is actually closable, or enumerate the exact remaining blockers without inventing work.

Check at minimum:

- IPv4 evidence;
- IPv6 environment status;
- controlled vs natural UDP degradation/fallback evidence;
- periodic/longer bounded Session evidence;
- NAT/source-endpoint change status;
- repeated cross-process failover/recovery;
- HY2 comparison status;
- package/operator/resource evidence;
- exact-head CI/evidence provenance.

If item 3 truly satisfies its declared acceptance boundary, update it only with exact evidence and move the rolling queue to independent release/security review. If not, keep it unchecked and name only the real remaining blockers. Do not weaken the acceptance criteria just to reach RC.

**RC boundary:** do not set `RELEASE_CANDIDATE=true`, `FREEZE=true`, `PRODUCTION_READY=true`, or `RELEASED=true`. Independent release/security review and a separate RC decision remain later gates.

**Commit/push condition:** closure/blocker reconciliation only if facts changed.

**Stop after J only if:** the remaining work genuinely requires maintainer value judgment, new credentials/server/third-party access, action outside standing authorization, a major architecture choice, or independent external review that the coding agent cannot perform itself. Otherwise continue any newly discovered dependency-safe work recorded by the repository plan.

## Queue-wide continuation rules

- Complete coherent slice -> validate -> commit/push -> immediately consume the next pre-authorized dependency-safe slice.
- Do **not** stop after an arbitrary hour, one commit, CI submission, or reviewer interval.
- CI pending only blocks a slice explicitly requiring exact-head green CI; use that wait for independent local slices such as F/G/I when safe.
- A runtime/tool-budget forced stop is legitimate; record exact checkpoint and resume next wake.
- If a VPS lane fails, preserve the negative and continue independent READY lanes.
- No unchanged WAN/HY2 reruns.

## Completion gates for this rolling queue

- exact `a6003cc` shell-free repeated-command repair remains green in CI;
- one materially changed six-cycle real warm-failover batch is executed or retained as a typed partial/negative;
- repeated-failover evidence is reconciled without claim inflation;
- the known periodic zero-client orchestration boundary is closed before another long run, and one distinct longer bounded sample is obtained or retained as a typed negative;
- release/status docs reflect only exact evidence obtained;
- NAT/source-endpoint and other live-blocked rows are classified from executable reality; only architecture-safe smallest seams are implemented automatically;
- HY2 diagnostics materially improve before one future retry; no repeated generic `client_exit`;
- independent review preparation is assembled without calling it an audit;
- release-evidence item 3 remains open unless its explicit criteria are truly met;
- `RELEASE_CANDIDATE=false`, `PRODUCTION_READY=false`, `FREEZE=false`, `RELEASED=false` remain unchanged.

## Do not expand into

- protocol/wire/Session/Noise redesign merely to obtain nicer evidence;
- weakening authentication, integrity, readiness, provenance or resource limits;
- production firewall/route/qdisc/DNS/proxy/tunnel changes;
- third-party targets or scanning;
- committing endpoint credentials/private topology/local experiment plans;
- repeated unchanged WAN/HY2 attempts;
- reliability-rate, public-reachability, production-readiness or superiority claims from bounded samples;
- treating fixture-only key-update/PLPMTUD/manager behavior as live evidence;
- speculative FEC/0-RTT/striping/exotic-carrier work without an observed-problem gate.

## Questions requiring maintainer decision

none.

Standing authorization covers A, D, any immediately `READY_LIVE` F row, and H provided each stays within the existing bounded self-owned TCP/UDP experiment contract. No additional maintainer approval is required for those runs.
