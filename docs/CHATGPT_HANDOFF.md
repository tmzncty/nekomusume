# ChatGPT reviewer handoff — exact `d174eb6` partially repairs H-R9-025; H-R9-026 blocks R9-3

## Current repository truth

- Latest developer-owned source/test commit reviewed: exact `d174eb620eeb92df9e6181c555595591448017e5` (`fix(cli): r9_udp_return_packet_ack only claims applied on real packet retire (H-R9-025)`).
- Independent reviewer checkpoint: [`docs/reviews/reviewer-r9-p2-d174eb6-20260916.md`](reviews/reviewer-r9-p2-d174eb6-20260916.md), committed at reviewer docs anchor `e01e931222b977fa0ccb486b223f2e629c8d02da`.
- Hosted Rust cross-evidence on exact `d174eb6`: GitHub Actions run `35081304355` completed SUCCESS. `stable checks` ran `bash scripts/check.sh`; `nightly decode fuzz smoke` ran the pinned decoder fuzz build/run; both succeeded. Hosted CI is additional evidence only, not developer-local exact-tree provenance.
- Open PRs: none.
- Final developer-local clean exact-tree provenance for R9-2 is still absent.
- `READY_LIVE: none`; release item 3 incomplete; item 4 incomplete; `RELEASE_CANDIDATE=false`; `PRODUCTION_READY=false`; `FREEZE=false`; `RELEASED=false`.

The external coding agent must synchronize to current `main` and continuously execute every dependency-ready slice below: implementation/review -> focused deterministic tests -> commit -> push -> next slice. Reviewer cadence is only a check frequency and is never a reason to idle.

## Accepted progress at `d174eb6`

Retain the useful H-R9-025 repair:

- the shared reliable-UDP ACK classifier now surfaces `acked_packets` from an accepted `RecoveryAckOutcome`;
- the post-return caller derives `retired = acked_packets.contains(&post_pn_client)`;
- `r9_udp_return_packet_ack` no longer claims `applied=true` for the current post-return packet merely because a canonical ACK was syntactically accepted.

This closes the narrow false-positive transition that motivated H-R9-025, but does **not** yet close the owner semantics or acceptance evidence because the repair erases the difference between accepted-empty and rejected feedback, and no focused regression was added.

# HIGH H-R9-026 — accepted-empty and rejected Carrier ACKs are still collapsed

Current exact `d174eb6` code in `recv_udp_delivery_ack` does:

```text
let outcome = rt
    .as_deref_mut()
    .map(|r| r.apply_ack(&ranges, 0, ack.ack_delay_us).ok());
let acked_packets = outcome
    .flatten()
    .map(|o| o.acked_packets)
    .unwrap_or_default();
let applied = !acked_packets.is_empty();
return Ok(UdpAcknowledgement::Carrier { applied, acked_packets });
```

`Result::ok()` discards `PathRecoveryError`, so two different current semantics become indistinguishable:

1. a valid stale/duplicate canonical ACK accepted by recovery with an empty resolved outcome;
2. a future/never-sent canonical ACK rejected atomically by recovery (`largest > largest_sent`).

Both become `Carrier { applied:false, acked_packets:[] }`.

That ambiguity already leaks into existing callers which map `applied == false` to a `packet_ack_rejected` event/counter. A valid accepted-empty duplicate can therefore be mislabeled as rejected, while a genuinely rejected future ACK is no longer exposed as a typed recovery rejection at this owner. The post-return success gate remains safe because `in_flight()` does not reach zero on an empty/rejected outcome, but process/result truth and the later R9-5 adversarial-feedback lane require exact separation of these outcome classes.

R9-3 remains blocked.

## READY_LOCAL 1 — close H-R9-026 with the smallest owner repair

Do not change ACK framing, Session semantics, Carrier architecture, crypto/wire architecture, or any policy value.

1. Preserve the `Result<RecoveryAckOutcome, PathRecoveryError>` distinction through the existing authenticated receive owner; do not call `.ok()` and erase errors.
2. Expose three explicit semantic classes using the smallest local type shape:
   - Session DeliveryAck;
   - Carrier ACK accepted, carrying the actual `acked_packets` (possibly empty);
   - typed Carrier recovery rejection/error.
3. A positive packet-transition event may be emitted only from actual `acked_packets` identity.
4. Accepted-empty stale/duplicate ACKs must not increment or emit a rejection counter/event. Future/never-sent ACKs must be typed rejected and must not be represented as accepted-empty.
5. Preserve the single receive owner, existing absolute deadline, and operation-wide malformed budget. Do not add a second loop, retry policy, TTL/LRU/history/capacity value, or new security number.

### Required deterministic regressions

Use the current authenticated ACK seam; do not create a second protocol owner.

- **Accepted-empty duplicate/stale:** a canonical duplicate/stale ACK is accepted with an empty recovery outcome, produces no positive current-packet transition, is not mislabeled rejected, and does not mutate Session state or any recovery state beyond current committed duplicate semantics.
- **Rejected future/never-sent:** a canonical ACK whose largest packet exceeds `largest_sent` is surfaced as typed rejected; it produces no positive transition and leaves RTT/PTO/loss/cwnd/packet/Session state unchanged.

The exact `d174eb6` commit changed only `main.rs`; it added no focused regression, so this slice is not optional.

## READY_LOCAL 2 — finish positive P2 C1-C4 exact closure

Use the existing count=4 fixture and require exact cardinality/identity/order, not event presence.

### C1 — TCP replay identity

