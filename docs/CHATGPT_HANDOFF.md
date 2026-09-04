# Nekomusume ChatGPT Handoff

Checked at: 2026-09-04 22:44 Asia/Shanghai
Repository HEAD reviewed: `062150ffe75e82f89abd8532dfb53ab35e4f0343`
Latest reviewed implementation HEAD: `ea1c05216b1b6e2ec198b907477af5056e4a956c`
Previous reviewer handoff commit: `062150ffe75e82f89abd8532dfb53ab35e4f0343`

## What changed

No new implementation commit landed after the 21:00 review. `062150f` is reviewer/handoff-only and preserves the implementation tree at `ea1c052`.

Exact `062150f` GitHub Actions are green on both relevant refs:

- main run `33876203452` — `success`;
- work/continue-20260904 run `33881852848` — `success`.

The current D019 findings were re-checked against the actual ADR and current implementation rather than only the previous review prose. The gaps are real and concrete:

1. `ProcessPreauthLimits` still has no explicit `response_send_deadline_ms = 100`; `ListenerAdmission::charge_response` only charges immediately before a caller send and returns no permit/deadline object.
2. `ProcessPreauthAdmission::{enqueue,dequeue}` exists, but `ListenerAdmission` exposes no queue reservation lifecycle, so application-owned pending pre-auth work is not structurally charged before ownership.
3. `source_key(peer)` currently contains address family/address/port but no carrier discriminator. `ProcessPreauthAdmission::release` deletes the source accounting row when the final live state disappears, erasing cumulative source input/work/response counters. The existing CLI regression explicitly expects release to reopen the source.
4. A failed `charge_input`, `charge_response` or `enqueue` does not terminally poison/consume the reusable logical pre-auth state; the same state may be retried later after a one-second global window rollover.

The D019 text itself creates a real design checkpoint for item 3: it says per-source counters are scoped to the source/state lifetime, but also says a counter is never reset by retry, reconnect, carrier change, identity change or error. Persisting source counters after terminal state release requires bounded retained source accounting; D019 currently defines no retention TTL/count/table ceiling for terminal source tombstones. Silently retaining every historical source forever would create a new unbounded resource, while silently inventing a TTL/count would introduce a new numeric security policy.

Therefore RSEC-001C is not an ordinary implementation detail. It is an ADR clarification/amendment checkpoint unless a bounded interpretation can be proved from already-reviewed repository text without adding a new value.

## Review verdict

**CONTINUE_WITH_REQUIRED_FIXES — A/B/D/E are concrete dependency-ready local engineering work; C is isolated as a possible ADR checkpoint and must not block the independent deterministic repairs before it.**

The previous queue ordering was too serial: placing C before D could cause an ADR question to idle several hours of unrelated deterministic security work. The queue is reordered below so the agent should consume A -> B -> D -> E continuously, then address C.

Do not use VPS time for this lane. These are deterministic admission/accounting correctness questions. Do not describe the release/security gate as “only waiting for external review” while A/B/C/D remain open.

## Evidence boundaries

- `IMPLEMENTATION_COMPLETE=true` remains the bounded research baseline only.
- `CANONICAL_CORPUS_V1_FROZEN=true` remains corpus-specific only.
- `RELEASE_CANDIDATE=false`, `PRODUCTION_READY=false`, `FREEZE=false`, `RELEASED=false` remain required.
- Exact `062150f` CI success is repository CI evidence, not a security audit or release approval.
- Existing `PreauthBudget` anti-amplification/state-local accounting remains useful and must not be weakened merely to simplify process accounting.
- `ProcessPreauthAdmission` and listener integrations are real implementation progress, but they do not yet satisfy the full D019 contract.
- Process accounting cannot claim to bound kernel SYN backlog, provider NAT state or other resources outside the process.
- No current VPS row should be manufactured merely because the VPS rental clock is running; use VPS again only when release-closure reclassification produces a genuine dependency-ready live evidence question.
- Protected identity/secrets/private endpoint material remain unread/untracked/uncommitted.

## Rolling Work Queue

