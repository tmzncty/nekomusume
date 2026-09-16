# ChatGPT reviewer handoff — `0d5d90c` closes H-R9-024; finish R9-2 exact evidence continuously

## Current repository truth

- Latest developer-owned source/test commit reviewed: exact `0d5d90ce436024dbfa3f1c333ddc6f8cd4e33a86` (`test(cli): cross-process Carrier packet_number bind (H-R9-024 complete)`).
- Independent reviewer checkpoint: [`docs/reviews/reviewer-r9-p2-0d5d90c-20260916.md`](reviews/reviewer-r9-p2-0d5d90c-20260916.md), committed at reviewer docs anchor `60e1d57dd50978ebb5999eabcfad55643f7d2edc`.
- Hosted Rust cross-evidence on exact `0d5d90c`: GitHub Actions run `35065452165` completed SUCCESS. `stable checks` ran `bash scripts/check.sh`; `nightly decode fuzz smoke` ran pinned `cargo fuzz build decode` and `cargo fuzz run decode -- -max_total_time=30 -max_len=8192`; both succeeded. Hosted CI is additional evidence, not the final developer-local exact-tree provenance.
- Open PRs: none.
- Final developer-local clean exact-tree provenance for R9-2 is still absent.
- No new WAN/VPS experiment in this sequence.
- `READY_LIVE: none`; release item 3 incomplete; item 4 incomplete; `RELEASE_CANDIDATE=false`; `PRODUCTION_READY=false`; `FREEZE=false`; `RELEASED=false`.

The external coding agent must synchronize to current `main` and continuously execute every dependency-ready slice below: implementation/review -> focused tests -> exact-tree local gate where appropriate -> commit -> push -> next slice. Reviewer cadence is only a check frequency and is never a reason to idle.

## Newly accepted progress

### H-R9-024 closed narrowly at exact `0d5d90c`

The post-return positive P2 fixture now mechanically cross-binds the Carrier packet sequence space:

- client `r9_udp_post_return_sent` is emitted immediately after the exact `pn` is registered via the existing `ReliableUdpRuntime::on_packet_sent`, and carries `stream`, logical `offset`, and `packet_number=pn`;
- server `udp_return_packet_ack_sent` carries the actual `post_pn` passed to `server_rt.on_packet_received`;
- the count=4 built-binary fixture extracts client/server `packet_number` and requires exact equality.

Session DeliveryAck identity remains separate stream/offset evidence. Do not regress to using logical offset as a Carrier packet-number oracle.

No new BLOCKER/HIGH was found in this repair. The numeric test extraction currently falls back to `u64::MAX` if parsing fails; exact current diagnostic shape makes the value parseable and the fixture is green, so this is not a current correctness finding. If the diagnostic is reshaped later, prefer explicit parse failure rather than sentinel equality.

### Preserve earlier accepted R9-2 progress

- H-R9-023 P3 exact terminal oracle is closed at `2bddf1d`.
- P2 C1/C2 process evidence at `ae0bea8` already enforces positive client/server success, a one-record TCP replay partition at logical offset 32, no reserved offset-48 uncertain send, one post-return send, and migration-before-post-return-send.
- P2 C3/C4 diagnostics/tests at `c2d4d1a` plus `4b07c60`/`0d5d90c` already establish one server Session ACK line, one server Carrier ACK line, one actual client Session ACK application, one applied client Carrier ACK, terminal Recovery zero, and cross-process Carrier packet-number equality.
- H-R9-022 applied Carrier ACK evidence is closed at `779efab`; H-R9-021 malformed -> Carrier ACK -> malformed source ordering remains preserved from `966eb49`.
- H-R9-020 final count=4 accounting remains `2 reliable UDP + 1 uncertain TCP replay + 1 post-return reliable = 4`.
- Reversed initial Session ACK ordering remains covered by its built-binary fixture: offset 16 is buffered while watermark is 0, then actual confirmation is applied in offset 0 -> 16 order with no covered shortcut and terminal Recovery zero.
- Earlier candidate A (`Recovery::on_ack` future/never-sent rejection) and candidate B (`record_datagrams` mixed drop classification) remain closed absent a contradictory reproducer.

