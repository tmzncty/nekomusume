# ChatGPT reviewer handoff — R9-2 blocked on applied Carrier-ACK evidence at `966eb49`

## Current repository truth

- Latest developer-owned source/test commit reviewed: exact `966eb49775296a46d59e6326404ea012b29eea7b` (`fix(cli): P3 malformed->CarrierACK->malformed ordering + typed bound evidence (H-R9-021)`).
- Reviewer checkpoint: [`docs/reviews/reviewer-r9-p3-applied-evidence-966eb49-20260916.md`](reviews/reviewer-r9-p3-applied-evidence-966eb49-20260916.md).
- Exact `966eb49` repairs the **server source ordering** for P3, but the built-binary regression still does not prove the interleaved Carrier ACK was successfully applied before malformed #3. H-R9-022 below is therefore the current STOP front.
- Hosted Rust CI on exact `966eb49`: `stable checks` SUCCESS (`bash scripts/check.sh`) and `nightly decode fuzz smoke` SUCCESS. Hosted CI is cross-evidence only.
- Open PRs: none.
- No new WAN/VPS experiment in this sequence.
- Final developer-local clean exact-tree provenance for R9-2 is still absent.
- `READY_LIVE: none`; item 3 incomplete; item 4 incomplete; `RELEASE_CANDIDATE=false`; `PRODUCTION_READY=false`; `FREEZE=false`; `RELEASED=false`.

The external coding agent must synchronize to current `main` and continue every dependency-ready slice below without waiting for reviewer cadence. Reviewer cadence is a check frequency, not a work-ticket length.

# STOP FRONT — H-R9-022 P3 does not prove the interleaved Carrier ACK was applied

**Severity: HIGH for R9-2 evidence correctness.** Mechanically repairable under current semantics; no maintainer policy or architecture decision required.

The server-side `--malformed-budget-test` seam now sends in the intended source order:

```text
malformed #1 -> malformed #2 -> canonical Carrier packet ACK -> Session DeliveryAck -> malformed #3
```

Preserve that implementation. The remaining defect is the **acceptance evidence**.

Current client receive ownership returns `UdpAcknowledgement::Carrier { applied }`, but the outer loop only increments counters. The applied counter is emitted later in `r9_udp_packet_ack_outcomes`, after logical settlement. P3 deliberately dies on malformed #3 before that later output, so the current test would still pass if the interleaved Carrier ACK were rejected (`applied=false`) or otherwise failed to create valid Carrier feedback, as long as malformed #3 still hits a terminal error.

The current test also still accepts the broad terminal condition:

```text
client_err contains "malformed" OR "bound"
```

and it does not mechanically forbid downstream health/failover continuation beyond the two settled markers.

## Required smallest repair

Do not change `MAX_POST_HANDSHAKE_MALFORMED`; do not redesign Session/Carrier/ACK/wire semantics; do not create a second receive owner.

1. Keep exact `966eb49` server send ordering unchanged.
2. At the existing outer reliable receive-owner point that handles `UdpAcknowledgement::Carrier { applied }`, emit the smallest diagnostic needed to prove the P3 event was actually applied. For this fixture the evidence must identify:
   - `applied=true`;
   - persistent operation-wide malformed count already exactly `2` at the Carrier-ACK handling point.
3. Tighten the built-binary P3 regression to require exactly one such applied event with `malformed_seen=2` (or equivalent exact fields), followed by the existing malformed-budget terminal error from malformed #3.
4. Require client nonzero exit unconditionally and pin the specific existing error `UDP delivery acknowledgement malformed bound exceeded` (or its exact stable terminal marker). Remove the generic `malformed || bound` disjunction.
5. Require no selected Carrier-ACK rejection for this interleaved event.
6. Require no `r9_udp_in_flight_settled`, no `r9_udp_post_return_settled`, and no downstream health/failover/migration continuation after the malformed bound terminates the operation.
7. Focused built-binary regression -> commit/push -> continue immediately. No decoder/framing fuzz is required solely for this diagnostic/test repair.

# Accepted progress — preserve it

## H-R9-021 source-order repair — ACCEPTED_WITH_REMAINING_EVIDENCE_GATE

