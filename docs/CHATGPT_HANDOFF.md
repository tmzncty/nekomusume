# ChatGPT reviewer handoff — `442f60c` is useful R9-2H progress; buffered applied identity remains HIGH

## Current repository truth

- Latest developer source/test SHA reviewed: exact `442f60c8d1a3719b3b5b8c86e1f6e60017bb6ac0` (`fix(cli): offset-bearing ACK evidence + reversed-order process test (H-R9-013, M-R9-008 P1)`).
- Reviewer bounded recheck: `docs/reviews/r9-2h-exact-applied-identity-recheck-442f60c-20260914.md` (reviewer commit `a26a3a7`).
- Hosted checks on exact `442f60c`: `stable checks` success; `nightly decode fuzz smoke` success. Hosted checks are supplementary cross-evidence only, not developer-local exact-tree provenance.
- Open PRs: none. No new WAN/VPS experiment occurred. `READY_LIVE: none` remains authoritative.
- Governance remains unchanged: item 3 incomplete; item 4 incomplete; `RELEASE_CANDIDATE=false`; `PRODUCTION_READY=false`; `FREEZE=false`; `RELEASED=false`.

The coding agent is explicitly pre-authorized to finish R9-2H, persist final exact-tree local provenance, and then continue through R9-3..R9-12 and Q10/Q11/Q12 without waiting for reviewer cadence.

## Accepted progress at `442f60c`

- H-R9-011 pending logical-ACK buffering remains the accepted state behavior: a later exact Session DeliveryAck cannot advance the cumulative Session watermark across an earlier unconfirmed range.
- H-R9-012 mutation/evidence ordering remains repaired: an immediate Session mutation emits its applied event before buffered pending ACKs are drained.
- The immediate/current successful Session DeliveryAck now emits `r9_udp_delivery_ack_validated` with explicit `stream + offset`; retaining the legacy first-confirmation event in parallel is acceptable.
- A real built-binary `--reverse-ack-order` process regression now exists and exercises the actual failover server/client path.
- Prior closures remain controlling: one operation-wide malformed counter; one absolute application deadline; Carrier packet ACK remains Carrier-local; incomplete Carrier settlement is terminal/nonzero and cannot feed downstream health/failover as success.

# OPEN HIGH — H-R9-014 buffered applied ACK still lacks exact stream identity

The R9-2H contract requires exact `stream + offset` attribution after **every** successful Session DeliveryAck mutation.

The direct/current path now does this, but the pending-buffer drain still mutates Session state and emits:

```text
r9_udp_delivery_ack_validated
  ciphertext_bytes=0
  offset=<...>
  buffered=true
```

without `stream`.

Therefore `442f60c` only partially closes H-R9-013. This is an evidence-integrity HIGH for R9-2H closure, not a new Session semantic defect.

## Smallest H-R9-014 repair

Preserve the current pending-ACK state machine. After each buffered `SessionRuntime::delivery_ack` succeeds, emit the same exact applied identity as the immediate path: at minimum `stream`, `offset`, and `buffered=true`. Do not change Session/Carrier/ACK/wire/crypto architecture and do not add TTL/LRU/capacity policy.

Before transitioning from logical confirmation to Carrier-only settlement, explicitly enforce `outstanding.is_empty() && pending_acks.is_empty()`; otherwise fail typed. This is ownership truthfulness and requires no new numeric policy.

# R9-2H queue front — execute continuously

## R9-2H-A — close H-R9-014 exact buffered identity

Add exact stream attribution to buffered applied events and the explicit pre-settlement logical-owner invariant. Preserve the accepted order/buffering behavior.

## R9-2H-P1 — strengthen the real reversed-order process regression

The `442f60c` process test is useful but still too permissive for closure. Strengthen it against the structured event class rather than generic substring presence.

Require from the real client process:

1. successful client exit;
2. exactly one `r9_udp_delivery_ack_buffered` for stream 1 / offset 16 while watermark is 0;
3. exactly two `r9_udp_delivery_ack_validated` applied events;
4. applied event 1 = stream 1 / offset 0 / non-buffered;
5. applied event 2 = stream 1 / offset 16 / `buffered=true`;
6. each exact applied identity occurs once;
7. no `r9_udp_delivery_ack_covered` shortcut for those two records;
8. logical `outstanding + pending` ownership is empty before Carrier-only settlement;
9. final Recovery settlement reaches `remaining_in_flight=0`.

Do not satisfy this with generic `contains("offset")`, event presence, or inference from the server seam.

## R9-2H-P2 — strengthen migration-back reserved-record ownership

Current `reliable_udp_migration_back_reserves_final_record` still proves only a narrow negative string boundary. Make it prove the exact reserved final logical record:

- is not Recovery-tracked before post-promotion return authorization;
- is not sent by the legacy pre-promotion `udp_uncertain_range_sent` owner;
- is tracked/sent only by the explicit later post-promotion owner;
- is identified by exact stream/offset/record identity, not aggregate counts.

Do not add migration policy.

## R9-2H-P3 — persistent malformed budget across valid Carrier feedback

Through the real built process path drive one reliable receive/settlement operation:

```text
malformed #1
malformed #2
canonical Carrier ACK
malformed #3
```

Malformed #3 must hit the same existing `MAX_POST_HANDSHAKE_MALFORMED` ceiling. The valid Carrier ACK must not reset the operation-wide malformed count. Termination must be typed, finite and non-spinning. Add no numeric policy.

## R9-2H-P4 — preserve incomplete-settlement terminality

Keep `reliable_udp_incomplete_settlement_fails_not_settled` green and semantically unchanged: remaining in-flight != 0 is terminal/nonzero, emits incomplete, never emits settled, and never drives downstream health/failover from a false premise.

## R9-2H-GATE — final exact pushed-tree developer-local provenance

After H-R9-014 and P1-P4 land on one reachable source/test SHA, run in a clean worktree and persist:

```text
PYTHONDONTWRITEBYTECODE=1 bash scripts/check.sh
git diff --check
```

Record exact pushed SHA, UTC start/end, Linux OS/arch, Rust stable version, both exit codes, and clean initial/final tree. No wire decoder/parser/crypto framing change => do not invent a fuzz obligation; hosted fuzz remains supplementary.

R9-2H closes only when one reachable tree proves exact reversed logical confirmation, exact stream+offset applied identity for direct and buffered mutations, bounded pending ownership, one operation-wide malformed budget, one absolute operation deadline, logical + Carrier settlement before success, migration-back reserved ownership, incomplete-settlement terminality, and local exact-tree provenance.

**Then continue immediately to R9-3. Do not wait for reviewer cadence.**

# Continuous queue after R9-2H

Preserve this queue; do not collapse it into a micro-ticket.

## R9-3 — process Data-loss recovery

Add bounded deterministic post-admission suppression of one/periodic reliable-owned Data packet. Prove congestion admission precedes suppression; PTO fires only after deadline; retransmission uses a fresh authenticated packet number/nonce with stable Session/frame identity; application delivery is exactly once; Carrier ACK and Session DeliveryAck settle independently; final Recovery drains to zero or produces an explicit bounded incomplete/error.

## R9-4 — ACK-loss + reorder / delayed original

Cover Carrier packet ACK emitted then suppressed, retransmitted replacement before delayed original, and delayed original after replacement. Require one logical Session delivery, Session-owned duplicate suppression, no conflict, fresh packet numbers/nonces, truthful ACK-loss/PTO counters and final settlement.

## R9-5 — tamper / future ACK / malformed-feedback negatives

Prove tampered Data creates no Carrier ACK obligation or Session receive; tampered ACK creates no recovery mutation; future/never-sent ACK is typed rejected atomically; stale/duplicate ACK fabricates no RTT/loss/Session evidence; malformed feedback is finite and panic-free.

