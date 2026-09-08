# Nekomusume ChatGPT Handoff

Checked at: 2026-09-08 15:58 Asia/Shanghai
Repository main HEAD reviewed before this handoff: `d4434dc6745abf44f1a4e7d92a0f4260ab2fb2b9`
Previous reviewer handoff commit: `d4434dc6745abf44f1a4e7d92a0f4260ab2fb2b9`
Previous checked implementation HEAD: `c81fa736bf6d551aab8405a5c32c104772496bf8`
Current execution branch: `work/e1a-staged-accounting-20260907` at exact `7405da4e9275e596b1e2db67494390ef742c4271`
New implementation/evidence commits under review: `a8f49fcbffa7c0e13063db8772470d7b41f3e55d` (`fix: synchronize endpoint promotion`) + `7405da4e9275e596b1e2db67494390ef742c4271` (`docs: accept barrier endpoint evidence`)
Exact implementation-head GitHub Actions: `a8f49fc` run `34199496127` — `success`; current execution-head `7405da4` run `34201786719` — `success`
Previous reviewer-head GitHub Actions: `d4434dc` run `34198142169` — `success`
Historical partial-E2 branch retained: `work/continue-20260904` at `d271a99a2ab26abbcb146c411ba0fde697395abe`

## What changed

The coding agent resolved the two HIGH endpoint-rebinding findings instead of inventing a new wire message. Endpoint B now originates an ordinary authenticated `ReadinessRequest` as candidate/request input; the server treats that arrival as candidate-only metadata, generates its own independent fresh nonzero challenge, validates the exact Session/path/generation/epoch/challenge response from exact B, atomically promotes B, and only then sends the authenticated response to B's original request. The client blocks on and authenticates that post-promotion response before it can send the second previously-unassigned application `Data` from B.

This closes EPREB-006/007 structurally. The former unsolicited `ReadinessResponse(challenge_id=0, admitted=false)` sentinel is gone, path/control evidence remains separate from Session `DeliveryAck`, and post-rebind application traffic has an authenticated server->B synchronization point that can occur only after promotion. Exact `a8f49fc` also corrects future/non-next generation classification to tuple mismatch rather than `OldGeneration`.

The agent chose the narrower honest resolution for EPREB-008: carrier/source-binding generation advances 0 -> 1, while the authenticated crypto `RecordContext.path_generation` remains the existing fixed context. The implementation and evidence do not claim cryptographic path-generation migration.

After green exact-head CI, one fresh bounded self-owned VPS observation was retained at `docs/notes/endpoint-rebind-vps-a8f49fc-20260908.md`. The run used one Session, two 16-byte application records, a temporary UDP listener and a real source-port change. The recorded order is A confirmation -> B candidate/request -> server fresh challenge -> exact response -> promotion -> authenticated promotion sync -> B application Data -> Session `DeliveryAck`; both endpoints exited 0 and cleanup was independently verified. Raw endpoints, credentials, private keys and payloads are not tracked. This is accepted only as one same-Session source-port/source-binding change under the fixed crypto record context; it is not general NAT traversal, roaming/public reachability, natural recovery, performance, reliability or production evidence.

ROADMAP, IMPLEMENTATION_PLAN and status were reconciled to `ALREADY_SUFFICIENT_FOR_BOUNDED_QUESTION` for that narrow endpoint question. Exact `c81fa73` remains superseded candidate provenance. There is no reason to rerun endpoint rebinding unchanged.

The visible-output check is satisfied: the last 24–48 hours produced synchronized live key update, scripted migration-back/recovery, a real endpoint-rebinding runtime and fresh VPS observations. Do not return the project to generic checker/schema hardening. The next harness work is justified only because it directly unlocks a previously failed repeated-warm-failover live question.

## Review verdict

**ACCEPT_ENDPOINT_REBINDING_CLOSURE_AND_ADVANCE_REPEATED_FAILOVER_DIAGNOSTICS.**

No new HIGH/BLOCKER was found in exact `a8f49fc`/`7405da4` for the narrow claim above. Exact-head CI is green and the fresh VPS evidence is acceptable within its stated boundary. Do not repeat that live run merely to obtain another PASS.

One MEDIUM regression-proof gap remains: the previous handoff asked for a deterministic test that delays server validation/promotion and proves the client cannot emit the second B application record before the authenticated promotion synchronization response. The current code is serial/blocking in the correct order, and the accepted VPS observation records the correct order, so absence of that delay hook does **not** invalidate the narrow evidence. Add one compact deterministic regression test, but do not make it a reason to rerun the VPS observation or to delay the outward repeated-failover lane.

