# ChatGPT reviewer handoff — P2 C1/C2 advanced at `ae0bea8`; exact post-return C3/C4 next

## Current repository truth

- Latest developer-owned source/test commit reviewed: exact `ae0bea87c45b8197cae6e4e6aa8268e171f54389` (`test(cli): P2 C1-C2 exact identity/cardinality — TCP replay seq2 offset32 + post-return send once after migrated`).
- Independent reviewer checkpoint: [`docs/reviews/reviewer-r9-p2-ae0bea-20260916.md`](reviews/reviewer-r9-p2-ae0bea-20260916.md).
- `ae0bea8` is evidence-oriented implementation + process-test work: it enriches existing TCP DeliveryAck diagnostics with stream/offset fields and tightens the existing count=4 P2 process fixture. It does **not** change Session/Carrier/ACK/wire semantics.
- Hosted Rust cross-evidence on exact `ae0bea8`: `stable checks` SUCCESS (`bash scripts/check.sh`) and `nightly decode fuzz smoke` SUCCESS. This remains hosted evidence, not the required final developer-local clean exact-tree provenance.
- Open PRs: none.
- No new WAN/VPS experiment in this sequence.
- Final developer-local clean exact-tree provenance for R9-2 is still absent.
- `READY_LIVE: none`; release item 3 incomplete; item 4 incomplete; `RELEASE_CANDIDATE=false`; `PRODUCTION_READY=false`; `FREEZE=false`; `RELEASED=false`.

The external coding agent must synchronize to current `main` and continue every dependency-ready slice below without waiting for reviewer cadence. Reviewer cadence is a check frequency, not a work-ticket length. Adjacent evidence assertions may be landed together when coherent, but the deep queue must remain visible and must not collapse to one small ticket.

# Accepted progress — preserve it

- **H-R9-023 P3 terminal oracle:** closed narrowly at `2bddf1d`.
- **P2 C1 progress at `ae0bea8`:** existing fixture now has one-only client/server TCP DeliveryAck evidence and binds the client to stream 1 / offset 32 and the server to offset 32; client-side negative replay evidence excludes offsets 0/16/48.
- **P2 C2 progress at `ae0bea8`:** exactly one post-return send is required at offset 48 and `udp_migrated_back < r9_udp_post_return_sent` is proven; earlier challenge/validation/migration ordering remains green.
- **Automatic-health TCP replay ownership challenge:** bounded **no finding** at `ae0bea8`. Current reachable CLI constructs the `FailoverController` uncertain set exactly from immutable contiguous `records[uncertain_start..uncertain_end]` using `DataId(record.offset)` + `record.data`; `tcp_resend().len()` therefore selects the same current slice, and permitted matching UDP progress before promotion is terminal rather than partially retiring the set. Do not manufacture a helper/refactor merely to preserve `(DataId,payload)` under current semantics. Re-open only if future code allows sparse/partially-confirmed uncertainty, a different payload source, or multi-stream identity mapping where offset alone is insufficient.
- **H-R9-022 applied Carrier-ACK evidence:** closed narrowly at `779efab`.
- **H-R9-021 source ordering:** accepted; preserve the test-only defer ordering from `966eb49`.
- **H-R9-020 final accounting:** closed; count=4 remains `2 reliable UDP + 1 uncertain TCP replay + 1 post-return reliable = 4`.
- **Reversed initial Session-ACK order:** current built-binary fixture proves buffered offset 16, exact applied offsets 0 then 16, no covered shortcut, and terminal Recovery zero.
- **Post-return reliable ownership:** current source admits the reserved final record through `ReliableUdpRuntime`, waits for independent Session DeliveryAck and Carrier packet ACK, and requires post-return Recovery settlement before success.
- Earlier candidate A (`Recovery::on_ack` future/never-sent rejection) and candidate B (`record_datagrams` mixed drop reasons) remain closed absent a contradictory reproducer.

# READY_LOCAL — finish R9-2 continuously

## 1. Finish exact P2 C1 + C2 in the existing count=4 fixture

Do not create a parallel migration-back fixture.

### C1 TCP replay identity/cardinality

Keep one-only client and server TCP DeliveryAck evidence and make the selected evidence mechanically exact:

- client: `seq=2`, stream 1, offset 32, exactly once;
- server: `seq=2`, stream 1, offset 32, exactly once;
- no TCP replay/ACK evidence for offsets 0/16 (reliable-UDP-owned) or 48 (reserved post-return reliable).

The diagnostics already carry stream/offset after `ae0bea8`; this should be test-oracle tightening only unless a missing field is discovered.

### C2 migration precedes the one post-return send

Require one-only selected milestones and strict client order:

```text
udp_recovery_challenge_sent
  < udp_recovery_validated
  < udp_migrated_back
  < r9_udp_post_return_sent(stream=1, offset=48)
```

Require the post-return send exactly once. Keep offset 48 absent from `udp_uncertain_range_sent`, TCP replay, and any pre-promotion reliable ownership evidence. Do not infer one-only from `find()` alone.

## 2. Exact P2 C3 — server post-return identity/cardinality

The current server source truth is Session DeliveryAck first, then Carrier packet ACK for the same bounded post-return Data packet. Preserve that semantic separation; do not require one ACK domain to stand in for the other.

Require exactly once each, after the recovery owner is established and recovery validation succeeds:

```text
udp_recovery_owner_started
  < udp_recovery_validated
  < udp_return_delivery_ack_sent
  < udp_return_packet_ack_sent
```

Add only minimum secret-free evidence at the existing emit points:

- `udp_return_delivery_ack_sent`: `seq=3`, stream 1, offset 48, len 16 in the current fixture;
- `udp_return_packet_ack_sent`: bind to the actual received post-return packet number/owner. Prefer an explicit packet number field rather than pretending packet ACK is a logical offset ACK.
- If needed for cross-process binding, add the same post-return packet number to `r9_udp_post_return_sent` at the already-established `on_packet_sent` point.

Then assert one-only cardinality and exact identity. No new ACK format, Session semantics or receive owner.

## 3. Exact P2 C4 — client dual-domain settlement

Make the existing post-return success proof exact without creating another receive loop:

- exactly one Session DeliveryAck validation for stream 1 / offset 48 / len 16;
- exactly one post-return Carrier ACK with `applied=true`; reject/veto false/rejected evidence for this positive fixture;
- exactly one `r9_udp_post_return_settled` for stream 1 / offset 48 / `remaining_in_flight=0`;
- settlement strictly after both ACK-domain events.

`udp_return_delivery_ack_validated` currently carries only ciphertext size; add stream/offset/len at that already-established point. For the Carrier domain, the current loop has only the post-return recovery packet outstanding, so one-only `applied=true` + terminal `in_flight=0` is sufficient; do not redesign the demux solely to manufacture a logical offset on a packet ACK.

## 4. Post-return ACK arrival-order challenge

Exercise both deterministic delivery orders through the **same** bounded authenticated receive owner, one absolute operation deadline and one malformed budget:

- Session DeliveryAck -> Carrier packet ACK (current server order);
- Carrier packet ACK -> Session DeliveryAck (test-only bounded reorder seam).

Both must produce the same exact C4 terminal state: one Session confirmation, one applied Carrier ACK, no false/rejected shortcut, and final Recovery zero. A test seam may reorder the two already-produced authenticated ACK datagrams; it must not create a second protocol owner or new policy value.

## 5. P4 acknowledgement-domain suppression tightening

Preserve the two existing independent negative seams:

- Session DeliveryAck arrives, Carrier packet ACK is withheld;
- Carrier packet ACK arrives, Session DeliveryAck is withheld.

For each require:

- nonzero client terminal failure with the existing typed/stable error for the missing domain;
- no `r9_udp_post_return_settled` and no false Recovery-settled evidence;
- no downstream success/health/failover/migration continuation after the terminal point;
- the non-suppressed ACK domain is positively observed, proving the test did not simply fail before reaching the seam.

Do not replace this with a broad timeout-only oracle.

## 6. R9-2 final developer-local exact-tree provenance

On the final pushed source/test SHA, in a clean safe checkout/worktree, run:

```text
PYTHONDONTWRITEBYTECODE=1 bash scripts/check.sh
git diff --check
```

