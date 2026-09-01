# Nekomusume ChatGPT Handoff

Checked at: 2026-09-01 13:02 Asia/Shanghai
Reviewed implementation HEAD: `ecb8729a01761cb62ee889fa17e6c50790006d4f`
Previous reviewed implementation HEAD: `b191dd8181e3f6023eb4c1c43c43e5fd1ff0518c`
Previous reviewer handoff commit: `e7390312e0ca62a088e11fb7c2a6f5060cddcaea`

## What changed

One substantive coding-agent commit landed after the previous reviewer handoff:

- `ecb8729` — **runtime correctness repair + deterministic tests + replacement self-owned VPS evidence + status reconciliation**.

The commit closes all four parts of the previous Primary A:

1. `scripts/bench/process-resource-sampler.py` now returns a stable CPU tuple `(None, None)` when `/proc/<pid>/stat` disappears instead of leaving the caller with an index-unsafe nullable container. The regression imports `read_proc()` directly, uses a guaranteed-missing PID, and proves CPU/RSS/FD/socket values remain null rather than invented zero.
2. The generic UDP server no longer treats the 100 ms socket poll interval as the whole authenticated application wait. `recv_udp_until()` keeps short polling for shutdown responsiveness, tolerates `WouldBlock`/`TimedOut` before an explicit bounded application deadline, distinguishes shutdown/deadline/datagram, and still fails terminally on other socket errors.
3. Process tests prove authenticated UDP application data delayed 250 ms succeeds while a delay beyond a one-second configured application deadline fails bounded with `data timeout`.
4. Replacement real-socket evidence records a 14/14 alternating TCP/UDP self-owned cross-host IPv4 lifecycle sample after the code/instrumentation change, preserves the prior 7/8 result as historical negative evidence, and updates `docs/status.md` without promoting public/general reachability or production status.

Independent GitHub Actions evidence is green at the exact implementation HEAD: Rust CI run #83 completed successfully for `ecb8729`.

## Review verdict

**SAFE TO CONTINUE — previous Primary A accepted; advance immediately into the bounded release evidence matrix.**

The prior sampler race and UDP 100 ms application-timeout mismatch are closed by code, deterministic regression tests, replacement VPS evidence, and green exact-head CI. No new correctness/security blocker was found in this review.

The repository is therefore not waiting on N9 or ordinary WAN permission. `IMPLEMENTATION_PLAN.md` now correctly shows N9 and negotiation-path completion complete; the first unchecked release-engineering item is the bounded release evidence matrix.

The highest-value next work is the project-defining gap that remains visible in `ROADMAP.md`: controlled-stop resume exists, but **health/degradation-driven UDP -> TCP failover does not yet have real-socket evidence**. The rented VPS should be used immediately once the local adapter is green.

`RELEASE_CANDIDATE=false`, global `FREEZE=false`, `PRODUCTION_READY=false`, and `RELEASED=false` remain correct. `CANONICAL_CORPUS_V1_FROZEN=true` remains a corpus-specific fact only.

## Evidence boundaries

### Accepted from `ecb8729`

- The previous B3 7/8 lifecycle result remains valid negative historical evidence and was not overwritten.
- The 100 ms UDP server poll interval was a real code/experiment-contract mismatch for post-authenticated application data; the server now enforces a distinct bounded application-stage deadline.
- A >100 ms but < configured-duration authenticated application delay is now proven to succeed locally.
- A delay beyond the configured overall application deadline is proven to fail bounded rather than wait indefinitely.
- The replacement self-owned cross-host sample completed 14/14 alternating TCP/UDP cycles on the selected owned IPv4 path after the repair.
- `live-udp` may truthfully cite bounded self-owned cross-host negotiated/authenticated UDP evidence; this is not public/general reachability.
- `live-tcp` may truthfully cite the existing controlled-stop authenticated DeliveryAck resume evidence; this is not natural/automatic degradation failover.
- Process-resource sampling remains **direct-child scoped**, not cgroup/descendant/host capacity evidence.
- Exact-head Rust CI #83 is independently green.

### Still missing / explicitly not promoted

- `ROADMAP.md` real `UDP degradation / TCP fallback` remains unchecked. `controlled_udp_stop` is an application fault injection and does not prove automatic health/PTO-driven failover.
- No sustained/long-lived real authenticated socket session is yet proven beyond the short lifecycle exchanges.
- IPv6 remains `BLOCKED_ENVIRONMENT` for the currently owned end-to-end path; do not manufacture a software PASS from historical IPv6 rows.
- NAT/endpoint-change evidence remains absent unless an owned environment can produce a genuine source endpoint change without modifying production routing.
- HY2 v2.9.3 is pinned and the forwarding comparison seam is documented, but no fair paired Nekomusume/HY2 self-owned-VPS performance sample exists yet.
- Resource samples are bounded observations, not capacity/stress conclusions.
- Independent release/security review remains a later gate.

