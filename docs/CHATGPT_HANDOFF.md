# Nekomusume ChatGPT Handoff

Checked at: 2026-09-01 22:57 Asia/Shanghai
Reviewed implementation HEAD: `df61091d379aa10ad001e24f04e2143e13c0cb08`
Previous reviewed implementation HEAD: `1eed79c01c82f723beb496100c5d3ebf74a970e4`
Previous reviewer handoff commit: `0d5b54f5059c8b84241f1373b0a8770a2bf8244d`

## What changed

One substantive coding-agent commit landed after the previous reviewer handoff:

- `df61091` — **runtime fail-closed repair + negative process tests + runtime-parser fuzz coverage + status/plan/ADR update; no new VPS evidence**. The failover responder now bounds accepted-stream negotiation/handshake/data reads by the remaining experiment duration, applies a one-second whole-sequence readiness deadline, refuses application-data admission after any unadmitted readiness request, emits `tcp_resource_admitted` only after the accepted sequence, repairs `failure_observation_elapsed_us` to mean the actual observation-window duration, adds explicit wrong-tuple/unadmitted/replayed-challenge/tampered/fewer-than-three/stall process failures, and adds `ProcessMessage::decode` to the existing fuzz target with readiness seeds.

The exact `df61091` GitHub Actions run (`Rust CI` run 101) completed successfully. Both jobs passed:

- `stable checks` -> `bash scripts/check.sh` passed;
- `nightly decode fuzz smoke` -> pinned `cargo-fuzz`, `cargo fuzz build decode`, and `cargo fuzz run decode -- -max_total_time=30 -max_len=8192` passed.

This is materially stronger verification than the previous handoff could observe. The previous D064 correctness/fuzz blocker is closed for the bounded research runner. No current-exact-head owned-VPS warm evidence has yet been committed, so the correct next move is to spend the rented-VPS window immediately rather than open another local feature slice.

## Review verdict

**SAFE TO PROCEED TO CURRENT-EXACT-HEAD VPS EVIDENCE — D064 local fail-closed gate accepted.**

Do not spend another cycle on unrelated local polish before using the VPS. The active phase remains `IMPLEMENTATION_PLAN.md` item 3, the bounded release evidence matrix.

The next coding-agent batch should use `df61091` as the reviewed implementation candidate for the first exact-head D064 warm/cold and periodic/resource rows. This handoff itself will create a later reviewer-only descendant commit; do not silently substitute that coordination commit for the reviewed implementation identity when recording evidence. Build/test the exact implementation commit or explicitly record both the reviewer descendant and the unchanged implementation tree identity.

## Review findings

### R-201 PASS — responder admission now fails closed before application data

The responder evaluates every readiness request against the live bounded runtime and terminates the candidate when `admitted=false`. `tcp_resource_admitted` is emitted only after the three-request loop. The application-data receive loop is therefore unreachable on the official bounded responder after a failed/unadmitted readiness proof.

The new process test exercises six negative classes and requires both peers to fail without `tcp_resource_admitted`, `tcp_warm`, `tcp_resumed`, or authenticated delivery-ack success. This directly closes the prior advisory-admission defect for the research runner.

### R-202 PASS — accepted-stream and readiness waits are now bounded

`bound_stream_to_deadline` limits accepted TCP negotiation, handshake and data I/O by the experiment deadline. The readiness sequence additionally uses a one-second whole-sequence deadline, matching the current D064 ADR. The stalled-readiness process test must finish within a bounded wall-clock interval and passed the current CI gate.

This is a bounded CLI/runtime property, not a general daemon/service liveness guarantee.

### R-203 PASS — health timing semantics were repaired

`failure_observation_started_us` remains a common-origin timestamp while `failure_observation_elapsed_us` is again the duration from the individual observation start to the current observation. The previous cumulative/mislabeled elapsed-field defect is closed in code.

The upcoming VPS collector must still reject semantically invalid/non-monotonic timestamps rather than accepting output merely because fields exist.

### R-204 PASS — new peer-controlled runtime codec has active fuzz/property coverage

The existing `decode` fuzz target now calls `neko_session::ProcessMessage::decode` on arbitrary input. Successful decodes must re-encode exactly and remain within `PROCESS_FRAME_MAX`. Readiness request/response corpus seeds are checked as exact process messages. GitHub Actions run 101 independently completed the 30-second bounded fuzz smoke successfully.

This is bounded fuzz evidence, not exhaustive parser proof or a security audit.

