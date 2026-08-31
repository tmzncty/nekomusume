# VPS Rental-Window Priority

This file is a coordination/prioritization policy, not a protocol specification and not an expansion of network authorization.

The maintainer has stated that the current VPS is a **time-limited one-month research asset**. While it remains available, agents and reviewers should optimize for **evidence value per rental day**, not for artificial CPU/network utilization and not for feature count.

All execution remains bounded by `SECURITY.md` and `docs/standing-vps-lab-authorization.md`.

## 1. Scheduling principle

When two READY tasks are otherwise comparable, prefer the task whose evidence would be difficult or impossible to obtain after the rented VPS disappears.

Priority order while the VPS is available:

1. **VPS-only / real-network evidence** that cannot be reconstructed later from unit tests or netns;
2. **local harness/instrumentation work that directly unlocks the next VPS evidence run**;
3. release correctness/security gates that must precede a truthful VPS claim;
4. ordinary local implementation/test work;
5. documentation polish, speculative features, and experimental-track ideas without an observed-problem gate.

A correctness/security blocker still wins over benchmark throughput or additional WAN sampling. Do not run a known-invalid candidate merely to consume rental time.

## 2. Evidence backlog to prioritize during the rental window

The reviewer should continuously reconcile this list against current repository facts and remove items already proven.

### A. Real socket/WAN behavior

Highest value because netns/loopback cannot substitute for it:

- authenticated TCP and UDP session establishment on administrator-controlled client/VPS endpoints;
- repeated real socket open/exchange/close cycles with cleanup verification;
- UDP degradation -> TCP fallback using the same logical Session when the current implementation path is ready;
- uncertain resend / receiver dedup / exactly-once application delivery evidence across real sockets;
- carrier recovery and migration-back after a real bounded failure;
- endpoint/path migration where the available owned environment can produce a genuine source endpoint change without modifying production routing;
- key update during a real authenticated session, including interaction with carrier transition when implemented;
- bounded PMTUD/packetization behavior on the real path when current protocol instrumentation can observe it truthfully;
- IPv4 and IPv6 rows separately when both address families really exist in the owned environment;
- preserve negative results with exact experiment metadata rather than rerunning unchanged failures.

### B. Bounded resilience/soak under standing authorization

Standing authorization permits single runs up to 10 minutes. Use this window for **scientifically distinct bounded scenarios**, not to mechanically split one forbidden long soak.

Useful distinct runs include:

- 5-10 minute authenticated steady session;
- 5-10 minute idle-with-periodic-exchange session;
- repeated socket open/close lifecycle run;
- repeated cross-process failover/recovery run;
- key-update cycle run;
- reconnect/restart behavior that the current implementation actually supports.

Record at least application records/bytes, failures, duplicates/missing delivery, reconnect/failover events, CPU/RSS/FD/socket observations when available, timestamps, binary identity, and cleanup state.

Do **not** claim durable Session survival across process restart unless a durable session-store contract is actually implemented; current repository evidence explicitly does not establish that property.

### C. Nekomusume vs HY2 comparison

The repository already has a pinned HY2 artifact/setup and a comparison workload contract. During the VPS rental window, prioritize closing the remaining gap:

1. implement/verify an equivalent Nekomusume application exchange command;
2. make both commands satisfy the same application payload and metadata contract;
3. run bounded paired samples on the same owned VPS/client, route/time window, MTU, security class and load;
4. report raw samples, median, P95, failures, CPU, RSS, FD count and application bytes; wire bytes only when capture metadata is trustworthy;
5. preserve negative or slower Nekomusume results exactly as evidence.

Do not publish superiority claims from a one-off or semantically unequal comparison.

### D. Native VPS performance/resource evidence

The existing Era-4 microbenchmark is only a single bounded observation. Use spare VPS compute, when it does not interfere with WAN measurements, for evidence that is repeatable and attributable:

- release-build reproducibility checks after meaningful build/package changes;
- targeted full workspace gates on the native x86_64 VPS after release-relevant changes;
- bounded parser/property/fuzz campaigns after parser/wire changes, if the toolchain supports them;
- repeated microbenchmark samples with explicit warm-up/sample protocol when performance work is the current question;
- process-scoped CPU/RSS/FD/socket instrumentation for real application runs;
- low-concurrency resource observations within standing limits, aimed at finding leaks/pathological growth rather than claiming capacity limits.

Do not keep the VPS busy merely to increase utilization statistics. A repeated run must answer a new question, sample a deliberately different condition/time window, or validate a meaningful code/configuration change.

### E. Package/operator evidence

Use the owned VPS for release engineering that is hard to validate purely in a build directory:

- install / smoke / upgrade / rollback in the dedicated experimental install path;
- verify external identity/state retention without reading or committing protected identity material;
- listener/readiness/shutdown cleanup behavior;
- stale-process/listener detection after failed or interrupted experiments;
- binary/package hash and version provenance.

## 3. Local work that should immediately unlock VPS work

During the rental window, local slices should be selected partly by whether they make the next VPS run possible. Examples:

- Nekomusume benchmark command compatible with the existing HY2 workload contract;
- remote experiment runner and evidence collector;
- CPU/RSS/FD/socket sampler;
- endpoint-migration/failover injection harness that stays within standing authorization;
- structured experiment metadata and result validator;
- negotiation/failover path completion required before a truthful real-WAN claim.

When one of these is READY and blocks a high-value VPS experiment, prefer it over unrelated local enhancements.

## 4. Reviewer duty during hourly handoff

Every hourly reviewer pass should ask, in this order:

1. Is there a correctness/security blocker that must be repaired before any further claim?
2. Is there a READY VPS-only evidence task under standing authorization?
3. If not, is there a READY local slice that directly unlocks one?
4. If not, what is the earliest genuine release-engineering task?

When writing `docs/CHATGPT_HANDOFF.md`, include at least one **VPS opportunity** whenever a truthful, dependency-satisfied VPS task exists. Do not let a short local task consume the whole package if a ready VPS evidence run can follow it safely.

If a VPS run is blocked, state the exact blocker as one of:

- implementation dependency;
- missing environment/address family;
- evidence instrumentation gap;
- out-of-standing-authorization requirement;
- independent review requirement.

Do not use generic `need WAN authorization` for work already covered by `docs/standing-vps-lab-authorization.md`.

## 5. Efficient experiment batching

To reduce deployment/setup overhead, group compatible evidence runs into one bounded lab session when doing so does not confound measurements. A typical lab batch may:

1. verify binary/package identity and clean baseline;
2. start the temporary listener;
3. run one or more logically distinct bounded scenarios;
4. capture structured events/resource samples;
5. stop the listener;
6. verify cleanup;
7. archive small evidence summaries/hashes.

Performance-comparison runs should not be mixed with unrelated CPU-heavy builds/fuzzing at the same time because that would invalidate the measurement environment.

## 6. What not to do just because the VPS rental is expiring

The rental deadline does not justify:

- bypassing correctness/security gates;
- widening standing authorization;
- scanning or targeting third parties;
- changing production firewall/route/DNS/proxy/tunnel/qdisc;
- mechanically splitting a >10 minute soak or pressure test into repeated runs to evade authorization;
- turning a one-off observation into a production/reachability/performance conclusion;
- implementing speculative FEC/0-RTT/exotic carriers merely to generate VPS activity;
- leaving long-lived experimental daemons behind.

The goal is to leave the rental window with a dense, reproducible **evidence archive**, not with maximum resource utilization graphs.