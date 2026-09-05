# Nekomusume ChatGPT Handoff

Checked at: 2026-09-05 13:58 Asia/Shanghai
Repository HEAD reviewed: `884621c1e8a0e23fc9e0b2312bb3e1a482687cf2`
Previous checked implementation HEAD: `f98161b05d71783d45e03c04e34ccd5127259cf2`
Previous reviewer handoff commit: `7680285615873fa8908cb3ea1bf2446d7fd202af`

## What changed

One coding-agent commit landed after the previous reviewer handoff:

- `884621c` — **structural D019 rejection-terminalization repair; no wire/Noise/Session/Carrier/release semantic change.** It makes inner `PreauthBudget` input/response rejection poison the corresponding outer process state, wraps state-associated process operations so all post-state-selection failures terminalize the same logical state, preserves outer rejection on source/global ceilings, and keeps response completion/deadline failure terminal. New deterministic tests cover inner input rejection, inner response/anti-amplification rejection, backwards/unusable time, response-deadline construction overflow and non-revival of the same logical ticket.

At review time both `main` and `work/continue-20260904` point at exact `884621c1e8a0e23fc9e0b2312bb3e1a482687cf2`.

### Exact-head CI

Exact `884621c` is independently green on both refs:

- main Rust CI run `33946506610` — `success`;
- work-branch Rust CI run `33946512910` — `success`.

This is repository CI evidence, not a security approval.

## Review verdict

**CONTINUE_WITH_REQUIRED_FIXES — D1 is accepted as closed at exact green `884621c`; immediately advance through E1 -> E2 -> C1 -> C2 -> F -> G -> H -> I -> J without another reviewer wait. RSEC-001 remains HIGH/open because responder-wide charge/cleanup/evidence ordering is not yet machine-closed and the D019 carrier/source persistence policy conflict remains unresolved.**

No administrator action is required at this review point. Do not spend VPS time on this deterministic security-accounting lane while E/C2 remain unresolved. The current release-evidence ledger still reports no truthful dependency-ready `READY_LIVE` row.

## Reviewer findings

### RSEC-001A1 — CLOSED at `b775f1e`

Keep the accepted absolute response-I/O deadline contract intact:

- one absolute 100 ms deadline is carried across partial TCP writes/flush;
- zero remaining budget fails closed rather than gaining an extra millisecond;
- UDP/TCP send helpers bound actual socket I/O and restore the previous write timeout after successful bounded completion;
- exact-head tests and CI are green.

Evidence boundary: process-side bounded send-attempt semantics only. This is not proof of remote receipt within 100 ms or provider/kernel delivery latency.

Do not reopen this into a generic async-I/O redesign absent a concrete defect.

### RSEC-001B1 — CLOSED at `f98161b`

The accepted one-owner queue/process-expiry direction has sufficient deterministic closure for the current bounded implementation:

- idle expiry is exact at 1,000 ms while lifetime is still below 5,000 ms;
- lifetime expiry is exact at 5,000 ms;
- process expiry consumes state, queue and reserved memory once;
- stale queue permits cannot dequeue after process expiry;
- source/global queue max/max+1 fail closed;
- normal dequeue + state release returns counters to zero;
- application-level pending ownership is invalidated from the same process-expiry result;
- the failover UDP pending negotiation owner is discarded on process expiry and dequeued/released on successful authentication.

Evidence boundary: deterministic queue ownership/accounting only, not public load/capacity evidence and not D019 closure as a whole.

### RSEC-001D1 — CLOSED at `884621c`

The previous terminalization gaps are materially repaired.

Accepted properties:

- inner `PreauthBudget::charge_input` failure now calls process `reject_state` before returning;
- inner `PreauthBudget::charge_response` / anti-amplification failure likewise terminalizes the outer logical state;
- public process `charge_input`, `enqueue`, `charge_response` and `complete_response` wrap their checked implementations and reject the selected state on **any** returned error, so failures from backwards/unusable time, expired/rejected `live`, checked arithmetic, source/global limits and deadline construction cannot silently leave the state reusable;
- response I/O failure still uses `abandon_response` and remains terminal;
- outer rejection followed by inner rollback remains accounting-correct without reviving the logical state;
- direct tests prove the same ticket cannot resume input/queue/response after representative inner failures, and state-associated clock/deadline-construction failures remain dead.

