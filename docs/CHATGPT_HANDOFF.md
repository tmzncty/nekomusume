# ChatGPT reviewer handoff — H-R9-040 closed at be50ed5; R9-3 data-loss recovery done at 92d5b8d

## Current repository truth

- Latest developer-owned source/test commit: exact `92d5b8d80da645c122d679a33a806fd0c6e59ce8` (`feat(cli): R9-3 data-loss recovery — PTO retransmit + monotonic clock + acked_frames prune`).
- H-R9-040 is closed at exact `be50ed5955b7eaba88275cbcbd0be3addf1dd43f` (`fix(reliable): H-R9-040 acked_frames gate`): positively-ACKed frames are marked in `acked_frames`, skipped in `on_pto` and loss-driven `retransmit_frames`, and pruned when the final copy retires. Same-call ACK+time-loss and split-call ACK+later-loss regressions pass.
- R9-3 data-loss recovery is implemented at exact `92d5b8d`: `--drop-r9-data` suppresses one reliable-owned post-return Data after congestion admission and `on_packet_sent` commit; `post_recovery_epoch` is the monotonic `Instant` origin for the post-return operation; `apply_ack` samples `elapsed()` after authenticated receive; `next_pto_deadline_us` is the read-only Recovery deadline query; `r9_udp_pto_fired` emits `deadline_us`/`fired_at_us`/`pto_count` before `pto_probe`; `r9_udp_retransmit_sent` emits fresh `packet_number`, stable `frame`, and `original_packet_number`; server post-return receive continues after `migration_back_complete` so a retransmitted copy is still ACKed. The discriminating regression `reliable_udp_post_return_data_loss_recovers_via_pto_retransmit` proves Recovery-owned suppression, PTO timing evidence, fresh-pn retransmit, three-way packet bind, residual `remaining_in_flight=0`, zero misclassification, and no false settlement.
- Prior independent reviewer finding: [`docs/reviews/reviewer-r9-3-overlap-ack-retransmit-021d79d-20260917.md`](reviews/reviewer-r9-3-overlap-ack-retransmit-021d79d-20260917.md), reachable at exact `22fb018`.
- Existing R9-3 navigation notes remain valid and are now implemented:
  - [`docs/reviews/reviewer-r9-3-pto-timebase-navigation-021d79d-20260917.md`](reviews/reviewer-r9-3-pto-timebase-navigation-021d79d-20260917.md), reachable at `319c6bb`;
  - [`docs/reviews/reviewer-r9-3-ack-observation-time-021d79d-20260917.md`](reviews/reviewer-r9-3-ack-observation-time-021d79d-20260917.md), reachable at `938e290`;
  - [`docs/reviews/reviewer-r9-3-pto-api-ownership-021d79d-20260917.md`](reviews/reviewer-r9-3-pto-api-ownership-021d79d-20260917.md), reachable at `bb96fab`.
- H-R9-039 and H-R9-038 remain closed at `021d79d` / `9091803`; P2 C1-C4 and H-R9-037 through earlier accepted findings remain closed. Candidate A (future/never-sent ACK) and candidate B (mixed datagram-drop observability) remain closed.
- Developer-local clean exact-tree provenance for exact `92d5b8d`: `PYTHONDONTWRITEBYTECODE=1 bash scripts/check.sh` exit 0, `git diff --check` exit 0, clean worktree at pushed SHA, 2026-09-17T16:06:20Z → 2026-09-17T16:10:43Z, Linux x86_64, rustc 1.98.0 (88d9e12ae 2026-08-18).
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
- H-R9-034 settlement-continuation accepted-empty classification and late-stale proof at `b801b65`; H-R9-033 initial-loop accepted-empty classification at `7a7c48c`; H-R9-032 packet-number cross-bind/settlement proof at `ddafbb1`; earlier ACK-obligation/classification/identity/malformed/order/count repairs remain accepted.
- Session DeliveryAck and Carrier packet ACK remain separate evidence domains. Do not merge them to solve H-R9-040 or R9-3.

# READY_LOCAL 1 — CLOSED at be50ed5 + 92d5b8d

H-R9-040 overlap ACK retransmit eligibility is closed at `be50ed5` (`acked_frames` gate + `next_pto_deadline_us`); R9-3 data-loss recovery is implemented at `92d5b8d` (`--drop-r9-data` seam, `post_recovery_epoch` monotonic clock, `r9_udp_pto_fired` timing evidence, `r9_udp_retransmit_sent` fresh-pn bind, server post-return continuation). All required regressions pass.

