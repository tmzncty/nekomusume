# ChatGPT reviewer handoff — H-R9-038 HIGH: P4 dual-domain negative oracles still under-prove residual state

## Current repository truth

- Latest developer-owned source/test commit reviewed: exact `a5237330b16ca199dc2c16bebcf728859c8a0491` (`test(cli): P4 dual suppression — Carrier-ACK-withheld + Session-ACK-withheld negatives (READY_LOCAL 1+2)`).
- Independent finding: [`docs/reviews/reviewer-r9-h-r9-038-p4-dual-suppression-oracle-a523733-20260917.md`](reviews/reviewer-r9-h-r9-038-p4-dual-suppression-oracle-a523733-20260917.md), reachable through reviewer commit `4dd120d15cbfce054a792ab48f9a8b711c03fa67`.
- **H-R9-038 HIGH is open.** The new P4 fault seams are directionally consistent with current Session/Carrier separation, but both built-binary tests remain too weak to prove that one evidence domain cannot substitute for the other or to prove the exact residual domain at bounded failure.
- Exact `a523733` GitHub-hosted Rust CI run `35183369645` is SUCCESS. This is hosted cross-evidence only and does not replace developer-local clean exact-tree provenance.
- Open PRs at review time: none.
- P2 C1-C4 and H-R9-037/H-R9-036/H-R9-035/H-R9-034/H-R9-033/H-R9-032 and earlier accepted repairs remain closed unless contradictory exact-current evidence appears.
- Candidate A future/never-sent ACK fail-closed guard and candidate B mixed queue/generic-drop observability repair remain closed.
- `READY_LIVE: none`; release item 3 incomplete; release item 4 incomplete; `RELEASE_CANDIDATE=false`; `PRODUCTION_READY=false`; `FREEZE=false`; `RELEASED=false`.

The external coding agent must synchronize to current `main` and continuously execute every dependency-ready slice below: implementation/review -> focused deterministic tests -> commit -> push -> next slice. Reviewer cadence is only a check frequency and is never a work-ticket length or reason to idle.

## Accepted progress that must remain closed

Do not revert these exact-current repairs without contradictory evidence:

- H-R9-037 reverse-order true three-way packet bind at `d5d5b23`, independently reviewed at `f2529087`.
- H-R9-036 reverse-order exact oracle at `855c679`.
- Post-return ACK arrival-order challenge at `83d10b0` + `424583d` + `855c679` + `d5d5b23`.
- P2 C1-C4 exact closure at `142f0a1` + `8d0ccd5` + `1f8bbb0` + `be03dc3`: TCP replay identity bound on both sides; migration milestones exact cardinality/order; server dual-domain stream/offset/len; positive-path three-way packet-number equality; exactly-one settled cardinality and post-transition order.
- `424583d` ordinary no-seam post-return path sends no duplicate Carrier ACK; ordinary order remains Session DeliveryAck then Carrier ACK, while `--reverse-post-return-ack` sends Carrier ACK before Session DeliveryAck.
- H-R9-034 settlement-continuation accepted-empty classification and late-stale discriminating proof at `b801b65`.
- H-R9-033 initial-loop accepted-empty classification at `7a7c48c`.
- H-R9-032 post-return stale/future packet-number cross-bind and settlement-order proof at `ddafbb1`.
- H-R9-030 ACK-obligation source repair and H-R9-029 post-return three-way classification.
- H-R9-027 stale semantic ACKs may be re-sealed under fresh authenticated envelopes; future ACKs use a non-empty never-sent range.
- H-R9-026 keeps real retirement, accepted-empty and typed rejection distinct.
- H-R9-025/H-R9-024 exact current-packet identity and positive post-return client/server packet binding source repair.
- H-R9-023/H-R9-022/H-R9-021 malformed terminal/order evidence.
- H-R9-020 count=4 ownership accounting (`2 reliable UDP + 1 uncertain TCP + 1 post-return reliable = 4`).
- reversed initial Session ACK buffering/ordered application.

## H-R9-038 exact finding

`a523733` adds the smallest post-return-only Carrier-ACK suppression seam and two P4 process tests, but the tests do not yet satisfy the handoff acceptance contract.

### P4-A current weakness — Session ACK present / Carrier ACK withheld

`reliable_udp_post_return_carrier_ack_withheld_fails` currently:

- discards server exit status returned by `finish_server`;
- checks only broad presence of `r9_udp_return_delivery_ack`, not exactly one `stream=1 offset=48 len=16` transition;
- checks broad server presence/absence rather than exact cardinality;
- has no classification-only residual-state proof that Session is complete while Recovery remains nonzero;
- does not fully exclude downstream success continuation from a false settlement premise.

