# ChatGPT reviewer handoff — exact `a27e79a` closes H-R9-026 source collapse; focused regressions still block R9-3

## Current repository truth

- Latest developer-owned source/test commit reviewed: exact `a27e79a44d11b31a06c45432b392cba05c7f0f45` (`fix(cli): primary Carrier branch separates accepted-empty from rejected (H-R9-026)`).
- Independent reviewer checkpoint: [`docs/reviews/reviewer-r9-p2-a27e79a-20260916.md`](reviews/reviewer-r9-p2-a27e79a-20260916.md), committed at reviewer docs anchor `f31ad7774b41345496bde13ee2594b8bc75fa9c5`.
- Hosted Rust cross-evidence on exact `a27e79a`: GitHub Actions run `35092126535` completed SUCCESS. `stable checks` ran `bash scripts/check.sh`; `nightly decode fuzz smoke` ran the pinned decoder fuzz build/run; both succeeded. Hosted CI is additional evidence only, not developer-local exact-tree provenance.
- Open PRs at review time: none.
- Final developer-local clean exact-tree provenance for R9-2 is still absent.
- `READY_LIVE: none`; release item 3 incomplete; item 4 incomplete; `RELEASE_CANDIDATE=false`; `PRODUCTION_READY=false`; `FREEZE=false`; `RELEASED=false`.

The external coding agent must synchronize to current `main` and continuously execute every dependency-ready slice below: implementation/review -> focused deterministic tests -> commit -> push -> next slice. Reviewer cadence is only a check frequency and is never a reason to idle.

## Accepted progress at exact `a27e79a`

H-R9-026's source-level result collapse is repaired and should not be re-opened without a contradictory exact-current reproducer:

- `recv_udp_delivery_ack` preserves three Carrier ACK classes from `ReliableUdpRuntime::apply_ack`: applied (`acked_packets` non-empty), accepted-empty stale/duplicate (`Ok` with empty `acked_packets`), and rejected (`Err`);
- the primary logical-confirmation caller now increments positive evidence only for `applied=true`, increments rejection evidence only for `rejected=true`, and consumes accepted-empty as a typed non-event;
- the settlement drain has the same classification and cannot manufacture settlement because `rt.in_flight()` remains authoritative;
- the post-return owner still requires actual `acked_packets.contains(post_pn_client)` for its positive packet-transition claim, and success requires both exact Session confirmation and Recovery zero.

**H-R9-026 source correctness defect is CLOSED at `a27e79a`.**

The acceptance/evidence gate remains **OPEN (HIGH)** because the required focused regressions did not land. Exact `d133dbb` and exact `a27e79a` changed `crates/neko-cli/src/main.rs` only; current `crates/neko-cli/tests/probe.rs` has no focused accepted-empty Carrier ACK test and no focused future/never-sent Carrier ACK test through the primary authenticated receive/demux owner.

R9-3 remains blocked until those regressions and the remaining R9-2 acceptance work close.

# READY_LOCAL 1 — close H-R9-026 acceptance with focused deterministic regressions

Do not change ACK framing, Session semantics, Carrier architecture, crypto/wire architecture, deadlines, malformed budgets, or any policy value. Do not create a second receive owner.

Use the existing authenticated ACK receive/demux seam while logical Session confirmation is still outstanding.

### A. Accepted-empty duplicate/stale

1. Cause a legitimate Carrier ACK retirement.
2. Feed the duplicate/stale canonical ACK through the same primary owner.
3. Require no new positive transition and no rejection event/counter from the duplicate itself.
4. Require no Session logical-confirmation/watermark mutation from the duplicate itself.
5. Require Recovery/Session accounting after the duplicate to equal the state established by the first real retirement.

### B. Rejected future/never-sent

1. Feed a canonical ACK with largest packet number greater than `largest_sent` through the same primary owner.
2. Require exactly one typed rejection and zero positive transition.
3. Snapshot and require unchanged RTT/PTO/loss/cwnd/packet ownership and Session logical state across the rejection.
4. Do not incorrectly require every acknowledged number to remain present in the sent map; the challenged invariant is future/never-sent atomic rejection.

Audit all current `UdpAcknowledgement::Carrier` consumers only for re-collapse. Do not redesign the result type unless an exact-current defect requires it.

## READY_LOCAL 2 — finish positive P2 C1-C4 exact closure

Use the existing count=4 fixture and preserve already-landed strong assertions; add only missing exact cardinality/identity/order proof.

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

Require:

- exactly one actual `r9_udp_return_delivery_ack` at `stream=1`, `offset=48`, `len=16`;
- exactly one positive actual Carrier retirement for the exact cross-bound post-return packet number;
- zero false/rejected shortcut;
- exactly one `r9_udp_post_return_settled` at `stream=1`, `offset=48`, `remaining_in_flight=0`;
- terminal settlement strictly after both actual transitions.

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

Future/never-sent/stale/duplicate/tampered feedback must be atomic and fail closed. Re-check candidate A against exact-current code. Rejected feedback cannot mutate RTT/PTO/loss/cwnd/Session state; accepted-empty feedback cannot be mislabeled as a packet transition **or as a rejection**. Exercise every current `UdpAcknowledgement::Carrier` caller, including post-return diagnostics, so process/result truth cannot re-collapse typed owner outcomes.

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

- H-R9-026 source outcome-class repair: exact `d133dbb` + exact `a27e79a`; focused regressions still pending before acceptance closure.
- H-R9-025 narrow positive-transition repair: post-return current-packet success is identity-bound to actual `acked_packets`.
- H-R9-024 cross-process client-send/server-ACK packet-number binding.
- H-R9-023 exact P3 malformed terminal oracle.
- H-R9-022 applied Carrier ACK position evidence.
- H-R9-021 malformed -> malformed -> Carrier ACK -> malformed source order.
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
