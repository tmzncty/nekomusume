# ChatGPT reviewer handoff — H-R9-034 closed at b801b65; R9-3 blocked on P2 C1-C4 + ACK order + P4

## Current repository truth

- Latest developer-owned source/test commit reviewed: exact `b801b65cee8bfef196b6142ee6f5a9bfa876691a` (`fix(cli): H-R9-034 settlement-phase accepted-empty classification + late stale seam`). Source + tests.
- Prior reviewer checkpoint: [`docs/reviews/reviewer-r9-h-r9-034-7a7c48c-20260917.md`](reviews/reviewer-r9-h-r9-034-7a7c48c-20260917.md), reachable through reviewer commit `32775cffce78623e42221e9461f2ee08c75b9b51`.
- H-R9-034 is closed at exact `b801b65`: the settlement continuation now emits `r9_udp_packet_ack_accepted_empty` for `applied=false && rejected=false` (same classification-only evidence as the initial loop); after `in_flight` reaches zero on the non-migration-back initial path, one final bounded drain captures a late-arriving stale/duplicate. The new `--send-stale-ack-late` seam retains record-0's canonical ACK plaintext and re-seals it under a fresh envelope only after the last reliable record's Session ACK. `reliable_udp_stale_ack_settlement_phase_is_accepted_empty` proves exactly one settlement-phase accepted-empty after both Session confirmations, two real applied Carrier ACKs, zero typed rejection, both processes success, and final Recovery zero.
- H-R9-033 initial-loop accepted-empty classification remains closed at `7a7c48c`.
- H-R9-028 one-shot initial injection remains closed at `206b4b9`.
- H-R9-032 post-return packet cross-bind/order proof remains closed at `ddafbb1`.
- H-R9-031/H-R9-030/H-R9-029/H-R9-026 source semantics remain accepted absent contradictory exact-current evidence.
- Candidate A is rechecked closed in exact-current `Recovery::on_ack`: future/never-sent `largest > largest_sent` rejects before RTT/loss/PTO mutation.
- Candidate B is rechecked closed in exact-current `record_datagrams`: only the queue-dropped subset is `queue_full`; remaining generic drops are `terminal`.
- Developer-local clean exact-tree provenance for `b801b65`: `PYTHONDONTWRITEBYTECODE=1 bash scripts/check.sh` exit 0, `git diff --check` exit 0, clean worktree at pushed SHA, 2026-09-16T23:12:11Z → 2026-09-16T23:15:35Z, Linux x86_64, rustc 1.98.0 (88d9e12ae 2026-08-18).
- Open PRs at review time: none.
- `READY_LIVE: none`; release item 3 incomplete; item 4 incomplete; `RELEASE_CANDIDATE=false`; `PRODUCTION_READY=false`; `FREEZE=false`; `RELEASED=false`.

The external coding agent must synchronize to current `main` and continuously execute every dependency-ready slice below: implementation/review -> focused deterministic tests -> commit -> push -> next slice. Reviewer cadence is only a check frequency and is never a reason to idle.

## Accepted progress that must remain closed

Do not revert these exact-current repairs without contradictory evidence:

- H-R9-033 initial-loop accepted-empty classification at `7a7c48c`.
- H-R9-032 exact post-return packet-number cross-bind and settlement-order proof at `ddafbb1`.
- H-R9-030 ACK-obligation source repair and H-R9-029 post-return three-way classification.
- H-R9-027 stale semantic ACKs may be re-sealed under fresh authenticated envelopes; future ACKs use a non-empty never-sent range.
- H-R9-026 keeps real retirement, accepted-empty and typed rejection distinct.
- H-R9-025/H-R9-024 exact current-packet identity and positive P2 client/server packet binding.
- H-R9-023/H-R9-022/H-R9-021 malformed terminal/order evidence.
- H-R9-020 count=4 ownership accounting (`2 reliable UDP + 1 uncertain TCP + 1 post-return reliable = 4`).
- reversed initial Session ACK buffering/ordered application.
- candidate A future-ACK engine guard and candidate B mixed queue/generic-drop observability repair.

# READY_LOCAL 1 — H-R9-034 HIGH: settlement-continuation accepted-empty classification

