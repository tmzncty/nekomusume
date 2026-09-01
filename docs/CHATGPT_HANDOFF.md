# Nekomusume ChatGPT Handoff

Checked at: 2026-09-01 16:01 Asia/Shanghai
Reviewed implementation HEAD: `c7e0a211cbc74f065d03b374bd3cc1bbf2a97356`
Previous reviewed implementation HEAD: `ed08b644b6cc88ca2b322fd09b3f971d604f791c`
Previous reviewer handoff commit: `877e50f7a37144bc070377092056d79324aebb9b`

## What changed

Two substantive coding-agent commits landed after the previous reviewer handoff:

- `41eace3` — **implementation/test/evidence repair**. It separates explicit health events from measured `HealthSample` telemetry, uses an explicit D064-style `k_failure=3` profile in the automatic-failover runner, moves the final UDP->TCP switch decision into `CarrierManager`, uses stable reason `udp_path_degraded`, adds bounded post-handshake duplicate admission, and records a new bounded two-owned-host recovery row. This is real implementation and real socket evidence, not just documentation.
- `c7e0a21` — **implementation/tests plus historical-evidence preservation**. It adds a bounded one-TCP-connection periodic authenticated Session runner with encrypted Session `DeliveryAck`, confirmation latency/accounting, resource bounds, missing/duplicate-ACK tests and signal cleanup tests. Its five-minute VPS sample is explicitly historical from source commit `d5f0170`; the runner was re-ported and locally validated on current parent `41eace3`, but that five-minute sample was **not rerun on current HEAD**.

The repository is therefore making meaningful progress and is not waiting on an external authorization. N9 is already closed and the execution plan is now correctly in the **bounded release evidence matrix** phase.

The previous automatic-health repair fixed several real defects, but this review found two remaining D064 correctness/evidence defects in the exact current implementation, plus one long-session transport correctness defect that must be fixed before treating the periodic runner as current-head soak evidence.

## Review verdict

**CONTINUE WITH REQUIRED FIXES — application recovery is now real, but D064 active-path semantics and health-failure admission are still not truthful enough for a release-matrix PASS; the periodic runner needs a stateful TCP framing repair before current-head soak.**

The new two-host automatic row is useful and should be retained. It proves that, after an explicit bounded application-level UDP reply-cessation seam, the current family of code can recover the outstanding logical records over an authenticated TCP resume path and produce exact final application bytes. It does **not** yet prove the accepted D064 manager transition contract or its reported `new_active`/recovery timing semantics.

Do not discard or rewrite the existing positive/negative VPS rows. Repair the implementation, then run replacement rows on the exact repaired commit.

No maintainer decision is required. The findings below are implementation/spec-alignment issues inside already accepted architecture and standing authorization.

## Review findings

### R-001 HIGH — unauthenticated/unadmitted UDP traffic can advance the health-failure counter

In the current `failover-client` automatic-health loop, the receive result is first classified as `unexpected_peer`, `malformed_or_unadmitted`, or `authenticated_delivery_ack_timeout`, but the code then unconditionally records:

```text
HealthObservation::Failure(AuthenticatedDeliveryAckTimeout)
```

for all of those non-progress cases.

Consequences:

- wrong-peer traffic can accelerate the three-failure threshold;
- same-peer malformed or unauthenticated traffic can accelerate the threshold;
- a 100 ms socket polling timeout is currently treated as a complete failed health observation, so the observed failures in the VPS row occur at roughly 100/200/300 ms even though D064's accepted initial probe interval is 1 second;
- an authenticated datagram is currently treated as generic progress based on successful decryption before exact expected health/application semantics are established.

This violates the D064 boundary that the manager may convert only explicitly permitted observations into path state. Untrusted traffic may consume a bounded diagnostic/admission budget, but it must not create a health failure or health success.

### R-002 HIGH — `CarrierManager` promotes the TCP fallback before TCP is ready

`CarrierManager::fail_udp_to_tcp()` currently mutates `active` to the fallback path immediately when UDP reaches `Failed`. The process runner then asserts TCP is active **before it creates the TCP connection**. `tcp_active_at` is captured immediately after `TcpStream::connect_timeout`, still before canonical negotiation, Noise authentication, resume authentication/readiness and first resumed logical data.

