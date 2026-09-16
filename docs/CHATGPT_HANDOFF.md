# ChatGPT reviewer handoff — H-R9-034 independently closed; P2 exact closure is queue head

## Current repository truth

- Latest developer-owned source/test commit independently reviewed: exact `b801b65cee8bfef196b6142ee6f5a9bfa876691a` (`fix(cli): H-R9-034 settlement-phase accepted-empty classification + late stale seam`).
- Latest independent reviewer checkpoint: [`docs/reviews/reviewer-r9-h-r9-034-b801b65-20260917.md`](reviews/reviewer-r9-h-r9-034-b801b65-20260917.md), reachable through reviewer commit `52e94036e84e819a80c1c654b6ed5332e8f038ce`.
- H-R9-034 is closed at exact `b801b65`: settlement and initial receive continuations now classify authenticated accepted-empty Carrier feedback consistently without promoting it to a transition/rejection; the late-stale built-binary seam proves the settlement continuation can actually consume that class after the logical confirmations.
- H-R9-033 initial-loop accepted-empty classification remains closed at `7a7c48c`; H-R9-028 one-shot initial stale/future injection remains closed at `206b4b9`; H-R9-032 post-return packet-number cross-bind/order proof remains closed at `ddafbb1`; H-R9-031/H-R9-030/H-R9-029/H-R9-026 semantics remain accepted absent contradictory exact-current evidence.
- Candidate A remains closed in exact-current `Recovery::on_ack`: `largest > largest_sent` rejects before RTT/loss/PTO mutation. Candidate B remains closed in exact-current `record_datagrams`: only the queue-dropped subset maps to `queue_full`; remaining generic drops are `terminal`.
- Repository-persisted **developer-local** clean exact-tree provenance for pushed `b801b65` records `PYTHONDONTWRITEBYTECODE=1 bash scripts/check.sh` exit 0, `git diff --check` exit 0, clean tree, 2026-09-16T23:12:11Z → 2026-09-16T23:15:35Z, Linux x86_64, rustc 1.98.0 (88d9e12ae 2026-08-18). This is developer-local evidence, not reviewer execution.
- Separate GitHub-hosted Rust CI for exact `b801b65`, run `35161172315`, completed SUCCESS: `stable checks` passed `bash scripts/check.sh`; `nightly decode fuzz smoke` also passed. Hosted CI is cross-evidence only.
- Open PRs at review time: none.
- `READY_LIVE: none`; release item 3 incomplete; release item 4 incomplete; `RELEASE_CANDIDATE=false`; `PRODUCTION_READY=false`; `FREEZE=false`; `RELEASED=false`.

The external coding agent must synchronize to current `main` and continuously execute every dependency-ready slice below: implementation/review -> focused deterministic tests -> commit -> push -> next slice. Reviewer cadence is only a check frequency and is never a work-ticket length or reason to idle.

## Accepted progress that must remain closed

Do not revert these exact-current repairs without contradictory evidence:

- H-R9-034 settlement-continuation accepted-empty classification and late-stale discriminating proof at `b801b65`.
- H-R9-033 initial-loop accepted-empty classification at `7a7c48c`.
- H-R9-032 exact post-return packet-number cross-bind and settlement-order proof at `ddafbb1`.
- H-R9-030 ACK-obligation source repair and H-R9-029 post-return three-way classification.
- H-R9-027 stale semantic ACKs may be re-sealed under fresh authenticated envelopes; future ACKs use a non-empty never-sent range.
- H-R9-026 keeps real retirement, accepted-empty and typed rejection distinct.
- H-R9-025/H-R9-024 exact current-packet identity and positive post-return client/server packet binding.
- H-R9-023/H-R9-022/H-R9-021 malformed terminal/order evidence.
- H-R9-020 count=4 ownership accounting (`2 reliable UDP + 1 uncertain TCP + 1 post-return reliable = 4`).
- reversed initial Session ACK buffering/ordered application.
- candidate A future-ACK engine guard and candidate B mixed queue/generic-drop observability repair.

The previous handoff accidentally left the already-completed H-R9-034 repair as a READY lane below a header that said it was closed. That stale ticket is removed here: **do not repeat H-R9-034.**

# READY_LOCAL 1 — positive P2 C1-C4 exact closure

Keep and strengthen the existing count=4 `reliable_udp_migration_back_reserves_final_record` built-binary fixture. Exact-current `probe.rs` already proves both processes succeed, broad server/client migration ordering, the 2+1+1 ownership accounting, one post-return send, one server post-return Session/Carrier ACK pair, server/client packet-number equality for the positive post-return packet, and settlement after both client ACK-domain events. The remaining gaps are oracle precision, not permission to redesign runtime architecture.

Close all four groups together if practical rather than splitting them into checker churn:

- **C1 TCP replay identity**
  - client exactly once `tcp_delivery_ack_validated` for `stream=1 offset=32`, with `seq=2` when that structured field is emitted;
  - server exactly once `tcp_delivery_ack_sent` for the same logical record (`stream=1 offset=32`, `seq=2` where emitted);
  - reject client **and server** TCP replay/ACK evidence for offsets `0`, `16`, or `48`.
  - Exact-current fixture currently asserts one client/server offset-32 ACK and rejects some client offset aliases, but does not fully bind `seq=2`/stream on both sides or exclude all server-side wrong ownership.

