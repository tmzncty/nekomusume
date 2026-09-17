# ChatGPT reviewer handoff — R9-3 READY with implementation-ready PTO/ACK ownership

## Current repository truth

- Current `main` reviewer/navigation anchor before this handoff update: exact `bb96fabf666e776584f1d9f6d286974752434ef8` (`docs(review): make R9-3 PTO ownership implementation-ready`).
- Latest developer-owned source/test commit reviewed remains exact `021d79d88dadc9dbc7cf749745b2395c06602b29` (`test(cli): P4-B assert server success (H-R9-039)`), building on `9091803`.
- R9-3 navigation notes:
  - [`docs/reviews/reviewer-r9-3-pto-timebase-navigation-021d79d-20260917.md`](reviews/reviewer-r9-3-pto-timebase-navigation-021d79d-20260917.md), reachable at `319c6bb`;
  - [`docs/reviews/reviewer-r9-3-ack-observation-time-021d79d-20260917.md`](reviews/reviewer-r9-3-ack-observation-time-021d79d-20260917.md), reachable at `938e290`;
  - [`docs/reviews/reviewer-r9-3-pto-api-ownership-021d79d-20260917.md`](reviews/reviewer-r9-3-pto-api-ownership-021d79d-20260917.md), reachable at `bb96fab`.
- H-R9-039 is closed at exact `021d79d`: P4-B retains `srv_status` and asserts `srv_status.success()` while preserving all exact P4-B residual-domain/oracle checks.
- H-R9-038 is closed across exact `236e962` + `9091803` + `021d79d`: both P4 negatives prove server success, exact cardinality/identity, residual-domain terminal evidence, zero misclassification, typed nonzero client exit and no false settled premise.
- P2 C1-C4 and H-R9-037/H-R9-036/H-R9-035/H-R9-034/H-R9-033/H-R9-032 and earlier accepted repairs remain closed. Candidate A and B remain closed.
- Developer-local clean exact-tree provenance for `021d79d`: `PYTHONDONTWRITEBYTECODE=1 bash scripts/check.sh` exit 0, `git diff --check` exit 0, clean worktree at pushed SHA, 2026-09-17T06:54:00Z → 2026-09-17T06:57:45Z, Linux x86_64, rustc 1.98.0 (88d9e12ae 2026-08-18). One flaky failure of `sigterm_after_ready_stops_and_releases_tcp_and_udp_bindings` on the first run; isolated rerun at the same SHA passed.
- GitHub-hosted Rust CI for exact `021d79d` is separately green (run `35191357800`); hosted CI is cross-evidence, not a substitute for developer-local provenance.
- Open PRs at review time: none.
- `READY_LIVE: none`; release item 3 incomplete; release item 4 incomplete; `RELEASE_CANDIDATE=false`; `PRODUCTION_READY=false`; `FREEZE=false`; `RELEASED=false`.

The external coding agent must synchronize to current `main` and continuously execute every dependency-ready slice below: implementation/review -> focused deterministic tests -> commit -> push -> next slice. Reviewer cadence is only a check frequency and is never a work-ticket length or reason to idle.

## Accepted progress that must remain closed

Do not revert these exact-current repairs without contradictory evidence:

- H-R9-039 P4-B server-exit assertion at `021d79d`.
- H-R9-038 P4 dual-domain residual diagnostics and exact-oracle strengthening at `236e962` + `9091803` + `021d79d`.
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

## R9-3 reviewer navigation — exact-current timing seam

At exact developer tree `021d79d`, the cross-process R9 no-loss path still uses recovery time `0` for every current reliable first send and for Carrier ACK application:

- initial reliable-owned Data calls `on_packet_sent(..., sent_at_us = 0, ...)`;
- post-return reliable Data also calls `on_packet_sent(..., sent_at_us = 0, ...)`;
- the shared authenticated Carrier ACK owner calls `apply_ack(&ranges, now_us = 0, ack_delay_us)`.

This is internally consistent only for the existing no-loss surface. R9-3 cannot prove a real PTO deadline while every timestamp is zero, and it must not timestamp a retransmission at positive `sent_at_us` while later applying its ACK with `now_us = 0`: Recovery RTT sampling can then hit checked subtraction and return `Error::Arithmetic`.

Use **one bounded monotonic relative recovery clock** for the complete cross-process reliable operation. Smallest current-semantics shape:

1. establish one local `Instant` origin for the reliable operation;
2. convert `elapsed().as_micros()` into the existing `u64` recovery time domain with checked/saturating conversion;
3. pass that time consistently to every relevant `on_packet_sent`, `on_retransmit_sent`, and Carrier `apply_ack` call owned by the operation;
4. keep the existing shared `recv_udp_delivery_ack` ACK parser/demux owner rather than creating a second owner;
5. keep Session DeliveryAck evidence separate from Carrier packet-ACK evidence.

### ACK observation-time refinement

