# Nekomusume ChatGPT Handoff

Checked at: 2026-09-05 11:00 Asia/Shanghai
Repository HEAD reviewed: `61d69bdd58b071f7d9c3e1ec99602cebbf032787`
Previous reviewed implementation HEAD: `f99c52c0d6c12a4a60e4d909e40230eca2131bed`
Previous reviewer handoff commit: `5931523b3173c72c0246664daa61ca0e2c320105`

## What changed

One coding-agent commit landed after the previous reviewer handoff:

- `61d69bd` — **moves pre-auth responses into bounded TCP/UDP send helpers and converts the known real responder call sites away from the old post-send-only completion shape.** It introduces an absolute permit deadline in the helper path, deterministic partial-writer coverage, charged-response abandonment, and uses the helper in ordinary TCP/UDP negotiation+Noise responses, periodic/multistream TCP handshakes, and failover UDP/TCP pre-auth responses. It does not change wire format, Noise, Session delivery semantics, Carrier selection, or release flags.

Both `main` and `work/continue-20260904` point at exact `61d69bd`.

### Exact-head CI

Exact `61d69bd` is independently green on both refs:

- main Rust CI run `33938543214` — overall `success`;
- work-branch Rust CI run `33938550077` — overall `success`.

This is repository CI evidence, not a security approval.

## Review verdict

**CONTINUE_WITH_REQUIRED_FIXES — the A1 direction is materially correct and real responder coverage improved, but the implementation still does not prove one absolute 100 ms I/O budget across blocking partial TCP writes and it leaks the temporary socket write timeout into later traffic. Repair that narrow seam, then continue B1v -> D1 -> E -> C1/C2 without another reviewer wait.**

No administrator action is required now. Do not spend VPS time on this deterministic security-accounting lane while A1r/D1 remain open.

## Reviewer findings

### RSEC-001A1 — PARTIALLY CLOSED — response I/O is now routed through the permit, but three concrete deadline/ownership defects remain

`61d69bd` fixes the largest previous gap: real pre-auth call sites now use `send_tcp_response` / `send_udp_response` rather than sending first and checking the permit afterward. The response remains charged on failure and `abandon_response` rejects the logical state.

However, A1 is not evidence-complete yet.

#### A1.1 — HIGH — TCP partial writes still reuse the initial OS timeout instead of the remaining absolute deadline

`send_tcp_response` computes one `budget` before the first write and calls `TcpStream::set_write_timeout(Some(budget))` once. `write_frame_until` checks the monotonic deadline before each `write`, but the kernel blocking timeout for every later partial `write` remains the original full budget.

Example failure model:

```text
permit admitted at 0 ms, deadline 100 ms
first blocking write returns partial data at 90 ms
helper checks now=90 and starts second write
socket still has the original ~100 ms write timeout
second write may block well beyond absolute 100 ms
```

A post-return `complete_response` can reject the attempt, but bytes may already have been accepted by the socket after the intended absolute budget. The deterministic `PartialWriter` regression models partial progress but not a blocking syscall consuming most of the deadline.

**Required repair:** before every potentially blocking TCP write syscall, recompute the remaining time from the same absolute permit deadline and apply that remaining timeout, or use an equivalent nonblocking/deadline-aware primitive. Never refresh to a new 100 ms budget after partial progress.

#### A1.2 — MEDIUM/HIGH — zero remaining budget is converted to an extra 1 ms

`remaining_response_budget` currently uses `Duration::from_millis(remaining_ms.max(1))`. At `now == deadline`, a send has zero remaining budget but receives a fresh 1 ms blocking allowance.

Rust's official `TcpStream::set_write_timeout` contract rejects a zero `Duration`, which explains why a nonzero value was used, but that does not authorize extending the D019 deadline. Official reference: <https://doc.rust-lang.org/std/net/struct.TcpStream.html#method.set_write_timeout>.

**Required repair:** if remaining budget is zero before a blocking send begins, abandon/fail closed rather than inventing 1 ms. Exact-boundary success is still legal when a send that began earlier actually completes at the inclusive deadline.

#### A1.3 — HIGH regression risk — temporary write timeout is not restored on successful reusable sockets

Both `send_tcp_response` and `send_udp_response` mutate the socket write timeout. On success the code does not restore the previous timeout. The same TCP/UDP socket is then reused by later authenticated/failover/session traffic, so a pre-auth safety mechanism can silently alter post-auth I/O behavior.

Rust exposes `write_timeout()` and `set_write_timeout()` as persistent socket options; the timeout remains in force until changed. Official reference: <https://doc.rust-lang.org/std/net/struct.TcpStream.html#method.write_timeout>.

