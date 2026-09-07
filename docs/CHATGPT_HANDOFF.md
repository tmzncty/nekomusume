# Nekomusume ChatGPT Handoff

Checked at: 2026-09-07 17:58 Asia/Shanghai
Repository main HEAD reviewed: `65acf6901d3de38b2354623145a55c6cbdb20542`
Previous checked implementation HEAD: `a6b0c3327c34a068fd66bc7dc44d63b959b9536b`
Current execution branch: `work/e1a-staged-accounting-20260907` at exact `14fe30d72b447a0b6fe94664734d184ff2995f36`
Historical partial-E2 branch retained: `work/continue-20260904` at `d271a99a2ab26abbcb146c411ba0fde697395abe`

## What changed

The coding agent consumed the previous two HIGH E1A findings instead of stopping after one slice. The active branch now contains current reviewer `main` plus two new implementation/test commits:

- `70d1e99` — ordinary non-force merge of current reviewer `main` into the execution branch; no implementation claim by itself.
- `3b684d5` — **real E1A runtime closure progress**: adds explicit staged-input abandonment, migrates ordinary TCP and failover TCP negotiation/first-Noise input to the shared staged helper, and updates their responder-inventory anchors.
- `14fe30d` — adds one-shot terminal ownership tests for inner staged input permits and process staged permits.

The full branch is now six commits ahead of current reviewer `main`, because it also retains the previously accepted `ccacec1` / `6974ff6` / `a6b0c33` staged accounting, staged framed-reader, periodic and multistream migrations. All four required TCP responder families now use the staged receive path for pre-auth negotiation / first Noise input:

1. ordinary TCP server/probe;
2. periodic TCP;
3. multistream TCP;
4. failover TCP.

Exact `14fe30d72b447a0b6fe94664734d184ff2995f36` GitHub Actions run `34106119750` is green. Both jobs succeeded:

- `stable checks`: `bash scripts/check.sh` success;
- `nightly decode fuzz smoke`: pinned cargo-fuzz build + 30-second decode fuzz success.

This is exact-head local/CI/fuzz evidence. It is not WAN evidence, a security approval, RC, freeze, release or production readiness.

## Review verdict

**CONTINUE_WITH_NARROW_E1A_CLOSURE — accept the four-responder staged migration and helper-level terminal-abandon direction. Do not reopen those solved seams. One small staged-primitive terminality/test package remains before E1A can close; then continue directly through E2/C1 rather than returning to checker-only churn.**

No administrator action is required. Do not spend VPS time yet: the remaining E1A issue is deterministic local accounting semantics. The active agent should continue on `work/e1a-staged-accounting-20260907`, consume the narrow closure package below, push, and immediately proceed to E2/C1 if green.

## Reviewer findings

### RSEC-001E1A-1 — CLOSED directionally at `3b684d5` / `14fe30d`

The previous HIGH finding about silent reservation drop is resolved for the real CLI staged helper:

- a post-begin non-complete `read_until_staged` result now calls `TcpInputReservation::abandon`;
- inner permit ownership becomes inactive;
- process staged ownership is abandoned/rejected when live;
- the logical process state is explicitly rejected regardless;
- oversize/truncation/deadline/I/O error therefore cannot return a reusable CLI ticket;
- charged attacker-caused input is not refunded.

`14fe30d` additionally proves complete/abandon are one-shot in the inner budget and proves a normal process abandon terminalizes the state and prevents later input.

Keep this behavior. Do not redesign it into generic RAII machinery unless a concrete test exposes another leak.

### RSEC-001E1A-2 — CLOSED at `3b684d5`

All four required TCP responder families now use the shared staged helper for negotiation and first Noise input. Ordinary/failover preserve their existing experiment/setup deadlines, negotiation binding, Noise/resume/readiness boundaries and post-auth framing. Periodic/multistream remain migrated from the prior slice.

Do not reopen post-auth data framing merely for symmetry.

### RSEC-001E1A-3 — MEDIUM/HIGH — new staged process primitives still do not terminalize *every* returned error structurally

The real CLI helper is fail-closed because `ListenerAdmission::{begin,extend}_tcp_input_record` explicitly rejects the process state on an outer failure and `read_staged_frame` abandons the reservation on every non-complete result. However, the reusable `ProcessPreauthAdmission` staged API itself is weaker than the already-accepted D1 contract:

- `extend_input_record` rejects the state for explicit per-record/work/limit failures, but `refresh_window(now_ms)?`, `live(...) ?`, source lookups and checked arithmetic can return `Err(SessionRejected)` before `reject(state_id)` runs;
- a backwards-time or checked-arithmetic failure can therefore return an error while leaving the permit logically active and, for some error classes, leave the state otherwise reusable by a direct caller;
- `complete_input_record` similarly returns live/clock errors without structurally consuming the permit/rejecting the state;
- inner `PreauthBudget::extend_input_record` also leaves its permit active on an error when used directly, although the current CLI composition subsequently abandons it.

This does **not** currently create an externally reachable responder bypass because the CLI composition terminalizes the ticket, so do not escalate it into a redesign. It is nevertheless a contract regression in the reusable admission primitive and should be fixed before E1A is declared closed.

**Required repair:** use the same pattern already used by `charge_input`/`charge_response`: a checked internal implementation plus a public terminalizing wrapper, or an equivalent one-shot state transition. On any staged begin/extend/complete failure associated with an existing logical record, make later reuse of that record/state impossible. Abandon should remain safe when the state is already rejected/expired; it need not resurrect `live()` merely to mark permit ownership terminal.

Minimum deterministic regressions:

- staged extend with backwards monotonic time -> error -> same permit/state cannot later extend/complete successfully;
- staged extend arithmetic/limit failure -> same permit/state cannot later succeed;
- staged complete clock/liveness failure -> permit cannot later become complete;
- direct inner staged extension failure cannot be retried as the same logical record if the API remains public in its current form.

Keep this a small primitive-semantics patch; no new numeric policy, wire bytes, Session/Carrier/ACK/Noise behavior or runtime dependency.

### RSEC-001E1A-4 — MEDIUM evidence gap — helper failure ordering needs explicit staged tests, but no new checker framework

Current exact-head CI/fuzz is green and the primitive one-shot tests are useful. The staged framed reader still lacks direct deterministic tests showing the event/order contract on its failure paths. Existing `framed.rs` tests mainly exercise the legacy non-staged reader.

Add a small staged-reader test matrix using the existing scripted reader/clock rather than creating another harness:

- successful fragmented header/body emits `Header -> Body -> Complete` exactly once;
- oversize length emits `Header` before the length rejection and does not allocate/read the body;
- truncated/deadline after accepted length emits `Header -> Body` and no `Complete`;
- zero-length body still produces one logical record and exactly one `Complete`;
- callback rejection returns before body allocation/read where applicable.

At the admission layer, add one regression where inner staged admission succeeds but the outer layer rejects (for example via an existing work/window limit): prove the process ticket is terminal and no response/enqueue/input success can follow. Exact equality of inner/outer terminal counters is **not** required for E1A if the implementation deliberately uses an `all-or-terminal` invariant; conservative partial charge is acceptable only because the ticket is terminal and cannot produce protocol evidence. Do not spend another day engineering rollback bookkeeping solely to make dead-state counters aesthetically equal.

This closes the previous cross-layer-fidelity concern as a safety question once the all-or-terminal regression exists.

## Evidence boundaries

- `IMPLEMENTATION_COMPLETE=true` remains a bounded research-baseline flag only.
- `CANONICAL_CORPUS_V1_FROZEN=true` remains corpus-specific only.
- `RELEASE_CANDIDATE=false`, `PRODUCTION_READY=false`, `FREEZE=false`, `RELEASED=false` remain required.
- A1 response-I/O deadline, B1 queue ownership/expiry and D1 ordinary terminal rejection remain accepted closed subfindings unless a concrete regression appears.
- `ccacec1` / `6974ff6` / `a6b0c33` / `3b684d5` / `14fe30d` are real staged-accounting/runtime/test progress. They remain deterministic/local evidence only.
- All four TCP responder families now stage pre-auth accounting before attacker-controlled length/body work. This is accepted implementation progress, not security approval.
- Exact `14fe30d` has green stable checks and green nightly decode fuzz smoke.
- Existing 16 KiB per-state reservation remains a memory ceiling, not a substitute for input/work accounting.
- Historical WAN/HY2/failover/periodic positive and negative evidence remains immutable at its exact commit boundary.
- Standing VPS authorization remains valid for future dependency-ready self-owned work; no new per-run permission is needed when the release matrix truthfully becomes `READY_LIVE`.
- Protected identity material, SSH private keys, credentials, private endpoint material and raw private diagnostics remain unread/untracked/uncommitted.

## Rolling Work Queue

This is a multi-hour pre-authorized queue. The coding agent owns ordinary local design choices and should propose/choose the smallest fail-closed shape inside current architecture. Finish a coherent closure package -> targeted/full gates -> commit -> push -> immediately continue. One commit, nominal hour, reviewer interval or proposal note is never a stop condition.

### A — Final E1A primitive terminality + staged-reader adversarial closure

