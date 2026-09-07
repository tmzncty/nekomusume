# Nekomusume ChatGPT Handoff

Checked at: 2026-09-07 19:00 Asia/Shanghai
Repository main HEAD reviewed: `5b8be61d800ba282e18fb19e6e45238010f44a25`
Previous checked implementation HEAD: `14fe30d72b447a0b6fe94664734d184ff2995f36`
Current execution branch: `work/e1a-staged-accounting-20260907` at exact `f066af59a04a2e0fb83b10cfe6bdb2b530be10e4`
Historical partial-E2 branch retained: `work/continue-20260904` at `d271a99a2ab26abbcb146c411ba0fde697395abe`

## What changed

The coding agent continued through the previous narrow E1A closure instead of stopping after one repair. The active branch is now ten commits ahead of reviewer `main` and has integrated the current reviewer lineage normally; no force-push/rewrite is involved.

Substantive new checkpoints after the previously reviewed `14fe30d` are:

- `164731d` — **real staged-admission correctness closure**: public process staged `extend` now terminalizes permit/state on every returned error through a checked-inner/public-wrapper pattern; failed `complete`/`abandon` also consume/reject safely; direct inner staged extension failure becomes non-retryable. It also adds direct staged-reader event/failure tests and the requested cross-layer all-or-terminal regression.
- `900ea76` — ordinary merge of current reviewer `main` into the work branch; coordination only, no implementation claim by itself.
- `f7e2cf1` — **C1 carrier-aware source projection**: all current pre-auth listener call sites identify TCP vs UDP explicitly, the bounded source key carries a carrier discriminator before family/address/port, and deterministic tests prevent carrier/family/port aliasing.
- `f066af5` — records a focused ADR amendment request for the remaining D019 terminal-source-retention conflict without inventing a TTL, LRU size, history bound or other security-policy number.

Exact `f066af59a04a2e0fb83b10cfe6bdb2b530be10e4` GitHub Actions run `34112531048` is green:

- `stable checks` — `bash scripts/check.sh` success;
- `nightly decode fuzz smoke` — pinned `cargo-fuzz` build plus 30-second decode fuzz success.

This is exact-head local/CI/fuzz evidence. It adds no WAN result, performance conclusion, release approval, protocol freeze or production claim.

The active branch now contains all four TCP staged migrations, structural staged terminality, the direct staged-reader negative matrix, the all-or-terminal cross-layer regression, and C1 carrier/source separation. The previous E1A implementation/security findings are therefore closed at the current bounded implementation level.

The next real local gap is **E2 semantic responder closure**, not more E1A work. Current runtime mostly has the intended pending-UDP queue lifecycle, but the active inventory/checker lost useful semantic assertions that existed on historical `d271a99`, and one pending-UDP receive path still performs duplicate classification before charging that received datagram's bounded pre-auth work.

## Review verdict

**CONTINUE — E1A CLOSED at exact `f066af5`; C1 CLOSED; C2 isolated as a release/security policy checkpoint. Finish E2 now, then deliberately leave the audit-only loop and return to a concrete runtime seam + truthful VPS evidence.**

Do not reopen solved staged-TCP work absent a concrete regression. Do not invent a source-retention number. The C2 policy conflict blocks a claim of literal D019 source-retention compliance and therefore blocks final D019/release-security closure, but it does **not** block independent compatibility/package/runtime research or bounded self-owned VPS work once a truthful `READY_LIVE` row exists.

No administrator action is required for the current execution queue. The terminal-source-retention decision is deferred to an explicit release/security policy review; until then preserve bounded cleanup and state the limitation rather than pretending D019 is fully satisfied.

## Reviewer findings

### RSEC-001E1A — CLOSED at `164731d` / exact `f066af5`

The staged single-record contract now satisfies the reviewed bounded implementation requirements:

