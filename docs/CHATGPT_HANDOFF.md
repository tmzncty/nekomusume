# ChatGPT reviewer handoff — R9-2H source semantics improved; truthful reversed-ACK evidence still HIGH

## Current repository truth

- Latest developer source/test SHA reviewed: exact `676dca307f00ad36de69600c8a330ab58d60524f` (`fix(cli): R9-2 pending logical-ACK buffer — no cumulative over-confirmation (H-R9-011)`).
- Reviewer bounded recheck: `docs/reviews/r9-2h-pending-ack-recheck-676dca-20260914.md` (reviewer commit `c58298f`).
- `676dca3` changes only `crates/neko-cli/src/main.rs`; it adds no new process regression and no developer-local exact-tree provenance file.
- GitHub-hosted checks on exact `676dca3`: `stable checks` success; `nightly decode fuzz smoke` success. Hosted checks are supplementary cross-evidence only, not developer-local provenance.
- Open PRs: none. No new WAN/VPS experiment occurred. `READY_LIVE: none` remains authoritative.
- Governance remains unchanged: item 3 incomplete; item 4 incomplete; `RELEASE_CANDIDATE=false`; `PRODUCTION_READY=false`; `FREEZE=false`; `RELEASED=false`.

The coding agent is explicitly pre-authorized to finish R9-2H, persist final exact-tree local provenance, and then continue through R9-3..R9-12 and Q10/Q11/Q12 without waiting for reviewer cadence.

## Accepted progress

### H-R9-011 state mutation — source repair ACCEPTED, closure evidence still pending

`676dca3` introduces a bounded `pending_acks` owner in the reliable receive path. A later exact Session DeliveryAck no longer directly advances the cumulative Session watermark across an earlier unconfirmed range:

- only `record.offset == SessionRuntime::confirmed_watermark(stream)` may call `delivery_ack` immediately;
- a later exact ACK is retained pending;
- pending ACKs apply only after the watermark becomes contiguous;
- Session core semantics, wire grammar and crypto framing are unchanged.

Pending ownership introduces no new numeric policy: recognized records move from the already-bounded reliable-owned `outstanding` set into `pending_acks`.

Keep the following prior closures intact:

- **H-R9-010B:** incomplete Carrier settlement is terminal/nonzero and never emits `r9_udp_in_flight_settled` or falls through as success;
- **M-R9-009:** Carrier settlement uses the same absolute application deadline; no fresh settlement time budget;
- one operation-wide malformed counter remains threaded through logical confirmation and Carrier settlement;
- Carrier packet ACK remains a separate typed `apply_ack` outcome and cannot itself advance Session delivery.

# OPEN HIGH — H-R9-012 truthful logical-ACK application ordering

The state mutation order is now correct, but the structured evidence order is not.

For the required reversed-ACK case (`record 1 offset=16` arrives before `record 0 offset=0`):

1. record 1 is correctly observed as `r9_udp_delivery_ack_buffered` at watermark 0;
2. record 0 later arrives and its exact `SessionRuntime::delivery_ack(offset=0)` is applied first;
3. the code immediately drains pending record 1, applies it second, and emits record 1's `r9_udp_delivery_ack_validated` event;
4. only after that drain loop does it emit the validated event for record 0.

Thus diagnostics say record 1 applied before record 0 even though Session state applied record 0 then record 1. In the reversed case the legacy first-confirmation `udp_delivery_ack_validated` event can also disappear because `logical_confirmations` is already 2 by the time the current-record diagnostic is selected.

This is an evidence-integrity HIGH because P1 must prove exact logical confirmation order using offset-bearing process evidence. Do not advance to R9-3 until the event stream truthfully represents the successful Session mutations.

## Smallest repair contract

Do not change Session/Carrier/ACK/wire/crypto architecture and do not add TTL/LRU/capacity policy.

After an in-order current ACK successfully calls `delivery.delivery_ack`, emit that exact record's applied/validated diagnostic **immediately**. Only then drain `pending_acks`, emitting each buffered record's applied/validated event after its own successful Session mutation. Keep these concepts distinct:

- `observed` / `buffered` — evidence arrived but did not advance Session confirmation;
- `applied` / `validated` — exact Session confirmation mutation succeeded;
- Carrier packet ACK — Carrier-local recovery feedback only.

Before a successful reliable receive is considered logically complete, no unmatched `outstanding` or `pending_acks` evidence may remain. This is ownership truthfulness, not a new capacity policy.

