# Nekomusume ChatGPT Handoff

Checked at: 2026-09-05 12:59 Asia/Shanghai
Repository HEAD reviewed: `f98161b05d71783d45e03c04e34ccd5127259cf2`
Previous checked implementation HEAD: `b775f1e241ba58785f80708ba41ef51856a5e259`
Previous reviewer handoff commit: `7da4d205a991d8b865748643d6a95a20bb5404b2`

## What changed

One coding-agent commit landed after the previous reviewer handoff:

- `f98161b` — **deterministic D019 queue-ownership verification only; no wire/Noise/Session/Carrier/release semantic change.** It adds exact idle/lifetime expiry tests, stale queue-owner invalidation, source/global queue max+1 coverage, exactly-once queue/state/memory cleanup checks, and a controller-survival check showing ordinary unauthenticated expiry does not make the admission controller unusable.

Both `main` and `work/continue-20260904` point at exact `f98161b05d71783d45e03c04e34ccd5127259cf2` at review time.

### Exact-head CI

Exact `f98161b` is independently green on both refs:

- main Rust CI run `33943764474` — `success`;
- work-branch Rust CI run `33943769792` — `success`.

This is repository CI evidence, not a security approval.

## Review verdict

**CONTINUE_WITH_REQUIRED_FIXES — B1v is accepted as closed at exact green `f98161b`; continue immediately through D1 -> E -> C1 -> C2 -> F/G/H/I/J without another reviewer wait. RSEC-001 remains HIGH/open because terminal rejection is still not structural across all inner/outer failure classes, responder charge-order coverage is not yet machine-closed, and the D019 terminal-source persistence policy conflict remains unresolved.**

No administrator action is required at this review point. Do not spend VPS time on this deterministic security-accounting lane while D1/C2 remain unresolved.

## Reviewer findings

### RSEC-001A1 — CLOSED at `b775f1e`

Keep the accepted absolute response-I/O deadline contract intact:

- one absolute 100 ms deadline is carried across partial TCP writes/flush;
- zero remaining budget fails closed rather than gaining an extra millisecond;
- UDP/TCP send helpers restore the preexisting write timeout on successful bounded completion;
- exact-head tests and CI are green.

Evidence boundary: process-side bounded send-attempt semantics only. This is not proof of remote receipt within 100 ms or provider/kernel delivery latency.

Do not reopen this into a generic async-I/O redesign absent a concrete defect.

### RSEC-001B1 — CLOSED at `f98161b`

The accepted one-owner queue/process-expiry direction now has sufficient deterministic closure for the current bounded implementation:

- idle expiry is exact at 1,000 ms while lifetime is still below 5,000 ms;
- lifetime expiry is exact at 5,000 ms;
- process expiry consumes state, queue and reserved memory once;
- stale queue permits cannot dequeue after process expiry;
- source queue max/max+1 and global queue max/max+1 across distinct sources fail closed;
- normal dequeue + state release returns counters to zero;
- the `ListenerAdmission` integration invalidates application-level queue ownership after process expiry and remains able to admit a later peer rather than turning ordinary unauthenticated expiry into controller failure;
- the real failover responder already uses the same expired-state list to discard its `PendingUdpNegotiation` owner, and successful authentication dequeues then releases the pending ownership.

The low-level promotion/cancellation test names two terminal paths through the same one-shot dequeue/release primitive; do not expand this into a new queue subsystem unless a concrete responder path demonstrates a missing owner transition.

Evidence boundary: this closes current deterministic queue ownership/accounting semantics. It is not a public-load/capacity claim and does not close D019 as a whole.

### RSEC-001D1 — HIGH — every rejection class is still not terminal across both accounting layers

Current exact `f98161b` still has two important structural gaps.

1. `ListenerAdmission::charge_input` and `charge_response` call the inner `PreauthBudget` first and return immediately when that inner limit/anti-amplification check rejects. Those errors never reach `ProcessPreauthAdmission::reject`, so the same logical ticket may still be live at the outer layer.
2. Several state-associated outer failures return through `?` before `reject(id)` is reached: backwards/unmeasurable window time in `refresh_window`, `live` deadline/clock rejection, checked-add overflow while computing source/global byte/packet/work counters, and response-deadline construction overflow. A caller retaining the logical ticket can therefore receive an error without a mechanically terminal state in those classes.