**Status:** `READY_LOCAL`; immediate narrow correctness package.

**Goal / why now:** close the only remaining E1A semantic gap without reopening solved runtime migrations.

**Files/concepts:** staged permit methods in `crates/neko-crypto/src/lib.rs`; small staged-reader tests in `crates/neko-cli/src/framed.rs`; only minimal composition tests in `crates/neko-cli/src/preauth.rs` if needed.

**Protected invariants:** one frame == one input packet; bytes/work may stage but packet ownership does not; every staged-operation failure is terminal/non-revivable; attacker-caused reserved input is not refunded after accepted reservation; no success evidence after rejection; no new policy values/wire/runtime architecture.

**Behavior:** terminalize all staged primitive error returns; add the direct staged-reader event/failure matrix and one all-or-terminal cross-layer regression. Do not add a new checker/harness framework.

**Validation:** targeted tests, `scripts/check.sh`, `git diff --check`; CI already carries the required decode fuzz job, and rerun exact-head CI after push. Run local fuzz smoke if available, otherwise rely only on the exact-head CI fuzz result after push and report the local limitation truthfully.

**Commit/push:** one coherent implementation+tests package is preferred. Push exact head.

**Continue immediately to B:** yes, if targeted/full gates are green; do not wait for reviewer merely because exact-head CI is pending unless B depends on the final security conclusion.

### B — E2 semantic responder inventory/evidence-barrier closure

**Status:** `PREAUTHORIZED_AFTER_A`.

Reconcile `docs/preauth-responder-inventory.v1.json` and its checker against the now-repaired runtime. Reuse only semantically valid parts of historical `d271a99`; do not preserve stale anchors.

Required assertions:

- every TCP responder anchors the shared staged receive before negotiation/Noise parse;
- every UDP responder anchors bounded raw receive -> accounting -> parse;
- pending UDP ownership proves queue-reserve-before-store and exactly-once terminal dequeue/cancel/expiry invalidation;
- malformed/rejection/timeout/I/O paths cannot reach auth/readiness/Session/Delivery/PathValidated/ACK/authz-equivalent success anchors;
- externally reachable responder set remains explicit.

Fix real uncovered call-site seams; do not invent checker layers when the existing semantic inventory can express the rule.

**Gate:** semantic checker + full repository gate; commit/push.

**Continue immediately to C:** yes.

### C — C1 explicit carrier/source projection

**Status:** `PREAUTHORIZED_AFTER_B`.

Add an explicit bounded carrier discriminator so current TCP and UDP pre-auth source domains cannot accidentally alias. Preserve family/address/port representation without textual/raw logging. Add deterministic non-collision tests across carrier/family/address/port. Do not add terminal-source TTL/LRU/history limits.

**Gate:** targeted/full tests; commit/push.

**Continue immediately to D:** yes.

### D — C2 terminal-source persistence policy checkpoint

**Status:** `ADR_CHECKPOINT_AFTER_C`.

Re-read D019 against the exact implementation. If literal no-reset-on-retry/reconnect/carrier-change semantics cannot coexist with bounded source-accounting memory without a new retention policy, write a compact ADR amendment request with exact conflicting clauses, attacker/resource rationale, feasible policy shapes **without convenience numbers**, and required tests/evidence.

Do not invent TTL/LRU/history numeric policy. If this becomes a genuine maintainer/reviewer policy wait, stop only this lane and continue F/G/H independent work below.

### E — Full D019 adversarial matrix + exact-tree security evidence closure

**Status:** `PREAUTHORIZED_AFTER_D_RESOLVED`.

Complete the remaining source/global concurrency, input/work one-second windows, memory, queue, response/3x amplification, idle/lifetime/100 ms deadline, terminal non-revival, resolved retry/reconnect/carrier transition, cleanup and no-success-evidence matrix. Do not duplicate the E1A staged tests just added.

Push the exact repair head, require green exact-head stable+fuzz CI for the security conclusion, then reconcile:

- `docs/reviews/resource-abuse-evidence-2026-09-04.md`;
- `docs/release-security-review-packet.md`;
- `docs/status.md`;
- release closure/navigation.

RSEC-001 may close as an implementation finding only if the exact tree supports it. Independent/two-person release review remains a separate gate. Never promote RC/production/freeze/release automatically.

**Continue immediately to F if no new HIGH/BLOCKER:** yes.

### F — Compatibility/freeze-boundary closure

**Status:** `READY_LOCAL_AFTER_E`; also safe fallback during genuine D policy wait.

