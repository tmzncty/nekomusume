# Nekomusume ChatGPT Handoff

Checked at: 2026-09-05 00:57 Asia/Shanghai
Repository HEAD reviewed: `96ba80fe876021998b30c5cacfeaa756100789bc`
Previous reviewed implementation HEAD: `ea1c05216b1b6e2ec198b907477af5056e4a956c`
Previous reviewer handoff commit: `a139a4a653f930d4e26bc1144ed698f803162704`

## What changed

The external agent consumed the first two implementation slices quickly after the previous handoff. Two coding commits are now ahead of `a139a4a`:

- `01c876c` — adds explicit `response_send_deadline_ms = 100`, a one-shot `PreauthResponsePermit`, deterministic 99/100/101 ms completion tests, and threads response permits through ordinary probe, periodic, multistream and failover pre-auth response call sites. This is meaningful D019 progress, but the call-site ordering still checks the 100 ms deadline **after** the socket write/send returns; it does not yet bound the I/O attempt itself.
- `96ba80f` — adds one-shot `PreauthQueuePermit` / CLI `QueueReservation` ownership and charges the failover UDP `PendingUdpNegotiation` before storing it. It releases the queue on successful authentication and adds a local five-second pending cleanup path. This is also meaningful progress, but it currently races/desynchronizes with the already-existing process `expire()` path: process admission can expire/release the underlying state at the 1 s idle timeout while application `pending` still retains the ticket/reservation until its separate 5 s timer.

The two commits changed five files (`neko-cli` main/multistream/periodic/preauth plus `neko-crypto`) and are coherent with the D019 lane rather than unrelated feature work.

Exact current HEAD `96ba80fe876021998b30c5cacfeaa756100789bc` has independent GitHub Actions success on both refs:

- main run `33895747196` — `success`;
- `work/continue-20260904` run `33895758937` — `success`.

This is exact-head CI evidence, not security approval.

The fast completion cadence is healthy, but A and B are **not yet security-closed**. The next work should repair the two concrete integration defects below, then continue directly into terminal-rejection and charge-order work. Do not discard the new typed permit primitives; repair their runtime integration.

## Review verdict

**CONTINUE_WITH_REQUIRED_FIXES — typed response/queue ownership is accepted as useful infrastructure, but RSEC-001A and RSEC-001B remain HIGH at the runtime integration boundary. Repair A1/B1, then continue D -> E without another reviewer round-trip.**

The project is not globally blocked and no administrator action is required for A1/B1/D/E. Do not use VPS time for these deterministic security-accounting defects.

## Reviewer findings

### RSEC-001A1 — HIGH — the 100 ms permit is checked after I/O, so a late response can already have been emitted

D019 requires a **100 ms monotonic budget from response admission to completed bounded send attempt; expiry abandons the response**.

`01c876c` correctly introduces the 100 ms policy value and a move-only permit. However, every real call site currently follows the semantic shape:

```text
charge_response -> socket write/send -> complete_response(now)
```

`complete_response()` rejects elapsed completion after 100 ms, but by then `write_frame` / `send_to` may already have blocked and emitted bytes. A response that completes at 101+ ms can therefore be observed on the wire before the code reports `pre-auth response deadline elapsed`. Post-send detection is not the same as a bounded send attempt or expiry abandonment.

Existing broader experiment/setup timeouts do not solve this: they may be seconds long and are not bound to the individual response permit.

**Required repair**

- Preserve the typed `PreauthResponsePermit` and exact 100 ms value.
- Bind the permit deadline to the **actual socket send/write attempt before I/O begins**. The response path must derive the remaining permit budget and configure/use a bounded write/send operation whose deadline is no later than the permit deadline.
- For TCP framed writes, the complete frame write must be bounded by the permit, not merely the outer experiment deadline. A partial/timeout write is failure and must not be reported as successful negotiation/authentication/readiness/session evidence.
- For UDP, use a bounded/nonblocking or write-timeout path consistent with the same permit semantics; do not assume `send_to` is semantically instantaneous merely because it normally returns quickly.
- Completion still consumes the permit exactly once. Scheduled response accounting remains charged on timeout/failure as D019 requires; do not refund it merely because the I/O attempt fails.
- Review whether response admission/send is `bounded progress` for the D019 1 s idle clock. If it is, update progress consistently; if not, document/test the stricter behavior. Do not leave this accidental.

**Deterministic verification**

Keep the core 99/100/101 ms tests, but add an I/O-boundary regression using an injectable/fake writer/sink or equivalent deterministic seam that proves:

- a response whose sink completes inside the permit succeeds;
- a sink that would complete after the permit is cancelled/refused before success evidence;
- the call-site helper cannot perform an unbounded write under a response permit;
- no wall-clock sleeps are needed for the semantic boundary test;
- negotiation/authentication/readiness/Delivery/PathValidated/ACK-equivalent success evidence is emitted only after the bounded send completes.

### RSEC-001B1 — HIGH — process expiry can invalidate a pending queue reservation behind application ownership

`96ba80f` charges queue capacity before constructing `PendingUdpNegotiation`, which is the right direction. But there are now two independent lifetime mechanisms:

1. `preauth.expire()` runs on every failover-server loop and `ProcessPreauthAdmission::expire()` releases states at D019 idle timeout (1 s) or lifetime (5 s), including their queued counts;
2. the application `pending` object retains `AdmissionTicket + QueueReservation` and only explicitly tears itself down when its separate `created_at.elapsed() >= 5 s` condition fires.

This creates stale split ownership. After approximately one second without bounded progress, process accounting may already remove the state/queue while `pending` still exists. Then:

- a later packet for that pending peer attempts `charge_input` against an already-expired state and currently takes the process-level `fail("pre-auth admission rejected")` path rather than a clean bounded rejection; or
- the local 5 s cleanup later calls `dequeue` / `release` using permits whose underlying process state has already been removed, producing `pre-auth queue release failed` / stale cleanup behavior.

This is exactly the class of ownership divergence the queue guard was intended to eliminate.

**Required repair**

Choose one coherent lifecycle model; do not keep independent process/app timers that can release the same logical state behind each other.

Acceptable shapes include:

- make `ListenerAdmission::expire()` return/identify expired state IDs and have the application atomically discard/invalidate the matching `PendingUdpNegotiation` without trying to dequeue a reservation the process already consumed; or
- move pending expiration/queue ownership into one admission-owned abstraction so expiry consumes state + queue + ticket ownership in one place; or
- another equally fail-closed ownership model that proves application pending state can never outlive its process admission state.

Requirements:

- D019 idle 1 s and lifetime 5 s remain the authoritative values; do not invent another timeout;
- promotion/authentication releases/dequeues once;
- malformed rejection, idle expiry, lifetime expiry, cancellation/replacement and server shutdown cannot double-dequeue or act on stale tickets;
- queue capacity is charged before pending ownership and no rejected enqueue stores application pending state;
- source/global queue accounting remains exact across expiration;
- ordinary idle expiry must reject/cleanup the pending peer without crashing the bounded server process merely because a stale ticket remained in application state.

Add deterministic integration tests for the 1 s idle-expiry-before-5 s-lifetime case, successful promotion, max/max+1 queue saturation across distinct sources, cancellation/replacement and double-cleanup rejection.

### RSEC-001D — still open — rejected states remain reusable

The new typed response/queue permits do not yet make the underlying logical pre-auth state terminal when a charge/enqueue/deadline operation is rejected. After A1/B1, continue directly to the already-defined terminal/non-revivable state repair; do not wait for another review.

### RSEC-001C — ADR checkpoint remains isolated

The carrier/source persistence conflict is unchanged: D019 forbids reset by retry/reconnect/carrier change/error while the current source row is removed after its last live state, but D019 defines no bounded terminal-source retention TTL/count/table ceiling. Do not improvise a new numeric retention policy. This checkpoint still comes **after** A1/B1/D/E so it does not idle independent work.

## Evidence boundaries

- `IMPLEMENTATION_COMPLETE=true` remains the bounded research baseline only.
- `CANONICAL_CORPUS_V1_FROZEN=true` remains corpus-specific only.
- `RELEASE_CANDIDATE=false`, `PRODUCTION_READY=false`, `FREEZE=false`, `RELEASED=false` remain required.
- Exact `96ba80f` CI is green; this proves repository gates on the exact tree, not D019 semantic closure or security approval.
- `PreauthResponsePermit` and `PreauthQueuePermit` are useful one-shot primitives and should normally be repaired/integrated rather than reverted.
- A late post-write deadline check does **not** prove bounded response I/O.
- Queue counters in process state do **not** prove correct application pending ownership when process expiry can invalidate the underlying state independently.
- Existing inner `PreauthBudget` remains useful and must not be weakened merely to simplify outer process admission.
- Source/global process admission cannot claim to bound kernel SYN backlog, provider NAT state or other resources outside the process.
- No VPS/load run substitutes for deterministic A1/B1/D019 correctness.
- Protected identity/secrets/private endpoint material remain unread/untracked/uncommitted.