D064 explicitly requires:

```text
new path starts standby
-> authenticated/validated/readiness gates
-> warm/eligible
-> manager promotion to active
```

and defines cold recovery as including candidate creation, handshake, validation, resume and first accepted logical data. A TCP connect event is not readiness and cannot make the fallback active.

The existing `41eace3` VPS row therefore remains valid as **bounded application recovery over TCP after controlled UDP degradation**, but its manager-active timestamp / D064 ownership-transition subclaim is not accepted as conformant release evidence yet.

### R-003 HIGH — periodic TCP framing loses partial-frame state across read timeouts

The new periodic runner sets finite TCP read timeouts and repeatedly calls the shared `read_frame()`. That helper uses `read_exact()` for a fresh four-byte header and then the payload on every call. If a real TCP stream delivers only part of the header or payload before a timeout, `read_exact()` may already have consumed bytes when it returns `TimedOut`/`WouldBlock`. The next loop call starts again as though no bytes were consumed, which can desynchronize the length framing.

The existing loopback periodic tests do not establish correctness under delayed fragmentation across poll timeouts. This matters specifically because the runner is intended to remain open for minutes on a real path.

Do not take a new current-head five-minute periodic sample until the framed receive path preserves partial header/payload bytes across polling timeouts or otherwise uses a bounded absolute-deadline frame accumulator.

### R-004 MEDIUM — repository status has not caught up with the new evidence

`docs/status.md` still describes the CLI/live-TCP boundary as lacking automatic threshold-driven degradation and does not link the new automatic-health or periodic evidence. `ROADMAP.md` correctly leaves UDP degradation/TCP fallback and long-lived stability unchecked.

Do not fix this by simply checking boxes now. Reconcile status only after R-001/R-002 replacement evidence and R-003 current-head periodic evidence exist.

## Evidence boundaries

### Accepted from `41eace3`

- The delayed exact duplicate negotiation/Noise response admission boundary is materially stronger than the previous single-datagram assumption.
- Health evidence no longer needs to invent RTT/loss/PTO values for an application acknowledgement timeout.
- The runner's explicit failure threshold is three and the stable manager reason is `udp_path_degraded`.
- `FailoverController` no longer independently chooses the fallback; it consumes a manager decision for its uncertain/replay/dedup bookkeeping.
- The bounded self-owned two-host repaired row delivered exactly 3 records / 96 application bytes after explicit UDP reply cessation, used authenticated ResumeGuard and exact DeliveryAck semantics, exited cleanly, and preserved the earlier public-address handshake timeout as negative evidence.
- The run is truthfully `cold`, not warm.

### Not accepted from `41eace3` yet

- Wrong-peer or malformed traffic must not count toward the three failure observations.
- A 100 ms socket poll timeout is not automatically one D064 failed probe/observation. The 1 s accepted default must be represented as an actual observation window unless a reviewed decision changes it.
- TCP is not D064-active immediately after the UDP failure decision or raw TCP connect.
- The `new_active` timestamp and D064 recovery interval cannot start/end at pre-readiness events.
- The row does not prove warm standby, natural WAN blackhole detection, public/general reachability, capacity, production readiness or release readiness.

### Accepted from `c7e0a21`

- A bounded single-TCP-connection periodic Session runner now exists.
- Its negotiation, Noise authentication, Session-runtime delivery accounting, encrypted DeliveryAck, missing/duplicate-ACK handling, bounds, and signal cleanup have deterministic current-tree tests.
- The historical five-minute execution is carefully provenance-separated and may remain as historical evidence for its exact source/binary: 60/60 confirmations, 1,920 application bytes, roughly 295 s, plus direct-child CPU/RSS/FD samples and cleanup.

### Not accepted from `c7e0a21` yet

- The historical five-minute row is not a current-head execution.
- The current periodic framed receive loop is not yet proven safe under partial TCP frames spanning polling timeouts.
- Therefore `ROADMAP.md` long-lived stability remains unchecked.

### Other release-matrix facts

