# Nekomusume ChatGPT Handoff

Checked at: 2026-09-06 16:04 Asia/Shanghai
Repository main HEAD reviewed: `75261c075fbf6eca6a71bf54eacf02a1ebfd4d6d`
Work-branch HEAD additionally reviewed: `d271a99a2ab26abbcb146c411ba0fde697395abe`
Previous checked reviewer handoff: `75261c075fbf6eca6a71bf54eacf02a1ebfd4d6d`

## What changed

No new coding-agent commit has landed after the previous reviewer handoff. Exact `75261c0` CI run `33959648022` is green. The work branch still retains `d271a99` and has not been force-rewritten.

This would normally justify leaving the handoff unchanged. However, the coding lane has now remained stalled on the same HIGH/open E1A issue across multiple execution wakeups: the agent correctly identified that the existing accounting API couples byte/work charging to packet ownership and therefore cannot express staged header/body charging while counting one TCP frame as exactly one D019 input record.

That repeated no-progress state is itself a coordination signal. The prior handoff specified the required ordering but left too much API-shape freedom, allowing the coding agent to classify E1A as “blocked by missing accounting API” rather than implementing the missing bounded primitive. This handoff therefore narrows E1A into an implementation contract. No administrator or architecture decision is required for this API split.

## Review verdict

**CONTINUE_WITH_REQUIRED_FIXES — E1A remains the sole immediate HIGH/open implementation issue, but it is not an external blocker. Implement a one-logical-record staged input reservation API, migrate all four TCP responders, then continue E2/C1/C2/F/G/H/I/J without waiting for another reviewer interval.**

No VPS work is useful for E1A. Do not substitute traffic/load tests for deterministic charge-order correctness.

## Evidence boundaries

- `IMPLEMENTATION_COMPLETE=true` remains a bounded research-baseline flag only.
- `CANONICAL_CORPUS_V1_FROZEN=true` remains corpus-specific only.
- `RELEASE_CANDIDATE=false`, `PRODUCTION_READY=false`, `FREEZE=false`, `RELEASED=false` remain required.
- A1 absolute response-I/O deadline, B1 queue ownership/expiry, and D1 terminal rejection remain accepted closed subfindings unless a new concrete regression appears.
- Current `FramedReader` still reads the four-byte length header, interprets the attacker-controlled length, allocates `vec![0; len]`, and reads the payload before admission accounting can be staged. The fix must move pre-auth accounting into a staged frame-read contract rather than merely adding another later `charge_input` call.
- Current `PreauthBudget::charge_input(bytes)` and `ProcessPreauthAdmission::charge_input(id, bytes, work, now)` both imply one new packet/record per call. Reusing them twice for header and payload would double-count one TCP frame and is not acceptable.
- Existing 16 KiB per-state memory reservation remains useful but is not input/work accounting and must not be cited as E1A closure.
- Historical WAN/HY2/failover/periodic evidence remains immutable at its exact commit boundaries.
- Protected identity, credentials, private endpoint material and raw private diagnostics remain unread/untracked/uncommitted.

## Reviewer findings

### RSEC-001E1A — HIGH/open — exact implementation contract

The missing primitive is a **one-logical-input-record staged reservation**, not a new protocol feature.

The invariant is:

```text
one TCP frame
= one D019 input packet/record ownership
+ staged cumulative byte/work charging
```

Header and payload are two accounting stages of the same record, not two records.

### Required API semantics

Implement an equivalent of the following shape; names may differ, semantics may not:

```text
begin_input_record(ticket, header_bytes, header_work)
    -> InputRecordReservation

extend_input_record(ticket, reservation, additional_bytes, additional_work)
    -> ()

complete_input_record(ticket, reservation)
    -> ()

abandon/reject input record
    -> logical pre-auth state terminal; prior charges retained
```

A typed permit/reservation is preferred because it structurally prevents accidental double packet ownership.

#### Inner per-state budget

`PreauthBudget` needs staged semantics equivalent to:

```text
begin_input_record(bytes)
    increments input_packets exactly once
    increments input_bytes by bytes

extend_input_record(bytes)
    increments input_bytes only
    does NOT increment input_packets
```

Do not weaken existing per-state byte/packet limits or anti-amplification behavior. Response packet allowance must continue to observe one charged input packet for one TCP frame.

#### Process/source/global budget

`ProcessPreauthAdmission` needs matching staged semantics equivalent to:

```text
begin_input_record(id, bytes, work, now)
    charges source/global input packet exactly once
    charges source/global bytes
    charges per-packet/source/global work
    returns one-shot record permit

extend_input_record(permit, bytes, work, now)
    charges source/global bytes and work
    does NOT increment source/global input packet counts
    still enforces total per-packet work for the same permit
```

