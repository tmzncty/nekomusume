# Nekomusume ChatGPT Handoff

Checked at: 2026-09-02 07:58 Asia/Shanghai
Repository HEAD: `62d4d3576e24d4b0f951a615b0d116a74f5d7a73`
Previous reviewed implementation HEAD: `a9a96b16050e38de158033de5e9bb4406414df58`
Previous reviewer handoff commit: `06abe32668310b401523f9120ca9cba88b705e3e`

## What changed

One substantive coding-agent commit landed after the previous reviewer handoff:

- `62d4d35` — **HY2 comparison harness/evidence-retention repair; no Nekomusume production runtime change and no new VPS/WAN result.** It adds a `BLOCKED_HARNESS` artifact/schema, atomically retained per-sample JSONL, a strict GNU-time sentinel parser, complete-set summary validation, stronger cleanup bookkeeping, new validator/mutation tests, and wires the validator test into `scripts/check.sh`.

The intent is correct: a failed paired comparison should retain already-observed raw samples and cleanup facts instead of disappearing or emitting a misleading partial performance summary. However, this commit is **not green** and is not safe to use as the next VPS comparison harness yet.

GitHub Actions `Rust CI` run 108 for exact HEAD `62d4d35` completed with:

- `nightly decode fuzz smoke`: **PASS**;
- `stable checks` / `bash scripts/check.sh`: **FAIL**.

The immediate regression is therefore in the ordinary repository gate introduced or exposed by the benchmark-harness change, not in the nightly decode fuzz target. Parent implementation HEAD `a9a96b1` had a green Rust CI run 106.

Independent code review also found a cleanup/evidence-integrity issue that must be fixed before this harness is executed on the rented VPS: `process-resource-sampler.py` forks the sampled command into a separate process group, while the new harness cleanup kills/waits the sampler wrapper PID. Killing the wrapper does not prove or guarantee that the sampled HY2 child process group is terminated. Both the local HY2 client and remote HY2 server use this wrapper. The harness can therefore detect a residual listener, but can still leave an orphaned experimental child after the wrapper is killed; conversely, the successful result currently hard-codes `*_processes_reaped=true` without independently proving every tracked child/group is gone. This conflicts with the standing cleanup requirement and with the result's claimed cleanup semantics.

A second evidence issue exists in blocked artifacts: if failure occurs after traps are installed but before the deterministic payload is constructed, the fallback currently supplies `sha256("empty")` while still declaring the configured `payload_bytes`. That is not the hash of the declared payload and must not be represented as exact payload evidence. Cleanup failure also collapses actual/unknown residual state into fixed synthetic `1` counts for both local and remote listeners rather than preserving what was really observed.

## Review verdict

**HOLD VPS HARVEST ON CURRENT HEAD — repair exact-head stable CI plus process-tree/blocked-evidence truthfulness, then immediately resume the rented-VPS evidence batch without waiting for another reviewer.**

This is a bounded harness/safety repair, not a Session/Carrier architecture blocker. Do not spend the cycle on unrelated local polish. The previous D064 and periodic runtime repairs remain accepted; the VPS window is still the highest-value resource immediately after this harness gate turns green.

## Review findings

### R-310 BLOCKER — exact HEAD fails the stable repository gate

Rust CI run 108 for `62d4d35` failed in `bash scripts/check.sh`; the nightly bounded decode fuzz job passed. The coding agent must inspect/reproduce the exact failing stable sub-check and repair it rather than relying on the commit message or reported local success.

Because the parent `a9a96b1` was green and `62d4d35` changes only benchmark/docs/schema/test/check infrastructure, keep the repair scoped to the changed harness/evidence path unless the reproduced failure proves otherwise.

Do not remove the new tests, skip `scripts/check.sh`, weaken fail-closed validation, or mark CI failure as irrelevant merely because production Rust runtime code did not change.

### R-311 BLOCKER — cleanup does not prove sampled child process groups are reaped

`process-resource-sampler.py` forks the sampled command and calls `setpgid(child, child)`. The harness records and later terminates the sampler wrapper PID. External termination of the wrapper currently has no signal-forwarding/finally contract that guarantees the sampled child process group is terminated and reaped.

This matters in both directions:

- local HY2 client: `local_pids` contains the sampler wrapper, not proof of the HY2 child group;
- remote HY2 server: the remote `pids` file contains the sampler wrapper, not proof of the Hysteria child group.