- Existing B1 cross-host generic TCP/UDP IPv4 sanity remains valid.
- Existing B2 controlled endpoint-stop UDP->TCP resume remains valid but is a controlled application fault, not automatic/natural failure detection.
- Existing B3 lifecycle sample remains 7/8 successful with the failed UDP cycle preserved.
- IPv6 remains `BLOCKED_ENVIRONMENT`: the recorded owned endpoints did not have an actual end-to-end IPv6 route. Do not spend rental time retrying unchanged IPv6 probes unless the environment changes.
- The HY2 v2.9.3 artifact/setup and official TCP-forwarding comparison seam are already pinned. The remaining useful gap is an equal-application Nekomusume command/orchestrator plus the first real paired sample.
- No GitHub commit status or PR workflow run is attached to current HEAD through the available GitHub checks. Reported local full gates are coding-environment evidence, not independent CI attestation.
- `RELEASE_CANDIDATE=false`, global `FREEZE=false`, `PRODUCTION_READY=false`, and `RELEASED=false` remain correct. `CANONICAL_CORPUS_V1_FROZEN=true` is corpus-specific only.

## Work Package — repair release-matrix semantics, then immediately spend the VPS window on current-head evidence

Execute A -> B -> C -> D -> E in dependency order where applicable. This package is intentionally thick. Do not stop after a small helper if the next test/evidence step is already READY.

### Primary A — Close R-001/R-002/R-003 without changing protocol bytes unnecessarily

#### A1 — Make one health failure represent a truthful bounded observation window

For the D064 automatic-degradation release path:

- keep short socket read timeouts only as **internal polling quanta**;
- one `HealthObservation::Failure(...)` may be recorded only after the complete configured health/probe observation window expires without permitted progress;
- use the accepted D064 initial 1 s probe/observation interval for this release-evidence path unless an explicit reviewed decision changes it;
- wrong-peer traffic is ignored/diagnosed under a bounded count/rate policy and does not advance or reset health state;
- malformed, unauthenticated, stale, or otherwise unadmitted same-peer traffic is bounded/diagnostic only and does not advance or reset health state;
- authenticated progress must have the exact semantics permitted by the current health/application contract. “AEAD opened successfully” alone is not enough to reset health;
- deadline exhaustion, not the number of junk packets received, creates the failed observation.

The implementation may reuse the existing absolute-deadline admission pattern. Do not add sleeps that merely make the test pass.

Required deterministic tests:

1. many wrong-peer datagrams within the bounded admission policy cannot advance `consecutive_bad` or trigger failover;
2. bounded malformed same-peer datagrams cannot advance it;
3. one and two complete 1 s failed observation windows do not switch;
4. the third failed observation reaches `Failed` and creates only a pending manager failure/recovery transition;
5. exact authenticated permitted progress resets the failure counter per D064;
6. authenticated but stale/nonmatching application/control data cannot masquerade as health success;
7. no untrusted packet rate can make the transition occur earlier than the configured observation schedule.

#### A2 — Split UDP failure from TCP promotion; make D064 readiness own `active`

Refactor the smallest possible manager seam so a failed UDP path does **not** instantly make an unconnected TCP path active.

Required ownership sequence:

```text
UDP active
-> D064 failure decision for current generation
-> old UDP generation stops owning new Session data / enters the documented failure-or-drain boundary
-> TCP candidate remains standby/pending during connect
-> canonical negotiation
-> Noise authentication
-> authenticated resume/readiness validation bound to Session + generation + delivery epoch
-> required readiness policy satisfied
-> CarrierManager alone promotes TCP to active
-> uncertain ranges replay
-> first resumed logical data accepted
```

If the current repository already has a reusable authenticated readiness primitive, use it. If it does not, implement the **smallest D064-conformant bounded readiness seam** rather than treating TCP connect, successful write or Noise completion as readiness. Do not invent a second competing manager state machine.

D064's accepted initial `k_ready=3` contract remains the target for an `active`/`warm` claim. If the current slice cannot yet satisfy that contract, it is acceptable to preserve a narrower “authenticated cold recovery succeeded” evidence class, but then do **not** emit or store an `active` promotion until the readiness gate exists.