## R9-6 — pacing / cwnd / plaintext-owner atomicity

Prove initial and retransmit sends consult congestion admission; refusal consumes no Session byte space and commits no packet/plaintext/recovery owner; retransmit plaintext has one bounded owner; pacing deadlines are finite; teardown releases ownership.

## R9-7 — truthful process observability/result contract

Keep Data, Carrier ACK, Session DeliveryAck, PTO/retransmit, resolved recovery, Session delivery, malformed-budget, cleanup and final-outcome evidence domains distinct. Counters/events increment only after the action they claim succeeds.

## R9-8 — actual authenticated warm TCP standby

Reuse existing TCP connect/accept, canonical negotiation, fresh Noise trust/authz, resume/readiness tuple+generation+delivery-epoch binding and resource admission. Warm/standby carries no application Data before atomic promotion.

## R9-9 — resolved UDP health -> hysteresis -> real TCP promotion

Fresh resolved UDP outcomes drive Carrier health. Recoverable loss stays on UDP; PTO-only observation cannot erase later resolved loss; distinct bad resolved intervals cross committed hysteresis and promote only to an actually ready TCP standby. Invalid/unready standby yields typed failure.

## R9-10 — uncertain Session replay across UDP -> TCP

At promotion replay at least one genuinely uncertain logical Session range over promoted TCP. Draining UDP accepts no new application Data. Receiver deduplicates by Session/stream/offset and application bytes remain exactly once. Do not add TCP packet ACK.

## R9-11 — timeout / shutdown / cleanup matrix

Cover setup/application/PTO/single-owner receive deadline, controlled stop, failed standby promotion, malformed/tampered input, listener/socket/process cleanup and same-address rebind where supported.

## R9-12 — coherent exact-tree gate + independent bounded review

Independently challenge Session-above-Carrier layering, exact Session ACK evidence, authenticated packet identity/Carrier ACK, bounded plaintext/pending-ACK ownership, single receive-owner classification + operation-wide malformed/deadline ownership, PTO/pacing/cwnd, fresh health/hysteresis, real warm TCP readiness/promotion, uncertain Session replay/dedup, result truthfulness and cleanup/no-public-exposure. BLOCKER/HIGH -> smallest repair + regression + re-gate + continue; LOW/NOTE does not halt progression.

## Q10 — observability reconciliation

Integrate only genuinely new R9 evidence into existing `neko-observe`/result surfaces. Do not create a second logging framework.

## Q11 — factual status/release reconciliation

Reconcile `docs/status.md`, `IMPLEMENTATION_PLAN.md`, `ROADMAP.md` and the release packet only for evidence actually earned by the final R9 tree. Only a green independently reviewed cross-process R9 plus real TCP promotion may create a new specific `READY_LIVE` row.

## Q12 — one changed-hypothesis self-owned VPS run

Only after Q11 creates a specific `READY_LIVE` question, execute exactly one minimal bounded self-owned client<->VPS run under standing authorization. Record exact commit/binary/parameters, authenticated recovery/Session/switch evidence and cleanup. Preserve negative evidence; no unchanged same-class retry.

# VPS opportunity

**Not READY — implementation/evidence dependency.** Current unlock chain: R9-2H -> R9-3..R9-12 -> Q10/Q11. Standing authorization already covers the eventual bounded self-owned TCP/UDP run once a specific `READY_LIVE` row exists; do not ask for generic WAN permission.

# Non-blocking policy/authority gates

Keep separate from the mechanically determined R9 chain:

- `SessionRuntime.events` retention (`POLICY_BLOCKED_RESOURCE_BOUND`);
- D019 source-retention/no-reset policy;
- RSEC-001 capacity/adversarial-load suitability;
- signing/key custody/SBOM/publication trust;
- previous frozen-release interoperability;
- final RC/freeze/release/production authority.

Do not invent policy values while working R9.
