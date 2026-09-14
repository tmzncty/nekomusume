# ChatGPT reviewer handoff — R9-2 implementation-stagnation override; execute authenticated demux repair now

## Reviewed repository truth

- Current default branch before this refresh: exact `5d2d188f78c65e8f11009b0ccb50c84c1425bd57` (`docs(handoff): keep R9-3 blocked on authenticated demux closure`).
- Latest developer source/test SHA remains exact `fb09cf59e81782a5b2ecf12466bf47f72a9e51d6` (`fix(cli): R9-2 multi-record demux + reserved-record ownership`). There has been no newer source/test commit or open PR for multiple reviewer cycles.
- Exact `f74f1e6d711258bb2cba0853feed4359d799bee6` records the clean developer-local exact-tree gate for `fb09cf5`: `PYTHONDONTWRITEBYTECODE=1 bash scripts/check.sh` exit 0, `git diff --check` exit 0, clean initial/final tree, Linux x86_64, rustc 1.98.0.
- Current `5d2d188` has green GitHub-hosted `stable checks` and `nightly decode fuzz smoke`; hosted checks are extra cross-evidence only.
- Independent reviewer recheck remains `docs/reviews/independent-r9-repair-recheck-fb09cf5-20260914.md`.
- Open PRs: none. No WAN/VPS execution occurred in this sequence. `READY_LIVE: none` remains authoritative.
- Governance unchanged: item 3 incomplete; item 4 incomplete; `RELEASE_CANDIDATE=false`; `PRODUCTION_READY=false`; `FREEZE=false`; `RELEASED=false`.

## Implementation stagnation verdict

The current stop is **not** an external blocker and **not** a request for maintainer policy. `AGENTS.md` section 3.2 applies: the same mechanically repairable HIGH has remained at queue head across multiple review cycles while repository integrity and CI are green. The coding agent must stop waiting and implement the bounded demultiplexer now.

Do not create another proposal-only/doc-only cycle for R9-2. The existing semantics already determine the repair shape closely enough for implementation.

## H-R9-001 — OPEN HIGH and exact current defect

Current `crates/neko-cli/src/main.rs::recv_udp_delivery_ack` still accepts one `expected: &OutboundRecord` and loops until that one Session `DeliveryAck` arrives. In reliable mode:

- `fb09cf5` calls the helper serially for record 0 and then record 1;
- an authenticated record-1 Session DeliveryAck arriving while the first call waits for record 0 fails `delivery_ack_matches(expected=record0)`, then enters the Carrier-ACK branch;
- if it is not a canonical Carrier ACK, the `rt.is_some()` branch unconditionally `continue`s, consuming/discarding that legitimate logical confirmation;
- authenticated unexpected/control/malformed plaintext therefore bypasses `MAX_POST_HANDSHAKE_MALFORMED` whenever reliable mode is active;
- `rt.apply_ack(...)` errors are still discarded through `let _ = ...`, so rejected/future/stale Carrier feedback is not typed/observable at this boundary.

This is a local correctness/evidence defect. No wire grammar, Session architecture, Carrier architecture, crypto design, security numeric policy, D019 decision or release authority is needed to fix it.

## R9-2E — required implementation shape (pre-authorized)

Implement **one bounded authenticated receive/demux owner** in `neko-cli` integration glue for the reliable-owned logical records. Do not move Session semantics into `neko-carrier` and do not add a second delivery ledger.

The current R9 slice has exactly two reliable-owned records (`records[0]` and `records[1]`), already bounded by the existing process workload. Build the outstanding logical-confirmation set from those existing records; do not invent a new retention/capacity policy value.

A helper may be named/structured differently, but it should have the equivalent ownership model of:

- socket + expected peer + authenticated `SecureSession`;
- bounded outstanding Session DeliveryAck expectations keyed by exact `(session, stream, offset, len)`;
- mutable `SessionRuntime` as the sole owner that applies logical `delivery_ack`;
- mutable `ReliableUdpRuntime` as the sole owner that applies Carrier packet ACK feedback;
- existing negotiation/noise duplicate inputs and one absolute bounded deadline;
- existing finite malformed/ignored budget;
- typed counters/events for logical ACK applied/duplicate/unexpected, Carrier ACK applied/rejected, and malformed/unexpected authenticated plaintext.

For **every successfully authenticated plaintext, classify exactly once**:

1. Try `ProcessMessage::decode` and accept only exact `DeliveryAck` for Session 7001 whose `(stream, offset, len)` exists in the outstanding reliable-owned set. Apply `SessionRuntime::delivery_ack` exactly once and retire that expectation. A duplicate/stale/unexpected logical ACK is typed and bounded; it is never silently consumed as Carrier feedback.
2. Otherwise try canonical `neko_wire::decode` + `RecordType::Ack` + `decode_ack` + bounded `AckRanges`. Call `ReliableUdpRuntime::apply_ack`. Emit a typed applied or rejected outcome; an error must not be discarded and rejected feedback must not mutate recovery.
3. Otherwise consume the existing authenticated malformed/ignored budget, emit a typed diagnostic, and fail once the finite bound is exhausted. Reliable mode must not have an unconditional silent `continue` for authenticated unknown plaintext.

The two grammars are already distinguishable without architecture change: `ProcessMessage` requires process version byte `1`, while the candidate outer `neko_wire` record currently uses wire version `0`. Do not add a new protocol tag merely for this repair.

The demux loop may return success when **both** conditions hold:

- all required reliable-owned Session DeliveryAck expectations have been applied exactly once;
- Carrier recovery reports `in_flight()==0`.

Otherwise it must end with an explicit bounded timeout/partial failure at the existing deadline. Do not manufacture Session confirmation from packet ACK and do not manufacture Carrier ACK from Session DeliveryAck.

Legacy non-`--reliable-udp` behavior may keep the simpler single-record helper if that minimizes change; reliable mode must use the single demux owner rather than two serial `recv_udp_delivery_ack` calls.

## R9-2F — discriminating regressions required in the same coherent repair

Add bounded built-binary/process tests that would fail on the current serial helper.

### 1. Reversed logical DeliveryAck order

Add a test-only server seam that retains the first reliable-owned **Session DeliveryAck** until the second reliable-owned Data has been accepted, then sends the second logical DeliveryAck before the first. Carrier packet ACK behavior can remain otherwise normal.

Prove:

- both logical confirmations are classified regardless of order;
- both exact logical keys are applied to `SessionRuntime` exactly once;
- no logical ACK is silently consumed as Carrier feedback;
- final Carrier in-flight state settles to zero;
- clean baseline uses zero PTO/retransmit and zero Session conflict; duplicate count is zero unless the test intentionally creates one and documents why.

### 2. Reliable UDP + migration-back reservation

Run the existing recovery/migration-back path together with `--reliable-udp` and the minimum count that exercises the reserved final record. Prove the reserved final logical offset (the current three-record case is index 2 / offset 32 for 16-byte records) is neither reliable-tracked nor legacy wire-sent before the post-promotion return-to-UDP authorization milestone. It may be sent only by the explicitly authorized later owner.

Also retain the ordinary no-loss multi-record test, but make its assertions discriminating: two reliable logical first-deliveries, two independent Session DeliveryAck applications, typed Carrier ACK outcomes, `remaining_in_flight:0`, zero PTO/retransmit, finite malformed budget, zero conflict, cleanup.

## R9-2G — exact-tree closure gate

After one coherent pushed source/test SHA closes H-R9-001 and M-R9-004, run on the exact pushed clean tree:

```bash
PYTHONDONTWRITEBYTECODE=1 bash scripts/check.sh
git diff --check
```

Record exact pushed SHA, UTC start/end, OS/arch, rustc, exit codes, clean initial/final tree. Hosted CI remains separate cross-evidence. No decode fuzz is required unless external decoder/framing grammar changes.

Reviewer then checks the actual diff and tests. Do not wait for reviewer cadence to start the downstream queue if the repair is green and no new BLOCKER/HIGH is discovered locally.

# Pre-authorized continuous queue after R9-2 closes

Preserve this queue; do not collapse it to one hourly ticket.

## R9-3 — process Data-loss recovery

Add bounded deterministic post-admission suppression of one/periodic reliable-owned Data packet. Prove congestion admission precedes suppression; PTO fires only after deadline; retransmission uses fresh authenticated packet number/nonce with stable frame/Session identity; exactly-once Session delivery; Carrier ACK and Session DeliveryAck settle independently; final recovery drains to zero or yields explicit bounded partial/failure.

## R9-4 — ACK-loss + reorder/delayed-original

Cover packet ACK emitted then suppressed, retransmitted replacement before delayed original, and delayed original after replacement. Require one logical Session delivery, Session-owned duplicate suppression, no conflict, fresh packet numbers/nonces, truthful ACK-loss/PTO counters and final settlement.