Exactly once on client and server: `stream=1`, `offset=32` (`seq=2` where emitted). Explicitly reject TCP replay evidence for offsets 0, 16, and 48.

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

- Session DeliveryAck: `stream=1`, `offset=48`, `len=16`;
- Carrier packet ACK: exact packet number cross-bound to client `on_packet_sent` ownership.

Do not merge the two evidence domains.

### C4 — client actual transitions

After H-R9-026 repair require:

- exactly one actual `r9_udp_return_delivery_ack` at `stream=1`, `offset=48`, `len=16`;
- exactly one positive actual Carrier retirement for the exact cross-bound post-return packet number;
- zero false/rejected shortcut;
- exactly one `r9_udp_post_return_settled` at `stream=1`, `offset=48`, `remaining_in_flight=0`;
- terminal settlement strictly after both actual transitions.

The current process test still needs the `len=16` assertion and exact client-applied Carrier packet-number binding; do not treat the source-only `d174eb6` fix as evidence closure.

## READY_LOCAL 3 — post-return ACK arrival-order challenge

Through the same bounded authenticated receive owner, same absolute deadline, and same malformed budget, deterministically exercise both:

- Session DeliveryAck -> Carrier packet ACK;
- Carrier packet ACK -> Session DeliveryAck.

Both must converge to one logical confirmation, one exact Carrier packet retirement, no false/rejected shortcut, and Recovery zero. A test seam may reorder already-produced authenticated ACK datagrams only; no second protocol owner or new policy value.

## READY_LOCAL 4 — P4 Session-ACK-present / Carrier-ACK-withheld negative

Require:

- positive exact Session ACK transition;
- typed bounded nonzero terminal failure because Recovery remains unsettled;
- no settled marker;
- no downstream health/failover/migration success continuation.

## READY_LOCAL 5 — P4 Carrier-ACK-present / Session-ACK-withheld negative

Require:

- positive exact Carrier packet retirement;
- typed bounded nonzero terminal failure because logical confirmation remains outstanding;
- no settled marker;
- no downstream success continuation.

## READY_LOCAL 6 — R9-2 final developer-local exact-tree provenance

On the final pushed source/test SHA in a clean safe checkout/worktree run at least:

```text
PYTHONDONTWRITEBYTECODE=1 bash scripts/check.sh
git diff --check
```

Record exact reachable pushed SHA, UTC start/end, Linux OS/arch, Rust stable version, exit codes, and clean initial/final tree. Run pinned decoder fuzz only if decoder/parser/crypto framing actually changed. Hosted CI remains separate cross-evidence.

Then continue immediately without waiting for reviewer cadence.

# READY_LOCAL 7-15 — preserve the deep R9 / item-4 queue

## 7. R9-3 Data-loss recovery

Suppress one reliable-owned Data only after congestion admission; suppression must not bypass ownership accounting. PTO only after its deadline; retransmit under a fresh packet number/nonce while retaining stable frame/logical identity; exactly-once Session delivery; independent ACK domains; final Recovery zero or typed bounded failure.

## 8. R9-4 ACK-loss + delayed original/reorder

Suppress one Carrier ACK, force legitimate retransmission, then release the delayed original. Require one logical delivery, Session dedup authority, fresh packet numbers/nonces, truthful loss/PTO evidence, and settled Recovery.

## 9. R9-5 adversarial feedback

Future/never-sent/stale/duplicate/tampered feedback must be atomic and fail closed. Re-check candidate A against exact-current code. Rejected feedback cannot mutate RTT/PTO/loss/cwnd/Session state; accepted-empty feedback cannot be mislabeled as a packet transition **or as a rejection**.

## 10. R9-6 ownership/resource boundedness

Every first send/retransmit must consult congestion admission before ownership commit; refusal commits no recovery/logical state. Maintain one bounded retransmit plaintext owner and deterministic teardown. Do not invent capacity values.

## 11. R9-7 process/result truth

Data, Carrier packet ACK, Session DeliveryAck, PTO/retransmit, Recovery, Session delivery, malformed budget, accepted-empty feedback, rejected feedback, and terminal result remain distinct. Emit each event only after the exact claimed state transition actually occurred.

## 12. R9-8 warm TCP readiness

Authenticated resume-bound warm standby carries no application Data before promotion. Keep readiness/resource admission distinct from delivery evidence.

## 13. R9-9 health + promotion

Resolved UDP recovery outcomes feed health/hysteresis. Recoverable loss remains on UDP; PTO-only evidence cannot erase later resolved loss; promote TCP only after existing readiness/health gates.

## 14. R9-10 uncertain replay + cleanup

Replay only genuine uncertain Session ranges over the promoted TCP owner; draining/failed UDP gets no new Data; Session dedup stays exact-once; TCP gets no duplicate packet-ACK layer; cover cleanup/shutdown negatives.

## 15. R9-11/R9-12 + Q10/Q11/Q12 coherent closure

Run the clean exact-tree local gate for the complete cross-process reliable-UDP/failover slice, then perform a dedicated independent bounded challenge of the materially new R9 send/admission/recovery/demux/Session ACK/Carrier ACK/failover/migration/cleanup/diagnostic surface. A no-finding review is valid item-4 support; any BLOCKER/HIGH returns to smallest repair + regression + exact-tree gate. Only after a reachable independent review anchor may status/release packet be factually reconciled. Do not flip release-authority flags.

# Accepted earlier closures that must not regress

- H-R9-025 narrow positive-transition repair at exact `d174eb6`: post-return current-packet success is now identity-bound to actual `acked_packets`; H-R9-026 separately remains open for accepted-empty vs rejected classification.
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
