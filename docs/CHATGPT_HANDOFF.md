# Nekomusume ChatGPT Handoff

Checked at: 2026-09-05 04:58 Asia/Shanghai
Repository HEAD reviewed: `5ffc277c9faef56b541d0e84f16951cc88264abc`
Previous reviewed implementation HEAD: `b3490a2e4b5405f102c45e7463ddbbc7b8192e1c`
Previous reviewer handoff commit: `d9fde3608abca79e009f0bfe198501b8aface02a`

## What changed

One coding-agent commit landed after the previous reviewer handoff:

- `5ffc277` — **test-fixture adjustment only; no production/runtime/wire/Noise/Session behavior change.** It reformats the prior terminal-state assertion, separates the already-rejected state from a fresh source used for response/queue checks, and moves part of the lifetime/window fixture forward in monotonic time.

Both `main` and `work/continue-20260904` currently point at exact `5ffc277`.

### Exact-head CI

Exact `5ffc277` CI is still **red on stable checks and green on nightly decode fuzz smoke** on both refs:

- main run `33914456554` — stable checks failed; nightly decode fuzz smoke succeeded;
- work-branch run `33914467976` — same exact tree and same overall outcome.

The failure is now independently localized from the GitHub job log. Formatting is no longer the problem: `scripts/check.sh` reaches the Rust unit tests, and all shown workspace/integration tests pass except one deterministic `neko-crypto` fixture:

```text
preauth_tests::process_window_queue_and_lifetime_fail_closed
left:  Ok(0)
right: Ok(1)
crates/neko-crypto/src/lib.rs:1495
```

The fixture admits the final `other` state at monotonic time `20`, charges input at `20`, and uses test limits `idle_timeout_ms = 200` and `max_lifetime_ms = 200`; therefore `expire(200)` is only 180 ms after creation/progress and correctly returns zero expired states. The first inclusive expiry boundary for that state is `220`. This is a test-expectation/timeline defect, not evidence that `expire_states` itself failed its documented boundary.

Do not weaken the production expiry rule to satisfy this fixture. Repair the fixture/timeline so it asserts the intended 200 ms inclusive boundary from the state’s actual creation/progress time, then rerun the exact gate.

## Review verdict

**CONTINUE_WITH_REQUIRED_FIXES — `5ffc277` is a useful fixture cleanup attempt but exact-head stable CI remains red for one deterministic timestamp expectation. The previous A1 and D1 HIGH findings remain open because this commit is test-only. Fix the fixture immediately, then continue the existing multi-hour D019 queue without waiting for another reviewer interval.**

No administrator action is required. Do not spend VPS time on this deterministic security-accounting lane while A1/D1 remain open.

## Reviewer findings

### CI-002 — REQUIRED GATE REPAIR — expiry fixture uses the wrong absolute boundary

The prior rustfmt-only failure is gone. Exact `5ffc277` now compiles and runs the test suite, but the revised fixture expects one state to expire at absolute `200` even though that state was created/progressed at `20` under a 200 ms idle/lifetime limit.

Required repair:

- preserve the implementation semantics that expiry is measured from the state’s actual creation/last-progress timestamp;
- change the fixture so its terminal assertion is made at the true inclusive boundary (`220` for the current test timeline), or restructure the fixture so creation/progress occurs at `0` if absolute `200` is the intended boundary;
- retain a boundary assertion proving one millisecond before expiry remains live and exact-boundary expiry is terminal where practical;
- run the targeted `neko-crypto` test, full `scripts/check.sh`, and `git diff --check`;
- push normally and inspect exact-head CI.

This should be folded into the next coherent A1/B1v/D1 work rather than turned into a standalone waiting cycle.

### RSEC-001A1 — HIGH — 100 ms response permit still does not bound actual I/O

`5ffc277` changes tests only. The production response path remains semantically equivalent to:

```text
charge_response -> socket write/send -> complete_response(now)
```

