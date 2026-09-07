# Nekomusume ChatGPT Handoff

Checked at: 2026-09-07 15:57 Asia/Shanghai
Repository main HEAD reviewed: `f8898d3b0725cae032b36b2d5fe8e3a3a177941a`
Previous checked implementation HEAD: `d271a99a2ab26abbcb146c411ba0fde697395abe`
Current execution branch: `work/e1a-staged-accounting-20260907` at exact `a6b0c3327c34a068fd66bc7dc44d63b959b9536b`
Historical partial-E2 branch retained: `work/continue-20260904` at `d271a99a2ab26abbcb146c411ba0fde697395abe`

## What changed

The external coding agent finally resumed real implementation on the dedicated E1A branch. Three implementation commits are now ahead of current reviewer-only `main`:

- `ccacec1` — adds staged one-logical-record accounting primitives to inner `PreauthBudget`, process/source/global admission, and the CLI `ListenerAdmission` composition layer;
- `6974ff6` — adds a staged framed-reader contract that charges the fixed four-byte header before interpreting attacker-controlled length and reserves declared body bytes/work before allocation;
- `a6b0c33` — migrates **periodic TCP** and **multistream TCP** pre-auth negotiation/Noise reads to the staged helper and updates their inventory anchors.

Exact `a6b0c3327c34a068fd66bc7dc44d63b959b9536b` Rust CI run `34094391386` concluded `success`. This is exact-head repository CI evidence, not E1A/security closure.

This is meaningful runtime/accounting progress and resolves the previous executor-inactivity condition. The proposal protocol worked: the agent introduced the needed permit/state-machine shape without a maintainer design decision or core Session/Carrier/ACK/crypto/wire change.

However, E1A is still incomplete and currently has two concrete HIGH contract gaps plus one accounting-fidelity gap. Do not advance to E2 closure until these are repaired and covered by deterministic tests.

## Review verdict

**CONTINUE_WITH_REQUIRED_FIXES — accept the staged accounting direction and exact-head green CI, but keep E1A open. Finish all four responder migrations and make incomplete/malformed staged records terminal before E2.**

No administrator action is required. Do not spend VPS time on this deterministic security lane yet. The agent should continue on `work/e1a-staged-accounting-20260907`, test/commit/push coherent repairs, then consume E2/C1 immediately when E1A is actually closed.

## Reviewer findings

### RSEC-001E1A-1 — HIGH — incomplete/malformed staged record can drop its reservation without terminalizing the logical pre-auth state

`read_staged_frame` keeps its `TcpInputReservation` only as a local `Option`. `FrameStage::Header` begins the logical record and `FrameStage::Body` extends/reserves it, but the helper only consumes the reservation on `FrameStage::Complete`.

After a successful begin/extend, these paths return an error without an explicit abandon/reject transition for the staged reservation:

- declared length exceeds `max_frame_len` after the header was already charged;
- EOF/truncation after header or body reservation;
- deadline/timeout after a partial reserved record;
- ordinary I/O error after the record began.

The local reservation then drops silently. Current fail-fast CLI callers may terminate their process, but the reusable helper/ticket contract itself does not make D019's malformed/timed-out operation terminal. E1A requires structural fail-closed behavior, not reliance on every caller exiting forever.

**Required repair:** give staged input ownership an explicit one-shot terminal path. Acceptable shapes include an active reservation guard plus `abandon_tcp_input_record`, or a helper-owned state machine that calls `reject_state` on every post-begin non-complete outcome. Charged bytes/packet/work must remain charged; do not refund attacker-caused reservation on truncation/timeout/oversize. Double complete/abandon must fail harmlessly and must not reopen the ticket.

Minimum regressions: oversize-after-header, truncated body after reservation, deadline after reservation, I/O error after reservation, and same logical ticket cannot subsequently begin/charge/respond/enqueue.

### RSEC-001E1A-2 — HIGH — only two of the required four TCP responder surfaces are migrated

`a6b0c33` correctly migrates periodic and multistream TCP handshakes. The execution-branch `main.rs` still contains the known complete-frame-read -> later-charge order in two real responder families:

- ordinary TCP probe/server: `read_frame(&mut s, ...)` precedes `preauth.charge_input(...)` for negotiation and first Noise message;
- failover TCP responder: `read_frame(&mut stream, ...)` precedes `preauth.charge_input(...)` for negotiation and first Noise message.

The original E1A closure package explicitly required ordinary TCP + periodic TCP + multistream TCP + failover TCP in the same semantic migration. Green CI does not make a half-migration complete.

**Required repair:** migrate ordinary TCP and failover TCP to the same shared staged helper. Preserve their existing setup/experiment deadlines, negotiation binding, Noise semantics, response deadline, readiness/resume behavior and evidence boundaries. Do not redesign post-auth data framing.

### RSEC-001E1A-3 — MEDIUM — staged cross-layer admission can leave inner/outer counters divergent on a later layer failure

`ListenerAdmission::begin_tcp_input_record` charges the inner `PreauthBudget` before the process/source/global layer. `extend_tcp_input_record` similarly mutates the inner staged byte counter before the outer extension. If the outer operation rejects, the logical state is rejected, so this is conservative rather than an admission bypass; however, inner and outer accounting can still disagree for the terminal state.

Previous D019 work deliberately repaired analogous cross-layer charge rollback/atomicity. The staged path should not quietly weaken that invariant.

**Required repair:** either make staged cross-layer admission atomic before protected work begins, or add a narrowly scoped rollback for cross-layer setup failure where no attacker-protected work/allocation/read has yet occurred. Do **not** refund successfully reserved attacker-caused body budget on later EOF/timeout/truncation. Add a deterministic outer-reject-after-inner-success regression proving the chosen invariant.

### RSEC-001E1A-4 — MEDIUM evidence gap — exact-head CI is green but the new staged lifecycle lacks the required adversarial tests

The branch adds the primitives/helper and two migrations, but the required E1A matrix is not yet demonstrated: one frame == one packet, extension adds no packet, cumulative per-record work exact/max+1, oversize/truncation/timeout remains charged and terminal, double extend/complete rejection, cross-layer exhaustion, and anti-amplification packet ownership.

Do not treat existing framed fragmentation tests or exact-head CI as equivalent to these staged-accounting boundary tests.

## Evidence boundaries

- `IMPLEMENTATION_COMPLETE=true` remains a bounded research-baseline flag only.
- `CANONICAL_CORPUS_V1_FROZEN=true` remains corpus-specific only.
- `RELEASE_CANDIDATE=false`, `PRODUCTION_READY=false`, `FREEZE=false`, `RELEASED=false` remain required.
- A1 absolute response-I/O deadline, B1 queue ownership/expiry and D1 terminal rejection remain accepted closed subfindings unless a concrete regression appears.
- `ccacec1`/`6974ff6`/`a6b0c33` are real implementation changes with exact-head green CI. They are local deterministic implementation evidence only; no WAN/VPS/performance evidence changed.
- Header charge now precedes attacker-controlled length interpretation in the staged helper, and body reservation precedes allocation there. This is accepted directionally, not full E1A closure.
- Periodic and multistream TCP use the staged helper; ordinary and failover TCP do not yet.
- Existing 16 KiB per-state reservation remains a memory ceiling, not a substitute for input/work accounting.
- Historical WAN/HY2/failover/periodic positive and negative evidence remains immutable at its exact commit boundary.
- Standing VPS authorization remains valid for future dependency-ready self-owned work; deterministic D019 correctness must not be substituted with load/WAN evidence.
- Protected identity, credentials, private endpoints and raw private diagnostics remain unread/untracked/uncommitted.

## Rolling Work Queue

This is a multi-hour pre-authorized queue. The coding agent owns ordinary local design choices and may propose/choose the smallest fail-closed API shape inside current architecture. Finish a coherent closure package -> targeted/full gates -> commit -> push -> immediately continue. One commit, nominal hour, reviewer interval or proposal note is not a stop condition.

### A — Finish E1A staged-record terminal ownership

**Status:** `READY_LOCAL`; highest priority correctness repair.

**Goal / why now:** a begun staged record must end exactly once as completed or terminally abandoned; malformed/truncated/timed-out input must not leave a reusable logical ticket.

