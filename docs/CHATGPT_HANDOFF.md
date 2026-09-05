# Nekomusume ChatGPT Handoff

Checked at: 2026-09-05 17:59 Asia/Shanghai
Repository main HEAD reviewed: `ea0ca8e4ee4ac571247a9130f3db7b593c689152`
Work-branch HEAD additionally reviewed: `d271a99a2ab26abbcb146c411ba0fde697395abe`
Previous checked implementation HEAD: `fdbcae78eecf8423306dfcf8ba0f66a533317d78`
Previous reviewer handoff commit: `ea0ca8e4ee4ac571247a9130f3db7b593c689152`

## What changed

No new implementation landed on `main` after the previous reviewer handoff. The external coding branch did advance once:

- `d271a99` — **partial E2 responder-inventory hardening; no protocol/wire/Noise/Session/Carrier semantic change.** It adds explicit `queue_reserve_anchor`, `pending_store_anchor`, and `pending_cancel_anchor` fields for the two failover-UDP pending-owner inventory entries and makes the checker require those fields/anchors for any `pending_owner` responder.

Exact-head CI is green on both current tips:

- `main` exact `ea0ca8e` Rust CI run `33951530698` — `success`;
- `work/continue-20260904` exact `d271a99` Rust CI run `33951594506` — `success`.

The refs are currently **diverged by one commit each** from merge base `fdbcae7`:

- `main` contains the reviewer handoff `ea0ca8e`;
- the work branch contains coding commit `d271a99`.

This is a normal coordination divergence, not a reason to discard either side. Before the next implementation slice, the coding branch should normally merge/fetch-integrate current `origin/main` without force-pushing, preserving `d271a99` and the reviewer-owned handoff.

`d271a99` is useful and should be retained, but it does **not** close E2 and must not bypass E1A. Its new pending-owner checks prove that named reservation/store/cancel anchors exist somewhere in the bounded inventory region; they do not by themselves prove all required reserve-before-store / terminal-cleanup ordering, and they do not repair the still-open staged TCP charge-order defect.

## Review verdict

**CONTINUE_WITH_REQUIRED_FIXES — retain `d271a99` as partial E2 infrastructure, integrate the reviewer handoff into the work branch, then execute E1A before completing E2. Do not wait for another reviewer interval after integration.**

No administrator action is required. Do not spend VPS time on this deterministic D019 accounting defect. Existing standing VPS authorization remains valid for later dependency-ready real-network evidence.

## Reviewer findings

### RSEC-001A1 — CLOSED at `b775f1e`

Keep the accepted absolute 100 ms response-I/O deadline intact:

- one absolute deadline spans partial TCP writes/flush;
- zero remaining budget fails closed;
- UDP/TCP send helpers bound actual socket I/O and restore prior timeout state after successful completion;
- this proves process-side bounded send-attempt semantics only, not remote receipt/provider/kernel delivery time.

Do not reopen this into generic async-I/O work absent a concrete defect.

### RSEC-001B1 — CLOSED at `f98161b`

Keep the accepted one-owner queue/process-expiry direction intact. Queue reservation, process expiry, stale-permit invalidation, source/global max/max+1 and exactly-once release tests are sufficient for the current bounded queue primitive. This remains deterministic accounting evidence only.

### RSEC-001D1 — CLOSED at `884621c`

Keep structural rejection terminalization intact. Inner `PreauthBudget` failure poisons the outer logical state; state-associated process operations terminalize on returned failure; response-I/O abandonment remains terminal; representative clock/arithmetic/inner/outer rejection paths cannot revive the same logical ticket.

Runtime cancellation/evidence-barrier behavior remains part of E/F rather than being implied by the terminal bit alone.

### RSEC-001E1A — HIGH/open — TCP framed input is still charged after attacker-controlled framing work

D019 requires applicable input/work accounting to be charged before the protected parse/work. The current real TCP responder paths still violate that order:

- ordinary TCP probe reads a complete framed hello before `charge_input`;
- failover TCP has the same complete-frame-read then charge order;
- periodic TCP obtains a full frame before charge;
- multistream TCP reads the length, allocates the attacker-declared payload, and reads the body before the later handshake charge.

The existing 16 KiB state-memory reservation usefully bounds state-owned allocation but is not a substitute for source/global input-byte, packet/record or parser/work charging.

**Required repair:** implement one reusable staged/pre-auth-aware TCP frame-receive contract and migrate every real TCP responder handshake to it.

Required semantic order:

1. admit source/state before pre-auth framing work;
2. bounded raw read of the fixed four-byte length header;
3. charge header bytes, one logical record ownership, and conservative header-parse work before interpreting attacker-controlled length;
4. parse/check the declared length;
5. reserve/charge declared payload bytes and conservative protected work before allocation/body read;
6. truncated/EOF/timed-out body remains conservatively charged and terminal; do not refund attacker-consumed budget;
7. only then allow negotiation/Noise parsing under the already-reserved budget.

One TCP frame must remain one D019 input record/packet even though byte/work charging is staged. Do not introduce new numeric ceilings or alter wire format.

### RSEC-001E1B — MEDIUM/open — responder inventory guard is improving but still not semantic closure

`fdbcae7` established the inventory; `d271a99` adds explicit pending-owner queue/store/cancel anchors and is accepted as useful partial hardening.

Remaining gaps after E1A:

- every TCP responder must anchor the repaired staged charged-frame primitive before negotiation/Noise parse;
- every UDP responder must anchor charge-after-bounded-raw-receive and before protocol parsing;
- pending-owner checks must establish reserve-before-store and terminal dequeue/cancel/expiry behavior rather than only the existence of substrings;
- rejection/timeout/malformed/I/O paths must not reach auth/readiness/Delivery/PathValidated/ACK/authz-equivalent success anchors;
- the expected responder surface set must stay explicit so adding a new externally reachable listener requires a new semantic inventory entry.

Do not weaken the guard merely to make current source pass.

### RSEC-001C — ADR checkpoint remains isolated, not yet an administrator blocker

Current source projection still needs an explicit bounded carrier discriminator so TCP and UDP pre-auth source domains are not accidental aliases.

The policy tension in D019 remains:

- per-source input/packet/work/response budgets are described in state-lifetime terms;
- counters are also stated not to reset on retry, reconnect, carrier change, identity change or error.

Current terminal source removal can let reconnect obtain fresh per-source accounting. Retaining arbitrary terminal sources forever makes the source map unbounded, while the reviewed ADR provides no retention TTL/history/LRU/eviction bound.

Do not invent convenience numbers. Complete E1A/E2, implement only the noncontroversial carrier/source projection in C1, then re-read the exact policy. If bounded memory and literal no-reset semantics still conflict, C2 should produce a compact ADR amendment request and stop only that policy-dependent lane.

## Evidence boundaries

- `IMPLEMENTATION_COMPLETE=true` remains a bounded research-baseline flag only.
- `CANONICAL_CORPUS_V1_FROZEN=true` remains corpus-specific only.
- `RELEASE_CANDIDATE=false`, `PRODUCTION_READY=false`, `FREEZE=false`, `RELEASED=false` remain required.
- Exact `d271a99` has green CI and improves static pending-owner inventory assertions. It does not repair TCP staged charge ordering, does not close E2, and adds no WAN/VPS evidence.
- Existing inner `PreauthBudget` remains the stricter per-state input/anti-amplification bound; do not weaken it to simplify staged framing.
- Existing 16 KiB per-state memory reservation remains useful but is not input/work charge evidence.
- Process admission does not claim control over kernel SYN backlog, provider NAT state or resources outside the process.
- Standing VPS authorization still covers future dependency-ready self-owned TCP/UDP work; it does not justify load/WAN testing as a substitute for deterministic D019 correctness.
- Historical WAN/HY2/failover/periodic positive and negative evidence remains immutable at its exact commit boundary.
- Protected identity material, credentials, private endpoint material and raw private diagnostics remain unread/untracked/uncommitted.

## Rolling Work Queue

This is a rolling multi-hour queue. Finish one coherent slice -> targeted/full gates -> commit -> push -> immediately consume the next dependency-satisfied slice. Do not stop for a reviewer interval. Only a new HIGH/BLOCKER that invalidates downstream work, a genuine ADR/core-architecture conflict, action beyond authorization, production impact, missing credentials/third-party authority, repository breakage, runtime/tool-budget termination or real queue exhaustion is a stop condition.

### Q0 — Reconcile the one-commit branch divergence

**Status:** `READY_LOCAL`; coordination prerequisite, not a design gate.

On `work/continue-20260904`, fetch current `origin/main` and integrate it normally so the branch contains both:

- reviewer handoff lineage from `ea0ca8e` and later main reviewer commits;
- coding commit `d271a99`.

Prefer an ordinary merge/fast-forward-safe reconciliation. Do not force-push and do not discard `d271a99`. Resolve only genuine textual conflicts; reviewer-owned `docs/CHATGPT_HANDOFF.md` remains read-only to the coding agent.

If this integration has already happened by the time work resumes, skip Q0 and continue directly to E1A.

**Continue immediately to E1A:** yes.

### E1A — Repair staged TCP pre-auth input/work charging

**Status:** `READY_LOCAL`; immediate security-priority slice.

