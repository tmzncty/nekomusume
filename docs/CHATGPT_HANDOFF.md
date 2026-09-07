# Nekomusume ChatGPT Handoff

Checked at: 2026-09-07 23:00 Asia/Shanghai
Repository main HEAD reviewed: `5c99c45010fabd68d9270fddc414ac37af25aa41`
Previous checked implementation HEAD: `f066af59a04a2e0fb83b10cfe6bdb2b530be10e4`
Current execution branch: `work/e1a-staged-accounting-20260907` at exact `b041a64e0ecce69d5549847864936f36cbda4b75`
Historical partial-E2 branch retained: `work/continue-20260904` at `d271a99a2ab26abbcb146c411ba0fde697395abe`

## What changed

The coding agent resumed and consumed the stalled E2 front with two coherent checkpoints after `f066af5`:

- `7e337d4` — restores current carrier-aware pending-UDP inventory ownership fields and the existing reserve-before-store checker rule;
- `b041a64` — moves the existing-pending failover UDP datagram charge to one conservative `charge_input(..., n, 4096)` **before** duplicate-vs-Noise classification, and removes the old branch-specific duplicate/non-duplicate charges.

Exact `b041a64e0ecce69d5549847864936f36cbda4b75` GitHub Actions run `34131611274` is green: both `stable checks` (`scripts/check.sh`) and `nightly decode fuzz smoke` completed successfully. This is exact-head repository CI evidence, not a security/release/WAN claim.

The runtime E2 defect identified in the previous handoff is repaired in the intended minimal form: after peer match, one received pending UDP datagram is charged exactly once with the existing conservative work ceiling before `datagram == pending_state.hello.as_slice()` and before `receive_first`. Duplicate selection retry and Noise paths no longer apply a second input charge.

The existing inventory checker also again requires reserve/store/cancel fields for entries marked `pending_owner=true` and mechanically checks queue reservation before pending storage. One small guard-fidelity gap remains: the `failover_udp_pending` ordered sequence does not name the duplicate-classification anchor itself, so the checker can prove the conservative charge precedes response/Noise anchors but does not mechanically prove it remains before the duplicate comparison. This is a narrow drift-guard issue, not a runtime correctness blocker and not grounds for another pre-auth audit project.

Branch coordination remains ordinary: `b041a64` is based on the implementation lineage from `f066af5` and is behind the two latest reviewer-only `main` handoffs. Before the next coding package, normally integrate current `origin/main` without force-push or history loss.

## Review verdict

**ACCEPT_E2_RUNTIME_WITH_ONE_NARROW_GUARD_FIX — the pending-UDP single-charge runtime repair is accepted. Close the remaining inventory anchor gap in-place, then leave the pre-auth audit loop. Continue immediately through compatibility/package/reclassification, select one real runtime capability seam, and use the rented VPS when that seam becomes truthfully READY_LIVE.**

E1A remains CLOSED. C1 remains CLOSED. C2 remains an intentionally isolated release/security policy checkpoint and must not freeze independent engineering. No administrator action is required to continue the queue.

## Reviewer findings

### RSEC-001E2-RUNTIME — CLOSED at exact `b041a64`

Accepted bounded semantics:

- source/peer match occurs before charging an already-admitted pending source, so unrelated peers do not consume that source's budget;
- one matched pending UDP datagram is charged exactly once before duplicate/Noise classification;
- the existing `4096` work reservation is conservative D019 accounting, not a CPU-cycle claim and not a new policy value;
- duplicate selection retry has no second input charge and remains subject to existing response/anti-amplification accounting;
- non-duplicate Noise parsing has no second input charge;
- authentication still takes pending ownership, dequeues/releases, and only then emits authenticated success evidence;
- process expiry still invalidates the application queue owner;
- exact-head stable checks and decode fuzz are green.

This is deterministic/runtime implementation evidence only. It adds no WAN, reliability-rate, public-reachability, performance, security-approval or production evidence.

### RSEC-001E2-GUARD — LOW/MEDIUM — add the duplicate-classification anchor, then close

`7e337d4` restored the useful historical pending-owner reserve/store/cancel schema and `b041a64` restored the checker logic. The remaining `failover_udp_pending` `ordered` list currently has:

```text
charge_input(..., n, 4096)
-> charge_response / send duplicate response
-> receive_first(...)
```

but omits the actual classification anchor:

```text
if datagram == pending_state.hello.as_slice()
```

**Required narrow repair:** add the smallest unique current duplicate-comparison snippet to the `failover_udp_pending` `ordered` sequence immediately after the single conservative charge and before the duplicate response anchors. The existing checker already enforces list order, so no new framework, schema version or test harness is needed.

Do not reopen runtime accounting unless this small guard change exposes a real mismatch. Run the existing inventory checker / `scripts/check.sh` / `git diff --check`, commit and push if a tracked change is needed, then consider E2 closed and continue immediately.

### RSEC-001E1A — CLOSED at `164731d` / exact implementation lineage

Retain the accepted four-responder staged TCP semantics. Do not reopen staged TCP work absent a concrete regression.

### RSEC-001C1 — CLOSED at `f7e2cf1`

Carrier-aware source projection remains accepted. TCP/UDP source domains are explicit and bounded and family/address/port remain represented without text logging.

### RSEC-001C2 — POLICY CHECKPOINT / RELEASE BLOCKER at `f066af5`

The source-retention amendment request remains truthful and unresolved. Preserve bounded cleanup and do not invent TTL/LRU/history/epoch/eviction numbers. Literal terminal-source no-reset compliance remains unclaimed.

This is a **specific release/security policy blocker only**. It does not block compatibility review, package/provenance review, release-opportunity recomputation, new runtime integration, or a bounded self-owned VPS experiment whose own question is dependency-ready.

## Evidence boundaries

- `IMPLEMENTATION_COMPLETE=true` remains a bounded research-baseline flag only.
- `CANONICAL_CORPUS_V1_FROZEN=true` remains corpus-specific only.
- `RELEASE_CANDIDATE=false`, `PRODUCTION_READY=false`, `FREEZE=false`, `RELEASED=false` remain required.
- A1 response-I/O deadline, B1 queue ownership/expiry, D1 terminal rejection, E1A staged TCP accounting, E2 pending-UDP runtime ordering, and C1 carrier projection are accepted bounded implementation findings.
- C2 is unresolved policy text, not approved D019 compliance.
- Exact `b041a64` is green under stable checks and nightly decode fuzz smoke.
- No new WAN/VPS evidence landed in this review interval.
- Historical positive and negative WAN/HY2/failover/periodic evidence remains immutable at its exact commit boundaries.
- Standing VPS authorization remains valid; C2 is not a generic WAN authorization blocker.
- Protected identity material, SSH private keys, credentials, private endpoint material and raw private diagnostics remain unread/untracked/uncommitted.

## Rolling Work Queue

This is a multi-hour pre-authorized queue. The coding agent owns ordinary local design choices. Finish one coherent package -> targeted/full gates -> commit -> push -> immediately continue to the next dependency-satisfied package. One commit, nominal hour, reviewer interval, CI pending state or proposal note is not a stop condition.

### Q0 — Reconcile current reviewer main into the implementation branch

**Status:** `READY_LOCAL`; coordination only.

Fetch current `origin/main` and normally merge/reconcile it into `work/e1a-staged-accounting-20260907`, preserving all implementation commits. Do not force-push. `docs/CHATGPT_HANDOFF.md` remains reviewer-owned/read-only to the coding agent.

If already reconciled when work resumes, skip Q0.

**Continue immediately to A0:** yes.

### A0 — Close the final E2 inventory ordering guard

**Status:** `READY_LOCAL`; deliberately narrow and should not grow.

**Goal / why now:** make the existing machine guard reflect the runtime property already implemented at `b041a64`: the single conservative charge precedes both duplicate classification and Noise parsing.

**Files:** `docs/preauth-responder-inventory.v1.json` only unless the existing checker reveals a genuine mismatch.

**Behavior:** add `if datagram == pending_state.hello.as_slice()` (or the smallest unique exact current equivalent) to the `failover_udp_pending` `ordered` sequence immediately after `.charge_input(&mut pending_state.admission, n, 4096)` and before duplicate response anchors. Do not add a new checker framework, runtime API, numeric budget or schema version.

**Protected invariants:** one datagram == one input packet; charge before classification; queue reserve before store; existing exactly-once cleanup; no success evidence after rejection.

**Validation:** `python3 scripts/check-preauth-responder-inventory.py`, `./scripts/check.sh`, `git diff --check`. No new fuzz campaign is required for a manifest-only guard adjustment.

**Commit/push:** one small guard commit if needed.