Under the test seam the already-canonical Carrier ACK is deferred until after malformed #1/#2; ordinary non-test ordering remains unchanged. Session DeliveryAck remains a separate evidence domain. Do not revert this shape while closing H-R9-022.

## H-R9-020 final accounting — CLOSED

Current runtime derives `uncertain_records` from `uncertain_end - uncertain_start` and counts the reserved post-return reliable record only after the dual-domain post-return loop completes. The count=4 positive P2 fixture pins:

```text
udp_confirmed_records = 2
udp_confirmed_bytes   = 32
uncertain_records     = 1
uncertain_bytes       = 16
replayed_records      = 1
replayed_bytes        = 16
confirmed_records     = 4
confirmed_bytes       = 64
```

Do not reopen absent a contradictory reproducer.

## P4 acknowledgement-domain suppression — ACCEPT_WITH_BOUNDARIES

Preserve both seams:

- `--suppress-r9-ack`: Session DeliveryAck may arrive while Carrier packet ACK is withheld; in-flight must not be reported settled.
- `--suppress-r9-dack`: post-return Carrier packet ACK may arrive while Session DeliveryAck is withheld; client must terminate nonzero and must not emit post-return success.

Before R9-2 closure, tighten both tests so comments become mechanical assertions: no `r9_udp_in_flight_settled`, no `r9_udp_post_return_settled`, and no downstream health/failover continuation from incomplete settlement.

## Earlier candidates A/B — CLOSED on current tree

- `Recovery::on_ack` rejects `largest > largest_sent` before RTT/loss/PTO mutation.
- `neko-observe::record_datagrams` preserves mixed generic vs queue-full drop reasons and keeps `queue_dropped` a subset of `dropped`.

Do not reopen without a new reproducer.

# READY_LOCAL after H-R9-022 — exact P2 closure in the existing count=4 fixture

Do not create a parallel P2 scenario.

## P2-C1 — exact TCP replay identity/cardinality

Require:

- exactly one client TCP replay/DeliveryAck for `seq=2`, stream 1, offset 32;
- exactly one corresponding server TCP DeliveryAck for `seq=2` / offset 32;
- no TCP replay/ACK evidence for reliable-owned offsets 0/16 or reserved offset 48.

Use existing diagnostics; add only minimal stream/offset fields when exact identity is otherwise impossible.

## P2-C2 — migration -> exact post-return reliable send

Require strict order:

```text
udp_recovery_challenge_sent
  < udp_recovery_validated
  < udp_migrated_back
  < r9_udp_post_return_sent(offset=48)
```

Require exactly one post-return send at offset 48. Reserved offset 48 must never appear in any legacy uncertain/replay path.

## P2-C3 — exact server post-return identity/cardinality

Require exactly once each, in strict order:

```text
udp_recovery_owner_started
  < udp_recovery_validated
  < udp_return_delivery_ack_sent(seq=3 / offset=48)
  < udp_return_packet_ack_sent(same bounded owner)
```

Do not accept first-occurrence-only checks without cardinality/identity.

## P2-C4 — exact client dual-domain settlement

Require exactly once each:

- Session DeliveryAck validation for stream 1 / offset 48;
- Carrier packet ACK `applied=true` for the post-return owner;
- `r9_udp_post_return_settled` for stream 1 / offset 48 / `remaining_in_flight=0`.

The settled event must occur after **both** acknowledgement-domain events regardless of arrival order.

# R9-2 remaining closure slices — continue without waiting

1. **H-R9-022 repair + discriminating P3 regression** described above.
2. **Exact P2 C1-C4** in the existing count=4 fixture.
3. **Post-return ACK arrival-order seam:** one bounded receive owner must succeed for Session-ACK -> Carrier-ACK and Carrier-ACK -> Session-ACK; same terminal event; no fresh per-order deadline or second receive owner.
4. **Automatic-health replay exact-identity regression:** compare actual replay `(DataId, payload)` identities against the exact `FailoverController::tcp_resend()` set. If current contiguous reconstruction differs, consume exact controller ownership instead of cardinality.
5. **P4 tighten:** both missing-Carrier-ACK and missing-Session-ACK cases explicitly forbid every success/health/failover continuation marker.
6. **R9-2 exact-tree provenance:** on the final pushed source/test SHA run in a clean safe checkout/worktree:

```text
PYTHONDONTWRITEBYTECODE=1 bash scripts/check.sh
git diff --check
```

Record exact pushed SHA, UTC start/end, Linux OS/arch, Rust stable version, exit codes, clean initial/final tree. Run pinned decode fuzz only if decoder/parser/crypto framing changed. Hosted CI remains additional cross-evidence.

After these six slices, continue immediately into R9-3. Do not wait for reviewer cadence.

# Continuous dependency-ordered queue — preserve depth

7. **R9-3 Data-loss recovery:** suppress one reliable-owned Data only after admission; cwnd admission precedes suppression; PTO after deadline; retransmit under fresh packet number/nonce with stable Session/frame identity; exactly-once logical delivery; independent Session/Carrier ACK domains; final Recovery zero or typed bounded failure.
8. **R9-4 ACK-loss + delayed original/reorder:** suppress Carrier ACK, retransmit, release delayed original; one logical delivery; Session dedup; fresh packet numbers/nonces; truthful loss/PTO; settled recovery.
9. **R9-5 tamper/future-ACK/malformed negatives:** tamper mutates neither Recovery nor Session; future/never-sent ACK rejection remains atomic; stale/duplicate feedback fabricates no evidence; malformed input finite/panic-free.
10. **R9-6 pacing/cwnd/plaintext ownership:** every send/retransmit consults congestion admission; refusal commits no logical/recovery ownership; one bounded retransmit plaintext owner; teardown releases state.
11. **R9-7 process/result truth:** Data, packet ACK, Session ACK, PTO/retransmit, Recovery, Session delivery, malformed budget and terminal result stay distinct; emit only after the claimed transition.
12. **R9-8 authenticated warm TCP standby:** negotiation + Noise trust/authz + resume/readiness + resource admission; zero application Data before promotion.
13. **R9-9 resolved UDP health -> hysteresis -> real TCP promotion:** resolved packet outcomes drive health; recoverable loss stays UDP; PTO-only samples cannot erase later loss; only ready TCP promotes.
14. **R9-10 uncertain Session replay UDP -> TCP + cleanup matrix:** replay only genuine uncertain Session ranges over promoted TCP; draining UDP receives no new Data; exact-once Session dedup; no TCP packet ACK; include timeout/shutdown/cleanup negative matrix.
15. **R9-11/R9-12 independent closure:** coherent exact-tree gate, then dedicated independent bounded review of the entire new cross-process reliable-UDP integration surface. Any BLOCKER/HIGH returns to smallest repair + regression + re-gate; then Q10 observability reconciliation, Q11 factual release/status reconciliation, and only a specific Q11-created `READY_LIVE` row may unlock one changed-hypothesis self-owned VPS Q12 run.

# Core-surface / item-4 boundary

The pre-R9 item-4 sweep already challenged the earlier reliable engine, CarrierState/Manager, scheduler/flow accounting, adapters, SessionRuntime, observability, package/reproducibility, dependency/build, CLI portability/output, algorithmic boundedness/validators, wire/parser and candidate enhancement gates.

The new **cross-process R9 integration surface** is not covered by those historical notes. Its dedicated independent bounded review remains required before item-4 support can be reconciled. Item 4 therefore remains incomplete even if the local R9 tests become green.

# VPS opportunity

**Not READY — current blocker is local correctness/evidence + independent review, not permission.** Standing VPS authorization remains valid. Rental-window priority is acknowledged, but do not run a candidate whose P3 acceptance evidence still cannot prove the interleaved Carrier ACK applied successfully and whose R9 integration has not yet received the dedicated bounded review.

Unlock sequence:

```text
H-R9-022
  -> exact P2 + ACK-order + auto replay identity + P4
  -> clean R9-2 provenance
  -> R9-3..R9-12
  -> Q10/Q11
  -> specific READY_LIVE row
  -> one bounded changed-hypothesis self-owned VPS run
```

# Separate non-blocking policy/authority gates

`SessionRuntime.events` retention; D019 source retention/no-reset; RSEC-001 adversarial-load/capacity suitability; signing/key custody/SBOM/publication trust; previous frozen-release interoperability; final RC/freeze/release/production authority.

Do not invent policy values while working R9, and do not let these independent gates starve dependency-ready R9 implementation/review work.
