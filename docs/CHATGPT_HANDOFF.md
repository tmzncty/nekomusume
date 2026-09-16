# ChatGPT reviewer handoff — `0534397` exposes H-R9-025; keep R9-3 blocked

## Current repository truth

- Latest developer-owned source/test commit reviewed: exact `0534397121f989d5ebb42f6b50bea5fa1d9a3f91` (`test(cli): P2 C4 exact dual-domain — len + packet_number on actual ACK events, settled after both (M-R9-008)`).
- Independent reviewer checkpoint: [`docs/reviews/reviewer-r9-p2-0534397-20260916.md`](reviews/reviewer-r9-p2-0534397-20260916.md), committed at reviewer docs anchor `2f31474e457deae1b0bd0d8c607a6704a16af388`.
- Hosted Rust cross-evidence on exact `0534397`: GitHub Actions run `35070406429` completed SUCCESS. `stable checks` ran `bash scripts/check.sh`; `nightly decode fuzz smoke` ran the pinned decode fuzz build/run; both succeeded. Hosted CI is additional evidence, not developer-local exact-tree provenance.
- Open PRs: none.
- Final developer-local clean exact-tree provenance for R9-2 is still absent.
- `READY_LIVE: none`; release item 3 incomplete; item 4 incomplete; `RELEASE_CANDIDATE=false`; `PRODUCTION_READY=false`; `FREEZE=false`; `RELEASED=false`.

The external coding agent must synchronize to current `main` and continuously execute every dependency-ready slice below: implementation/review -> focused deterministic tests -> commit -> push -> next slice. Reviewer cadence is only a check frequency and is never a reason to idle.

## Newly accepted progress at `0534397`

The P2 post-return path now emits richer actual-transition diagnostics:

- `r9_udp_return_delivery_ack` carries `stream`, `offset`, and `len`;
- `r9_udp_return_packet_ack` carries `applied` and a `packet_number` field;
- the process oracle orders terminal settlement after the actual Session/Carrier ACK events rather than the later summary event.

Earlier H-R9-024 cross-process send/server-ACK packet-number equality remains present. Hosted checks are green.

These changes are useful, but they do **not** close P2 C4 because the Carrier event currently overclaims what recovery actually proved.

# HIGH H-R9-025 — actual Carrier ACK diagnostic is not bound to the actual recovery outcome

`recv_udp_delivery_ack` currently reduces `ReliableUdpRuntime::apply_ack(...)` to:

```text
UdpAcknowledgement::Carrier { applied: apply_ack(...).is_ok() }
```

This conflates **canonical ACK accepted by recovery** with **the current packet actually retired**.

Current recovery semantics intentionally allow an authenticated duplicate/stale ACK at or below `largest_sent` to return `Ok` with an empty `acked_packets` / empty resolved outcome. `PathRecovery::on_ack` explicitly treats such an empty ACK as no new resolved transition.

The post-return caller nevertheless emits:

```text
r9_udp_return_packet_ack applied=true packet_number=<post_pn_client>
```

where `packet_number` is copied from the local expected packet, not from `RecoveryAckOutcome`. Therefore a stale/duplicate authenticated Carrier ACK can be reported as application of the current post-return packet even while that packet remains in flight. A later correct ACK or terminal timeout does not make the earlier event truthful.

This is a process/result evidence correctness HIGH and blocks R9-3 expansion.

## READY_LOCAL 1 — close H-R9-025 with the smallest repair

Do not change ACK framing, Session semantics, Carrier architecture, wire/crypto architecture, or policy values.

1. Preserve enough of the existing `RecoveryAckOutcome` through the shared authenticated receive owner to distinguish:
   - ACK accepted but no recovery transition;
   - actual packet(s) acknowledged/retired;
   - typed rejection/error.
2. In the post-return owner, emit the positive `r9_udp_return_packet_ack` for `post_pn_client` only when the actual outcome contains that exact packet number in `acked_packets`.
3. A valid-but-stale/empty canonical ACK must not be represented as application of the current packet. Keep it under the same owner, absolute deadline, and existing bounded negative handling; do not create another receive loop or policy value.
4. Add a focused deterministic regression proving a stale/duplicate canonical ACK may be accepted with an empty outcome but cannot produce the positive current-packet transition.
5. Keep terminal success gated on both exact Session confirmation and Recovery `in_flight()==0`.