### Architecture constraint for the next failover slice

`docs/adr/m3-concurrent-carrier-semantics.md` is the design contract. Preserve these facts:

- Session owns logical delivery; carriers expose observations; the Carrier Manager owns active-path selection.
- UDP is primary; TCP fallback may be prepared/warm but must not receive new application data while UDP is active.
- one probe miss must not switch paths;
- initial policy uses bounded staged failure evidence (`k_failure=3` in the ADR) and explicit stable reason codes;
- packet/socket observations must not become Session delivery evidence;
- old active data without logical delivery proof becomes `UNCERTAIN` and is replayed/deduplicated by Session identity/stream/offset semantics;
- a failed generation must not silently re-enter as active;
- striping/aggregation remains disabled.

The repository contains multiple historical state helpers (`CarrierState`, `CarrierHealthEvidence`, `CarrierManager`, `FailoverController`). Do not create a fourth independent timeout controller. Reuse the current accepted manager/health vocabulary and keep any adapter narrow and explicitly documented.

## Work Package — automatic health failover first, then spend the VPS rental window on resilience and comparison evidence

Execute A -> B -> C -> D in dependency order. This package is intentionally thick: the coding agent has demonstrated that it can close a full correctness/evidence batch within one cycle, so do not stop after one helper or one unit test if the next same-gate evidence step is READY.

### Primary A — Drive real failover from bounded carrier-health evidence and take VPS evidence immediately

**Goal**

Replace the current unconditional `controlled_udp_stop` decision path with a second, explicitly separate **health-driven experimental path** in which bounded real-socket UDP delivery/probe failures are converted into the existing carrier-health state, the manager decides the switch, and the same logical Session resumes over TCP with truthful uncertain-range replay/dedup evidence.

Do **not** remove the existing controlled-stop fixture; it remains useful deterministic evidence. Add a distinct path so the two mechanisms cannot be confused in reports.

#### A1 — Define the real-socket observation -> health adapter without inventing a new state machine

Use the accepted M3 ADR and current carrier health types.

Required behavior:

- treat a successfully authenticated/negotiated UDP application exchange or authenticated readiness response as progress/healthy observation only in the health domain; it is not Session-delivery proof unless a real encrypted Session DeliveryAck is separately validated;
- treat a bounded missed authenticated application/readiness response as a health/probe miss, not immediate peer-closed proof;
- one miss must not switch paths;
- require the repository's documented staged failure threshold before the manager can declare degradation/failure eligible for failover; use the accepted `k_failure`/`HealthLimits` vocabulary rather than adding a new magic `N`;
- retain exact transition evidence (`healthy -> degraded -> failed` or the repository-equivalent staged sequence), path/generation, sample/reason and monotonic relative timestamps;
- do not infer RTT/loss values that were not measured. If the real runner only truthfully observes bounded timeout/PTO-like misses, map only that observable field and leave unrelated metrics at a documented neutral/measured value through one explicit adapter contract;
- the active-path decision must flow through the existing manager/health contract, not `if recv_timeout { connect_tcp(); }`.

If the present `CarrierManager`/`CarrierHealthEvidence` APIs cannot represent one required transition without violating D064, add the smallest adapter/API needed and document why. Do not wholesale redesign carrier management.

#### A2 — Make TCP a bounded warm fallback for this experimental path

The M3 accepted contract is single-active / multi-ready. For the health-driven runner:

- establish TCP fallback in advance or otherwise prove it meets the repository's current warm/readiness contract before the UDP failure decision;
- canonical version negotiation and fresh Noise authentication remain required;
- bind readiness/resume to Session identity/generation/delivery epoch as current APIs permit;
- warm TCP may carry readiness/control/resume only, not new application data before promotion;
- keep resources bounded and close unused/failed fallback state deterministically.

If the current executable path can only provide cold fallback without a larger redesign, do not fake `warm`. Record the exact limitation, keep the cold row truthful, and implement only the smallest prerequisite that the accepted ADR already requires for warm readiness.

#### A3 — Use an application-level self-owned degradation injection inside standing authorization

Do not modify VPS firewall, route, qdisc, production tunnel or existing HY2 service.