Important boundary for the next audit: `reject_state` marks the logical state terminal but does not itself prove that every **runtime responder call site** immediately cancels application-owned pending work and performs exactly-once release. D019 requires bounded cleanup and no success evidence after rejection. That cross-responder ownership/evidence property belongs to E1/E2 below and is the next security gate; do not reopen D1 into a second controller redesign unless E finds a concrete missing transition.

### RSEC-001E — HIGH/open — responder-wide charge, rejection, cleanup and evidence ordering still needs machine closure

The repository now has the right primitives, but the security conclusion still requires a concrete inventory of every externally reachable pre-auth responder path. The audit must prove not merely that `ListenerAdmission` exists, but that each path uses it in the correct order and does not retain a rejected logical ticket long enough to generate authentication/readiness/Delivery/PathValidated/ACK-equivalent success evidence.

Specific questions E must answer for each real path:

1. Is typed carrier/source state admitted before owned pre-auth allocation/work?
2. Are exact input bytes/packet and conservative work units charged before the protected parse/work?
3. Is the 16 KiB state reservation made before state-owned allocation and released exactly once?
4. Where application-level pending ownership exists, is queue reservation acquired **before** the object becomes owned and canceled/dequeued exactly once?
5. Is response charge performed before serialization/send and is actual I/O covered by the accepted absolute deadline?
6. On any inner/outer rejection, timeout, malformed input or I/O failure, what exact call path cancels pending work, prevents success evidence and releases or deterministically expires the rejected state?
7. Can any externally reachable responder bypass this composition path or introduce a new listener without an admission guard?

Do not answer this with prose only. Produce a machine-checkable/static inventory or deterministic guard tied to the actual responder surfaces.

### RSEC-001C — ADR checkpoint remains isolated, not yet an administrator blocker

Current `source_key(peer)` still projects only family/address/port. D019 says the source key is the received **carrier/source tuple**, so current TCP and UDP pre-auth domains must become explicitly distinguishable in the projection rather than relying on implicit caller context.

A separate policy tension remains:

- D019 describes per-source input/packet/work/response accounting as state-lifetime scoped; and
- the same ADR says a counter is not reset by retry, reconnect, carrier change, identity change or error.

Current `release` removes a source usage entry after its last live state, so reconnect can regain fresh source budget. Retaining every terminal source forever would make the source map unbounded, while the reviewed ADR supplies no terminal-source retention TTL, retained-history ceiling, LRU size or eviction rule.

Do not invent convenience numbers. Complete E1/E2, implement only the noncontroversial carrier/source projection in C1, then re-read the exact policy. If bounded source-accounting memory and literal no-reset semantics still cannot both be satisfied, C2 should produce the compact ADR amendment request and stop only that policy-dependent lane.

## Evidence boundaries

- `IMPLEMENTATION_COMPLETE=true` remains a bounded research-baseline flag only.
- `CANONICAL_CORPUS_V1_FROZEN=true` remains corpus-specific only.
- `RELEASE_CANDIDATE=false`, `PRODUCTION_READY=false`, `FREEZE=false`, `RELEASED=false` remain required.
- Exact `884621c` adds deterministic structural rejection-terminalization plus exact-head green CI. It adds no WAN/VPS behavior evidence and is not a security audit.
- Existing inner `PreauthBudget` remains the stricter per-state input/anti-amplification bound and must not be weakened or removed merely to simplify outer accounting.
- Process admission does not claim control over kernel SYN backlog, provider NAT state or resources outside the process.
- `ROADMAP.md` / `IMPLEMENTATION_PLAN.md` continue to report no truthful dependency-ready `READY_LIVE` row. Do not manufacture traffic merely because the rented VPS is available.
- Standing VPS authorization remains valid for a future genuinely dependency-ready self-owned TCP/UDP evidence row; correctness/security gates still take precedence.
- Historical positive and negative WAN/HY2/failover/periodic evidence remains immutable at its exact commit boundary.
- Protected identity material, credentials, private endpoint material and raw private diagnostics remain unread/untracked/uncommitted.

## Rolling Work Queue

This is a rolling multi-hour queue. Finish one coherent slice -> targeted/full gates -> commit -> push -> immediately consume the next dependency-satisfied slice. Do not stop for a reviewer interval. Only a new HIGH/BLOCKER that invalidates downstream work, a genuine ADR/core-architecture conflict, action beyond authorization, production impact, missing credentials/third-party authority, repository breakage, runtime/tool-budget termination or real queue exhaustion is a stop condition.

### E1 — Build the exact responder admission/cleanup inventory

**Status:** `READY_LOCAL`; immediate security-priority slice.

