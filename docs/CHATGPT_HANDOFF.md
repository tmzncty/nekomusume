# ChatGPT reviewer handoff — R9-2 blocked on exact P3 terminal oracle at `779efab`

## Current repository truth

- Latest developer-owned source/test commit reviewed: exact `779efab949f2b383599a0c6b33781ab5615e830a` (`fix(cli): applied Carrier-ACK evidence between malformed #2/#3 (H-R9-022)`).
- Reviewer checkpoint: [`docs/reviews/reviewer-r9-p3-oracle-779efab-20260916.md`](reviews/reviewer-r9-p3-oracle-779efab-20260916.md).
- H-R9-022 is **closed narrowly**: the existing reliable receive owner now emits `r9_udp_packet_ack_applied` when the interleaved Carrier ACK is actually applied, carrying the live operation-wide malformed counter; the P3 test observes `malformed=2`.
- New STOP front is H-R9-023 below: the P3 built-binary acceptance oracle is still broad enough to false-pass on the wrong terminal path or downstream continuation.
- Hosted Rust CI on exact `779efab`: `stable checks` SUCCESS (`bash scripts/check.sh`) and `nightly decode fuzz smoke` SUCCESS. Hosted CI is cross-evidence only.
- Open PRs: none.
- No new WAN/VPS experiment in this sequence.
- Final developer-local clean exact-tree provenance for R9-2 is still absent.
- `READY_LIVE: none`; item 3 incomplete; item 4 incomplete; `RELEASE_CANDIDATE=false`; `PRODUCTION_READY=false`; `FREEZE=false`; `RELEASED=false`.

The external coding agent must synchronize to current `main` and continue every dependency-ready slice below without waiting for reviewer cadence. Reviewer cadence is a check frequency, not a work-ticket length.

# STOP FRONT — H-R9-023 exact P3 terminal/oracle closure

**Severity: HIGH for R9-2 evidence correctness.** Mechanically repairable under current semantics; no maintainer policy or architecture decision required.

Preserve the accepted source ordering and applied-event seam:

```text
malformed #1 -> malformed #2 -> Carrier packet ACK(applied=true, malformed=2)
             -> Session DeliveryAck -> malformed #3 -> terminal malformed bound
```

Current P3 still has four oracle gaps:

1. `r9_udp_packet_ack_applied` is checked with broad `contains`; require exactly one selected applied event with `malformed=2`.
2. The test does not forbid `r9_udp_packet_ack_rejected`; require none for this fixture.
3. The terminal assertion remains `contains("malformed") || contains("bound")`; pin the existing exact terminal error `UDP delivery acknowledgement malformed bound exceeded` and require nonzero exit unconditionally.
4. It forbids the two settled markers but not downstream health/failover/migration continuation. After the malformed bound, require no health sample/degraded/failed transition, no TCP promotion, no migration-back, and no final success marker from this path.

## Required smallest repair

Do not change `MAX_POST_HANDSHAKE_MALFORMED`; do not change Session/Carrier/ACK/crypto/wire semantics; do not add a second receive owner or a fresh deadline.

Tighten the existing built-binary P3 test only. Use structured-event line collection/cardinality rather than broad substring presence where possible. Focused test -> commit/push -> continue immediately to the next READY_LOCAL slice. No decoder/framing fuzz is required solely for assertion changes.

# Accepted progress — preserve it

- **H-R9-022 applied Carrier-ACK evidence:** closed narrowly at `779efab`.
- **H-R9-021 source ordering:** accepted; keep the test-only defer ordering from `966eb49`.
- **H-R9-020 final accounting:** closed; the count=4 ownership partition remains `2 UDP reliable-confirmed / 1 TCP uncertain replay / 1 post-return reliable / 4 total`.
- **Reversed Session-ACK order:** current built-binary test proves one buffered offset-16 ACK, exact applied offsets 0 then 16, no covered shortcut, and terminal `remaining_in_flight=0`.
- Earlier candidate A (`Recovery::on_ack` future/never-sent rejection) and candidate B (`record_datagrams` mixed drop reasons) remain closed absent a contradictory reproducer.

# READY_LOCAL after H-R9-023 — finish R9-2 without inventing parallel fixtures

## 1. Exact P2 C1 — TCP replay identity/cardinality

Use the existing count=4 migration-back fixture. Require mechanically:

- exactly one client TCP replay/DeliveryAck for `seq=2`, stream 1, offset 32;
- exactly one corresponding server TCP DeliveryAck for the same logical range;
- no TCP replay/ACK evidence for offsets 0/16 (already reliable-UDP-owned) or 48 (reserved post-return reliable).

The current test counts one client TCP ACK but does not yet pin all exact identities/cardinalities.

## 2. Exact P2 C2 — migration strictly precedes the one post-return send

Require strict client order and one-only cardinality:

```text
udp_recovery_challenge_sent
  < udp_recovery_validated
  < udp_migrated_back
  < r9_udp_post_return_sent(stream=1, offset=48)
```

Offset 48 must never appear on a legacy uncertain/TCP replay path.

## 3. Exact P2 C3 — server post-return identity/cardinality

Require exactly once each, with exact logical identity where applicable:

```text
udp_recovery_owner_started
  < udp_recovery_validated
  < udp_return_delivery_ack_sent(seq=3 / stream=1 / offset=48)
  < udp_return_packet_ack_sent(same bounded owner)
```

Current test proves order but not full identity/cardinality.

