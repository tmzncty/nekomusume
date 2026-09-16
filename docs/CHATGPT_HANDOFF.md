# ChatGPT reviewer handoff — P2 C3/C4 advanced at `c2d4d1a`; Carrier-ACK identity remains HIGH

## Current repository truth

- Latest developer-owned source/test commit reviewed: exact `c2d4d1a507f69bbc768abe38d179518099131508` (`test(cli): P2 C3/C4 exact post-return dual-ACK identity + one-only cardinality (M-R9-008)`).
- Independent reviewer checkpoint: [`docs/reviews/reviewer-r9-p2-c2d4d1a-20260916.md`](reviews/reviewer-r9-p2-c2d4d1a-20260916.md).
- `c2d4d1a` is evidence-oriented CLI diagnostic/process-test work. It does **not** change Session/Carrier/ACK/crypto/wire architecture.
- Hosted Rust cross-evidence on exact `c2d4d1a`: GitHub Actions run `35056977108` completed SUCCESS; `stable checks` ran `bash scripts/check.sh`, and `nightly decode fuzz smoke` also succeeded. This is hosted evidence, not final developer-local exact-tree provenance.
- Open PRs: none.
- Final developer-local clean exact-tree provenance for R9-2 is still absent.
- No new WAN/VPS experiment in this sequence.
- `READY_LIVE: none`; release item 3 incomplete; item 4 incomplete; `RELEASE_CANDIDATE=false`; `PRODUCTION_READY=false`; `FREEZE=false`; `RELEASED=false`.

The external coding agent must synchronize to current `main` and continue every dependency-ready slice below without waiting for reviewer cadence. Reviewer cadence is only a check frequency. Adjacent coherent repairs/tests may land together; the deep queue must remain visible.

# Accepted progress — preserve it

- **H-R9-023 P3 terminal oracle:** closed narrowly at `2bddf1d`.
- **P2 C1 progress at `ae0bea8`:** one-only client/server TCP DeliveryAck evidence exists; client binds stream 1 / offset 32; server binds offset 32; client negative evidence excludes offsets 0/16/48.
- **P2 C2 progress at `ae0bea8`:** exactly one post-return send at offset 48 is required and `udp_migrated_back < r9_udp_post_return_sent` is proven; earlier challenge/validation/migration ordering remains green.
- **P2 C3/C4 progress at `c2d4d1a`:** server Session DeliveryAck and packet-ACK events are one-only; client actual Session-ACK application event is one-only at stream 1 / offset 48; client packet ACK is one-only with `applied=true`; terminal settlement still requires stream 1 / offset 48 / Recovery zero.
- **Automatic-health TCP replay ownership challenge:** bounded no-finding at `ae0bea8` under current contiguous immutable uncertain-set semantics. Re-open only if future code permits sparse/partially-retired uncertainty, different payload ownership, or multi-stream identity where offset alone is insufficient.
- **H-R9-022 applied Carrier-ACK evidence:** closed narrowly at `779efab`.
- **H-R9-021 source ordering:** preserve the test-only defer ordering from `966eb49`.
- **H-R9-020 final accounting:** closed; count=4 remains `2 reliable UDP + 1 uncertain TCP replay + 1 post-return reliable = 4`.
- **Reversed initial Session-ACK order:** current built-binary fixture proves buffered offset 16, exact applied offsets 0 then 16, no covered shortcut, and terminal Recovery zero.
- **Post-return reliable ownership:** current source admits the reserved final record through `ReliableUdpRuntime`, waits for independent Session DeliveryAck and Carrier packet ACK, and requires post-return Recovery settlement before success.
- Earlier candidate A (`Recovery::on_ack` future/never-sent rejection) and candidate B (`record_datagrams` mixed drop reasons) remain closed absent a contradictory reproducer.

# HIGH — H-R9-024 Carrier packet ACK is mis-bound to Session identity

## Evidence-domain invariant

Session DeliveryAck and Carrier packet ACK are separate evidence domains. A Carrier packet-ACK proof must identify the Recovery packet / ACK range it owns, not borrow the logical Session stream/offset as if that were the packet sequence space.

## Exact current source truth

The server already derives the actual post-return packet number `pn` from the received authenticated datagram and calls:

```text
server_rt.on_packet_received(pn, true)
```

