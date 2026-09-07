# Nekomusume ChatGPT Handoff

Checked at: 2026-09-07 23:57 Asia/Shanghai
Repository main HEAD reviewed: `517083ba148c61601ca7fd5b3238e4b163b23b26`
Previous checked implementation HEAD: `b041a64e0ecce69d5549847864936f36cbda4b75`
Current execution branch: `work/e1a-staged-accounting-20260907` at exact `655df00b0b6439339ffccdf9a0962f5bb1c66b82`
Historical partial-E2 branch retained: `work/continue-20260904` at `d271a99a2ab26abbcb146c411ba0fde697395abe`

## What changed

The implementation branch consumed the final narrow E2 guard item after the previous review:

- `75df9b3` — normally merges/reconciles current reviewer `main` into the implementation lineage without force-push or history loss;
- `655df00` — adds the exact duplicate-classification anchor `if datagram == pending_state.hello.as_slice()` to the `failover_udp_pending` ordered responder inventory immediately after the single conservative input charge.

The branch is now **ahead of current `main` and not behind it**; ordinary coordination is healthy. Exact `655df00b0b6439339ffccdf9a0962f5bb1c66b82` GitHub Actions run `34137385366` concluded `success`. Exact `main` reviewer HEAD `517083b` also has green Rust CI run `34136493377`.

`655df00` is manifest/checker evidence only; it adds no new runtime, WAN, security-approval, reliability-rate, reachability or performance evidence. It does, however, close the one remaining machine-guard fidelity gap identified in the previous handoff: the inventory now mechanically orders the existing single pending-UDP charge before duplicate classification as well as before duplicate response / Noise processing.

No new correctness/security blocker was introduced. The pre-auth responder accounting package should now be treated as locally closed unless a concrete regression appears. Do **not** reopen it into another checker/audit project.

## Review verdict

**ACCEPT_E2_GUARD_CLOSURE — E1A/E2/C1 local responder-accounting work is closed at the current implementation lineage with exact-head green CI. Move outward now. Compatibility/freeze, package/provenance and release-opportunity recomputation are the immediate local work; then convert one historical `BLOCKED_IMPLEMENTATION` capability into real runtime + bounded self-owned VPS evidence when its dependencies are truthfully ready.**

C2 terminal-source retention remains an intentionally isolated release/security policy checkpoint. It does not block the independent queue below. No administrator action is required to continue.

## Reviewer findings

### RSEC-001E2-GUARD — CLOSED at `655df00`

Accepted bounded guard semantics:

- `failover_udp_pending` inventory names the single conservative `charge_input(..., n, 4096)`;
- the ordered guard now explicitly requires that charge before `if datagram == pending_state.hello.as_slice()`;
- the same ordered surface continues through duplicate response anchors and `receive_first`;
- the existing checker already enforces ordered-anchor presence, so no new checker framework/schema/version was required;
- exact-head CI is green.

This closes the previously identified low/medium drift-guard gap. It does not upgrade deterministic implementation evidence into security approval or WAN evidence.

### RSEC-001E2-RUNTIME — CLOSED at `b041a64`

Retain the accepted runtime direction: one matched existing-pending UDP datagram is charged exactly once before duplicate-vs-Noise classification, with no second duplicate/non-duplicate charge. Queue ownership and cleanup remain as previously reviewed.

### RSEC-001E1A — CLOSED at `164731d` / current implementation lineage

Retain the accepted staged TCP one-record semantics across ordinary, periodic, multistream and failover TCP responders. Do not reopen staged TCP work absent a concrete regression.

### RSEC-001C1 — CLOSED at `f7e2cf1`

Carrier-aware source projection remains accepted and bounded; TCP/UDP source domains do not alias.

### RSEC-001C2 — POLICY CHECKPOINT / RELEASE BLOCKER at `f066af5`

`docs/adr/m1-g0-preauth-source-retention-amendment-request.md` remains truthful and unresolved. Do not invent TTL/LRU/history/epoch/eviction numbers. Literal terminal-source no-reset compliance remains unclaimed.

This blocks only the corresponding D019/release-security conclusion. It does **not** block compatibility review, package/provenance review, release-opportunity recomputation, independent runtime integration or a bounded self-owned VPS experiment whose own question is dependency-ready.

## Evidence boundaries

- `IMPLEMENTATION_COMPLETE=true` remains a bounded research-baseline flag only.
- `CANONICAL_CORPUS_V1_FROZEN=true` remains corpus-specific only.
- `RELEASE_CANDIDATE=false`, `PRODUCTION_READY=false`, `FREEZE=false`, `RELEASED=false` remain required.
- A1 response-I/O deadline, B1 queue ownership/expiry, D1 terminal rejection, E1A staged TCP accounting, E2 pending-UDP runtime/guard ordering and C1 carrier projection are accepted bounded implementation findings.
- C2 is unresolved policy text, not approved D019 compliance.
- Exact `655df00` has green CI and closes a manifest/checker guard only; it adds no WAN/VPS evidence.
- Historical positive and negative WAN/HY2/failover/periodic evidence remains immutable at its exact commit boundaries.
- Standing VPS authorization remains valid; C2 is not a generic WAN authorization blocker.
- Protected identity material, SSH private keys, credentials, private endpoint material and raw private diagnostics remain unread/untracked/uncommitted.

## Rolling Work Queue

This is a multi-hour pre-authorized queue. The coding agent owns ordinary local design choices. Finish one coherent package -> targeted/full gates -> commit -> push -> immediately continue to the next dependency-satisfied package. One commit, nominal hour, reviewer interval, CI pending state or proposal note is not a stop condition.