A blocking/partial socket write can therefore emit bytes before a post-I/O deadline check discovers that the 100 ms D019 permit expired.

Required repair remains:

- preserve the exact D019 100 ms value and one-shot `PreauthResponsePermit` ownership;
- derive remaining time from the same monotonic clock **before and during** the real I/O attempt;
- TCP complete-frame response writes must use one absolute deadline across all partial writes; a per-call timeout that resets after partial progress is insufficient;
- UDP sends must execute under the remaining permit budget as well;
- partial/timeout/error attempts stay charged and emit no negotiation/authentication/readiness/Session success evidence;
- completion/abandonment consumes response ownership exactly once;
- deterministic injected-writer/sink coverage must prove inside-budget success, exact-boundary semantics and over-budget abandonment without wall-clock sleeps.

Do not substitute a seconds-long outer socket timeout or another post-send bookkeeping check.

### RSEC-001B1 — implementation shape accepted; verification still incomplete

`ea5b257` remains accepted as the correct one-owner direction for process expiry and application pending ownership. The queue/state lifetime should not be redesigned unless deterministic tests falsify it.

Still required before B is evidence-complete:

- idle expiry at the exact D019 1 s boundary while five-second lifetime has not elapsed;
- lifetime expiry;
- successful authentication/promotion;
- cancellation/replacement;
- source/global queue max/max+1 across distinct source projections;
- ordinary expiry leaves the bounded server loop alive;
- no double dequeue/release and exact queue/memory accounting across every terminal path.

### RSEC-001D1 — HIGH — terminal rejection is incomplete across inner and arithmetic failures

`5ffc277` does not change this finding.

The process-level rejected bit added by `b3490a2` is useful, but D019 requires every exhausted/saturated/unmeasurable/timed-out/malformed/over-limit operation on a logical pre-auth state to become terminal.

Concrete remaining gaps:

1. `ListenerAdmission` can return immediately when the inner `PreauthBudget` rejects input/response before the outer process state is marked rejected.
2. checked arithmetic/deadline construction still has `ok_or(SessionRejected)?`-style paths that can fail before the explicit `reject(id)` branch.

Required repair:

- make inner per-state rejection explicitly consume/mark the associated process state terminal;
- route state-associated arithmetic/unmeasurable failures through the terminal-rejection path;
- preserve cross-layer accounting atomicity while keeping the logical state non-revivable;
- regress inner input rejection, inner response/anti-amplification rejection, global window rejection, queue rejection, response deadline rejection and arithmetic/deadline overflow;
- prove the same logical state cannot become usable after a one-second window rollover.

### RSEC-001C — isolated ADR checkpoint remains unchanged

The source/carrier persistence conflict still must not block A1/B1v/D1/E:

- D019 says counters are not reset by retry, reconnect, carrier change, identity change or error;
- current source rows disappear when the final live state is released;
- retaining terminal source rows forever would create an unbounded source-accounting map;
- existing D019 text defines no terminal-source retention TTL/history ceiling/LRU policy.

Do not invent a numeric retention policy. First make the carrier/source projection explicit and bounded. If reviewed text still provides no bounded terminal-retention interpretation after E, write the ADR amendment request and stop only C.

## Evidence boundaries

- `IMPLEMENTATION_COMPLETE=true` remains a bounded research-baseline flag only.
- `CANONICAL_CORPUS_V1_FROZEN=true` remains corpus-specific only.
- `RELEASE_CANDIDATE=false`, `PRODUCTION_READY=false`, `FREEZE=false`, `RELEASED=false` remain required.
- Exact `5ffc277` is test-only relative to `d9fde36`; it does not close A1, B1v, D1, E or C.
- Exact `5ffc277` stable CI is red on one deterministic unit-test expectation; nightly decode fuzz smoke is green. Do not describe the tree as fully green.
- The failing `expire(200)` expectation is inconsistent with the fixture’s state creation/progress at `20` and 200 ms limits; do not change production expiry semantics merely to make that assertion pass.
- A post-send deadline check is not bounded response I/O.
- Process admission does not claim to bound kernel SYN backlog, provider NAT state or resources outside the process.
- Existing inner `PreauthBudget` remains useful and must not be weakened to simplify outer accounting.
- No VPS/load run substitutes for deterministic D019 counter, ownership and evidence-barrier correctness.
- Standing VPS authorization remains valid for genuinely READY self-owned TCP/UDP work, but the corrected release/security state currently gives this deterministic local lane precedence.
- Protected identity/secrets/private endpoint material remain unread/untracked/uncommitted.