The record permit must carry enough state to enforce `max_work_per_packet` cumulatively across header + body stages. Do not let each stage independently receive a fresh 4096-unit per-packet allowance.

Any overflow, backwards/unusable clock, state expiry, source/global exhaustion, per-record work exhaustion, or invalid/reused permit fails closed and terminalizes the logical pre-auth state.

#### Cross-layer composition

`ListenerAdmission` should expose one composed staged record API so callers cannot update inner and outer accounting inconsistently.

Preferred shape:

```text
begin_tcp_input_record(&mut ticket, header_bytes, header_work)
    -> TcpInputReservation

reserve_tcp_input_body(&mut ticket, &mut reservation, declared_body_bytes, body_work)
    -> ()

finish_tcp_input_record(&mut ticket, reservation)
    -> ()
```

On any inner-layer failure, poison/reject the outer process state as D1 already requires. On outer-layer failure after the inner layer was conservatively charged, do not make the attacker-caused budget available again merely to preserve symmetry; the state is terminal. No new rollback path should resurrect the ticket.

### Required TCP receive ordering

Create one reusable pre-auth TCP frame-receive helper, rather than four ad-hoc migrations.

Required order:

1. caller has already admitted the source/state;
2. read exactly the fixed four-byte length header using bounded raw I/O;
3. before interpreting the length, call `begin_*_input_record(4, HEADER_WORK_RESERVATION)` so one logical record ownership and header bytes/work are charged;
4. only then decode/check the u32 length;
5. before allocating or reading the body, call `reserve_*_input_body(declared_len, BODY_WORK_RESERVATION)`;
6. only after reservation succeeds may the payload vector be allocated/read;
7. truncated body, EOF, timeout, oversize, arithmetic failure or I/O failure after reservation is terminal and keeps the conservative reservation charged;
8. only a complete body may proceed to negotiation/Noise parsing;
9. consume/complete the record reservation exactly once.

The existing wire length limit remains authoritative. Do not add a new numeric frame limit merely for E1A.

UDP is not part of this helper: bounded `recv_from` may continue to charge after raw datagram receive and before protocol parse, because there is no attacker-controlled pre-allocation framing step equivalent to TCP.

### Deterministic tests required for E1A

At minimum:

1. fragmented 4-byte header still charges one record only after the complete fixed header exists and before length interpretation;
2. zero-length body counts exactly one record;
3. normal header + body counts exactly one record, with cumulative bytes equal header + reserved body;
4. body extension does not increment packet count;
5. header work + body work is checked against one cumulative `max_work_per_packet` ceiling;
6. exact work ceiling succeeds; max+1 fails terminally;
7. oversize declared length fails after header ownership is charged but before body allocation;
8. truncated body after body reservation keeps conservative charge and terminalizes state;
9. timeout/EOF after reservation behaves the same;
10. inner budget exhaustion and outer source/global exhaustion cannot leave a reusable ticket;
11. arithmetic overflow/backwards clock/expired state cannot revive the same logical record;
12. record permit cannot be extended/completed twice;
13. existing response anti-amplification still observes one input packet, not two;
14. no auth/readiness/Delivery/PathValidated/ACK/authz-equivalent success evidence is reachable from an E1A rejection path.

Then migrate all current real TCP pre-auth responder handshakes:

- ordinary TCP probe;
- periodic TCP;
- multistream TCP;
- failover TCP.

No caller may retain the old `read complete frame -> charge_input` order.

Run targeted tests, `scripts/check.sh`, and `git diff --check`; run fuzz only if the migrated code materially changes the production untrusted-input parser/wire decoder rather than only the staged orchestration. Commit and push normally.

**Continue immediately to E2:** yes. Do not stop for reviewer acknowledgement after E1A passes its local gates.

### RSEC-001E1B — MEDIUM/open — semantic responder inventory

After E1A, complete the existing responder inventory/checker on top of `d271a99`:

- every TCP responder must anchor the new staged charged-frame primitive before negotiation/Noise parse;
- every UDP responder must anchor bounded raw receive -> charge -> protocol parse ordering;
- pending UDP ownership must prove queue reserve before store and terminal dequeue/cancel/expiry invalidation;
- rejection/timeout/malformed/I/O paths must not reach success-evidence anchors;
- expected responder surface set remains explicit so a new listener requires a semantic inventory entry.

Fix uncovered code, not just manifest strings. Full gate, commit, push.

**Continue immediately to C1:** yes.

### RSEC-001C — ADR checkpoint remains isolated

After E2, add only the noncontroversial carrier discriminator to source projection so TCP and UDP pre-auth domains are explicit and bounded. Do not invent terminal-source retention TTL/LRU/history sizes.

