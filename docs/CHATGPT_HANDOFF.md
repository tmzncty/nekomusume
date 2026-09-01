# Nekomusume ChatGPT Handoff

Checked at: 2026-09-01 10:59 Asia/Shanghai
Repository HEAD: `4ecd29979e633cea02c17abb8e448e1e37ab8ad7`
Previous reviewed implementation HEAD: `f680702a45a1bbbf0672a0b8d46b3756308c99b1`
Previous reviewer handoff commit: `aa0c73970544c21c333ad6851abde650085efc3a`

## What changed

One substantive coding-agent commit landed after the previous reviewer handoff:

- `4ecd299` — **test seam + deterministic retry regression + evidence/status/documentation closure; no VPS run.** It adds a bounded `--drop-first-udp-noise-response` injection to the controlled failover server, sends an intentionally late UDP negotiation hello in the existing selection-loss regression, adds a process-level first-Noise-response-loss regression, asserts that selection/Noise retries do not re-negotiate, re-authenticate, replace the ResumeGuard/session identity, increment the path generation, or reset delivery state, and preserves the authenticated exact-semantic UDP/TCP `DeliveryAck` checks introduced by `f680702`.

The same commit also adds explicit supersession notes to the pre-`f680702` failover evidence, updates `docs/m3-wan-failover-gate.md` to the current negotiated/authenticated controlled-stop runner and standing-authorization boundary, and updates `docs/status.md` so the CLI/failover row no longer claims that failover/resume negotiation is absent.

Independent GitHub Actions evidence is green for exact HEAD `4ecd299`: Rust CI run #77 completed successfully. The workflow runs `bash scripts/check.sh` on stable and the pinned nightly decode fuzz smoke separately. No post-`f680702` replacement VPS evidence has landed yet.

## Review verdict

**SAFE TO CONTINUE — Primary A from the previous handoff is accepted; the next Primary is the replacement rented-VPS evidence batch.**

The retry/evidence-contract closure is sufficient to move immediately to the time-limited VPS work already authorized by `docs/standing-vps-lab-authorization.md`. Do not spend another cycle on unrelated local polish before attempting the READY VPS rows.

The bounded release evidence matrix remains unchecked. The controlled runner still demonstrates an explicit application-level UDP stop, not automatic health/PTO-threshold-driven degradation detection. N9 remains closed as `CANONICAL_CORPUS_V1_FROZEN=true`, while global `FREEZE=false`, `RELEASE_CANDIDATE=false`, `PRODUCTION_READY=false`, and `RELEASED=false` remain correct.

## Evidence boundaries

- `4ecd299` is deterministic process/test-seam + documentation/status work. It is **not** new WAN evidence.
- The first-selection-loss and first-Noise-response-loss claims are now directly exercised by process tests, and GitHub CI is green at the exact commit.
- The server retry cache is bounded to the same peer and replays the cached selection/Noise response rather than rerunning negotiation/authentication; the new regressions assert one negotiated/authenticated event and the same resumed Session/path generation.
- A late post-auth UDP negotiation hello is ignored by the established secure-data path rather than restarting negotiation in the tested scenario.
- The pre-`f680702` VPS run remains valid only for its preserved facts: canonical negotiation, authenticated UDP admission, authenticated TCP resume, ordered server receive, bounded execution and cleanup. Its old ACK wording is explicitly superseded.
- `docs/status.md` truthfully keeps replacement authenticated-DeliveryAck VPS evidence pending and natural threshold-driven failover absent.
- `IMPLEMENTATION_PLAN.md` correctly has N9 and negotiation path completion checked, while **Bounded release evidence matrix** remains the first unchecked release-engineering item.
- Standing authorization permits the READY self-owned client/VPS TCP/UDP/failover/benchmark/capture/cleanup work without another per-run permission request, within the documented 10-minute / 256 MiB / 32-session maxima.
- The VPS is a one-month time-limited research asset. VPS-only evidence now outranks unrelated local implementation or documentation work.
- No performance superiority conclusion is currently supported.

## Work Package — harvest replacement VPS evidence first, then convert the rented host into reusable release evidence