**Continue immediately to B:** yes. Do not wait for reviewer.

### B — Compatibility / freeze-boundary closure package

**Status:** `READY_LOCAL_AFTER_A0`; independent of C2.

**Goal / why now:** stop spending the project on pre-auth audit infrastructure and verify that the already-implemented compatibility boundary still matches the provisional spec and corpus freeze.

Audit current/current negotiation, unsupported/future rejection, exact negotiation transcript binding into Noise, resume/version binding, replay boundary, corpus-v1 content-addressed freeze vs global protocol non-freeze, and docs that might accidentally imply corpus freeze == protocol/release freeze.

**Protected invariants:** do not change frozen corpus bytes without a concrete correctness defect; do not broaden release/freeze claims; Session remains carrier-agnostic.

**Implementation:** add a regression or fix only for a concrete mismatch. If no mismatch exists, record the audit result in the smallest existing evidence/status surface only if repository convention requires it; otherwise do not manufacture a commit.

**Gate:** relevant tests; full `scripts/check.sh` for code changes; `git diff --check`.

**Continue immediately to C:** yes.

### C — Package/operator + evidence-provenance integrity closure

**Status:** `READY_LOCAL_AFTER_B`; independent of C2.

Verify the existing x86_64 package/build identity, dedicated experimental-path install/readiness/smoke/upgrade/rollback contract, shutdown/listener/temp cleanup, canonical Git-blob/checksum manifests, exact-head CI references and stale release-packet hashes/links. Do not read protected identity material.

Do not rerun already-sufficient VPS/package work merely for freshness. If the bounded package/operator question is already answered, classify it as sufficient and continue without manufacturing traffic.

**Continue immediately to D:** yes.

### D — Recompute release opportunities from exact current code/evidence

**Status:** `READY_LOCAL_AFTER_C`.

Re-evaluate every remaining release/evidence row rather than preserving 2026-09-03/04 labels by inertia:

- bounded question already answered -> `ALREADY_SUFFICIENT_FOR_BOUNDED_QUESTION`;
- specific executable missing assertion with satisfied dependencies -> `OPEN_READY` with exact `evidence_needed`, `next_action`, `requires`, `execution_scope`;
- missing implementation/environment/governance/review dependency -> exact blocker;
- C2 remains only its specific source-retention release/security blocker;
- standing-authorized self-owned TCP/UDP work must not be labelled `need WAN authorization`.

Explicitly recompute whether any `READY_LIVE` row now exists. Do not preserve `READY_LIVE: none` simply because an older document said so.

**Continue immediately to E:** yes.

### E — Select and implement one high-value previously blocked runtime capability

**Status:** `PREAUTHORIZED_AFTER_D`.

Compare 1–3 exact-current candidates among:

- NAT/source-endpoint change;
- migration-back;
- live key update;
- live PMTUD.

Choose the one with the smallest new state/API, strongest already-implemented primitives, clearest deterministic test boundary and highest real-VPS evidence value. The agent owns this ordinary engineering choice and should not wait for reviewer approval unless the selected shape changes core Session/Carrier/ACK/crypto/wire semantics or invents new security policy.

Prefer a visible-output closure:

1. actual runtime integration, not fixture-only wrapping;
2. bounded loopback/process validation;
3. structured diagnostics sufficient for a real-socket result;
4. failure paths that do not fabricate Session/Delivery/PathValidated/ACK evidence;
5. no new wire/crypto/security policy unless already approved.

If one candidate genuinely needs maintainer-level architecture choice, select another dependency-safe candidate instead of stopping the project.

**Gate:** targeted tests + `scripts/check.sh` + `git diff --check`; fuzz only if untrusted parser/wire semantics change; commit/push.

**Continue immediately to F when a declared question becomes truthfully READY_LIVE:** yes.

### F — One bounded self-owned VPS evidence run

**Status:** `PREAUTHORIZED_AFTER_E_WHEN_READY_LIVE_EXISTS`.

Use `docs/standing-vps-lab-authorization.md` and local secret endpoint configuration. Execute one self-owned client<->VPS row that directly answers the newly runnable capability or the recomputed highest-value READY_LIVE question. Use the smallest profile that answers the question; standing ceilings are limits, not targets.

Record experiment ID, exact git/binary identity, actual parameters/timestamps, client/server structured result, relevant CPU/RSS/FD/socket observations, bounded capture metadata only if needed, and explicit cleanup verification.

