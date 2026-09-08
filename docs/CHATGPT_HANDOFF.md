# Nekomusume ChatGPT Handoff

Checked at: 2026-09-08 15:02 Asia/Shanghai
Repository main HEAD reviewed before this handoff: `85cbca01ad87a965aabfd71450ec64ce39716324`
Previous reviewer handoff commit: `85cbca01ad87a965aabfd71450ec64ce39716324`
Previous checked implementation HEAD: `a4da7b244511f687e513f8276bdf15d305e53397`
Current execution branch: `work/e1a-staged-accounting-20260907` at exact `c81fa736bf6d551aab8405a5c32c104772496bf8`
New implementation commits under review: `18a93b725192165f72b94776b528ec5ee7581a0f` (`fix: bind endpoint challenge ownership`) + `c81fa736bf6d551aab8405a5c32c104772496bf8` (`feat: add authenticated endpoint rebinding runtime`)
Exact current execution-head GitHub Actions: run `34195337097` — `success`
Previous reviewer-head GitHub Actions: run `34192977090` — `success`
Historical partial-E2 branch retained: `work/continue-20260904` at `d271a99a2ab26abbcb146c411ba0fde697395abe`

## What changed

The coding agent consumed the prior review and made substantive runtime progress. Exact `18a93b7` repairs the earlier endpoint-state ownership defects: the peer no longer supplies the server challenge through `EndpointRebindCandidate`; the state stores exact trusted Session/epoch expectations; only the next candidate generation on the same path is accepted; a wrong Session/challenge is terminal for the pending candidate; and `source_tag` is explicitly an opaque token allocated only after an exact runtime socket-address comparison. The immediately retired source remains bounded to one entry.

Exact `c81fa73` then adds a real opt-in UDP endpoint-rebinding CLI path and process tests. It authenticates a Session on endpoint A, confirms one Session `DeliveryAck`, opens a genuinely different local UDP socket B, observes exact B, generates a fresh server-side challenge, validates an authenticated exact-tuple response from B, promotes B, and confirms a second application record on B. A real process negative reaches the candidate/challenge stage and fails on a wrong challenge; it is not a setup-only false positive. Exact-head CI is green.

This is a new demonstrable runtime capability and satisfies the visible-output requirement; do not divert the project back into generic checker/harness work. However, the current implementation still has two live-evidence correctness/semantic blockers that must be repaired before the self-owned VPS run.

## Review verdict

**CONTINUE_WITH_RUNTIME_BARRIER_REPAIR — accept the state-contract correction, real-socket endpoint-rebinding implementation direction, true process negative and green exact-head CI. Do not run the VPS endpoint-rebinding experiment from exact `c81fa73` yet. First add an authenticated promotion synchronization barrier and remove the current semantic misuse of an unsolicited D064 `ReadinessResponse` as a candidate announcement. Then re-run the focused local proof and exact-head CI and go directly to one bounded self-owned VPS source-endpoint-change observation.**

No administrator action is required for the preferred repair because it can remain inside the existing authenticated control vocabulary and current Session/Carrier architecture. If the coding agent concludes that a new ProcessMessage kind, a changed D064 readiness meaning, a new crypto/wire semantic, or a new numeric security policy is actually necessary, that specific choice becomes a maintainer checkpoint; do not silently invent it. While such a checkpoint is unresolved, continue the independent repeated-failover diagnostic lane rather than idling the project.

## Reviewer findings

### EPREB-006 — HIGH runtime ordering/evidence risk — client sends post-rebind traffic before it has authenticated proof that promotion completed

The server promotes B only after receiving and validating the server-generated challenge response. The client currently sends that response and then immediately may send a stale-A control and the second application `Data` from B. There is no authenticated server -> B confirmation between those steps.

That violates the intended invariant that the second record is **previously unassigned and emitted only after promotion**. UDP ordering across two different client sockets A and B is not guaranteed. On loopback the challenge response happened to arrive first, but on a real path the stale-A or B application datagram may arrive before the challenge response; the server currently treats an unexpected source while waiting for the challenge response as terminal. A successful loopback ordering therefore cannot be promoted into truthful WAN evidence.

Required behavior before live execution:

1. B becomes candidate-only;
2. server generates and validates its own fresh challenge against exact B;
3. server atomically promotes B;
4. server sends an authenticated exact-peer synchronization/confirmation to B that is emitted **only after** step 3;
5. client must receive and authenticate that confirmation before it emits the post-promotion application record.

Do not use Session `DeliveryAck` as the promotion confirmation; delivery and path/control evidence remain separate.

Preferred minimal proposal shape, if compatible with the existing D064 control semantics: let B originate a normal authenticated `ReadinessRequest` as its candidate/readiness request and keep its request identity pending; server independently performs its own fresh challenge/response against B; after promotion, server answers B's original request with the ordinary authenticated `ReadinessResponse`. The delayed ordinary response then acts as a synchronization barrier without becoming Session delivery or the server's path-validation proof. This is a proposal, not a mandated API signature: the agent may choose another existing-semantics shape that preserves the same ordering and evidence separation.

### EPREB-007 — HIGH spec/semantic drift — unsolicited `ReadinessResponse(challenge_id=0, admitted=false)` is not a candidate message

`ProcessMessage::{ReadinessRequest,ReadinessResponse}` is documented in `neko-session` as D064 standby-path control. The current endpoint client sends an unsolicited `ReadinessResponse` with `challenge_id=0` and `admitted=false`, and the endpoint server interprets that sentinel as “new endpoint candidate”. No preceding readiness request exists for that response.

This does not add a new wire kind, but it assigns a new protocol meaning to an existing response frame and risks conflating three evidence domains: D064 readiness/resource admission, endpoint candidate observation/path validation, and later Session delivery.

Repair before VPS evidence:

- do not use an unsolicited `ReadinessResponse` sentinel as the candidate announcement;
- reuse an existing control exchange only with its ordinary request/response meaning, or treat exact-source authenticated control arrival as candidate-only metadata without promoting it to readiness/path evidence;
- keep the server-generated validation challenge independent from the peer candidate signal;
- tracked events must distinguish `candidate_seen`, server challenge/validation, promotion synchronization, and Session `DeliveryAck`.

If no existing message semantics can express the candidate/request + server-validation + post-promotion synchronization sequence honestly, stop this endpoint lane at a **WIRE_SEMANTIC_CHECKPOINT** rather than adding a new message kind or redefining D064 on your own. That checkpoint does not block the independent diagnostics lane.

### EPREB-008 — MEDIUM evidence boundary — carrier path generation and authenticated record context are not yet reconciled

The endpoint state/control currently models active generation `0` -> candidate generation `1`, while the `SecureSession` is created with the CLI `RecordContext` whose `path_generation` is already `1` and remains fixed for the record session. Post-promotion application data therefore continues under the existing crypto record context even though carrier events report a generation transition.

This is not by itself evidence that the authenticated crypto context migrated to the new path generation. Before status/evidence reconciliation, choose the smallest honest shape already supported by current architecture:

- either implement a synchronized path-generation context advance that is already implied by existing `RecordContext` / Session migration invariants, with no nonce reset/reuse and deterministic old-context rejection tests; or
- narrow the experiment claim so carrier/source-binding generation is explicitly separate from crypto `RecordContext.path_generation`, and do not claim cryptographic path-generation migration.

The agent may propose the local API shape. Do not introduce a new wire field, delivery ACK meaning, key schedule or numeric policy merely to make labels line up.

### EPREB-009 — MEDIUM diagnostics — future generation is mislabeled as `OldGeneration`

`EndpointRebindState::observe_candidate()` currently returns `OldGeneration` for every generation that is not exactly `active + 1`, including a too-future generation. The test even records future generation 5 from active generation 3 as `OldGeneration`. Rejection is safe, but the classification is false and would contaminate structured failure evidence.

Use `OldGeneration` only for stale/older values; classify a future/non-next generation as tuple/generation mismatch. No new policy is needed.

### EPREB-010 — ACCEPTED implementation/evidence direction

The following are accepted and should be preserved:

- exact `18a93b7` server-owned challenge state and terminal pending-candidate failure;
- exact runtime `SocketAddr` comparison before assigning bounded opaque source tags 1/2;
- fresh nonzero server challenge generated after exact B is observed;
- exact-source checks on challenge response and post-rebind Data;
- one bounded retired old source, not unbounded source history;
- a true process-level wrong-challenge negative that launches both endpoints and reaches candidate/challenge handling;
- green exact-head CI at `c81fa73`.