**Files/concepts:** `crates/neko-cli/src/preauth.rs`, `crates/neko-cli/src/framed.rs`, staged permit types in `crates/neko-crypto/src/lib.rs`.

**Protected invariants:** no refund of attacker-caused reserved input; one TCP frame remains one packet; rejection cannot become auth/readiness/Session/PathValidated/Delivery/ACK evidence; no new policy values or wire change.

**Behavior:** add explicit abandonment/guard semantics for every post-begin non-complete result; make complete/abandon one-shot; preserve charged budgets on truncation/timeout/oversize/I/O failure.

**Validation:** deterministic oversize/truncated/deadline/I/O-error + non-revival + double-terminal tests; targeted tests, `scripts/check.sh`, `git diff --check`, fuzz-smoke because untrusted framing behavior changed (or truthful `NOT_RUN_ENVIRONMENT`).

**Commit/push:** coherent runtime + tests; push exact head.

**Continue immediately to B:** yes.

### B — Complete all four TCP responder migrations

**Status:** `PREAUTHORIZED_AFTER_A`.

**Goal:** remove the remaining ordinary/failover complete-frame-read -> later-charge seams.

**Files/concepts:** ordinary TCP server/probe and failover TCP responder in `crates/neko-cli/src/main.rs`; shared staged helper; inventory anchors.

**Protected invariants:** existing negotiation/Noise/Session/resume/readiness semantics and deadlines unchanged; only pre-auth input accounting/receive ordering changes.

**Behavior:** migrate negotiation + first Noise record on ordinary TCP and failover TCP; do not modify post-auth data path merely for symmetry.

**Validation:** process/loopback tests for both responder families plus full gate/fuzz-smoke.

**Commit/push:** exact coherent migration with tests.

**Continue immediately to C:** yes.

### C — Restore staged cross-layer accounting fidelity and finish E1A adversarial matrix

**Status:** `PREAUTHORIZED_AFTER_B`.

Resolve the inner/outer partial-charge case without weakening conservative attacker-budget retention. Add the missing matrix: one record==one packet, extension no extra packet, cumulative work exact/max+1, source/global/window exhaustion, cross-layer failure, zero body, fragmentation, overflow/backwards/expiry, anti-amplification and no-success-evidence on rejection.

If a narrower API shape than the reviewer suggestion better preserves atomicity, the agent may propose and implement it under the proposal protocol without waiting.

**Gate:** targeted tests + full repository gate + fuzz smoke; commit/push.

**Continue immediately to D:** yes.

### D — E2 semantic responder inventory/evidence-barrier closure

**Status:** `PREAUTHORIZED_AFTER_C`.

Reconcile the inventory against the repaired runtime. Reuse semantically valid parts of historical `d271a99`, but do not preserve stale anchors.

Required: every TCP responder anchors staged charge before negotiation/Noise parse; every UDP responder anchors bounded raw receive -> charge -> parse; pending ownership proves queue-reserve-before-store and exactly-once terminal cleanup; rejection/malformed/timeout/I/O paths cannot reach success-evidence anchors; externally reachable responder set remains explicit.

**Gate:** semantic checker + full repository gate; commit/push.

**Continue immediately to E:** yes.

### E — C1 explicit carrier/source projection

**Status:** `PREAUTHORIZED_AFTER_D`.

Add an explicit bounded carrier discriminator so current TCP and UDP pre-auth source domains cannot accidentally alias. Preserve family/address/port representation without textual/raw logging and add deterministic non-collision tests. Do not invent terminal-source retention TTL/LRU/history limits.

**Continue immediately to F:** yes.

### F — C2 terminal-source persistence policy checkpoint

**Status:** `ADR_CHECKPOINT_AFTER_E`.

Re-read D019 against exact implementation. If literal no-reset-on-retry/reconnect/carrier-change semantics cannot coexist with bounded source-accounting memory without a new retention policy, write a compact ADR amendment request containing exact conflicting clauses, attacker/resource rationale, feasible policy shapes without convenience numbers, and required tests/evidence.

Do not invent numeric policy. If this becomes genuine external wait, stop only this lane and continue H/I/J-equivalent independent work.

### G — Full D019 adversarial matrix + exact-tree security closure