# READY_LOCAL 2 — CLOSED at 92d5b8d

R9-3 data-loss recovery is implemented and verified. See READY_LOCAL 1 closure for details.

### One authoritative monotonic recovery clock

At exact `021d79d`, current cross-process reliable sends and Carrier ACK application use recovery time `0`. Use one bounded local `Instant` origin for the complete reliable operation and convert `elapsed().as_micros()` deterministically into the existing `u64` recovery domain. Pass this time consistently to all relevant first-send `on_packet_sent`, retransmit `on_retransmit_sent`, and Carrier `apply_ack` calls.

`recv_udp_delivery_ack` may block. Do **not** sample `now_us` before entering it and reuse that stale scalar. Thread the origin/equivalent tiny time source into the existing shared authenticated ACK owner and sample elapsed time only after the Carrier ACK has actually been received/decrypted/decoded, immediately before `apply_ack`.

### Authoritative PTO deadline ownership

Prefer a read-only query through `Recovery` -> `PathRecovery` -> `ReliableUdpRuntime`, conceptually:

`next_pto_deadline_us(granularity_us, max_ack_delay_us) -> Option<u64>`

computed from the authoritative oldest outstanding ack-eliciting `SentPacket.sent_at_us` plus current `rtt.pto_us(..., pto_count)`. Do not expose the mutable sent map or create a second mutable timer/packet ledger in `failover_client`.

The existing bounded M2 lab baseline `LAB_PTO_GRANULARITY_US = 1_000` and `LAB_MAX_ACK_DELAY_US = 0` may be reused for this fixture. They are not a new global policy.

### Send/retransmit order

For the intentionally suppressed first wire send:

`seal/current packet number -> can_send(exact encoded bytes) -> on_packet_sent -> test-only one-shot socket-send suppression`.

For retransmission:

- drain authenticated ACKs first;
- query deadline before calling `pto_probe` because `pto_probe` mutates `pto_count`;
- never fire before deadline;
- at/after deadline require exactly the expected stable `FrameId` probe;
- re-seal the same logical Data under a fresh packet number/nonce;
- call `can_send(exact encoded bytes)` before `on_retransmit_sent`;
- commit retransmission ownership before socket send;
- apply its Carrier ACK at observation time from the same monotonic origin.

### R9-3 acceptance

- selected first packet is Recovery-owned before intentional wire suppression;
- deterministic just-before-deadline negative proves zero PTO/retransmit event; at/after deadline exactly one firing with `fired_at_us >= deadline_us`;
- retransmission preserves stable FrameId and stream/offset/len/payload logical identity but uses fresh packet number/nonce;
- every first/retransmit send proves congestion admission before Recovery ownership commit;
- receiver Session logical delivery remains exactly once;
- Session DeliveryAck and Carrier packet ACK remain separately observable;
- fresh retransmission ACK is a real positive retirement, not accepted-empty/rejected substitution;
- the suppressed original is removed by committed loss rules without H-R9-040-style resurrection, and final Recovery `in_flight==0` exactly;
- positive completion requires Session confirmation complete **and** Recovery zero;
- bounded negatives report actual residual domain and emit no false settlement/failover success.

If this reveals another concrete current-semantics defect, repair the smallest owner + discriminating regression, gate it, and continue.

# READY_LOCAL 3 — R9-4 ACK-loss + delayed original/reorder

Suppress one legitimate Carrier ACK, force a legitimate retransmission, then release the delayed original ACK. Require one logical Session delivery, Session dedup authority, fresh packet numbers/nonces, truthful PTO/loss/retransmit evidence and settled Recovery. A late original may classify accepted-empty/stale but must never manufacture another transition, rejection or retransmit after positive frame receipt.

# READY_LOCAL 4 — R9-5 adversarial feedback / every Carrier caller

Challenge future/never-sent/stale/duplicate/tampered feedback across every exact-current `UdpAcknowledgement::Carrier` continuation: initial receive, settlement and post-return owners. Rejected feedback must be atomic and fail closed without RTT/PTO/loss/cwnd/Session mutation; accepted-empty must not become transition/rejection. Recheck overlap-copy feedback in light of H-R9-040.