Execute A -> B -> C -> D in order while dependencies stay green. A is the immediate Primary and should be attempted before unrelated local work. If an individual VPS row is environment-blocked, preserve the exact blocker and continue with the other READY rows or the local instrumentation fallback; do not turn one environment miss into a global stop.

### Primary A — Post-fix rented-VPS release-evidence lab batch

**Goal**

Replace the superseded pre-`f680702` ACK evidence with real self-owned-socket evidence from exact current code, then use the same cleanup-safe lab setup to collect several logically distinct, non-performance release rows before the VPS rental window expires.

**Preconditions**

- exact current commit/binary identity recorded;
- self-owned client and self-owned VPS only;
- fresh temporary unprivileged ports within the repository/standing-authorization limits;
- temporary experiment identity/config only; do not read, copy or commit protected `neko-server.identity` or production secrets;
- one cleanup trap/path that removes only this lab batch's listeners/processes/temp files;
- preserve negative results.

#### A1 — Generic negotiated TCP and UDP real-socket sanity

Run separate bounded current/current generic TCP and UDP authenticated exchanges using current HEAD. Record for each row:

- experiment id;
- exact git/binary/package identity;
- address-family label (`IPv4` / `IPv6`) without unnecessary addresses in Git;
- actual ports, count, payload bytes and duration;
- selected canonical version;
- authentication/application success or exact failure;
- start/end timestamps;
- cleanup verification.

If the existing generic command makes it trivial and safe, add one bounded unsupported-version or malformed-negotiation negative row. Do not create a new malformed-input framework only for this run.

This is reachability/behavior evidence, not a throughput comparison.

#### A2 — Replacement controlled UDP -> TCP resume with authenticated DeliveryAck

Run the corrected controlled-stop failover path on the self-owned client/VPS pair. The small committed evidence summary must make the event/evidence chain machine- or log-auditable enough to establish:

```text
UDP canonical negotiation
-> UDP Noise authentication
-> first UDP logical record
-> encrypted + exact-semantic udp_delivery_ack_validated
-> next logical range becomes uncertain at the controlled UDP stop
-> TCP canonical negotiation
-> fresh TCP Noise authentication
-> authenticated ResumeBinding / ResumeGuard
-> uncertain logical range resent
-> receiver dedup / exactly-once application delivery where applicable
-> encrypted + exact-semantic tcp_delivery_ack_validated
-> ordered application completion
-> cleanup
```

Record application bytes and ciphertext bytes separately when emitted by the runner. Do not use `udp_ack_observed` / opaque-byte wording.

Classification must remain explicit:

```text
real self-owned TCP/UDP sockets + controlled application endpoint stop
!= automatic natural WAN degradation detection
!= FailoverController health/PTO threshold evidence
!= production failover
```

#### A3 — Repeated real-socket lifecycle sample

If A1/A2 are green, run one bounded repeated open/exchange/close sample, e.g. 8-16 small cycles, comfortably below standing limits. Record:

- cycles attempted/succeeded/failed;
- records/application bytes;
- duplicate/missing-delivery observation only if real Session state supports it;
- final listener/process count relevant to the experiment;
- cleanup result.

This is lifecycle/leak-resilience evidence, not stress/capacity evidence.

#### A4 — Real-socket retry rows using the newly tested response-loss seams

If the commands accept the test injections safely on the self-owned pair, run the smallest separate real-socket rows for:

1. first UDP negotiation-selection response dropped;
2. first UDP Noise response dropped.

Require eventual successful negotiated/authenticated application exchange and no duplicate negotiated/authenticated state transition in the structured events. These rows answer whether the deterministic retry seams also survive an actual remote socket path; they do not simulate arbitrary packet loss and must not be promoted to a general loss-tolerance claim.

If either injection flag is intentionally local-only or unsuitable for remote use, record that exact limitation and continue; do not redesign the CLI solely for this row.

#### A5 — Opportunistic IPv6 sanity only when the owned environment actually supports it

If the current client and VPS both have a usable owned IPv6 path and the existing command accepts the address representation truthfully, repeat only the smallest generic A1 TCP and UDP sanity rows over IPv6. If IPv6 is unavailable or the CLI needs a bounded local address-format fix, record the blocker and continue with IPv4; IPv6 must not block A1-A4.