### P4-B current weakness — Carrier ACK present / Session ACK withheld

`reliable_udp_post_return_session_ack_withheld_fails` currently:

- discards server exit status;
- checks only broad presence of `r9_udp_return_packet_ack`, not exactly one positive `applied=true, retired=true` transition;
- does not require exactly-one client send / server Carrier ACK / client retirement or their three-way packet-number equality;
- does not exclude accepted-empty/rejected substitutes;
- has no residual-state proof that Recovery is zero while one Session range remains outstanding.

The current source seam itself does not prove a new runtime-state contradiction. Repair the oracle first; if the stronger oracle exposes one, then perform the smallest current-semantics runtime repair.

# READY_LOCAL 1 — H-R9-038A exact P4 Carrier-ACK-withheld proof

Keep `--suppress-r9-post-return-pack` as a post-return-only fault seam. Add one common classification-only terminal diagnostic at the existing bounded post-return dual-settlement failure boundary if needed; it must report the actual remaining domains (at minimum `session_outstanding` and `remaining_in_flight`), mutate no state, add no policy value and never count as success.

Strengthen `reliable_udp_post_return_carrier_ack_withheld_fails` so it requires:

- server process succeeds;
- exactly one server `udp_return_delivery_ack_sent` for the post-return range and zero server `udp_return_packet_ack_sent` for that packet;
- exactly one client `r9_udp_return_delivery_ack` with `stream=1 offset=48 len=16`;
- zero positive client post-return Carrier retirement, zero `r9_udp_return_packet_ack_rejected`, zero `r9_udp_return_packet_ack_accepted_empty` substitute;
- terminal residual-domain diagnostic immediately before bounded failure with `session_outstanding=0` and `remaining_in_flight>0`;
- client exits typed/nonzero on the existing bounded post-return terminal path;
- zero `r9_udp_post_return_settled` and zero downstream health/failover/accounting/summary success continuation caused by a false settlement premise.

Run the focused built-binary test first. No decoder/parser/crypto-framing change is expected; do not mechanically run fuzz for this seam/test repair.

# READY_LOCAL 2 — H-R9-038B exact P4 Session-ACK-withheld proof

Keep existing `--suppress-r9-dack`; do not change Session/Carrier/ACK semantics merely to make the test convenient.

Strengthen `reliable_udp_post_return_session_ack_withheld_fails` so it requires:

- server process succeeds;
- exactly one client `r9_udp_post_return_sent`, exactly one server `udp_return_packet_ack_sent`, exactly one positive client `r9_udp_return_packet_ack` with `applied=true, retired=true`;
- exact three-way packet-number equality between client send, server Carrier ACK and client retirement;
- zero server `udp_return_delivery_ack_sent` for post-return offset 48 and zero client `r9_udp_return_delivery_ack` transition for that range;
- zero `r9_udp_return_packet_ack_rejected` / zero `r9_udp_return_packet_ack_accepted_empty` substitute;
- terminal residual-domain diagnostic with `remaining_in_flight=0` and `session_outstanding=1` immediately before bounded failure;
- client exits typed/nonzero on the existing bounded post-return terminal path;
- zero `r9_udp_post_return_settled` and zero downstream success continuation.

If READY_LOCAL 1/2 reveal a real state contradiction, smallest repair -> positive/negative regression -> pushed exact-tree gate, then continue immediately.

# READY_LOCAL 3 — final R9-2 developer-local exact-tree provenance

After READY_LOCAL 1-2 are on one final pushed R9-2 source/test SHA, use a safe clean checkout/worktree and run at least:

```text
PYTHONDONTWRITEBYTECODE=1 bash scripts/check.sh
git diff --check
```

Persist exact reachable pushed SHA, UTC start/end, OS/arch, Rust stable version, both exit codes, clean initial/final tree, and clearly label the evidence developer-local. Run `cargo fuzz build decode` plus `cargo fuzz run decode -- -max_total_time=30 -max_len=8192` through `scripts/fuzz-toolchain.sh` only if this R9-2 closure actually changes decoder/parser/crypto framing. Hosted Actions remain separate cross-evidence. Continue directly to R9-3 after this gate; do not wait for the next reviewer.

# READY_LOCAL 4 — R9-3 Data-loss recovery

Suppress one reliable-owned Data only after congestion admission so the fault seam cannot bypass ownership accounting. Challenge the full current recovery contract:

- PTO fires only after its deadline;
- retransmission uses a fresh packet number / crypto nonce but stable frame/logical identity;
- receiver Session delivery remains exactly once;
- Session DeliveryAck and Carrier packet ACK remain separate evidence domains;
- positive completion reaches Recovery zero, otherwise failure is typed and bounded.

