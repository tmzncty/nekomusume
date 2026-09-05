# Nekomusume ChatGPT Handoff

Checked at: 2026-09-05 12:00 Asia/Shanghai
Repository HEAD reviewed: `b775f1e241ba58785f80708ba41ef51856a5e259`
Previous checked implementation HEAD: `61d69bdd58b071f7d9c3e1ec99602cebbf032787`
Previous reviewer handoff commit: `9fe35727255245d43b51638fa3bc1d91d1ddbe5f`

## What changed

One coding-agent commit landed after the previous reviewer handoff:

- `b775f1e` — **narrow D019 response-I/O deadline repair; no wire/Noise/Session/Carrier/release semantic change.** It replaces the previous single socket timeout with a `BoundedWrite` path that recomputes the remaining time from the same absolute response-permit deadline before every potentially blocking TCP partial write and flush; zero remaining budget now fails closed instead of gaining an extra millisecond; successful TCP/UDP pre-auth response helpers preserve and restore the preexisting socket write timeout. It adds deterministic partial-write boundary coverage and socket-level timeout-restoration regressions.

Both `main` and `work/continue-20260904` point at exact `b775f1e241ba58785f80708ba41ef51856a5e259` at review time.

### Exact-head CI

Exact `b775f1e` is independently green on both refs:

- main Rust CI run `33941126349` — `success`;
- work-branch Rust CI run `33941131928` — `success`.

This is repository CI evidence, not a security approval.

## Review verdict

**CONTINUE_WITH_REQUIRED_FIXES — A1r is accepted as closed at exact green `b775f1e`; continue immediately through B1v -> D1 -> E -> C1/C2 -> F/G/H/I/J without another reviewer wait. RSEC-001 as a whole remains open because queue ownership verification, terminal rejection, charge-order inventory and the source-retention policy conflict are not yet closed.**

No administrator action is required now. Do not spend VPS time on this deterministic security-accounting lane while D1/C2 remain unresolved.

## Reviewer findings

### RSEC-001A1 — CLOSED at `b775f1e`

The three concrete defects from the previous review are now addressed within the stated D019 process-side boundary:

1. **One absolute deadline across partial TCP writes.** `write_all_until` recomputes `remaining_budget(now, deadline)` before every `write_with_budget`; the `TcpStream` implementation applies that remaining duration to the blocking syscall. Partial progress therefore does not refresh a new 100 ms window.
2. **Zero budget fails closed.** `remaining_budget` rejects `remaining_ms == 0`; the previous `max(1)` extension is gone.
3. **Reusable socket timeout state is restored on success.** TCP and UDP helpers read the prior timeout, apply the bounded pre-auth timeout, restore the prior value, then complete the response permit. The new regression exercises a preexisting two-second timeout for both transports.

The deterministic partial-writer test now has an inside-budget completion at 99 ms and a refusal when a later blocking operation would begin at the 100 ms boundary. Exact-head CI is green.

Evidence boundary: this proves the process-side bounded send-attempt contract used by current responder call sites. It does **not** prove remote receipt by 100 ms or provider/kernel delivery latency, and no such claim should be added.

Do not reopen this into a generic asynchronous I/O framework unless a concrete failing responder or platform behavior appears.

### RSEC-001B1 — implementation direction accepted; verification remains READY

The existing one-owner queue/expiry direction remains valid. Finish the lifecycle/boundary matrix before calling queue ownership evidence-complete. No redesign is justified by `b775f1e`.

### RSEC-001D1 — HIGH — terminal rejection still incomplete across inner + outer accounting

Current `ListenerAdmission::charge_input` still performs `ticket.budget.charge_input(...)` and returns immediately on an inner `PreauthBudget` rejection before mechanically terminalizing the process/logical state. `charge_response` has the same shape for inner response/anti-amplification rejection. This means D019's “rejected work cannot later become successful merely because a window refilled or the caller retried” contract is still not structurally enforced across every rejection layer.

State-associated checked arithmetic/deadline/clock failures must also terminate the same logical pre-auth ownership rather than merely returning an error when a reusable ticket remains live.

Do not weaken the inner `PreauthBudget`; compose terminal rejection across both layers.