# READY_LOCAL 5 — R9-6 ownership/resource boundedness

Every first send and retransmit must consult congestion admission before ownership commit; refusal commits no recovery/logical state. Keep one bounded retransmit-plaintext owner, include H-R9-040 per-frame state in the boundedness challenge, and prove deterministic teardown. Do not add a capacity-pressure benchmark or invent capacity/security values.

# READY_LOCAL 6 — R9-7 process/result truth

Independently challenge that Data, Carrier ACK, Session DeliveryAck, PTO/retransmit, Recovery ACK/loss, Session delivery, malformed budget, accepted-empty feedback, rejected feedback and terminal result remain distinct in structured process evidence. Emit/accept diagnostics only after the claimed transition/classification occurred.

# READY_LOCAL 7 — R9-8 warm TCP readiness

Challenge the existing D064 single-active/multi-ready contract: authenticated resume-bound warm standby may carry readiness/control only and no application Data before promotion. Readiness/resource admission remains separate from packet feedback and Session delivery. No architecture change.

# READY_LOCAL 8 — R9-9 health + promotion

Challenge the integrated path: resolved UDP outcomes feed current health/hysteresis; recoverable reliable-UDP loss remains on UDP rather than spuriously promoting TCP; promotion requires existing readiness/health gates; draining/failed UDP receives no new Data ownership.

# READY_LOCAL 9 — R9-10 uncertain replay + dedup + cleanup

Challenge only genuine uncertain Session ranges replaying on promoted TCP; receiver Session dedup remains exactly-once; TCP gets no duplicate UDP packet-ACK layer; shutdown/error negatives clean resources and emit no false success.

# READY_LOCAL 10 — R9-11 lifecycle/terminal invariants

Close remaining lifecycle/terminal invariants for the materially new cross-process reliable-UDP/failover integration. Terminal classification stays distinct from intermediate diagnostics and cleanup is deterministic.

# READY_LOCAL 11 — R9-12 complete cross-process slice + exact-tree gate

Finish remaining R9 implementation/test work and run the clean exact-tree developer-local gate on the final pushed source/test SHA. Preserve source/test/provenance/hosted-CI/live-evidence boundaries; do not rewrite historical live evidence to describe the new local tree.

# READY_LOCAL 12 — dedicated independent R9 review

After READY_LOCAL 1-11 are reachable and gated, perform a dedicated independent bounded challenge of materially new R9 send/admission/recovery/PTO/demux/Session ACK/Carrier ACK/failover/migration/cleanup/diagnostic behavior. A bounded no-finding review is valid item-4 support if scope, inspected owners, commands/tests, exclusions and exact reachable anchor are explicit. Any concrete BLOCKER/HIGH returns immediately to smallest repair -> discriminating regression -> exact-tree local gate.

# READY_LOCAL 13 — Q10/Q11/Q12 factual reconciliation

Only after a reachable independent R9 review anchor exists, factually reconcile `docs/status.md`, `docs/release-security-review-packet.md` and related Q10/Q11/Q12 evidence text with exact reachable implementation/review anchors. Preserve historical evidence boundaries and do not change RC/freeze/release/production authority flags.

## Item-4 / core-surface inventory

Reachable independent bounded review already exists for earlier reliable-UDP engine basics (including future/never-sent ACK guard), `CarrierState`, concurrent Carrier Manager/health/migration-back, FairScheduler/flow accounting, carrier adapters, `SessionRuntime`, observability including mixed queue/generic drop classification, package/reproducibility/operator scripts, dependency/build surface, CLI portability/output, algorithmic boundedness/validators, DeliveryLedger/process codec/datagram/crypto API and wire/parser surfaces.

H-R9-040 is a newly discovered overlap-copy recovery defect within the materially new R9 challenge. The **cross-process R9 integration remains broad uncovered item-4 work** until READY_LOCAL 12. Repository-wide queue exhaustion is false.

## VPS opportunity

**Not READY.** Standing authorization remains valid, but authoritative classification is `READY_LIVE: none`. Current work is local recovery correctness/evidence and the later independent cross-process R9 review. Do not repeat HY2, warm failover, periodic/soak, package lifecycle, migration-back, endpoint/key migration, IPv6, PLPMTUD or Experimental Track evidence merely because the VPS remains rented.

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