Positive or negative is valid. Do not unchanged-retry historical HY2/repeated-failover/periodic failures. HY2 is not automatic and requires a genuinely changed diagnostic hypothesis plus a declared missing comparison question.

**Continue immediately to G after cleanup:** yes.

### G — Reconcile new runtime/VPS evidence and close the bounded question

**Status:** `PREAUTHORIZED_AFTER_F`.

Update the relevant release matrix/status/evidence artifact at the same semantic boundary. State exactly what the run proves and does not prove. One bounded pass is not a reliability rate, public reachability, production readiness or performance superiority result.

If the run exposes a concrete defect, put that fix first. If it answers the bounded question, classify it sufficient rather than scheduling freshness reruns.

**Continue immediately to H:** yes.

### H — Second runtime/VPS opportunity or independent release work

**Status:** `PREAUTHORIZED_AFTER_G`.

Recompute priorities again. If another distinct VPS-only question is now READY and materially valuable during the rental window, take the next smallest one. Otherwise continue independent release/security/package/provenance work that does not depend on C2.

Do not let C2 or the existence of an independent-review gate turn the repository back into idle review-only mode.

**Continue immediately to I:** yes when safe.

### I — Independent D019/security debt while C2 waits

**Status:** `READY_LOCAL_FALLBACK`; must not displace a READY runtime/VPS path.

Cover only genuinely missing D019 boundaries independent of terminal-source retention: concurrency ceilings, global one-second windows, memory/queue/response/anti-amplification, idle/lifetime/100 ms deadline, cancellation/double cleanup and no-success-evidence barriers. Reuse existing tests; add only distinct missing boundaries.

Do not claim full D019/RSEC-001 closure until C2 has an approved policy resolution. Do not invent retention numbers.

## 24–48 hour output check

The project is now producing real implementation progress again: staged TCP accounting and all four responder migrations landed; terminal ownership and cross-layer behavior were repaired; carrier-aware source projection landed; the terminal-source policy conflict was isolated honestly; and the pending-UDP runtime charge-order defect is now repaired at `b041a64` with green stable/fuzz CI.

The next coordination goal is therefore **not another day of pre-auth checker work**. A0 is the final narrow guard correction. After it, B/C/D must rapidly move the queue outward, and E/F should convert one historical `BLOCKED_IMPLEMENTATION` capability into actual runtime + real-VPS evidence if the exact dependencies support it.

## Completion gates

The current local pre-auth responder package is complete when:

- accepted E1A staged semantics remain green;
- all seven inventoried responder surfaces retain accounting-before-protected-work order;
- existing-pending UDP charges exactly once before the duplicate comparison and Noise parsing;
- the inventory mechanically orders the single charge before that duplicate comparison;
- queue reservation is before pending ownership store;
- rejection/expiry/auth-success cleanup remains exactly-once and cannot emit forbidden success evidence;
- exact-head repository gate is green.

After A0, treat that local responder package as closed unless a new concrete regression appears.

Full D019/RSEC-001 release-security closure additionally requires an approved terminal-source retention/no-reset policy, the remaining independent adversarial matrix, exact-tree security/evidence review and repository-required independent release review. The broader project must not wait for that policy decision before independent runtime/VPS research proceeds.

## Do not expand into

- new terminal-source TTL/LRU/history/epoch/eviction numbers without explicit reviewed policy;
- another staged-UDP accounting subsystem or another pre-auth checker framework;
- public or production listener deployment;
- protocol/wire/Noise/Session/Carrier redesign unrelated to an observed blocker;
- repeated unchanged HY2/repeated-failover/periodic historical failures;
- VPS load testing as a substitute for deterministic correctness;
- speculative FEC/0-RTT/striping/multipath/exotic-carrier work without an observed-problem gate;
- reading, hashing, copying, modifying, uploading or committing protected identity/SSH-secret material;
- release/RC/freeze/production promotion.

## Questions requiring maintainer decision

**No maintainer decision is required to continue the current queue.**

D019 terminal-source retention remains a future release/security policy decision recorded in `docs/adr/m1-g0-preauth-source-retention-amendment-request.md`. Until that review occurs, preserve bounded cleanup, state the limitation honestly, and do not claim literal terminal-source no-reset compliance. Continue all independent engineering and bounded self-owned VPS work whose own dependencies are satisfied.