## R9-5 — tamper/future ACK/malformed-feedback negatives

Prove unauthenticated/tampered Data creates no packet-ACK obligation and no Session receive; tampered ACK creates no recovery mutation; future/never-sent ACK is typed rejected atomically; stale/duplicate ACK fabricates no RTT/loss/Session evidence; malformed ACK/range/control handling is finite and panic-free.

## R9-6 — pacing/cwnd/plaintext-owner atomicity

Using bounded test-only limits, prove initial/retransmit sends consult congestion admission; refusal consumes no Session byte space and no committed packet/plaintext/recovery owner; retransmission plaintext owner is bounded/single-source-of-truth; pacing deadlines finite; teardown releases ownership.

## R9-7 — truthful process observability/result contract

Distinguish Data offered/admitted/wire-sent/suppressed; Carrier ACK emitted/wire-sent/suppressed/applied/rejected; Session DeliveryAck emitted/applied/duplicate/rejected; PTO due/fired; retransmit attempted/admitted/wire-sent/refused; resolved acked/lost/in-flight; Session first-delivery/duplicate/conflict/application bytes; cleanup/final outcome. Increment counters only after represented actions succeed.

## R9-8 — actual authenticated warm TCP standby

Reuse existing failover TCP connect/accept, canonical negotiation, fresh Noise trust/authz, resume/readiness tuple+generation+delivery-epoch binding and resource admission. No standby application Data before atomic promotion.

## R9-9 — resolved UDP health -> hysteresis -> real TCP promotion

Fresh resolved UDP outcomes drive Carrier health. Recoverable loss stays on UDP; PTO-only observation cannot erase later resolved loss; distinct bad resolved intervals cross committed hysteresis and promote only to an actually ready TCP standby; invalid/unready standby yields `FallbackFailed`; old historical loss is not replayed as new evidence.

## R9-10 — uncertain Session replay across UDP -> TCP

At promotion, replay at least one genuinely uncertain logical Session range over promoted TCP. Draining UDP accepts no new application Data. Receiver deduplicates by Session/stream/byte offset; application bytes exactly once. Do not add TCP packet ACK.

## R9-11 — timeout/shutdown/cleanup matrix

Cover setup/application/PTO/settlement deadlines, controlled stop, failed standby promotion, malformed/tampered input, listener/socket/process cleanup and same-address rebind where supported.

## R9-12 — coherent exact-tree gate + independent bounded review

Independent review challenges Session-above-Carrier layering, authenticated packet identity/ACK, bounded plaintext ownership, deadline-driven PTO/pacing/cwnd, fresh health/hysteresis, real warm TCP readiness/promotion, uncertain Session replay/dedup, result truthfulness and cleanup/no-public-exposure. BLOCKER/HIGH -> smallest repair + regression + re-gate + continue; LOW/NOTE does not halt progression.

# Q10/Q11/Q12 after R9

- **Q10:** integrate only genuinely new R9 evidence into existing observability; no second logging framework.
- **Q11:** reconcile `docs/status.md`, `IMPLEMENTATION_PLAN.md`, `ROADMAP.md` and release packet only for earned status. Only a green independently reviewed cross-process R9 + real TCP promotion may create the changed specific `READY_LIVE` row.
- **Q12:** then execute exactly one minimal changed-hypothesis self-owned client<->VPS run under standing authorization, with temporary unprivileged listeners, exact commit/binary/parameters, authenticated recovery/Session/switch evidence and cleanup. Preserve negative evidence; no unchanged same-class retry.

# VPS opportunity

**Not READY yet.** Current multi-record receive semantics are still order-sensitive and therefore not trustworthy enough for WAN promotion. This is an implementation dependency, not a permission blocker.

Once repaired R9-2..R9-12 and Q11 establish a new specific live question, standing authorization already covers the bounded self-owned TCP/UDP run. Do not ask again for ordinary WAN authorization.

# Non-blocking policy/authority gates

Keep these separate from the mechanically determined R9 chain:

- `SessionRuntime.events` retention (`POLICY_BLOCKED_RESOURCE_BOUND`);
- D019 source-retention/no-reset policy;
- RSEC-001 capacity/adversarial-load suitability;
- signing/key custody/SBOM/publication trust;
- previous frozen-release interoperability;
- final RC/freeze/release/production authority.

Do not invent policy values while working R9.
