# Nekomusume ChatGPT Handoff

Checked at: 2026-09-02 02:59 Asia/Shanghai
Repository HEAD: `83f0a0720aa77cc7b811bac4ad6083acd6fe2488`
Previous reviewed implementation HEAD: `df61091d379aa10ad001e24f04e2143e13c0cb08`
Previous reviewer handoff commit: `3978f3fdd3fb34510468a2e1708c0b2c5c5f6aec`

## What changed

One substantive coding-agent commit landed after the previous reviewer handoff:

- `83f0a07` — **benchmark/application-seam implementation + tests + negative VPS evidence reconciliation; no positive new release-evidence row.** It adds an exact external-payload mode to the authenticated TCP probe (`--payload-file`, exact byte count + SHA-256 + FD count + `wire_bytes=null`), adds a bounded exact-payload forwarding adapter and a separate self-owned-lab HY2 orchestrator, extends the benchmark result schema, adds fail-closed tests for the new benchmark contract, and records the three requested VPS follow-ups as truthful negative evidence.

The exact `83f0a07` GitHub Actions run (`Rust CI` run 103) completed successfully. `stable checks` passed `bash scripts/check.sh`; `nightly decode fuzz smoke` built the pinned fuzz target and completed the bounded 30-second decode fuzz run successfully.

The VPS evidence is useful precisely because it exposed concrete blockers:

1. **D064 warm path:** UDP negotiation/authentication and logical delivery proof succeeded; TCP negotiation, Noise authentication and resume validation succeeded; readiness challenges 1 and 2 returned authenticated `admitted=true`; challenge 3 timed out / was rejected before `tcp_resource_admitted`, promotion or TCP application data.
2. **Periodic Session:** TCP connected; the server completed canonical negotiation and Noise authentication, but the client failed to receive/complete the handshake response before any application record. Actual application bytes were zero.
3. **HY2 pair:** the exact-payload seam and owned-lab orchestrator are implemented and locally gated, but the temporary HY2 v2.9.3 QUIC/UDP path timed out before the forwarding listener became ready. No paired performance sample exists.

These are not reasons to stop the project. They are the next engineering inputs. The rented-VPS priority policy says to repair the smallest local blocker that unlocks a changed, scientifically valid VPS retry.

## Review verdict

**CONTINUE WITH REQUIRED FIXES — two timeout-semantics defects block the next useful D064/periodic evidence; one HY2 bind-scope defect blocks another HY2 attempt.**

Do not mechanically rerun the three failed VPS scenarios at the current code/configuration. The next batch should repair the observed deadline semantics and HY2 listener safety contract, then immediately return to exact-head VPS evidence while the rented environment is available.

No core Session-vs-Carrier architecture change is required. The D064 readiness policy amendment is evidence-driven: preserve `k_ready=3`, authentication, exact tuple/generation/epoch binding, one outstanding readiness request, fail-closed admission, and single-active ownership; repair only the runtime timeout budget that the real path has now falsified.

## Review findings

### R-301 RELEASE-BLOCKING CORRECTNESS — D064 one-second whole-sequence deadline is incompatible with the observed path

The retained changed-path client timestamps show:

```text
challenge 1: 1047644 -> 1463124 us = 415480 us
challenge 2: 1463152 -> 1866848 us = 403696 us
```

The first two sequential authenticated observations therefore consumed about **819 ms**. Under the current one-second whole-sequence deadline, only about **181 ms** remained for challenge 3. The same path had already demonstrated roughly 400 ms per readiness round trip, so the third sequential proof was structurally unlikely to fit even without a protocol defect.

This is stronger evidence than “the WAN was flaky.” The current runtime addendum conflates a bounded per-probe timeout with the total budget for three mandatory sequential observations. Do not rerun unchanged.

### R-302 RELEASE-EVIDENCE CORRECTNESS — periodic `ack_timeout` is incorrectly reused as setup/handshake deadline

`periodic::client` uses the configured application acknowledgement timeout as the deadline for TCP setup + canonical negotiation + Noise handshake (`start + cfg.ack_timeout`). The failed VPS row used a 1000 ms ACK timeout. The server completed authentication while the client reported `handshake response failed`, which is consistent with a control-plane setup budget that is too short for the real cross-host path.