Implement one reusable bounded staged TCP pre-auth frame-receive path and migrate all real TCP responder handshakes:

- ordinary TCP probe;
- periodic TCP;
- multistream TCP;
- failover TCP.

Required invariants:

- state admitted before framing work;
- fixed header raw I/O remains bounded;
- header bytes + one record/packet ownership + conservative header-parse work are charged before attacker length interpretation;
- declared payload bytes/work are reserved before payload allocation/read;
- truncated/EOF/timed-out body remains conservatively charged and terminal rather than refunded;
- one TCP frame counts as one D019 input record/packet despite staged header/body accounting;
- max-frame rejection, backwards/unusable clock, arithmetic overflow and admission failure are terminal and emit no auth/readiness/session/path/delivery/ACK-equivalent success evidence;
- no new numeric ceilings and no wire-format change.

Add deterministic tests for fragmented header/body, oversize length, exact/max+1 accounting, truncated body after reservation, timeout after reservation, and no double packet count.

Run targeted tests + full `scripts/check.sh` + `git diff --check`; fuzz only if production untrusted-input parser/wire behavior materially changes. Commit and push.

**Continue immediately to E2:** yes.

### E2 — Complete semantic responder admission/cleanup/evidence coverage

**Status:** `PREAUTHORIZED_AFTER_E1A`; `d271a99` is partial progress already retained.

Update `docs/preauth-responder-inventory.v1.json` and `scripts/check-preauth-responder-inventory.py` on top of the repaired implementation.

Required:

- every TCP responder anchors the staged charged-frame primitive before negotiation/Noise parse;
- every UDP responder anchors charge-after-raw-receive/before-protocol-parse;
- pending UDP ownership proves queue-reserve-before-store and exactly-once dequeue/cancel/expiry invalidation;
- rejection/timeout/malformed/I/O paths cannot reach auth/readiness/Delivery/PathValidated/ACK/authz-equivalent success anchors;
- success cleanup, immediate rejection cleanup and deterministic process expiry are distinguished explicitly;
- conservative `64` / `4096` work reservations remain documented as accounting units only where they dominate bounded protected work;
- expected responder surface set remains explicit; new listeners require semantic inventory entries.

Fix a real uncovered call-site seam rather than documenting around it. Full gate, commit, push.

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

Independently re-read exact implementation/tests and reconcile:

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

Re-evaluate every remaining release/evidence row:

- bounded question already answered -> `ALREADY_SUFFICIENT_FOR_BOUNDED_QUESTION`;
- executable specific missing assertion with dependencies satisfied -> `OPEN_READY` with exact `evidence_needed`, `next_action`, `requires`, `execution_scope`;
- implementation/environment/governance/review dependency absent -> classify exact blocker;
- never use generic `need WAN authorization` for work already covered by standing authorization.

Then reconsider the live matrix. Execute a VPS-only row only if a genuine dependency-ready row exists and it answers a declared missing release question. Otherwise record `READY_LIVE: none` and do not manufacture traffic.

No unchanged retry of already-sufficient repeated/periodic/HY2 lines.

## Completion gates

The D019/RSEC-001 implementation lane is complete only when all are true:

- A1 absolute response-I/O deadline remains green and bounded;
- B1 queue ownership/expiry verification remains green;
- D1 structural rejection terminalization remains green;
- every real responder has machine-checkable admission, staged input/work charge, rejection, evidence-barrier and cleanup ordering;
- carrier/source projection is explicit and bounded;
- terminal-source persistence semantics are resolved by reviewed policy without unbounded source-accounting state or invented convenience limits;
- complete deterministic adversarial/overflow/timeout/cleanup matrix passes;
- exact-head local gate and GitHub CI are green;
- reviewer security/evidence prose names the exact reviewed tree and does not outrun implementation;
- governance/release flags remain unchanged.

The broader rolling queue remains active through H/I/J unless a real stop condition occurs.

## Do not expand into

- public or production listener deployment;
- new numeric D019 ceilings, retention TTLs, LRU/history sizes or eviction rules without reviewed ADR policy;
- protocol/wire/Noise/Session/Carrier redesign unrelated to concrete admission findings;
- VPS load testing as a substitute for deterministic security accounting;
- renewed HY2 work without a changed hypothesis and declared missing comparison question;
- speculative FEC/0-RTT/exotic-carrier work;
- reading, hashing, copying, modifying or committing protected identity/secrets/private endpoint material;
- release/RC/freeze/production promotion.

## Questions requiring maintainer decision

None at this review point.

C2 may become a genuine ADR decision after C1 and the responder audit are complete. If so, record the exact policy conflict and continue independent H/I/J work rather than blocking the whole project or inventing a numeric retention rule.
