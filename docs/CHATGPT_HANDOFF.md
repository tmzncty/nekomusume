# Nekomusume ChatGPT Handoff

Checked at: 2026-09-04 21:00 Asia/Shanghai
Repository HEAD reviewed: `ea1c05216b1b6e2ec198b907477af5056e4a956c`
Previous reviewed coding/evidence HEAD: `bb80ab73c721125bfac55c21475a86672493cd94`
Previous reviewer handoff commit: `dc259eee2d49d235497b43090d9801ebecb781a8`

## What changed

The external agent consumed a substantial part of the RSEC-001 queue after the previous review. Thirteen commits are now ahead of `bb80ab7`; one is the previous reviewer handoff and the remaining coding/docs commits build and integrate process-owned pre-auth admission.

Accepted progress:

- `0a28a35` — adds `ProcessPreauthAdmission` / `ProcessPreauthLimits` with process-global state, memory, queue and one-second rate-window accounting plus deterministic boundary tests.
- `7be9918` / `9231628` / `2f83f75` — adds the CLI `ListenerAdmission` composition layer, integrates ordinary TCP/UDP probe listener admission, adds source-lifetime input/response/work counters and idle/lifetime handling, and corrects lifetime-window tests.
- `fbeaad0` — integrates the same admission path into periodic and multistream TCP responder handshakes.
- `91258eb` — integrates one shared admission controller across failover-server UDP and TCP pre-auth paths, including the pending UDP negotiation ticket.
- `cbef929` — reconciles listener-admission documentation/navigation.
- `40773c1` / `ea1c052` — repairs cross-layer rollback so an outer process-budget rejection does not leave the inner `PreauthBudget` charged, then formats the result.

Exact current HEAD `ea1c05216b1b6e2ec198b907477af5056e4a956c` has green GitHub Actions on both `main` and `work/continue-20260904`; main run `33874213735` and work-branch run `33874222896` both concluded `success`. This is exact-head CI evidence, not a security approval.

The implementation direction is materially better and the ordinary/failover/periodic/multistream listener coverage is useful. However, independent comparison against the actual D019 candidate contract finds several concrete semantic gaps. RSEC-001 therefore remains HIGH and must not be reworded as “implementation complete, only external review missing” yet.

## Review verdict

**CONTINUE_WITH_REQUIRED_FIXES — A-D made strong progress, but D019 security closure is still blocked by concrete admission-contract gaps. Keep the rolling queue moving; repair these gaps before E/F security-evidence closure or any release/security promotion.**

Do not spend VPS time on this security lane. Current release closure still reports no truthful dependency-ready live row, and these defects are deterministic local/process-accounting questions.

## Reviewer findings

### RSEC-001A — HIGH — response-send deadline is not implemented

D019 requires a **100 ms monotonic budget from response admission to completed bounded send attempt**. The current `ProcessPreauthLimits` exposes admission-window, idle-timeout and lifetime values but no response-send-deadline field/state. `ListenerAdmission::charge_response` charges bytes immediately before the caller writes/sends, but there is no response permit/deadline token and no check that the bounded send attempt completes within 100 ms.

Existing larger experiment/setup socket deadlines are not equivalent to this D019 response deadline.

**Required repair:** represent the existing 100 ms D019 value explicitly; bind it to response admission; prove complete-attempt-before-deadline and fail/abandon-after-deadline with an injectable clock or deterministic time input. Do not invent a different numeric value.

### RSEC-001B — HIGH — pending pre-auth queue accounting exists as a primitive but is not integrated

`ProcessPreauthAdmission::{enqueue,dequeue}` exists, but `ListenerAdmission` exposes no queue operation and the live responder paths do not charge queue entry ownership before storing pending work. In particular the failover UDP `PendingUdpNegotiation` becomes application-owned pending state without a queue charge/dequeue lifecycle.

D019 explicitly requires per-source/global pending pre-auth queue ceilings and charging before enqueue. A dormant primitive is not runtime evidence.

**Required repair:** expose a small fail-closed queue reservation/guard through `ListenerAdmission` (or equivalent), charge before the pending entry becomes owned, release/dequeue exactly once on authentication, rejection, timeout or cancellation, and demonstrate source/global queue saturation. Keep the existing 16 KiB per-state conservative reservation unless a concrete accounting bug requires another representation; do not invent new memory limits.

