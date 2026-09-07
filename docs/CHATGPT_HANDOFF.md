# Nekomusume ChatGPT Handoff

Checked at: 2026-09-07 21:19 Asia/Shanghai
Repository main HEAD reviewed: `cf8aa7c49031f5eba013afe9cc815bd5ec656eeb`
Previous checked implementation HEAD: `f066af59a04a2e0fb83b10cfe6bdb2b530be10e4`
Current execution branch: `work/e1a-staged-accounting-20260907` at exact `f066af59a04a2e0fb83b10cfe6bdb2b530be10e4`
Historical partial-E2 branch retained: `work/continue-20260904` at `d271a99a2ab26abbcb146c411ba0fde697395abe`

## What changed

No new coding checkpoint has landed after `f066af5`. The implementation branch remains at the exact E1A/C1/C2 checkpoint reviewed previously, while `main` advanced only by the reviewer handoff `cf8aa7c`.

Current refs are therefore a normal coordination divergence:

- `main` exact `cf8aa7c` contains the current reviewer handoff and is green under Rust CI run `34115252988`;
- `work/e1a-staged-accounting-20260907` exact `f066af5` contains the ten implementation/merge/policy commits ahead of merge base `5b8be61d` and is green under Rust CI run `34112531048` (`stable checks` + nightly decode fuzz smoke);
- the work branch is one reviewer commit behind current `main`. Before the next implementation checkpoint, integrate current `origin/main` normally; do not force-push or discard implementation history.

E1A and C1 remain accepted closed bounded implementation findings. C2 remains an intentionally isolated release/security policy checkpoint. The queue front has instead remained **E2 pending-UDP semantic closure** through multiple execution opportunities without a coding checkpoint, despite green CI, a clean technical dependency chain, no missing authorization, and no maintainer-level design requirement.

This is now a `STALLED_IMPLEMENTATION` coordination signal. The previous E2 handoff was semantically correct but still left too much room to treat “duplicate or Noise?” accounting as an API-design question. It is not. The smallest existing-architecture repair is concrete and pre-authorized below.

## Review verdict

**CONTINUE_WITH_STALL_BREAK — E1A CLOSED; C1 CLOSED; C2 stays isolated. Execute the concrete E2 single-charge pending-UDP repair now, restore the existing pending-owner guard, then leave the pre-auth audit loop and proceed through compatibility/package/reclassification into one real runtime seam and one truthful VPS row.**

Do not reopen staged TCP work. Do not invent terminal-source retention numbers. Do not wait for another reviewer interval to choose an E2 API shape: the current `charge_input` primitive already suffices for the pending UDP datagram because UDP preserves one bounded datagram boundary.

No administrator action is required. The C2 source-retention question remains deferred to release/security policy review and must not freeze independent engineering.

## Reviewer findings

### RSEC-001E1A — CLOSED at `164731d` / exact `f066af5`

Retain the accepted staged TCP contract:

- one TCP frame owns one input record/packet while bytes/work are staged;
- ordinary, periodic, multistream and failover TCP all use the shared staged helper before attacker-controlled length/body parsing;
- header accounting precedes length interpretation and body reservation precedes allocation/read;
- non-complete staged reads terminalize without refunding attacker-caused reservation;
- staged extension/complete/abandon failure is one-shot/non-revivable;
- direct staged-reader and cross-layer all-or-terminal regressions exist;
- exact-head stable checks and decode fuzz are green.

This is deterministic local security-accounting evidence, not a security approval or WAN result.

### RSEC-001E2 — MEDIUM/HIGH, `STALLED_IMPLEMENTATION` — pending UDP classification is still pre-charge

The active failover pending-UDP branch currently does:

```text
recv_from -> source/peer match -> compare datagram with pending hello
                              -> duplicate branch charge_input(..., 64)
                              -> otherwise charge_input(..., 4096) -> Noise parse
```

The comparison `datagram == pending_state.hello.as_slice()` is bounded but attacker-controlled work and is performed before the datagram's D019 input/work charge. The current runtime then charges the same received datagram differently depending on the result of work that should itself have been pre-charged.

There is no need for a new staged-UDP API. UDP already gives one bounded raw datagram. Use the conservative existing work reservation before classification:

```text
recv_from
-> confirm the datagram belongs to the already-admitted pending source/peer
-> charge_input(&mut pending_state.admission, n, 4096) EXACTLY ONCE
-> only then compare datagram == pending_state.hello
   -> duplicate: charge/send the already-bounded selection response; no second input charge
   -> non-duplicate: call receive_first(...); no second input charge
```

Why `4096` is the preferred shape:

- it is the existing D019 per-packet work ceiling and the already-used conservative Noise-path reservation, not a new policy value;
- before duplicate/Noise discrimination the runtime cannot truthfully know which protected path will be taken, so reserving the bounded worst case is fail-closed;
- D019 explicitly defines work units as conservative accounting units, not measured CPU cycles;
- one received UDP datagram remains exactly one input packet; no staged double packet count is introduced;
- duplicate retransmission may consume more conservative work budget than before, which is acceptable for unauthenticated fail-closed admission and does not create a protocol-success bypass.

Do not refund or reclassify that charge after the comparison. Do not introduce `begin_udp_record`, a second queue subsystem, or new numeric budgets for this repair.

Required E2 runtime/evidence behavior:

1. The single pending-datagram `charge_input(..., n, 4096)` appears before the duplicate comparison and before `receive_first`.
2. Remove the old duplicate-only `charge_input(..., n, 64)` and the later non-duplicate `charge_input(..., n, 4096)` so one datagram cannot be counted twice.
3. A pre-charge rejection must produce neither selection retry nor Noise/auth success evidence.
4. Duplicate selection retransmission remains bounded by existing response/anti-amplification accounting.
5. New pending ownership still charges negotiation input before parse, sends the selection before queue ownership, reserves queue ownership before storing `PendingUdpNegotiation`, and releases on rejection.
6. Expiry invalidates the application queue owner exactly once; authentication takes ownership, dequeues/releases exactly once, then emits authenticated success evidence.

Existing process coverage already exercises dropped-selection / duplicate-hello retry and proves it does not restart negotiation or authentication. Preserve that behavior. Add only a narrowly scoped distinct regression if needed to prove the new pre-charge/no-double-charge boundary; do not manufacture a new test harness solely for accounting aesthetics.

### RSEC-001E2-GUARD — MEDIUM — restore pending-owner semantics in the existing inventory checker

The active inventory currently has `pending_owner=true` but no longer requires the three semantic fields that historical exact `d271a99` already introduced:

- `queue_reserve_anchor`;
- `pending_store_anchor`;
- `pending_cancel_anchor`.

Restore those fields using the **current carrier-aware source/API text**, not a blind cherry-pick. For pending owners, the existing checker should again fail if any of those fields is absent. Additionally, enforce the one ordering property that matters mechanically: the declared queue-reservation anchor must occur before the declared pending-store anchor. Cancellation/expiry anchor existence plus runtime tests is sufficient; do not build another checker framework or attempt static proof of every branch.

For the current runtime, the intended semantic anchors are equivalent to:

```text
queue_reserve_anchor: preauth.enqueue(&mut admission)
pending_store_anchor: pending = Some(PendingUdpNegotiation
pending_cancel_anchor (expired owner): expired.queue.invalidate_after_process_expiry()
```

The new-pending rejection/cancellation entry should retain a current exact release anchor proving no owner survives failed enqueue/setup. If a broad text anchor is ambiguous, choose the smallest unique current snippet rather than weakening the checker.

Update the pending-UDP `ordered` anchors so the single conservative input charge precedes both duplicate classification and Noise parsing. Keep the explicit seven-responder surface set and current carrier-aware admission count.

### RSEC-001C1 — CLOSED at `f7e2cf1`

Carrier-aware source projection is accepted: TCP and UDP source domains are explicit and bounded and family/address/port remain represented without text logging. Do not reopen this absent a concrete collision regression.

### RSEC-001C2 — POLICY CHECKPOINT / RELEASE BLOCKER at `f066af5`

The amendment request correctly records the unresolved tension between literal no-reset semantics and bounded terminal-source memory. Current development disposition remains:

- preserve bounded cleanup behavior;
- do not add TTL/LRU/history/epoch/eviction numbers;
- explicitly state that literal terminal-source no-reset compliance is unresolved;
- keep this as a specific release/security blocker only.

This policy wait does not block E2, compatibility/package review, runtime integration research, release-opportunity reclassification, or a later bounded self-owned VPS experiment whose own release question is dependency-ready.

## Evidence boundaries

- `IMPLEMENTATION_COMPLETE=true` remains a bounded research-baseline flag only.
- `CANONICAL_CORPUS_V1_FROZEN=true` remains corpus-specific only.
- `RELEASE_CANDIDATE=false`, `PRODUCTION_READY=false`, `FREEZE=false`, `RELEASED=false` remain required.
- A1 response-I/O deadline, B1 queue ownership/expiry, D1 terminal rejection, E1A staged TCP accounting and C1 carrier projection remain accepted closed bounded implementation findings.
- `f066af5` records an unresolved policy conflict; it is not an approved ADR amendment or full D019 compliance.
- Exact `f066af5` has green stable checks and green nightly decode fuzz smoke; exact reviewer `cf8aa7c` is also green.
- No new WAN, performance, reliability-rate, public reachability, security approval or production evidence has landed since the previous review.
- Historical WAN/HY2/failover/periodic positive and negative evidence remains immutable at its exact commit boundary.
- Standing VPS authorization remains valid. C2 is not a generic WAN authorization blocker.
- Protected identity material, SSH private keys, credentials, private endpoint material and raw private diagnostics remain unread/untracked/uncommitted.