**Evidence/cleanup requirements**

For every executed row preserve small redacted summaries containing the required standing-authorization metadata: experiment id, exact implementation identity, parameters, timestamps, client/server result, structured event summary, capture metadata only if actually captured, and cleanup state. Large raw logs/pcaps stay outside Git with hashes/locations summarized when useful. Never commit private keys, production credentials, unnecessary addresses or plaintext payloads.

**Completion definition**

At minimum, unless the environment itself is genuinely unavailable:

- current-head generic TCP and UDP real-socket sanity exists;
- current-head controlled failover evidence proves authenticated exact-semantic Session DeliveryAck on UDP and resumed TCP;
- old pre-fix ACK evidence remains superseded rather than rewritten;
- one repeated lifecycle sample exists;
- all experiment listeners/processes/temp files are cleaned;
- `RELEASE_CANDIDATE=false` and the overall bounded release matrix remains unchecked.

### Follow-up B — Reusable bounded process-resource sampler + one measured VPS use

**Dependency:** A1/A2 green, or execute locally while one VPS row is temporarily environment-blocked.

The repository still needs reusable process-scoped CPU/RSS/FD/socket evidence for both Nekomusume and the future HY2 paired comparison. Before inventing a new tool, inspect `scripts/bench/` for an exact-purpose sampler and extend it if suitable.

Build a bounded sampler/collector with a small machine-readable schema and deterministic fixture/validator tests. Minimum fields per role/process:

- experiment id;
- implementation + role;
- supplied git/binary identity;
- start/end/elapsed;
- exit status;
- CPU user/system time where available;
- max RSS or sampled RSS, units + collection source;
- FD count/peak or sampled count, method identified;
- owned experimental listener/socket count/peak without dumping unrelated socket details;
- application bytes from workload metadata;
- cleanup state;
- unavailable metrics represented as unavailable/null, never fake zero.

Requirements:

- finite interval/count/duration;
- no root requirement where `/proc`, `ps`, GNU `time`, `ss` or equivalent read-only sources suffice;
- no unrelated process payload/secret inspection;
- schema/validator fails closed on malformed required metadata;
- deterministic local parsing/aggregation regression;
- no CPU-heavy build/fuzz during measured network samples.

After local validation, exercise it once on a short A1/A2-style Nekomusume VPS run and commit the small redacted summary. This first measured use is resource-observation evidence, not a capacity limit.

### Follow-up C — Connect real carrier-health degradation to the real-socket failover runner

**Dependency:** A2 green. This is the next major release-evidence implementation gap after controlled-stop evidence.

**Goal**

Make the real-socket runner transition from UDP to TCP because the project's actual carrier-health/failure contract observes UDP degradation, rather than because the client executes an unconditional scripted stop.

Inspect and reuse the existing `CarrierState` / carrier-health / PTO / hysteresis / `CarrierManager` seams. Do not invent a parallel `if timeout { use_tcp }` state machine.

Use the smallest bounded self-owned fault source that stays inside standing authorization. Prefer application-level experimental behavior such as the self-owned test server deliberately ceasing UDP replies after a deterministic point while leaving TCP available. Do not modify VPS production firewall, route or qdisc.

Required implementation/test behavior:

- exact existing state/threshold events that move UDP from Active to Degraded/Failed (using actual repository vocabulary) are observable;
- retries/timeouts/resources are bounded;
- the same logical Session/DeliveryLedger state survives transition;
- uncertain data is resent according to the current Session contract;
- receiver dedup/exactly-once delivery is asserted where supported;
- TCP resume keeps negotiation binding and ResumeGuard semantics;
- deterministic local/process integration test proves the threshold drives failover;
- the test/runner never calls an unconditional `controlled_udp_stop` and then labels the result automatic;
- no new experimental carrier or aggregation behavior.

If the carrier model is not connected enough, do not declare a generic blocker. Produce a concrete dependency map and implement the smallest missing adapter/seam that connects the existing health state to the runner. Once green, a bounded VPS run is immediately READY under standing authorization and should be taken before unrelated work.

### Follow-up D — Prepare and execute a fair self-owned-VPS HY2 paired comparison