### RSEC-001C — ADR checkpoint remains isolated

The current source projection still lacks an explicit carrier discriminator and terminal source usage can disappear after the last live state. D019 simultaneously says source counters are state-lifetime scoped and must not reset through retry/reconnect/carrier change/identity change/error. Keeping terminal sources forever would create an unbounded source-accounting map, while the reviewed ADR defines no tombstone retention TTL/history ceiling/eviction rule.

Do not invent a numeric retention policy. Complete the concrete responder/charge-order inventory first, make the carrier/source projection explicit, then re-read the reviewed policy. If bounded storage and no-reset semantics still conflict, produce the compact ADR amendment request and stop only that policy-dependent lane. H/I remain independent fallback work.

## Evidence boundaries

- `IMPLEMENTATION_COMPLETE=true` remains a bounded research-baseline flag only.
- `CANONICAL_CORPUS_V1_FROZEN=true` remains corpus-specific only.
- `RELEASE_CANDIDATE=false`, `PRODUCTION_READY=false`, `FREEZE=false`, `RELEASED=false` remain required.
- Exact `b775f1e` is real implementation with deterministic/socket-level tests and exact-head green CI. It closes the narrow A1r finding only; it does not close D019/RSEC-001 globally.
- Existing inner `PreauthBudget` remains a useful stricter per-state anti-amplification/input-response bound and must not be removed to simplify outer accounting.
- Process admission does not claim control over kernel SYN backlog, provider NAT state or resources outside the process.
- No VPS/load run substitutes for deterministic D019 counter, ownership, timeout and evidence-barrier correctness.
- Standing VPS authorization remains valid for genuinely dependency-ready self-owned TCP/UDP work, but the current correctness/security lane has precedence.
- Protected identity material, credentials, private endpoint material and raw private diagnostics remain unread/untracked/uncommitted.

## Rolling Work Queue

This is a rolling multi-hour queue. Finish one coherent slice -> targeted/full gates -> commit -> push -> immediately consume the next dependency-satisfied slice. Do not stop for a reviewer interval. Only a new HIGH/BLOCKER that invalidates downstream work, a genuine ADR/core-architecture conflict, action beyond authorization, production impact, missing credentials/third-party authority, repository breakage, runtime/tool-budget termination or real queue exhaustion is a stop condition.

### B1v — Finish queue-expiry ownership verification

**Status:** `READY_LOCAL`; immediate next slice.

Do not redesign the accepted queue/state ownership shape. Close the deterministic matrix:

- exact idle expiry at 1,000 ms while five-second lifetime has not elapsed;
- exact lifetime expiry at 5,000 ms;
- successful authentication/promotion removes queue ownership exactly once;
- cancellation/replacement removes queue ownership exactly once;
- source queue max/max+1;
- global queue max/max+1 across distinct source projections;
- ordinary expiry keeps the bounded responder/server loop alive rather than turning normal unauthenticated timeout into process failure;
- queue/state/memory counters return to the expected values exactly once on every terminal path;
- stale application-level pending ownership cannot successfully dequeue or continue after process expiry.

Use injectable/deterministic time; no wall-clock soak. Run targeted tests, full repository gate and `git diff --check`, commit, push.

**Continue immediately to D1:** yes.

### D1 — Make every rejection class terminal across inner + outer accounting

**Status:** `PREAUTHORIZED_AFTER_B1v`; security priority.

Make terminal rejection structural at the logical ticket/controller boundary without changing D019 numeric ceilings, wire bytes, Noise, Session or Carrier semantics.

Required behavior/tests:

- inner per-state input rejection -> process/logical ticket terminal; no later input/queue/response operation on the same logical state;
- inner response / anti-amplification rejection -> terminal; no later send after additional input or window rollover;
- global input/work rejection -> no revival after one-second window rollover;
- source/global response rejection -> no later response;
- source/global queue rejection -> no later enqueue on that rejected logical ownership;
- response-deadline/I/O failure -> no later success;
- state-associated checked-add / deadline-construction / backwards-or-unmeasurable-clock failure -> terminal;
- cross-layer rollback remains truthful: a failed outer charge may roll back an inner accounting charge where appropriate, but rollback must never resurrect the logical state;
- cleanup/release is one-shot and bounded; rejection cannot reopen a source domain contrary to the later C policy.

