# ChatGPT reviewer handoff — H-R9-029/H-R9-028 block R9-3 on exact Carrier-ACK outcome proof

## Current repository truth

- Latest developer-owned source/test commit reviewed: exact `bbb3e7f5310dea6a2342a2eda212cbc34326fd89` (`fix(cli): stale/future ACK seams hit real Recovery branches (H-R9-027)`).
- New independent reviewer checkpoint: [`docs/reviews/reviewer-r9-h-r9-029-post-return-ack-outcome-bbb3e7f-20260917.md`](reviews/reviewer-r9-h-r9-029-post-return-ack-outcome-bbb3e7f-20260917.md), created after exact-current source review.
- Prior H-R9-028 checkpoint remains valid: [`docs/reviews/reviewer-r9-h-r9-028-bbb3e7f-20260916.md`](reviews/reviewer-r9-h-r9-028-bbb3e7f-20260916.md).
- Hosted Rust cross-evidence on exact `bbb3e7f`: GitHub Actions run `35104209670` completed SUCCESS. `stable checks` ran `bash scripts/check.sh`; `nightly decode fuzz smoke` ran the pinned decode fuzz commands. Hosted CI is cross-evidence only, not developer-local exact-tree provenance.
- Open PRs at review time: none.
- Final developer-local clean exact-tree provenance for the complete R9-2 source/test tree is still absent.
- `READY_LIVE: none`; release item 3 incomplete; item 4 incomplete; `RELEASE_CANDIDATE=false`; `PRODUCTION_READY=false`; `FREEZE=false`; `RELEASED=false`.

The external coding agent must synchronize to current `main` and continuously execute dependency-ready slices below: implementation/review -> focused deterministic tests -> commit -> push -> next slice. Reviewer cadence is only a check frequency and is never a reason to idle.

## Accepted progress that must remain closed

Do not revert these exact-current repairs without a contradictory reproducer:

- H-R9-027 source shape at exact `bbb3e7f`: stale semantic ACKs are re-sealed under a fresh authenticated envelope; future ACKs use a non-empty range whose largest exceeds any bounded `largest_sent`, so the intended Recovery guards are reachable.
- H-R9-026 shared-owner + primary/settlement classification at exact `d133dbb` + `a27e79a`: real retirement, accepted-empty stale/duplicate, and typed Recovery rejection remain distinct; primary and settlement counters count only the true class.
- H-R9-025: post-return current-packet success remains identity-bound to actual `acked_packets` containing the exact post-return packet number.
- H-R9-024 cross-process client-send/server-ACK packet-number binding.
- H-R9-023 exact P3 malformed terminal oracle.
- H-R9-022 applied Carrier ACK position evidence.
- H-R9-021 malformed -> malformed -> Carrier ACK -> malformed source order.
- H-R9-020 count=4 final accounting: `2 reliable UDP + 1 uncertain TCP + 1 post-return reliable = 4`.
- reversed initial Session ACK handling: later ACK buffers until the Session watermark reaches it; no cumulative over-promotion.
- candidate A engine guard: future/never-sent largest is rejected before RTT/loss/PTO mutation.
- candidate B mixed queue/generic-drop observability repair remains accepted absent contradictory exact-current evidence.

# READY_LOCAL 1 — H-R9-029 HIGH: post-return Carrier outcome classification must remain typed

Exact-current post-return migration-back code still consumes:

```rust
Ok(UdpAcknowledgement::Carrier {
    applied,
    acked_packets,
    ..
})
```

and therefore discards `rejected`. It emits the same `r9_udp_return_packet_ack applied=false ... retired=false` shape for both:

- legal accepted-empty stale/duplicate ACKs; and
- future/never-sent ACKs atomically rejected by Recovery.

The dual-domain success gate is still conservative (`Session` confirmation plus `rt.in_flight()==0`), so this does not manufacture success, but it re-collapses process/result truth at one current `UdpAcknowledgement::Carrier` caller and leaves item-4 evidence unable to distinguish two materially different authenticated outcomes.

Smallest repair only; no ACK/Session/Carrier/crypto/wire redesign:

1. retain `rejected` in the existing post-return consumer;
2. emit mutually exclusive diagnostics:
   - current positive retirement only when `acked_packets` contains exact `post_pn_client`;
   - accepted-empty classification-only diagnostic, no applied/rejected/delivery/health/failover mutation;
   - distinct typed rejection diagnostic, with no claim that the current packet was ACKed;