## READY_LOCAL 2 — finish positive P2 C1-C4 exact closure

Use the existing count=4 fixture and require exact cardinality/identity/order, not mere event presence.

### C1 — TCP replay identity

Exactly once on client and server: stream 1 / offset 32 (seq 2 where emitted). Explicitly reject TCP replay evidence for offsets 0, 16, and 48.

### C2 — migration -> post-return ownership chain

Require exactly once and strict order:

```text
udp_recovery_challenge_sent
  < udp_recovery_validated
  < udp_migrated_back
  < r9_udp_post_return_sent(stream=1, offset=48, packet_number=<pn>)
```

Keep offset 48 absent from uncertain/pre-promotion/TCP replay ownership.

### C3 — server dual-domain evidence

Require exactly once after recovery validation:

- Session DeliveryAck: stream 1 / offset 48 / len 16;
- Carrier packet ACK: the exact packet number cross-bound to client `on_packet_sent` ownership.

Do not merge the two evidence domains.

### C4 — client actual transitions

After H-R9-025 repair require:

- exactly one actual `r9_udp_return_delivery_ack` at stream 1 / offset 48 / **len 16**;
- exactly one positive actual Carrier transition for the exact cross-bound post-return packet number;
- zero false/rejected shortcuts;
- exactly one `r9_udp_post_return_settled` at stream 1 / offset 48 / `remaining_in_flight=0`;
- terminal settlement strictly after both actual transitions.

The exact `0534397` test currently checks Session stream/offset but does not assert the newly added `len=16`, and checks Carrier `applied=true` but does not bind that actual client application event back to the cross-bound packet number. Close both gaps.

## READY_LOCAL 3 — post-return ACK arrival-order challenge

Through the same bounded authenticated receive owner, same absolute deadline, and same malformed budget, deterministically exercise both:

- Session DeliveryAck -> Carrier packet ACK;
- Carrier packet ACK -> Session DeliveryAck.

Both must converge to one logical confirmation, one exact Carrier packet retirement, no false/rejected shortcut, and Recovery zero. A test seam may reorder already-produced authenticated ACK datagrams only; no second protocol owner or new policy value.

## READY_LOCAL 4 — P4 independent ACK-domain suppression negatives

### Session ACK present / Carrier ACK withheld

Require positive actual Session application, typed bounded nonzero terminal failure because Recovery remains unsettled, no settled marker, and no downstream health/failover/migration success continuation.

### Carrier ACK present / Session ACK withheld

Require positive exact Carrier packet retirement, typed bounded nonzero terminal failure because logical confirmation remains outstanding, no settled marker, and no downstream success continuation.

## READY_LOCAL 5 — R9-2 final developer-local exact-tree provenance

On the final pushed source/test SHA in a clean safe checkout/worktree run at least:

```text
PYTHONDONTWRITEBYTECODE=1 bash scripts/check.sh
git diff --check
```

Record exact reachable pushed SHA, UTC start/end, Linux OS/arch, Rust stable version, exit codes, and clean initial/final tree. Run pinned decoder fuzz only if decoder/parser/crypto framing actually changed. Hosted CI remains separate cross-evidence.

Then continue immediately without waiting for reviewer cadence.

# READY_LOCAL 6-14 — preserve the deep R9 / item-4 queue

## 6. R9-3 Data-loss recovery

Suppress one reliable-owned Data only after congestion admission; suppression must not bypass ownership accounting. PTO only after its deadline; retransmit under a fresh packet number/nonce while retaining stable frame/logical identity; exactly-once Session delivery; independent ACK domains; final Recovery zero or typed bounded failure.

## 7. R9-4 ACK-loss + delayed original/reorder

Suppress one Carrier ACK, force legitimate retransmission, then release the delayed original. Require one logical delivery, Session dedup authority, fresh packet numbers/nonces, truthful loss/PTO evidence, and settled Recovery.