### RSEC-001C — HIGH — source accounting resets on terminal release and does not encode the carrier dimension

The D019 text says the source key is the received **carrier/source tuple** and that a counter is not reset by retry, reconnect, carrier change, identity change or error. The current CLI `source_key(peer)` contains IP family/address/port only; it has no carrier discriminator. More importantly, `ProcessPreauthAdmission::release` removes the `sources` entry when its last live state disappears, which discards accumulated source input/packet/work/response counters. A later retry/reconnect can therefore obtain fresh per-source accounting after the previous state is terminal.

This is a contract mismatch even though process-global one-second counters and concurrency ceilings still provide useful bounds.

**Required repair:** reconcile the implementation with the exact D019 counting-domain language. At minimum, make the carrier/source projection explicit and prove retry/reconnect/carrier transition cannot reset counters inside the D019 source accounting lifetime. Do not solve this by creating an unbounded forever-growing source map; if the literal ADR wording cannot be implemented without introducing a new bounded source-retention rule, stop this slice and record the ADR conflict for maintainer/reviewer decision rather than silently inventing a new numeric ceiling.

### RSEC-001D — MEDIUM/HIGH — rejected operations are not terminal in the reusable state machine

D019 says an exhausted/saturated operation fails closed, cancels pending work and must not become successful later merely because a rate window refilled. `ProcessPreauthAdmission::charge_input/charge_response/enqueue` return `Err` atomically but do not mark the state rejected/terminal. Many current CLI call sites immediately release the ticket or terminate the bounded process, which is useful integration behavior, but the reusable process-admission state machine itself still allows a caller to retry the same live state later.

**Required repair:** make terminal-rejection semantics machine-enforced at the controller/ticket boundary, or make the consuming API structurally consume/close the ticket on any rejected charge so reuse is impossible. Add a regression where a global-window rejection cannot be retried successfully after window rollover using the same logical pre-auth state.

### RSEC-001E — MEDIUM evidence drift — security review prose is ahead of the reviewed tree

`docs/reviews/resource-abuse-evidence-2026-09-04.md` still declares `Reviewed tree: bb9e268...` while its prose now describes `ProcessPreauthAdmission` and all listener integrations that landed later. `docs/release-security-review-packet.md` likewise says process/source/global accounting is integrated. These are useful navigation updates but they are not an independent exact-tree review and currently overstate completeness because A-D above remain open.

Do not delete the useful links. Correct the reviewed-tree/evidence wording only after the concrete gaps are fixed and the exact repair HEAD has green CI.

## Evidence boundaries

- `IMPLEMENTATION_COMPLETE=true` remains the bounded research baseline only.
- `CANONICAL_CORPUS_V1_FROZEN=true` remains corpus-specific only.
- `RELEASE_CANDIDATE=false`, `PRODUCTION_READY=false`, `FREEZE=false`, `RELEASED=false` remain required.
- The new process admission implementation is real code with deterministic/process tests and exact-head CI. It is not yet faithful enough to D019 to close RSEC-001.
- Existing inner `PreauthBudget` remains a useful stricter per-state anti-amplification/input-response bound. Do not weaken or delete it to simplify outer accounting.
- Source/global process admission does not and must not claim control over kernel SYN backlog, provider NAT state or other resources outside the process.
- No production/public listener or security-approved service claim is allowed.
- No current VPS row is dependency-ready according to the corrected release closure map. Do not create traffic merely to use the rental window while this deterministic security lane is open.
- Protected identity material, credentials, private endpoint material and raw private diagnostics remain unread/untracked/uncommitted.

## Rolling Work Queue

This is a multi-hour rolling queue. Every dependency-satisfied item below is pre-authorized to start immediately after the previous coherent slice is tested, committed and pushed. **Do not stop after one commit, one nominal hour or one reviewer interval.** Only a new BLOCKER/HIGH that invalidates the plan, an unresolved ADR/core-architecture conflict, action beyond authorization, production impact, missing credentials/third-party authority, repository breakage, runtime/tool-budget termination or real queue exhaustion is a stop condition.

### A — Close response-admission/send-deadline semantics

**Status:** `READY_LOCAL`; highest priority with B/C/D.

Implement the existing D019 `response-send deadline = 100 ms` without changing wire bytes, response shape, Noise, Session or Carrier semantics.