3. if an accepted ACK retires a different packet, do not claim current post-return retirement; remain in the same bounded receive owner until the existing dual-domain success condition or deadline;
4. add deterministic built-binary post-return regressions for exactly one fresh-envelope stale semantic ACK and exactly one future/never-sent ACK after migration-back;
5. stale case: exactly one accepted-empty classification, zero rejection, no extra positive retirement, exact Session confirmation, Recovery zero, client/server success;
6. future case: exactly one typed rejection, no false current-packet retirement, later legitimate Carrier progress, exact Session confirmation, Recovery zero, client/server success.

Reuse H-R9-028 one-shot injection machinery where practical. Do not add a second receive owner, second ACK architecture, new deadline, new malformed budget, or new capacity value.

# READY_LOCAL 2 — H-R9-028 HIGH: one-shot and discriminating initial stale/future regressions

The exact `bbb3e7f` source seams now reach the intended Recovery branches, but the current process regressions are still false-positiveable.

## A. Accepted-empty stale/duplicate

Current server injection is inside every packet-ACK emission and sends the re-sealed semantic duplicate before the ordinary ACK. Existing process proof only checks no rejection substring plus eventual Recovery zero; it does not prove an accepted-empty classification was actually consumed.

Repair contract:

1. one-shot stale injection ownership outside the per-record emission loop;
2. exactly one fresh-envelope semantic duplicate while the bounded operation is active;
3. emit `r9_udp_packet_ack_accepted_empty` (or equivalent) only for `!applied && !rejected && acked_packets.is_empty()`; diagnostic only;
4. client/server success;
5. exactly one accepted-empty classification, zero rejection, no extra positive Carrier retirement, unchanged Session logical-confirmation cardinality, final Recovery zero.

## B. Future/never-sent rejection

Current future injection also repeats for every packet-ACK emission; the test only requires at least one rejection substring.

Repair contract:

1. one-shot future injection ownership;
2. exactly one typed rejection;
3. no false positive retirement attributable to the future ACK;
4. subsequent legitimate Carrier ACK progress and final Recovery zero;
5. client/server success and unchanged Session logical completion;
6. preserve/reuse the engine-level atomic future-ACK regression; snapshot directly exposed RTT/PTO/Reno/in-flight state only where useful, without inventing policy values.

The intended H-R9-029/H-R9-028 repairs do not change decoder/framing semantics, so pinned decode fuzz is not required solely because of these slices. Run focused deterministic tests and then the normal local gate on the final pushed source/test tree.

**R9-3 remains blocked until H-R9-029 and H-R9-028 are both closed.**

# READY_LOCAL 3 — positive P2 C1-C4 exact closure

Keep the existing count=4 fixture and strengthen only missing identity/cardinality/order proof.

- **C1 TCP replay identity:** exactly once client and server `stream=1`, `offset=32` (`seq=2` where emitted); explicitly reject replay evidence for offsets 0, 16, 48.
- **C2 migration ownership chain:** exactly once and strict order `udp_recovery_challenge_sent < udp_recovery_validated < udp_migrated_back < r9_udp_post_return_sent(stream=1,offset=48,packet_number=<pn>)`; offset 48 absent from uncertain/pre-promotion/TCP replay ownership.
- **C3 server dual-domain:** exactly one Session DeliveryAck `stream=1 offset=48 len=16` plus exactly one Carrier ACK for the exact client-owned post-return packet number; do not merge domains.
- **C4 client actual transitions:** exactly one Session transition for `stream=1 offset=48 len=16`; exactly one positive Carrier retirement for the cross-bound packet number; zero false/rejected shortcut; exactly one `r9_udp_post_return_settled ... remaining_in_flight=0`; settlement strictly after both actual transitions.

# READY_LOCAL 4 — post-return ACK arrival-order challenge

Through the same authenticated receive owner, same absolute deadline and same malformed budget, deterministically cover:

- Session DeliveryAck -> Carrier packet ACK;
- Carrier packet ACK -> Session DeliveryAck.

Both must converge to one logical confirmation, one exact packet retirement, no false/rejected shortcut, and Recovery zero. A test seam may reorder already-produced authenticated ACK datagrams only; no new protocol owner or policy value.

# READY_LOCAL 5 — P4 Session ACK present / Carrier ACK withheld

Require positive exact Session transition, typed bounded nonzero terminal failure because Recovery remains unsettled, no settled marker, and no downstream health/failover/migration success continuation.