Do not rewrite these into a generic audit framework. Repair the concrete runtime boundary and continue outward.

## Evidence and repository boundaries

- Exact `c81fa73` is **local/CI real-socket implementation evidence**, not WAN/NAT evidence and not yet acceptable as the VPS candidate because EPREB-006/007 remain open.
- The loopback test proves A and B are distinct local socket endpoints and that the current ordered run can reach promotion and a second `DeliveryAck`; it does not prove ordering safety across a real WAN path.
- The current stale-old-A process check is useful locally, but do not make the VPS positive depend on cross-socket arrival ordering of stale A versus the new B application record. Keep the WAN run minimal; deterministic stale-source rejection can remain a local negative unless a clean explicit synchronization is added.
- NAT/source-endpoint change remains `BLOCKED_IMPLEMENTATION` in ROADMAP/IMPLEMENTATION_PLAN/status until the barrier/semantic repair passes local + exact-head gates and one bounded self-owned VPS observation is retained.
- Accepted migration-back evidence at exact `5d6582c`/`f024458` and live key-update evidence at exact `2f4f59a`/`69d0ed9` remain valid and narrow; do not rerun them unchanged.
- C2 source-retention remains an explicit release/security policy limitation. Do not invent TTL/LRU/history/epoch values.
- `RELEASE_CANDIDATE=false`, `PRODUCTION_READY=false`, `FREEZE=false`, `RELEASED=false` remain unchanged.
- IPv6 remains environment-blocked.
- `main` still contains reviewer documentation lineage, not the accepted migration-back/endpoint runtime lineage. Do not describe default `main` as already carrying these runtime capabilities.
- Protected identities, SSH keys, credentials, raw private addresses and raw private diagnostics remain unread/untracked/uncommitted.

## Rolling Work Queue

This is a continuous pre-authorized queue. One commit, one test, one reviewer interval or one nominal hour is never a stop condition.

### A — Repair endpoint candidate/control semantics and add post-promotion synchronization

**Status:** `READY_LOCAL`; highest priority correctness repair. Proposal authority applies.

**Files/concepts:** `crates/neko-cli/src/main.rs`, `crates/neko-session/src/lib.rs` only if no codec/wire change is needed, endpoint state helper/tests in `crates/neko-carrier/src/lib.rs`.

Implement EPREB-006/007 with the smallest existing-semantics design. Before coding, make a short developer-owned proposal if useful: 1–3 shapes, invariant/risk/minimum test; choose the smallest new state/API with no new wire kind or numeric policy.

Protected invariants:

- candidate arrival is authenticated but candidate-only; it is not readiness admission, path validation or Session delivery;
- server owns the fresh validation challenge;
- exact B + Session/path/generation/epoch/challenge must validate before promotion;
- client emits no new application Data from B until it has authenticated a server message that can only occur after promotion;
- no Session `DeliveryAck` is repurposed as migration proof;
- no raw endpoint material enters tracked logs.

If the only honest implementation requires a new ProcessMessage kind or a changed normative meaning of D064 readiness, mark this endpoint lane `WIRE_SEMANTIC_CHECKPOINT` and continue immediately to E rather than idling.

**Continue immediately to B when resolved:** yes.

### B — Reconcile generation semantics and prove the runtime barrier locally

**Status:** `PREAUTHORIZED_AFTER_A`.

Resolve EPREB-008/009 without inventing policy. Keep the claim narrow if crypto context generation is not advanced. Correct future-generation diagnostics.

Process-level proof must include:

- genuine local A != B source endpoint;
- one pre-rebind application `DeliveryAck`;
- candidate request/signal distinct from server validation evidence;
- fresh server challenge exact-peer round trip;
- server promotion;
- authenticated post-promotion synchronization received by B;
- only then one previously-unassigned B application Data + Session `DeliveryAck`;
- true wrong-challenge/source/tuple negative reaches the validation stage and produces no promotion/post-delivery/success;
- deterministic stale-old-A rejection remains local and cannot create delivery/validation evidence.