Record exact reachable pushed SHA, UTC start/end, Linux OS/arch, Rust stable version, exit codes, clean initial/final tree. Run the pinned decode fuzz commands only if decoder/parser/crypto framing changed. Hosted CI remains additional cross-evidence.

After this gate, continue immediately into R9-3; do not wait for reviewer cadence.

# Continuous dependency-ordered queue after R9-2

7. **R9-3 Data-loss recovery:** suppress one reliable-owned Data only after congestion admission; suppression must not bypass ownership accounting; PTO only after its deadline; retransmit under a fresh packet number/nonce while retaining stable Session/frame identity; exactly-once logical delivery; independent Session and Carrier ACK domains; final Recovery zero or typed bounded failure.
8. **R9-4 ACK-loss + delayed original/reorder:** suppress one Carrier ACK, force a legitimate retransmit, then release the delayed original; one logical delivery only; Session dedup remains authoritative; packet numbers/nonces stay fresh; loss/PTO evidence truthful; Recovery settles.
9. **R9-5 adversarial feedback correctness:** authenticated tamper/future-ACK/stale feedback are atomic and fail closed; rejected feedback must not mutate RTT/PTO/loss/cwnd/Session state; malformed input finite/panic-free. Re-check candidate A against the exact current engine rather than trusting stale history.
10. **R9-6 ownership/resource boundedness:** every first send/retransmit consults congestion admission before ownership commit; refusal commits no logical/recovery state; one bounded retransmit-plaintext owner only; teardown releases retained state. Do not invent capacity-pressure policy values. If multi-stream failover identity is introduced here, re-open the `DataId(offset)` exclusion from the `ae0bea8` no-finding review.
11. **R9-7 process/result truth:** Data, Carrier packet ACK, Session DeliveryAck, PTO/retransmit, Recovery, Session delivery, malformed budget and terminal result remain distinct and emitted only after the claimed transition actually occurred.
12. **R9-8/R9-9 warm TCP + health promotion:** authenticated/resume-bound warm standby carries no application Data before promotion; resolved UDP outcomes feed health/hysteresis; recoverable loss remains on UDP; PTO-only samples cannot erase later resolved loss; only a ready TCP path promotes.
13. **R9-10 uncertain Session replay + cleanup:** replay only genuine uncertain Session ranges over promoted TCP; draining UDP receives no new Data; Session dedup remains exact-once; TCP gets no duplicate packet-ACK layer; cover timeout/shutdown/cleanup negatives. Re-open automatic replay ownership only if this work creates sparse/partially-retired uncertainty.
14. **R9-11/R9-12 coherent gate:** run the exact-tree local gate for the complete cross-process reliable-UDP/failover slice and reconcile factual implementation/evidence claims only; do not flip release-authority flags.
15. **Q10 independent R9 integration review -> Q11/Q12 reconciliation:** dedicated bounded challenge of send/admission/recovery/demux/Session ACK/Carrier ACK/failover/migration/cleanup/diagnostics. A no-finding review is valid item-4 support; any BLOCKER/HIGH returns to smallest repair + regression + exact-tree gate. Only after a reachable independent review anchor may status/release packet be reconciled and a genuinely new real-network question classified.

# Item-4 / core-surface boundary

The pre-R9 deep item-4 sweep already challenged the earlier reliable engine, CarrierState/Manager, scheduler/flow accounting, adapters, SessionRuntime, observability, package/reproducibility, dependency/build, CLI portability/output, boundedness/validators and wire/parser surfaces.

The materially new **cross-process R9 integration surface** still requires its own dedicated independent bounded challenge before item-4 support can be reconciled. Do not mark item 4 complete merely because R9 process tests turn green.

# VPS opportunity

**Not READY.** Standing authorization remains valid and the rental window remains valuable, but authoritative classification remains `READY_LIVE: none`. Current blocker class: local R9 correctness/evidence plus later independent review. Do not repeat HY2, warm-failover, periodic/soak, package lifecycle, migration-back, endpoint/key migration, IPv6, PLPMTUD or Experimental Track evidence merely because the VPS is rented.

Unlock sequence:

```text
exact P2 C1-C4
  -> ACK-order + P4
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