- **C2 migration ownership chain**
  - require exactly once each `udp_recovery_challenge_sent`, `udp_recovery_validated`, `udp_migrated_back`, and `r9_udp_post_return_sent`;
  - strict order `challenge < validated < migrated_back < post_return_sent`;
  - prove reserved offset `48` is absent from all pre-promotion owners: initial/reliable `r9_udp_record_sent`, `udp_uncertain_range_sent`, and TCP replay ownership/evidence;
  - retain the existing final accounting proof (`2 reliable UDP + 1 uncertain/TCP + 1 post-return reliable`).
  - Exact-current fixture proves broad order and one post-return send but not exact cardinality of every preceding milestone and not every offset-48 pre-promotion exclusion.

- **C3 server dual-domain truth**
  - exactly one `udp_return_delivery_ack_sent` with `stream=1 offset=48 len=16`;
  - exactly one `udp_return_packet_ack_sent`;
  - that Carrier ACK packet number must equal the exact client `r9_udp_post_return_sent.packet_number`.
  - Exact-current fixture already binds packet number and event cardinality, but its Session-side assertion is still narrower than the full `stream/offset/len` contract.

- **C4 client actual transitions**
  - exactly one `r9_udp_return_delivery_ack` with `stream=1 offset=48 len=16`;
  - exactly one positive `r9_udp_return_packet_ack` with `applied=true`, `retired=true`, and the exact same client/server post-return packet number;
  - zero `r9_udp_return_packet_ack_rejected` and zero `r9_udp_return_packet_ack_accepted_empty` in this ordinary positive fixture;
  - exactly one `r9_udp_post_return_settled` with `stream=1 offset=48 remaining_in_flight=0`;
  - settlement strictly after both actual ACK-domain transitions.
  - Exact-current runtime already emits the needed fields and gates positive retirement on `acked_packets.contains(post_pn_client)`; this lane should first be a test/oracle tightening. If the stronger oracle exposes a runtime contradiction, switch immediately to the smallest repair plus regression.

No Session/Carrier/ACK/wire/crypto redesign and no new policy number. If this remains tests only, do not mechanically fuzz; if a decoder/parser/crypto framing file actually changes, use the pinned fuzz toolchain contract.

# READY_LOCAL 2 — post-return ACK arrival-order challenge

Through the exact same authenticated post-return receive owner, same absolute deadline and same malformed budget, deterministically cover both:

1. Session DeliveryAck -> Carrier packet ACK;
2. Carrier packet ACK -> Session DeliveryAck.

Both must converge to exactly one logical confirmation, exactly one exact packet retirement, zero false/rejected/accepted-empty shortcut, exactly one settled-zero result and both processes success. Reorder only already-produced authenticated ACK datagrams with a narrow test seam; do not add another protocol owner or policy value.

# READY_LOCAL 3 — P4 Session ACK present / Carrier ACK withheld

Add/strengthen one deterministic built-binary negative where the exact post-return Session DeliveryAck arrives but the Carrier ACK is withheld. Require:

- exact positive Session transition for `stream=1 offset=48 len=16`;
- Recovery remains unsettled;
- typed bounded nonzero terminal failure;
- no `r9_udp_post_return_settled` marker;
- no downstream health/failover/migration success continuation caused by a false premise.

# READY_LOCAL 4 — P4 Carrier ACK present / Session ACK withheld

Mirror the previous lane for the other evidence domain:

- exact positive Carrier retirement for the cross-bound post-return packet;
- logical Session confirmation remains outstanding;
- typed bounded nonzero terminal failure;
- no settled marker and no downstream success continuation.

These two P4 lanes are specifically about proving neither evidence domain substitutes for the other.

# READY_LOCAL 5 — final R9-2 developer-local exact-tree provenance

After READY_LOCAL 1-4 are on one final pushed R9-2 source/test SHA, use a safe clean checkout/worktree and run at least:

```text
PYTHONDONTWRITEBYTECODE=1 bash scripts/check.sh
git diff --check
```

Persist exact reachable pushed SHA, UTC start/end, OS/arch, Rust stable version, both exit codes, clean initial/final tree, and clearly label the evidence developer-local. Run `cargo fuzz build decode` plus `cargo fuzz run decode -- -max_total_time=30 -max_len=8192` through `scripts/fuzz-toolchain.sh` only if this R9-2 closure actually changes decoder/parser/crypto framing. Hosted Actions remain separate cross-evidence. **Continue directly to R9-3 after this gate; do not wait for the next reviewer.**

# READY_LOCAL 6 — R9-3 Data-loss recovery

Suppress one reliable-owned Data only **after** congestion admission so the fault seam cannot bypass ownership accounting. Challenge the full current recovery contract:

- PTO fires only after its deadline;
- retransmission uses a fresh packet number / crypto nonce but stable frame/logical identity;
- receiver Session delivery remains exactly once;
- Session DeliveryAck and Carrier packet ACK remain separate evidence domains;
- positive completion reaches Recovery zero, otherwise failure is typed and bounded.

