# Nekomusume ChatGPT Handoff

Checked at: 2026-09-03 02:01 Asia/Shanghai
Repository HEAD: `63958278a291613c44f936b27157ae97e0359cfd`
Previous reviewed implementation HEAD: `ee371694eaa9aff782ee5e5b7e933144447b4d46`
Previous reviewer handoff commit: `d3bb1dc0e74c2c38fa8760d2f52240e8c86fa4b4`

## What changed

One coding-agent commit is visible after the previous reviewer handoff:

- `6395827` — **benchmark harness / validator / resource-sampler / regression / documentation repair; no Nekomusume production wire, Session or failover semantic change; no new VPS run.** It separates server bind authority from client connect authority, uses the SSH-reachable endpoint for both clients and client-side route/MTU lookup, pins the disposable HY2 self-signed certificate with its exact SHA-256 fingerprint, creates a fresh HY2 transport client for each timed sample, moves each timed client command beneath the process-group sampler, strengthens complete-result client-resource membership checks, and documents the fresh-client fair-pair contract.

The exact `6395827` GitHub Actions run is green. `stable checks` completed successfully and the nightly 30-second `decode` fuzz smoke also completed successfully.

The previous handoff's topology/TLS/lifecycle direction is therefore materially implemented. Two evidence-contract defects remain before another VPS comparison should be spent, plus one readiness/diagnostic hardening item that should be closed in the same batch.

## Review verdict

**continue with required benchmark-evidence repair — `6395827` is useful and CI-green, but do not run the paired VPS comparison yet**

The next VPS run is still high priority because the rental window is time-limited, but the current result path can misattribute client CPU/RSS and the harness parameter space can exceed the standing ten-minute wall-clock authorization while claiming a smaller hard-coded bound. Fix those locally first, then immediately run the changed-hypothesis paired experiment with bounded path diagnostics.

This is a benchmark/evidence integrity gate, not a production protocol correctness finding. Do not change Nekomusume wire/Session semantics to fix it.

## Evidence boundaries

- `IMPLEMENTATION_COMPLETE=true` remains the bounded research baseline status.
- `CANONICAL_CORPUS_V1_FROZEN=true` remains corpus-specific only.
- `RELEASE_CANDIDATE=false`, `PRODUCTION_READY=false`, `FREEZE=false`, and `RELEASED=false` remain correct.
- `6395827` contains no new real WAN/VPS comparative samples and no performance conclusion.
- The accepted D064 controlled-fault warm evidence and approximately five-minute periodic direct-path evidence remain valid at their original exact implementation identities; do not rerun them merely to consume VPS time.
- IPv6 remains environment-blocked unless a genuinely owned end-to-end IPv6 path becomes available.
- `docs/status.md` and `IMPLEMENTATION_PLAN.md` still contain stale text saying current-lineage D064 warm / periodic VPS evidence is absent. Preserve that drift until the benchmark-contract/VPS step below is closed, then reconcile it without deleting historical negative attempts.
- Official Hysteria 2 documentation supports the `insecure: true` + exact `pinSHA256` pattern for a self-signed certificate; `6395827` now follows that upstream security pattern. The fair lifecycle/measurement design remains a Nekomusume project decision.

### Reviewer finding R-HY2-01 — successful sample CPU/RSS are not yet transport-group metrics

`run_client` now correctly places the actual client lifecycle under `process-resource-sampler.py`, and that sampler records process-group CPU/RSS/FD/socket evidence. However the executable result adapter does not actually use all of those process-group values for the sample row:

- GNU `time` wraps the **Python resource sampler**;
- `make_sample()` takes `cpu_user_seconds`, `cpu_system_seconds`, and `rss_kib` from that outer GNU-time sentinel;
- only `fd_count` is taken from the process-resource JSON;
- the separate resource artifact does contain transport process-group CPU/RSS/FD, but `validate_resource()` currently checks mainly cleanup, identity/role/experiment/scope membership and does not require complete non-null group CPU/RSS/FD for a successful comparative sample;
- the validator regression's synthetic complete `client_resource()` rows omit CPU/RSS/FD entirely and still validate.