Audit corpus-v1 content freeze vs global protocol non-freeze, current/current negotiation, unsupported/future rejection, downgrade/transcript binding into Noise, resume/version binding and replay boundaries. Add a regression only for a concrete defect; do not reopen frozen corpus bytes absent correctness evidence.

**Continue immediately to G:** yes.

### G — Package/operator + evidence provenance integrity

**Status:** `READY_LOCAL_AFTER_F`; safe fallback during genuine D policy wait.

Verify existing x86_64 package/build identity, dedicated-path install/readiness/smoke/upgrade/rollback, shutdown/listener/temp cleanup, canonical Git-blob/checksum manifests and exact-head CI references. Do not read protected identity material. Do not rerun already-sufficient VPS/package evidence merely for freshness; fix concrete defects only.

**Continue immediately to H:** yes.

### H — Reclassify release opportunities and deliberately return to runtime output

**Status:** `READY_LOCAL_AFTER_G`.

Re-evaluate every release/evidence row:

- bounded question already answered -> `ALREADY_SUFFICIENT_FOR_BOUNDED_QUESTION`;
- executable specific missing assertion -> `OPEN_READY` with exact `evidence_needed`, `next_action`, `requires`, `execution_scope`;
- otherwise exact implementation/environment/governance/review blocker.

Then deliberately end the audit-only loop. If no new HIGH/BLOCKER remains, choose one highest-value `BLOCKED_IMPLEMENTATION` runtime seam that can become locally executable without maintainer-level design: NAT/source-endpoint change, migration-back, live key update or live PMTUD only where current architecture supports a bounded honest path. Treat runtime implementation + local validation + instrumentation as one closure package.

If that produces a genuine `READY_LIVE` missing question, continue to I rather than waiting for a reviewer interval.

### I — One changed-hypothesis bounded VPS evidence closure when truthful

**Status:** `PREAUTHORIZED_AFTER_H_WHEN_READY_LIVE_EXISTS`.

Under `docs/standing-vps-lab-authorization.md`, execute one self-owned VPS row only when a declared missing release question is dependency-ready. Use the smallest profile that answers the question, preserve exact commit/binary/parameters/events/resource observations and cleanup, and retain positive **or negative** evidence. Do not repeat an unchanged historical failure.

After the run, reconcile the release matrix/status in the same evidence boundary. Do not turn one bounded sample into a performance/reliability/production claim.

## 24–48 hour output check

This review contains real progress: the coding agent closed the prior two HIGH runtime seams, migrated all four TCP responder families, added terminal staged ownership and produced green exact-head stable+fuzz CI. The next package should finish E1A primitive semantics/tests, then move forward through E2/C1 rather than generate more reviewer-only scaffolding.

After D019 is closed or isolated behind a genuine policy checkpoint, the queue intentionally returns to one real runtime seam and one truthful VPS evidence question so the project produces a new runnable capability or real-network conclusion within the rental window.

## Completion gates

E1A closes when all are true:

- every real TCP pre-auth responder uses the shared staged path for negotiation/first Noise input — **satisfied at `3b684d5`**;
- post-begin non-complete helper outcomes terminalize the CLI logical ticket without refund — **satisfied directionally at `3b684d5`**;
- staged primitive errors are structurally non-revivable for direct callers — **open**;
- one frame remains one packet and cumulative per-record work is bounded across stages — implementation exists; direct adversarial evidence still to finish;
- header accounting precedes attacker length interpretation and body reservation precedes allocation/read — **satisfied in staged helper**;
- cross-layer failure follows a reviewed `all-or-terminal` or atomic invariant and cannot produce success evidence — one direct regression still required;
- staged-reader oversize/truncation/deadline/callback ordering tests exist — **open**;
- full gate/fuzz/exact-head CI are green — exact `14fe30d` green now; rerun after final closure patch.

Broader RSEC-001/D019 closure additionally requires E2/C1/C2/full adversarial/evidence review, with governance flags unchanged.

## Do not expand into

- another staged-accounting checker/harness framework before A is closed;
- protocol/wire/Noise/Session/Carrier redesign for this local accounting seam;
- new numeric D019/source-retention limits without reviewed ADR work;
- public/production listener deployment;
- VPS/load testing as a substitute for deterministic security accounting;
- unchanged HY2/repeated-failover retries;
- speculative FEC/0-RTT/striping/multipath/exotic carriers;
- reading, hashing, copying, modifying or committing protected identity/SSH-key/credential/private-endpoint material;
- RC/freeze/release/production promotion.

## Questions requiring maintainer decision

None at this review point.

The current E1A remainder is ordinary local correctness/test work. The later terminal-source persistence checkpoint may require a real policy decision, but only after C1 and the exact responder semantics are complete.