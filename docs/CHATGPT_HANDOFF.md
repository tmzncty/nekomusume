# Nekomusume ChatGPT Handoff

Checked at: 2026-09-02 04:59 Asia/Shanghai
Repository HEAD: `a9a96b16050e38de158033de5e9bb4406414df58`
Previous reviewed implementation HEAD: `83f0a0720aa77cc7b811bac4ad6083acd6fe2488`
Previous reviewer handoff commit: `244534bc870e45ef6f66d7d65797301ff4b30e22`

## What changed

Two substantive coding-agent repair commits landed after the previous reviewer handoff:

- `95aa7e3` — **runtime/spec/test correctness repair.** It separates D064 readiness into a bounded one-second per-probe timeout plus three-second whole-sequence timeout, additionally capped by the experiment deadline; it also separates periodic TCP Session setup/negotiation/Noise timeout from the per-record `DeliveryAck` timeout, with a 5000 ms default setup budget and 10000 ms maximum. The implementation adds behavior tests rather than only parsing/configuration checks and updates release-plan/status wording to the new bounded semantics.
- `a9a96b1` — **HY2 owned-lab safety repair.** The owned-lab comparator now requires a distinct `LAB_REMOTE_BIND_ADDRESS`, rejects wildcard/unspecified/loopback/multicast/broadcast forms, verifies by read-only `ip -j address show` over SSH that the exact address is actually assigned on the VPS, emits an explicit-address HY2 server/client config, and adds fail-closed tests for unsafe/nonlocal bind requests. It does not weaken the pinned HY2 v2.9.3 identity, disposable TLS/auth, port bounds or cleanup contract.

GitHub Actions `Rust CI` run 106 for exact HEAD `a9a96b1` completed successfully. The `stable checks` job passed `bash scripts/check.sh`; the nightly job built the pinned fuzz target and completed the bounded 30-second `decode` fuzz smoke successfully.

The local blockers identified in the previous handoff are therefore repaired. The rented VPS is now the highest-value READY resource: changed code makes the D064 and periodic retries scientifically distinct, and changed bind/capture coverage makes one HY2 diagnostic retry scientifically distinct. Do not spend the next cycle on unrelated local polish.

## Review verdict

**SAFE TO CONTINUE — local repair gate is green; immediately harvest exact-head VPS evidence.**

No core Session/Carrier architecture change is requested. Preserve the current fail-closed contracts and use the standing authorization directly. The next coding-agent batch should build/deploy exact `a9a96b1`, run the changed-path D064 and periodic rows, then perform the explicit-bind HY2 diagnostic and first fair pair only if the path becomes viable without network-policy changes.

If any branch finds a new correctness defect, preserve the exact negative row and stop only that branch. Continue the other independent READY VPS branch rather than waiting.

## Review findings

### R-306 PASS — D064 readiness deadline semantics now match the three-observation contract

The previous real path showed two successful authenticated readiness round trips consuming about 819 ms total, making the old one-second whole-sequence budget structurally incompatible with three mandatory sequential proofs. `95aa7e3` now preserves `k_ready=3` while applying a one-second per-observation bound and a three-second whole-sequence bound, both subordinate to remaining experiment duration. Client and server use the same bounded model and the fail-closed readiness/admission/promotion boundaries remain in place.

This is a runtime-policy repair, not evidence that warm recovery now works on the VPS. A changed-path exact-head run is still required.

### R-307 PASS — periodic setup and application ACK deadlines are now separate contracts

`95aa7e3` no longer uses the per-record `ack_timeout` as the connect + canonical negotiation + Noise handshake deadline. The default bounded setup budget is 5000 ms (maximum 10000 ms), while the existing per-record acknowledgement timeout still governs post-establishment delivery confirmation.

This directly addresses the prior cross-host row where the server authenticated but the client failed before entering the periodic application loop. It does not itself prove five-minute stability.

### R-308 PASS — HY2 comparator no longer requires or generates a wildcard listener

`a9a96b1` requires an explicit, assigned, non-wildcard owned-lab bind address and verifies it from read-only remote interface metadata before launch. The generated temporary HY2 server config uses that exact address; unsafe/nonlocal addresses fail closed. This closes the previous safety blocker.

A remotely assigned address is not automatically a viable client path. The next diagnostic must still distinguish `no packets arrive`, `packets arrive but no usable response`, and `bidirectional QUIC succeeds`. Do not widen firewall/NAT/provider policy to force success.

### R-309 PASS — exact-head CI attestation exists for the repair state

For `a9a96b1`, GitHub Actions run 106 passed both the stable repository gate and the bounded nightly decode fuzz smoke. This is independent CI evidence for the checked-in repair state. It is not VPS/WAN evidence.

## Evidence boundaries