This is a rolling multi-hour queue. Every dependency-satisfied item is pre-authorized to continue immediately after the preceding coherent slice is tested, committed and pushed. Do not stop after one commit, one hour, or one reviewer interval. A new HIGH/BLOCKER that invalidates downstream work, a genuine ADR/core-architecture conflict, authorization boundary, production impact, missing credentials/third-party authority, repository breakage, runtime/tool-budget termination, or true queue exhaustion is a stop condition.

### A — Implement typed 100 ms response-send ownership

**Status:** `READY_LOCAL`.

**Goal**

Implement the exact D019 `Response-send deadline = 100 ms` as an explicit process-admission property without changing protocol bytes, response shape, Noise, Session or Carrier semantics.

**Preferred implementation shape**

- Add `response_send_deadline_ms: u64` to `ProcessPreauthLimits`; default exactly `100`.
- Validate it as non-zero and no broader than the state lifetime.
- Do not let `charge_response` be the final semantic operation. Have response admission create a typed one-shot ownership/permit containing at least state identity, charged bytes/packet ownership and monotonic admitted-at/deadline data.
- Completing a bounded send attempt before/at the defined boundary consumes/completes that permit exactly once.
- Expiry/abandonment consumes it exactly once and cannot later become success evidence.
- A late completion is not successful pre-auth response evidence.
- Cumulative response accounting remains charged even for scheduled/failed attempts as D019 requires; do not “refund” a response merely because the socket send failed or expired.

`ListenerAdmission` should expose the smallest API needed to admit and complete/abandon a response; do not leak raw `ProcessPreauthAdmission` internals into every responder.

**Tests**

- deterministic injectable time: 99 ms / 100 ms / 101 ms boundary according to one explicit inclusive/exclusive interpretation documented from D019;
- complete exactly once;
- double completion/abandon rejected;
- expiry cannot produce auth/Delivery/PathValidated/ACK/readiness-equivalent evidence;
- response rate/anti-amplification accounting remains atomic across inner/outer layers;
- no wall-clock sleeps.

Run targeted tests, full local gate, `git diff --check`, commit and push.

**Continue immediately to B:** yes.

### B — Integrate pending queue ownership with RAII/one-shot cleanup

**Status:** `PREAUTHORIZED_AFTER_A`.

**Goal**

Turn the dormant process queue primitive into actual runtime ownership accounting.

**Preferred implementation shape**

- Add a small `QueueReservation` / guard-style API through `ListenerAdmission` (or an equivalent move-only one-shot ownership object).
- Charge source/global queue capacity **before** application code stores a pending pre-auth entry.
- The reservation must be structurally released exactly once when pending ownership ends: authentication/promotion, terminal malformed rejection, idle/lifetime expiry, cancellation, replacement, or shutdown cleanup.
- At minimum integrate `failover-server` UDP `PendingUdpNegotiation`.
- Audit other actual application-owned pre-auth queues; do not invent a new queue subsystem where none exists.
- Keep the existing candidate numeric limits and 16 KiB conservative state reservation; do not introduce new values.

**Tests**

- source max/max+1;
- global max/max+1;
- rejected enqueue stores no pending object;
- promotion/rejection/timeout/cancel/shutdown release exactly once;
- no double dequeue under replacement/cancellation;
- queue rejection produces no success/session/path evidence.

Run targeted tests + full gate, commit and push.

**Continue immediately to D:** yes. C no longer blocks D.

### D — Make every rejected pre-auth operation terminal/non-revivable

**Status:** `PREAUTHORIZED_AFTER_B`.

**Goal**

Enforce D019’s “fails closed; do not retry” rule at the reusable state/ticket boundary rather than relying on each CLI caller to remember to exit.

**Preferred implementation shape**

Choose one structurally hard-to-misuse model:

1. mark the process pre-auth state terminal/rejected on any exhausted/saturated/unmeasurable/deadline rejection and make every later operation fail; or
2. make charge/queue/response APIs consume a ticket/ownership token on failure so the same logical state cannot be called again.

Do not create a hidden path that can revive a rejected ticket after the global one-second window refreshes.

**Required regressions**