## Rolling Work Queue

This remains a rolling multi-hour queue. Complete a coherent slice -> targeted/full required gates -> commit -> push -> immediately consume the next dependency-satisfied slice. Do not stop for a reviewer interval. A new HIGH/BLOCKER that invalidates downstream work, genuine ADR/core-architecture conflict, authorization boundary, production impact, missing credentials/third-party authority, repository breakage, runtime/tool-budget termination or true queue exhaustion is a stop condition.

### A1 — Make response permits bound the actual I/O attempt

**Status:** `READY_LOCAL`; highest priority.

Implement the RSEC-001A1 repair above. Keep the exact D019 100 ms value and typed permit; bind it to real TCP/UDP response I/O before bytes can be emitted outside the permit budget.

Audit all response-permit call sites introduced by `01c876c`: ordinary TCP/UDP probe, periodic TCP, multistream TCP, failover UDP selection/Noise responses and failover TCP negotiation/Noise response. No pre-auth response path may use `charge -> unbounded send -> post-check`.

Run targeted deterministic deadline/I/O tests, all affected CLI integration tests, full repository gate, `git diff --check`; fuzz only if network-input/parser semantics change. Commit and push.

**Continue immediately to B1:** yes.

### B1 — Unify queue reservation and state expiry ownership

**Status:** `PREAUTHORIZED_AFTER_A1`.

Repair RSEC-001B1. Preserve one-shot queue ownership but remove stale split lifetime between `ProcessPreauthAdmission::expire` and `PendingUdpNegotiation`.

Required deterministic coverage: idle expiry at 1 s while pending lifetime is 5 s, lifetime expiry, successful authentication/promotion, cancellation/replacement, source/global queue max/max+1, server-loop continuation after ordinary expiry, exactly-once accounting cleanup.

Run targeted tests + full gate, commit and push.

**Continue immediately to D:** yes.

### D — Make rejected pre-auth state terminal/non-revivable

**Status:** `PREAUTHORIZED_AFTER_B1`.

Make any exhausted/saturated/unmeasurable/deadline rejection terminal at the reusable process state/ticket boundary, or structurally consume the ownership token so reuse is impossible.

Required regressions:

- global input/work rejection cannot succeed on same logical state after one-second rollover;
- source/global response rejection cannot later send on same state;
- queue rejection cannot later enqueue on same rejected state;
- response deadline failure cannot later become success;
- inner/outer budget failure remains cross-layer atomic;
- release/cleanup remains one-shot.

Run targeted tests + full gate, commit and push.

**Continue immediately to E:** yes.

### E — Audit and machine-check charge ordering across every real responder

**Status:** `PREAUTHORIZED_AFTER_D`.

For each externally reachable pre-auth responder, machine-check or inventory:

1. typed carrier/source projection + state admission;
2. input bytes/packet charge before parse;
3. parser/work reservation before work;
4. state memory reservation before owned allocation;
5. queue reservation before pending ownership;
6. response charge + actual bounded response I/O permit before send;
7. terminal rejection/evidence barrier;
8. exactly-once cleanup.

Current conservative 64/4096 work-unit reservations may remain only if they dominate bounded parser work; they are accounting units, not CPU cycles. Fix concrete uncovered seams only.

Add a machine-checkable/static inventory or deterministic integration guard so a new external responder cannot silently bypass `ListenerAdmission`.

Run full gate, commit and push.

**Continue immediately to C:** yes.

### C — Resolve D019 carrier/source projection and persistence semantics

**Status:** `ADR_CHECKPOINT_AFTER_E`.

First perform the noncontroversial typed projection work: explicit bounded carrier discriminator (`TCP`, `UDP`, one bounded unknown bucket where applicable), deterministic non-collision tests, no raw source logging.

Then address terminal source persistence. Do not silently retain all sources forever or invent TTL/LRU/max-history values. If existing reviewed text cannot establish a bounded interpretation of “never reset by retry/reconnect/carrier change/identity change/error”, write a compact ADR amendment request with the exact conflict/options and stop **this slice only** for reviewed policy. A/B/D/E progress remains valid.

If a reviewed bounded policy already exists by then, implement retry/reconnect/carrier-transition persistence and bounded storage tests.

**Continue immediately to F only when C is resolved:** yes.

### F — Complete full D019 adversarial/evidence-barrier matrix

**Status:** `PREAUTHORIZED_AFTER_C`.