Use current M2 semantics; do not invent a new ACK or retransmission architecture.

# READY_LOCAL 5 — R9-4 ACK-loss + delayed original/reorder

Suppress one legitimate Carrier ACK, force a legitimate retransmission, then release the delayed original ACK. Require one logical Session delivery, Session dedup authority, fresh packet numbers/nonces, truthful PTO/loss/retransmit evidence and settled Recovery. A late original may be accepted-empty/stale but must never manufacture a second transition or rejection.

# READY_LOCAL 6 — R9-5 adversarial feedback / every Carrier caller

Challenge future/never-sent/stale/duplicate/tampered feedback across every exact-current `UdpAcknowledgement::Carrier` continuation, including initial receive, settlement and post-return owners. Rejected feedback must be atomic and fail closed without RTT/PTO/loss/cwnd/Session mutation; accepted-empty must not become transition or rejection. H-R9-033/H-R9-034 are closed examples, not reasons to skip the rest of the exact-current call surface.

# READY_LOCAL 7 — R9-6 ownership/resource boundedness

Every first send and retransmit must consult congestion admission before ownership commit; refusal commits no recovery or logical state. Keep one bounded retransmit-plaintext owner and deterministic teardown. Challenge algorithmic boundedness without adding a capacity-pressure benchmark or inventing new capacity/security values.

# READY_LOCAL 8 — R9-7 process/result truth

Independently challenge that Data, Carrier ACK, Session DeliveryAck, PTO/retransmit, Recovery, Session delivery, malformed budget, accepted-empty feedback, rejected feedback and terminal result remain distinct in structured process evidence. Emit/accept a diagnostic only after the exact claimed transition or classification has occurred.

# READY_LOCAL 9 — R9-8 warm TCP readiness

Challenge the existing single-active/multi-ready contract: authenticated resume-bound warm standby may carry readiness/control only and **no application Data before promotion**. Readiness/resource admission remains separate from packet feedback and Session delivery evidence. Use D064/current Carrier Manager semantics; no architecture change.

# READY_LOCAL 10 — R9-9/R9-10 health, promotion, uncertain replay and cleanup

Challenge the integrated path rather than isolated helpers:

- resolved UDP outcomes feed existing health/hysteresis;
- recoverable reliable-UDP loss remains on UDP instead of spuriously promoting TCP;
- promotion happens only after existing readiness/health gates;
- only genuine uncertain Session ranges replay on promoted TCP;
- draining/failed UDP receives no new Data ownership;
- receiver Session dedup is exactly-once;
- TCP gets no duplicate UDP packet-ACK layer;
- shutdown/error negatives clean resources and do not emit false success.

# READY_LOCAL 11 — R9-11/R9-12 complete cross-process slice

Close remaining lifecycle/terminal invariants for the materially new cross-process reliable-UDP/failover integration and run its clean exact-tree developer-local gate. Preserve source/test/evidence provenance boundaries; do not rewrite historical live evidence to describe the new local tree.

# READY_LOCAL 12 — dedicated independent R9 review

After the implementation/test lanes above are reachable and gated, perform a dedicated independent bounded challenge of materially new R9 send/admission/recovery/demux/Session ACK/Carrier ACK/failover/migration/cleanup/diagnostic behavior.

A bounded no-finding review is valid release-item-4 support if scope, inspected owners, tests/commands, exclusions and reachable anchor are explicit. Any concrete BLOCKER/HIGH returns immediately to smallest repair -> discriminating regression -> exact-tree local gate.

# READY_LOCAL 13 — Q10/Q11/Q12 factual reconciliation

Only after a reachable independent R9 review anchor exists, factually reconcile `docs/status.md`, `docs/release-security-review-packet.md` and the related Q10/Q11/Q12 evidence text with the exact reachable implementation/review anchors. Preserve historical evidence boundaries and do **not** change RC/freeze/release/production authority flags.

## Item-4 / core-surface inventory

Reachable independent bounded review already exists for earlier reliable-UDP engine basics (including the future/never-sent ACK guard), `CarrierState`, concurrent Carrier Manager/health/migration-back, FairScheduler/flow accounting, carrier adapters, `SessionRuntime`, observability (including mixed queue/generic drop classification), package/reproducibility/operator scripts, dependency/build surface, CLI portability/output, algorithmic boundedness/validators, DeliveryLedger/process codec/datagram/crypto API, and wire/parser surfaces.

The **materially new cross-process R9 integration** remains the broad uncovered item-4 surface until READY_LOCAL 12. Therefore repository-wide queue exhaustion is false even if any one narrow seam yields no finding.

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