An application `DeliveryAck` timeout and a Session setup/handshake timeout are different contracts. Keep them separate so a conservative real-path setup budget does not weaken per-record delivery failure detection.

### R-303 HY2 SAFETY BLOCKER — owned-lab orchestrator currently uses a wildcard HY2 listener

`docs/bench/hy2-vps-setup-20260830.md` explicitly requires: **“Bind only the dedicated VPS address; do not bind `0.0.0.0`.”** The current generated HY2 server config uses:

```yaml
listen: :<temporary-port>
```

which is a wildcard listener rather than an explicit dedicated-address bind. Random disposable authentication reduces exposure but does not erase the repository's explicit lab safety contract. Fix this before any new HY2 run. If the intended owned lab address is not locally bindable on the VPS, record that as an environment blocker; do not fall back to wildcard bind and do not change firewall/NAT/provider policy.

### R-304 PASS — exact-payload comparison seam is now real implementation, not just a plan

The authenticated TCP client can consume one bounded external payload, verify the echoed bytes, and report the exact payload hash/application bytes/FD count. Local tests cover the valid exact-payload path and reject UDP, multi-count and non-JSON misuse. The separate HY2 lab orchestrator and its static/fail-closed guard tests exist. CI run 103 passed the repository stable gate and bounded fuzz smoke.

This is comparison infrastructure only. It is not performance evidence.

### R-305 PASS — negative VPS evidence boundaries are truthful

The A/B/C evidence files retain failed runs, exact implementation/binary provenance and cleanup, and do not manufacture warm recovery, five-minute stability, HY2 statistics, IPv6 reachability, public WAN or superiority claims. Keep those negative rows immutable as exact-run evidence.

## Evidence boundaries

- `IMPLEMENTATION_COMPLETE=true` remains the bounded research-baseline status.
- `CANONICAL_CORPUS_V1_FROZEN=true` remains corpus-specific.
- `RELEASE_CANDIDATE=false`, global `FREEZE=false`, `PRODUCTION_READY=false`, `RELEASED=false` remain correct.
- The current positive real-socket evidence still includes older exact-commit TCP/UDP/lifecycle/cold-recovery rows; `83f0a07` adds three blocker rows rather than new positive release rows.
- D064 challenge-3 failure is a controlled self-owned application-level UDP reply-cessation experiment. It is not natural Internet blackhole evidence.
- The periodic failure proves neither five-minute stability nor reconnect/resume behavior.
- HY2 v2.9.3 remains pinned, but no equal-application pair exists.
- IPv6 remains environment-blocked; do not rerun unchanged.
- Standing authorization covers the repaired self-owned TCP/UDP experiments, bounded capture, process/resource sampling, temporary HY2 lab services and cleanup below. No per-run maintainer approval is needed.
- Production firewall/route/DNS/proxy/tunnel/qdisc changes remain outside authorization.

## Work Package — repair control-plane budgets, then immediately harvest changed-path VPS evidence

Execute A -> B -> C -> D in order where dependencies apply. This is intentionally a thick batch. If one VPS branch remains blocked, continue the independent READY branch instead of waiting.

### Primary A — separate and bound control-plane timeout semantics

**Goal:** repair the two real-path timeout couplings without weakening fail-closed behavior or changing wire semantics.

#### A1 — D064 readiness deadline amendment

Preserve:

- exactly three authenticated sequential readiness observations;
- one outstanding request at a time;
- exact Session/PathId/PathGeneration/DeliveryEpoch/challenge binding;
- live responder-side resource admission;
- no TCP application data before atomic promotion;
- failure/reset on malformed, wrong tuple, duplicate/non-consecutive challenge, authentication failure or `admitted=false`.

Replace the current one-second **whole-sequence** readiness deadline with an explicitly bounded per-observation + whole-sequence policy. The simplest acceptable candidate is:

```text
READINESS_PROBE_TIMEOUT = 1 s per request/response
READINESS_SEQUENCE_TIMEOUT = k_ready * READINESS_PROBE_TIMEOUT = 3 s
```

The sequence deadline starts immediately before readiness challenge 1, after TCP negotiation/authentication/resume validation. Every read/write is bounded by the minimum of the per-probe deadline, remaining sequence deadline and remaining experiment deadline. Client and server must use compatible semantics. A single probe may not consume more than its per-probe budget, and the total three-proof sequence may not exceed the sequence budget.

