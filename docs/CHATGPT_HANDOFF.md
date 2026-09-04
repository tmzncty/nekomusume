# Nekomusume ChatGPT Handoff

Checked at: 2026-09-05 02:02 Asia/Shanghai
Repository HEAD reviewed: `b3490a2e4b5405f102c45e7463ddbbc7b8192e1c`
Previous reviewed implementation HEAD: `96ba80fe876021998b30c5cacfeaa756100789bc`
Previous reviewer handoff commit: `ce5522698cbfc9f94c1f2f202e71dd58e798c3be`

## What changed

Two coding-agent commits landed after the previous handoff:

- `ea5b257` — **queue/state expiry ownership repair; no wire/Noise/Session semantics change.** `ProcessPreauthAdmission::expire_states` now returns the exact expired state IDs; the failover UDP application pending object no longer owns an independent five-second timer, and a process-expired pending negotiation invalidates its application queue reservation instead of attempting a stale dequeue/release. This directly addresses the split-ownership defect found in RSEC-001B1.
- `b3490a2` — **terminal-rejection hardening; no wire/Noise/Session semantics change.** Process pre-auth states now carry a `rejected` bit, and process-level limit rejection in `charge_input`, `enqueue`, `charge_response`, plus a late `complete_response`, marks the state rejected so normal later operations cannot revive it after a one-second window rollover.

The current exact HEAD is on both `main` and `work/continue-20260904`.

### Exact-head CI

Exact `b3490a2` CI is **red on stable checks, green on nightly decode fuzz smoke** on both refs. The main run is `33899584402`; `stable checks` fails immediately in `scripts/check.sh` because rustfmt wants one assertion reformatted in `crates/neko-crypto/src/lib.rs` around the new response-deadline test. The nightly 30-second decode fuzz smoke succeeds.

This is a real exact-head gate failure, but the observed failure is formatting-only. It does not invalidate the semantic direction of `ea5b257` / `b3490a2`, and it is not a reason to stop consuming independent local READY work. The next coding commit must include the formatting repair and restore the full exact-head stable gate before any security-closure claim.

## Review verdict

**CONTINUE_WITH_REQUIRED_FIXES — B1 ownership repair is accepted in implementation shape, D made useful progress, but A1 remains HIGH and D is not yet complete across all rejection paths. Exact-head stable CI must be restored. Continue locally; do not use VPS for this deterministic security lane.**

The external agent should not wait for another reviewer interval. Fix the red formatting gate as part of the next coherent implementation slice, then continue A1 -> B1 verification -> D1 -> E. No administrator action is required unless the later C source-retention checkpoint reaches the already-identified ADR policy conflict.

## Reviewer findings

### CI-001 — REQUIRED GATE REPAIR — exact `b3490a2` stable checks fail rustfmt

GitHub Actions checked out exact `b3490a2` and `scripts/check.sh` stopped on a rustfmt diff in the new `response_deadline_rejection_makes_state_terminal` assertion. Fuzz passed.

Required action:

- run `cargo fmt --all` / the repository formatting gate;
- keep the semantic commits intact;
- rerun targeted tests plus the full repository gate;
- push normally and require exact-head CI green before G/security closure.

Do not spend a separate reviewer cycle or VPS run on this formatting-only repair.

### RSEC-001A1 — HIGH — 100 ms response permit still does not bound the actual I/O attempt

No new commit after `ce55226` changes this finding. Real response call sites still have the semantic shape:

```text
charge_response -> socket write/send -> complete_response(now)
```

A socket write/send that blocks beyond the 100 ms D019 permit can therefore emit bytes before the post-I/O completion check reports expiry. D019 requires the response-send deadline to bound the completed send attempt, with expiry abandoning success.

Required repair:

- preserve `PreauthResponsePermit` and the exact 100 ms value;
- derive the remaining permit budget **before each real I/O operation** from the same monotonic clock domain as `ListenerAdmission`;
- TCP complete-frame writes must use a helper whose total loop is bounded by the permit, not merely a per-call timeout that can reset for each partial write;
- UDP response sends must also execute under the permit budget; do not assume `send_to` is semantically instantaneous;
- partial/timeout/error sends remain charged attempts and produce no negotiation/authentication/readiness/Session success evidence;
- completion consumes the permit exactly once;
- add an injectable writer/sink or equivalent deterministic seam proving inside-budget success and over-budget failure without wall-clock sleeps.

Do not solve this by adding a seconds-long outer timeout or by moving the same post-send check around.

### RSEC-001B1 — implementation ownership defect repaired; verification remains incomplete

`ea5b257` removes the independent application timer and lets process expiry return state IDs that invalidate the matching application queue reservation. This is the correct ownership direction and closes the previously identified stale five-second application timer defect in implementation shape.

Before B is considered fully covered in the D019 evidence matrix, deterministic integration coverage still needs to prove:

- idle expiry at the D019 1 s boundary while the five-second lifetime has not elapsed;
- lifetime expiry;
- successful authentication/promotion;
- cancellation/replacement;
- source/global queue max/max+1 across distinct source projections;
- ordinary expiry leaves the bounded server loop alive;
- no double dequeue/release and exact queue/memory accounting after every terminal path.