The existing outer limit branches for input/response/queue ceilings correctly call `reject(id)`, and response I/O failure correctly routes through `abandon_response`. Preserve those improvements; do not regress them while making rejection uniform.

**Required repair:** make terminalization structural at the logical ticket/controller boundary rather than relying on each call site to remember it.

Preferred bounded shape:

- expose one small process `reject_state(id)`/equivalent primitive that is idempotent when the state still exists;
- give `ListenerAdmission` one `terminalize(ticket)` or failure helper so an inner `PreauthBudget` rejection also poisons the outer process state before returning;
- refactor state-associated `ProcessPreauthAdmission` operations so any error after a state id has been selected either marks that id rejected before return or consumes/releases the logical state in a clearly one-shot API;
- do not turn `admit_state` failure into a retained tombstone because no logical state exists yet;
- preserve cross-layer accounting truth: rollback may undo a just-applied inner charge after an outer rejection, but rollback must never make the logical state reusable;
- deliberate experimental response suppression is not automatically the same as budget rejection; do not break an explicitly controlled fault seam merely because an unused response permit is dropped. Only actual rejection/timeout/I/O failure must be terminal under D019.

Required deterministic regressions include:

- inner per-state input limit failure -> later input/queue/response on same logical ticket all fail;
- inner response/3x anti-amplification failure -> later input or window rollover cannot revive response ability on that ticket;
- global input/work rejection -> same logical state remains dead after one-second rollover;
- source/global response rejection -> same logical state cannot respond later;
- source/global queue rejection -> same logical ownership cannot enqueue later;
- backwards time / expired-live-state / checked arithmetic or response-deadline construction failure cannot leave a reusable state when the failure is associated with an existing id;
- response I/O/deadline failure remains terminal;
- cleanup/release after rejection is bounded and exactly once;
- no Session/PathValidated/Delivery/ACK/readiness/authz-equivalent success evidence follows any pre-auth rejection.

### RSEC-001C — ADR checkpoint remains isolated, not yet an administrator blocker

Current `source_key(peer)` still projects only family/address/port. D019 says the source key is the received carrier/source tuple, so TCP and UDP must be distinguishable in the explicit projection rather than relying on separate caller context.

A separate policy tension still exists in D019:

- per-source byte/packet/work/response counters are described as state-lifetime accounting; and
- counters also must not reset through retry, reconnect, carrier change, identity change or error.

Current `release` removes a source usage entry after its last live state, so reconnect can regain fresh source budget. Retaining every terminal source forever would make the source map unbounded, while the reviewed ADR provides no terminal-source retention TTL, history ceiling, LRU size or eviction policy.

Do not invent convenience numbers. Finish D1/E first, make the carrier/source projection explicit in C1, then re-read the policy against the concrete inventory. If bounded storage and no-reset semantics still cannot both be satisfied, C2 should produce the compact ADR amendment request and stop only that policy-dependent lane.

## Evidence boundaries

- `IMPLEMENTATION_COMPLETE=true` remains a bounded research-baseline flag only.
- `CANONICAL_CORPUS_V1_FROZEN=true` remains corpus-specific only.
- `RELEASE_CANDIDATE=false`, `PRODUCTION_READY=false`, `FREEZE=false`, `RELEASED=false` remain required.
- Exact `f98161b` adds deterministic queue-ownership verification and exact-head green CI. It does not add new WAN/VPS behavior evidence, close RSEC-001 globally, or constitute a security audit.
- Existing inner `PreauthBudget` remains the stricter per-state input/anti-amplification bound and must not be weakened or removed to simplify the outer controller.
- Process admission does not claim control over kernel SYN backlog, provider NAT state or resources outside the process.
- `ROADMAP.md` / `IMPLEMENTATION_PLAN.md` continue to report no truthful `READY_LIVE` row in the current release-evidence matrix. Do not manufacture traffic merely because the rented VPS is available.
- Standing VPS authorization remains valid for a future genuinely dependency-ready self-owned TCP/UDP evidence row; correctness/security gates still take precedence.
- Historical negative and bounded positive WAN/HY2/failover/periodic evidence remains immutable at its exact commit boundary.
- Protected identity material, credentials, private endpoint material and raw private diagnostics remain unread/untracked/uncommitted.