Preferred shape: a typed response admission/permit carrying exact charged bytes/packet ownership and monotonic admission time, with deterministic completion/expiry. A caller must charge exact response bytes/packet before serialization/send; a send attempt completed after the deadline is not success evidence and the response ownership is terminally abandoned.

Tests: exact 99/100/101 ms boundaries as the ADR defines, no wall-clock sleeps, timeout does not emit auth/Delivery/PathValidated/ACK/readiness-equivalent evidence, cleanup is exactly once.

Run targeted tests + full gate + `git diff --check`, commit and push.

**Continue immediately to B:** yes.

### B — Integrate pending queue ownership and cleanup

**Status:** `PREAUTHORIZED_AFTER_A`.

Wire `ProcessPreauthAdmission` queue ceilings into actual pre-auth pending ownership. At minimum cover failover UDP `PendingUdpNegotiation`; audit whether any other listener owns application-level pre-auth queue entries.

Required: charge before storing; no queue entry on rejection; dequeue/release exactly once on authentication, malformed terminal rejection, idle/lifetime expiry, cancellation and process shutdown path; source/global queue max/max+1 deterministic tests; rejected queue operation produces no success evidence.

Do not add a separate queue subsystem or new numeric limits.

**Continue immediately to C:** yes.

### C — Reconcile carrier/source counting-domain persistence with D019

**Status:** `PREAUTHORIZED_AFTER_B`, but **stop only if the ADR truly requires an amendment/new numeric retention policy**.

Make source-key semantics explicit rather than accidental:

- carrier dimension must be represented or a documented reviewed reason must prove the current projection is intentionally stricter/equivalent;
- retry/reconnect/carrier transition must not gain fresh per-source budget inside the D019 source accounting lifetime;
- source cleanup must not erase counters in a way that violates the no-reset rule;
- unknown/unusable identity must remain one bounded shared bucket when applicable;
- source-accounting storage itself must remain bounded.

Add deterministic reconnect/retry and UDP/TCP carrier-transition tests. If literal D019 persistence and bounded source-table memory cannot both be met without a new policy value, record the exact conflict and stop this slice for an ADR/maintainer decision; do not improvise a magic retention count/time.

**Continue immediately to D if resolved without ADR conflict:** yes.

### D — Make rejection terminal and non-revivable

**Status:** `PREAUTHORIZED_AFTER_C`.

Strengthen the reusable controller/ticket contract so any exhausted/saturated/unmeasurable/timed-out charge cannot later succeed on the same logical pre-auth state merely because the one-second window rolls over or the caller retries.

Required regressions:

- global input-window rejection -> same state cannot succeed after rollover;
- global response-window rejection -> same state cannot send after rollover;
- queue rejection -> same rejected ownership cannot later enqueue;
- inner-budget rejection and outer-budget rejection remain cross-layer atomic;
- cleanup/release remains exactly once and cannot reopen an exhausted source domain contrary to C.

**Continue immediately to E:** yes.

### E — Audit parser/work and memory reservation charge ordering

**Status:** `PREAUTHORIZED_AFTER_D`.

Do a concrete call-site audit rather than a generic rewrite.

For each real responder pre-auth path, map:

- received bytes/packet charge point;
- parser/work reservation before parse/work;
- state memory reservation before owned allocation;
- queue ownership charge before enqueue;
- response charge + deadline before send;
- terminal release.

The current `64` / `4096` work-unit reservations may remain if they are conservative bounded reservations that dominate the actual parser work and are documented/tested as such; do not pretend they are measured CPU cycles. If an input-controlled loop can execute more work than was reserved, fix that exact seam.

Produce a machine-checkable/static inventory or deterministic test so a new externally reachable responder cannot silently omit admission.

**Continue immediately to F:** yes.

### F — D019 adversarial boundary and evidence-barrier closure

**Status:** `PREAUTHORIZED_AFTER_E`.

Run/extend deterministic adversarial coverage for the complete D019 matrix:

- source/global concurrency max/max+1;
- source lifetime bytes/packets/work;
- global input/work one-second windows;
- per-packet work;
- state/global memory;
- source/global pending queue;
- source/global response + inner 3x anti-amplification;
- idle 1 s / lifetime 5 s / response-send 100 ms via injectable time;
- overflow;
- retry/reconnect/carrier transition persistence;
- cancellation/timeout/double cleanup;
- no Delivery/PathValidated/ACK/readiness/authz-equivalent evidence on pre-auth rejection;
- secret-safe diagnostics.