- one TCP frame owns one input record/packet while bytes/work are staged;
- all four real TCP responder families use the shared staged helper before attacker-controlled length/body parsing: ordinary TCP, periodic TCP, multistream TCP and failover TCP;
- header accounting precedes length interpretation and body reservation precedes body allocation/read;
- non-complete staged reads explicitly abandon/terminalize the logical ticket without refunding attacker-caused reserved input;
- inner staged extension failure is one-shot/non-retryable;
- process staged `extend` terminalizes permit/state on backwards time, liveness/source/window/arithmetic/limit and other returned errors;
- failed staged `complete`/`abandon` consume/reject rather than becoming successful later;
- direct tests cover success stage ordering, oversize-after-header/no-body-allocation, truncation, zero body, callback rejection, backwards time, per-record work failure and cross-layer outer rejection;
- exact-head stable checks and decode fuzz are green.

This is deterministic local security-accounting evidence only. It is not a security audit or WAN validation.

### RSEC-001E2 — MEDIUM/HIGH — pending UDP semantic ordering/evidence guard is not yet closed

The current failover-UDP runtime direction is mostly correct:

- new pending ownership charges input before negotiation parse;
- response is charged/sent before queue ownership;
- `preauth.enqueue(&mut admission)` occurs before `pending = Some(PendingUdpNegotiation { ... })`;
- enqueue/parse rejection releases the admission state;
- process expiry consumes state/queue accounting and the application owner invalidates the stale queue permit;
- successful authentication takes pending ownership, dequeues exactly once, releases admission, then emits negotiated/authenticated events.

Two concrete closure gaps remain:

1. **Pending UDP duplicate discrimination is performed before the datagram's pre-auth charge.** In the existing-pending branch, `datagram == pending_state.hello.as_slice()` is evaluated before the subsequent `charge_input`. This is attacker-controlled bounded comparison work before accounting and conflicts with D019's charge-before-protected-work rule. Repair this with the smallest fail-closed design. A conservative one-record charge before duplicate/Noise discrimination is acceptable if it preserves existing source/global/packet limits; an equivalent staged datagram reservation is also acceptable. Do not double-count one received datagram as two packets merely because work is refined after classification.

2. **The active semantic inventory/checker no longer requires explicit pending-owner reserve/store/cancel anchors.** Historical `d271a99` had `queue_reserve_anchor`, `pending_store_anchor`, and `pending_cancel_anchor` for the two pending-owner entries. Reuse the valid semantics, translated to current carrier-aware APIs, rather than cherry-picking stale text blindly. The checker should fail closed if a pending owner can store before reservation or loses its terminal cancellation/expiry anchor.

Required E2 closure:

- TCP entries keep staged-receive-before-negotiation/Noise anchors;
- ordinary/failover UDP charge after bounded raw receive and before protocol/duplicate/Noise work that needs accounting;
- pending queue reserve is structurally before store;
- enqueue rejection produces no pending owner;
- expiry invalidates application queue ownership exactly once;
- auth success dequeues/releases exactly once before success evidence;
- malformed/rejection/timeout/I/O paths cannot reach auth/readiness/Session/Delivery/PathValidated/ACK/authz-equivalent success anchors;
- expected externally reachable responder set remains explicit.

Use the existing inventory/checker plus targeted runtime tests. **Do not create another checker framework.**

### RSEC-001C1 — CLOSED at `f7e2cf1`

Current pre-auth source projection now explicitly includes a bounded carrier discriminator and preserves family/address/port. All real current admission call sites pass the actual TCP/UDP class, and deterministic tests prove TCP and UDP projections do not alias for the same socket address.

This closes only the noncontroversial projection portion. It does not resolve terminal source retention across retries/reconnects/carrier changes.

### RSEC-001C2 — POLICY CHECKPOINT / RELEASE BLOCKER, correctly isolated at `f066af5`

The new amendment request accurately captures the remaining D019 conflict:

- literal D019 language says counters are not reset by retry/reconnect/carrier change/identity change/error;
- current bounded in-process source accounting removes a source entry after its final live state is released, so a later attempt can obtain fresh per-source counters;
- retaining arbitrary attacker-controlled terminal sources forever creates unbounded source-map memory;
- the reviewed ADR supplies no terminal-source TTL, history size, LRU size or eviction semantics.