A rejected TCP negotiation/auth/resume/readiness attempt must be atomic with respect to fallback activation. It may leave the Session temporarily without an active path during cold recovery, but must not create a false TCP-active state or discard uncertain logical ranges.

Required deterministic tests:

- TCP connect alone cannot become active;
- Noise authentication alone cannot become active;
- failed negotiation/auth/resume/readiness never produces a TCP-active manager state;
- stale/wrong generation cannot mutate failure intent, readiness counters or active selection;
- the manager remains the sole selection owner;
- at most one active owner is exposed;
- promotion happens only after the accepted readiness gate;
- `udp_path_degraded` remains the stable switch reason for the explicit cessation scenario;
- uncertain replay/dedup/conflict behavior remains fail-closed and exactly-once at the Session-data boundary.

#### A3 — Correct recovery timestamps and classification

Replace the current ambiguous timing labels with actual monotonic stages, for example:

```text
failure_observation_started
failure_decided_at
tcp_connect_started
tcp_connected
tcp_negotiated
tcp_authenticated
resume_validated
readiness_satisfied
new_active_at
first_resumed_data_accepted
```

Only emit `new_active_at` after the manager promotion gate. For cold recovery, `recovery_latency` begins at the accepted failure decision and ends at the first accepted resumed logical data, while the component intervals remain separately inspectable.

Do not require synchronized host wall clocks; client-side monotonic intervals are sufficient.

#### A4 — Replace timeout-fragile TCP framing with a bounded stateful/deadline reader

Add a reusable framed receive mechanism that preserves partial TCP header and payload bytes across `WouldBlock`/`TimedOut` polling events. It must:

- retain 0–3 already-read length bytes;
- retain partial payload bytes;
- enforce the existing maximum frame length before allocating/continuing;
- use an absolute caller deadline, not an unbounded retry loop;
- distinguish complete frame, deadline, clean EOF-before-frame, and truncated partial frame at terminal EOF/deadline sufficiently for the caller to fail closed;
- never interpret leftover payload bytes as a new length header.

Use it at least in the periodic runner's handshake/data/DeliveryAck receive path. Reuse elsewhere only where it reduces duplicated timeout-framing bugs without turning this into a broad I/O rewrite.

Required deterministic tests must deliberately fragment:

- the 4-byte length header across multiple writes separated by delays longer than one poll timeout;
- the payload across multiple writes/timeouts;
- header + payload with multiple timeout gaps;
- oversized length;
- EOF/truncation after a partial header;
- EOF/truncation after a partial payload;
- delayed DeliveryAck that remains within the absolute ACK deadline.

A missing ACK must not leave partially consumed frame bytes that poison the next logical record. If the ACK deadline expires with an incomplete frame, fail that Session path terminally rather than pretending the next record can safely continue on a desynchronized stream.

#### A5 — Full local closure

Run at minimum:

- focused carrier-manager/health tests;
- focused failover process tests;
- focused periodic fragmented-frame and ACK-accounting tests;
- `cargo fmt --all -- --check`;
- `cargo check --workspace --locked`;
- `cargo test --workspace --all-targets --locked --no-fail-fast`;
- `cargo clippy --workspace --all-targets --locked -- -D warnings`;
- `bash scripts/check.sh`;
- `git diff --check`.

Run fuzz smoke only if wire/parser behavior changes or the repository gate requires it. A TCP framing state-machine change is transport/runtime behavior, not automatically a wire-format change.

**Primary A completion definition:** unauthenticated/junk traffic cannot accelerate health failure; one health failure reflects the accepted full observation window; TCP cannot become manager-active before D064 readiness; recovery timing names the real stages; periodic TCP framing survives partial frames across poll timeouts; all local gates pass.

### Follow-up B — Replacement exact-head automatic-degradation VPS row

**Dependency:** A1–A3 green; A4 may proceed independently but should be complete before the whole package closes.

Immediately use the rented self-owned VPS path under standing authorization. Run one small current-exact-commit application-level UDP reply-cessation -> D064 manager-driven cold TCP recovery row.