**Status:** `PREAUTHORIZED_AFTER_F_RESOLVED`.

Close all remaining concurrency, staged input/work, one-second windows, memory, queue, response/3x anti-amplification, idle/lifetime/100 ms deadline, terminal non-revival, resolved retry/reconnect/carrier transition, cleanup and evidence-barrier cases. Full gate, push, wait only for exact-head green CI required for the security conclusion, then reconcile resource-abuse review, release-security packet and `docs/status.md` without promoting RC/production/freeze/release.

**Continue immediately to H if no new HIGH/BLOCKER:** yes.

### H — Compatibility/freeze + package/operator/provenance closure

**Status:** `READY_LOCAL_AFTER_G`; also safe fallback during genuine F policy wait.

Audit corpus-v1 freeze vs protocol non-freeze, negotiation/downgrade/transcript/resume/replay boundaries, then verify existing package install/readiness/upgrade/rollback, cleanup and evidence manifests without reading protected identity material. Fix only concrete defects; do not rerun sufficient VPS/package evidence for freshness.

**Continue immediately to I:** yes.

### I — Reclassify release opportunities and return to runtime/milestone output

**Status:** `READY_LOCAL_AFTER_H`.

Re-evaluate every release/evidence row: answered bounded question -> `ALREADY_SUFFICIENT_FOR_BOUNDED_QUESTION`; executable missing assertion -> `OPEN_READY` with exact evidence/action/dependencies/scope; otherwise exact blocker.

Then deliberately end the audit-only loop. If no new HIGH/BLOCKER remains, select one highest-value `BLOCKED_IMPLEMENTATION` runtime seam that can become locally executable without maintainer-level design: NAT/source endpoint change, migration-back, live key update or live PMTUD only where current architecture supports a bounded path. Treat implementation + local validation + instrumentation as one closure package.

If exact-head green makes one declared missing question genuinely `READY_LIVE`, continue immediately to one changed-hypothesis bounded self-owned VPS row under standing authorization, retain positive or negative evidence with provenance/cleanup, then reconcile matrix/status. Do not repeat unchanged failures.

## 24–48 hour output check

This review finally contains real runtime/accounting progress rather than reviewer-only churn: staged accounting exists and two real responder families have migrated. Keep that momentum. The next checkpoint should close the two concrete HIGH E1A gaps rather than create more planning/checker-only commits.

After D019/security closure (or a genuine isolated policy wait), the queue deliberately returns to one concrete runtime seam and, when truthful, one bounded VPS evidence question so the project produces a new runnable capability or real evidence conclusion.

## Completion gates

E1A closes only when all are true:

- every begun staged record completes or terminally abandons exactly once;
- malformed/oversize/truncated/timeout/I/O failure after begin cannot reuse the logical ticket;
- one frame remains one packet under staged accounting;
- header charge precedes length interpretation and body reservation precedes allocation/read;
- cumulative per-record work is bounded across stages;
- cross-layer staged accounting has a reviewed atomic/conservative failure invariant;
- ordinary, periodic, multistream and failover TCP all use the shared staged path for pre-auth negotiation/first Noise input;
- required deterministic adversarial tests exist and fuzz/full gates pass;
- exact-head CI is green.

Broader RSEC-001/D019 closure additionally requires E2/C1/C2/full adversarial/evidence review, with governance flags unchanged.

## Do not expand into

- new reviewer-only branch provisioning or checker layers before current E1A code is repaired;
- protocol/wire/Noise/Session/Carrier redesign for staged accounting;
- new numeric D019/security/source-retention limits without reviewed ADR work;
- public or production listener deployment;
- VPS/load tests as a substitute for deterministic security accounting;
- renewed HY2 work without a changed missing-question hypothesis;
- speculative FEC/0-RTT/striping/multipath/exotic carriers;
- reading, hashing, copying, modifying or committing protected identity/secrets/private endpoint material;
- RC/freeze/release/production promotion.

## Questions requiring maintainer decision

None at this review point.

The current E1A defects are ordinary local correctness/design work within existing authorization and architecture. The external agent should repair them autonomously. A maintainer decision is needed only if the later terminal-source persistence checkpoint genuinely requires a new security policy value or core semantic choice.