A residual-port check is useful but insufficient to assert `local_processes_reaped=true` / `remote_process_groups_reaped=true`: a child may remain without the checked listener, and killing the supervisor may remove the only component enforcing its bounded max duration.

**Required repair:** establish one explicit bounded process-tree ownership contract. Preferred options are:

1. make `process-resource-sampler.py` handle external `SIGTERM`/`SIGINT`, terminate the sampled child's process group, wait/reap it, and still emit truthful cleanup/result state; or
2. change the orchestrator so it tracks and terminates/verifies the actual sampled child process groups directly without orphaning the sampler.

Whichever option is chosen, tests must prove externally interrupted samplers do not leave a child/listener behind. The final harness may claim a process/group is reaped only after direct PID/PGID disappearance verification, not merely because the experiment port is closed.

Do not solve this with broad `pkill hysteria`, production service operations, or unscoped process killing.

### R-312 BLOCKER — blocked artifacts must not synthesize payload or cleanup facts

The current blocked path can create a contract with configured `payload_bytes` plus a fallback SHA-256 of the literal string `empty` before the intended deterministic payload exists. That is not exact payload evidence.

Likewise, when cleanup is not verified, the blocked artifact currently substitutes fixed nonzero listener counts rather than carrying the actual measured count or an explicit unknown/not-observed state.

**Required repair:** make phase boundaries truthful. Acceptable designs include:

- construct and hash the deterministic payload before any trapped execution stage that can emit a blocked artifact declaring payload identity; or
- allow a pre-payload blocked artifact to represent `payload_prepared=false` / null payload hash and validate that distinction explicitly.

For cleanup, preserve actual measured residual PID/PGID/listener/temp-path facts. If a fact could not be observed, represent it as unknown/not-verified rather than inventing a count. `cleanup_status=failed` must not imply a fabricated topology of failure.

### R-313 PASS WITH GATE — raw sample retention and no-partial-summary direction is sound

Atomic per-sample retention, complete-set validation, strict GNU-time sentinel parsing, and a separate `BLOCKED_HARNESS` document are appropriate release-evidence infrastructure. Preserve these properties while repairing R-310/R-312.

A blocked comparison may retain a valid prefix of the planned interleaved sample order, but it must not calculate or publish a partial median/P95 comparison as if the pair completed.

### R-314 PASS — no new release/runtime claim was made by `62d4d35`

The commit changes benchmark/evidence infrastructure only. It does not create a HY2 result, D064 result, periodic result, public-WAN claim, security approval, or RC state. Keep that boundary.

## Evidence boundaries

- `IMPLEMENTATION_COMPLETE=true` remains the bounded research-baseline status.
- `CANONICAL_CORPUS_V1_FROZEN=true` remains corpus-specific.
- `RELEASE_CANDIDATE=false`, global `FREEZE=false`, `PRODUCTION_READY=false`, `RELEASED=false` remain correct.
- Parent runtime repair state `a9a96b1` had green CI; exact current repository HEAD `62d4d35` has a failing stable gate and must not be described as fully gated.
- Nightly decode fuzz passing at `62d4d35` does not cancel the failed stable gate.
- No new VPS/WAN evidence landed after the previous handoff.
- The earlier D064 negative row remains immutable: two readiness observations succeeded and challenge 3 failed; no warm promotion/resumed application delivery was proven by that row.
- The earlier periodic negative row remains immutable: server authentication succeeded but no application record was confirmed.
- The earlier HY2 negative row remains immutable: temporary QUIC/UDP setup did not reach a usable forwarding path; no fair paired sample exists.
- Standing authorization still covers bounded self-owned TCP/UDP experiments, temporary HY2 lab services, resource sampling, capture and cleanup. It does **not** permit leaving experimental processes/listeners behind.
- IPv6 remains environment-blocked unless a genuinely usable owned path appears; do not repeat unchanged failure.
- Production route/firewall/DNS/proxy/tunnel/qdisc changes, third-party targets and scope expansion remain prohibited.

## Work Package — repair the comparison harness gate, then spend the VPS window

Execute A -> B -> C/D/E -> F. C/D/E are independent after B and cleanup. This is deliberately thick: once A/B are green, do not wait for another reviewer before collecting the already-READY VPS evidence.

