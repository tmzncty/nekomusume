# ChatGPT reviewer handoff — exact `bbb3e7f` reaches the intended ACK branches; H-R9-028 keeps acceptance open on discriminating process proof

## Current repository truth

- Latest developer-owned source/test commit reviewed: exact `bbb3e7f5310dea6a2342a2eda212cbc34326fd89` (`fix(cli): stale/future ACK seams hit real Recovery branches (H-R9-027)`).
- Independent reviewer checkpoint: [`docs/reviews/reviewer-r9-h-r9-028-bbb3e7f-20260916.md`](reviews/reviewer-r9-h-r9-028-bbb3e7f-20260916.md), reviewer docs anchor `972d37a41b06a1b215c38253bad1486d839ab2e8`.
- Hosted Rust cross-evidence on exact `bbb3e7f`: GitHub Actions run `35104209670` completed SUCCESS. `stable checks` ran `bash scripts/check.sh`; `nightly decode fuzz smoke` ran the pinned `cargo fuzz build decode` and `cargo fuzz run decode -- -max_total_time=30 -max_len=8192`; both succeeded. Hosted CI is additional evidence only, not developer-local exact-tree provenance.
- Open PRs at review time: none.
- Final developer-local clean exact-tree provenance for R9-2 is still absent.
- `READY_LIVE: none`; release item 3 incomplete; item 4 incomplete; `RELEASE_CANDIDATE=false`; `PRODUCTION_READY=false`; `FREEZE=false`; `RELEASED=false`.

The external coding agent must synchronize to current `main` and continuously execute every dependency-ready slice below: implementation/review -> focused deterministic tests -> commit -> push -> next slice. Reviewer cadence is only a check frequency and is never a reason to idle.

## Accepted source progress that must remain closed

H-R9-026's source-level result collapse is repaired at exact `d133dbb` + `a27e79a` and should not be reopened without a contradictory exact-current reproducer:

- `recv_udp_delivery_ack` preserves applied, accepted-empty and rejected Carrier ACK classes;
- primary and settlement callers count positive transitions only on real retirement and rejection only on typed `Err`;
- accepted-empty is not mislabeled as either transition or rejection;
- post-return success remains identity-bound to actual `acked_packets`, and terminal success still requires exact Session confirmation plus Recovery zero.

Exact `bbb3e7f` also closes the **source-shape** part of H-R9-027:

- `--send-stale-ack` re-seals the same canonical ACK plaintext with a fresh authenticated envelope, so crypto replay protection no longer prevents the duplicate semantics from reaching Recovery;
- `--send-future-ack` uses a non-empty range whose maximum is `u64::MAX - 1`, so the input reaches the existing future/never-sent guard instead of failing first on empty `AckRanges`;
- the existing engine guard still rejects `largest > largest_sent` before RTT/loss/PTO mutation, and earlier engine/authenticated-path regressions remain valid.

Do not revert those repairs. Acceptance remains open only because the process tests still do not prove the intended outcome classes were actually observed with exact cardinality and terminal success.

# READY_LOCAL 1 — H-R9-028 HIGH: make stale/future ACK regressions one-shot and discriminating

This is a bounded acceptance/evidence repair. Do **not** redesign ACK framing, Session semantics, Carrier architecture, crypto/wire architecture, deadlines, malformed budgets, capacity policy, or the primary receive owner.

## A. Accepted-empty stale/duplicate must be positively observed

Exact `bbb3e7f` now creates a valid fresh-envelope semantic duplicate, but the server evaluates `--send-stale-ack` inside every reliable packet-ACK emission and currently sends the injected re-sealed ACK before the ordinary `sealed_ack`. The inherited process test only proves:

- no `r9_udp_packet_ack_rejected` substring; and
- eventual `r9_udp_in_flight_settled(...remaining_in_flight=0)`.

Those assertions can still pass if no injected duplicate is actually consumed as `UdpAcknowledgement::Carrier { applied:false, rejected:false, acked_packets:[] }`.

Smallest repair:

1. add explicit one-shot injection ownership (`stale_ack_injected` or equivalent) outside the per-record ACK emission loop;
2. arrange one normal canonical ACK retirement first, then exactly one fresh-envelope copy of the same ACK semantics while the bounded receive/settlement operation is still active;
3. make accepted-empty directly observable only as a diagnostic classification, e.g. `r9_udp_packet_ack_accepted_empty`, emitted only for `!applied && !rejected && acked_packets.is_empty()`; this diagnostic must not increment delivery, applied, rejected, health or failover evidence;
4. require client and server success;
5. require exactly one accepted-empty diagnostic, zero packet-ACK rejection, no extra positive Carrier retirement from the duplicate, unchanged Session logical-confirmation cardinality, and terminal Recovery zero.

Do not add a second receive owner or a second ACK architecture.

## B. Future/never-sent rejection must be exactly one and recovery must still settle

Exact `bbb3e7f` reaches the intended future guard, but `--send-future-ack` is likewise evaluated on every reliable packet-ACK emission. The inherited process test only asks for at least one `r9_udp_packet_ack_rejected` substring, so repeated injections or later broken settlement could still pass.

Smallest repair:

1. make the future injection one-shot (`future_ack_injected` or equivalent);
2. require exactly one typed rejection from the injected future ACK;
3. require zero positive packet retirement attributable to the future ACK;
4. require subsequent legitimate Carrier ACK progress and final Recovery zero;
5. require client and server success and unchanged Session logical completion;
6. reuse/strengthen the existing engine-level atomic future-ACK regression rather than inventing a new checker framework. Snapshot additional directly exposed `ReliableUdpRuntime`/Reno/PTO state only where useful; do not invent policy values.