Prefer a small `reject/terminalize` primitive or an API shape that consumes/poisons the ticket on error over scattered caller discipline. Run targeted/full gate, commit, push.

**Continue immediately to E:** yes.

### E — Audit and machine-check charge ordering across every real responder

**Status:** `PREAUTHORIZED_AFTER_D1`.

Create or maintain a machine-checkable/static inventory for every externally reachable pre-auth responder path, including ordinary TCP/UDP probe, periodic, multistream and failover TCP/UDP paths.

For each path prove/order:

1. typed carrier/source projection + state admission;
2. input byte/packet charge before parse;
3. parser/work reservation before protected work;
4. state memory reservation before owned allocation;
5. queue reservation before application-level pending ownership;
6. response charge + bounded actual I/O before send;
7. terminal rejection/evidence barrier;
8. exactly-once cleanup.

The current conservative `64` / `4096` work reservations may remain only if they dominate the bounded parser work they protect; describe them as accounting units, never CPU-cycle measurements. Fix concrete uncovered seams only. Add a guard/test so a newly externally reachable responder cannot silently omit the admission path.

Run full gate, commit, push.

**Continue immediately to C1:** yes.

### C1 — Make carrier/source projection explicit and bounded

**Status:** `PREAUTHORIZED_AFTER_E`.

Implement only the noncontroversial source-key portion:

- explicit bounded carrier discriminator at least distinguishing current TCP and UDP pre-auth source domains;
- family/address/port remain represented without textual/raw logging;
- deterministic non-collision tests across carrier/family/address/port combinations;
- one bounded unknown/unusable-source bucket only if a current real call site actually needs it;
- no new terminal-source retention duration, LRU size, history count or eviction parameter.

Run targeted/full gate, commit, push.

**Continue immediately to C2:** yes.

### C2 — Resolve terminal-source persistence semantics or produce ADR amendment request

**Status:** `ADR_CHECKPOINT_AFTER_C1`.

Re-read D019 and adjacent reviewed decisions against the now-concrete inventory.

Do **not** retain arbitrary terminal sources forever. Do **not** invent TTL/LRU/history counts. If the reviewed text still cannot simultaneously satisfy:

- source counters do not reset through retry/reconnect/carrier transition/identity change/error; and
- source-accounting memory remains bounded,

write a compact ADR amendment request containing:

- exact conflicting clauses;
- attacker/resource rationale;
- current implementation consequences;
- feasible policy shapes **without convenience numbers**;
- tests/evidence each policy shape would require;
- what remains safe to continue independently.

Stop only this policy-dependent lane if reviewer/maintainer choice is genuinely required. Do not falsely mark D019 complete.

**If C2 becomes external-wait:** continue H then I; J may perform local reclassification but must not claim D019 closure.

### F — Complete full D019 adversarial/evidence-barrier matrix

**Status:** `PREAUTHORIZED_AFTER_C2_RESOLVED`.

Cover at minimum:

- source/global concurrency max/max+1;
- resolved source-lifetime input bytes/packets/work semantics;
- global one-second input/work/response windows;
- per-packet work ceiling;
- state/global memory;
- source/global pending queue;
- source/global response + inner 3x anti-amplification;
- idle 1 s / lifetime 5 s / response-send 100 ms using deterministic time/I/O controls;
- arithmetic/clock overflow and backwards time;
- terminal non-revival;
- retry/reconnect/carrier-transition persistence under resolved C semantics;
- cancellation/timeout/double cleanup;
- no Session/PathValidated/Delivery/ACK/readiness/authz-equivalent evidence on rejection;
- secret-safe bounded diagnostics.

Do not substitute VPS/load tests for deterministic accounting correctness. Full gate, commit, push. Exact repair-head CI must be green before G security closure.

**Continue immediately to G after exact-head CI green:** yes.

### G — Fresh exact-tree D019/security evidence review

**Status:** `PREAUTHORIZED_AFTER_F`.

Independently re-read the exact implementation and tests, then reconcile:

- `docs/reviews/resource-abuse-evidence-2026-09-04.md`;
- `docs/release-security-review-packet.md`;
- `docs/status.md`;
- release closure/navigation records.