before `poll_outgoing_ack(0)` emits the packet ACK.

But `c2d4d1a` annotates `udp_return_packet_ack_sent` only with the logical Data `stream` / `offset`, and the current C3 test accepts `offset=48` as packet-ACK identity. That proves only that the diagnostic was emitted while processing logical offset 48; it does **not** mechanically prove that the outgoing ACK ranges acknowledge the same post-return Recovery packet registered by the client.

This is a release/evidence-integrity HIGH, not a newly demonstrated transport-runtime defect.

## Smallest repair contract

Do not change ACK framing, Session semantics, receive ownership, or policy values.

1. Preserve the actual post-return `pn` already available at `server_rt.on_packet_received` and emit it as secret-free `packet_number` on `udp_return_packet_ack_sent`.
2. Add the same `packet_number` to client `r9_udp_post_return_sent` at the existing `on_packet_sent` point so the process fixture can bind sender Recovery ownership to server packet-ACK emission.
3. Logical `stream` / `offset` may remain as correlation metadata, but the Carrier-domain assertion must use the packet number / real Carrier owner. Do not treat logical offset as packet-ACK identity.
4. Keep Session DeliveryAck evidence separate.

Close H-R9-024 before claiming C3/C4 exact closure.

# READY_LOCAL — finish R9-2 continuously

## 1. H-R9-024 exact Carrier packet identity

Repair the diagnostic/test binding above. No new wire or runtime owner.

## 2. Finish P2 C1 TCP replay identity/cardinality

Use the existing count=4 fixture. Require exactly once:

- client TCP DeliveryAck: `seq=2`, stream 1, offset 32;
- server TCP DeliveryAck: `seq=2`, stream 1, offset 32;
- no TCP replay/ACK evidence for offsets 0/16 (reliable-UDP-owned) or 48 (reserved post-return reliable).

Current test still does not mechanically assert all selected `seq`/stream fields.

## 3. Finish P2 C2 migration -> post-return ownership chain

Require one-only selected milestones and strict order:

```text
udp_recovery_challenge_sent
  < udp_recovery_validated
  < udp_migrated_back
  < r9_udp_post_return_sent(stream=1, offset=48, packet_number=<pn>)
```

Require the post-return send exactly once. Keep offset 48 absent from `udp_uncertain_range_sent`, TCP replay, and any pre-promotion reliable ownership evidence. Do not infer one-only from `find()` alone.

## 4. Finish exact P2 C3 server dual-domain evidence

Require exactly once after recovery ownership/validation:

```text
udp_recovery_owner_started
  < udp_recovery_validated
  < udp_return_delivery_ack_sent
  < udp_return_packet_ack_sent
```

- Session DeliveryAck: `seq=3`, stream 1, offset 48, len 16.
- Carrier packet ACK: bind to the actual post-return `packet_number`/owner from H-R9-024; do not use logical offset as the Carrier identity.
- Cross-process bind the server packet number to client `r9_udp_post_return_sent`.

Current `c2d4d1a` Session-ACK test still omits `seq=3`/stream/len exactness and packet-ACK packet identity.

## 5. Finish exact P2 C4 client dual-domain settlement

The useful actual Session-confirmation event is now `r9_udp_return_delivery_ack`; keep it one-only and add/assert stream 1 / offset 48 / len 16.

Require:

- exactly one actual Session DeliveryAck application;
- exactly one `r9_udp_return_packet_ack` with `applied=true` and no rejected/false shortcut;
- exactly one `r9_udp_post_return_settled` for stream 1 / offset 48 / `remaining_in_flight=0`;
- settlement strictly after both **actual** ACK-domain events.

Do not use only the later `udp_return_delivery_ack_validated` post-loop summary as the Session-domain ordering point; that event is emitted after the dual-settlement loop has already completed.

## 6. Post-return ACK arrival-order challenge

Exercise both deterministic orders through the same bounded authenticated receive owner, one absolute operation deadline and one malformed budget:

- Session DeliveryAck -> Carrier packet ACK;
- Carrier packet ACK -> Session DeliveryAck.

Both must converge to the same exact C4 state: one logical confirmation, one applied Carrier ACK, no false/rejected shortcut, Recovery zero. A test-only seam may reorder the two already-produced authenticated ACK datagrams; it must not create a second protocol owner or new policy value.