## Rolling Work Queue

This is a rolling multi-hour queue. Finish one coherent slice -> targeted/full gates -> commit -> push -> immediately consume the next dependency-satisfied slice. Do not stop for a reviewer interval. Only a new HIGH/BLOCKER that invalidates downstream work, a genuine ADR/core-architecture conflict, action beyond authorization, production impact, missing credentials/third-party authority, repository breakage, runtime/tool-budget termination or real queue exhaustion is a stop condition.

### D1 — Make every rejection class terminal across inner + outer accounting

**Status:** `READY_LOCAL`; immediate security-priority slice.

Implement the structural terminalization described in RSEC-001D1 without changing D019 numeric ceilings, wire bytes, Noise, Session or Carrier semantics.

Likely areas:

- `crates/neko-cli/src/preauth.rs`;
- `crates/neko-crypto/src/lib.rs`;
- existing pre-auth tests only as needed.

Required behavior/tests:

- inner input rejection terminalizes the outer logical state;
- inner response / anti-amplification rejection terminalizes it;
- outer source/global input/work/response/queue rejection remains terminal;
- state-associated clock/deadline/overflow failures cannot leave a reusable logical state;
- response I/O/deadline failure remains terminal;
- cross-layer rollback remains accounting-correct but never revives admission;
- exactly-once cleanup remains valid;
- deliberate controlled fault-injection paths remain explicit and are not silently reclassified as budget failures.

Run targeted tests, full `scripts/check.sh`, relevant fuzz only if production network-input/parser semantics change, and `git diff --check`; commit and push.

**Continue immediately to E:** yes.

### E — Audit and machine-check charge ordering across every real responder

**Status:** `PREAUTHORIZED_AFTER_D1`.

Create or maintain a machine-checkable/static inventory for every externally reachable pre-auth responder path, including ordinary TCP/UDP probe, periodic, multistream and failover TCP/UDP paths.

For each path prove/order:

1. explicit typed carrier/source projection + state admission;
2. input byte/packet charge before parse;
3. parser/work reservation before protected work;
4. state-memory reservation before owned allocation;
5. queue reservation before application-level pending ownership where such ownership exists;
6. response charge + bounded actual I/O before send;
7. terminal rejection/evidence barrier;
8. exactly-once cleanup.

The current conservative `64` / `4096` work reservations may remain only if they dominate the bounded parser work they protect; document them as accounting units, never CPU-cycle measurements. Fix only concrete uncovered seams. Add a guard/test so a newly externally reachable responder cannot silently omit admission.

Run full gate, commit, push.

**Continue immediately to C1:** yes.

### C1 — Make carrier/source projection explicit and bounded

**Status:** `PREAUTHORIZED_AFTER_E`.

Implement only the noncontroversial source-key portion:

- explicit bounded carrier discriminator at least distinguishing current TCP and UDP pre-auth domains;
- family/address/port remain represented without textual/raw logging;
- deterministic non-collision tests across carrier/family/address/port combinations;
- one bounded unknown/unusable-source bucket only if a current real call site actually needs it;
- no new terminal-source retention duration, LRU size, history count or eviction parameter.

Run targeted/full gate, commit, push.

**Continue immediately to C2:** yes.

### C2 — Resolve terminal-source persistence semantics or produce ADR amendment request

**Status:** `ADR_CHECKPOINT_AFTER_C1`.

Re-read D019 and adjacent reviewed decisions against the now-concrete responder inventory.

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

RSEC-001 may close as an implementation finding only when D1/E/C1/C2/F are actually satisfied and exact-head CI is green. Independent external/two-person security review remains a separate release gate. Never promote RC/production/freeze/release automatically.

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

- A1r absolute response-I/O deadline remains green and bounded;
- B1v queue ownership/expiry verification remains green;
- every rejection class terminalizes the logical pre-auth state across inner and outer accounting;
- every real responder has machine-checkable charge ordering and evidence-barrier coverage;
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
