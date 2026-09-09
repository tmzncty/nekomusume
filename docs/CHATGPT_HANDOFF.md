# ChatGPT reviewer handoff — bind cached UDP Noise retries before RSEC closure

## Reviewed state

- Previous reviewer-owned handoff: exact `152d3128e2face4370ba95abc41dd7b4772cf171` (`docs(handoff): continue bounded RSEC process observation`).
- Previous reviewed developer implementation/test head: exact `f74b8231ed8a088ceba20d4367993094c3ccbae5` (`test: restore timeout on failed preauth writes`).
- Previous reviewed developer documentation/evidence head: exact `d9401f1125bd2ea9752ae917a2131953659eaaee` (`docs: record bounded preauth response observation`).
- Current default-branch developer implementation/test head reviewed this cycle: exact `21e42af49a9bbe639aefa3dca887e17b03cad07c` (`test: observe bounded preauth recovery`).
- Current default-branch developer documentation/evidence head reviewed this cycle: exact `f9511f33cc8a3f777026e5a7ab78a37142d303c4` (`docs: record bounded preauth process recovery`).
- Developer sequence since the previous reviewer handoff:
  - `21e42af49a9bbe639aefa3dca887e17b03cad07c` adds compact deterministic coverage at the unchanged `ProcessPreauthLimits::default()` candidate boundaries plus one local long-lived failover-server malformed-UDP/recovery observation.
  - `f9511f33cc8a3f777026e5a7ab78a37142d303c4` persists exact-`21e42af` developer-local gate provenance and reconciles the release packet, item-4 factual support, and resource-abuse review to index that observation.
- No new VPS/WAN experiment was performed in this sequence.
- GitHub currently exposes no hosted combined-status records for exact `21e42af`. This is not a blocker and must not trigger Actions polling.

## Review verdict

### ACCEPT WITH BOUNDS — default candidate boundary tests and bounded process recovery observation

Exact `21e42af` now directly exercises the unchanged D019 candidate values for representative source/global state, global queue, and source response accounting boundaries. The first excess operation rejects, and an explicitly abandoned charged response remains counted. This is deterministic model evidence at the actual candidate numbers; it does not establish that those numbers are suitable for production or public exposure.

The same exact tree also runs one bounded local process observation: one long-lived loopback failover server receives eight five-byte malformed UDP negotiation attempts, remains able to complete one later authenticated 16-byte exchange, and verifies listener rebind after exit. Optional Linux `/proc` FD/RSS checks are bounded regression observations only, not stress/capacity results.

`docs/local-preauth-process-observation-21e42af-20260910.md` records developer-local `PYTHONDONTWRITEBYTECODE=1 bash scripts/check.sh` exit `0`, `git diff --check` exit `0`, distinct UTC start/end times, Linux/x86_64, Rust 1.98.0, and clean initial/final source tree. Reviewer did not execute this CI; hosted CI is absent and not required.

### HIGH / READY_LOCAL — cached UDP Noise-response retries escape the pre-auth accounting owner after authentication

The current failover UDP responder still has a release/accounting hole that invalidates the broad claim that every pre-auth response surface is exact-charged and bounded.

Current shape on `main`:

1. while `pending` owns the UDP pre-auth `AdmissionTicket`, the first Noise message is charged, `ResponderHandshake::receive_first` produces the cached response and secure session, and the first Noise response is charged and either sent through `send_udp_response` or explicitly suppressed for the loss fixture;
2. immediately afterward the code stores `handshake_cache`, sets `secure = Some(...)`, dequeues the pending queue reservation, and **releases the admission ticket/source state**;
3. once `secure.is_some()`, any later datagram from the same peer that byte-matches the cached first Noise message is answered by raw `udp.send_to(response, peer)` and `continue`, with no `ListenerAdmission::charge_input`, no response charge/permit, no source/global response ceiling, and no response-send deadline.

That retry is still a retransmission of a pre-auth/handshake response. D019 explicitly counts retransmissions and says retry/reconnect/error must not reset the relevant budget. A captured/replayed first Noise message, or a client repeatedly retransmitting it, can therefore trigger cached responses after the accounting owner was discarded. The process is time-bounded, but the path bypasses the candidate source/global response contract and contradicts the current `8 surfaces / 7 admission sites` closure wording.

Treat this as a correctness/security/evidence HIGH for the candidate pre-auth boundary. Do not expand unrelated runtime work until it is closed. This does **not** imply a new wire, Session, Carrier, ACK, Noise-pattern, or crypto design defect, and it is not a production exploit claim.

#### Required invariant