Use a deterministic experimental server seam such as:

- after a documented logical record / authenticated readiness point, stop sending UDP application/readiness responses while keeping the UDP socket/process alive and keeping TCP fallback available;
- continue accepting enough bounded UDP input to distinguish blackhole/degraded behavior from process death where useful;
- never expose the seam as an unbounded service mode.

The fault injection must produce structured metadata identifying that it is a **controlled self-owned application-level degradation**, not an arbitrary Internet/network blackhole.

#### A4 — Preserve Session delivery safety across the manager-driven switch

At the switch boundary:

- the last UDP logical range without validated encrypted Session DeliveryAck becomes `UNCERTAIN`;
- do not convert UDP send success, packet receipt, timeout, TCP connect, or health state into delivery confirmation;
- resume TCP under the existing ResumeGuard / negotiation binding;
- replay the uncertain range plus later queued ranges according to the Session contract;
- receiver dedup must yield exactly-once final logical application bytes for exact duplicates where the current Session model supports that claim;
- conflicting duplicate bytes at one logical identity fail closed;
- report confirmed / uncertain / replayed / duplicate / missing records or bytes from actual Session state, not reconstructed guesses.

#### A5 — Deterministic tests before VPS

At minimum prove:

1. one UDP health miss does not switch;
2. misses below the documented failure threshold do not switch;
3. threshold-crossing observations generate the expected health transition and stable switch reason (`udp_path_degraded` or `udp_blackhole`, whichever the injection actually models);
4. active ownership remains single-valued; warm TCP carries no new application data before promotion;
5. the manager-driven path switches without invoking the old unconditional controlled-stop branch;
6. the uncertain logical range is replayed and exact duplicate reception remains idempotent;
7. stale/wrong generation cannot mutate active state;
8. successful UDP progress resets/recovers the bounded failure counter according to the accepted health model;
9. all timers/counters/resources remain bounded.

Run targeted tests, `bash scripts/check.sh`, and `git diff --check`. Run fuzz smoke only if parser/wire behavior changes.

#### A6 — VPS opportunity immediately after local green

Run one bounded self-owned cross-host IPv4 health-driven degradation -> TCP resume experiment under standing authorization.

Record at least:

- experiment ID;
- exact commit and release binary SHA-256;
- endpoint ownership classification and path class, without committing unnecessary addresses/secrets;
- UDP/TCP ports and actual bounds;
- fault injection logical point;
- health samples/transitions and final reason code;
- failure_decided / TCP-active / first accepted resumed-data timestamps and recovery latency;
- whether TCP was genuinely warm or cold before the decision;
- Session confirmed/uncertain/replayed/duplicate/missing records or bytes;
- final application records/bytes observed at the receiver;
- client/server exit status;
- CPU/RSS/FD/owned-socket observations using the repaired sampler where useful;
- cleanup verification.

Preserve any failure as evidence. Do not immediately rerun an unchanged failed scenario merely to obtain PASS.

**Primary A completion definition**

A deterministic manager/health-driven path exists; threshold semantics and Session replay safety are tested; one bounded real self-owned VPS row is recorded after local green; cleanup passes; claims remain controlled-degradation evidence only.

### Follow-up B — Build one real authenticated bounded periodic-session runner and take a 5-minute VPS sample

**Dependency:** Primary A green, or A is blocked only by a specific manager-integration defect while generic negotiated/authenticated TCP/UDP remains green.

The current `workload` command is in-process and the generic probe is short. The rental-window backlog still needs a genuine longer-lived real socket observation.

Build the smallest experimental runner that reuses the real generic negotiation + Noise path rather than inventing another protocol surface.

Required contract:

- transport: TCP or UDP; prefer UDP for the first VPS sample;
- self-owned endpoints only;
- total duration explicit and bounded `1..600 s`;
- periodic authenticated application echo at a bounded interval, not a busy loop;
- payload/count/total application bytes bounded below standing limits;
- same authenticated Session remains open for the run; do not simulate longevity by repeatedly restarting short probes;
- structured stage/counter output: attempted exchanges, successful exchanges, failures/timeouts, application bytes, last success stage, start/end/elapsed;
- process sampler integration;
- graceful signal/normal completion cleanup;
- no forwarding/proxy/tunnel behavior.

Local/process tests must cover duration bounds, interval bounds, byte accounting, successful multi-interval exchange, bounded peer silence/failure, and cleanup.