# R9-2H queue front — execute continuously

## R9-2H-A — repair H-R9-012 diagnostic ordering

Implement the smallest source adjustment above. Preserve all existing H-R9-011 pending-ACK state behavior and H-R9-010/M-R9-009 settlement behavior.

## R9-2H-P1 — actual built-binary reversed logical ACK regression

The earlier `88ffa5f` commit did **not** add the required built-binary reversed-ACK process test. Add one using real `failover-server` / `failover-client` with the existing `--reverse-ack-order` seam. Require offset-bearing evidence proving this exact sequence:

1. offset 16 ACK is observed first;
2. offset 16 is buffered while Session watermark remains 0;
3. exact offset 0 confirmation applies exactly once and its applied event appears first;
4. only then buffered exact offset 16 confirmation applies exactly once and its applied event appears second;
5. logical outstanding + pending ownership becomes empty only after both exact confirmations;
6. Carrier packet ACK stays Carrier-local;
7. Recovery settles to zero in-flight on success;
8. no duplicate/conflict application delivery is created.

A test that checks only exit success, only event presence, or the current inverted diagnostics is invalid.

## R9-2H-P2 — strengthen migration-back reserved-record ownership

Keep and strengthen `reliable_udp_migration_back_reserves_final_record`. Prove all of:

- exact reserved final record is **not Recovery-tracked** before post-promotion return authorization;
- it is not sent by the legacy pre-promotion `udp_uncertain_range_sent` owner;
- only the explicit later post-promotion owner may track/send it;
- assertions identify the exact reserved offset/record rather than only aggregate counts or one exact negative log string.

Do not add migration policy.

## R9-2H-P3 — persistent malformed budget across valid Carrier feedback

Through the real built process path drive a single reliable receive/settlement operation:

```text
malformed #1
malformed #2
canonical Carrier ACK
malformed #3
```

Malformed #3 must hit the same existing `MAX_POST_HANDSHAKE_MALFORMED` ceiling. The valid Carrier ACK must not reset the operation-wide malformed count. Termination must be typed, finite and non-spinning. Add no new numeric value.

## R9-2H-P4 — preserve incomplete settlement terminality

Keep `reliable_udp_incomplete_settlement_fails_not_settled` green and semantically unchanged: remaining in-flight != 0 is terminal, nonzero, emits incomplete, never emits settled, and never drives downstream health/failover from a false premise.

## R9-2H-GATE — one final exact pushed-tree developer-local provenance

After H-R9-012 and P1-P4 land on one reachable source/test SHA, run in a clean worktree and persist:

```text
PYTHONDONTWRITEBYTECODE=1 bash scripts/check.sh
git diff --check
```

Record exact pushed SHA, UTC start/end, Linux OS/arch, Rust stable version, both exit codes, and clean initial/final tree. If wire decoder/parser/crypto framing did not change, do not invent a fuzz obligation; hosted fuzz remains separate supplementary evidence.

R9-2H closes only when the same reachable tree proves exact reversed logical confirmation, truthful diagnostics, bounded pending ownership, one operation-wide malformed budget, one absolute operation deadline, logical + Carrier settlement before success, migration-back reserved ownership, incomplete-settlement terminality, and exact-tree local provenance.

**Then continue immediately to R9-3. Do not wait for reviewer cadence.**

# Continuous queue after R9-2H

Preserve this deep queue; do not collapse it into one micro-ticket.

## R9-3 — process Data-loss recovery

Add bounded deterministic post-admission suppression of one/periodic reliable-owned Data packet. Prove congestion admission precedes suppression; PTO fires only after deadline; retransmission uses a fresh authenticated packet number/nonce with stable Session/frame identity; application delivery is exactly once; Carrier ACK and Session DeliveryAck settle independently; final Recovery drains to zero or produces an explicit bounded incomplete/error.

## R9-4 — ACK-loss + reorder / delayed original

Cover Carrier packet ACK emitted then suppressed, retransmitted replacement before delayed original, and delayed original after replacement. Require one logical Session delivery, Session-owned duplicate suppression, no conflict, fresh packet numbers/nonces, truthful ACK-loss/PTO counters and final settlement.

## R9-5 — tamper / future ACK / malformed-feedback negatives