The next real blocker is repeated warm failover diagnostics. The outer six-cycle runner already distinguishes generic collector outcomes (`nonzero_exit`, `timeout`, malformed output, missing event, invalid evidence), but the inner `run-live-warm-failover-cycle.py` still collapses all collector-side `CollectionError` failures to one free-text stderr line and exit 2. That is why the exact `a117086`/`c6ab8fd` line can retain `invalid_cycle_evidence` / `missing JSON event: start` but cannot safely distinguish setup/startup, negotiation/auth, readiness, application/runtime, evidence validation/serialization, or cleanup without reading a private diagnostic manually. This is a concrete instrumentation blind spot from an observed failed WAN run, not a request for a new audit framework.

## Reviewer findings

### EPREB-006 — CLOSED

Authenticated post-promotion synchronization now exists. Server promotion precedes `endpoint_promotion_sync_sent`; client requires exact-peer authenticated `endpoint_promotion_sync_received` before the second B application record is constructed/sent through the positive path. Session `DeliveryAck` remains separate.

### EPREB-007 — CLOSED

The unsolicited response sentinel is removed. B uses an ordinary authenticated request; the server's independent challenge supplies freshness/path validation; the delayed ordinary response is synchronization after promotion. No new `ProcessMessage` wire kind, crypto primitive, delivery-ACK meaning or numeric security policy was introduced.

### EPREB-008 — CLOSED BY NARROW CLAIM

Carrier/source-binding generation changes 0 -> 1. Crypto `RecordContext.path_generation` stays fixed and is explicitly excluded from the claim. Do not later rewrite this evidence as cryptographic path-generation migration.

### EPREB-009 — CLOSED

Stale generation remains `OldGeneration`; future/non-next generation is rejected as `TupleMismatch`.

### EPREB-011 — MEDIUM regression guard — deterministic promotion-delay proof still absent

The implementation's blocking order is correct, but there is no focused process regression that deliberately holds server promotion/sync and proves no second B application Data can arrive during that hold.

Preferred minimum shape: add a test-only server delay/barrier hook or an equivalent deterministic interception seam. During the hold, the server must be able to assert that no post-rebind application Data was accepted/sent as success evidence; after promotion + sync, the same test completes the second Data/DeliveryAck. Keep it fixed/bounded and test-only. This is not a protocol timer or numeric security policy.

Do not add another general endpoint state framework. Do not rerun the VPS evidence after a test-only guard unless the positive runtime semantics themselves change.

### RWFDIAG-001 — HIGH for the repeated-failover evidence lane, not for endpoint/runtime correctness — inner collector failure stage is not propagated

Current outer `run-repeated-warm-failover.py` can retain a sanitized private stderr hash and a generic diagnostic category, but `run-live-warm-failover-cycle.py` catches broad collector errors and emits only `live failover collector: <text>` before exit 2. A failed no-row cycle therefore loses the stage boundary in the tracked typed result.

Implement the smallest stage-carrying seam. Proposal authority applies. A good minimal shape is:

- preserve existing valid-cycle row semantics unchanged;
- maintain a bounded inner stage such as `setup`, `server_startup`, `negotiation_auth`, `readiness`, `application_runtime`, `evidence_validation`, `cleanup`;
- on a no-row collector failure, emit exactly one bounded machine-readable failure marker to stderr containing only a stage and a small stable reason code, never raw argv/endpoints/secrets/log text;
- make the outer runner recognize only that marker and propagate the stage/category into its existing typed `first_failure`, while retaining the sanitized private stderr hash as before;
- if the inner process dies before it can emit a valid marker, keep the current generic `nonzero_exit`/timeout behavior instead of inventing specificity;
- historical artifacts must remain schema-valid and immutable. Prefer an optional/backward-compatible field or enum extension over rewriting old results; do not create a new evidence framework unless the existing v1 shape truly cannot carry the distinction.

The stage classification must reflect the earliest evidence boundary actually known. Do not infer “negotiation failed” merely because later events are absent. For example, a server missing its validated `start` event is startup evidence; only after valid start evidence should negotiation/auth/readiness/application stages become claimable.

Synthetic tests should exercise at least one genuine failure in each supported stage and verify that tracked JSON contains only the bounded stage/reason plus hash metadata, not the raw private diagnostic.

### RSEC-001 — remains HIGH for release/security promotion, not a reason to block the outward diagnostics lane

Process-owned pre-auth accounting exists, but independent source-projection/charge-order/listener-coverage review plus bounded adversarial concurrency/rate/expiry evidence remain open before public-listener/RC/security promotion. Do not change candidate numeric limits merely to make tests pass. Keep this as a later focused release-gate package after the repeated-failover outward closure; do not let security tooling become the only active lane.