# READY_LOCAL 6 — P4 Carrier ACK present / Session ACK withheld

Require positive exact Carrier retirement, typed bounded nonzero terminal failure because logical confirmation remains outstanding, no settled marker, and no downstream success continuation.

# READY_LOCAL 7 — R9-2 developer-local exact-tree provenance

On the final pushed R9-2 source/test SHA in a clean safe checkout/worktree run at least:

```text
PYTHONDONTWRITEBYTECODE=1 bash scripts/check.sh
git diff --check
```

Record exact reachable pushed SHA, UTC start/end, Linux OS/arch, Rust stable version, both exit codes, and clean initial/final tree. Run pinned decoder fuzz only if decoder/parser/crypto framing actually changed. Hosted CI remains separate cross-evidence.

Then continue immediately; do not wait for reviewer cadence.

# READY_LOCAL 8-15 — preserve the deep R9 / item-4 queue

## 8. R9-3 Data-loss recovery

Suppress one reliable-owned Data only after congestion admission; suppression must not bypass ownership accounting. PTO only after its deadline; retransmit under a fresh packet number/nonce while retaining stable frame/logical identity; exactly-once Session delivery; independent ACK domains; final Recovery zero or typed bounded failure.

## 9. R9-4 ACK-loss + delayed original/reorder

Suppress one Carrier ACK, force legitimate retransmission, then release the delayed original. Require one logical delivery, Session dedup authority, fresh packet numbers/nonces, truthful loss/PTO evidence, and settled Recovery.

## 10. R9-5 adversarial feedback

Future/never-sent/stale/duplicate/tampered feedback must be atomic and fail closed. Rejected feedback cannot mutate RTT/PTO/loss/cwnd/Session state; accepted-empty cannot be mislabeled as transition or rejection. Re-check every current `UdpAcknowledgement::Carrier` caller, including initial, settlement, and post-return diagnostics.

## 11. R9-6 ownership/resource boundedness

Every first send/retransmit must consult congestion admission before ownership commit; refusal commits no recovery/logical state. Keep one bounded retransmit plaintext owner and deterministic teardown. Do not invent capacity values.

## 12. R9-7 process/result truth

Data, Carrier ACK, Session DeliveryAck, PTO/retransmit, Recovery, Session delivery, malformed budget, accepted-empty feedback, rejected feedback and terminal result remain distinct. Emit each event only after the exact claimed state transition.

## 13. R9-8 warm TCP readiness

Authenticated resume-bound warm standby carries no application Data before promotion. Readiness/resource admission remains distinct from delivery evidence.

## 14. R9-9/R9-10 health, promotion, uncertain replay and cleanup

Resolved UDP outcomes feed existing health/hysteresis; recoverable loss remains on UDP; promotion only after existing readiness/health gates. Replay only genuine uncertain Session ranges over promoted TCP; draining/failed UDP gets no new Data; Session dedup exact-once; TCP gets no duplicate packet-ACK layer; cover shutdown/cleanup negatives.

## 15. R9-11/R9-12 + Q10/Q11/Q12 coherent closure

Run the clean exact-tree local gate for the complete cross-process reliable-UDP/failover slice, then perform a dedicated independent bounded challenge of the materially new R9 send/admission/recovery/demux/Session ACK/Carrier ACK/failover/migration/cleanup/diagnostic surface. A no-finding review is valid item-4 support; any BLOCKER/HIGH returns to smallest repair + regression + exact-tree gate. Only after a reachable independent review anchor may status/release packet be factually reconciled. Do not flip release-authority flags.

# Item-4 / core-surface inventory

Reachable independent bounded review already exists for the earlier reliable-UDP engine basics, CarrierState, Carrier Manager/health/migration-back, FairScheduler/flow accounting, carrier adapters, SessionRuntime, observability, package/reproducibility, dependency/build, CLI portability/output, algorithmic boundedness/validators, DeliveryLedger/process codec/datagram/crypto API, and wire/parser surfaces.

The materially new cross-process R9 integration is not yet independently closed. Queue exhaustion is false.

# VPS opportunity

**Not READY.** Standing authorization remains valid, but authoritative classification remains `READY_LIVE: none`. Current blocker class is local R9 correctness/evidence plus later independent cross-process R9 review. Do not repeat HY2, warm failover, periodic/soak, package lifecycle, migration-back, endpoint/key migration, IPv6, PLPMTUD or Experimental Track evidence merely because the VPS remains rented.

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