## 8. R9-5 adversarial feedback

Future/never-sent/stale/duplicate/tampered feedback must be atomic and fail closed. Re-check candidate A against exact-current code. Rejected feedback cannot mutate RTT/PTO/loss/cwnd/Session state; accepted-but-empty feedback cannot be mislabeled as a packet transition.

## 9. R9-6 ownership/resource boundedness

Every first send/retransmit must consult congestion admission before ownership commit; refusal commits no recovery/logical state. Maintain one bounded retransmit plaintext owner and deterministic teardown. Do not invent capacity values.

## 10. R9-7 process/result truth

Data, Carrier packet ACK, Session DeliveryAck, PTO/retransmit, Recovery, Session delivery, malformed budget, and terminal result remain distinct. Emit each event only after the exact claimed state transition actually occurred.

## 11. R9-8 warm TCP readiness

Authenticated resume-bound warm standby carries no application Data before promotion. Keep readiness/resource admission distinct from delivery evidence.

## 12. R9-9 health + promotion

Resolved UDP recovery outcomes feed health/hysteresis. Recoverable loss remains on UDP; PTO-only evidence cannot erase later resolved loss; promote TCP only after existing readiness/health gates.

## 13. R9-10 uncertain replay + cleanup

Replay only genuine uncertain Session ranges over the promoted TCP owner; draining/failed UDP gets no new Data; Session dedup stays exact-once; TCP gets no duplicate packet-ACK layer; cover cleanup/shutdown negatives.

## 14. R9-11/R9-12 + Q10/Q11/Q12 coherent closure

Run the clean exact-tree local gate for the complete cross-process reliable-UDP/failover slice, then perform a dedicated independent bounded challenge of the materially new R9 send/admission/recovery/demux/Session ACK/Carrier ACK/failover/migration/cleanup/diagnostic surface. A no-finding review is valid item-4 support; any BLOCKER/HIGH returns to smallest repair + regression + exact-tree gate. Only after a reachable independent review anchor may status/release packet be factually reconciled. Do not flip release-authority flags.

# Accepted earlier closures that must not regress

- H-R9-024 cross-process client-send/server-ACK packet-number binding: exact `0d5d90c`.
- H-R9-023 exact P3 malformed terminal oracle: `2bddf1d`.
- H-R9-022 applied Carrier ACK position evidence: `779efab`.
- H-R9-021 malformed -> malformed -> Carrier ACK -> malformed source order: `966eb49`.
- H-R9-020 count=4 final accounting: `2 reliable UDP + 1 uncertain TCP + 1 post-return reliable = 4`.
- Reversed initial Session ACK process fixture: later ACK buffered while watermark 0, then offset 0 -> 16 actual application order, no covered shortcut, terminal Recovery zero.
- Candidate A future/never-sent ACK rejection remains present in current `Recovery::on_ack` before mutation.
- Candidate B mixed queue/generic drop observability repair remains accepted absent contradictory current reproducer.

# Item-4 / core-surface inventory

Reachable independent bounded review already exists for the earlier reliable-UDP engine basics, CarrierState, Carrier Manager/health/migration-back, FairScheduler/flow accounting, carrier adapters, SessionRuntime, observability, package/reproducibility, dependency/build, CLI portability/output, algorithmic boundedness/validators, DeliveryLedger/process codec/datagram/crypto API, and wire/parser surfaces.

The materially new cross-process R9 integration is not yet independently closed. Queue exhaustion is false.

# VPS opportunity

**Not READY.** Standing authorization remains valid, but authoritative classification remains `READY_LIVE: none`. Current blocker class is local R9 correctness/evidence plus later independent cross-process R9 review. Do not repeat HY2, warm failover, periodic/soak, package lifecycle, migration-back, endpoint/key migration, IPv6, PLPMTUD, or Experimental Track evidence merely because the VPS remains rented.

Only create a new `READY_LIVE` row if later code/instrumentation/hypothesis/path conditions create a concrete unresolved real-network question.

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