Therefore a result can claim a successful fair sample while its sample-level CPU/RSS fields describe sampler/wrapper resource use rather than the actual transport group. This violates the current owned-lab contract that per-sample CPU/RSS/FD cover the actual transport client process/group.

### Reviewer finding R-HY2-02 — whole-lab duration bound is not fail-closed

Standing authorization limits a single public/VPS experiment to at most 10 minutes. The owned-lab harness currently accepts:

```text
BENCH_RUNS = 3..10
BENCH_TIMEOUT_SEC = 1..30
```

and each pair contains two client regions bounded approximately by `TIMEOUT + 2` seconds. At the allowed maximum this alone can exceed 640 seconds before setup, SSH operations, server readiness, diagnostics and cleanup. Even the default five-run / 30-second configuration has a client-region worst-case above 300 seconds once wrapper margins are included, while final result assembly currently hard-codes:

```text
bounds.maximum_duration_ms = 300000
```

That field is not an enforced whole-lab wall bound. The harness must not admit a configuration outside standing authorization and must not emit a bound smaller than the actually enforced budget.

### Reviewer finding R-HY2-03 — timed samples should start only after observable server readiness

The prior HY2 attempt failed during QUIC establishment. `6395827` fixes the address hypothesis, but the current orchestrator still relies on background startup timing and a fixed short sleep for the per-run Nekomusume server rather than a bounded, observed readiness gate before entering a timed client region. Before the next paid-VPS run, make server/listener readiness explicit enough to distinguish startup race from path failure. This is benchmark harness instrumentation, not a release capability promotion.

## Work Package — Evidence Attribution Repair -> Exact-Head CI -> VPS Fair Pair -> Matrix Reconciliation

### Primary A — Make fair-pair resource and wall-clock evidence fail closed

**Goal**

Close R-HY2-01 and R-HY2-02 without changing Nekomusume protocol semantics, so every successful sample has attributable transport-client resource evidence and every admitted owned-lab run is mechanically inside standing authorization.

#### A1. Use process-group resource evidence for comparative CPU/RSS/FD

For successful owned-lab sample rows:

- use the process-resource artifact as the authoritative source for client transport `cpu_user_seconds`, `cpu_system_seconds`, `rss_kib`, and `fd_count`;
- use GNU time only for the elapsed wall/lifecycle timing if that remains the selected clock source;
- require non-null valid process-group CPU, RSS and FD evidence for a successful comparative sample;
- keep the resource row's `sampling.scope == "sampler-created process group"` requirement;
- require the resource experiment id / implementation / role / application-byte contract to match the sample;
- bind resource binary identity to the corresponding pinned implementation identity strongly enough that a stale/wrong resource file cannot satisfy a new sample;
- require the resource process exit/timed-out state and the GNU-time sentinel exit to agree with the observed command return code. Any disagreement is typed evidence failure, not a success row.

If wrapper/sampler CPU/RSS are still useful diagnostics, keep them separately and label them as wrapper metrics; do not put them into fields documented as transport-client resource evidence.

Update the complete-result validator so a synthetic resource object missing group CPU/RSS/FD cannot pass merely because cleanup is complete.

#### A2. Enforce a truthful whole-lab wall budget

Before any remote setup or experiment process starts:

- compute an explicit worst-case whole-lab budget from run count, per-sample timeout, paired lifecycle count and finite setup/cleanup/readiness/diagnostic allowances;
- reject parameter combinations whose enforced worst-case can exceed 600 seconds;
- leave meaningful margin for cleanup rather than treating 600 seconds as useful workload time;
- give remote SSH/readiness/capture steps finite deadlines so one stalled control-plane operation cannot silently escape the experiment bound;
- make the emitted `bounds.maximum_duration_ms` equal the actual enforced whole-lab bound (or a truthful parameter-derived upper bound), not a hard-coded 300000 ms unrelated to admitted parameters;
- keep application traffic and concurrency inside the existing standing limits.