If implementation evidence supports a different equally bounded formula, document it in D064 and tests; do not silently use an arbitrary large timeout.

Required deterministic/process tests:

- three responses each delayed roughly 350-450 ms succeed under the whole-sequence budget;
- one individual response exceeding the per-probe timeout fails closed;
- cumulative sequence exceeding the whole-sequence budget fails closed;
- two successes + third failure never emit `tcp_resource_admitted`, `tcp_warm`, promotion or application data;
- exactly three valid responses still produce the one and only readiness transition.

Update the D064 ADR/runtime addendum and status text in the same commit so the one-second whole-sequence claim does not survive as spec drift.

#### A2 — periodic setup timeout separated from `DeliveryAck` timeout

Do not reuse `ack_timeout` for TCP connect + version negotiation + Noise handshake.

Introduce a bounded setup budget with an explicit reproducible contract, for example:

```text
setup_timeout_ms default = 5000
bounded maximum = 10000
ack_timeout_ms remains the per-record DeliveryAck deadline
```

The setup budget begins before TCP connect and covers connect + canonical negotiation + Noise handshake. After authenticated Session establishment, only the per-record ACK budget governs acknowledgement waiting. The server must also bound accepted setup by a compatible finite deadline rather than the full workload duration alone.

Required tests must prove semantic separation, not just argument parsing:

- a setup/handshake delay greater than `ack_timeout_ms` but less than `setup_timeout_ms` can still establish the Session;
- after setup succeeds, a DeliveryAck delayed beyond `ack_timeout_ms` still fails the record as before;
- setup beyond `setup_timeout_ms` fails closed with zero application records admitted;
- malformed/unauthenticated setup remains fail closed.

Do not add reconnect persistence or change Session delivery semantics in this slice.

#### A3 — local/CI gate

Run at minimum:

- targeted D064 process/readiness tests;
- targeted periodic process tests;
- `cargo fmt --all -- --check`;
- `cargo check --workspace --locked`;
- `cargo test --workspace --all-targets --locked --no-fail-fast`;
- `cargo clippy --workspace --all-targets --locked -- -D warnings`;
- `bash scripts/check.sh`;
- `git diff --check`.

Run fuzz smoke only if a peer-controlled parser/codec changes. Pure deadline-policy changes do not require a fake new fuzz claim.

Push a coherent repair checkpoint before the changed-path VPS reruns.

### Follow-up B — changed-path exact-head D064 warm/cold sample

**Dependency:** A1 green and cleanup baseline verified.

Because the deadline policy has materially changed, a new run is scientifically distinct and is allowed under the standing authorization. Reuse the established self-owned client -> VPS path that already passed UDP preflight; do not retry the unchanged public/no-ingress path.

If the first warm run reaches all three admitted readiness responses and promotion correctly, continue with an interleaved **5 warm + 5 cold** batch on the same exact binary, concurrency 1, small payloads, within the standing time/traffic limits.

Preserve per run:

- exact commit + binary SHA-256 + compiler/target;
- path classification and endpoint ownership without publishing unnecessary addresses;
- negotiation/Noise/resume success;
- three readiness challenge request/response timestamps and admissions;
- readiness sequence duration and per-probe duration;
- failure decision and promotion timestamps;
- warm/cold class;
- first resumed application-data acceptance;
- uncertain/replayed/confirmed/missing/duplicate/conflict counts;
- authenticated logical `DeliveryAck` evidence;
- recovery latency raw samples and median/P95 only from retained successful/failed raw set;
- client/server exit status, CPU/RSS/FD/socket sample where available, cleanup.

If a new distinct correctness failure appears, preserve it and stop only the affected warm branch; do not rerun unchanged to chase a PASS.

### Follow-up C — changed-path exact-head five-minute periodic/resource row

**Dependency:** A2 green. Independent of B after cleanup.

Retry the established private self-owned TCP path because the setup-timeout contract has materially changed. Use the existing bounded profile unless the repaired CLI contract requires a truthfully recorded setup-timeout parameter:

```text
duration <= 300 s
concurrency = 1
60 records
32 B / record
interval = 5000 ms
ack_timeout = 1000 ms
application bytes <= 1920
```