Every cached handshake/Noise response that can be emitted because unauthenticated or handshake-stage bytes are received must remain under one bounded source-owned admission lifetime and must be charged exactly before send, including retransmissions. A successful server-side `receive_first` alone must not discard the accounting owner while the executable still accepts/replies to duplicate first-handshake bytes.

#### Preferred minimal implementation shape

Run a short 1-3 option proposal cycle against current `failover_server` and choose the smallest fail-closed shape. Prefer retaining a bounded post-Noise handshake-response owner derived from the existing `AdmissionTicket` rather than inventing a second resource-policy system:

- after `receive_first`, dequeue the pending queue reservation if appropriate, but retain the admission/source accounting needed for cached first-Noise retries;
- while the peer has not yet demonstrated possession of the server response by sending a valid authenticated application/session record, charge each received retry before classification/work and charge each cached response through the existing bounded UDP response helper;
- preserve the existing source/global response packet/byte ceilings, response-send deadline, state idle/lifetime limits and one-pending response ownership;
- once the first valid post-handshake authenticated application/session record proves the peer progressed beyond the handshake response, drop the cached pre-auth retry capability and release the retained accounting owner exactly once;
- on state expiry, error, experiment shutdown, or fail-closed budget exhaustion, settle queue/response ownership and remove the cached retry capability; do not silently fall back to raw uncharged `send_to`;
- audit the opt-in delayed cached-Noise duplicate seam in the same bounded slice. If it emits the same cached handshake response, either route it through truthful accounting while an owner exists or make its post-auth test-only boundary explicit without widening the production responder inventory. Do not touch normal authenticated DeliveryAck sends merely because they also use UDP.

An alternate implementation is acceptable only if it proves the same no-reset/exact-charge invariant with less state and no new policy values. Do not solve this by deleting retransmission recovery unless the existing first-Noise-response-loss positive remains truthfully supported by another bounded mechanism.

#### Minimum focused evidence

- preserve the existing first UDP Noise-response loss/retry positive: a lost/suppressed first server response can still recover through a bounded cached response retry;
- prove cached first-Noise retries consume the same source-owned input/response accounting rather than a fresh reset domain;
- prove the first response packet/byte operation beyond the configured candidate source ceiling fails closed and no extra cached response is sent;
- prove state/owner cleanup on first valid authenticated application progress, expiry, error and normal experiment termination, with no orphan queue/pending-response ownership;
- prove controller/response deadline semantics remain unchanged;
- update the responder inventory only after the exact current call sites support the claim.

No fuzz is required unless the implementation actually changes wire decoder/parser/crypto framing.

### MEDIUM / READY_LOCAL — the new process observation does not yet prove `distinct-source` UDP churn

The exact-`21e42af` process test creates a fresh `UdpSocket::bind("127.0.0.1:0")` inside each loop iteration, sends one malformed datagram, then drops that socket. It never records/asserts the source port and does not keep prior sockets alive. The OS may reuse an ephemeral port after a socket is dropped, so the current documentation claim that **each** malformed datagram came from a distinct source port is stronger than the executable evidence.

This does not invalidate the bounded recovery observation itself. It invalidates only the source-domain classification.

Repair this in the same RSEC package after/with the HIGH:

- preferred: pre-bind all eight local UDP sender sockets first, keep all of them alive for the complete malformed phase, collect their local addresses/ports, assert they are unique, and send exactly one five-byte malformed datagram from each; or
- if uniqueness is intentionally not guaranteed, retract `distinct-source` wording everywhere and describe the workload truthfully as eight fresh-socket attempts with source-port reuse not asserted. Do not relabel it as same-source saturation.

If cheaply available while touching this test, tighten two LOW evidence details rather than creating another framework: assert temporary identity cleanup instead of ignoring `remove_file` failure, and either directly establish that the malformed phase emits no authentication/Delivery/PathValidated/ACK-success evidence or soften that documentation sentence to exactly what the test observes.

### MEDIUM / FACTUAL RECONCILIATION — keep reviewer foundation and developer-tested anchors unambiguous

The release packet and item-4 factual review now truthfully anchor developer-local gates at exact `21e42af`. The resource-abuse review, however, still carries a historical `Current exact-tree reconciliation — 6952fa9 lineage` / `Reviewed tree: 6952fa9...` header while its engineering-evidence paragraph says controls/model/process observation exist through exact `21e42af`.

Do not rewrite that into a false independent-review claim. When the HIGH/source-domain repair reaches one final green developer SHA, reconcile these documents together so they explicitly distinguish:

- historical reviewer/review-foundation tree, if that anchor is intentionally retained;
- current developer-tested implementation/evidence tree;
- this ChatGPT reviewer handoff, which is navigation/review support and not the independent two-person security review required by D019/item 4.

RSEC-001 remains open for promotion suitability/independent review even after this bounded engineering package is closed.

## Local-CI-first rule

For every implementation/test slice below, push the coherent developer SHA first, then validate that exact SHA in a clean temporary worktree/clone:

```text
PYTHONDONTWRITEBYTECODE=1 bash scripts/check.sh
git diff --check
git status --porcelain   # empty
```

Persist minimum provenance only after the final replacement SHA is green: exact SHA, exact commands, distinct UTC start/end timestamps, exits, host/OS/arch, stable Rust version and initial/final clean-tree state. If red, preserve the concrete failure and repair it before closure; do not unchanged-rerun until lucky.

Developer-reported/local persisted CI, reviewer-executed CI and hosted CI remain separate evidence classes. Reviewer did not execute the gate this cycle. Hosted CI is optional cross-evidence and must not become a wait condition.

## Rolling queue — execute continuously in dependency order

The coding agent must continue through every dependency-ready item below without waiting for the next reviewer hour. The HIGH is first and blocks expansion into unrelated runtime work, but it does not require maintainer approval because the invariant and policy numbers already exist.

### A. HIGH / READY_LOCAL — bind cached failover UDP Noise retries to live pre-auth ownership

Implement the minimum current-main repair described above. Use existing admission/response ownership and existing candidate values. No new capacity, TTL, LRU/history, response count, wire field, crypto primitive, Session semantic or carrier architecture is authorized.

Continue immediately to B.

### B. HIGH-CLOSURE / READY_LOCAL — focused cached-response accounting and recovery tests

Add the minimum positives/negatives listed above. In particular, preserve first-response-loss recovery while proving repeated cached responses cannot escape source/global charging or lifetime/deadline ownership.

Perform one bounded source search for other **cached pre-auth/handshake response** raw sends in currently advertised responder paths and fix only concrete equivalents found. Do not turn this into a generic `send_to` checker and do not migrate ordinary post-auth DeliveryAck traffic into pre-auth accounting.

Continue immediately to C.

### C. READY_LOCAL — make the malformed-process source-domain evidence truthful

Repair the eight-attempt UDP process fixture so source-port distinctness is executable/asserted, or retract the distinct-source claim. Keep the workload small: eight five-byte attempts and one later authenticated 16-byte exchange are sufficient unless the HIGH repair itself requires a slightly different deterministic positive.

Tighten identity cleanup/no-success wording only where it is cheap and directly testable. Continue immediately to D.

### D. READY_LOCAL — final exact-tree gate and concise provenance

After A-C land in the final implementation/test SHA, run the exact-tree local gate. Persist one concise provenance note containing exact SHA, commands, UTC interval, host/OS/arch, stable Rust, clean-tree state, focused cached-response tests, malformed workload parameters, and only sanitized aggregate observations.

If the gate is red, repair the concrete failure before any closure claim. Do not wait for GitHub Actions.

Continue immediately to E.

### E. BOUNDED RELEASE/SECURITY RECONCILIATION — repair the responder/source-domain claims

Update together after the final green developer SHA:

- `docs/reviews/resource-abuse-evidence-2026-09-04.md`;
- `docs/release-security-review-packet.md`;
- `docs/reviews/release-item4-subgates-20260909.md`;
- the local process/pre-auth provenance note(s) whose source-domain or responder-inventory wording changed.

Allowed claim: exact bounded local deterministic/process evidence exists for the exercised candidate values, cached-response ownership, malformed workload, cleanup/recovery result and exact tested tree.

Forbidden claims: D019 resolved, RSEC-001 fully closed, candidate limits production-suitable, independent review complete, public listener approved, RC/release/production ready.

Continue immediately to F.

### F. REVIEW-CLOSURE / READY_LOCAL — close this bounded RSEC engineering observation lane

Perform exactly one bounded adjacent review of the final package for:

- any remaining raw cached pre-auth/handshake response send that bypasses charging;
- source-domain misclassification;
- stale tested-tree/reviewer anchors;
- secret/private-topology leakage;
- accidental capacity/policy invention;
- concrete ownership/cleanup regression introduced by A-C.

Fix concrete defects found. If no BLOCKER/HIGH remains, explicitly mark this bounded engineering observation package complete. **Stop extending the RSEC checker/test harness after this closure unless a new demonstrated defect appears.**

Continue immediately to G.

### G. LOCAL OUTPUT SELECTION — choose the next real release/runtime/operator output