Cover source/global concurrency, source lifetime input/packet/work under resolved C semantics, global one-second windows, per-packet work, state/global memory, source/global queue, source/global response + inner 3x anti-amplification, idle/lifetime/response deadlines with injectable time, overflow, terminal non-revival, retry/reconnect/carrier transition, cancellation/timeout/double cleanup, no session/path/delivery/readiness/authz evidence on rejection, secret-safe diagnostics.

Do not substitute VPS/load tests for deterministic accounting semantics.

Full local gate, commit, push; require exact repair-head CI green for security closure.

**Continue immediately to G:** yes after exact-head CI green.

### G — Fresh exact-tree D019/security evidence review

**Status:** `PREAUTHORIZED_AFTER_F`.

Re-read exact implementation/tests and then update `docs/reviews/resource-abuse-evidence-2026-09-04.md`, `docs/release-security-review-packet.md`, `docs/status.md` and closure/navigation records. Reviewed-tree claims must name the actual reviewed implementation head.

RSEC-001 closes as an implementation finding only when A1/B1/D/E/C/F are truly satisfied. Independent external/two-person security review remains separate. No automatic RC/production/freeze/release promotion.

**Continue immediately to H if no new HIGH/BLOCKER:** yes.

### H — Compatibility / freeze-boundary review

**Status:** `READY_LOCAL_AFTER_G`.

Audit corpus-v1 content-addressed freeze vs global protocol non-freeze, current/current negotiation, unsupported/future rejection, downgrade/transcript binding into Noise, resume/version binding and replay boundary, and stale docs that imply corpus freeze == release/protocol freeze. Add regression only for a real defect; do not reopen frozen corpus bytes without correctness evidence.

**Continue immediately to I:** yes.

### I — Package/operator and evidence-provenance integrity review

**Status:** `READY_LOCAL_AFTER_H`.

Verify existing bounded evidence for x86_64 package/build identity, install/readiness/smoke/upgrade/rollback, retained external state without reading protected identity material, shutdown/listener/temp cleanup, canonical Git-blob/checksum manifests, exact-head CI references and stale release-packet links/hashes. Do not rerun already-sufficient VPS/package work merely for freshness.

**Continue immediately to J:** yes.

### J — Reclassify release opportunities and reconsider VPS

**Status:** `READY_LOCAL_AFTER_I`.

Re-evaluate each release-closure row:

- answered bounded question -> `ALREADY_SUFFICIENT_FOR_BOUNDED_QUESTION`;
- concrete executable missing assertion -> `OPEN_READY` with exact evidence/action/dependencies/scope;
- environment/governance/implementation dependency -> truthful blocked class.

Then use the rented VPS only if a genuine dependency-ready missing live release question exists under standing authorization. Otherwise record `READY_LIVE: none`. No unchanged retry of closed repeated/periodic/HY2 lines.

## Completion gates

RSEC-001 implementation closure requires all of:

- the 100 ms response permit bounds actual response I/O, not only post-send bookkeeping;
- application pending ownership cannot outlive/process-diverge from its pre-auth queue/state reservation;
- queue reservations are charged before ownership and released exactly once across promotion/rejection/idle/lifetime/cancel/shutdown;
- rejected logical pre-auth states cannot revive after retry/window rollover;
- charge ordering is auditable across every external responder;
- carrier/source projection and persistence are reconciled with D019 under a bounded reviewed policy;
- full deterministic boundary/overflow/timeout/evidence-barrier matrix passes;
- full local gate + exact-head GitHub CI are green;
- security review prose names the exact reviewed implementation tree and does not outrun code.

The broader rolling queue remains active through H-J unless a real stop condition occurs.

## Do not expand into

- public/production listener deployment;
- new numeric D019 retention/source-table ceilings without explicit reviewed ADR amendment;
- protocol/wire/Noise/Session/Carrier redesign unrelated to these concrete admission defects;
- VPS load testing as a substitute for deterministic security accounting;
- endless generic harness/adversarial review after the stated A1/B1/D/E/C/F gates are satisfied;
- reopening frozen corpus bytes without correctness evidence;
- speculative FEC/0-RTT/striping/multipath/exotic-carrier work;
- third-party targets/scanning;
- production route/firewall/DNS/proxy/tunnel/qdisc changes;
- reading/copying/hashing/committing protected identity/secrets/private topology.

## Questions requiring maintainer decision

No immediate administrator action is required for A1/B1/D/E.

A maintainer/reviewer decision is required only if C reaches the already-described ADR conflict and no existing reviewed text provides a bounded source-retention interpretation. At that point present the exact policy options and trade-offs; do not invent a numeric retention rule autonomously.
