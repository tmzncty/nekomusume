# Nekomusume ChatGPT Handoff

Checked at: 2026-09-03 03:02 Asia/Shanghai
Repository HEAD: `fde80eba5aafa0394e994f40e345bc2f0b381e0f`
Previous reviewed implementation HEAD: `63958278a291613c44f936b27157ae97e0359cfd`
Previous reviewer handoff commit: `3693b6b4375df440a831e7e25e4a1143681126c3`

## What changed

One coding-agent commit is visible after the previous reviewer handoff:

- `fde80eb` — **benchmark harness / evidence validator / regression repair; no Nekomusume production wire, Session or failover semantic change; no new VPS comparison run.** It moves successful sample CPU/RSS/FD attribution to the process-group resource artifact, requires process-group CPU/RSS/FD for successful evidence, switches benchmark binary identities to full SHA-256 values, adds an arithmetic whole-lab budget calculation, emits that derived duration bound, and attempts to replace the fixed Nekomusume server startup sleep with a listener readiness gate.

The exact `fde80eb` GitHub Actions run is green. `stable checks` completed successfully and the nightly 30-second `decode` fuzz smoke completed successfully.

The previous resource-attribution direction is materially improved, but the current commit does **not** yet close the evidence-integrity gate for a paid-VPS paired comparison. Review of the executable harness finds four remaining correctness/evidence defects: the per-sample Nekomusume readiness probe observes the wrong host, persistent HY2 readiness is still not explicitly observed, the ten-minute wall bound is arithmetic metadata rather than an enforced whole-lab deadline and undercounts repeated readiness/control-plane time, and evidence identity/exit/bounds validation is still incomplete.

## Review verdict

**needs repair before VPS pair — `fde80eb` is CI-green and useful, but the next paid-VPS comparison is not yet admissible**

Do not spend the next VPS comparison window on the current harness. A run at this HEAD is likely to fail the Nekomusume readiness stage for the wrong reason because the server is remote while the readiness `ss` command is local; even if that were bypassed, the harness still cannot mechanically prove that the complete lab remained within the standing ten-minute limit.

This is a benchmark/evidence-harness correctness finding, not a production Nekomusume protocol defect. Keep wire/Session/failover semantics unchanged while repairing it.

## Evidence boundaries

- `IMPLEMENTATION_COMPLETE=true` remains the bounded research baseline status.
- `CANONICAL_CORPUS_V1_FROZEN=true` remains corpus-specific only.
- `RELEASE_CANDIDATE=false`, `PRODUCTION_READY=false`, `FREEZE=false`, and `RELEASED=false` remain correct.
- `fde80eb` contains no new HY2/Nekomusume VPS paired samples and no performance conclusion.
- The accepted D064 controlled application-fault warm result and approximately five-minute periodic direct-path result remain valid only at their recorded exact implementation identities. Do not rerun them merely to consume VPS time.
- IPv6 remains environment-blocked unless a genuinely owned end-to-end IPv6 path becomes available.
- `docs/status.md`, `IMPLEMENTATION_PLAN.md`, and `ROADMAP.md` still contain stale wording relative to the accepted positive D064/periodic evidence. Preserve historical negative attempts and repair the drift after the paired-comparison step closes or is honestly blocked.
- Standing authorization still permits the intended self-owned bounded HY2 comparison, but it requires one public/VPS experiment to remain `<= 10 minutes`, with cleanup and no production-network modifications.
- Exact-head CI green is necessary evidence for the harness but does not independently prove remote topology/readiness behavior or whole-lab runtime bounds.

### Reviewer finding R-HY2-04 — Nekomusume readiness is observing the local client, not the remote server

The per-sample Nekomusume server is launched through SSH on the VPS, but the new readiness loop executes:

```text
ss -H -lnt "sport = :<neko-port>"
```

on the local client host. The intended listener exists on the remote VPS, so the current gate can fail closed even when the remote server is correctly ready. The local SSH process liveness check does not repair this locality error.

The readiness observation must execute against the VPS and verify the intended remote bind address/port. A local listener on the same numeric port must not be capable of satisfying remote-server readiness.

### Reviewer finding R-HY2-05 — persistent HY2 server readiness is still not an explicit pre-sample gate

The disposable HY2 server and remote echo server are started in the background before the sample loop, but the current harness does not establish a bounded, explicit remote observation that the temporary HY2 UDP listener is present on the intended remote bind address/port before timed samples begin.