`recv_udp_delivery_ack` may block in `recv_from` while waiting for a datagram. Therefore **do not sample one scalar `now_us` before entering the helper and reuse it after the blocking receive**. Thread the operation clock origin (or an equivalently tiny value source derived from it) into the existing ACK owner and sample elapsed time only after the authenticated Carrier ACK has actually been received/decrypted/decoded, immediately before `apply_ack`.

This placement is required because `Recovery::on_ack` performs checked `now_us - sent_at_us`, `RttEstimator::update` ignores a zero RTT sample, time-threshold loss remains disabled while `loss_delay_us()==0`, and PTO itself does not declare the suppressed original lost. A retransmission ACK must therefore not be allowed to retire only the fresh copy while an intentionally suppressed original silently remains in flight.

### PTO scheduling / API ownership refinement

The existing bounded `lab_pump` is the scheduling reference: drain ACKs, compute the PTO deadline from the oldest outstanding send plus committed Recovery PTO state, never fire before the deadline, call `pto_probe` at/after deadline, run `can_send` before retransmission ownership, re-seal the same logical Data under a fresh secure envelope/packet number, then call `on_retransmit_sent(fresh_pn, now_us, ..., stable_frame_id)` before socket send. Preserve current R9 `seal_unreliable(ProcessMessage::Data)` framing; do not copy the R8 lab's extra wire wrapper into R9.

Exact-current ownership is now narrow enough to implement without a maintainer decision:

- `Recovery` already owns the authoritative outstanding `SentPacket` map, RTT estimator and `pto_count`;
- `ReliableUdpRuntime::pto_probe()` advances PTO state, so the deadline must be queried before firing it;
- `ReliableUdpRuntime::on_retransmit_sent(...)` does not itself enforce congestion admission, so the caller must prove `can_send(exact_encoded_bytes)` before retransmission Recovery ownership commit;
- the R8 lab already uses `LAB_PTO_GRANULARITY_US = 1_000` and `LAB_MAX_ACK_DELAY_US = 0`. Those values may be reused for this existing M2 fixture baseline; do not turn them into a new global runtime/security policy.

Prefer a small **read-only Recovery/Runtime deadline query** over a second mutable ledger in `failover_client`. The smallest acceptable conceptual API is:

`next_pto_deadline_us(granularity_us, max_ack_delay_us) -> Option<u64>`

computed from the authoritative oldest outstanding ack-eliciting `SentPacket.sent_at_us` plus the current `rtt.pto_us(..., pto_count)`, then threaded read-only through `PathRecovery` / `ReliableUdpRuntime`. Exact function naming is not normative. Do not expose the mutable sent map, allocate a second packet number, or alter ACK/Session/crypto/wire semantics.

For the discriminating regression, suppress exactly one reliable-owned Data **after** congestion admission and Recovery ownership commit. Because exact congestion bytes are the encoded packet bytes, sealing may occur before the gate if needed to know the exact length; burning a fresh nonce is acceptable, nonce reuse is not. The invariant is:

`seal/current packet number -> can_send(exact encoded bytes) -> on_packet_sent -> [test-only one-shot socket-send suppression]`.

No socket send and no Recovery ownership commit may occur after a failed congestion gate.

# READY_LOCAL 1 — R9-3 Data-loss recovery

Implement and independently challenge the complete current M2 recovery contract on the cross-process R9 path:

- the selected first packet is Recovery-owned before intentional wire suppression;
- no PTO/retransmission event occurs before the computed deadline; diagnostic evidence must be strong enough to show `fired_at_us >= deadline_us`;
- add a deterministic just-before-deadline negative proving zero PTO/retransmit event, then advance to/after deadline and prove exactly one firing;
- after the deadline, exactly the expected stable `FrameId` probe is scheduled;
- retransmission preserves stream/offset/payload/logical identity and stable `FrameId`, but uses a fresh packet number / crypto nonce;
- every first send and retransmission calls `can_send` before committing Recovery ownership;
- receiver Session delivery for the logical range remains exactly once;
- Session DeliveryAck and Carrier ACK are separately observable and neither substitutes for the other;
- retransmission ACK application samples observation time after receipt from the same monotonic recovery origin and is not spuriously rejected;
- the positive Carrier retirement for the fresh retransmission is not by itself final settlement: prove the intentionally suppressed original is also removed by the committed loss rules and final Recovery `in_flight == 0` exactly;
- accepted-empty or typed-rejected ACK evidence cannot substitute for the required real retransmission retirement/loss closure;
- positive completion requires Session logical confirmation complete and Recovery `in_flight == 0`;
- any bounded terminal negative reports the actual residual domain and emits no false settlement/failover success.

If this focused proof exposes a concrete runtime contradiction, repair the smallest existing-semantics owner and add positive/negative regression. Do not invent a new ACK/retransmission architecture. Runtime orchestration/test-only changes do not require decode fuzz unless decoder/parser/crypto framing is actually changed.