RSEC-001 may close as an implementation finding only when B1v/D1/E/C1/C2/F are actually satisfied and exact-head CI is green. Independent external/two-person security review remains a separate release gate. Never promote RC/production/freeze/release automatically.

**Continue immediately to H if no new HIGH/BLOCKER:** yes.

### H — Compatibility / freeze-boundary review

**Status:** `READY_LOCAL_AFTER_G`; also safe fallback if C2 is externally waiting.

Audit corpus-v1 content-addressed freeze vs global protocol non-freeze, current/current negotiation, unsupported/future rejection, downgrade/transcript binding into Noise, resume/version binding, replay boundary and stale wording implying corpus freeze == protocol/release freeze.

Add a regression only for a concrete defect. Do not reopen frozen corpus bytes without correctness evidence.

**Continue immediately to I:** yes.

### I — Package/operator and evidence-provenance integrity review

**Status:** `READY_LOCAL_AFTER_H`; safe independent fallback if C2 is externally waiting.

Verify existing bounded evidence for x86_64 package/build identity, install/readiness/smoke/upgrade/rollback, retained external state without reading protected identity material, shutdown/listener/temp cleanup, canonical Git-blob/checksum manifests, exact-head CI references and stale release-packet links/hashes.

Do not rerun already-sufficient VPS/package work merely for freshness. Fix only concrete defects.

**Continue immediately to J:** yes.

### J — Reclassify release opportunities and reconsider VPS

**Status:** `READY_LOCAL_AFTER_I`; live execution depends on truthful dependency status.

Re-evaluate every remaining release/evidence row against actual current evidence:

- bounded question already answered -> `ALREADY_SUFFICIENT_FOR_BOUNDED_QUESTION`;
- executable specific missing assertion with dependencies satisfied -> `OPEN_READY` with exact `evidence_needed`, `next_action`, `requires`, `execution_scope`;
- implementation/environment/governance/review dependency absent -> classify the exact blocker;
- never use generic “need WAN authorization” for work already covered by standing authorization.

Then reconsider the live matrix. If and only if a genuine dependency-ready VPS-only row exists and it answers a declared missing release question, execute the smallest bounded self-owned experiment under standing authorization and preserve provenance, failures and cleanup. Otherwise record `READY_LIVE: none` and do not manufacture traffic.

No unchanged retry of already-sufficient repeated/periodic/HY2 lines.

## Completion gates

The current D019/RSEC-001 implementation lane is complete only when all are true:

- A1r remains green at exact code and no regression reopens its absolute I/O deadline/socket-state contract;
- pending pre-auth queue ownership/expiry is proven through the B1v lifecycle matrix;
- every rejection class terminalizes the logical pre-auth state across inner and outer accounting;
- every real responder has machine-checkable charge ordering and evidence barrier coverage;
- carrier/source projection is explicit and bounded;
- terminal-source persistence semantics are resolved by reviewed policy without unbounded source-accounting state or invented convenience limits;
- the complete deterministic adversarial/overflow/timeout/cleanup matrix passes;
- exact-head local gate and GitHub CI are green;
- reviewer security/evidence prose names the exact reviewed tree and does not outrun implementation;
- governance/release flags remain unchanged.

The broader rolling queue remains active through H/I/J unless a real stop condition occurs.

## Do not expand into

- public or production listener deployment;
- new numeric D019 ceilings, retention TTLs, LRU/history sizes or eviction rules without reviewed ADR policy;
- protocol/wire/Noise/Session/Carrier redesign unrelated to the concrete admission findings;
- VPS load testing as a substitute for deterministic security accounting;
- renewed HY2 work without a changed hypothesis and a declared missing comparison question;
- speculative FEC/0-RTT/exotic-carrier work;
- reading, hashing, copying, modifying or committing protected identity/secrets/private endpoint material;
- release/RC/freeze/production promotion.

## Questions requiring maintainer decision

None at this review point.

C2 may become a genuine ADR decision after C1 and the responder inventory are complete. If that happens, record the exact policy conflict and continue independent H/I work rather than blocking the whole project or inventing a numeric retention rule.