The prior changed-hypothesis attempt failed during QUIC establishment. The next attempt therefore needs an observable `server-listener-ready` fact before entering sample timing so startup race, bind failure and path failure remain distinguishable.

### Reviewer finding R-HY2-06 — the whole-lab budget is calculated but not enforced

`fde80eb` adds:

```text
WHOLE_LAB_SEC = RUNS * 2 * (TIMEOUT + 2)
              + SETUP_SEC + READINESS_SEC + DIAGNOSTIC_SEC + CLEANUP_SEC
```

and rejects arithmetic totals above 600 seconds. This is better metadata, but it is not a fail-closed wall-clock boundary:

- SSH/control-plane commands have no global experiment deadline and can stall beyond the declared allowance;
- setup/tar/copy/remote cleanup operations are not mechanically bounded by the emitted whole-lab value;
- per-sample Nekomusume readiness can consume up to about 10 seconds **for each run**, while the formula allocates only one fixed `READINESS_SEC=10` bucket;
- persistent HY2 readiness is not yet represented by a real bounded observation;
- the emitted `bounds.maximum_duration_ms` records the arithmetic estimate, not a demonstrated/enforced deadline.

Standing authorization is a real execution boundary, not documentation. The harness needs one enforced global wall deadline with cleanup reserve, plus finite per-operation control-plane deadlines, so no admitted parameter/configuration can exceed the experiment limit even when a remote operation hangs.

### Reviewer finding R-HY2-07 — binary identity, exit provenance and result bounds are not fully fail closed

The sample adapter improved resource metrics, but several provenance gaps remain:

1. For Nekomusume, `make_sample()` derives the expected resource identity from the **resource file itself**. That is tautological: an otherwise well-formed stale/wrong Nekomusume resource file with a different SHA can satisfy this check. The validator must bind Nekomusume resource identity to the pinned `contract.nekomusume_binary_sha256` (and HY2 to its pinned full SHA) rather than trust the row's own claimed identity.
2. `make_sample()` checks the process-resource exit code against the observed return code, but does not require the GNU-time sentinel's `exit_code` to match the same return code. A contradictory time sentinel can therefore remain part of a nominally successful sample.
3. `validate_result()` does not currently require/validate the result `bounds` object or prove that `bounds.maximum_duration_ms` matches the enforced harness budget. The previous completion gate explicitly required that disagreement to fail.
4. Complete-result resource membership currently keys mainly on implementation + experiment id; successful client resources should also be validated against the exact pinned binary identity and internally consistent exit/timed-out state.

These are evidence-integrity defects. They do not imply a Nekomusume transport bug, but they must close before comparative CPU/RSS/latency evidence is accepted.

## Work Package — Remote Readiness + Enforced Deadline + Evidence Binding -> Exact-Head CI -> VPS Pair -> Matrix Reconciliation

### Primary A — Make readiness host-correct and observable for both server paths

**Goal**

Close R-HY2-04 and R-HY2-05 so every timed sample begins only after the correct remote experiment-owned server path is observably ready.

**A1. Nekomusume remote readiness**

For each per-sample Nekomusume server:

- launch the server on the VPS as today;
- observe readiness **on the VPS**, not with local `ss`;
- verify the listener is on the exact experiment-owned Nekomusume port and intended remote bind address;
- keep the observation finite and bounded by the global experiment deadline;
- also detect early SSH/server exit so a dead process does not wait the full readiness period;
- if readiness is not proven, emit a typed setup/readiness blocker and do not start client timing.

A local listener on the same port must not satisfy this proof.

**A2. HY2 remote readiness**

After starting the disposable HY2 server and before admitting any timed HY2 sample:

- perform a bounded remote observation that the intended UDP listener exists on the exact temporary HY2 port/bind address;
- distinguish `hy2-server-readiness` from later QUIC/path/TLS/auth failures;
- verify the remote echo-server prerequisite as needed without turning echo startup time into client latency;
- do not infer readiness from a fixed sleep or merely from the SSH launch command returning.

**A3. Deterministic readiness regressions**

Add tests proving:

- a mocked local listener cannot satisfy remote Nekomusume readiness;
- the generated readiness command targets the remote VPS and exact bind/port;
- absent remote listener becomes a typed readiness failure before client timing;
- persistent HY2 listener readiness is required before samples;
- an early remote server exit fails promptly;
- readiness checks cannot escape the configured deadline.

Do not change Nekomusume protocol semantics.

### Follow-up B — Enforce one real whole-lab wall deadline with cleanup reserve