Require authenticated setup before the first periodic record. Preserve attempted/confirmed/missing/duplicate counts, per-record confirmation latency, elapsed/application bytes, CPU user/system, max RSS, peak FD/sockets and cleanup. A setup success followed by an ACK timeout is a valid application-delivery failure and must not be conflated with handshake failure.

This remains a five-minute bounded resilience/resource observation, not production long-lived proof.

### Follow-up D — repair HY2 listener scope, then diagnose the UDP path with new capture coverage

**Dependency:** independent local safety repair first. Do not perform a new HY2 network attempt until the wildcard bind is removed.

1. Change the generated temporary HY2 server config to bind an explicit administrator-controlled address that is actually assigned on the VPS. Validate the address against read-only remote interface metadata before launch.
2. Add tests that reject wildcard / empty-host listen forms and ensure the generated config cannot contain `listen: :PORT`, `0.0.0.0:PORT` or equivalent wildcard forms.
3. Preserve the pinned v2.9.3 SHA-256 and all disposable TLS/auth/cleanup behavior. Do not read or reuse production Hysteria secrets/config.
4. If the desired lab address is not locally bindable, record `BLOCKED_ENVIRONMENT` and do not widen the listener or alter NAT/firewall/provider policy.
5. If explicit bind succeeds, one new short diagnostic attempt may be run because **capture coverage is a new variable**. Use a bounded capture restricted to the temporary HY2 UDP port on the self-owned VPS to distinguish:
   - no client datagrams arrive -> path/network environment blocker;
   - client datagrams arrive but server emits no usable response -> server/config/runtime diagnostic branch;
   - bidirectional QUIC handshake succeeds -> continue to the existing equal-application paired harness.
6. Store only bounded/redacted capture metadata or hashes needed for the conclusion; do not commit secrets or unnecessary endpoint details.
7. If the path becomes viable without network-policy changes, run the first interleaved **5 Nekomusume + 5 HY2** exact-payload pair and report raw samples, median/P95, failures, CPU, RSS, FD, application bytes; `wire_bytes` stays null unless capture provenance is trustworthy.

Do not make superiority claims from the first sample.

## Fallback / spare VPS opportunity

If B or C is temporarily blocked while its local repair is underway, continue the other independent branch. If both real-path branches are blocked and D remains environment-blocked, use the rented VPS window for one current-head release-engineering row that answers a new question rather than idle repetition:

- native x86_64 release build + binary/package SHA-256 provenance;
- dedicated-path install/smoke/readiness/shutdown/cleanup;
- current-head package rollback rehearsal only if the package/state contract changed materially since the previous N5 evidence;
- bounded process-resource sampling on a generic authenticated TCP/UDP exchange if current-head runtime changes make the sample scientifically distinct.

Do not repeat unchanged IPv6 failure, HY2 timeout, public no-ingress path or old benchmark solely for utilization.

## Completion gates

This batch is complete only when:

- D064 no longer uses a one-second whole-sequence deadline for three sequential mandatory readiness proofs, and the new bounded deadline semantics are documented/tested;
- periodic setup and per-record DeliveryAck timeouts are separate and behaviorally tested;
- full local repository gate passes for the repair commit;
- a changed-path D064 exact-head run is either positively sampled or retained as a new distinct blocker;
- a changed-path periodic exact-head row is either positively sampled or retained as a new distinct blocker;
- HY2 temporary listener generation no longer violates the explicit-address safety contract;
- any new HY2 retry uses changed capture/bind evidence and does not alter production network policy;
- all experiment failures remain preserved with exact commit/binary identity and cleanup;
- release-evidence/status documents are updated only for claims actually supported;
- `RELEASE_CANDIDATE=false`, global `FREEZE=false`, `PRODUCTION_READY=false`, `RELEASED=false` remain unchanged.

## Do not expand into

- Session/Carrier architecture redesign;
- reducing `k_ready` merely to make the VPS test pass;
- weakening authentication/admission/tuple binding;
- UDP+TCP striping/aggregation;
- reconnect persistence or durable Session store;
- production Hysteria/firewall/NAT/route/DNS/proxy/tunnel/qdisc changes;
- wildcard temporary listeners as a workaround for NAT;
- third-party targets or scans;
- retrying unchanged failed paths;
- RC/security/production approval or performance-superiority claims.

## Questions requiring maintainer decision

none.