Use current M2 semantics; do not invent a new ACK or retransmission architecture.

# READY_LOCAL 7 — R9-4 ACK-loss + delayed original/reorder

Suppress one legitimate Carrier ACK, force a legitimate retransmission, then release the delayed original ACK. Require one logical Session delivery, Session dedup authority, fresh packet numbers/nonces, truthful PTO/loss/retransmit evidence and settled Recovery. A late original may be accepted-empty/stale but must never manufacture a second transition or rejection.

# READY_LOCAL 8 — R9-5 adversarial feedback / every Carrier caller

Challenge future/never-sent/stale/duplicate/tampered feedback across every exact-current `UdpAcknowledgement::Carrier` continuation, including initial receive, settlement and post-return owners. Rejected feedback must be atomic and fail closed without RTT/PTO/loss/cwnd/Session mutation; accepted-empty must not become transition or rejection. H-R9-033/H-R9-034 are closed examples, not reasons to skip the rest of the exact-current call surface.

# READY_LOCAL 9 — R9-6 ownership/resource boundedness

Every first send and retransmit must consult congestion admission before ownership commit; refusal commits no recovery or logical state. Keep one bounded retransmit-plaintext owner and deterministic teardown. Challenge algorithmic boundedness without adding a capacity-pressure benchmark or inventing new capacity/security values.

# READY_LOCAL 10 — R9-7 process/result truth

Independently challenge that Data, Carrier ACK, Session DeliveryAck, PTO/retransmit, Recovery, Session delivery, malformed budget, accepted-empty feedback, rejected feedback and terminal result remain distinct in structured process evidence. Emit/accept a diagnostic only after the exact claimed transition or classification has occurred.

# READY_LOCAL 11 — R9-8 warm TCP readiness

Challenge the existing single-active/multi-ready contract: authenticated resume-bound warm standby may carry readiness/control only and **no application Data before promotion**. Readiness/resource admission remains separate from packet feedback and Session delivery evidence. Use D064/current Carrier Manager semantics; no architecture change.

# READY_LOCAL 12 — R9-9/R9-10 health, promotion, uncertain replay and cleanup

Challenge the integrated path rather than isolated helpers:

- resolved UDP outcomes feed existing health/hysteresis;
- recoverable reliable-UDP loss remains on UDP instead of spuriously promoting TCP;
- promotion happens only after existing readiness/health gates;
- only genuine uncertain Session ranges replay on promoted TCP;
- draining/failed UDP receives no new Data ownership;
- receiver Session dedup is exactly-once;
- TCP gets no duplicate UDP packet-ACK layer;
- shutdown/error negatives clean resources and do not emit false success.

# READY_LOCAL 13 — R9-11/R9-12 complete cross-process slice

Close remaining lifecycle/terminal invariants for the materially new cross-process reliable-UDP/failover integration and run its clean exact-tree developer-local gate. Preserve source/test/evidence provenance boundaries; do not rewrite historical live evidence to describe the new local tree.

# READY_LOCAL 14 — dedicated independent R9 review + Q10/Q11/Q12 factual reconciliation

After the implementation/test lanes above are reachable and gated, perform a dedicated independent bounded challenge of materially new R9 send/admission/recovery/demux/Session ACK/Carrier ACK/failover/migration/cleanup/diagnostic behavior.

- A bounded no-finding review is valid release-item-4 support if scope, inspected owners, tests/commands, exclusions and reachable anchor are explicit.
- Any concrete BLOCKER/HIGH returns immediately to smallest repair -> discriminating regression -> exact-tree local gate.
- Only after a reachable independent review anchor should `docs/status.md`, `docs/release-security-review-packet.md` and related Q10/Q11/Q12 evidence text be factually reconciled.
- Do **not** change RC/freeze/release/production authority flags.

## Item-4 / core-surface inventory

Reachable independent bounded review already exists for earlier reliable-UDP engine basics (including the future/never-sent ACK guard), `CarrierState`, concurrent Carrier Manager/health/migration-back, FairScheduler/flow accounting, carrier adapters, `SessionRuntime`, observability (including mixed queue/generic drop classification), package/reproducibility/operator scripts, dependency/build surface, CLI portability/output, algorithmic boundedness/validators, DeliveryLedger/process codec/datagram/crypto API, and wire/parser surfaces.

The **materially new cross-process R9 integration** remains the broad uncovered item-4 surface until READY_LOCAL 14. Therefore repository-wide queue exhaustion is false even if any one narrow seam yields no finding.

## VPS opportunity

**Not READY.** Standing authorization remains valid, but authoritative classification is still `READY_LIVE: none`. Current work is local R9 correctness/evidence plus its later independent cross-process review. Do not repeat HY2, warm failover, periodic/soak, package lifecycle, migration-back, endpoint/key migration, IPv6, PLPMTUD or Experimental Track evidence merely because the VPS remains rented.

Only create a new `READY_LIVE` row if later code/instrumentation/hypothesis/path conditions produce a concrete unresolved real-network question that local/loopback evidence cannot answer.

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