**Dependency:** A green.

Close R-HY2-06 mechanically, not just arithmetically.

Required contract:

1. Establish a monotonic/global experiment deadline **before any remote setup or experiment process starts**.
2. Reserve explicit cleanup time so useful workload cannot consume the full 600 seconds. A conservative maximum work deadline (for example <= 540 seconds with >= 60 seconds reserved for cleanup) is acceptable if implemented truthfully.
3. Every potentially blocking control-plane operation used by this harness — SSH connect/command, remote setup/copy, readiness observation, diagnostics/capture orchestration and cleanup — must have a finite timeout derived from remaining budget or a stricter stage cap.
4. Recompute admitted parameter bounds using the actual repeated readiness/sample lifecycle, not one fixed readiness bucket. Narrow `BENCH_RUNS`/`BENCH_TIMEOUT_SEC` if that yields a simpler truthful safety envelope.
5. If the global deadline is reached, stop admitting new samples, retain completed prefix evidence, execute bounded cleanup, and emit a typed `BLOCKED_HARNESS` artifact. Do not turn a budget expiry into comparative failure statistics.
6. Emit `bounds.maximum_duration_ms` from the actual enforced global bound, and include enough contract metadata to audit the cleanup reserve/stage budget.
7. Do not split an over-limit configuration into multiple nominal experiments to evade standing authorization.

Add deterministic tests with fake/hanging control-plane commands showing an operation cannot run beyond the global/stage deadline and that a just-over-budget profile fails before remote execution.

### Follow-up C — Bind result/resource identity, exit provenance and bounds end-to-end

**Dependency:** B green; may be developed alongside B if files do not conflict.

Close R-HY2-07.

**C1. Exact binary identity**

- pass the expected full Nekomusume/HY2 client binary SHA-256 into sample construction or otherwise bind it to the result contract;
- require every successful client resource row identity to equal `sha256:<contract binary hash>` for its implementation;
- reject stale/wrong identity even if experiment id, metrics and cleanup are otherwise valid;
- retain exact binary identity in blocked artifacts when available without fabricating it before preparation.

**C2. Exit agreement**

For any successful or typed failed sample with available evidence, require agreement among:

```text
observed command return code
GNU-time sentinel exit_code
process-resource exit.code / timed_out
```

A contradiction is an evidence-integrity failure and cannot validate as success. Timeout semantics must remain explicit rather than assuming every code 124 has the same provenance if the sampler distinguishes it.

**C3. Bounds validation**

- make `bounds` required in complete result schema/validation;
- require finite non-negative `maximum_duration_ms` and `application_bytes_max`;
- bind `maximum_duration_ms` to the harness's actual enforced whole-lab deadline;
- require the application-byte bound to be consistent with runs × payload × paired implementations;
- add mutation tests showing missing/altered duration bounds, stale identity and mismatched exit sentinels are rejected.

Do not turn wrapper CPU/RSS into transport metrics; process-group resource evidence remains authoritative for client CPU/RSS/FD.

### Follow-up D — Run the complete local gate and require exact-repair-HEAD CI green

**Dependency:** A-C complete.

Run at minimum:

- benchmark harness deterministic regressions;
- resource sampler regressions;
- result validator regressions;
- `cargo fmt --all -- --check`;
- `cargo check --workspace --locked`;
- `cargo test --workspace --all-targets --locked --no-fail-fast`;
- `cargo clippy --workspace --all-targets --locked -- -D warnings`;
- `bash scripts/check.sh`;
- `git diff --check`.

Push the coherent repair. Wait for the **exact repair HEAD** GitHub `stable checks` and nightly decode fuzz smoke to be green before touching the VPS comparison. The green `fde80eb` run does not attest to the next repair commit.

### Follow-up E — Immediately spend one VPS window on the changed-hypothesis fair HY2/Nekomusume pair

**Dependency:** D exact-head CI green.

The VPS rental window is time-limited, so do not insert unrelated local polish between D and this run.

Recommended bounded profile:

```text
self-owned client <-> owned VPS only
pinned HY2 v2.9.3 full SHA-256
5 paired runs if admitted by the repaired global budget
1200-byte deterministic payload
concurrency 1
fresh unprivileged experiment ports
fresh transport client/session per timed sample
whole-lab enforced wall deadline < 10 minutes with cleanup reserve
```

Use the existing standing authorization only; do not modify production firewall/route/qdisc/DNS/proxy/tunnel or the existing production HY2 service/config.