- `IMPLEMENTATION_COMPLETE=true` remains the bounded research-baseline status.
- `CANONICAL_CORPUS_V1_FROZEN=true` remains corpus-specific.
- `RELEASE_CANDIDATE=false`, global `FREEZE=false`, `PRODUCTION_READY=false`, `RELEASED=false` remain correct.
- The prior D064 negative row at `df61091d` remains immutable evidence: UDP delivery proof and two readiness admissions succeeded; challenge 3 failed; no warm promotion or resumed application data was proven.
- The prior periodic row at `df61091d` remains immutable evidence: server authentication succeeded, client setup failed, actual application bytes were zero.
- The prior HY2 row remains immutable evidence: pinned HY2 v2.9.3 temporary QUIC/UDP path timed out before a forwarding listener became usable; no comparative sample exists.
- `95aa7e3` and `a9a96b1` create legitimate changed-code/configuration variables for new bounded retries. They do not erase the negative rows.
- IPv6 remains environment-blocked unless the current owned endpoints actually expose a usable tested IPv6 path. Do not repeat the unchanged failure just to fill the matrix.
- Standing authorization covers the self-owned TCP/UDP runs, bounded capture, resource sampling, temporary HY2 lab service and cleanup described below. No per-run maintainer approval is needed.
- Production route/firewall/DNS/proxy/tunnel/qdisc changes, third-party targets and authorization-boundary expansion remain prohibited.

## Work Package — exact-head rented-VPS evidence harvest

Execute A -> B/C -> D in dependency order. B and C are independent after A and cleanup; D is independent after A and its explicit-bind preflight. This is intentionally a thick batch. Do not wait for another reviewer between successful sub-slices.

### Primary A — exact-head release build, provenance and clean deployment baseline

**Goal:** make every new VPS row traceable to the exact reviewed repair state before collecting evidence.

1. Fetch/pull current GitHub `main` and require exact implementation HEAD `a9a96b16050e38de158033de5e9bb4406414df58` or a later direct descendant containing no unreviewed behavior change relevant to these scenarios. If a later coding commit exists, record it and do not silently call it `a9a96b1` evidence.
2. Use a fresh detached/clean worktree; do not read/copy/commit protected identity material.
3. Build the release binary natively for `x86_64-unknown-linux-gnu`; record git/tree identity, rustc/Cargo versions, target, binary size and SHA-256.
4. Deploy the exact same binary/hash to the known self-owned client and VPS endpoints used by the successful established-path preflight. Do not rediscover the public no-ingress path and do not scan candidate networks.
5. Verify pre-run cleanup: no experiment process/listener/temp directory remains on intended high ports.
6. Run one minimal authenticated TCP/UDP established-path sanity only if required to prove the deployment/path is still viable; do not repeat already-proven baselines unnecessarily.

**Gate:** if exact binary identity or cleanup cannot be proven, do not start B/C/D. Preserve the blocker and repair provenance/cleanup first.

### Follow-up B — changed-code exact-head D064 warm/cold evidence

**Dependency:** A green.

Use the same established self-owned cross-host path that previously passed UDP preflight. The timeout policy is materially changed, so one new warm run is valid.

#### B1 warm preflight

Run one bounded warm D064 sample with the existing small workload/profile and fresh high ports. Require:

- canonical UDP negotiation + Noise authentication;
- authenticated logical delivery proof before controlled UDP reply cessation;
- TCP negotiation + Noise + resume validation;
- exactly three authenticated readiness challenges with `admitted=true`;
- per-probe and whole-sequence timing evidence;
- no TCP application data before atomic promotion;
- first resumed application data only after readiness/promotion;
- uncertain/replayed/confirmed/missing/duplicate/conflict accounting;
- authenticated logical `DeliveryAck` evidence;
- client/server exit status and cleanup.

If B1 fails with a **new** defect, preserve it and stop the warm branch. Do not rerun unchanged.

#### B2 interleaved sample

Only if B1 proves correct warm promotion and resumed delivery, run an interleaved **5 warm + 5 cold** batch on the same exact binary, concurrency 1, small payloads and within standing time/traffic limits.

Preserve raw per-run samples. Report recovery-latency median/P95/failure count only from the retained raw set; do not turn this controlled application-level UDP reply-cessation experiment into natural Internet blackhole evidence.

Also retain CPU/RSS/FD/socket samples when the existing sampler can collect them without perturbing the semantics.

### Follow-up C — changed-code five-minute periodic/resource observation

**Dependency:** A green; independent of B after cleanup.

Retry the established private self-owned TCP path with the repaired setup contract:

```text
duration <= 300 s
concurrency = 1
records = 60
application bytes = 32 / record
interval = 5000 ms
setup_timeout = 5000 ms
ack_timeout = 1000 ms
expected total application bytes <= 1920
```

Require authenticated setup before record 1. Preserve:

- setup duration and whether it fits the setup deadline;
- attempted/confirmed/missing-after-attempt/duplicate/conflict counts;
- per-record confirmation latency and raw timing samples;
- actual application bytes, not only configured workload annotation;
- CPU user/system, max RSS, peak FD, peak owned sockets, sample count;
- client/server exit status and exact binary identity;
- final listener/process/socket/temp cleanup.

If setup succeeds but a later record exceeds the 1000 ms ACK deadline, keep that as an application-delivery failure; do not relabel it as handshake failure. If the full row succeeds, classify it only as a five-minute bounded cross-host resilience/resource observation, not production long-lived proof.

