# ChatGPT reviewer handoff — R9-2 P3 closed at `2bddf1d`; exact P2 closure next

## Current repository truth

- Latest developer-owned source/test commit reviewed: exact `2bddf1d5ce255f94d990d56dffcd6f2e2dae9058` (`test(cli): P3 exact terminal oracle — applied ACK at malformed=2, typed bound, no continuation (H-R9-023)`).
- Reviewer checkpoint: [`docs/reviews/reviewer-r9-p3-close-2bddf1d-20260916.md`](reviews/reviewer-r9-p3-close-2bddf1d-20260916.md).
- H-R9-023 is **closed narrowly**: the built-binary P3 fixture now requires exactly one applied Carrier ACK at live operation-wide `malformed=2`, forbids rejected packet-ACK evidence, requires the exact existing malformed-bound terminal error with nonzero exit, and forbids successful settlement plus downstream health/TCP-warm/migration continuation.
- Hosted Rust cross-evidence on exact `2bddf1d`: `stable checks` SUCCESS and `nightly decode fuzz smoke` SUCCESS. The commit is test-only; no decoder/framing source changed.
- Open PRs: none.
- No new WAN/VPS experiment in this sequence.
- Final developer-local clean exact-tree provenance for R9-2 is still absent.
- `READY_LIVE: none`; release item 3 incomplete; item 4 incomplete; `RELEASE_CANDIDATE=false`; `PRODUCTION_READY=false`; `FREEZE=false`; `RELEASED=false`.

The external coding agent must synchronize to current `main` and continue every dependency-ready slice below without waiting for reviewer cadence. Reviewer cadence is a check frequency, not a work-ticket length. Several adjacent P2 assertions may be landed as one coherent implementation/test commit when that is cleaner, but the queue below must remain visible and must not collapse to one small ticket.

# Accepted progress — preserve it

- **H-R9-023 P3 terminal oracle:** closed narrowly at `2bddf1d`.
- **H-R9-022 applied Carrier-ACK evidence:** closed narrowly at `779efab`; the applied marker is emitted only after the Recovery owner actually accepts Carrier feedback and carries the live malformed counter.
- **H-R9-021 source ordering:** accepted; keep the test-only defer ordering from `966eb49`.
- **H-R9-020 final accounting:** closed; the count=4 ownership partition remains `2 UDP reliable-confirmed / 1 TCP uncertain replay / 1 post-return reliable / 4 total`.
- **Reversed Session-ACK order:** current built-binary test proves one buffered offset-16 ACK, exact applied offsets 0 then 16, no covered shortcut, and terminal `remaining_in_flight=0`.
- **Post-return reliable ownership:** current source admits the reserved final record through `ReliableUdpRuntime`, waits for independent Session DeliveryAck and Carrier packet ACK, and requires post-return Recovery settlement before success.
- Earlier candidate A (`Recovery::on_ack` future/never-sent rejection) and candidate B (`record_datagrams` mixed drop reasons) remain closed absent a contradictory reproducer.

# READY_LOCAL — finish R9-2 without inventing parallel fixtures

## 1. Exact P2 C1 — TCP replay identity/cardinality

Use the existing count=4 migration-back fixture. Do not create a second fixture merely to satisfy the oracle.

Require mechanically:

- exactly one client `tcp_delivery_ack_validated` for `seq=2`, stream 1, offset 32;
- exactly one corresponding server TCP DeliveryAck for the same logical range;
- zero TCP replay/ACK evidence for offsets 0/16 (already reliable-UDP-owned) and 48 (reserved post-return reliable).

The current test only counts one client TCP ACK. If the existing diagnostic lacks stream/offset fields, add the smallest secret-free diagnostic fields at the already-established evidence point; do not change Session/TCP semantics.

## 2. Exact P2 C2 — migration strictly precedes the one post-return send

Require exactly one selected event for each step and strict client order:

```text
udp_recovery_challenge_sent
  < udp_recovery_validated
  < udp_migrated_back
  < r9_udp_post_return_sent(stream=1, offset=48)
```

Offset 48 must never appear on `udp_uncertain_range_sent`, TCP replay, or any pre-promotion reliable send path. Current source ownership is intended to satisfy this; the process oracle must prove it mechanically.

## 3. Exact P2 C3 — server post-return identity/cardinality

Require exactly once each, with exact logical identity where applicable:

```text
udp_recovery_owner_started
  < udp_recovery_validated
  < udp_return_delivery_ack_sent(seq=3 / stream=1 / offset=48)
  < udp_return_packet_ack_sent(same post-return bounded owner)
```

The current test proves first-occurrence order only. Add only the minimum diagnostic identity needed to bind the events to the actual post-return record/owner.

## 4. Exact P2 C4 — client dual-domain settlement

Require exactly once each:

- Session DeliveryAck validation for stream 1 / offset 48;
- Carrier packet ACK `applied=true` for the post-return reliable owner;
- `r9_udp_post_return_settled` for stream 1 / offset 48 / `remaining_in_flight=0`.

The settled event must occur after both ACK-domain events. Do not infer exact settlement from broad event-name presence or from one domain alone.

## 5. Post-return ACK arrival-order challenge

Use the one existing bounded authenticated receive owner, one absolute operation deadline and one malformed budget. Exercise both deterministic orders:

- Session DeliveryAck -> Carrier packet ACK;
- Carrier packet ACK -> Session DeliveryAck.

Both must end in the same exact post-return settled state. Do not add a second receive owner or a fresh per-order deadline.