Record at minimum:

- experiment ID, commit, client/server binary SHA-256;
- owned-path classification and actual ports/bounds;
- real observation interval and the timestamps of all three failed observations;
- bounded ignored wrong-peer/malformed counts if any, explicitly showing they did not affect health state;
- UDP failure generation;
- TCP standby/connect/negotiation/auth/resume/readiness progression;
- exact moment of manager promotion and stable reason;
- cold classification and component/recovery intervals;
- confirmed/uncertain/replayed/duplicate/missing logical ranges from real Session state;
- final receiver records/bytes;
- CPU/RSS/FD/owned-socket samples where the existing sampler is applicable;
- client/server exit and cleanup.

Preserve the existing `primary-a-877e50f-vps-r2` row as historical application-recovery evidence. If the new row passes, explicitly supersede only its inaccurate pre-readiness manager/timing subclaim; do not delete the old row.

If the repaired current exact commit fails for a new cause, preserve it and do not rerun unchanged.

### Follow-up C — Current-head five-minute periodic Session row

**Dependency:** A4 green.

The historical 60-record five-minute row should now be replaced by a **current exact-head execution**, because the current runner has materially different framing/error semantics.

Use one genuine authenticated TCP Session for approximately five minutes under standing authorization, for example the existing bounded profile:

```text
duration: 300 s
interval: 5 s
count: 60
payload: 32 B
concurrency: 1
```

Keep the one-session/no-reconnect classification explicit. Run both roles through the existing process resource sampler when practical.

Record exact commit/binary hash, 60 attempted/confirmed/missing/duplicates, p50/P95 confirmation latency, elapsed time, CPU user/system, max RSS, peak FD/owned sockets, exit status and cleanup. A failure is useful evidence; do not mechanically rerun it without a changed hypothesis/instrumentation/code/path condition.

Only after this current-head row exists may `ROADMAP.md` consider a narrowly worded long-lived/stability checkbox update. Five minutes on one owned path is not production soak.

### Follow-up D — Unlock and run the first fair HY2 v2.9.3 paired application sample

**Dependency:** Primary A local gates green. This task is independent of whether Follow-up B exposes another path-specific automatic-failover failure.

The repository already has:

- pinned HY2 v2.9.3 artifact/version/hash;
- an official `tcpForwarding` seam note;
- the fair comparison result schema/methodology;
- a bounded process resource sampler.

Do **not** weaken the existing loopback-only guard in `scripts/bench/compare-hy2.sh`. Build a separate self-owned-VPS orchestrator or wrapper with a fail-closed owned-target contract.

First close the smallest remaining Nekomusume equality gap: the comparison command must consume the exact deterministic payload supplied by the workload contract rather than silently generating its own fixed `x` bytes. Prefer a narrow benchmark wrapper/CLI option that:

- reads only the explicitly supplied bounded payload file;
- enforces the existing 1–1200 B first-sample payload cap (or another already justified bounded contract);
- verifies/records exact application byte count and SHA-256;
- performs one authenticated encrypted Nekomusume application exchange over the owned client/VPS path;
- emits the comparison JSON fields required by the orchestrator without exposing keys/topology;
- does not become a generic file-transfer/proxy feature.

Then run a first paired sample, preferably 5 runs each in a close time window, with the same owned client/VPS, route metadata, MTU metadata, exact payload/hash, one-stream load, run count and timeout. For HY2 use only disposable experiment config/certificate/auth and fresh high ports; use its temporary client `tcpForwarding` to a temporary VPS-loopback echo target. Never read/reuse production Hysteria credentials, stop/reconfigure the existing Hysteria service, or use port hopping/Mimic.

Report raw samples plus median/P95/failures, CPU user/system, RSS, FD and application bytes. Keep `wire_bytes=null` unless a bounded trusted capture is deliberately collected. The first result is comparison evidence, not a superiority claim.

### Follow-up E — Reconcile the release-evidence ledger only after replacement evidence

**Dependency:** B and/or C/D produce exact current evidence.

Update `docs/status.md`, `ROADMAP.md`, `IMPLEMENTATION_PLAN.md` only to the level actually proved.

