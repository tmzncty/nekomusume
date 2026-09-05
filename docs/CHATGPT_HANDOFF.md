# Nekomusume ChatGPT Handoff

Checked at: 2026-09-05 08:00 Asia/Shanghai
Repository HEAD reviewed: `f99c52c0d6c12a4a60e4d909e40230eca2131bed`
Previous reviewed implementation HEAD: `5ffc277c9faef56b541d0e84f16951cc88264abc`
Previous reviewer handoff commit: `f902c36cfacae2b487fc47b0db10fb855c32cd13`

## What changed

One coding-agent commit landed after the previous reviewer handoff:

- `f99c52c` — **deterministic expiry-fixture repair only; no production/runtime/wire/Noise/Session behavior change.** It removes the premature manual release of the state whose lifetime is being tested, keeps a distinct live `other` state, advances that state at monotonic time 100, and checks expiry at absolute 210 so the original state created at 10 reaches the exact 200 ms inclusive lifetime boundary while the later state remains live.

Both `main` and `work/continue-20260904` now point at exact `f99c52c`.

### Exact-head CI

Exact `f99c52c` is independently green on both refs.

- main Rust CI run `33928230290` — `stable checks` succeeded, including `bash scripts/check.sh`; `nightly decode fuzz smoke` succeeded, including the pinned 30-second / 8192-byte decode fuzz run.
- work-branch Rust CI run `33928237749` — overall `success` on the same exact tree.

This closes the previous CI-002 fixture/gate defect. It is repository CI evidence, not security approval.

No new correctness/security blocker is introduced by `f99c52c`. The substantive D019 findings remain A1, B1 verification, D1, E and the isolated C ADR checkpoint.

## Review verdict

**CONTINUE_WITH_REQUIRED_FIXES — Q0/CI-002 is closed and exact-head CI is green. Proceed directly into A1 actual response-I/O deadline enforcement, then B1v, D1, E and the isolated C checkpoint without waiting for another reviewer interval.**

No administrator action is required now. Do not spend VPS time on this deterministic security-accounting lane while A1/D1 remain open.

## Reviewer findings

### CI-002 — CLOSED

`f99c52c` repairs the test timeline without weakening production expiry semantics. Exact-head stable CI and required fuzz smoke are green. Do not reopen this fixture unless a later semantic change invalidates the boundary.

### RSEC-001A1 — HIGH — 100 ms response permit still does not bound actual I/O

The current production response shape remains effectively:

```text
charge_response -> socket write/send -> complete_response(now)
```

A blocking or partial socket write can therefore emit bytes before a post-I/O completion check notices that the exact D019 100 ms response permit expired.

Required repair:

- preserve the existing one-shot `PreauthResponsePermit` and exact 100 ms D019 value;
- derive one absolute monotonic deadline from response admission;
- TCP complete-frame writes must share that one absolute deadline across every partial write; never reset a fresh 100 ms timeout after partial progress;
- UDP sends must execute under the same remaining permit budget;
- partial/timeout/error attempts remain charged and cannot emit negotiation/authentication/readiness/Session success evidence;
- completion or abandonment consumes response ownership exactly once;
- deterministic injected writer/sink tests must prove inside-budget success, exact-boundary behavior and over-budget abandonment without wall-clock sleeps;
- affected real responder call sites must use the bounded helper rather than retaining a post-send-only completion path.

Do not substitute an outer seconds-long socket timeout or another post-send bookkeeping check.

### RSEC-001B1 — implementation direction accepted; verification still incomplete

`ea5b257` remains the accepted one-owner direction for process expiry and application pending ownership. Do not redesign it merely because the queue is now green.

Still required before B is evidence-complete:

- idle expiry at exact D019 1 s boundary while five-second lifetime has not elapsed;
- lifetime expiry;
- successful authentication/promotion;
- cancellation/replacement;
- source/global queue max/max+1 across distinct source projections;
- ordinary expiry leaves the bounded server loop alive;
- no double dequeue/release and exact queue/memory accounting across every terminal path.

### RSEC-001D1 — HIGH — terminal rejection remains incomplete across inner and arithmetic failures

The process-level rejected state added by `b3490a2` is useful, but D019 requires every exhausted, saturated, unmeasurable, timed-out, malformed or over-limit operation on one logical pre-auth state to become terminal.

Remaining concrete gaps to close:

1. `ListenerAdmission` can still return immediately when inner `PreauthBudget` rejects input/response before the outer process state is marked rejected.
2. State-associated checked arithmetic/deadline construction can still fail before an explicit terminal reject path.

Required repair:

- inner per-state rejection must explicitly consume/mark the associated process state terminal;
- state-associated arithmetic/unmeasurable/deadline failures must route through the terminal-rejection path;
- preserve cross-layer accounting atomicity while making the logical state non-revivable;
- regress inner input rejection, inner response/anti-amplification rejection, global window rejection, queue rejection, response deadline rejection and arithmetic/deadline overflow;
- prove the same logical state cannot become usable after one-second window rollover.