### R-205 PASS WITH SMALL PREFLIGHT — local gate is strong enough to move to VPS

GitHub Actions run 101 independently passed `scripts/check.sh` and the decode fuzz smoke on exact `df61091`. `scripts/check.sh` includes fmt, locked workspace check/test, all-target Clippy with warnings denied, governance/evidence/status/observability/release/plan/decision/canonical-vector checks and metadata/license gates.

The previous handoff explicitly requested an all-target/no-fail-fast workspace test command and `git diff --check`; the CI workflow does not separately expose those exact two commands. This is not a reason to delay another review cycle. Before deployment, the coding agent should run the small exact-head preflight below and then continue directly into VPS evidence.

### R-206 NOTE — old VPS evidence remains exact-commit evidence, not current warm proof

Existing self-owned VPS records already prove older-commit cross-host IPv4 TCP/UDP behavior, automatic **cold** threshold recovery, periodic Session behavior, resource sampling and package lifecycle slices. Those records remain valid for their exact commits, but none proves the new authenticated pre-failure D064 warm path at `df61091`.

The preserved older automatic-health row is explicitly `controlled self-owned application-level UDP reply cessation -> threshold -> cold TCP recovery`; do not relabel it warm or natural Internet loss.

### R-207 NOTE — IPv6 remains environment-blocked; HY2 remains high-value and READY after the D064 sample

The previously observed owned path lacked a usable global IPv6/default-route environment. Do not mechanically rerun that unchanged failure.

HY2 v2.9.3 remains pinned with its repository-recorded commit/hash, and `docs/research/hy2-forwarding-comparison-note-20260901.md` identifies a bounded application-forwarding seam that avoids the existing production Hysteria configuration. No valid equal-application paired sample exists yet. This is a high-value rented-VPS target after the D064 and periodic rows.

## Evidence boundaries

- `RELEASE_CANDIDATE=false`, global `FREEZE=false`, `PRODUCTION_READY=false`, and `RELEASED=false` remain correct.
- `CANONICAL_CORPUS_V1_FROZEN=true` remains corpus-specific and is not reopened by the D064 runtime codec.
- `df61091` has verified local/CI fail-closed and fuzz evidence, but **no current-exact-head VPS warm evidence yet**.
- Current GitHub Actions success is CI verification of repository gates and bounded fuzz smoke, not an independent release/security review.
- The upcoming warm/cold experiment remains a controlled self-owned fault-injection experiment. It cannot prove natural WAN blackhole behavior, public reachability, production failover, capacity or security approval.
- Existing historical five-minute periodic/resource evidence is exact to an older implementation. Repeating a scientifically distinct current-head periodic row is justified because the runtime baseline materially changed; do not repeat it merely to chase better numbers.
- Standing authorization covers the owned-endpoint TCP/UDP failover, periodic Session, resource sampling, temporary HY2 comparison services, bounded benchmark/capture and cleanup required below. No new per-run approval is required.
- IPv6 remains blocked until the actual owned environment changes.
- VPS rental time is a priority constraint. VPS-only evidence now outranks unrelated local feature/document work.

## Work Package — harvest `df61091` VPS evidence, then execute the first fair HY2 pair

Execute A -> B -> C -> D in dependency order. This is intentionally a thick evidence batch. Do not stop after the preflight or after the first successful warm run if the remaining work is still within standing authorization and the environment is healthy.

### Primary A — exact-head preflight + interleaved D064 warm/cold VPS batch

**Goal:** obtain the first reviewed current-exact-head real-socket D064 warm evidence while preserving a directly comparable cold baseline.

#### A0 — exact-head preflight

Use exact implementation commit:

```text
df61091d379aa10ad001e24f04e2143e13c0cb08
```

Before deployment:

1. fetch and verify the commit exists on the authoritative remote;
2. use a clean detached/integration worktree that does not read/copy/commit protected identity material;
3. run at minimum:
   - `cargo test --workspace --all-targets --locked --no-fail-fast`;
   - `git diff --check`;
4. build the release binary once for the VPS batch and record its SHA-256;
5. record exact compiler/target/host metadata needed to reproduce the artifact;
6. if A0 exposes a correctness failure, stop the affected D064 path and use the correctness fallback below. Otherwise continue directly into A1 without waiting for another reviewer turn.

The already successful GitHub Actions stable+fuzz run does not need to be mechanically rerun just for ceremony unless the local build environment exposes a real difference.

#### A1 — interleaved warm/cold recovery sample