## 7. P4 ACK-domain suppression tightening

Preserve both independent negatives:

- Session DeliveryAck arrives, Carrier packet ACK withheld;
- Carrier packet ACK arrives, Session DeliveryAck withheld.

For each require nonzero typed terminal failure, no `r9_udp_post_return_settled`, no false Recovery-settled evidence, no downstream success/health/failover/migration continuation, and positive observation of the non-suppressed ACK domain so the test cannot fail vacuously before the seam.

## 8. R9-2 final developer-local exact-tree provenance

On the final pushed source/test SHA, clean safe checkout/worktree:

```text
PYTHONDONTWRITEBYTECODE=1 bash scripts/check.sh
git diff --check
```

Record exact reachable pushed SHA, UTC start/end, Linux OS/arch, Rust stable version, exit codes, clean initial/final tree. Only run the pinned decoder fuzz commands when decoder/parser/crypto framing changes. Hosted CI remains additional cross-evidence.

After this gate, continue immediately into R9-3 without waiting for reviewer cadence.

# Continuous dependency-ordered queue after R9-2

9. **R9-3 Data-loss recovery:** suppress one reliable-owned Data only after congestion admission; suppression must not bypass ownership accounting; PTO only after its deadline; retransmit under a fresh packet number/nonce while retaining stable Session/frame identity; exactly-once logical delivery; independent Session and Carrier ACK domains; final Recovery zero or typed bounded failure.
10. **R9-4 ACK-loss + delayed original/reorder:** suppress one Carrier ACK, force a legitimate retransmit, then release the delayed original; one logical delivery only; Session dedup authoritative; packet numbers/nonces fresh; loss/PTO evidence truthful; Recovery settles.
11. **R9-5 adversarial feedback correctness:** authenticated tamper/future-ACK/stale feedback atomic and fail closed; rejected feedback must not mutate RTT/PTO/loss/cwnd/Session state; malformed input finite/panic-free. Re-check candidate A against the exact current engine rather than trusting stale history.
12. **R9-6 ownership/resource boundedness:** every first send/retransmit consults congestion admission before ownership commit; refusal commits no logical/recovery state; one bounded retransmit-plaintext owner only; teardown releases retained state. Do not invent capacity-pressure policy values. Re-open the `DataId(offset)` no-finding only if this work creates sparse/partially-retired uncertainty or multi-stream identity ambiguity.
13. **R9-7 process/result truth:** Data, Carrier packet ACK, Session DeliveryAck, PTO/retransmit, Recovery, Session delivery, malformed budget and terminal result remain distinct and emitted only after the claimed transition actually occurred.
14. **R9-8/R9-9 warm TCP + health promotion:** authenticated/resume-bound warm standby carries no application Data before promotion; resolved UDP outcomes feed health/hysteresis; recoverable loss remains on UDP; PTO-only samples cannot erase later resolved loss; only a ready TCP path promotes.
15. **R9-10 uncertain Session replay + cleanup:** replay only genuine uncertain Session ranges over promoted TCP; draining UDP receives no new Data; Session dedup exact-once; TCP gets no duplicate packet-ACK layer; cover timeout/shutdown/cleanup negatives.
16. **R9-11/R9-12 coherent gate:** exact-tree local gate for the complete cross-process reliable-UDP/failover slice and factual implementation/evidence reconciliation only; do not flip release-authority flags.
17. **Q10 independent R9 integration review -> Q11/Q12 reconciliation:** dedicated bounded challenge of send/admission/recovery/demux/Session ACK/Carrier ACK/failover/migration/cleanup/diagnostics. A no-finding review is valid item-4 support; any BLOCKER/HIGH returns to smallest repair + regression + exact-tree gate. Only after a reachable independent review anchor may status/release packet be reconciled and a genuinely new real-network question classified.

# Item-4 / core-surface boundary

The pre-R9 deep item-4 sweep already challenged the earlier reliable engine, CarrierState/Manager, scheduler/flow accounting, adapters, SessionRuntime, observability, package/reproducibility, dependency/build, CLI portability/output, boundedness/validators and wire/parser surfaces.

The materially new **cross-process R9 integration surface** still requires its own dedicated independent bounded challenge before item-4 support can be reconciled. Do not mark item 4 complete merely because R9 process tests become green.

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