## Rolling Work Queue

This is a rolling multi-hour queue. Finish a coherent slice -> run its targeted/full gates -> commit -> push -> immediately consume the next dependency-satisfied slice. Do not stop for a reviewer interval. Only a new HIGH/BLOCKER that invalidates downstream work, a genuine ADR/core-architecture conflict, authorization boundary, production impact, missing credential/third-party authority, repository breakage, runtime/tool-budget termination or real queue exhaustion is a stop condition.

### Q0 — Repair the exact-head deterministic expiry fixture

**Status:** `READY_LOCAL`; immediate first action, expected to be tiny.

Correct `process_window_queue_and_lifetime_fail_closed` without weakening implementation semantics. The current final state is created/progressed at `20`; under 200 ms idle/lifetime limits it is not expired at `200` and reaches the inclusive boundary at `220`.

Prefer explicit boundary coverage (`219` live, `220` expired) or an equivalent timeline that makes the intended origin obvious.

Verification: targeted `neko-crypto` test + full `scripts/check.sh` + `git diff --check`. Push. Do **not** wait after this small gate repair.

**Continue immediately to A1:** yes.

### A1 — Bind response permits to actual I/O

**Status:** `READY_LOCAL_AFTER_Q0`; highest substantive priority.

Required behavior:

- all ordinary TCP/UDP probe, periodic TCP, multistream TCP, failover UDP selection/Noise and failover TCP negotiation/Noise pre-auth response paths use bounded I/O under the exact 100 ms permit;
- TCP framing uses one total monotonic deadline across partial writes;
- UDP response send is permit-bounded;
- failed/partial/late send produces no success evidence and response accounting remains charged;
- completion/abandonment is one-shot;
- no new wire/protocol numeric value.

Verification: deterministic injected sink/writer tests + affected CLI integration tests + full gate. Push and continue; exact CI may run while independent B1v/D1 preparation proceeds, but security closure later requires green exact-head CI.

**Continue immediately to B1v:** yes.

### B1v — Finish queue-expiry ownership verification

**Status:** `PREAUTHORIZED_AFTER_A1`.

Do not redesign `ea5b257` unless a deterministic test falsifies it. Complete the missing matrix: idle exact-boundary expiry, lifetime expiry, promotion, cancellation/replacement, source/global queue max/max+1, ordinary server-loop continuation and exactly-once state/queue/memory cleanup.

**Continue immediately to D1:** yes.

### D1 — Make every rejection class terminal across inner + outer accounting

**Status:** `PREAUTHORIZED_AFTER_B1v`.

Close all D019 rejection classes without changing candidate numeric limits or wire/session semantics.

Required regressions:

- inner per-state input rejection -> same logical ticket cannot continue;
- inner response/anti-amplification rejection -> same ticket cannot later send;
- global input/work rejection -> no revival after window rollover;
- source/global response rejection -> no later response;
- queue saturation rejection -> no later enqueue on the rejected state;
- response deadline failure -> no later success;
- checked-add/deadline overflow -> terminal rejection;
- cleanup remains one-shot and bounded.

Run targeted/full gate, push, continue.

**Continue immediately to E:** yes.

### E — Audit and machine-check charge ordering across every real responder

**Status:** `PREAUTHORIZED_AFTER_D1`.

For every externally reachable pre-auth responder, machine-check or maintain a static inventory of:

1. typed carrier/source projection + state admission;
2. input byte/packet charge before parse;
3. parser/work reservation before work;
4. state memory reservation before owned allocation;
5. queue reservation before pending ownership;
6. response charge + actual bounded response I/O before send;
7. terminal rejection/evidence barrier;
8. exactly-once cleanup.

Current conservative 64/4096 work reservations may remain only if they dominate bounded parser work; they are accounting units, not measured CPU cycles. Fix concrete uncovered seams only. Add a guard so a newly externally reachable responder cannot silently bypass admission.

**Continue immediately to C:** yes.

### C — Resolve carrier/source projection and bounded persistence semantics

**Status:** `ADR_CHECKPOINT_AFTER_E`.

First implement the noncontroversial typed projection: explicit bounded carrier discriminator (`TCP`, `UDP`, one bounded unknown bucket where applicable), deterministic non-collision tests and no raw source logging.

Then evaluate terminal source persistence. Do not retain all sources forever and do not invent TTL/LRU/history limits. If reviewed text still lacks a bounded interpretation of the D019 no-reset rule, write a compact ADR amendment request with exact conflict/options/trade-offs and stop **only C** for policy review.

**Continue immediately to F only after C is resolved:** yes.

### F — Complete the full D019 adversarial/evidence-barrier matrix

**Status:** `PREAUTHORIZED_AFTER_C`.

Cover source/global concurrency, source-lifetime input/packet/work under resolved C semantics, global one-second windows, per-packet work, state/global memory, source/global queue, source/global response + inner 3x anti-amplification, idle/lifetime/response deadlines with injectable time, overflow, terminal non-revival, retry/reconnect/carrier transition, cancellation/timeout/double cleanup, no Session/Path/Delivery/readiness/authz-equivalent evidence on rejection, and secret-safe diagnostics.

Do not substitute VPS/load tests for deterministic accounting correctness. Full local gate, commit, push; exact repair-head CI must be green before security closure.

**Continue immediately to G after exact-head CI green:** yes.

### G — Fresh exact-tree D019/security evidence review

**Status:** `PREAUTHORIZED_AFTER_F`.

Re-read the exact implementation/tests, then correct `docs/reviews/resource-abuse-evidence-2026-09-04.md`, `docs/release-security-review-packet.md`, `docs/status.md` and closure/navigation records to name the actual reviewed implementation head.

RSEC-001 closes as an implementation finding only when Q0/A1/B1v/D1/E/C/F are actually satisfied. Independent external/two-person security review remains a separate release gate. Do not promote RC/production/freeze/release automatically.

**Continue immediately to H if no new HIGH/BLOCKER:** yes.

### H — Compatibility / freeze-boundary review

**Status:** `READY_LOCAL_AFTER_G`.

Audit corpus-v1 content-addressed freeze vs global protocol non-freeze, current/current negotiation, unsupported/future rejection, downgrade/transcript binding into Noise, resume/version binding and replay boundary, plus stale wording that implies corpus freeze == protocol/release freeze. Add a regression only for a real defect; do not reopen frozen corpus bytes without correctness evidence.

**Continue immediately to I:** yes.

### I — Package/operator and evidence-provenance integrity review

**Status:** `READY_LOCAL_AFTER_H`.

Verify existing bounded evidence for x86_64 package/build identity, install/readiness/smoke/upgrade/rollback, retained external state without reading protected identity material, shutdown/listener/temp cleanup, canonical Git-blob/checksum manifests, exact-head CI references and stale release-packet links/hashes. Do not rerun already-sufficient VPS/package work merely for freshness.

**Continue immediately to J:** yes.

### J — Reclassify release opportunities and reconsider VPS

**Status:** `READY_LOCAL_AFTER_I`.

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

No immediate administrator action is required for Q0/A1/B1v/D1/E.

A maintainer/reviewer decision is required only if C reaches the already-identified source-retention ADR conflict and no existing reviewed text provides a bounded policy. At that point present the exact policy options/trade-offs; do not invent a numeric retention rule autonomously.
