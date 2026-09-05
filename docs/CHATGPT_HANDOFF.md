# Nekomusume ChatGPT Handoff

Checked at: 2026-09-05 15:01 Asia/Shanghai
Repository HEAD reviewed: `fdbcae78eecf8423306dfcf8ba0f66a533317d78`
Previous checked implementation HEAD: `884621c1e8a0e23fc9e0b2312bb3e1a482687cf2`
Previous reviewer handoff commit: `2d8bc96efb4228e1514a6a23a9101973564c6267`

## What changed

One coding-agent commit landed after the previous reviewer handoff:

- `fdbcae7` — **E1 responder inventory/check infrastructure; no wire/Noise/Session/Carrier/release semantic change.** It adds `docs/preauth-responder-inventory.v1.json`, `scripts/check-preauth-responder-inventory.py`, wires the checker into `scripts/check.sh`, enumerates seven current responder surfaces, and records call-site/order anchors for ordinary TCP/UDP, periodic TCP, multistream TCP, failover TCP and the two failover-UDP pending/new paths.

At review time exact `fdbcae78eecf8423306dfcf8ba0f66a533317d78` is green on both refs:

- main Rust CI run `33949146568` — `success`;
- work-branch Rust CI run `33949152927` — `success`.

This is exact-head repository CI evidence, not security approval.

The new inventory is useful because it exposes a concrete D019 ordering defect that E1 was supposed to discover rather than merely document: current TCP pre-auth framed readers still parse the attacker-controlled length and allocate/read the payload before the corresponding exact input/work charge is committed. The new checker currently preserves or fails to observe that wrong order instead of rejecting it.

## Review verdict

**CONTINUE_WITH_REQUIRED_FIXES — accept `fdbcae7` as useful E1 inventory infrastructure, but do not close RSEC-001E. Repair staged TCP pre-auth framing/charge ordering first, then strengthen the inventory guard, continue C1/C2/F/G/H/I/J, and do not wait for another reviewer interval.**

No administrator action is required at this review point. Do not spend VPS time on this deterministic accounting defect. The release-evidence ledger still has no reason to manufacture live traffic while the D019 security lane is open.

## Reviewer findings

### RSEC-001A1 — CLOSED at `b775f1e`

Keep the accepted absolute 100 ms response-I/O deadline intact:

- one absolute deadline spans partial TCP writes/flush;
- zero remaining budget fails closed;
- UDP/TCP send helpers bound actual socket I/O and restore prior timeout state after successful completion;
- this proves only process-side bounded send-attempt semantics, not remote receipt or provider/kernel delivery time.

Do not reopen this into generic async-I/O work absent a concrete defect.

### RSEC-001B1 — CLOSED at `f98161b`

Keep the accepted one-owner queue/process-expiry direction intact. Queue reservation, process expiry, stale-permit invalidation, source/global max/max+1 and exactly-once release tests are sufficient for the current bounded queue primitive. This remains deterministic accounting evidence only, not public capacity evidence.

### RSEC-001D1 — CLOSED at `884621c`

Keep structural rejection terminalization intact. Inner `PreauthBudget` failure poisons the outer logical state; state-associated process operations terminalize on any returned failure; response I/O abandonment remains terminal; representative clock/arithmetic/inner/outer rejection paths cannot revive the same ticket.

Runtime cancellation/evidence-barrier behavior still belongs to E and is not implied merely by `rejected=true`.

### RSEC-001E1A — HIGH/open — TCP framed input is still charged after attacker-controlled framing work

D019 requires applicable input/work accounting to be charged before the protected parse/work. The current real TCP responder paths do not satisfy that ordering.

Concrete exact-head evidence:

- ordinary TCP probe: `read_frame(&mut s, MAX_NEGOTIATION_FRAME)` completes before `.charge_input(&mut admission, hello.len() + 4, 64)`;
- failover TCP has the same `read_frame(...) -> charge_input(...)` order;
- periodic TCP `frame_or_fail` / `FramedReader::read_until` completes a frame before `charge_input`;
- multistream TCP `frame_read` reads the 4-byte length, converts it, allocates `vec![0; n]`, reads the full body, and only then `server_handshake` calls `charge_input`.

`FramedReader` itself confirms the problem: after four header bytes it parses `u32::from_be_bytes`, checks the attacker-declared length, allocates the payload vector and reads payload bytes before the caller's later admission charge.

The 16 KiB state-memory reservation usefully bounds state-owned allocation and should remain, but it does not satisfy source/global input-byte, packet/record or parser/work charge ordering.

The new `fdbcae7` inventory checker is therefore not yet an E2 guard:

- ordinary/failover TCP inventory explicitly records `read_frame` before `charge_input`, so the checker currently enforces the known-wrong order;
- periodic/multistream `ordered` lists begin at `charge_input` and omit the preceding frame-read anchor, so the checker can pass without proving charge-before-framing work.