Because the earlier HY2 attempt timed out during QUIC establishment, collect bounded packet-direction diagnostics around only the temporary HY2 UDP port:

- prove the temporary remote HY2 listener is ready first;
- prove the client targets the verified connect address;
- record whether UDP packets leave the client, arrive at the VPS, elicit replies and whether replies return;
- retain packet counts/timestamps/capture hash or compact redacted metadata; raw pcap need not be committed;
- retain redacted HY2 client/server logs sufficient to distinguish path/NAT/provider failure from TLS/auth/config failure;
- do not change provider/firewall policy merely to force success.

If every required pair succeeds, retain the complete raw set and calculate median/P95/failures only from the complete contract, with exact application payload/hash and process-group client CPU/RSS/FD evidence. Make no superiority/general-WAN claim from one batch.

If any required pair fails, retain one typed changed-hypothesis blocker artifact with no comparative summary and do not rerun unchanged.

Always verify experiment-owned process/listener/temp-path cleanup.

### Follow-up F — Reconcile release matrix, then use at most one next READY VPS-only row

**Dependency:** E succeeds or closes honestly with a retained blocker artifact.

First repair status drift without erasing historical negative attempts:

- `docs/status.md`: add the accepted D064 controlled-fault warm positive row, accepted approximately five-minute periodic direct-path row, the latest fair-harness state and exact HY2 outcome;
- `IMPLEMENTATION_PLAN.md`: remove stale statements that current-lineage D064/periodic evidence is absent; keep bounded release matrix item 3 unchecked while declared rows remain unresolved;
- `ROADMAP.md`: preserve controlled-fault != natural UDP degradation and five-minute sample != general long-lived stability; update HY2 only to the exact outcome.

Then audit current runtime surfaces and choose **at most one** additional high-value VPS-only row that is already executable and truthfully instrumented: genuine owned endpoint/source change, real-session migration-back, real-session key update, or live PMTUD. A fixture/model is not a live runtime seam. If none is READY, record the exact implementation dependency and make that local unlock slice the next package instead of fabricating WAN evidence.

## Completion gates

This package is complete only when all are true:

- Nekomusume readiness is observed on the remote VPS exact bind/port, never satisfied by a local listener;
- persistent HY2 UDP listener readiness is explicitly and finitely observed before samples;
- one enforced global wall deadline plus cleanup reserve keeps every admitted lab execution inside standing authorization;
- control-plane/readiness operations have finite deadlines and cannot escape the whole-lab bound;
- successful sample CPU/RSS/FD remain actual sampler-created client transport process-group metrics;
- Nekomusume and HY2 resource identities are bound to exact pinned binary SHA-256 values rather than self-claims;
- command return code, GNU-time sentinel exit and process-resource exit/timed-out evidence agree or become typed evidence failure;
- complete result bounds are required and match the enforced duration/application budget;
- exact repair-HEAD CI stable + nightly fuzz are green before VPS execution;
- the changed-hypothesis VPS pair yields either a complete fair paired set or a packet-direction-classified blocker artifact;
- cleanup is verified;
- status/plan/roadmap preserve historical negatives and reflect accepted positive rows truthfully;
- `RELEASE_CANDIDATE=false`, `PRODUCTION_READY=false`, `FREEZE=false`, and `RELEASED=false` remain unchanged.

## Fallback

If the repaired VPS pair proves that temporary HY2 UDP packets leave the owned client but do not arrive at the owned VPS, or replies cannot traverse the owned provider/NAT path, preserve that as an environment/path blocker. Do not modify production/provider policy to force success. Move to the next dependency-ready VPS-only evidence row or the local runtime/instrumentation slice that directly unlocks one.

If A-C exposes a Nekomusume production correctness defect rather than benchmark-harness evidence drift, stop comparison work, preserve a minimal reproducer, repair correctness first, run the required parser/fuzz/security gates, and only then return to the paired run.

## Do not expand into

- changing Nekomusume wire/Session/failover semantics to make the benchmark pass;
- production HY2 config/service changes;
- third-party targets, scanning, production firewall/route/DNS/proxy/tunnel/qdisc changes;
- benchmark security downgrade;
- repeated unchanged HY2 timeout attempts;
- experiments exceeding the standing ten-minute/traffic/concurrency limits;
- previous/current interoperability without a prior frozen release;
- 0-RTT, enabled FEC, striping/aggregation or exotic carriers without an observed-problem gate;
- RC/security/production approval from this evidence batch.

## Questions requiring maintainer decision

none.