- global input-window rejection -> same state still rejected after rollover;
- global work-window rejection -> same state still rejected after rollover;
- global/source response rejection -> same state cannot later send;
- queue rejection -> same rejected ownership cannot later enqueue;
- response deadline expiry -> same response/state cannot later succeed;
- inner `PreauthBudget` failure and outer process-budget failure remain cross-layer atomic;
- release/cleanup remains one-shot.

Run targeted tests + full gate, commit and push.

**Continue immediately to E:** yes.

### E — Audit and machine-check charge ordering across every real responder

**Status:** `PREAUTHORIZED_AFTER_D`.

**Goal**

Make the “charge before protected work/ownership” rule auditable across current externally reachable responder code.

For each actual pre-auth responder path, map/check:

1. source/carrier projection and state admission;
2. input bytes/packet charge before parse;
3. parser/work reservation before bounded parsing/work;
4. state memory reservation before owned allocation;
5. queue reservation before application pending ownership;
6. exact response charge + response-send permit before serialization/send;
7. terminal rejection/evidence barrier;
8. exactly-once cleanup.

Current conservative work reservations such as 64/4096 units may remain if they demonstrably dominate the bounded parser work they protect; they are accounting units, not CPU-cycle claims. Fix only a real input-controlled work seam that can exceed reservation.

Add a machine-checkable/static inventory or deterministic integration test so a new externally reachable responder cannot silently bypass `ListenerAdmission`.

Run full gate, commit and push.

**Continue immediately to C:** yes.

### C — Resolve D019 carrier/source persistence semantics without inventing an unbounded map

**Status:** `ADR_CHECKPOINT_AFTER_E`.

This is the one slice that may legitimately require a reviewer/maintainer decision.

**Facts that must be preserved**

- current `source_key(peer)` lacks a carrier discriminator;
- current source counters are deleted when `usage.states == 0`;
- current test behavior explicitly allows terminal release to reopen the same source;
- D019 says source key is a received carrier/source tuple and says counters are never reset by retry/reconnect/carrier change/identity change/error;
- D019 also describes several per-source counters as state-lifetime scoped;
- keeping terminal source accounting forever is itself unbounded;
- no terminal-source retention TTL/count/table ceiling is currently specified by D019.

**First do the noncontroversial part**

- replace ad-hoc `Vec<u8>` construction at the caller boundary with a typed bounded source projection including an explicit carrier discriminator (`TCP`, `UDP`, and shared unknown bucket where applicable);
- add deterministic tests showing TCP/UDP cannot accidentally collide when they should be distinct and invalid/unknown projection goes to one bounded shared bucket;
- do not log raw source identifiers.

**Then resolve persistence**

Do **not** silently choose one of these without reviewed justification:

- retain all historical source rows forever;
- invent a source tombstone TTL;
- invent a max historical-source count/LRU;
- reuse the one-second global rate window as source lifetime;
- use `max_states_global` as an unrelated historical-source-table ceiling;
- drop the port from the source key merely to make reconnect persistence easier.

If existing reviewed text cannot prove a bounded interpretation, record a small ADR amendment request that states the exact conflict and presents bounded options. This is a real stop for **this slice only**. It must not erase the already completed A/B/D/E work.

If a reviewed interpretation/amendment is available, implement it with tests for retry/reconnect and TCP<->UDP carrier transition, bounded source-accounting storage, and no budget reset contrary to that interpretation.

**Continue immediately to F only after C is resolved:** yes.

### F — Complete D019 adversarial boundary/evidence-barrier matrix

**Status:** `PREAUTHORIZED_AFTER_C`.

Cover the full deterministic matrix:

- source/global concurrency max/max+1;
- source input bytes/packets/work lifetime semantics under the resolved C policy;
- global input/work one-second windows;
- per-packet work;
- state/global memory;
- source/global pending queue;
- source/global response plus inner 3x anti-amplification;
- idle 1 s / lifetime 5 s / response-send 100 ms with injectable time;
- overflow;
- terminal rejection/non-revival;
- retry/reconnect/carrier-transition behavior;
- cancellation/timeout/double cleanup;
- no Delivery/PathValidated/ACK/readiness/authz-equivalent evidence on pre-auth rejection;
- secret-safe diagnostics.

Do not substitute VPS/load tests for deterministic accounting correctness.