**Required repair:** add a small staged/pre-auth-aware TCP frame-receive contract. Do not add new numeric ceilings. The implementation must make it mechanically impossible to parse/allocate an attacker-declared TCP pre-auth record body without first reserving the existing D019 accounting for that record.

A valid shape is:

1. admit the source/state before pre-auth framing work;
2. read the fixed 4-byte header as bounded raw I/O;
3. charge the received header bytes plus the bounded header-parse work and the single record/packet ownership **before** interpreting the attacker-controlled length;
4. parse/check the declared length;
5. reserve/charge the declared payload bytes and conservative bounded parse/work units **before** allocating or reading the payload body;
6. if the peer truncates after reservation, keep the conservative charge and fail closed; do not roll back attacker-caused consumed budget merely because the body was incomplete;
7. only after the full body is received may negotiation/Noise parsing proceed under the already-reserved work budget.

The exact API shape is flexible, but it must not double-count one TCP record as two D019 input packets merely because header and payload charging are staged. A typed record-input permit/reservation or equivalent atomic staged API is preferable to ad-hoc caller arithmetic.

UDP `recv_from` into an already bounded/preallocated datagram buffer may continue to charge immediately after raw receive and before negotiation/Noise parsing; do not redesign UDP merely to match the TCP helper.

### RSEC-001E1B — MEDIUM/open — inventory completeness/order guard needs semantic assertions, not only string counts

`fdbcae7` is a good first inventory, but the current guard relies on fixed substring order plus `admits == 6`. That is useful drift detection, not proof that a new externally reachable responder cannot bypass admission or that every protected parse is preceded by accounting.

After E1A is repaired, strengthen the manifest/check so each TCP responder names the staged charged-frame primitive/anchors and each pending-owner path names reserve-before-store plus terminal cleanup. Keep the explicit expected responder set; do not replace it with only a raw `.admit(peer)` count.

If code search discovers an additional real listener, add it and its semantics rather than changing the expected count to silence the checker.

### RSEC-001C — ADR checkpoint remains isolated, not yet an administrator blocker

Current source projection still needs an explicit bounded carrier discriminator so TCP and UDP pre-auth source domains are not accidental aliases.

A separate policy tension remains in D019:

- per-source input/packet/work/response budgets are described as state-lifetime scoped; while
- counters are also stated not to reset on retry, reconnect, carrier change, identity change or error.

Current terminal release can remove the source usage entry and allow a reconnect to regain fresh per-source budget. Retaining every terminal source forever would make the source map unbounded, while the reviewed ADR supplies no terminal-source TTL/history/LRU/eviction bound.

Do not invent convenience numbers. Finish E1A/E2, implement the noncontroversial carrier/source projection in C1, then re-read the exact policy. If literal no-reset plus bounded source-accounting memory still conflicts, C2 should produce a compact ADR amendment request and stop only that policy-dependent lane.

## Evidence boundaries

- `IMPLEMENTATION_COMPLETE=true` remains a bounded research-baseline flag only.
- `CANONICAL_CORPUS_V1_FROZEN=true` remains corpus-specific only.
- `RELEASE_CANDIDATE=false`, `PRODUCTION_READY=false`, `FREEZE=false`, `RELEASED=false` remain required.
- Exact `fdbcae7` adds a static responder inventory/check with green CI. It does not close D019 responder ordering and adds no WAN/VPS behavior evidence.
- Existing inner `PreauthBudget` remains the stricter per-state input/anti-amplification bound; do not weaken it to simplify staged framing.
- Existing 16 KiB per-state memory reservation remains useful and should dominate bounded pre-auth frame allocations, but memory reservation is not a substitute for input/work charge ordering.
- Process admission does not claim control over kernel SYN backlog, provider NAT state or resources outside the process.
- Standing VPS authorization remains valid for future dependency-ready self-owned TCP/UDP evidence. It does not justify using load/WAN tests to substitute for deterministic D019 accounting correctness.
- Historical WAN/HY2/failover/periodic positive and negative evidence remains immutable at its exact commit boundary.
- Protected identity material, credentials, private endpoint material and raw private diagnostics remain unread/untracked/uncommitted.

## Rolling Work Queue

This remains a rolling multi-hour queue. Finish one coherent slice -> targeted/full gates -> commit -> push -> immediately consume the next dependency-satisfied slice. Do not stop for a reviewer interval. Only a new HIGH/BLOCKER that invalidates downstream work, a genuine ADR/core-architecture conflict, action beyond authorization, production impact, missing credentials/third-party authority, repository breakage, runtime/tool-budget termination or real queue exhaustion is a stop condition.

### E1A — Repair staged TCP pre-auth input/work charging

**Status:** `READY_LOCAL`; immediate security-priority slice.

Implement one reusable bounded staged TCP pre-auth frame-receive path and migrate all real TCP responder handshakes to it:

- ordinary TCP probe;
- periodic TCP;
- multistream TCP;
- failover TCP.

Required invariants:

- state admitted before framing work;
- fixed header raw I/O remains bounded;
- header bytes + one record/packet ownership + conservative header-parse work are charged before attacker length interpretation;
- declared payload bytes/work are reserved before payload allocation/read;
- a truncated/EOF/timed-out body remains conservatively charged and terminal rather than rolling the attacker's budget back;
- one TCP frame counts as one D019 input record/packet despite staged header/body accounting;
- max-frame rejection, backwards/unusable clock, arithmetic overflow and admission failure are terminal and emit no auth/readiness/session/path/delivery/ACK-equivalent success evidence;
- no new numeric ceilings and no wire-format change.

Add deterministic tests for fragmented header/body, oversize length, exact/max+1 accounting, truncated body after reservation, timeout after reservation, and no double packet count.

Run targeted tests + full `scripts/check.sh` + `git diff --check`; fuzz only if production untrusted-input parser/wire behavior materially changes. Commit and push.

**Continue immediately to E2:** yes.

### E2 — Make responder admission/cleanup/evidence coverage mechanically non-optional

**Status:** `PREAUTHORIZED_AFTER_E1A`.

Update `docs/preauth-responder-inventory.v1.json` and `scripts/check-preauth-responder-inventory.py` so the guard proves the repaired semantics rather than preserving old call order.

Required:

- every TCP responder anchors the staged charged-frame primitive before negotiation/Noise parse;
- every UDP responder anchors charge-after-raw-receive/before-protocol-parse;
- pending UDP path proves queue-reserve-before-store and exactly-once dequeue/cancel/expiry invalidation;
- rejection/timeout/malformed/I/O paths cannot reach auth/readiness/Delivery/PathValidated/ACK/authz-equivalent success anchors;
- success cleanup and caller-owned immediate rejection cleanup vs deterministic process expiry are distinguished explicitly;
- conservative `64` / `4096` work accounting remains documented as accounting units only where it dominates the bounded protected work;
- expected responder surface set remains explicit; a new listener must add an inventory entry rather than merely changing a count.

Fix any real uncovered call-site seam rather than documenting around it. Full gate, commit, push.

**Continue immediately to C1:** yes.

### C1 — Make carrier/source projection explicit and bounded

**Status:** `PREAUTHORIZED_AFTER_E2`.

Implement only the noncontroversial source-key portion:

- explicit bounded carrier discriminator at least distinguishing current TCP and UDP pre-auth domains;
- family/address/port remain represented without textual/raw logging;
- deterministic non-collision tests across carrier/family/address/port combinations;
- every current call site supplies the actual received carrier class;
- one bounded unknown/unusable-source bucket only if a real current call site needs it;
- no new terminal-source retention duration, LRU size, history count or eviction parameter.

Run targeted/full gate, commit, push.

**Continue immediately to C2:** yes.

### C2 — Resolve terminal-source persistence semantics or produce ADR amendment request

**Status:** `ADR_CHECKPOINT_AFTER_C1`.

Re-read `docs/adr/m1-g0-preauth-resource-budget.md` and adjacent reviewed decisions against the exact responder inventory.

Do not retain arbitrary terminal sources forever and do not invent TTL/LRU/history counts. If the reviewed text still cannot simultaneously satisfy no-reset semantics and bounded source-accounting memory, write a compact ADR amendment request with:

- exact conflicting clauses;
- attacker/resource rationale;
- current implementation consequences;
- feasible policy shapes without convenience numbers;
- tests/evidence each shape would require;
- independent work that can continue safely.

Stop only this policy-dependent lane if reviewer/maintainer choice is genuinely required. Do not falsely mark D019 complete.

**If C2 becomes external-wait:** continue H then I; J may locally reclassify but must not claim D019 closure.

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
- idle 1 s / lifetime 5 s / response-send 100 ms with deterministic time/I/O controls;
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

RSEC-001 may close as an implementation finding only when E1A/E2/C1/C2/F are actually satisfied and exact-head CI is green. Independent external/two-person security review remains a separate release gate. Never promote RC/production/freeze/release automatically.

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

Then reconsider the live matrix. Execute a VPS-only row only if a genuine dependency-ready row exists and it answers a declared missing release question. Otherwise record `READY_LIVE: none` and do not manufacture traffic.

No unchanged retry of already-sufficient repeated/periodic/HY2 lines.

## Completion gates

The current D019/RSEC-001 implementation lane is complete only when all are true:

- A1 absolute response-I/O deadline remains green and bounded;
- B1 queue ownership/expiry verification remains green;
- D1 structural rejection terminalization remains green;
- every real responder has machine-checkable admission, staged input/work charge, rejection, evidence-barrier and cleanup ordering;
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
- renewed HY2 work without a changed hypothesis and declared missing comparison question;
- speculative FEC/0-RTT/exotic-carrier work;
- reading, hashing, copying, modifying or committing protected identity/secrets/private endpoint material;
- release/RC/freeze/production promotion.

## Questions requiring maintainer decision

None at this review point.

C2 may become a genuine ADR decision after C1 and the responder audit are complete. If so, record the exact policy conflict and continue independent H/I/J work rather than blocking the whole project or inventing a numeric retention rule.