It is acceptable to narrow the owned-lab comparison profile (for example, a smaller maximum run count) if that is the simplest truthful contract. Do not split one over-limit benchmark into repeated nominally separate runs to evade the standing limit.

#### A3. Tests for the two evidence contracts

Add deterministic regressions proving at least:

- complete result rejects a client resource row with missing CPU, RSS or FD evidence;
- sample CPU/RSS/FD come from the process-group resource artifact rather than arbitrary GNU-time wrapper values;
- GNU-time exit / resource exit / command return-code mismatch is rejected or typed failed;
- wrong/stale resource identity is rejected;
- maximum accepted owned-lab parameters are demonstrably within the enforced whole-lab budget;
- one just-over-budget parameter combination fails before remote execution;
- result `maximum_duration_ms` cannot disagree with the enforced configured bound.

Do not weaken the existing payload/hash, cleanup, partial-prefix retention or client-resource membership gates.

### Follow-up B — Make pre-sample server readiness observable and rerun the full exact-head gate

**Dependency:** A green.

Close R-HY2-03 before spending the VPS window:

1. For the persistent HY2 server, require a bounded observation that the experiment-owned UDP listener is actually present on the intended remote bind address/port before admitting timed HY2 samples.
2. For each Nekomusume per-sample server, replace the fixed startup sleep as the sole gate with a bounded readiness/listener observation before starting that sample's timer. The readiness check itself is not part of client latency.
3. Fail closed with a typed setup/readiness stage if the listener never becomes observable; do not reinterpret it as transport performance.
4. Preserve non-wildcard bind, connect-vs-bind separation, exact certificate pin, fresh HY2 client per sample, process-group cleanup and negative-result retention.
5. Run the full local gate and push the repair.
6. Wait for exact repair-HEAD GitHub `stable checks` and nightly decode fuzz smoke to be green. Do not use an older green commit as authorization for the new VPS result.

### Follow-up C — Run one changed-hypothesis owned-lab HY2/Nekomusume paired batch with bounded path diagnostics

**Dependency:** B complete and exact repair HEAD CI green.

Use the VPS rental window immediately after the local gate closes.

Recommended profile remains:

```text
self-owned client <-> owned VPS only
HY2 v2.9.3 pinned SHA-256 already recorded
5 paired runs
1200-byte deterministic payload
concurrency 1
fresh unprivileged experiment ports
whole-lab enforced wall budget < 10 minutes
```

Do not modify production firewall, route, qdisc, DNS, proxy/tunnel or the existing production HY2 service/config.

Because the previous changed attempt reached `server up and running` but the client QUIC path timed out, add bounded packet-direction evidence around only the new temporary HY2 UDP port:

- record that the remote HY2 listener exists on the intended bind address/port;
- record that the client targets the verified connect address/port;
- bounded client/VPS capture or equivalent metadata should determine whether client UDP packets leave, arrive at the VPS, elicit replies, and whether replies return;
- record packet counts/timestamps/capture hashes or compact redacted metadata; raw pcap need not be committed;
- retain HY2 client/server diagnostic logs with secrets and unnecessary addresses redacted;
- distinguish path/NAT/provider failure from TLS/auth/config failure rather than merely recording another timeout.

If every required paired sample succeeds:

- preserve the complete raw paired set;
- calculate median/P95/failure counts only under the complete-success contract;
- report exact application bytes/hash and the repaired transport-process CPU/RSS/FD evidence;
- keep `wire_bytes=null` unless capture accounting is trustworthy enough to support it;
- make no superiority/general-WAN claim from one self-owned batch.

If any required pair fails, preserve one typed blocked/diagnostic artifact with no comparative summary. Do not rerun unchanged.

Always verify cleanup of experiment-owned local/remote process groups, listeners and temporary paths.