**Required repair:** preserve/restore the prior timeout on every success path where the socket remains live, and preferably through a small scoped guard so early returns cannot leak the temporary setting. If the previous timeout cannot be read/restored, fail closed before treating the pre-auth send as successful rather than silently changing the later Session contract. Apply the same principle to UDP.

#### A1 acceptance after repair

Keep the existing one-shot permit and exact candidate value. Add deterministic or socket-level regressions proving:

- one absolute deadline across multiple partial writes;
- no write starts with zero remaining budget;
- inside-budget and exact-completion-boundary behavior;
- timeout/partial/error remains charged and terminal;
- prior socket write-timeout state is restored after successful pre-auth response;
- all currently covered real responder call sites remain on the bounded helper;
- no success/readiness/Session-equivalent evidence is emitted after a failed/late pre-auth response.

Do not widen this into a new I/O framework.

### RSEC-001B1 — accepted implementation direction; verification still incomplete

`ea5b257` + `f99c52c` remain the accepted one-owner direction for process expiry and application pending ownership. Do not redesign the queue/state shape unless tests falsify it.

Still required before B is evidence-complete:

- exact idle expiry at D019 1 s while five-second lifetime has not elapsed;
- exact lifetime expiry;
- successful authentication/promotion;
- cancellation/replacement;
- source queue max/max+1;
- global queue max/max+1 across distinct source projections;
- ordinary expiry keeps the bounded responder/server loop alive;
- queue/state/memory release exactly once on every terminal path.

### RSEC-001D1 — HIGH — terminal rejection remains incomplete across inner and arithmetic failures

Current `ListenerAdmission::charge_input` / `charge_response` still allow inner `PreauthBudget` rejection to return before the corresponding process state is explicitly marked terminal. State-associated checked arithmetic/deadline construction can also fail before an explicit reject path.

D019 requires exhausted, saturated, unmeasurable, timed-out, malformed or over-limit operations to fail closed and not become usable later after window refill/retry.

Required repair remains:

- inner input rejection -> process/logical ticket terminal;
- inner response/anti-amplification rejection -> terminal;
- state-associated checked-add/deadline/clock-unmeasurable failure -> terminal;
- global/source/queue/deadline rejection remains terminal;
- preserve truthful cross-layer rollback/accounting;
- prove no revival after the one-second window rolls over.

### RSEC-001C — isolated ADR checkpoint remains unchanged

D019 says counters are not reset by retry, reconnect, carrier change, identity change or error. Current source rows are removed after the final live state is released. Keeping every terminal source forever would create an unbounded source-accounting map, while D019 defines no terminal-source retention TTL/history ceiling/eviction rule.

Do not invent a numeric retention policy. First make carrier/source projection explicit and bounded after the concrete charge-order audit. If reviewed text still cannot reconcile no-reset semantics with bounded source-accounting storage, produce the compact ADR amendment request and stop only this policy-dependent lane; H/I remain safe independent fallback work.

## Evidence boundaries

- `IMPLEMENTATION_COMPLETE=true` remains a bounded research-baseline flag only.
- `CANONICAL_CORPUS_V1_FROZEN=true` remains corpus-specific only.
- `RELEASE_CANDIDATE=false`, `PRODUCTION_READY=false`, `FREEZE=false`, `RELEASED=false` remain required.
- Exact `61d69bd` is real implementation plus deterministic tests and exact-head green CI. It materially improves A1 but does not close A1/B1v/D1/E/C.
- The bounded send helpers do not prove provider/kernel/network-level delivery timing; D019 requires the process-side bounded send attempt and truthful failure/evidence barrier, not a claim about remote receipt time.
- A temporary pre-auth socket timeout must not silently become a post-auth Session timeout.
- Process admission does not claim to bound kernel SYN backlog, provider NAT state or resources outside the process.
- Existing inner `PreauthBudget` remains useful and must not be weakened to simplify outer accounting.
- No VPS/load run substitutes for deterministic D019 counter, ownership and evidence-barrier correctness.
- Standing VPS authorization remains valid for genuinely READY self-owned TCP/UDP work, but the current deterministic security lane has precedence.
- Protected identity/secrets/private endpoint material remain unread/untracked/uncommitted.

## Rolling Work Queue

This is a rolling multi-hour queue. Finish one coherent slice -> targeted/full gates -> commit -> push -> immediately consume the next dependency-satisfied slice. Do not stop for a reviewer interval. Only a new HIGH/BLOCKER that invalidates downstream work, genuine ADR/core-architecture conflict, authorization boundary, production impact, missing credential/third-party authority, repository breakage, runtime/tool-budget termination or real queue exhaustion is a stop condition.

### A1r — Finish actual response-I/O deadline enforcement without leaking socket state

**Status:** `READY_LOCAL`; immediate priority.

Narrow repair only:

1. TCP: recompute remaining absolute permit time before every blocking partial-write syscall; no fresh 100 ms window after progress.
2. TCP/UDP: zero remaining time before send is fail-closed; do not use `max(1)` to extend the deadline.
3. TCP/UDP: preserve and restore the socket's previous write timeout on successful reusable sockets; use a scoped restore shape where practical.
4. Preserve charged bytes/packets and terminal abandonment on partial/error/late send.
5. Keep all real responder call sites on the bounded helper.

Tests must cover partial writes that consume most of the deadline, exact/over boundary, zero-remaining start, restoration of preexisting timeout state, and no success evidence after failure. Run targeted tests, full `scripts/check.sh`, required fuzz gate if repository policy triggers it, and `git diff --check`.

Push normally and continue immediately.

**Continue immediately to B1v:** yes.

### B1v — Finish queue-expiry ownership verification

**Status:** `PREAUTHORIZED_AFTER_A1r`.

Complete the accepted ownership matrix without redesign:

- exact idle boundary;
- exact lifetime boundary;
- promotion/authentication;
- cancellation/replacement;
- source queue max/max+1;
- global queue max/max+1 across distinct sources;
- ordinary expiry keeps responder loop alive;
- queue/state/memory release exactly once on every terminal path.

Run targeted/full gate, push, continue.

**Continue immediately to D1:** yes.

### D1 — Make every rejection class terminal across inner + outer accounting

**Status:** `PREAUTHORIZED_AFTER_B1v`.

Close all D019 rejection classes without changing candidate numeric limits or wire/session semantics.

Required regressions:

- inner per-state input rejection -> same logical ticket cannot continue;
- inner response/anti-amplification rejection -> same ticket cannot later send;
- global input/work rejection -> no revival after window rollover;
- source/global response rejection -> no later response;
- queue rejection -> no later enqueue;
- response deadline/I/O failure -> no later success;
- checked-add/deadline/clock-unmeasurable failure -> terminal rejection;
- cleanup remains one-shot and bounded;
- cross-layer rollback preserves truthful counters while never reviving the logical state.

Run targeted/full gate, push, continue.

**Continue immediately to E:** yes.

### E — Audit and machine-check charge ordering across every real responder

**Status:** `PREAUTHORIZED_AFTER_D1`.

Maintain a machine-checkable/static inventory for every externally reachable pre-auth responder:

1. typed carrier/source projection + state admission;
2. input byte/packet charge before parse;
3. parser/work reservation before protected work;
4. state memory reservation before owned allocation;
5. queue reservation before pending ownership;
6. response charge + bounded actual I/O before send;
7. terminal rejection/evidence barrier;
8. exactly-once cleanup.

Current conservative 64/4096 work reservations may remain only if they dominate bounded parser work; they are accounting units, not measured CPU cycles. Fix concrete uncovered seams only. Add a guard/test so a new externally reachable responder cannot silently bypass admission.

Run full gate, push, continue.

**Continue immediately to C1:** yes.

### C1 — Make carrier/source projection explicit and bounded

**Status:** `PREAUTHORIZED_AFTER_E`.

Implement only the noncontroversial projection portion:

- explicit bounded carrier discriminator at least distinguishing current TCP and UDP pre-auth sources;
- one bounded unknown/unusable-source bucket only where a live call site genuinely needs it;
- deterministic non-collision tests across family/address/port/carrier projection;
- no raw source logging or sensitive topology disclosure;
- no new retention duration/history ceiling.

Run targeted/full gate, push, continue to C2.

### C2 — Resolve terminal-source persistence semantics or produce ADR amendment request

**Status:** `ADR_CHECKPOINT_AFTER_C1`.

Re-read D019 and adjacent reviewed decisions after the implementation inventory is concrete.

Do not retain all terminal sources forever. Do not invent TTL/LRU/history counts. If reviewed text still cannot reconcile the no-reset rule with bounded source-accounting storage, write a compact ADR amendment request containing the exact conflicting clauses, attack/resource rationale, feasible policy shapes without convenience numbers, and dependent tests/evidence.

Stop only this policy-dependent lane if a maintainer/reviewer choice is genuinely required. Do not falsely mark D019 complete. H/I remain available as independent fallback work.

**Continue to F only when C2 is resolved by reviewed policy:** yes.

### F — Complete the full D019 adversarial/evidence-barrier matrix

**Status:** `PREAUTHORIZED_AFTER_C2`.

Cover:

- source/global concurrency max/max+1;
- source-lifetime input bytes/packets/work under resolved C semantics;
- global one-second input/work/response windows;
- per-packet work ceiling;
- state/global memory;
- source/global pending queue;
- source/global response + inner 3x anti-amplification;
- idle 1 s / lifetime 5 s / response-send 100 ms through deterministic time/I/O controls;
- checked arithmetic/clock overflow;
- terminal non-revival;
- retry/reconnect/carrier transition persistence;
- cancellation/timeout/double cleanup;
- no Session/Path/Delivery/readiness/authz-equivalent evidence on rejection;
- secret-safe bounded diagnostics.