### RSEC-001C — isolated ADR checkpoint remains unchanged

D019 states that counters are not reset by retry, reconnect, carrier change, identity change or error. The current source accounting row is removed after its final live state is released. Retaining terminal source rows forever would itself create an unbounded source-accounting map, while existing D019 text defines no terminal-source retention TTL, history ceiling or eviction policy.

Do not invent a numeric retention policy. First make the carrier/source projection explicit and bounded. After E, determine whether existing reviewed text provides a bounded interpretation. If not, write an ADR amendment request with exact conflict/options/trade-offs and stop **only C**; independent H/I work may remain available if it does not depend on the unresolved source-retention semantics.

## Evidence boundaries

- `IMPLEMENTATION_COMPLETE=true` remains a bounded research-baseline flag only.
- `CANONICAL_CORPUS_V1_FROZEN=true` remains corpus-specific only.
- `RELEASE_CANDIDATE=false`, `PRODUCTION_READY=false`, `FREEZE=false`, `RELEASED=false` remain required.
- Exact `f99c52c` changes one deterministic test fixture only; it closes CI-002 but does not close A1, B1v, D1, E or C.
- Exact `f99c52c` stable checks and nightly decode fuzz smoke are green on `main`; the same tree also has a green work-branch run.
- A post-send deadline check is not bounded response I/O.
- Process admission does not claim to bound kernel SYN backlog, provider NAT state or resources outside the process.
- Existing inner `PreauthBudget` remains useful and must not be weakened to simplify outer accounting.
- No VPS/load run substitutes for deterministic D019 counter, ownership and evidence-barrier correctness.
- Standing VPS authorization remains valid for genuinely READY self-owned TCP/UDP work, but the corrected release/security state currently gives this deterministic local lane precedence.
- Protected identity/secrets/private endpoint material remain unread/untracked/uncommitted.

## Rolling Work Queue

This is a rolling multi-hour queue. Finish one coherent slice -> run required targeted/full gates -> commit -> push -> immediately consume the next dependency-satisfied slice. Do not stop for a reviewer interval. Only a new HIGH/BLOCKER that invalidates downstream work, a genuine ADR/core-architecture conflict, authorization boundary, production impact, missing credential/third-party authority, repository breakage, runtime/tool-budget termination or real queue exhaustion is a stop condition.

### A1 — Bind response permits to actual I/O

**Status:** `READY_LOCAL`; immediate substantive priority.

Implement actual bounded I/O using one absolute 100 ms monotonic deadline from response admission.

Required coverage:

- ordinary TCP probe response;
- ordinary UDP probe response;
- periodic TCP pre-auth response;
- multistream TCP pre-auth response;
- failover UDP selection / Noise pre-auth response;
- failover TCP negotiation / Noise pre-auth response.

TCP framing must retain one deadline across partial writes. UDP send must execute under remaining budget. Failed/partial/late sends stay charged, consume the permit and emit no success evidence.

Tests: deterministic injected writer/sink or equivalent controllable I/O abstraction proving inside-budget, exact-boundary, partial-write and over-budget cases without sleeps; affected CLI/integration coverage; full `scripts/check.sh`; `git diff --check`.

Push normally and continue.

**Continue immediately to B1v:** yes.

### B1v — Finish queue-expiry ownership verification

**Status:** `PREAUTHORIZED_AFTER_A1`.

Do not redesign the accepted `ea5b257` ownership shape unless tests falsify it. Complete the deterministic matrix:

- exact idle boundary;
- exact lifetime boundary;
- promotion/authentication;
- cancellation/replacement;
- source queue max/max+1;
- global queue max/max+1 across distinct sources;
- ordinary expiry keeps bounded responder/server loop alive;
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
- queue saturation rejection -> no later enqueue on rejected state;
- response deadline failure -> no later success;
- checked-add/deadline/clock-unmeasurable failure -> terminal rejection;
- cleanup remains one-shot and bounded;
- cross-layer rollback preserves truthful counters while never reviving the logical state.

Run targeted/full gate, push, continue.

**Continue immediately to E:** yes.

### E — Audit and machine-check charge ordering across every real responder

**Status:** `PREAUTHORIZED_AFTER_D1`.

For every externally reachable pre-auth responder, maintain a machine-checkable/static inventory of:

1. typed carrier/source projection + state admission;
2. input byte/packet charge before parse;
3. parser/work reservation before protected work;
4. state memory reservation before owned allocation;
5. queue reservation before pending ownership;
6. response charge + actual bounded response I/O before send;
7. terminal rejection/evidence barrier;
8. exactly-once cleanup.