## Evidence and repository boundaries

- Exact `a8f49fc` is accepted implementation + exact-head CI for the repaired endpoint runtime; exact `7405da4` is its evidence/status reconciliation commit.
- `docs/notes/endpoint-rebind-vps-a8f49fc-20260908.md` is one bounded real VPS/source-port observation, not general NAT/public reachability evidence.
- Exact `c81fa73` remains superseded candidate evidence; do not upgrade or delete it.
- Accepted migration-back evidence at exact `5d6582c`/`f024458` and live key-update evidence at exact `2f4f59a`/`69d0ed9` remain valid and narrow; no unchanged reruns.
- Repeated warm failover remains `BLOCKED_DIAGNOSTICS`; exact `9fd2411`, `a117086` and `c6ab8fd` negatives are retained. No unchanged WAN retry is allowed until RWFDIAG-001 materially changes instrumentation/hypothesis.
- Live PLPMTUD remains `BLOCKED_IMPLEMENTATION`; its live probe/ACK wire/control/accounting/timer choices are not pre-authorized design inventions. Do not use it as filler work.
- IPv6 remains environment-blocked.
- `RELEASE_CANDIDATE=false`, `PRODUCTION_READY=false`, `FREEZE=false`, `RELEASED=false` remain unchanged.
- Default `main` is still reviewer-doc lineage at the start of this handoff; it does not yet contain the accepted migration-back/endpoint implementation/evidence lineage. That divergence should now be closed once, before new unreviewed implementation is mixed into the integration commit.
- Protected identities, SSH keys, credentials, raw private addresses and private diagnostics remain unread/untracked/uncommitted.

## Rolling Work Queue

This is a continuous pre-authorized queue. One commit, one nominal hour, one reviewer interval or one successful test is never a stop condition. If a slice finishes quickly and cleanly, continue to the next dependency-satisfied slice.

### A — Integrate the already-accepted runtime/evidence lineage into `main`

**Status:** `READY_NOW`; coordination closure, do it once.

Before starting new implementation, fetch current reviewer `main`, merge it into the execution branch without editing `docs/CHATGPT_HANDOFF.md`, then integrate the already-reviewed work through exact `7405da4` into `main` while preserving this newest reviewer handoff. Do not include any later unreviewed diagnostics implementation in that integration commit.

The purpose is to end the long default-branch truth split: after this closure, `main` should actually contain the accepted migration-back + endpoint-rebinding runtime/evidence lineage, not merely reviewer prose describing it. Run/observe exact-main CI. A merge commit or clean fast-forward shape is acceptable; do not rewrite history.

**Continue immediately to B/C on the work branch:** yes. Exact-main CI need not idle independent local work, but do not call integration complete until it is green.

### B — Add the endpoint promotion-delay regression guard

**Status:** `READY_LOCAL`; small bounded test-only repair; may run in parallel/order-adjacent to C after A is safely branched.

Implement EPREB-011 with the smallest test hook. Files should stay near `crates/neko-cli/src/main.rs` and `crates/neko-cli/tests/probe.rs`; do not touch wire/crypto/spec unless a real defect forces it.

Required proof: hold promotion/sync deterministically after B has answered the server challenge; prove no second B application Data is accepted/emitted before sync; release the hold; prove promotion sync then second Data + Session `DeliveryAck` complete. Existing wrong-challenge and stale-source tests remain valid.

Focused test + `./scripts/check.sh` + `git diff --check`. No VPS rerun for a test-only guard.

**Continue immediately to C:** yes.

### C — Carry inner repeated-failover failure stage into the existing typed result

**Status:** `READY_LOCAL`; highest-value new implementation slice after the tiny regression guard. Proposal authority applies.

Files/concepts: `scripts/bench/run-live-warm-failover-cycle.py`, `scripts/bench/run-repeated-warm-failover.py`, their existing tests, and `schema/repeated-warm-failover.v1.json` only as minimally necessary.

Implement RWFDIAG-001, not a generic logging system. Preserve valid cycle rows, six-cycle sequential semantics, existing secret-safe private diagnostic hashing, exact binary/commit checks, cleanup behavior and historical artifact validity.

At minimum distinguish setup, startup, negotiation/auth, readiness, application/runtime, evidence validation/serialization and cleanup when the evidence actually permits that distinction. Propagate a bounded stable stage/reason through the existing outer failure object. Raw stderr remains private/sanitized/hash-only and must not enter tracked evidence.