Challenge: the initial receive loop now emits `r9_udp_packet_ack_accepted_empty`, but the later R9 settlement continuation still silently consumes the same `applied=false && rejected=false` class.

Smallest repair only:

- emit the same classification-only `r9_udp_packet_ack_accepted_empty` event from the settlement continuation;
- do **not** increment applied/rejected counters and do not mutate Recovery/Session state;
- add one deterministic built-binary regression that deliberately makes a semantic duplicate reach the settlement continuation **after logical Session confirmations are complete**;
- require both processes success, exactly one settlement-phase accepted-empty classification, only the expected real positive Carrier retirements, zero typed rejection, unchanged exact Session confirmations, exactly one final zero-in-flight settlement;
- preserve the current future/never-sent fixture unchanged.

No ACK framing, Recovery, Session, crypto/wire, deadline, malformed-budget, capacity-policy or owner redesign. No fuzz unless a decoder/parser/crypto framing file actually changes.

**R9-3 remains blocked until READY_LOCAL 1 and the remaining R9-2 closure lanes are complete.**

# READY_LOCAL 2 — positive P2 C1-C4 exact closure

Keep the existing count=4 `reliable_udp_migration_back_reserves_final_record` fixture. Exact-current test already proves both process success, broad migration ordering, ownership accounting, one post-return send, one server post-return DACK/packet-ACK, and settlement after both client ACK-domain events. Close the remaining exact gaps without new architecture:

- **C1 TCP replay identity:** client **and server** exactly once `stream=1 offset=32`, `seq=2` where emitted; reject TCP replay/ACK evidence for offsets `0`, `16`, `48`.
- **C2 migration ownership chain:** exactly once each `udp_recovery_challenge_sent`, `udp_recovery_validated`, `udp_migrated_back`, `r9_udp_post_return_sent`; strict order challenge < validated < migrated < post-return send. Prove offset 48 absent from pre-promotion `r9_udp_record_sent`, `udp_uncertain_range_sent`, and TCP replay ownership.
- **C3 server dual-domain:** exactly one `udp_return_delivery_ack_sent` with `stream=1 offset=48 len=16`; exactly one `udp_return_packet_ack_sent` whose packet number equals the exact client `r9_udp_post_return_sent.packet_number`.
- **C4 client actual transitions:** exactly one `r9_udp_return_delivery_ack` with `stream=1 offset=48 len=16`; exactly one positive `r9_udp_return_packet_ack` with the same cross-bound packet number; zero rejected/accepted-empty shortcut in the ordinary positive fixture; exactly one `r9_udp_post_return_settled ... remaining_in_flight=0`; settlement strictly after both actual transitions.

# READY_LOCAL 3 — post-return ACK arrival-order challenge

Through the same authenticated receive owner, same deadline and same malformed budget, deterministically cover both:

- Session DeliveryAck -> Carrier packet ACK;
- Carrier packet ACK -> Session DeliveryAck.

Both converge to one logical confirmation, one exact packet retirement, no false/rejected/accepted-empty shortcut, and Recovery zero. A test seam may reorder already-produced authenticated ACK datagrams only; no new protocol owner or policy value.

# READY_LOCAL 4 — P4 Session ACK present / Carrier ACK withheld

Require positive exact Session transition, typed bounded nonzero terminal failure because Recovery remains unsettled, no settled marker, and no downstream health/failover/migration success continuation.

# READY_LOCAL 5 — P4 Carrier ACK present / Session ACK withheld

Require positive exact Carrier retirement, typed bounded nonzero terminal failure because logical confirmation remains outstanding, no settled marker, and no downstream success continuation.

# READY_LOCAL 6 — final R9-2 developer-local exact-tree provenance

On the final pushed R9-2 source/test SHA in a clean safe checkout/worktree run at least:

```text
PYTHONDONTWRITEBYTECODE=1 bash scripts/check.sh
git diff --check
```

Record exact reachable pushed SHA, UTC start/end, Linux OS/arch, Rust stable version, both exit codes, and clean initial/final tree. Run pinned decoder fuzz only if decoder/parser/crypto framing actually changed. Hosted CI remains separate cross-evidence. Continue immediately afterwards.