Current conservative 64/4096 work reservations may remain only if they dominate bounded parser work; they are accounting units, not measured CPU cycles. Fix concrete uncovered seams only. Add a guard/test so a new externally reachable responder cannot silently bypass admission.

Run full gate, push, continue.

**Continue immediately to C1:** yes.

### C1 — Make carrier/source projection explicit and bounded

**Status:** `PREAUTHORIZED_AFTER_E`.

Implement only the noncontroversial projection portion before policy resolution:

- explicit bounded carrier discriminator at least distinguishing current TCP and UDP pre-auth sources;
- one bounded unknown/unusable-source bucket where a live call site genuinely needs it;
- deterministic non-collision tests across family/address/port/carrier projection;
- no raw source logging or sensitive topology disclosure;
- no new retention duration/history ceiling.

Run targeted/full gate, push, continue to C2.

### C2 — Resolve terminal-source persistence semantics or produce ADR amendment request

**Status:** `ADR_CHECKPOINT_AFTER_C1`.

Re-read D019 and adjacent reviewed decisions after the implementation inventory is concrete.

Do not retain all terminal sources forever. Do not invent TTL/LRU/history counts. If existing reviewed text still cannot reconcile the no-reset rule with bounded source-accounting storage, write a compact ADR amendment request containing:

- exact conflicting clauses;
- attack/resource reason both requirements matter;
- feasible policy shapes without choosing numeric values by convenience;
- which later tests/evidence depend on the decision.

Stop **only this policy-dependent lane** if a maintainer/reviewer choice is genuinely required. Do not falsely mark D019 complete.

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

Independently re-read the exact implementation and tests. Then correct:

- `docs/reviews/resource-abuse-evidence-2026-09-04.md`;
- `docs/release-security-review-packet.md`;
- `docs/status.md`;
- release closure/navigation records.

RSEC-001 may close as an implementation finding only when A1/B1v/D1/E/C1/C2/F are actually satisfied and exact-head CI is green. Independent external/two-person security review remains a separate release gate. Never promote RC/production/freeze/release automatically.

**Continue immediately to H if no new HIGH/BLOCKER:** yes.

### H — Compatibility / freeze-boundary review

**Status:** `READY_LOCAL_AFTER_G`; also safe fallback research/review if C2 is externally waiting and no D019-dependent mutation is attempted.

Audit corpus-v1 content-addressed freeze vs global protocol non-freeze, current/current negotiation, unsupported/future rejection, downgrade/transcript binding into Noise, resume/version binding, replay boundary and stale wording implying corpus freeze == protocol/release freeze.

Add a regression only for a concrete defect. Do not reopen frozen corpus bytes without correctness evidence.

**Continue immediately to I:** yes.

### I — Package/operator and evidence-provenance integrity review

**Status:** `READY_LOCAL_AFTER_H`; safe independent fallback if C2 is externally waiting.

Verify existing bounded evidence for x86_64 package/build identity, install/readiness/smoke/upgrade/rollback, retained external state without reading protected identity material, shutdown/listener/temp cleanup, canonical Git-blob/checksum manifests, exact-head CI references and stale release-packet links/hashes.

Do not rerun already-sufficient VPS/package work merely for freshness. Fix only concrete defects.

**Continue immediately to J:** yes.

### J — Reclassify release opportunities and reconsider VPS

**Status:** `READY_LOCAL_AFTER_I`, but final live classification must remain truthful to unresolved C2/F/G dependencies if any.

Re-evaluate every release-closure row:

- answered bounded question -> `ALREADY_SUFFICIENT_FOR_BOUNDED_QUESTION`;
- concrete executable missing assertion -> `OPEN_READY` with exact evidence/action/dependencies/scope;
- environment/governance/implementation dependency -> truthful blocked class.

Then use the rented VPS only if a genuine dependency-ready missing live release question exists under standing authorization. Otherwise record `READY_LIVE: none`. No unchanged retry of closed repeated/periodic/HY2 lines.

## Completion gates

RSEC-001 implementation closure requires all of:

- exact implementation HEAD passes full stable repository gate and required fuzz gate;
- 100 ms response permit bounds actual response I/O, not only post-send bookkeeping;
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
- endless generic harness/adversarial review after the stated gates are satisfied;
- reopening frozen corpus bytes without correctness evidence;
- speculative FEC/0-RTT/striping/multipath/exotic-carrier work;
- third-party targets/scanning;
- production route/firewall/DNS/proxy/tunnel/qdisc changes;
- reading/copying/hashing/committing protected identity/secrets/private topology.

## Questions requiring maintainer decision

No immediate administrator action is required for A1/B1v/D1/E/C1.

A maintainer/reviewer decision is required only if C2 reaches the already-identified source-retention ADR conflict and no existing reviewed text provides a bounded policy. At that point present exact policy options/trade-offs; do not invent a numeric retention rule autonomously.