**Dependency:** B sampler available or equivalent resource metrics already exist; A generic network sanity green. C can continue first when it is the more direct release blocker, but do not postpone HY2 until the end of the rental month.

The repository already pins HY2 v2.9.3 and its hash and already has `docs/bench/hy2-comparison-workload.md`. Preserve the existing loopback-only `scripts/bench/compare-hy2.sh` fail-closed behavior; do not weaken its target guard just to reach the VPS.

Build the smallest separate self-owned-VPS comparison seam or exact workload wrappers that provide **equal application semantics** to both implementations under the existing comparison contract.

For HY2:

- reuse the pinned v2.9.3 binary/hash already documented;
- use fresh temporary experimental config/certificate/auth under a dedicated temp path;
- never read/reuse `/etc/hysteria/server.yaml` secrets;
- never restart/reconfigure/stop the existing HY2 service;
- fresh high experimental port only;
- trap cleanup and verify the experiment listener/process is gone.

For Nekomusume and HY2 fix the same:

- payload bytes + SHA-256;
- client/VPS pair and route/time window;
- MTU metadata;
- security class;
- stream/load shape;
- finite timeout and run count.

Produce raw paired samples plus median/P95/failures, CPU user/system, RSS, FD and application bytes. `wire_bytes` stays null unless a trustworthy bounded capture supplies it. Preserve slower/failing Nekomusume samples exactly.

A successful first paired sample set is comparison evidence only. Do not claim superiority or production capacity.

## Completion gates for this handoff

- `4ecd299` retry/evidence-contract closure remains green; exact-head GitHub Rust CI #77 is accepted independent repository evidence.
- No pre-`f680702` ACK row is reused as authenticated Session DeliveryAck evidence.
- Post-fix self-owned VPS TCP and UDP sanity is collected unless the environment itself is genuinely unavailable.
- Post-fix controlled failover proves encrypted exact-semantic UDP and TCP Session DeliveryAck and keeps the controlled-stop/automatic-degradation distinction explicit.
- Repeated real-socket lifecycle evidence records cycles/failures/cleanup without becoming a stress claim.
- Real-socket response-loss retry rows are collected when the existing injection seam supports them without redesign; otherwise the limitation is explicit.
- IPv6 absence never blocks available IPv4 evidence.
- A bounded process-resource sampler becomes reusable, schema-validated and exercised once before HY2 comparison work.
- Automatic health/PTO-driven failover remains unchecked until the actual existing health state drives the transition in deterministic tests and then a bounded VPS run.
- HY2 comparison preserves equal semantics and does not touch the existing production HY2 configuration/service.
- `IMPLEMENTATION_PLAN.md` Bounded release evidence matrix stays unchecked until the matrix criteria are actually met.
- `RELEASE_CANDIDATE=false`, global `FREEZE=false`, `PRODUCTION_READY=false`, `RELEASED=false` remain unchanged.

## Fallback

If any VPS/post-fix row exposes a new Session/crypto/correctness defect:

1. preserve the exact negative evidence and minimal reproducer;
2. stop additional release/performance claims on the affected path;
3. make the correctness repair the next Primary;
4. run the repository's required local/CI gates;
5. only rerun the affected VPS scenario after code/instrumentation/hypothesis has materially changed.

If VPS access/path is temporarily unavailable, do not idle: execute Follow-up B and/or the local implementation/test portion of Follow-up C that directly unlocks the next VPS evidence. If one address family is missing, continue with the other. If an action would exceed standing authorization, stop that action only and continue independent READY work.

## Do not expand into

- RC/production/security approval;
- claiming automatic failover from the controlled-stop runner;
- weakening authentication/integrity/resource bounds for benchmark speed;
- changing production HY2 config/service;
- third-party targets, scans, production firewall/route/DNS/proxy/tunnel/qdisc changes;
- >10-minute single runs, >256 MiB single-run application traffic, >32 concurrent experimental sessions, or disguised split pressure tests without new authorization;
- 0-RTT, enabled FEC, carrier striping/aggregation or exotic carriers without an observed-problem gate;
- global/public reachability or superiority claims from self-owned one-off evidence.

## Questions requiring maintainer decision

none.