### Primary A — reproduce and close the exact stable-check regression

**Goal:** restore the normal repository gate without weakening the new evidence-retention contract.

1. Fetch exact `62d4d35` and inspect Rust CI run 108 stable-check output.
2. Reproduce the failing `bash scripts/check.sh` sub-check locally.
3. Identify the exact failure before editing. Because `62d4d35` changed benchmark/docs/schema/test/check files only, start with:
   - `scripts/bench/compare-hy2-owned-lab.sh`;
   - `scripts/bench/compare-hy2-owned-lab-test.sh`;
   - `scripts/bench/validate-hy2-owned-lab.py`;
   - `scripts/bench/validate-hy2-owned-lab-test.py`;
   - `schema/benchmark-blocked-harness.v1.json`;
   - `docs/bench/result-schema-v1.md`;
   - `scripts/check.sh`.
4. Fix the cause; do not delete/skip the failing assertion unless the contract itself is demonstrably wrong and the replacement is stricter/truthful.
5. Run the narrow changed tests first, then full `bash scripts/check.sh` and `git diff --check`.

### Follow-up B — make process-tree cleanup and blocked evidence truthful

**Dependency:** A diagnosis available; may be combined with A if the same code path caused CI failure.

#### B1 sampler external-termination contract

Add executable tests around `process-resource-sampler.py` proving that when the sampler is externally interrupted while its child is alive:

- the child process group receives bounded termination;
- the child is reaped or independently proven gone;
- owned listener/socket state reaches zero;
- the sampler does not leave an orphan;
- result/cleanup state does not falsely say complete if cleanup could not be proven.

Keep termination scoped to the sampler-owned child PGID.

#### B2 harness cleanup verification

For local and remote experiment processes, track enough identity to verify the specific experiment-owned wrapper/child process groups disappear. Preserve port checks and temp-path removal as additional evidence; do not use them as a substitute for process disappearance.

A successful final result may set process/group cleanup booleans true only after direct verification. A failed cleanup must preserve actual observed residual/unknown facts.

#### B3 blocked payload/cleanup schema

Repair the pre-payload blocked-artifact ambiguity and validate it with mutation tests. Never pair configured payload length with a fabricated fallback hash. Never turn an unknown cleanup count into a made-up integer.

Add tests for failures at at least:

- post-trap/pre-payload or earliest artifact-producing setup stage;
- client sample failure after some retained rows;
- resource evidence failure;
- cleanup verification failure;
- final assembly/validation failure.

Each case must retain the valid sample prefix that actually existed and must not emit a partial performance summary.

#### B4 full green gate

Run:

- changed benchmark shell tests;
- validator tests;
- sampler tests;
- shell syntax gate;
- `bash scripts/check.sh`;
- `git diff --check`.

Push the repair. Confirm the new exact-head GitHub Actions stable job is green before using the repaired HY2 harness for a comparative VPS run. The nightly fuzz job is not a substitute for the stable gate.

### Follow-up C — exact-head D064 warm/cold VPS evidence

**Dependency:** B local gate green. This branch does not depend on HY2 path viability.

Use a fresh clean worktree and exact repaired HEAD. Record git/tree, native release binary SHA-256, rustc/Cargo versions and cleanup baseline. Use the known self-owned established path; do not rediscover blocked public ingress.

Run one changed-code warm D064 sample requiring canonical UDP negotiation + Noise, logical delivery proof, controlled UDP reply cessation, TCP negotiation + Noise + resume, exactly three authenticated readiness observations, atomic promotion before resumed data, exact DeliveryAck and complete uncertain/dedup accounting.

If warm succeeds, continue in the same batch with interleaved **5 warm + 5 cold** raw samples, concurrency 1, small payloads, retaining recovery latency/failures and existing resource samples. If warm fails with a new defect, retain it and stop only C.

Do not reclassify controlled application-level reply cessation as a natural Internet blackhole.

### Follow-up D — exact-head five-minute periodic/resource VPS observation

**Dependency:** B green; independent of C after cleanup.

Use the already repaired setup/ACK separation with the prior bounded profile:

```text
duration <= 300 s
concurrency = 1
records = 60
bytes = 32 / record
interval = 5000 ms
setup_timeout = 5000 ms
ack_timeout = 1000 ms
```