### Follow-up D — Reconcile release-matrix status after C

**Dependency:** C succeeds or closes honestly with a retained changed-hypothesis blocker artifact.

Repair the current status/evidence drift without erasing historical negatives:

1. `docs/status.md`
   - record the accepted `25e0daa`-lineage positive D064 controlled-fault warm result and its exact boundary;
   - record the accepted approximately five-minute periodic direct-path result as one bounded sample, not general long-lived proof;
   - replace the stale local-only HY2 note with the latest fair-harness repair and actual C outcome;
   - do not promote controlled application fault to natural UDP/PTO degradation.
2. `IMPLEMENTATION_PLAN.md`
   - remove stale language saying current-lineage warm/periodic evidence is absent;
   - keep bounded release matrix item 3 unchecked while declared HY2/IPv6/NAT/endpoint-change or other required rows remain unresolved;
   - record comparison evidence as positive only if C produced a complete valid paired set.
3. `ROADMAP.md`
   - preserve the distinction between controlled cross-host failover evidence and natural WAN degradation;
   - preserve the distinction between one five-minute periodic sample and general long-connection stability;
   - update HY2 only to the exact C outcome.

Governance flags remain unchanged.

### Follow-up E — Spend one additional VPS opportunity only if a genuinely missing row is already executable

**Dependency:** D complete.

Audit current runtime surfaces before selecting the next VPS row. Do not repeat generic TCP/UDP baselines, the accepted D064 warm controlled-fault run, the accepted periodic sample, or the HY2 attempt without a changed hypothesis.

Candidate rows include NAT/source-endpoint change, real-session migration-back, real-session key update or live PMTUD only when a current executable CLI/runtime seam and truthful instrumentation already exist. A model/fixture alone is not a live-runtime seam. If none is READY, record the exact implementation dependency and move to local work that directly unlocks the highest-value row rather than fabricating WAN evidence.

## Completion gates

This package is complete only when:

- successful paired samples source CPU/RSS/FD from the actual sampler-created client transport process group;
- missing/stale/mismatched resource evidence cannot validate as success;
- timing/resource/return-code provenance is internally consistent;
- the owned-lab whole-run budget is mechanically bounded below the standing ten-minute limit and the emitted bound is truthful;
- both remote server paths have explicit bounded readiness evidence before timed client admission;
- exact repair-HEAD CI stable + nightly fuzz are green before the VPS run;
- the changed-hypothesis VPS attempt either produces a complete fair paired set or a packet-direction-classified blocker artifact;
- cleanup is verified;
- status/plan/roadmap preserve historical negatives and reflect accepted positive rows truthfully;
- `RELEASE_CANDIDATE=false`, `PRODUCTION_READY=false`, `FREEZE=false`, and `RELEASED=false` remain unchanged.

## Fallback

If the repaired C run shows that temporary HY2 UDP packets leave the owned client but do not arrive at the owned VPS (or replies do not traverse the provider/NAT path), preserve that as an environment/path blocker. Do not modify provider/firewall/production network policy to force success. Move to the next dependency-ready VPS-only evidence row or the local instrumentation/runtime slice that directly unlocks one.

If A/B exposes a Nekomusume production correctness defect rather than benchmark-harness evidence drift, stop comparison work, preserve a minimal reproducer, repair correctness first, run the required parser/fuzz/security gates, and only then return to C.

## Do not expand into

- changing Nekomusume wire/Session semantics to make the benchmark pass;
- production HY2 config/service changes;
- third-party targets, scanning, production firewall/route/DNS/proxy/tunnel/qdisc changes;
- benchmark security downgrade;
- repeated unchanged HY2 timeout attempts;
- over-10-minute experiments or pressure/capacity testing outside standing authorization;
- previous/current interoperability without a prior frozen release;
- 0-RTT, enabled FEC, striping/aggregation or exotic carriers without an observed-problem gate;
- RC/security/production approval from this evidence batch.

## Questions requiring maintainer decision

none.