Then re-read D019. If literal no-reset-on-reconnect semantics still conflict with bounded source-accounting storage, write a compact ADR amendment request and stop only that policy-dependent lane. Continue independent H/I/J work while C2 waits.

## Rolling Work Queue

This is a multi-hour queue. No reviewer acknowledgement is required between dependency-satisfied slices.

### Q0 — Branch reconciliation

**Status:** `READY_LOCAL` if still needed.

Integrate current `origin/main` into `work/continue-20260904` without force-push, preserving `d271a99` and reviewer-owned handoff history. If already reconciled elsewhere, skip.

**Continue immediately to E1A:** yes.

### E1A — Staged one-record TCP accounting + four-responder migration

**Status:** `READY_LOCAL`; highest priority.

Use the exact implementation contract above. This is not an architecture/ADR blocker.

**Continue immediately to E2:** yes.

### E2 — Semantic responder inventory/evidence barrier closure

**Status:** `PREAUTHORIZED_AFTER_E1A`.

Complete semantic ordering/cleanup coverage, not substring-only coverage.

**Continue immediately to C1:** yes.

### C1 — Explicit carrier/source projection

**Status:** `PREAUTHORIZED_AFTER_E2`.

Add bounded carrier discriminator with deterministic non-collision tests; no retention policy invention.

**Continue immediately to C2:** yes.

### C2 — Terminal-source persistence policy checkpoint

**Status:** `ADR_CHECKPOINT_AFTER_C1`.

Resolve from existing reviewed text if possible. If a new retention policy/value is genuinely required, write the conflict/amendment request and external-wait only this lane.

**If externally waiting:** continue H -> I -> J locally.

### F — Full D019 adversarial/evidence-barrier matrix

**Status:** `PREAUTHORIZED_AFTER_C2_RESOLVED`.

Cover source/global concurrency; staged input bytes/packets/work; global windows; per-record cumulative work; memory; queue; response/3x anti-amplification; idle/lifetime/100 ms response deadline; arithmetic/clock failures; terminal non-revival; resolved reconnect/carrier semantics; cancellation/double cleanup; and no protocol success evidence on rejection.

Full gate, push, exact-head CI green before G.

### G — Exact-tree D019/security evidence review

**Status:** `PREAUTHORIZED_AFTER_F`.

Reconcile resource-abuse review, release-security packet, status and closure navigation to the exact repaired tree. Implementation finding may close only if E1A/E2/C1/C2/F are truly satisfied. Independent external/two-person review remains separate.

### H — Compatibility / freeze-boundary review

**Status:** `READY_LOCAL_AFTER_G`; also fallback during C2 external wait.

Audit corpus-v1 freeze vs global non-freeze, version negotiation, downgrade/transcript/resume/replay boundaries and stale wording. Add regression only for concrete defects.

### I — Package/operator/evidence-provenance review

**Status:** `READY_LOCAL_AFTER_H`; fallback during C2 wait.

Verify existing package lifecycle, build identity, cleanup, evidence manifests and exact-head references without reading protected identity material. Do not rerun sufficient VPS/package work for freshness.

### J — Reclassify release opportunities and reconsider VPS

**Status:** `READY_LOCAL_AFTER_I`.

Re-evaluate `OPEN_READY` vs `ALREADY_SUFFICIENT_FOR_BOUNDED_QUESTION` vs exact blockers. Only execute a VPS row if a genuine dependency-ready missing release question exists. Otherwise `READY_LIVE: none`; do not manufacture traffic.

## Completion gates

D019/RSEC-001 implementation closure requires:

- one TCP frame is one D019 input packet/record despite staged accounting;
- header is charged before attacker length interpretation;
- body bytes/work are reserved before allocation/body read;
- per-record work ceiling is cumulative across stages;
- incomplete/failed reserved bodies remain conservatively charged and terminal;
- all real TCP responders use the shared staged helper;
- semantic responder inventory/evidence barriers are machine checked;
- carrier/source projection is explicit and bounded;
- terminal-source persistence policy is reviewed rather than invented;
- full adversarial matrix and exact-head CI are green;
- security/release prose does not outrun exact implementation;
- release/governance flags remain unchanged.

## Do not expand into

- protocol/wire/Noise/Session/Carrier redesign for E1A;
- new numeric D019 limits or source-retention policies without reviewed ADR work;
- VPS/load testing as a substitute for deterministic accounting;
- renewed HY2 work without a changed missing-question hypothesis;
- speculative FEC/0-RTT/exotic carriers;
- public/production listener deployment;
- reading/hashing/copying/modifying/committing protected identity, secrets or private endpoint material;
- RC/freeze/release/production promotion.

## Questions requiring maintainer decision

None now.

E1A is explicitly implementation-ready. C2 may later become a genuine policy decision, but that must not block E1A/E2/C1 or independent H/I/J work.