Run focused deterministic tests and then the normal exact-tree local gate for the final pushed source/test SHA. The intended repair does not change decoder/framing semantics, so pinned decode fuzz is not required solely because of this slice; hosted fuzz remains separate cross-evidence when CI runs it.

**R9-3 remains blocked until A and B prove the intended accepted-empty and future-rejected classifications rather than merely making the process run green.**

# READY_LOCAL 2 — finish positive P2 C1-C4 exact closure

Use the existing count=4 fixture and preserve already-landed strong assertions; add only missing exact cardinality/identity/order proof.

## C1 — TCP replay identity

Exactly once on client and server: `stream=1`, `offset=32` (`seq=2` where emitted). Explicitly reject TCP replay evidence for offsets 0, 16 and 48.

## C2 — migration -> post-return ownership chain

Require exactly once and strict order:

```text
udp_recovery_challenge_sent
  < udp_recovery_validated
  < udp_migrated_back
  < r9_udp_post_return_sent(stream=1, offset=48, packet_number=<pn>)
```

Keep offset 48 absent from uncertain/pre-promotion/TCP replay ownership.

## C3 — server dual-domain evidence

Require exactly once after recovery validation:

- Session DeliveryAck: `stream=1`, `offset=48`, `len=16`;
- Carrier packet ACK: exact packet number cross-bound to the client `on_packet_sent` ownership.

Do not merge the two evidence domains.

## C4 — client actual transitions

Require:

- exactly one actual `r9_udp_return_delivery_ack` at `stream=1`, `offset=48`, `len=16`;
- exactly one positive actual Carrier retirement for the exact cross-bound post-return packet number;
- zero false/rejected shortcut;
- exactly one `r9_udp_post_return_settled` at `stream=1`, `offset=48`, `remaining_in_flight=0`;
- terminal settlement strictly after both actual transitions.

# READY_LOCAL 3 — post-return ACK arrival-order challenge

Through the same bounded authenticated receive owner, same absolute deadline and same malformed budget, deterministically exercise both:

- Session DeliveryAck -> Carrier packet ACK;
- Carrier packet ACK -> Session DeliveryAck.

Both must converge to one logical confirmation, one exact Carrier packet retirement, no false/rejected shortcut, and Recovery zero. A test seam may reorder already-produced authenticated ACK datagrams only; no second protocol owner or new policy value.

# READY_LOCAL 4 — P4 Session-ACK-present / Carrier-ACK-withheld negative

Require positive exact Session ACK transition, typed bounded nonzero terminal failure because Recovery remains unsettled, no settled marker, and no downstream health/failover/migration success continuation.

# READY_LOCAL 5 — P4 Carrier-ACK-present / Session-ACK-withheld negative

Require positive exact Carrier packet retirement, typed bounded nonzero terminal failure because logical confirmation remains outstanding, no settled marker, and no downstream success continuation.

# READY_LOCAL 6 — R9-2 final developer-local exact-tree provenance

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

Data, Carrier packet ACK, Session DeliveryAck, PTO/retransmit, Recovery, Session delivery, malformed budget, accepted-empty feedback, rejected feedback and terminal result remain distinct. Emit each event only after the exact claimed state transition actually occurred.

## 12. R9-8 warm TCP readiness

Authenticated resume-bound warm standby carries no application Data before promotion. Keep readiness/resource admission distinct from delivery evidence.

## 13. R9-9 health + promotion

Resolved UDP recovery outcomes feed health/hysteresis. Recoverable loss remains on UDP; PTO-only evidence cannot erase later resolved loss; promote TCP only after existing readiness/health gates.

## 14. R9-10 uncertain replay + cleanup

Replay only genuine uncertain Session ranges over the promoted TCP owner; draining/failed UDP gets no new Data; Session dedup stays exact-once; TCP gets no duplicate packet-ACK layer; cover cleanup/shutdown negatives.

## 15. R9-11/R9-12 + Q10/Q11/Q12 coherent closure

Run the clean exact-tree local gate for the complete cross-process reliable-UDP/failover slice, then perform a dedicated independent bounded challenge of the materially new R9 send/admission/recovery/demux/Session ACK/Carrier ACK/failover/migration/cleanup/diagnostic surface. A no-finding review is valid item-4 support; any BLOCKER/HIGH returns to smallest repair + regression + exact-tree gate. Only after a reachable independent review anchor may status/release packet be factually reconciled. Do not flip release-authority flags.

# Accepted earlier closures that must not regress

- H-R9-027 **source shape**: exact `bbb3e7f` provides a fresh authenticated envelope for semantic duplicate ACKs and a non-empty future range; acceptance remains open only under H-R9-028's discriminating-process-proof requirements.
- H-R9-026 **source outcome-class repair**: exact `d133dbb` + exact `a27e79a`.
- H-R9-025 narrow positive-transition repair: post-return current-packet success is identity-bound to actual `acked_packets`.
- H-R9-024 cross-process client-send/server-ACK packet-number binding.
- H-R9-023 exact P3 malformed terminal oracle.
- H-R9-022 applied Carrier ACK position evidence.
- H-R9-021 malformed -> malformed -> Carrier ACK -> malformed source order.
- H-R9-020 count=4 final accounting: `2 reliable UDP + 1 uncertain TCP + 1 post-return reliable = 4`.
- Reversed initial Session ACK process fixture: later ACK buffered while watermark 0, then offset 0 -> 16 actual application order, no covered shortcut, terminal Recovery zero.
- Candidate A future/never-sent guard remains present in exact-current `Recovery::on_ack` before mutation; exact `bbb3e7f` now reaches it at the process seam, while H-R9-028 requires exact acceptance evidence.
- Candidate B mixed queue/generic drop observability repair remains accepted absent contradictory current reproducer.

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