# READY_LOCAL — continuous R9 queue

## 1. Finish P2 C1 exact TCP replay identity/cardinality

Use the existing count=4 fixture. Require exactly once on both client and server:

- `seq=2`;
- stream 1;
- offset 32.

Keep explicit negative evidence for offsets 0/16/48. Do not infer identity from count alone.

## 2. Finish P2 C2 one-only migration -> post-return ownership chain

Require each selected milestone exactly once and in strict order:

```text
udp_recovery_challenge_sent
  < udp_recovery_validated
  < udp_migrated_back
  < r9_udp_post_return_sent(stream=1, offset=48, packet_number=<cross-bound pn>)
```

Keep offset 48 absent from `udp_uncertain_range_sent`, TCP replay, and any pre-promotion reliable ownership evidence. `find()` order alone is not a cardinality proof.

## 3. Finish P2 C3 exact server dual-domain evidence

Require exactly once after recovery-owner start/validation:

```text
udp_recovery_owner_started
  < udp_recovery_validated
  < udp_return_delivery_ack_sent
  < udp_return_packet_ack_sent
```

Session DeliveryAck must be `seq=3`, stream 1, offset 48, len 16. Carrier packet ACK must carry the exact packet number already cross-bound to client Recovery ownership. Keep the two domains separate.

## 4. Finish P2 C4 actual client dual-domain settlement

The authoritative Session mutation event is `r9_udp_return_delivery_ack`; the later `udp_return_delivery_ack_validated` summary is not a mutation-order oracle.

Require:

- exactly one actual Session DeliveryAck application at stream 1 / offset 48 / len 16 (add `len=16` to that actual diagnostic if necessary);
- exactly one `r9_udp_return_packet_ack` with `applied=true` and no rejected/false shortcut;
- exactly one `r9_udp_post_return_settled` at stream 1 / offset 48 / `remaining_in_flight=0`;
- settlement strictly after both actual ACK-domain events.

## 5. Post-return ACK-order challenge

Through the same bounded authenticated receive owner, same absolute operation deadline, and same malformed budget, deterministically exercise both orders:

- Session DeliveryAck -> Carrier packet ACK;
- Carrier packet ACK -> Session DeliveryAck.

Both must converge to the same exact terminal state: one logical confirmation, one applied Carrier ACK, no false/rejected shortcut, Recovery zero. A test-only reorder seam may reorder already-produced authenticated ACK datagrams; it must not create a second protocol owner or policy value.

## 6. P4 — Session ACK present / Carrier ACK withheld

Require positive observation/application of the Session ACK domain, typed nonzero bounded terminal failure because Carrier recovery remains unsettled, no `r9_udp_post_return_settled`, no false Recovery-settled evidence, and no downstream success/health/failover/migration continuation.

## 7. P4 — Carrier ACK present / Session ACK withheld

Require positive observation/application of the Carrier ACK domain, typed nonzero bounded terminal failure because logical Session confirmation remains outstanding, no `r9_udp_post_return_settled`, no false logical-settled evidence, and no downstream success/health/failover/migration continuation.

## 8. R9-2 final developer-local exact-tree provenance

On the final pushed source/test SHA in a clean safe checkout/worktree run at least:

```text
PYTHONDONTWRITEBYTECODE=1 bash scripts/check.sh
git diff --check
```

Record exact reachable pushed SHA, UTC start/end, Linux OS/arch, Rust stable version, exit codes, clean initial/final tree. Only run the pinned decoder fuzz commands when decoder/parser/crypto framing actually changed. Hosted CI remains separate cross-evidence.

Then continue immediately without waiting for reviewer cadence.

## 9. R9-3 Data-loss recovery