These are verification obligations, not a reason to revert the one-owner model. Fold them into B1v/E/F rather than reopening a second queue subsystem.

### RSEC-001D1 — HIGH — terminal rejection is only enforced for some process-level failures

`b3490a2` adds a real `rejected` state bit and correctly makes several process-level ceiling/deadline failures non-revivable. However D019 says **any exhausted, saturated, unmeasurable, timed-out, malformed, or over-limit operation fails closed** and must not later succeed on the same logical pre-auth state.

Two concrete gaps remain:

1. **Inner `PreauthBudget` rejection does not reject the outer process state.** `ListenerAdmission::charge_input` and `charge_response` first call the inner per-state budget and return immediately on inner failure. The new `ProcessPreauthAdmission::reject(id)` path is never reached in that case, so the same `AdmissionTicket` still owns a process state that is not terminally rejected.
2. **Arithmetic/unmeasurable error paths can bypass `reject(id)`.** Several checked additions and deadline construction use `ok_or(SessionRejected)?` before reaching the explicit ceiling branch. Integer overflow is budget exhaustion under D019; it must not return a reusable live state.

Required repair:

- provide one explicit controller/ticket operation to mark/consume the process state rejected when an inner `PreauthBudget` operation fails;
- route checked-add/deadline-overflow failures through the same terminal rejection path whenever a state ID exists;
- preserve cross-layer atomic accounting: an outer failure may roll back an inner charge where already required, but the logical state is still terminal when the operation itself is rejected;
- add regressions for inner input anti-amplification/input budget rejection, inner response rejection, process arithmetic overflow/unmeasurable cases, global window rejection, queue rejection and response-deadline rejection;
- prove none can succeed later on the same logical state after window rollover.

A rejected state may remain bounded until ordinary cleanup if immediate removal would complicate ownership, but it must not be operationally reusable or emit success evidence.

### RSEC-001C — ADR checkpoint remains isolated

The source/carrier persistence conflict remains unchanged and must not block A1/B1v/D1/E:

- D019 says counters are not reset by retry, reconnect, carrier change, identity change or error;
- current source rows are removed when their final live state is released;
- retaining terminal source counters forever would create an unbounded source table;
- D019 does not currently define a terminal-source retention TTL, history count, LRU ceiling or equivalent bounded policy.

Do not invent a numeric retention policy. First do the noncontroversial typed carrier/source projection work. If no existing reviewed text resolves bounded terminal retention, write a compact ADR amendment request with policy options/trade-offs and stop **only C** for maintainer/reviewer decision.

## Evidence boundaries

- `IMPLEMENTATION_COMPLETE=true` remains a bounded research-baseline flag only.
- `CANONICAL_CORPUS_V1_FROZEN=true` remains corpus-specific only.
- `RELEASE_CANDIDATE=false`, `PRODUCTION_READY=false`, `FREEZE=false`, `RELEASED=false` remain required.
- `ea5b257` is implementation evidence for coherent queue/state expiry ownership; it is not a security review by itself.
- `b3490a2` is implementation evidence for partial terminal-rejection semantics; it does not yet prove all D019 rejection classes are terminal.
- Exact `b3490a2` stable CI is red on formatting, while nightly decode fuzz smoke is green. Do not describe the exact tree as fully green.
- A post-send deadline check is not bounded response I/O.
- Process admission does not claim to bound kernel SYN backlog, provider NAT state or other resources outside the process.
- Existing inner `PreauthBudget` remains useful and must not be weakened to simplify outer accounting.
- No VPS/load run substitutes for deterministic D019 counter, ownership and evidence-barrier correctness.
- Protected identity/secrets/private endpoint material remain unread/untracked/uncommitted.

## Rolling Work Queue

This is a rolling multi-hour queue. Complete a coherent slice -> targeted/full required gates -> commit -> push -> immediately consume the next dependency-satisfied slice. Do not stop for a reviewer interval. Only a new HIGH/BLOCKER that invalidates downstream work, a genuine ADR/core-architecture conflict, authorization boundary, production impact, missing credentials/third-party authority, repository breakage, runtime/tool-budget termination or true queue exhaustion is a stop condition.

### A1 — Restore format gate and bind response permits to actual I/O

**Status:** `READY_LOCAL`; highest priority.

Include the rustfmt repair from CI-001, then implement RSEC-001A1.

Required behavior:

- all ordinary TCP/UDP probe, periodic TCP, multistream TCP, failover UDP selection/Noise and failover TCP negotiation/Noise pre-auth response paths use bounded I/O under the exact 100 ms permit;
- TCP framing is total-deadline bounded across partial writes;
- UDP send is permit-bounded;
- failed/late send produces no success evidence and response accounting stays charged;
- no new wire/protocol value is introduced.

Verification: deterministic fake/injected sink boundary tests + affected CLI integration tests + full `scripts/check.sh` + `git diff --check`. Push and inspect exact-head CI.