Add a test hook that delays server validation/promotion and proves the client does not emit the second Data before the synchronization barrier; do not rely only on natural loopback ordering.

Run focused carrier/process tests, `./scripts/check.sh`, `git diff --check`. Fuzz smoke is required only if the wire codec/parser actually changes; the preferred repair should not need that.

**Exact-head CI must be green before C. Continue immediately to C:** yes.

### C — One bounded self-owned VPS source-endpoint-change observation

**Status:** `PREAUTHORIZED_AFTER_B_GREEN`.

Standing authorization already covers this ordinary bounded self-owned UDP migration/rebinding experiment. Do not request another WAN permission.

Use the smallest genuine source endpoint change: one authenticated Session, endpoint A -> distinct ephemeral endpoint B against the controlled VPS, one pre-rebind confirmed record, candidate + independent server validation + post-promotion synchronization, one post-promotion confirmed record, then cleanup. Do not require a stale-A race in the live positive.

Retain exact commit/binary, actual parameters, start/end time, hash-safe/source-endpoint inequality without raw private address material, semantic path/binding generation chosen in B, challenge and promotion-sync outcome, both Session `DeliveryAck`s, client/server exits, cheap resource observations when already available, and cleanup verification.

Claim only one bounded authenticated source-port/endpoint-change observation. Do not claim general NAT traversal, roaming reliability, public reachability, performance or production readiness. Preserve the first meaningful negative; no unchanged retry.

**Continue immediately to D:** yes.

### D — Evidence/status reconciliation and lineage integration

**Status:** `PREAUTHORIZED_AFTER_C`.

Update only facts actually established in `ROADMAP.md`, `IMPLEMENTATION_PLAN.md`, `docs/status.md` and one compact evidence note. If the run proves only source-port rebinding under a fixed crypto context, say exactly that. Do not call it general NAT traversal.

Then integrate accepted migration-back + endpoint-rebinding implementation/evidence lineage into `main`, preserving the newest reviewer-owned handoff and normal history. Do not manufacture additional coordination-only merges before the closure.

**Continue immediately to E:** yes.

### E — Repeated warm-failover diagnostic blind spot

**Status:** `READY_LOCAL`; also the immediate fallback if A hits `WIRE_SEMANTIC_CHECKPOINT`.

Do not rerun exact `9fd2411` / `a117086`. Add bounded sanitized inner-collector failure categorization for at least startup/setup, negotiation/auth, readiness, application/runtime, evidence serialization and cleanup, then propagate that category to the existing outer typed result.

Synthetic/local proof first. This is direct instrumentation for a previously observed live blocker, not a general audit framework.

**Continue immediately to F after green:** yes.

### F — One materially changed repeated-failover live attempt

**Status:** `PREAUTHORIZED_AFTER_E_GREEN`.

Run exactly one changed-hypothesis bounded self-owned attempt after the new inner failure category is retained and exact-head CI is green. Preserve any valid prefix, failure category, exact binary/parameters and cleanup. If the same classified failure repeats without a new hypothesis, stop that lane rather than mechanically retrying.

### G — Next release-matrix gate selection

**Status:** `REVIEWER_CHECKPOINT_AFTER_D/F`.

Re-read the release matrix after endpoint/repeated-failover closure. Live PLPMTUD remains non-autonomous until its live wire/control/policy gate is explicit. Do not implement FEC/0-RTT/striping/multipath/exotic carriers merely to keep busy. Prefer the next real release-matrix blocker that can be closed without inventing core wire or numeric security policy.

## Stop conditions

Administrator escalation remains limited to: core Session/Carrier/ACK/crypto/wire architecture change; new numeric security policy/ADR value; destructive migration; work beyond standing authorization; possible production impact; new credentials/server/third-party authority; benchmark objective requiring maintainer value judgment; an unresolvable major security issue; or a genuinely new project phase.

The endpoint runtime should continue autonomously under the existing architecture **unless** the agent determines that honest candidate/promotion synchronization requires a new wire kind or changed D064 normative semantics. In that case stop only that lane, record the exact design checkpoint, continue the independent diagnostics lane, and wait for reviewer/maintainer resolution rather than silently inventing protocol meaning.