Prove tampered Data creates no Carrier ACK obligation or Session receive; tampered ACK creates no recovery mutation; future/never-sent ACK is typed rejected atomically; stale/duplicate ACK fabricates no RTT/loss/Session evidence; malformed feedback is finite and panic-free.

## R9-6 — pacing / cwnd / plaintext-owner atomicity

With existing bounded test-only controls, prove initial and retransmit sends consult congestion admission; refusal consumes no Session byte space and commits no packet/plaintext/recovery owner; retransmit plaintext has one bounded owner; pacing deadlines are finite; teardown releases ownership.

## R9-7 — truthful process observability/result contract

Keep these domains distinct and ordered by actual success:

- Data offered/admitted/wire-sent/suppressed;
- Carrier ACK emitted/wire-sent/suppressed/applied/rejected;
- Session DeliveryAck observed/buffered/applied/duplicate/rejected;
- PTO due/fired;
- retransmit attempted/admitted/wire-sent/refused;
- resolved acked/lost/in-flight;
- Session first-delivery/duplicate/conflict/application bytes;
- malformed-budget use, cleanup and final outcome.

Counters/events increment only after the action they claim succeeds.

## R9-8 — actual authenticated warm TCP standby

Reuse the existing TCP connect/accept, canonical negotiation, fresh Noise trust/authz, resume/readiness tuple+generation+delivery-epoch binding and resource admission. Warm/standby carries no application Data before atomic promotion.

## R9-9 — resolved UDP health -> hysteresis -> real TCP promotion

Fresh resolved UDP outcomes drive Carrier health. Recoverable loss stays on UDP; PTO-only observation cannot erase later resolved loss; distinct bad resolved intervals cross committed hysteresis and promote only to an actually ready TCP standby. Invalid/unready standby yields a typed failure.

## R9-10 — uncertain Session replay across UDP -> TCP

At promotion replay at least one genuinely uncertain logical Session range over promoted TCP. Draining UDP accepts no new application Data. Receiver deduplicates by Session/stream/offset and application bytes remain exactly once. Do not add TCP packet ACK.

## R9-11 — timeout / shutdown / cleanup matrix

Cover setup/application/PTO/single-owner receive deadline, controlled stop, failed standby promotion, malformed/tampered input, listener/socket/process cleanup and same-address rebind where supported.

## R9-12 — coherent exact-tree gate + independent bounded review

Independently challenge Session-above-Carrier layering, exact Session ACK evidence, authenticated packet identity/Carrier ACK, bounded plaintext/pending-ACK ownership, single receive-owner classification + operation-wide malformed/deadline ownership, deadline-driven PTO/pacing/cwnd, fresh health/hysteresis, real warm TCP readiness/promotion, uncertain Session replay/dedup, result truthfulness and cleanup/no-public-exposure. BLOCKER/HIGH -> smallest repair + regression + re-gate + continue; LOW/NOTE does not halt progression.

## Q10 — observability reconciliation

Integrate only genuinely new R9 evidence into existing `neko-observe`/result surfaces. Do not create a second logging framework.

## Q11 — factual status/release reconciliation

Reconcile `docs/status.md`, `IMPLEMENTATION_PLAN.md`, `ROADMAP.md` and the release packet only for evidence actually earned by the final R9 tree. Only a green independently reviewed cross-process R9 plus real TCP promotion may create a new specific `READY_LIVE` row.

## Q12 — one changed-hypothesis self-owned VPS run

Only after Q11 creates a specific `READY_LIVE` question, execute exactly one minimal bounded self-owned client<->VPS run under standing authorization. Record exact commit/binary/parameters, authenticated recovery/Session/switch evidence and cleanup. Preserve negative evidence; no unchanged same-class retry.

# VPS opportunity

**Not READY — implementation/evidence dependency.** The current unlock chain is R9-2H -> R9-3..R9-12 -> Q10/Q11. Once Q11 truthfully creates a specific changed `READY_LIVE` row, standing authorization already covers a bounded self-owned TCP/UDP run; do not ask for generic WAN permission again.

# Non-blocking policy/authority gates

Keep these separate from the mechanically determined R9 chain:

- `SessionRuntime.events` retention (`POLICY_BLOCKED_RESOURCE_BOUND`);
- D019 source-retention/no-reset policy;
- RSEC-001 capacity/adversarial-load suitability;
- signing/key custody/SBOM/publication trust;
- previous frozen-release interoperability;
- final RC/freeze/release/production authority.

Do not invent policy values while working R9.