Use the established self-owned path that previously produced valid cross-host evidence. Do not retry the unchanged public-address negative path merely because it exists historically.

Prefer an interleaved batch of **5 warm + 5 cold** recoveries on the same exact binary if the complete experiment remains comfortably within standing authorization. Concurrency should remain 1 and per-run application traffic should remain small; this is a recovery-evidence sample, not a stress test.

For every run preserve at least:

- exact implementation commit and binary SHA-256;
- experiment ID, run index/order, ports, count/bytes/duration, endpoint ownership/path class;
- canonical version negotiation result and Noise authentication success without secrets;
- warm runs: exactly three authenticated readiness request/response challenge IDs, exact tuple/generation/epoch, live `admitted=true`, and timestamp of the third accepted response;
- proof warm TCP connect/negotiation/auth/resume/readiness completed before UDP failure decision;
- proof UDP remained sole application-data owner before promotion;
- failure observation start + per-window elapsed durations;
- threshold/failure decision timestamp and reason/cause classification;
- warm/cold fallback class and promotion gate;
- first resumed application-data acceptance timestamp;
- uncertain resend range and exact authenticated Session `DeliveryAck` result;
- logical attempted/confirmed/missing/duplicate/conflict counts and application bytes;
- recovery latency per run;
- warm/cold median/P95 only if computed from the preserved raw sample set;
- client/server exit status and cleanup result;
- process CPU/RSS/FD/socket observations where the existing sampler can be attached without altering the protocol behavior.

The collector/review logic must fail closed if any warm run shows:

- fewer/more than three readiness successes;
- any `admitted=false`;
- repeated/non-consecutive challenge IDs;
- `tcp_resource_admitted` before the final accepted response;
- application data before failure/promotion;
- warm setup after the failure decision;
- non-monotonic common-origin timestamps;
- cumulative/mislabeled `failure_observation_elapsed_us`;
- missing/duplicate/conflicting logical delivery beyond the declared Session behavior.

Retain failed runs. Do not silently discard outliers or rerun unchanged failures to improve the distribution.

Classification remains exactly:

```text
controlled self-owned application-level UDP reply cessation
-> D064 threshold decision
-> authenticated warm/cold TCP recovery
```

It is not natural Internet blackhole evidence, public reachability, production failover or a superiority benchmark.

### Follow-up B — current-head five-minute periodic Session + process-resource sample

**Dependency:** A cleanup complete. Use the same exact `df61091` binary if the periodic runner is compatible.

Run one scientifically distinct current-head periodic authenticated Session, approximately the existing historical profile:

```text
duration <= 300 s
concurrency = 1
60 records
32 B / record
every 5 s
~1920 application bytes
```

Stay within the standing authorization and keep the application load intentionally low.

Record:

- exact commit/binary hash and endpoint/path class;
- negotiated/authenticated Session identity without secrets;
- attempted/confirmed/missing/duplicate counts;
- confirmation latency raw/summary values already produced by the runner;
- elapsed time/application bytes;
- process CPU user/system, max RSS, peak FD, peak owned sockets via the existing sampler;
- listener/process cleanup and any sampler failure;
- no reconnect/resume claim unless the runner actually exercises it.

This creates current-runtime resilience/resource evidence. It does not turn five minutes into production long-lived proof.

Do not retry IPv6 while the owned path is unchanged.

### Follow-up C — first valid equal-application Nekomusume/HY2 paired VPS sample

**Dependency:** A/B cleanup complete; do not run CPU-heavy builds/fuzz while collecting performance samples.

Reuse:

- pinned HY2 v2.9.3 artifact and repository-recorded SHA-256;
- `docs/research/hy2-forwarding-comparison-note-20260901.md`;
- `docs/bench/hy2-comparison-workload.md` result methodology/schema;
- existing process resource sampler.

Do **not** weaken the loopback-only guard in `scripts/bench/compare-hy2.sh`. If the self-owned-VPS orchestrator/adapter still does not exist, implement the smallest separate fail-closed lab orchestrator that:

1. accepts only the explicitly configured owned lab endpoint/path contract;
2. uses fresh experiment-only high ports;
3. starts a temporary HY2 v2.9.3 server using generated disposable TLS/auth material, without reading/reusing `/etc/hysteria/server.yaml` secrets and without stopping/reconfiguring the existing Hysteria service;
4. starts a temporary HY2 client `tcpForwarding` path and a temporary loopback TCP echo target on the VPS;
5. invokes an equivalent bounded Nekomusume authenticated echo command for the same application question: exact payload in -> exact payload echoed back;
6. uses one deterministic payload file with exact byte count + SHA-256 for both implementations;
7. records same client/VPS pair, close time window, route/MTU metadata, authenticated-encrypted security class, single-stream load shape, timeout and run count;
8. traps cleanup and verifies all temporary listeners/processes/config/certs/auth files are removed;
9. fails closed on payload mismatch, incomplete exchange, malformed JSON/result or cleanup failure.

After any necessary orchestrator commit passes its appropriate local/script checks, run a small nearby/interleaved paired sample, preferably **5 Nekomusume + 5 HY2** if bounded execution remains well inside standing authorization.

Preserve raw samples and report only:

- latency/timing raw values and median/P95;
- failure counts;
- CPU user/system;
- max RSS;
- FD count;
- application bytes;
- `wire_bytes=null` unless a bounded capture has trustworthy provenance.

Do not make a superiority claim from this first paired sample. A slower or failed Nekomusume row is valid evidence and must be preserved.

If a fair application-semantic pair is genuinely blocked, record the exact technical/environment blocker and continue to the fallback rather than altering production Hysteria/firewall/route/tunnel state.

### Follow-up D — reconcile the release-evidence ledger after the real runs

**Dependency:** A-C completed or truthfully blocked.

Update only the evidence/status/navigation that the actual runs justify:

- add exact evidence documents/artifact hashes for A/B/C;
- link the current-head D064 warm/cold result from `docs/status.md` if A genuinely passes;
- retain the older cold/periodic/negative rows as historical exact-commit evidence;
- keep IPv6 explicitly environment-blocked if unchanged;
- keep `IMPLEMENTATION_PLAN.md` bounded release evidence matrix open unless every required row is really evidenced or reviewed as inapplicable;
- preserve `RELEASE_CANDIDATE=false`, global `FREEZE=false`, `PRODUCTION_READY=false`, `RELEASED=false`;
- do not turn self-owned path evidence into public/general WAN claims.

Run the normal repository gate for any tracked changes and push a coherent checkpoint.

## Fallback

If A0/A1 exposes a real D064 correctness defect:

1. preserve the exact failing run/reproducer and binary identity;
2. stop only the affected warm claim;
3. make the smallest correctness repair and required tests/fuzz if parser behavior changes;
4. run the local gate on the repaired exact commit;
5. rerun only the changed hypothesis/path;
6. while the warm repair is in progress, independent current-head periodic/resource or HY2 orchestration work may continue if it does not depend on the broken warm path.

If the owned VPS path itself is temporarily unavailable, use the cycle to finish the HY2 self-owned-lab orchestrator/adapter and any current-head resource/package evidence that directly prepares the next valid VPS window; do not invent unrelated features.

If HY2 cannot be run fairly without touching production Hysteria, firewall, route, DNS, proxy or tunnel configuration, preserve that blocker and use the remaining VPS window for a current-head package build/install/smoke/readiness/cleanup row or another already-defined release-evidence row. Do not widen authorization.

Do not mechanically rerun the unchanged IPv6 failure.

## Completion gates

This package is complete only when the following are truthfully resolved:

- `df61091` exact-head preflight is green;
- the D064 current-head warm path has real self-owned socket evidence only if three authenticated peer readiness responses genuinely precede failure/promotion/data;
- cold comparison remains separately classified;
- raw warm/cold samples, failures, timing and cleanup are preserved;
- a current-head periodic/resource row is preserved if B runs;
- HY2 is marked compared only if a fair equal-application paired sample really executes;
- no production Hysteria config/service or network policy is modified;
- negative/superseded historical evidence remains retained;
- IPv6 remains blocked unless the environment actually changes;
- release/security/production/global-freeze flags remain unchanged;
- the coding agent does not stop after a small substep while later dependency-ordered work in this package remains READY.

## Do not expand into

- reopening or changing the frozen N9 corpus for D064 runtime messages;
- concurrent TCP+UDP application-data striping/aggregation;
- enabled FEC, 0-RTT or exotic carriers without an observed-problem gate;
- third-party targets, scanning or production network changes;
- repeated IPv6 probes without a real path change;
- >10-minute single experiments or high-volume/high-concurrency stress outside standing authorization;
- performance superiority claims from the first paired sample;
- RC/security/production approval before the independent review gate.

## Questions requiring maintainer decision

none.