Suppress one reliable-owned Data only after congestion admission; suppression must not bypass ownership accounting. PTO only after its deadline; retransmit under a fresh packet number/nonce while retaining stable Session/frame identity; exactly-once logical delivery; independent Session and Carrier ACK domains; final Recovery zero or typed bounded failure.

## 10. R9-4 ACK-loss + delayed original/reorder

Suppress one Carrier ACK, force a legitimate retransmit, then release the delayed original. Require one logical delivery only, Session dedup authoritative, fresh packet numbers/nonces, truthful loss/PTO evidence, and settled Recovery.

## 11. R9-5/R9-6 adversarial feedback + ownership/resource boundedness

Authenticated tamper/future-ACK/stale feedback must be atomic and fail closed: rejected feedback cannot mutate RTT/PTO/loss/cwnd/Session state; malformed input stays finite/panic-free. Re-check candidate A against the exact current engine. Every first send/retransmit must consult congestion admission before ownership commit; refusal commits no logical/recovery state; keep one bounded retransmit-plaintext owner and release retained state on teardown. Do not invent capacity policy values.

## 12. R9-7 process/result truth

Data, Carrier packet ACK, Session DeliveryAck, PTO/retransmit, Recovery, Session delivery, malformed budget, and terminal result stay distinct and are emitted only after the claimed transition actually occurs.

## 13. R9-8..R9-10 warm TCP / health / promotion / uncertain replay / cleanup

Authenticated resume-bound warm standby carries no application Data before promotion. Resolved UDP outcomes feed health/hysteresis; recoverable loss remains on UDP; PTO-only samples cannot erase later resolved loss; only a ready TCP path promotes. Replay only genuine uncertain Session ranges over promoted TCP; draining UDP receives no new Data; Session dedup remains exact-once; TCP gets no duplicate packet-ACK layer; cover timeout/shutdown/cleanup negatives.

## 14. R9-11/R9-12 + Q10/Q11/Q12 coherent closure

Run the clean exact-tree local gate for the complete cross-process reliable-UDP/failover slice, then perform a dedicated independent bounded challenge of the materially new cross-process R9 send/admission/recovery/demux/Session ACK/Carrier ACK/failover/migration/cleanup/diagnostic surface. A no-finding review is valid item-4 support; any BLOCKER/HIGH returns to smallest repair + regression + exact-tree gate. Only after a reachable independent review anchor may status/release packet be factually reconciled. Do not flip release-authority flags.

# Item-4 / core-surface inventory

The earlier deep item-4 sweep already has reachable independent bounded review coverage for reliable-UDP ACK/recovery basics, CarrierState, Carrier Manager/health/migration-back, FairScheduler/flow accounting, carrier adapters, SessionRuntime, observability, package/reproducibility, dependency/build, CLI portability/output, algorithmic boundedness/validators, DeliveryLedger/process codec/datagram/crypto API, and wire/parser surfaces. The materially new cross-process R9 integration surface is not yet independently closed and remains valid item-4 work even if all process tests become green.

Queue exhaustion is therefore false.

# VPS opportunity

**Not READY.** Standing authorization remains valid and the rental window remains valuable, but authoritative classification is still `READY_LIVE: none`. Current blocker class: local R9 correctness/evidence plus later independent cross-process R9 review. Do not repeat HY2, warm failover, periodic/soak, package lifecycle, migration-back, endpoint/key migration, IPv6, PLPMTUD, or Experimental Track evidence merely because the VPS remains rented.

Only create a new `READY_LIVE` row if later code/instrumentation/hypothesis/path conditions produce a concrete unresolved real-network question.

# Separate non-blocking policy / authority gates

Do not invent or change while executing the queue above:

- `SessionRuntime.events` retention/capacity policy;
- D019 source-retention/no-reset policy;
- RSEC-001 adversarial-load/capacity suitability conditions;
- signing/key custody/SBOM/publication trust;
- previous frozen-release interoperability policy;
- core Session/Carrier/ACK/crypto/wire architecture;
- destructive/canonical-meaning migration;
- final RC/freeze/release/production authority.

These gates do not justify idling dependency-ready local R9 work.