Do not substitute VPS/load tests for deterministic accounting correctness. Full gate, commit, push; exact repair-head CI must be green before security closure.

**Continue immediately to G after exact-head CI green:** yes.

### G — Fresh exact-tree D019/security evidence review

**Status:** `PREAUTHORIZED_AFTER_F`.

Independently re-read the exact implementation and tests, then correct:

- `docs/reviews/resource-abuse-evidence-2026-09-04.md`;
- `docs/release-security-review-packet.md`;
- `docs/status.md`;
- release closure/navigation records.

RSEC-001 may close as an implementation finding only when A1r/B1v/D1/E/C1/C2/F are actually satisfied and exact-head CI is green. Independent external/two-person security review remains a separate release gate. Never promote RC/production/freeze/release automatically.

**Continue immediately to H if no new HIGH/BLOCKER:** yes.

### H — Compatibility / freeze-boundary review

**Status:** `READY_LOCAL_AFTER_G`; also safe fallback if C2 is externally waiting.

Audit corpus-v1 content-addressed freeze vs global protocol non-freeze, current/current negotiation, unsupported/future rejection, downgrade/transcript binding into Noise, resume/version binding, replay boundary and stale wording implying corpus freeze == protocol/release freeze.

Add a regression only for a concrete defect. Do not reopen frozen corpus bytes without correctness evidence.

**Continue immediately to I:** yes.

### I — Package/operator and evidence-provenance integrity review

**Status:** `READY_LOCAL_AFTER_H`; safe independent fallback if C2 is externally waiting.

Verify existing bounded evidence for x86_64 package/build identity, install/readiness/smoke/upgrade/rollback, retained external state without reading protected identity material, shutdown/listener/temp cleanup, canonical Git-blob/checksum manifests, exact-head CI references and stale release-packet links/hashes.

Do not rerun already-sufficient VPS/package work merely for freshness. Fix only concrete defects.

**Continue immediately to J:** yes.

### J — Reclassify release opportunities and reconsider VPS

**Status:** `READY_LOCAL_AFTER_I`, with live classification bounded by unresolved C2/F/G dependencies if any.

Re-evaluate every release-closure row:

- answered bounded question -> `ALREADY_SUFFICIENT_FOR_BOUNDED_QUESTION`;
- concrete executable missing assertion -> `OPEN_READY` with exact evidence/action/dependencies/scope;
- environment/governance/implementation dependency -> truthful blocked class.

Then use the rented VPS only if a genuine dependency-ready missing live release question exists under standing authorization. Otherwise record `READY_LIVE: none`. No unchanged retry of closed repeated/periodic/HY2 lines.

## Completion gates

RSEC-001 implementation closure requires all of:

- exact implementation HEAD passes full stable repository gate and required fuzz gate;
- 100 ms response permit bounds actual response I/O across partial writes without leaking temporary socket timeout state;
- application pending ownership cannot outlive/diverge from process queue/state reservation;
- queue reservations are charged before ownership and released exactly once across promotion/rejection/idle/lifetime/cancel/shutdown;
- every rejection class, including inner-budget and arithmetic/unmeasurable failure, makes the logical state non-revivable;
- charge ordering is auditable across every externally reachable responder;
- carrier/source projection and persistence are reconciled with D019 under a bounded reviewed policy;
- deterministic boundary/overflow/timeout/evidence-barrier matrix passes;
- security-review prose names the exact reviewed tree and does not outrun implementation.

The broader rolling queue remains active through H-J unless a real stop condition occurs.

## Do not expand into

- public/production listener deployment;
- new numeric D019 retention/source-table ceilings without explicit reviewed ADR amendment;
- protocol/wire/Noise/Session/Carrier redesign unrelated to these concrete admission defects;
- VPS load testing as a substitute for deterministic security accounting;
- endless generic harness/adversarial review after stated gates are satisfied;
- reopening frozen corpus bytes without correctness evidence;
- speculative FEC/0-RTT/striping/multipath/exotic-carrier work;
- third-party targets/scanning;
- production route/firewall/DNS/proxy/tunnel/qdisc changes;
- reading/copying/hashing/committing protected identity/secrets/private topology.

## Questions requiring maintainer decision

No immediate administrator action is required for A1r/B1v/D1/E/C1.

A maintainer/reviewer decision is required only if C2 reaches the already-identified source-retention ADR conflict and no existing reviewed text provides a bounded policy. At that point present exact policy options/trade-offs; do not invent a numeric retention rule autonomously.