Full local gate, commit, push, then require exact repair HEAD CI green for security closure.

**Continue immediately to G:** yes after exact-head CI success.

### G — Fresh exact-tree security review and evidence correction

**Status:** `PREAUTHORIZED_AFTER_F`.

Re-read exact code/tests after A-F. Update only then:

- `docs/reviews/resource-abuse-evidence-2026-09-04.md`;
- `docs/release-security-review-packet.md`;
- `docs/status.md`;
- relevant closure/navigation records.

The reviewed-tree field must name the actual reviewed implementation HEAD. RSEC-001 may close as an implementation finding only if A-F are actually satisfied. Independent external/two-person security review remains a separate release gate.

Do not promote RC/production/freeze/release automatically.

**Continue immediately to H if no new HIGH/BLOCKER:** yes.

### H — Compatibility and freeze-boundary review

**Status:** `READY_LOCAL_AFTER_G`.

Audit:

- corpus-v1 content-addressed freeze vs global protocol non-freeze;
- current/current negotiation;
- unsupported/future rejection;
- downgrade/transcript binding into Noise;
- resume/version binding and replay boundary;
- docs that could imply corpus freeze == protocol/release freeze.

Add deterministic regression only for an actual gap. Do not reopen frozen corpus bytes without a correctness defect.

**Continue immediately to I:** yes.

### I — Package/operator and evidence-provenance integrity review

**Status:** `READY_LOCAL_AFTER_H`.

Verify bounded questions already claimed by repository evidence:

- x86_64 package/build identity;
- install -> readiness/smoke -> upgrade -> rollback;
- retained external state without reading protected identity material;
- shutdown/listener/temp cleanup;
- canonical Git-blob/checksum evidence manifests;
- exact-head CI references;
- stale links/hashes in release packet.

Do not rerun VPS/package experiments for freshness if the bounded question is already answered. Fix concrete defects only.

**Continue immediately to J:** yes.

### J — Reclassify remaining release opportunities and reconsider VPS

**Status:** `READY_LOCAL_AFTER_I`.

Re-evaluate each release-closure row:

- bounded question already answered -> `ALREADY_SUFFICIENT_FOR_BOUNDED_QUESTION`;
- specific executable missing assertion -> `OPEN_READY` with `evidence_needed`, `next_action`, `requires`, `execution_scope`;
- environment/governance/implementation dependency -> truthful blocked class.

Then reconsider the VPS. Execute a live row only when the closure map identifies a genuine dependency-ready missing release question under standing authorization. Otherwise record `READY_LIVE: none` and do not create traffic merely to consume rental time.

Do not unchanged-retry closed repeated/periodic/HY2 lines.

## Completion gates

RSEC-001 implementation closure requires all of:

- explicit 100 ms response-send ownership/deadline;
- actual runtime pending-queue reservations and exactly-once release;
- terminal/non-revivable rejection semantics;
- auditable charge ordering across every external responder;
- carrier/source projection and persistence reconciled with D019 under a bounded reviewed policy;
- complete deterministic boundary/overflow/timeout/evidence-barrier tests;
- full local gate + exact-head GitHub CI green;
- exact-tree security evidence prose not ahead of implementation.

The broader rolling queue remains active through H-J unless a real stop condition occurs.

## Do not expand into

- public/production listener deployment;
- new numeric D019 retention/source-table ceilings without explicit reviewed ADR amendment;
- protocol/wire/Noise/Session/Carrier redesign unrelated to the concrete admission defects;
- VPS load testing as a substitute for deterministic security accounting;
- reopening frozen corpus bytes without correctness evidence;
- speculative FEC/0-RTT/striping/multipath/exotic carrier work;
- third-party targets or scanning;
- production route/firewall/DNS/proxy/tunnel/qdisc changes;
- reading/copying/hashing/committing protected identity/secrets/private topology.

## Questions requiring maintainer decision

**No immediate administrator action is required for A/B/D/E.**

A decision may become necessary at C only if repository-reviewed text cannot already resolve terminal source-accounting retention while keeping the source table bounded. If that happens, the agent should record the exact ADR conflict and bounded options, then continue any independent READY review work that does not depend on C rather than idling the entire project.