**VPS opportunity:** run exactly one 5-minute self-owned IPv4 UDP periodic authenticated session. Record exact identity, interval/payload, attempts/successes/failures, CPU/RSS/FD/socket metrics, application bytes, timestamps and cleanup. This is resilience evidence, not capacity or production uptime. Do not concatenate additional runs to imitate an unauthorized >10-minute soak.

### Follow-up C — Produce the first fair HY2 v2.9.3 paired self-owned-VPS application sample

**Dependency:** repaired sampler green and generic Nekomusume TCP sanity green. Do not wait until the end of the VPS rental month.

Use the existing repository facts:

- pinned Hysteria2 v2.9.3 artifact/commit/SHA-256 in `docs/bench/hy2-vps-setup-20260830.md`;
- workload/result methodology in `docs/bench/hy2-comparison-workload.md`;
- official-forwarding research seam in `docs/research/hy2-forwarding-comparison-note-20260901.md`.

Do not reuse or read production HY2 credentials. Do not stop/reconfigure the existing production Hysteria process. Use a temporary high UDP port, experiment-generated cert/auth/config, disposable paths and explicit cleanup.

Preserve the existing loopback-only safety guard in `scripts/bench/compare-hy2.sh`; if WAN orchestration is needed, write a separate self-owned-VPS wrapper that reuses its result schema/methodology rather than weakening the guard.

Application semantics must be equal enough to answer one narrow question. Prefer a deterministic TCP echo payload path for both implementations if that is the cleanest shared seam. Fix/record for both sides:

- exact payload bytes, length and SHA-256;
- same client/VPS pair and close time window;
- route and MTU metadata;
- one stream/client, same run count and timeout;
- security semantics truthfully described (both authenticated+encrypted; no claim of cryptographic equivalence);
- application bytes;
- elapsed raw samples;
- failures;
- CPU user/system, max RSS and FD count where sampler scope is comparable;
- wire bytes only with trustworthy bounded capture metadata.

Use a small paired sample such as 5 runs per implementation inside standing limits. Report raw rows, median, P95 and failure count. A slower or failed Nekomusume result is valid evidence. Make **no superiority claim** from this first bounded sample.

### Follow-up D — Harvest one additional VPS-only release row chosen by current environment, without speculative implementation

**Dependency:** A/B/C completed or one is genuinely environment-blocked. Select the highest-value READY row from the existing release matrix, in this order:

1. real-session key update on an authenticated owned path, if the current real runner exposes the already-implemented key-update contract without adding a new protocol feature;
2. carrier recovery / migration-back after the new health-driven failure path, if the accepted manager contract can be exercised without production network changes;
3. genuine owned endpoint/source migration if the current environment can create it without modifying production routing;
4. package install/upgrade/rollback/readiness/cleanup recheck against the current release-relevant binary if package contents changed materially;
5. repeated native microbenchmark/resource sample only if it answers a currently open measurement question with warm-up/sample protocol.

Do not implement a speculative carrier or invent NAT semantics merely to fill the slot. If none is genuinely READY, stop after A-C and let the next reviewer choose from new evidence.

## Completion gates for this batch

- Previous A repair remains green and exact-head CI stays green or any new CI failure is investigated before more claims.
- Automatic failover evidence is manager/health-driven, not an unconditional timeout branch renamed as health.
- One miss never causes failover; staged threshold/hysteresis semantics are machine-tested.
- Session delivery evidence remains distinct from health/socket/packet observations.
- Real failover row records controlled injection, exact state transitions, recovery metrics, final delivery semantics and cleanup.
- Long-lived/resilience runner, if completed, keeps one real authenticated socket/session open and stays within the 10-minute standing bound.
- HY2 comparison, if completed, uses temporary isolated credentials/config and equal application semantics without changing the production Hysteria service or weakening the existing safety guard.
- Negative results are retained; no unchanged rerun is used to manufacture PASS.
- `RELEASE_CANDIDATE=false`, global `FREEZE=false`, `PRODUCTION_READY=false`, and `RELEASED=false` remain unchanged.

## Do not expand into

- production proxy/tunnel deployment;
- third-party targets or scanning;
- production route/firewall/DNS/proxy/tunnel/qdisc changes;
- ICMP/raw/SCTP/DCCP/GRE/ESP experiments outside standing authorization;
- weakening authentication/integrity for benchmark parity;
- UDP/TCP striping or aggregation;
- 0-RTT or enabling FEC without a new observed-problem gate;
- claiming application side-effect exactly-once semantics from Session dedup;
- claiming public Internet failover from one controlled self-owned path;
- declaring RC/security approval before the later independent review gate.

## Questions requiring maintainer decision

none.