# READY_LOCAL 2 — R9-4 ACK-loss + delayed original/reorder

Suppress one legitimate Carrier ACK, force a legitimate retransmission, then release the delayed original ACK. Require one logical Session delivery, Session dedup authority, fresh packet numbers/nonces, truthful PTO/loss/retransmit evidence and settled Recovery. A late original may be accepted-empty/stale but must never manufacture a second transition or rejection.

# READY_LOCAL 3 — R9-5 adversarial feedback / every Carrier caller

Challenge future/never-sent/stale/duplicate/tampered feedback across every exact-current `UdpAcknowledgement::Carrier` continuation, including initial receive, settlement and post-return owners. Rejected feedback must be atomic and fail closed without RTT/PTO/loss/cwnd/Session mutation; accepted-empty must not become transition or rejection. H-R9-033/H-R9-034 are closed examples, not reasons to skip the rest of the exact-current call surface.

# READY_LOCAL 4 — R9-6 ownership/resource boundedness

Every first send and retransmit must consult congestion admission before ownership commit; refusal commits no recovery or logical state. Keep one bounded retransmit-plaintext owner and deterministic teardown. Challenge algorithmic boundedness without adding a capacity-pressure benchmark or inventing new capacity/security values.

# READY_LOCAL 5 — R9-7 process/result truth

Independently challenge that Data, Carrier ACK, Session DeliveryAck, PTO/retransmit, Recovery, Session delivery, malformed budget, accepted-empty feedback, rejected feedback and terminal result remain distinct in structured process evidence. Emit/accept a diagnostic only after the exact claimed transition or classification has occurred.

# READY_LOCAL 6 — R9-8 warm TCP readiness

Challenge the existing single-active/multi-ready contract: authenticated resume-bound warm standby may carry readiness/control only and **no application Data before promotion**. Readiness/resource admission remains separate from packet feedback and Session delivery evidence. Use D064/current Carrier Manager semantics; no architecture change.

# READY_LOCAL 7 — R9-9 health + promotion

Challenge the integrated path rather than isolated helpers:

- resolved UDP outcomes feed existing health/hysteresis;
- recoverable reliable-UDP loss remains on UDP instead of spuriously promoting TCP;
- promotion happens only after existing readiness/health gates;
- draining/failed UDP receives no new Data ownership.

# READY_LOCAL 8 — R9-10 uncertain replay + dedup + cleanup

Challenge only genuine uncertain Session ranges replaying on promoted TCP; receiver Session dedup must remain exactly-once; TCP gets no duplicate UDP packet-ACK layer; shutdown/error negatives must clean resources and emit no false success.

# READY_LOCAL 9 — R9-11 lifecycle/terminal invariants

Close remaining lifecycle and terminal invariants for the materially new cross-process reliable-UDP/failover integration. Keep terminal classification distinct from intermediate diagnostics and preserve deterministic cleanup.

# READY_LOCAL 10 — R9-12 complete cross-process slice + exact-tree gate

Finish the remaining R9 cross-process implementation/test slice and run its clean exact-tree developer-local gate. Preserve source/test/evidence provenance boundaries; do not rewrite historical live evidence to describe the new local tree.

# READY_LOCAL 11 — dedicated independent R9 review

After the implementation/test lanes above are reachable and gated, perform a dedicated independent bounded challenge of materially new R9 send/admission/recovery/demux/Session ACK/Carrier ACK/failover/migration/cleanup/diagnostic behavior.

A bounded no-finding review is valid release-item-4 support if scope, inspected owners, tests/commands, exclusions and reachable anchor are explicit. Any concrete BLOCKER/HIGH returns immediately to smallest repair -> discriminating regression -> exact-tree local gate.

# READY_LOCAL 12 — Q10/Q11/Q12 factual reconciliation

Only after a reachable independent R9 review anchor exists, factually reconcile `docs/status.md`, `docs/release-security-review-packet.md` and related Q10/Q11/Q12 evidence text with exact reachable implementation/review anchors. Preserve historical evidence boundaries and do **not** change RC/freeze/release/production authority flags.

## Item-4 / core-surface inventory

Reachable independent bounded review already exists for earlier reliable-UDP engine basics (including the future/never-sent ACK guard), `CarrierState`, concurrent Carrier Manager/health/migration-back, FairScheduler/flow accounting, carrier adapters, `SessionRuntime`, observability (including mixed queue/generic drop classification), package/reproducibility/operator scripts, dependency/build surface, CLI portability/output, algorithmic boundedness/validators, DeliveryLedger/process codec/datagram/crypto API, and wire/parser surfaces.

The **materially new cross-process R9 integration** remains the broad uncovered item-4 surface until READY_LOCAL 11. Therefore repository-wide queue exhaustion is false even if any one narrow seam yields no finding.

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