## 6. Automatic-health TCP replay exact ownership

This remains a real source-level review/repair lane. Current automatic-health code reduces `FailoverController::tcp_resend()`'s exact `(DataId, payload)` ownership set to `.len()` and then reconstructs replay records positionally from `records.skip(uncertain_start)`.

Challenge with a focused deterministic case where the controller's actual uncertain set is not safely representable by count alone. If reproduced, preserve the exact controller-owned `DataId/payload` set through TCP replay, map it to exactly one original logical record, and fail closed on identity/payload mismatch. Do not add a new retention policy or change Session/TCP semantics.

## 7. P4 acknowledgement-domain suppression tightening

Preserve both existing seams:

- missing Carrier packet ACK while Session DeliveryAck arrives;
- missing Session DeliveryAck while Carrier packet ACK arrives.

For each: require nonzero typed terminal failure; require no `r9_udp_in_flight_settled`, no `r9_udp_post_return_settled`, and no downstream health/failover/migration continuation. Replace broad terminal disjunctions with the stable existing exact terminal marker/error where possible.

## 8. R9-2 final developer-local exact-tree provenance

On the final pushed source/test SHA, in a clean safe checkout/worktree, run:

```text
PYTHONDONTWRITEBYTECODE=1 bash scripts/check.sh
git diff --check
```

Record exact pushed SHA, UTC start/end, Linux OS/arch, Rust stable version, exit codes, clean initial/final tree. Run the pinned decode fuzz commands only if decoder/parser/crypto framing changed. Hosted CI remains additional cross-evidence.

After this gate, continue immediately into R9-3. Do not wait for reviewer cadence.

# Continuous dependency-ordered queue after R9-2

9. **R9-3 Data-loss recovery:** suppress one reliable-owned Data only after congestion admission; suppression must not bypass ownership accounting; PTO only after its deadline; retransmit under a fresh packet number/nonce while retaining stable Session/frame identity; exactly-once logical delivery; independent Session and Carrier ACK domains; final Recovery zero or typed bounded failure.
10. **R9-4 ACK-loss + delayed original/reorder:** suppress one Carrier ACK, force a legitimate retransmit, then release the delayed original; one logical delivery only; Session dedup remains authoritative; packet numbers/nonces stay fresh; loss/PTO evidence is truthful; Recovery settles.
11. **R9-5 adversarial feedback correctness:** authenticated tamper/future-ACK/stale feedback are atomic and fail closed; rejected feedback must not mutate RTT/PTO/loss/cwnd/Session state; malformed input remains finite/panic-free.
12. **R9-6 ownership/resource boundedness:** every first send/retransmit consults congestion admission before ownership commit; refusal commits no logical/recovery state; one bounded retransmit-plaintext owner only; teardown releases retained state. Do not invent capacity-pressure policy values.
13. **R9-7 process/result truth:** Data, Carrier packet ACK, Session DeliveryAck, PTO/retransmit, Recovery, Session delivery, malformed budget and terminal result remain distinct and are emitted only after the claimed transition actually happened.
14. **R9-8/R9-9 warm TCP + health promotion:** authenticated/resume-bound warm standby carries no application Data before promotion; resolved UDP outcomes feed health/hysteresis; recoverable loss remains on UDP; PTO-only samples cannot erase later resolved loss; only a ready TCP path promotes.
15. **R9-10 uncertain Session replay + cleanup:** replay only genuine uncertain Session ranges over promoted TCP; draining UDP receives no new Data; Session dedup remains exact-once; TCP gets no duplicate packet-ACK layer; cover timeout/shutdown/cleanup negatives.
16. **R9-11/R9-12 coherent gate:** run the exact-tree local gate for the complete cross-process reliable-UDP/failover slice and reconcile only factual implementation/evidence claims; do not flip release authority flags.
17. **Q10 independent R9 integration review:** dedicated bounded challenge of the materially new cross-process surface: send/admission/recovery/demux/Session ACK/Carrier ACK/failover/migration/cleanup/diagnostics. A no-finding review is valid item-4 support; any BLOCKER/HIGH returns to smallest repair + regression + exact-tree gate.
18. **Q11/Q12 factual reconciliation:** update observability/status/release packet only after the independent review anchor is reachable; classify whether the new implementation creates one genuinely unresolved real-network question. Only a specific Q11-created `READY_LIVE` row may unlock a bounded changed-hypothesis self-owned VPS run.

# Item-4 / core-surface boundary

The pre-R9 deep item-4 sweep already challenged the earlier reliable engine, CarrierState/Manager, scheduler/flow accounting, adapters, SessionRuntime, observability, package/reproducibility, dependency/build, CLI portability/output, boundedness/validators and wire/parser surfaces.

The new **cross-process R9 integration surface** is materially new and still requires its own dedicated independent bounded challenge before item-4 support can be reconciled. Do not mark item 4 complete merely because R9 process tests turn green.

# VPS opportunity

**Not READY.** Standing authorization remains valid and the rental window remains valuable, but the repository's authoritative classification is still `READY_LIVE: none`. Current blocker class: local R9 correctness/evidence plus later independent review. Do not repeat HY2, warm-failover, periodic/soak, package lifecycle, migration-back, endpoint/key migration, IPv6, PLPMTUD, or Experimental Track evidence merely because the VPS is rented.

Unlock sequence:

```text
exact P2 C1-C4
  -> ACK-order + automatic replay identity + P4
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