Possible narrow promotions after successful replacement rows:

- `UDP degradation / TCP fallback`: only if the corrected manager/readiness path passes the controlled self-owned row; describe it as explicit application-level degradation, not arbitrary natural Internet blackhole proof.
- `long-lived stability`: only if the current-head single-Session periodic row passes; state the exact five-minute/one-path boundary.
- `HY2 comparison`: only after an actual semantically equal paired sample; keep any performance conclusion bounded to that workload and time/path.

Keep NAT/endpoint-change unchecked until a genuine owned endpoint change is exercised. Keep IPv6 blocked until the environment actually has an end-to-end route.

## VPS opportunity after this package

The rental window still has valuable independent work after B/C/D. The next reviewer should prefer, in order, whichever has become READY and is not already evidenced:

1. real carrier recovery / migration-back with validated current generation;
2. real-session key update rather than the current socket-free fixture;
3. owned endpoint/source-port change if it can be produced without production route/firewall/qdisc changes;
4. bounded real-path PMTUD observation if current instrumentation can distinguish packetization evidence truthfully;
5. changed-instrumentation diagnosis of the preserved public-address UDP handshake failure (for example bounded endpoint capture), but only if it answers a concrete path question and does not repeat the old run unchanged.

Do not spend VPS rental time on ICMP/Raw-IP/SCTP/DCCP/GRE/ESP experiments that remain outside standing authorization or lack an observed release problem.

## Fallback

If A2 reveals that the current code has no executable D064 readiness primitive and implementing it safely is larger than one coherent slice:

- preserve the existing cold-recovery evidence without calling the TCP path manager-active;
- make the smallest authenticated Session/generation/delivery-epoch readiness primitive the next required implementation slice;
- continue A4 and the HY2 exact-payload local unlock in parallel only where they do not depend on the broken manager path;
- do not invent readiness from connect/write/Noise success.

If A4 exposes a broader shared TCP framing bug in other long-running commands, repair the shared stateful reader and migrate only the affected timeout-driven callers in one bounded commit; do not perform an unrelated async-runtime rewrite.

If the public-address path remains UDP-unreachable while the private owned path works, preserve that split as path evidence. Standing authorization permits bounded capture/diagnosis on owned endpoints; it does not authorize firewall/routing changes to force a PASS.

## Completion gates

- Untrusted UDP traffic cannot advance or reset health failure state.
- One D064 failed observation represents the configured full observation window; 100 ms polling is not counted as a probe failure by itself.
- Three real failed observations, not three arbitrary received/timeout events, trigger the accepted failure threshold.
- TCP does not become manager-active before authenticated readiness and generation gates.
- Failed TCP negotiation/auth/resume/readiness never creates a false active path.
- Recovery timestamps correspond to real semantic stages and cold recovery ends at first accepted resumed logical data.
- Partial TCP frames survive poll timeouts without header/payload desynchronization.
- Full local repository gate passes after repairs.
- At least one exact-current-head replacement VPS row is attempted after the meaningful repair, with negative evidence retained if it fails.
- Current-head five-minute periodic evidence is collected only after framing correctness is repaired.
- HY2 paired comparison uses equal application semantics and disposable experimental configuration; no production service/credential is touched.
- Status/roadmap promotions do not exceed the evidence actually collected.
- `RELEASE_CANDIDATE=false`, global `FREEZE=false`, `PRODUCTION_READY=false`, `RELEASED=false` remain unchanged pending independent release/security review and later RC decision.

## Do not expand into

- production deployment or public service exposure;
- changing production route/firewall/DNS/proxy/tunnel/qdisc;
- third-party targets or scanning;
- >10-minute single experiments, >256 MiB application traffic or >32 sessions without new authorization;
- fabricating warm readiness from TCP connect/Noise completion;
- natural-WAN/general-reachability claims from controlled application reply cessation;
- protocol byte changes merely to simplify the runner;
- 0-RTT, enabled FEC, striping/aggregation, ICMP/Raw-IP or exotic carriers without a new observed-problem gate;
- one-off HY2 superiority claims.

## Questions requiring maintainer decision

none.