Tests must show at least one stage-specific failure reaches the outer typed result, plus a malformed/unrecognized inner failure that correctly falls back to generic `nonzero_exit`/invalid evidence. Existing success/failed-row behavior must remain unchanged.

**Continue immediately to D:** yes.

### D — Local/exact-head gate for the diagnostics change

**Status:** `PREAUTHORIZED_AFTER_C`.

Run the focused inner/outer runner tests, schema validation, `./scripts/check.sh`, `git diff --check`; fuzz is required only if an actual parser/wire codec changed, which this slice should not require. Commit and push the instrumentation/test package. Require green exact-head CI before the live changed-hypothesis attempt.

If tests reveal that the stage cannot be known safely, report a coarser true stage; do not infer a deeper runtime cause from missing later events.

**Continue immediately to E after green:** yes.

### E — Exactly one materially changed repeated-warm-failover live attempt

**Status:** `PREAUTHORIZED_AFTER_D_GREEN`.

Standing authorization already covers this bounded self-owned client <-> VPS TCP/UDP failover experiment. Do not ask again for WAN permission.

This run is justified because RWFDIAG-001 materially changes instrumentation and the diagnostic hypothesis. Execute exactly one bounded changed-hypothesis outer attempt. Preserve exact commit/binary, actual parameters, any valid cycle prefix, typed inner stage/reason if available, exits/resource fields that actually exist, start/end time and cleanup. If it fails, preserve the negative. Do not mechanically rerun the same classified failure without another material instrumentation/code/config/path/hypothesis change.

A stage classification is not a root-cause conclusion. `startup` means the retained evidence stopped at startup; it does not prove why the remote process failed. Likewise `negotiation_auth` does not identify crypto/network cause unless more direct evidence exists.

**Continue immediately to F:** yes.

### F — Reconcile the repeated-failover result and choose only the next evidence-producing repair

**Status:** `PREAUTHORIZED_AFTER_E`.

Update a compact evidence/status note only if the live attempt creates a new fact. Preserve old negatives unchanged. If the new stage cleanly identifies a local implementation/orchestration defect with a safe fix inside existing architecture, fix it, test, commit and continue; if the same stage repeats without a new hypothesis, close this line as the new retained negative instead of retrying.

Do not spend a cycle expanding diagnostic schemas once the live blocker is discriminated enough to choose a real code/configuration fix or to conclude that the current line is blocked.

### G — Focused RSEC-001 adversarial closure package

**Status:** `READY_LOCAL_INDEPENDENT_AFTER_OUTWARD_CLOSURE`; security parallel lane, not a project-wide stop.

After E/F has produced the outward result, re-read exact current pre-auth implementation and `docs/reviews/resource-abuse-evidence-2026-09-04.md`. Build one bounded adversarial package around the **existing** candidate limits: source projection, charge-before-parse/response accounting, all real listener call-site coverage, concurrent reservations, rate-window rejection, expiry/release and cleanup/redacted counters. Do not invent or tune numeric limits merely to obtain PASS.

Prefer process/unit adversarial tests that close the actual RSEC-001 evidence requirements; do not create another checker framework. Commit/push tests and exact-tree evidence for reviewer challenge. This can progress while unrelated blocked WAN rows remain blocked.

### H — Next release-matrix/output gate

**Status:** `REVIEWER_CHECKPOINT_AFTER_F/G`, with autonomous safe work allowed where dependencies are already explicit.

Re-read the exact release matrix after the repeated-failover and RSEC packages. Prefer an executable/package/operator/release capability or a real VPS question that is already semantically specified. Live PLPMTUD remains a design checkpoint while probe/ACK wire fields, header accounting, PTB validation, timer/cooldown and fragmentation semantics are not settled; FEC/0-RTT/striping/multipath/exotic carriers remain non-TODO unless an observed problem selects them.

If the next safe release gate is package/operator install/upgrade/rollback in the dedicated experimental path, that is within standing authorization and preferable to speculative protocol work. If instead the choice requires a new wire meaning, numeric security policy or maintainer value judgment, freeze only that lane and continue any independent READY work.

## Stop conditions

Administrator escalation remains limited to: core Session/Carrier/ACK/crypto/wire architecture change; new numeric security policy/ADR value; destructive migration; work beyond standing authorization; possible production impact; new credentials/server/third-party authority; benchmark objective requiring maintainer value judgment; an unresolvable major security issue; or a genuinely new project phase.

Endpoint rebinding is now closed for its narrow bounded question. Repeated-failover diagnostics, one changed-hypothesis self-owned live run, and focused existing-limit RSEC testing are all within current architecture/authorization and should proceed autonomously. Do not stop because a commit completed, an hour elapsed, or a reviewer interval ended.