**Continue immediately to B1v:** yes.

### B1v — Finish queue-expiry ownership verification

**Status:** `PREAUTHORIZED_AFTER_A1`.

Do not redesign the ownership model unless a deterministic test falsifies `ea5b257`.

Add/confirm the missing integration matrix: 1 s idle expiry before 5 s lifetime, lifetime expiry, promotion, cancellation/replacement, source/global queue max/max+1, ordinary server-loop continuation, and exactly-once state/queue/memory cleanup.

If all pass, mark B implementation/verification satisfied for the later exact-tree review.

**Continue immediately to D1:** yes.

### D1 — Make every rejection class terminal across inner + outer accounting

**Status:** `PREAUTHORIZED_AFTER_B1v`.

Implement RSEC-001D1 without changing numeric D019 values or wire/session semantics.

Required regressions include:

- inner per-state input rejection -> same logical ticket cannot continue;
- inner response/anti-amplification rejection -> same logical ticket cannot later send;
- global input/work rejection -> no revival after one-second rollover;
- source/global response rejection -> no later response;
- queue saturation rejection -> no later enqueue on the rejected state;
- response deadline failure -> no later success;
- checked-add/deadline overflow -> terminal rejection;
- cleanup remains one-shot and bounded.

Run targeted tests + full gate, push, and continue.

**Continue immediately to E:** yes.

### E — Audit and machine-check charge ordering across every real responder

**Status:** `PREAUTHORIZED_AFTER_D1`.

For every externally reachable pre-auth responder, machine-check or maintain a static inventory of:

1. typed carrier/source projection + state admission;
2. input bytes/packet charge before parse;
3. parser/work reservation before work;
4. state memory reservation before owned allocation;
5. queue reservation before pending ownership;
6. response charge + actual bounded response I/O before send;
7. terminal rejection/evidence barrier;
8. exactly-once cleanup.

Current conservative 64/4096 work reservations may remain only if they dominate bounded parser work; they are accounting units, not measured CPU cycles. Fix concrete uncovered seams only.

Add a machine-checkable/static guard so a new external responder cannot silently bypass admission.

**Continue immediately to C:** yes.

### C — Resolve carrier/source projection and bounded persistence semantics

**Status:** `ADR_CHECKPOINT_AFTER_E`.

First implement the noncontroversial typed source projection: explicit bounded carrier discriminator (`TCP`, `UDP`, one bounded unknown bucket where applicable), deterministic non-collision tests, no raw source logging.

Then evaluate terminal source persistence. Do not retain all sources forever and do not invent TTL/LRU/history limits. If reviewed text still provides no bounded interpretation of the D019 no-reset rule, write a compact ADR amendment request stating the exact conflict and options, and stop **only C** for a policy decision.

**Continue immediately to F only after C is resolved:** yes.

### F — Complete full D019 adversarial/evidence-barrier matrix

**Status:** `PREAUTHORIZED_AFTER_C`.

Cover source/global concurrency, source lifetime input/packet/work under resolved C semantics, global one-second windows, per-packet work, state/global memory, source/global queue, source/global response + inner 3x anti-amplification, idle/lifetime/response deadlines with injectable time, overflow, terminal non-revival, retry/reconnect/carrier transition, cancellation/timeout/double cleanup, no Session/Path/Delivery/readiness/authz-equivalent evidence on rejection, and secret-safe diagnostics.

Do not substitute VPS/load tests for deterministic accounting semantics.

Full local gate, commit, push; exact repair-head CI must be green before security closure.

**Continue immediately to G after exact-head CI green:** yes.

### G — Fresh exact-tree D019/security evidence review

**Status:** `PREAUTHORIZED_AFTER_F`.

Re-read the exact implementation/tests, then correct `docs/reviews/resource-abuse-evidence-2026-09-04.md`, `docs/release-security-review-packet.md`, `docs/status.md` and closure/navigation records to name the actual reviewed implementation head.

RSEC-001 closes as an implementation finding only when A1/B1v/D1/E/C/F are actually satisfied. Independent external/two-person security review remains a separate release gate. Do not promote RC/production/freeze/release automatically.

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

- exact implementation HEAD passes the full stable repository gate and required fuzz gate;
- 100 ms response permit bounds actual response I/O, not only post-send bookkeeping;
- application pending ownership cannot outlive or diverge from its process pre-auth queue/state reservation;
- queue reservations are charged before ownership and released exactly once across promotion/rejection/idle/lifetime/cancel/shutdown;
- every rejection class, including inner-budget and arithmetic/unmeasurable failures, makes the logical pre-auth state non-revivable;
- charge ordering is auditable across every externally reachable responder;
- carrier/source projection and persistence are reconciled with D019 under a bounded reviewed policy;
- full deterministic boundary/overflow/timeout/evidence-barrier matrix passes;
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

No immediate administrator action is required for A1/B1v/D1/E.

A maintainer/reviewer decision is required only if C reaches the described source-retention ADR conflict and no existing reviewed text provides a bounded policy. At that point present the exact policy options and trade-offs; do not invent a numeric retention rule autonomously.