The request correctly avoids inventing a convenience number and records the main policy families.

**Reviewer disposition for current development:** preserve the current bounded cleanup behavior and explicitly treat literal terminal-source no-reset compliance as unresolved. Do not implement a retention TTL/LRU/history limit and do not claim RSEC-001/D019 fully closed. This is the conservative “record the limitation as a release/security blocker” posture until an explicit reviewed amendment selects another policy.

This policy wait blocks the final D019/release-security conclusion only. It must not freeze unrelated engineering.

## Evidence boundaries

- `IMPLEMENTATION_COMPLETE=true` remains a bounded research-baseline flag only.
- `CANONICAL_CORPUS_V1_FROZEN=true` remains corpus-specific only.
- `RELEASE_CANDIDATE=false`, `PRODUCTION_READY=false`, `FREEZE=false`, `RELEASED=false` remain required.
- A1 response-I/O deadline, B1 queue ownership/expiry, D1 ordinary terminal rejection and E1A staged TCP accounting are accepted closed bounded implementation findings unless a concrete regression appears.
- `f7e2cf1` adds carrier-aware pre-auth source projection; it does not prove no-reset semantics across terminal cleanup.
- `f066af5` records a policy conflict; it is not an approved ADR amendment or implementation authorization.
- Exact `f066af5` has green stable checks and green nightly decode fuzz smoke.
- Process admission still does not claim control over kernel SYN backlog, provider NAT state or resources outside the process.
- Historical WAN/HY2/failover/periodic positive and negative evidence remains immutable at its exact commit boundary.
- Standing VPS authorization remains valid for future dependency-ready self-owned work. A policy wait in C2 is not a reason to invent a new WAN authorization gate.
- Protected identity material, SSH private keys, credentials, private endpoint material and raw private diagnostics remain unread/untracked/uncommitted. Use maintainer-provided local secret configuration for owned VPS access; never put those secrets/addresses into repository evidence unless the existing redaction policy explicitly permits it.

## Rolling Work Queue

This is a multi-hour pre-authorized queue. The coding agent owns ordinary local design choices and should propose/choose the smallest fail-closed implementation shape inside current architecture. Finish one coherent closure package -> targeted/full gates -> commit -> push -> immediately continue to the next dependency-satisfied package. One commit, nominal hour, reviewer interval or proposal note is never a stop condition.

### A — Close E2 pending-UDP semantic ordering + inventory guard

**Status:** `READY_LOCAL`; highest-priority remaining pre-auth implementation/evidence package.

**Goal / why now:** close the last concrete responder-accounting seam without reopening solved E1A or creating more audit infrastructure.

**Files/concepts:** `crates/neko-cli/src/main.rs` failover pending-UDP receive path; `docs/preauth-responder-inventory.v1.json`; `scripts/check-preauth-responder-inventory.py`; targeted process/unit tests only where needed.

**Protected invariants:** one received datagram is one input packet; accounting precedes attacker-controlled protocol/duplicate classification work that consumes the budget; queue reservation precedes application ownership; terminal cleanup is exactly once; no protocol success evidence follows rejection/expiry; existing response/anti-amplification limits remain intact.

**Required behavior:** repair pre-charge duplicate classification with a conservative single-record design; reintroduce semantic pending-owner reserve/store/cancel requirements using current carrier-aware anchors; prove rejection/expiry/auth-success ordering. Do not clone a new checker or queue subsystem.

**Validation:** targeted tests + existing inventory checker + `scripts/check.sh` + `git diff --check`; fuzz only if untrusted parsing/framing semantics change materially (otherwise exact-head normal CI is sufficient).

**Commit/push:** one coherent runtime + semantic-inventory package preferred.

**Continue immediately to B:** yes if green; do not wait for reviewer merely because CI is pending if B is independent.

### B — Compatibility / freeze-boundary audit with defect-only changes

**Status:** `READY_LOCAL_AFTER_A`; independent of C2 policy wait.