# READY_LOCAL 7 — R9-3 Data-loss recovery

Suppress one reliable-owned Data only after congestion admission; suppression must not bypass ownership accounting. PTO only after its deadline; retransmit under a fresh packet number/nonce while retaining stable frame/logical identity; exactly-once Session delivery; independent ACK domains; final Recovery zero or typed bounded failure.

# READY_LOCAL 8 — R9-4 ACK-loss + delayed original/reorder

Suppress one Carrier ACK, force legitimate retransmission, then release the delayed original. Require one logical delivery, Session dedup authority, fresh packet numbers/nonces, truthful loss/PTO evidence, and settled Recovery.

# READY_LOCAL 9 — R9-5 adversarial feedback / all caller classification

Future/never-sent/stale/duplicate/tampered feedback must be atomic and fail closed. Rejected feedback cannot mutate RTT/PTO/loss/cwnd/Session state; accepted-empty cannot be mislabeled as transition or rejection. Re-check every exact-current `UdpAcknowledgement::Carrier` caller/continuation, including initial, settlement and post-return diagnostics; H-R9-034 is the first known exact-current gap.

# READY_LOCAL 10 — R9-6 ownership/resource boundedness

Every first send/retransmit consults congestion admission before ownership commit; refusal commits no recovery/logical state. Keep one bounded retransmit plaintext owner and deterministic teardown. Do not invent capacity values.

# READY_LOCAL 11 — R9-7 process/result truth

Data, Carrier ACK, Session DeliveryAck, PTO/retransmit, Recovery, Session delivery, malformed budget, accepted-empty feedback, rejected feedback and terminal result remain distinct. Emit each event only after the exact claimed state transition/classification.

# READY_LOCAL 12 — R9-8 warm TCP readiness

Authenticated resume-bound warm standby carries no application Data before promotion. Readiness/resource admission remains distinct from delivery evidence.

# READY_LOCAL 13 — R9-9/R9-10 health, promotion, uncertain replay and cleanup

Resolved UDP outcomes feed existing health/hysteresis; recoverable loss remains on UDP; promotion only after existing readiness/health gates. Replay only genuine uncertain Session ranges over promoted TCP; draining/failed UDP gets no new Data; Session dedup exact-once; TCP gets no duplicate packet-ACK layer; cover shutdown/cleanup negatives.

# READY_LOCAL 14 — R9-11/R9-12 complete cross-process slice

Close remaining lifecycle/terminal invariants and run the clean exact-tree local gate for the complete materially new cross-process reliable-UDP/failover slice. Preserve source/test/evidence provenance boundaries.

# READY_LOCAL 15 — independent R9 review + Q10/Q11/Q12 factual reconciliation

Perform a dedicated independent bounded challenge of materially new R9 send/admission/recovery/demux/Session ACK/Carrier ACK/failover/migration/cleanup/diagnostic surface. A no-finding review is valid item-4 support; any BLOCKER/HIGH returns to smallest repair + regression + exact-tree gate. Only after a reachable independent review anchor may status/release packet be factually reconciled. Do not flip release-authority flags.

## Item-4 / core-surface inventory

Reachable independent bounded review already exists for earlier reliable-UDP engine basics, CarrierState, Carrier Manager/health/migration-back, FairScheduler/flow accounting, carrier adapters, SessionRuntime, observability, package/reproducibility, dependency/build, CLI portability/output, algorithmic boundedness/validators, DeliveryLedger/process codec/datagram/crypto API, and wire/parser surfaces.

The materially new cross-process R9 integration is not yet independently closed. Queue exhaustion is false.

## VPS opportunity

**Not READY.** Standing authorization remains valid, but authoritative classification remains `READY_LIVE: none`. Current blocker class is local R9 correctness/evidence plus later independent cross-process R9 review. Do not repeat HY2, warm failover, periodic/soak, package lifecycle, migration-back, endpoint/key migration, IPv6, PLPMTUD or Experimental Track evidence merely because the VPS remains rented.

Only create a new `READY_LIVE` row if later code/instrumentation/hypothesis/path conditions create a concrete unresolved real-network question.

## Separate non-blocking policy / authority gates

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