Re-read exact-current `README.md`, `AGENTS.md`, `ROADMAP.md`, `IMPLEMENTATION_PLAN.md`, `SECURITY.md`, `docs/status.md`, release packet and relevant code. Produce 1-3 dependency-ready proposals and autonomously choose the smallest safe one. Prefer in order:

1. a demonstrated runtime/correctness/security defect with a concrete call site and no policy invention;
2. an advertised operator/runtime behavior lacking direct executable positive/negative evidence;
3. a named release-evidence question answerable locally without capacity/security claim inflation.

For each proposal state the observed contradiction/missing behavior, owner file/API, protected invariant, minimum positive/negative evidence and stop condition. Do not wait for reviewer selection when one option is clearly smallest and within existing architecture/authorization.

One area may be **inspected**, not presumed broken: D019 says queue storage participates in the bounded memory model, while current `QueueReservation` is logical and the listener already reserves `16 KiB` per pre-auth state. Determine whether real queue-owned allocation exists outside that reservation before proposing any memory-accounting change. If it is already covered by reserved state memory, DEFER it and do not create a checker merely to fill the queue.

Do not choose D019 source-retention TTL/LRU/history policy, signing/key-custody policy, SBOM publication policy, previous-release interoperability without a frozen prior release, Experimental Track carriers, or generic checker work merely to manufacture hours.

Continue immediately to H.

### H. READY_LOCAL / ROLLING VISIBLE OUTPUT — implement, exact-tree close, repeat

Implement the chosen G output, run the exact-tree local gate, reconcile only changed facts, then repeat G -> H while concrete safe work remains. If the agent repeatedly completes high-quality coherent slices in 10-30 minutes, enlarge the next coherent closure package rather than stopping after each small commit.

The repository does not currently expose enough predetermined truthful work to promise an artificial 6-12 hours of fixed tickets. The proposal -> implementation -> gate loop is explicitly pre-authorized so queue exhaustion must be real, not a handoff-staleness or watcher artifact.

### I. CONDITIONAL VPS OUTPUT — only if exact-current truth creates a new live question

Current repository truth still says `READY_LIVE: none`. Standing VPS authorization remains valid but is not a reason to duplicate old evidence.

Only execute a VPS run if a new implementation/instrumentation change creates a named unresolved real-network question with satisfied dependencies. Otherwise do not unchanged-rerun HY2, repeated warm failover, periodic/soak, package lifecycle, distinct A -> B -> A, already-answered endpoint migration/key update, IPv6 without a real owned IPv6 path, or live PMTUD before its separate authenticated wire/security design gate.

### J. POLICY-BLOCKED PARALLEL LANE — D019 source retention

**Status:** `SOURCE_RETENTION_POLICY_BLOCKED`.

The current `ProcessPreauthAdmission::release` removes a source usage entry after its last live state disappears; the D019 no-reset/retention semantics remain a maintainer/security-policy question. Do not solve the cached-response HIGH by inventing TTL, LRU/history capacity, external authority, or a weaker reset rule. The HIGH can be repaired inside the existing bounded state lifetime without deciding terminal source retention.

## Coordination notes

- Open PR #3 remains stale relative to current main and is research/history only. It independently discussed response-permit/cached-response concerns on an old tree, but current main has already evolved substantially. Do not merge or cherry-pick it wholesale; reason from current code/tests/specs.
- `IMPLEMENTATION_COMPLETE=true` remains bounded research-implementation status only.
- `RELEASE_CANDIDATE=false`, `PRODUCTION_READY=false`, `FREEZE=false`, and `RELEASED=false` remain authoritative.
- Release item 3 remains incomplete and current opportunity classification remains `READY_LIVE: none`.
- Item 4 remains independent-review incomplete; developer-prepared factual support and this ChatGPT reviewer handoff are not independent audit/security approval.
- RSEC-001 remains open for promotion suitability/independent review after the engineering package closes.

## Stop conditions

Stop continuous coding only for a real condition:

- unresolved BLOCKER/HIGH correctness/security/evidence finding outside the already-authorized repair path;
- core Session/Carrier/ACK/crypto/wire architecture change;
- destructive/canonical-meaning migration;
- action outside standing authorization;
- production impact;
- new credentials/server/third-party permission;
- benchmark conditions requiring maintainer value judgment;
- repository/tool/runtime breakage preventing safe progress;
- actual runtime/tool-budget exhaustion;
- or a genuinely exhausted rolling queue after the required proposal cycle finds no concrete safe work.

Do not stop because GitHub Actions does not run, because this handoff becomes older than a developer commit, or because one coherent slice finishes before the next reviewer hour.