### Follow-up D — explicit-bind HY2 diagnosis, then first fair pair if viable

**Dependency:** A green plus `a9a96b1` explicit-bind validation. Independent of B/C after cleanup.

1. Read only current VPS interface metadata (`ip -j address show`) and choose a concrete administrator-controlled address actually assigned to the VPS that is reachable over the intended self-owned comparison path. Prefer a path that both implementations can use under the same route/MTU/security/load contract. Do not modify routing/firewall/NAT/provider policy.
2. Set `LAB_REMOTE_BIND_ADDRESS` separately from the SSH/connection address as required. Run the adapter validation first. If no suitable assigned/reachable address exists, record `BLOCKED_ENVIRONMENT` and stop this branch.
3. If validation succeeds, make one new short HY2 diagnostic attempt with a bounded capture restricted to the temporary HY2 UDP port. This retry is valid because bind scope and capture coverage are new variables.
4. Classify the result exactly:
   - **no client datagrams arrive:** path/environment blocker;
   - **client datagrams arrive, no usable server response:** server/config/runtime diagnostic branch;
   - **bidirectional QUIC/TLS/auth succeeds and forwarding listener is ready:** proceed to the fair-pair harness.
5. If the path is viable without policy changes, run the first interleaved **5 Nekomusume + 5 HY2** equal-application pair using the existing exact-payload contract and pinned HY2 v2.9.3 artifact. Keep route/time-window/MTU/security/load/payload as equal as the harness requires.
6. Preserve raw samples, median/P95/failures, CPU user/system, RSS, FD and application bytes. `wire_bytes` remains null unless bounded capture provenance is sufficiently trustworthy for both sides.
7. Make no superiority claim from the first pair, even if one side wins every sample.

The temporary HY2 process/config/certificate/auth material must be experiment-local and cleaned; do not read/reuse `/etc/hysteria/server.yaml` secrets or disturb the existing Hysteria service.

### Follow-up E — evidence reconciliation and matrix update

**Dependency:** complete after each B/C/D branch reaches a retained positive or negative terminal row.

For every run:

- preserve exact experiment ID, commit/tree/binary SHA-256, actual parameters, start/end time, endpoint ownership/path classification, client/server results, resource metrics where available and cleanup;
- keep previous negative rows immutable rather than overwriting them;
- update only the relevant release-evidence/status rows actually changed by new evidence;
- distinguish `authorized execution`, `self-owned cross-host evidence`, `public-WAN/general reachability`, `release evidence`, `security approval` and `production readiness`;
- keep `RELEASE_CANDIDATE=false`, global `FREEZE=false`, `PRODUCTION_READY=false`, `RELEASED=false`.

Run `git diff --check` and the normal repository gate for any tracked code/script changes. Pure evidence-note updates still require link/status consistency checks already included by repository policy.

## Fallback / spare VPS opportunity

If one branch is blocked, continue the other independent READY VPS branches. If B, C and D all become blocked after retaining distinct evidence, use the rented window only for a current-head question that has changed since older evidence, in this order:

1. current-head native release/package SHA-256 + dedicated-path install/smoke/readiness/shutdown/cleanup;
2. bounded process-resource sampling on a generic authenticated TCP/UDP exchange if current runtime changes make the sample scientifically distinct;
3. repeated native microbenchmark only with an explicit warm-up/sample protocol and no concurrent CPU-heavy build/fuzz;
4. parser/property/fuzz only if relevant parser/wire code changed.

Do not repeat unchanged IPv6 failure, old public no-ingress path, old HY2 timeout or an unchanged benchmark merely to keep the VPS busy.

## Completion gates

This batch is complete when:

- exact current-head binary provenance and cleanup baseline are proven;
- the changed D064 timeout policy has one retained exact-head VPS outcome; if warm succeeds, the 5+5 warm/cold raw batch is retained;
- the separated periodic setup/ACK policy has one retained exact-head five-minute VPS outcome;
- HY2 explicit-bind validation is exercised against real VPS interface metadata, and one changed-capture diagnostic outcome is retained; if viable, the first 5+5 fair pair is retained;
- every positive/negative row preserves exact identity, parameters, evidence boundary and cleanup;
- no unchanged failed scenario was mechanically rerun;
- no production network policy or third-party target was touched;
- evidence/status docs reflect only facts actually observed;
- `RELEASE_CANDIDATE=false`, global `FREEZE=false`, `PRODUCTION_READY=false`, `RELEASED=false` remain unchanged.

## Do not expand into

- Session/Carrier architecture redesign;
- reducing `k_ready` merely to make D064 pass;
- weakening authentication, admission or exact tuple/generation/epoch binding;
- UDP+TCP striping/aggregation;
- reconnect persistence/durable Session store;
- production Hysteria/firewall/NAT/route/provider changes;
- third-party targets/scanning;
- repeated unchanged IPv6/public-path/HY2 failures;
- performance-superiority claims from one comparison batch;
- unrelated experimental carriers or speculative features.

## Questions requiring maintainer decision

none.