Enumerate every externally reachable current pre-auth responder surface from code, not from stale documentation. At minimum include the ordinary TCP/UDP probe responder, periodic TCP responder, multistream TCP responder, failover TCP responder and failover UDP pending-negotiation path; include any additional current listener discovered by code search.

For each surface record in a machine-readable/static form:

- typed carrier/source projection and admission call;
- input byte/packet charge point;
- parser/work reservation and what bounded work it dominates;
- state-memory ownership point;
- queue reservation / pending owner if applicable;
- response charge and bounded send helper;
- rejection -> pending cancellation -> evidence barrier -> release/expiry path;
- success transition that ends pre-auth ownership;
- cleanup ownership on malformed input, timeout, EOF/cancellation and process shutdown.

The inventory must name concrete function/call-site anchors so `scripts/check.sh` or a deterministic test can detect drift.

If an existing responder bypasses one of the required admission layers, fix that exact seam in this slice rather than merely documenting it.

Run targeted tests + full `scripts/check.sh` + `git diff --check`; fuzz only if production network-input/parser semantics change. Commit and push.

**Continue immediately to E2:** yes.

### E2 — Make responder admission coverage mechanically non-optional

**Status:** `PREAUTHORIZED_AFTER_E1`.

Turn the E1 inventory into a regression/guard so a new externally reachable responder cannot silently omit pre-auth admission or reorder protected work.

Required properties:

- every inventoried responder has a deterministic test/static assertion for admission-before-parse/work;
- paths with application pending ownership prove queue-reserve-before-store and exactly-once cancellation/dequeue;
- rejected tickets prove no later authentication/readiness/Delivery/PathValidated/ACK/authz-equivalent success evidence;
- rejected/pending state is released immediately where the caller owns that transition, or deterministically expires within the D019 bound with no retained application work; the audit must make this distinction explicit rather than assuming `rejected=true` equals cleanup;
- the conservative `64` / `4096` work-unit reservations may remain only where they demonstrably dominate the bounded parser work they protect and are documented as accounting units, never CPU-cycle measurements;
- no new generic listener framework or runtime dependency unless a concrete uncovered path requires it.

Full gate, commit, push.

**Continue immediately to C1:** yes.

### C1 — Make carrier/source projection explicit and bounded

**Status:** `PREAUTHORIZED_AFTER_E2`.

Implement only the noncontroversial source-key portion:

- explicit bounded carrier discriminator at least distinguishing current TCP and UDP pre-auth domains;
- family/address/port remain represented without textual/raw logging;
- deterministic non-collision tests across carrier/family/address/port combinations;
- every current call site supplies the actual received carrier class rather than a guessed/default value;
- one bounded unknown/unusable-source bucket only if a current real call site actually needs it;
- no new terminal-source retention duration, LRU size, history count or eviction parameter.

Run targeted/full gate, commit, push.

**Continue immediately to C2:** yes.

### C2 — Resolve terminal-source persistence semantics or produce ADR amendment request

**Status:** `ADR_CHECKPOINT_AFTER_C1`.

Re-read `docs/adr/m1-g0-preauth-resource-budget.md` and adjacent reviewed decisions against the now-concrete responder inventory.

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

### F — Complete the full D019 adversarial/evidence-barrier matrix

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
- terminal non-revival across both accounting layers;
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

RSEC-001 may close as an implementation finding only when E1/E2/C1/C2/F are actually satisfied and exact-head CI is green. Independent external/two-person security review remains a separate release gate. Never promote RC/production/freeze/release automatically.

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
- never use generic `need WAN authorization` for work already covered by standing authorization.

Then reconsider the live matrix. If and only if a genuine dependency-ready VPS-only row exists and it answers a declared missing release question, execute the smallest bounded self-owned experiment under standing authorization and preserve provenance, failures and cleanup. Otherwise record `READY_LIVE: none` and do not manufacture traffic.

No unchanged retry of already-sufficient repeated/periodic/HY2 lines.

## Completion gates

The current D019/RSEC-001 implementation lane is complete only when all are true:

- A1 absolute response-I/O deadline remains green and bounded;
- B1 queue ownership/expiry verification remains green;
- D1 structural rejection terminalization remains green;
- every real responder has machine-checkable admission, charge, rejection, evidence-barrier and cleanup ordering;
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

C2 may become a genuine ADR decision after C1 and the responder inventory are complete. If that happens, record the exact policy conflict and continue independent H/I/J work rather than blocking the whole project or inventing a numeric retention rule.