## 4. Exact P2 C4 — client dual-domain settlement

Require exactly once each:

- Session DeliveryAck validation for stream 1 / offset 48;
- Carrier packet ACK `applied=true` for the post-return reliable owner;
- `r9_udp_post_return_settled` for stream 1 / offset 48 / `remaining_in_flight=0`.

Settled must occur after both ACK-domain events regardless of arrival order.

## 5. Post-return ACK arrival-order seam

One bounded receive owner, one absolute operation deadline, one malformed budget. Exercise both:

- Session DeliveryAck -> Carrier packet ACK;
- Carrier packet ACK -> Session DeliveryAck.

Both must end in the same exact settled state; do not create a second receive owner or fresh per-order deadline.

## 6. Automatic-health replay exact identity

Challenge the automatic-health path against the actual `FailoverController::tcp_resend()` `(DataId, payload)` ownership set. Do not reduce exact controller ownership to a count and reconstruct positionally. Identity/payload mismatch must fail closed.

## 7. P4 acknowledgement-domain suppression tightening

Preserve both existing seams:

- missing Carrier packet ACK while Session DeliveryAck arrives;
- missing Session DeliveryAck while Carrier packet ACK arrives.

For each: require nonzero typed terminal failure; require no `r9_udp_in_flight_settled`, no `r9_udp_post_return_settled`, and no downstream health/failover/migration continuation. Replace broad terminal disjunctions with the existing exact terminal marker/error where stable.

## 8. R9-2 final developer-local exact-tree provenance

On the final pushed source/test SHA, in a clean safe checkout/worktree, run:

```text
PYTHONDONTWRITEBYTECODE=1 bash scripts/check.sh
git diff --check
```

Record exact pushed SHA, UTC start/end, Linux OS/arch, Rust stable version, exit codes, clean initial/final tree. Run pinned decode fuzz only if decoder/parser/crypto framing changed. Hosted CI remains additional cross-evidence.

After this, continue immediately into R9-3. Do not wait for reviewer cadence.

# Continuous dependency-ordered queue after R9-2

9. **R9-3 Data-loss recovery:** suppress one reliable-owned Data only after admission; cwnd admission precedes suppression; PTO after deadline; retransmit under fresh packet number/nonce with stable Session/frame identity; exactly-once logical delivery; independent Session/Carrier ACK domains; final Recovery zero or typed bounded failure.
10. **R9-4 ACK-loss + delayed original/reorder:** suppress Carrier ACK, retransmit, release delayed original; one logical delivery; Session dedup; fresh packet numbers/nonces; truthful loss/PTO; settled recovery.
11. **R9-5/R9-6 adversarial correctness + ownership:** tamper/future-ACK/stale feedback are atomic; malformed input finite/panic-free; every send/retransmit consults congestion admission; refusal commits no logical/recovery ownership; exactly one bounded retransmit-plaintext owner; teardown releases state.
12. **R9-7 process/result truth:** Data, Carrier packet ACK, Session ACK, PTO/retransmit, Recovery, Session delivery, malformed budget and terminal result remain distinct and are emitted only after the claimed transition.
13. **R9-8/R9-9 warm TCP + health promotion:** authenticated/resume-bound warm standby carries no application Data before promotion; resolved UDP outcomes feed health/hysteresis; recoverable loss remains UDP; PTO-only samples cannot erase later loss; only ready TCP promotes.
14. **R9-10 uncertain Session replay + cleanup:** replay only genuine uncertain Session ranges over promoted TCP; draining UDP receives no new Data; exact-once Session dedup; no TCP packet ACK; cover timeout/shutdown/cleanup negatives.
15. **R9-11/R9-12 + Q10/Q11/Q12:** coherent exact-tree gate, then a dedicated independent bounded review of the whole new cross-process reliable-UDP integration surface. Any BLOCKER/HIGH returns to smallest repair + regression + re-gate. Then reconcile observability/status/release packet factually. Only a specific Q11-created `READY_LIVE` row may unlock one bounded changed-hypothesis self-owned VPS run under standing authorization.

# Item-4 / core-surface boundary

The pre-R9 deep item-4 sweep already challenged the earlier reliable engine, CarrierState/Manager, scheduler/flow accounting, adapters, SessionRuntime, observability, package/reproducibility, dependency/build, CLI portability/output, boundedness/validators and wire/parser surfaces.

The new **cross-process R9 integration surface** is materially new and still requires its own dedicated independent bounded challenge before item-4 support can be reconciled. Do not mark item 4 complete merely because R9 tests turn green.

# VPS opportunity

**Not READY.** Standing authorization remains valid, and the rental window is still valuable, but a known-invalid local acceptance oracle must not be promoted to WAN evidence. Current blocker class: local correctness/evidence plus later independent review.

Unlock sequence:

```text
H-R9-023
  -> exact P2 + ACK-order + auto replay identity + P4
  -> clean R9-2 provenance
  -> R9-3..R9-12
  -> Q10/Q11
  -> specific READY_LIVE row
  -> one bounded changed-hypothesis self-owned VPS run
```

# Separate non-blocking policy / authority gates

- `SessionRuntime.events` retention policy;
- D019 source-retention/no-reset policy;
- RSEC-001 adversarial-load/capacity suitability conditions;
- signing/key custody/SBOM/publication trust;
- previous frozen-release interoperability;
- final RC/freeze/release/production authority.

Do not invent policy values while working R9, and do not let these independent gates starve dependency-ready local implementation/review work.