## Rolling Work Queue

This is a multi-hour pre-authorized queue. The coding agent owns ordinary local design choices. Finish one coherent package -> targeted/full gates -> commit -> push -> immediately continue to the next dependency-satisfied package. One commit, nominal hour, reviewer interval, CI pending state or proposal note is not a stop condition.

### A — E2 single-charge pending-UDP closure + semantic inventory

**Status:** `STALLED_IMPLEMENTATION / READY_LOCAL`; execute now after normally integrating current reviewer `main`.

**Goal / why now:** close the last concrete pre-auth responder ordering seam with one existing conservative charge, then stop expanding pre-auth audit infrastructure.

**Files:** `crates/neko-cli/src/main.rs`; `docs/preauth-responder-inventory.v1.json`; `scripts/check-preauth-responder-inventory.py`; existing `crates/neko-cli/tests/probe.rs` only if a distinct regression is needed.

**Implementation:** use exactly one `charge_input(..., n, 4096)` before duplicate/Noise discrimination in the existing-pending UDP branch; remove branch-specific duplicate/non-duplicate input charges; restore required pending-owner reserve/store/cancel fields and reserve-before-store validation in the existing checker.

**Protected invariants:** one datagram == one input packet; conservative work is charged before payload comparison/Noise work; anti-amplification remains based on charged input; queue reserve before store; exactly-once expiry/auth cleanup; no success evidence after rejection.

**Validation:** existing duplicate-selection retry process test; any one new distinct no-success-on-precharge-reject/no-double-charge test that can be expressed without invasive instrumentation; existing inventory checker; `scripts/check.sh`; `git diff --check`. Fuzz is not required merely for moving an existing bounded UDP accounting call unless parser/wire semantics also change.

**Commit/push:** prefer one coherent runtime + inventory/checker commit/package. Push exact head.

**Continue immediately to B:** yes. Do not wait for reviewer if green/local work is independent.

### B — Compatibility / freeze-boundary audit with defect-only changes

**Status:** `READY_LOCAL_AFTER_A`; independent of C2.

Verify corpus-v1 content-addressed freeze vs global protocol non-freeze, current/current negotiation, unsupported/future rejection, exact negotiation transcript binding into Noise, resume/version binding and replay boundaries against current code/specs.

Add a regression only for a concrete mismatch. Do not reopen frozen corpus bytes without correctness evidence and do not create style-only docs churn.

**Gate:** relevant tests + full gate for code changes. Commit only a real correction.

**Continue immediately to C:** yes.

### C — Package/operator + evidence-provenance integrity

**Status:** `READY_LOCAL_AFTER_B`; independent of C2.

Verify the existing x86_64 package/build identity, dedicated-path install/readiness/smoke/upgrade/rollback contract, shutdown/listener/temp cleanup, canonical Git-blob/checksum manifests and exact-head CI references. Do not read protected identity material. Do not rerun already-sufficient VPS/package work merely for freshness.

If the bounded question is already sufficiently answered, record/reclassify it without manufacturing a commit and continue.

**Continue immediately to D:** yes.

### D — Recompute release opportunities from current implementation

**Status:** `READY_LOCAL_AFTER_C`.

Re-evaluate every remaining release/evidence row instead of carrying stale 2026-09-03/04 labels forward:

- answered bounded question -> `ALREADY_SUFFICIENT_FOR_BOUNDED_QUESTION`;
- specific executable missing assertion -> `OPEN_READY` with exact `evidence_needed`, `next_action`, `requires`, `execution_scope`;
- missing implementation/environment/governance/review dependency -> exact blocker;
- C2 remains only a specific release/security policy blocker;
- standing-authorized self-owned TCP/UDP work must not be labelled `need WAN authorization`.

Do not preserve `READY_LIVE: none` by inertia. Recompute from exact current code and evidence.

**Continue immediately to E:** yes.

### E — Turn one high-value historical `BLOCKED_IMPLEMENTATION` seam into a runnable capability

**Status:** `PREAUTHORIZED_AFTER_D`.

Compare 1–3 current candidates among NAT/source-endpoint change, migration-back, live key update and live PMTUD. Choose the candidate with the smallest new state/API, strongest already-implemented primitive support, clearest deterministic validation and highest real-VPS evidence value. Live key update remains a natural first candidate because synchronized key-update state already exists, but exact code should decide.

Package a visible-output closure:

1. actual runtime integration, not fixture-only wrapping;
2. bounded process/loopback validation;
3. structured diagnostics sufficient for a real-socket result;
4. failure paths that do not fabricate Session/Delivery/PathValidated/ACK evidence;
5. no new wire/crypto/security policy unless already present in the accepted candidate contract.

If one candidate genuinely requires a maintainer-level core Session/Carrier/ACK/crypto/wire decision, choose another dependency-safe candidate instead of stopping the whole project.

**Gate:** targeted tests + `scripts/check.sh` + `git diff --check`; fuzz only if untrusted parser/wire semantics change; commit/push.

**Continue immediately to F when the resulting question is truthfully `READY_LIVE`:** yes.

### F — One bounded changed-capability VPS evidence run

**Status:** `PREAUTHORIZED_AFTER_E_WHEN_READY_LIVE_EXISTS`.

Use standing authorization and local secret endpoint configuration. Execute one self-owned client<->VPS row that directly answers the newly runnable capability or the recomputed highest-value `READY_LIVE` question. Use the smallest profile that answers the question; standing ceilings are limits, not targets.

Record experiment ID, exact git/binary identity, actual parameters/timestamps, client/server structured result, relevant CPU/RSS/FD/socket observations, bounded capture metadata only if needed, and explicit cleanup verification.

Positive or negative is valid. No unchanged retry. HY2 is not automatic; it requires a genuinely changed diagnostic hypothesis and a declared missing comparison question.

**Continue immediately to G after cleanup:** yes.

### G — Reconcile the new runtime/VPS evidence

**Status:** `PREAUTHORIZED_AFTER_F`.

Update the relevant release matrix/status/evidence artifact in the same semantic boundary. State exactly what the run proves and does not prove. One bounded pass is not a reliability rate, public reachability, production readiness or performance superiority result.

If the run exposes a concrete defect, put that fix first. If it answers the bounded question, classify it sufficient instead of scheduling freshness reruns.

**Continue immediately to H:** yes.

### H — Independent D019/security debt while C2 waits

**Status:** `READY_LOCAL_FALLBACK`; must not displace a READY runtime/VPS path.

Cover only distinct D019/security boundaries independent of terminal-source retention: concurrency ceilings, global one-second windows, memory/queue/response/anti-amplification, idle/lifetime/100 ms deadline, cancellation/double cleanup and no-success-evidence barriers. Reuse existing tests; add only genuinely missing boundaries.

Do not claim full D019/RSEC-001 closure until C2 receives an approved policy resolution. Do not invent a retention number.

## 24–48 hour output check

The last 24–48 hours did produce real implementation output: four TCP responders were moved behind staged pre-auth accounting, staged failure ownership became structural, carrier-aware source projection landed, and the source-retention contradiction was isolated honestly. However, the last two reviewer intervals produced no new coding checkpoint while E2 remained locally ready.

This review therefore spends its coordination budget on **one concrete stall-breaking E2 contract**, not another checker layer. After A, the project must move outward through B/C/D into a runnable capability and VPS evidence rather than spending another day polishing pre-auth infrastructure.

## Completion gates

The current local pre-auth responder package is complete when:

- accepted E1A staged semantics remain green;
- all seven inventoried responder surfaces retain accounting-before-protected-work order;
- existing-pending UDP charges exactly once before duplicate/Noise discrimination;
- queue reservation is mechanically before pending ownership store;
- rejection/expiry/auth-success cleanup remains exactly-once and cannot emit forbidden success evidence;
- exact-head repository gate is green.

Full D019/RSEC-001 release-security closure additionally requires an approved terminal-source retention/no-reset policy, the remaining independent adversarial matrix, exact-tree security/evidence review, and repository-required independent release review.

The broader project must not wait for that policy decision before independent runtime/VPS research proceeds.

## Do not expand into

- a new terminal-source TTL/LRU/history/epoch/eviction number without explicit reviewed policy;
- a new staged-UDP accounting subsystem for the current single-datagram E2 repair;
- a new checker framework instead of extending the existing inventory checker;
- public or production listener deployment;
- protocol/wire/Noise/Session/Carrier redesign unrelated to an observed blocker;
- repeated unchanged HY2/repeated-failover/periodic historical failures;
- VPS load testing as a substitute for deterministic correctness;
- speculative FEC/0-RTT/striping/multipath/exotic-carrier work without an observed-problem gate;
- reading, hashing, copying, modifying, uploading or committing protected identity/SSH-secret material;
- release/RC/freeze/production promotion.

## Questions requiring maintainer decision

**No maintainer decision is required to continue the current queue.**

D019 terminal-source retention remains a future release/security policy decision recorded in `docs/adr/m1-g0-preauth-source-retention-amendment-request.md`. Until that review occurs, preserve bounded cleanup, state the limitation honestly, and do not claim literal terminal-source no-reset compliance.