### B — Compatibility / freeze-boundary closure package

**Status:** `READY_LOCAL`; immediate front of queue; independent of C2.

**Goal / why now:** leave the pre-auth audit loop and verify that current compatibility behavior still matches the provisional spec and corpus-specific freeze boundary.

Audit:

- current/current negotiation acceptance;
- unsupported/future rejection;
- exact negotiation transcript binding into Noise;
- resume/version binding and replay boundary;
- corpus-v1 content-addressed freeze vs global protocol non-freeze;
- docs that might accidentally imply corpus freeze == protocol/release freeze.

**Protected invariants:** do not change frozen corpus bytes without a concrete correctness defect; do not broaden release/freeze claims; Session remains carrier-agnostic.

**Implementation:** add a regression/fix only for a concrete mismatch. If no mismatch exists, do not manufacture a code commit merely to prove the audit happened; record only the smallest repository evidence/status update if repository convention actually requires one.

**Gate:** relevant tests; full `scripts/check.sh` for code changes; `git diff --check`.

**Continue immediately to C:** yes.

### C — Package/operator + evidence-provenance integrity closure

**Status:** `READY_LOCAL_AFTER_B`; independent of C2.

Verify exact-current evidence for:

- x86_64 package/build identity;
- dedicated experimental-path install/readiness/smoke/upgrade/rollback;
- shutdown/listener/temp cleanup;
- retained external state without reading protected identity material;
- canonical Git-blob/checksum manifests;
- exact-head CI references;
- stale release-packet links/hashes.

Do not rerun already-sufficient VPS/package work merely for freshness. If the bounded package/operator question is already answered, classify it as sufficient and continue without traffic theater.

**Continue immediately to D:** yes.

### D — Recompute release opportunities from exact current code/evidence

**Status:** `READY_LOCAL_AFTER_C`.

Re-evaluate every remaining release/evidence row rather than preserving 2026-09-03/04 labels by inertia:

- bounded question already answered -> `ALREADY_SUFFICIENT_FOR_BOUNDED_QUESTION`;
- specific executable missing assertion with satisfied dependencies -> `OPEN_READY` with exact `evidence_needed`, `next_action`, `requires`, `execution_scope`;
- missing implementation/environment/governance/review dependency -> exact blocker;
- C2 remains only its specific source-retention release/security blocker;
- standing-authorized self-owned TCP/UDP work must not be labelled `need WAN authorization`.

Explicitly recompute whether any `READY_LIVE` row now exists. Do not preserve `READY_LIVE: none` because an older document said so.

**Continue immediately to E:** yes.

### E — Select and implement one high-value previously blocked runtime capability

**Status:** `PREAUTHORIZED_AFTER_D`.

Compare 1–3 exact-current candidates among:

- NAT/source-endpoint change;
- migration-back;
- live key update;
- live PMTUD.

Choose the candidate with the smallest new state/API, strongest already-implemented primitives, clearest deterministic test boundary and highest real-VPS evidence value. The coding agent owns this ordinary design choice under `AGENTS.md` and should not wait for reviewer approval unless the selected shape changes core Session/Carrier/ACK/crypto/wire semantics or invents new security policy.

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

Use `docs/standing-vps-lab-authorization.md` and local secret endpoint configuration. Execute one self-owned client<->VPS row that directly answers the newly runnable capability or recomputed highest-value `READY_LIVE` question. Use the smallest profile that answers the question; standing ceilings are limits, not targets.

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

Do not let C2 or the independent-review gate turn the repository back into idle review-only mode.

**Continue immediately to I:** yes when safe.

### I — Independent D019/security debt while C2 waits

**Status:** `READY_LOCAL_FALLBACK`; must not displace a READY runtime/VPS path.

Cover only genuinely missing D019 boundaries independent of terminal-source retention: concurrency ceilings, global one-second windows, memory/queue/response/anti-amplification, idle/lifetime/100 ms deadline, cancellation/double cleanup and no-success-evidence barriers. Reuse existing tests; add only distinct missing boundaries.

Do not claim full D019/RSEC-001 closure until C2 has an approved policy resolution. Do not invent retention numbers.

## 24–48 hour output check

The repository is now producing real engineering progress rather than only reviewer churn: staged TCP accounting and all four responder migrations landed; terminal ownership/cross-layer behavior were repaired; carrier-aware source projection landed; the terminal-source policy conflict was isolated; existing-pending UDP runtime charge ordering was repaired; and the final machine guard now reflects that runtime ordering.

The next coordination goal is **not more pre-auth checker work**. B/C/D should rapidly complete local closure and select the next runtime seam. E/F should convert one historical `BLOCKED_IMPLEMENTATION` capability into actual runtime + bounded real-VPS evidence if the exact-current dependencies support it.

## Completion gates

The current local pre-auth responder package is complete at this review boundary:

- accepted E1A staged semantics remain green;
- all seven inventoried responder surfaces retain accounting-before-protected-work order;
- existing-pending UDP charges exactly once before duplicate comparison and Noise parsing;
- the inventory mechanically orders the charge before that duplicate comparison;
- queue reservation is before pending ownership store;
- rejection/expiry/auth-success cleanup remains exactly-once and cannot emit forbidden success evidence;
- exact-head repository CI is green.

Do not reopen this package absent a new concrete regression.

Full D019/RSEC-001 release-security closure additionally requires an approved terminal-source retention/no-reset policy, remaining distinct adversarial coverage, exact-tree security/evidence review and repository-required independent release review. The broader project must not wait for that policy decision before independent runtime/VPS research proceeds.

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