Do not use VPS/high-load tests to substitute for deterministic counter correctness.

Push after the full local repository gate.

**Continue immediately to G:** yes, but exact-head CI must be green before declaring RSEC-001 closed.

### G — Fresh exact-tree security re-review and evidence correction

**Status:** `PREAUTHORIZED_AFTER_F`.

Wait only for exact repair HEAD CI required for the security conclusion. Independently re-read the exact code and tests; then update `docs/reviews/resource-abuse-evidence-2026-09-04.md`, `docs/release-security-review-packet.md`, `docs/status.md` and closure navigation so the reviewed tree and claims match reality.

RSEC-001 may be reduced/closed as an implementation finding only when A-F are actually satisfied. Even then, independent external/two-person review remains a separate release gate. Never promote RC/production/freeze/release automatically.

**Continue immediately to H if no new HIGH/BLOCKER:** yes.

### H — Compatibility / freeze-boundary review

**Status:** `READY_LOCAL_AFTER_G`.

Audit corpus-v1 content-addressed freeze vs global protocol non-freeze, current/current negotiation, unsupported/future rejection, downgrade/transcript binding into Noise, resume/version binding and replay boundary, and docs that might imply corpus freeze == protocol/release freeze.

Only add a deterministic regression for a real gap. Do not reopen frozen corpus bytes without a correctness defect.

**Continue immediately to I:** yes.

### I — Package/operator and evidence-provenance integrity review

**Status:** `READY_LOCAL_AFTER_H`.

Verify x86_64 package/build identity; install -> readiness/smoke -> upgrade -> rollback; retained external state without reading protected identity material; shutdown/listener/temp cleanup; tracked evidence manifests using canonical Git-blob checks; exact-head CI references; stale links/hashes in the release packet.

Do not rerun VPS/package work merely for freshness if the bounded question is already answered. Fix only concrete defects.

**Continue immediately to J:** yes.

### J — Reclassify remaining release opportunities and reconsider VPS

**Status:** `READY_LOCAL_AFTER_I`.

Re-evaluate every remaining `OPEN_READY` row against actual missing evidence after the security work:

- already answered bounded question -> `ALREADY_SUFFICIENT_FOR_BOUNDED_QUESTION`;
- executable specific missing assertion -> keep `OPEN_READY` with exact evidence/action/dependencies/scope;
- blocked dependency/governance/environment/implementation -> classify truthfully.

Then reconsider the live matrix. Execute a VPS row **only** if a genuine dependency-ready row now exists and it answers a declared missing release question under standing authorization. Otherwise record `READY_LIVE: none` and do not manufacture traffic.

No unchanged retry of repeated/periodic/HY2 closed current lines.

## Completion gates

The RSEC-001 implementation lane is complete only when all are true:

- D019 response-send deadline is explicit and enforced;
- runtime pending pre-auth queue ownership is actually charged/released;
- carrier/source counting-domain persistence is reconciled with the ADR without unbounded source-accounting state;
- rejected state cannot revive after rollover/retry;
- bytes/work/memory/queue/response charge ordering is audited across every externally reachable responder;
- adversarial boundary/overflow/timeout/cleanup/evidence-barrier tests pass;
- exact-head local gate and GitHub CI are green;
- reviewer evidence prose names the exact reviewed tree and no longer gets ahead of implementation;
- governance/release flags remain unchanged.

The broader rolling queue remains active through H-J unless a real stop condition occurs.

## Do not expand into

- public/production listener deployment;
- new numeric D019 ceilings without an explicit ADR decision;
- protocol/wire/Noise/Session/Carrier redesign unrelated to the concrete admission gaps;
- VPS load testing as a substitute for deterministic security accounting;
- unchanged repeated/periodic/HY2 reruns;
- NAT/migration-back/key-update/PMTUD live claims without a real executable seam;
- IPv6 claims without an owned end-to-end environment;
- 0-RTT, enabled FEC, striping/aggregation or exotic carriers without an observed-problem gate;
- third-party targets, scanning or production network changes;
- reading/committing protected identity or credentials.

## Questions requiring maintainer decision

None yet. Slice C must surface a maintainer/ADR decision only if the literal D019 no-reset source semantics cannot be implemented with bounded source-accounting storage without introducing a new policy value.