Require authenticated setup before record 1. Retain attempted/confirmed/missing/duplicate/conflict counts, raw confirmation latencies, actual application bytes, CPU/RSS/FD/owned-socket samples, exact binary identity, exit status and cleanup.

If setup succeeds and later application ACK misses its 1000 ms bound, classify it as application-delivery failure, not handshake failure. A full success remains only a five-minute bounded self-owned resilience/resource observation.

### Follow-up E — repaired HY2 explicit-bind diagnostic and first fair pair

**Dependency:** B green including process-tree cleanup tests; independent of C/D after cleanup.

1. Read only current owned VPS interface metadata and select the already-authorized explicit assigned bind address; do not modify firewall/route/NAT/provider policy.
2. Run adapter preflight and one short changed-harness HY2 diagnostic with bounded capture on the experiment UDP port if needed.
3. Classify exactly: no datagrams arrive / datagrams arrive but no usable response / QUIC+TLS+auth+forwarding ready.
4. If path is viable, run interleaved **5 Nekomusume + 5 HY2** equal-application samples with pinned HY2 v2.9.3 and the existing exact-payload contract.
5. Retain raw samples, failures, median/P95 only for a complete valid set, CPU/RSS/FD/application bytes, binary identities and verified process/listener/temp cleanup. `wire_bytes` remains null unless capture provenance is truly comparable.
6. Make no superiority claim from the first pair.

If the environment remains non-viable, preserve one changed diagnostic and stop E. Do not rerun unchanged or widen network policy.

### Follow-up F — evidence/status reconciliation

**Dependency:** after each C/D/E branch reaches one retained positive or negative terminal row.

- preserve exact experiment ID, commit/tree/binary hashes, actual parameters, time window, path/endpoint ownership class, raw results and cleanup;
- preserve older negative rows rather than overwrite them;
- update only evidence/status claims actually changed by the new row;
- distinguish authorized execution, self-owned cross-host evidence, public/general reachability, release evidence, security approval and production readiness;
- keep `RELEASE_CANDIDATE=false`, global `FREEZE=false`, `PRODUCTION_READY=false`, `RELEASED=false`.

## Fallback / spare VPS opportunity

If one VPS branch is blocked, continue the other independent READY branch. If C/D/E all terminate in distinct retained blockers, use remaining rental-window effort only for a question made scientifically distinct by current code:

1. exact-head package install/smoke/readiness/shutdown/cleanup with binary/package SHA-256;
2. bounded authenticated TCP/UDP process-resource sample if current runtime differs from the older resource row;
3. repeated native microbenchmark only with explicit warm-up/sample protocol and no concurrent CPU-heavy work;
4. parser/property/fuzz only if relevant parser/wire code changed.

Do not keep the VPS busy with unchanged failed scenarios.

## Completion gates

This batch is complete only when:

- the exact-head stable CI regression is repaired and a new exact-head stable job is green;
- sampler/harness cleanup proves experiment-owned child process groups are gone, not merely that a wrapper PID was waited;
- blocked artifacts carry truthful payload and cleanup facts without synthetic hashes/counts;
- no partial failed comparison can produce a misleading summary;
- one exact-head D064 terminal VPS row is retained, and 5+5 warm/cold follows if warm is correct;
- one exact-head five-minute periodic terminal row is retained;
- one changed HY2 explicit-bind terminal diagnostic is retained, and a complete 5+5 fair pair follows only if viable;
- every experiment records provenance, actual parameters and verified cleanup;
- no production network policy or third-party target is touched;
- status/evidence docs reflect only observed facts;
- `RELEASE_CANDIDATE=false`, global `FREEZE=false`, `PRODUCTION_READY=false`, `RELEASED=false` remain unchanged.

## Do not expand into

- Session/Carrier architecture redesign;
- reducing readiness proof count merely to make D064 pass;
- weakening authentication/admission/tuple-generation-epoch binding;
- UDP+TCP striping/aggregation;
- durable Session restart persistence;
- production Hysteria/firewall/NAT/route/provider changes;
- broad process killing such as production-scoped `pkill hysteria`;
- third-party targets/scanning;
- repeated unchanged IPv6/public-path/HY2 failures;
- performance-superiority claims from one paired batch;
- unrelated experimental carriers/speculative features.

## Questions requiring maintainer decision

none.