**Goal:** verify corpus-v1 content-addressed freeze remains distinct from global protocol/release freeze and that current/current negotiation, unsupported/future rejection, transcript binding into Noise, resume/version binding and replay boundaries still match current code.

**Behavior:** inspect exact code/tests/specs; add a regression only for a concrete mismatch. Do not reopen frozen corpus bytes without correctness evidence and do not rewrite docs for style alone.

**Gate:** relevant tests + full gate for any code change; commit/push only if a real defect/evidence drift is corrected.

**Continue immediately to C:** yes.

### C — Package/operator + evidence-provenance integrity

**Status:** `READY_LOCAL_AFTER_B`; independent of C2 policy wait.

Verify the existing x86_64 package/build identity, dedicated-path install/readiness/smoke/upgrade/rollback contract, shutdown/listener/temp cleanup, canonical Git-blob/checksum manifests and exact-head CI references. Do not read protected identity material. Do not rerun already-sufficient VPS/package evidence merely for freshness; fix only a concrete defect or stale provenance link.

If no defect exists, record that the bounded question is already sufficient and move on without manufacturing a commit.

**Continue immediately to D:** yes.

### D — Reclassify release opportunities under the explicit C2 policy blocker

**Status:** `READY_LOCAL_AFTER_C`.

Re-evaluate every remaining release/evidence row against current implementation rather than stale 2026-09-03/04 labels:

- bounded question already answered -> `ALREADY_SUFFICIENT_FOR_BOUNDED_QUESTION`;
- executable specific missing assertion with dependencies satisfied -> `OPEN_READY` with exact `evidence_needed`, `next_action`, `requires`, `execution_scope`;
- implementation/environment/governance/review dependency absent -> exact blocker;
- C2 must remain a specific release/security policy blocker, not a generic blocker for all runtime research;
- never use generic `need WAN authorization` for work covered by standing authorization.

Do not preserve `READY_LIVE: none` merely because it was historically true. Recompute from current code.

**Required output:** a truthful current matrix/status update only if classifications actually changed.

**Continue immediately to E:** yes.

### E — Convert one high-value `BLOCKED_IMPLEMENTATION` runtime seam into a real runnable capability

**Status:** `PREAUTHORIZED_AFTER_D`; choose only a seam that current architecture can implement without maintainer-level design.

The current roadmap historically lists NAT/source-endpoint change, migration-back, live key update and live PMTUD as implementation-blocked. Briefly compare 1–3 candidates against current code and choose the one with the smallest new state/API, strongest existing primitive support, clearest deterministic validation and highest VPS evidence value. **Live key update is a natural candidate because a bounded synchronized key-update primitive already exists, but it is not mandatory if exact code shows another seam is safer/smaller.**

Package this as one visible-output closure:

1. real runtime integration, not a fixture-only wrapper;
2. bounded process/loopback validation;
3. structured instrumentation sufficient for a later real-socket result;
4. failure paths that do not fabricate Session/Delivery/PathValidated/ACK evidence;
5. no new wire/crypto policy unless already defined by current accepted candidate contract.

If implementing the selected seam would require a core Session/Carrier/ACK/crypto/wire decision or new security policy, do not improvise; choose another dependency-safe candidate or record the exact design gate.

**Gate:** targeted tests + `scripts/check.sh` + `git diff --check`; fuzz if untrusted wire/parser semantics change; commit/push.

**Continue immediately to F when the resulting release question is truthfully `READY_LIVE`:** yes.

### F — One bounded changed-capability VPS evidence run

**Status:** `PREAUTHORIZED_AFTER_E_WHEN_READY_LIVE_EXISTS`.

Use `docs/standing-vps-lab-authorization.md` and maintainer-provided local secret endpoint configuration. Execute **one** self-owned client<->VPS row that directly tests the newly runnable capability or another newly recomputed highest-value `READY_LIVE` question. Use the smallest profile that answers the question; the standing limits remain absolute ceilings, not targets.

Required evidence linkage:

- experiment ID;
- exact git/binary identity;
- actual parameters and timestamps;
- client/server structured result;
- relevant CPU/RSS/FD/socket observations when available;
- capture metadata only if a bounded capture is needed;
- explicit cleanup verification.

Positive or negative result is valid. Do not retry an unchanged failure. Do not run HY2 merely because it exists; HY2 needs a genuinely changed diagnostic hypothesis and a declared missing comparison question.

**Continue immediately to G:** yes after cleanup, regardless of positive/negative outcome.

### G — Reconcile the new runtime/VPS evidence in the same boundary

**Status:** `PREAUTHORIZED_AFTER_F`.

Update the relevant release matrix/status/evidence artifact so it says exactly what the run proved and what it did not prove. One bounded pass is not a reliability rate, public reachability claim, production-readiness claim or performance-superiority result.

If the run exposes a concrete runtime defect, put that fix first and continue. If the question is answered, classify it `ALREADY_SUFFICIENT_FOR_BOUNDED_QUESTION` rather than scheduling freshness reruns.

**Continue immediately to H:** yes.

### H — Independent D019/release-security work while C2 remains explicit policy wait

**Status:** `READY_LOCAL_FALLBACK`; do not let it displace the runtime/VPS path above.

Complete only D019/security checks that are independent of terminal-source retention: concurrency ceilings, global one-second windows, memory/queue/response/anti-amplification, idle/lifetime/100 ms response deadline, cancellation/double cleanup and no-success-evidence barriers. Reuse existing tests; add only missing distinct boundaries.

Do **not** claim full RSEC-001/D019 closure until C2 has an approved policy resolution. Do not invent a retention number. This is fallback/security debt work, not the main output lane while a runtime/VPS package is READY.

## 24–48 hour output check

The latest coding interval contains real output, not reviewer churn:

- all four TCP pre-auth responders now stage accounting before attacker-controlled length/body work;
- staged record failure semantics are structurally terminal and directly tested;
- exact-head stable+fuzz CI is green;
- carrier-aware pre-auth source projection is implemented;
- the remaining source-retention contradiction is isolated in an honest ADR request instead of being hidden behind an invented limit.

The next review interval should close E2 and then **move outward** toward one formerly implementation-blocked runtime capability and one truthful VPS evidence row. Do not spend another day expanding pre-auth checkers once A is closed.

## Completion gates

The current E1A/E2 local pre-auth closure package is complete when:

- E1A staged semantics above remain green;
- all seven inventoried responder surfaces retain correct accounting-before-protected-work order;
- pending UDP duplicate/protocol classification no longer precedes its required input/work charge;
- queue reservation is proven before pending ownership store;
- rejection/expiry/auth-success cleanup is structurally exactly-once and cannot emit forbidden success evidence;
- exact-head repository gate is green.

Full D019/RSEC-001 release-security closure additionally requires:

- an **approved** resolution of terminal-source retention/no-reset semantics that preserves bounded memory;
- the remaining independent adversarial matrix;
- exact-tree security/evidence review;
- independent release review as required by repository governance.

The broader project should not wait for that policy decision before independent runtime/VPS research proceeds.

## Do not expand into

- a new terminal-source TTL/LRU/history/eviction number without explicit reviewed policy;
- public or production listener deployment;
- new protocol/wire/Noise/Session/Carrier redesign unrelated to an observed blocker;
- repeated unchanged HY2/repeated-failover/periodic historical failures;
- VPS load testing as a substitute for deterministic correctness;
- speculative FEC/0-RTT/striping/multipath/exotic-carrier work without an observed-problem gate;
- reading, hashing, copying, modifying, uploading or committing protected identity/SSH-secret material;
- release/RC/freeze/production promotion.

## Questions requiring maintainer decision

**No decision is required to continue the current execution queue.**

A future release/security decision is required for D019 terminal-source retention. The exact conflict and non-numeric policy families are recorded in `docs/adr/m1-g0-preauth-source-retention-amendment-request.md`. Until that review occurs, keep current bounded cleanup, state the limitation honestly, and do not claim literal D019 terminal-source no-reset compliance.
