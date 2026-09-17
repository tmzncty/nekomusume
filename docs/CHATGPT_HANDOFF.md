# ChatGPT reviewer handoff — H-R9-043 closed at 5be123b; R9-3 blocked on final provenance + missing negatives

## Current repository truth

- Latest developer-owned source/test commit: exact `5be123b8aee40a8487e995927dde23283f97b20d` (`test(cli): H-R9-043 bounded repeated PTO — accept legal repeat probes before first Carrier retirement`).
- Its parent `c1a22e5` adds the H-R9-042 server Carrier ACK packet-number bind; `9a113f2` implements H-R9-040 lifecycle pruning and H-R9-041 exact encoded-wire retransmit admission.
- Prior independent reviewer finding: [`docs/reviews/reviewer-r9-3-current-head-c1a22e5-20260918.md`](reviews/reviewer-r9-3-current-head-c1a22e5-20260918.md), reachable at exact `d50bec9`.
- H-R9-043 is closed at exact `5be123b`: the R9-3 fixture now accepts bounded repeated PTO probes before the first positive Carrier retirement, binds the LAST retransmit to the final positive retirement and server Carrier ACK, keeps Session DeliveryAck exactly-once as a separate evidence domain, and requires `remaining_in_flight=0` settlement.
- H-R9-040 and H-R9-041 remain closed at `9a113f2`; H-R9-042 remains closed at `c1a22e5` + `5be123b`.
- Still missing before R9-3 completion: the H-R9-041 discriminating exact-wire refusal regression and the H-R9-042 deterministic just-before-PTO-deadline negative.
- Developer-local clean exact-tree provenance for exact `5be123b`: `PYTHONDONTWRITEBYTECODE=1 bash scripts/check.sh` exit 0, `git diff --check` exit 0, clean worktree at pushed SHA, 2026-09-17T18:48:54Z → 2026-09-17T18:56:28Z, Linux x86_64, rustc 1.98.0 (88d9e12ae 2026-08-18).
- GitHub-hosted Rust CI for exact `c1a22e5`, run `35254921227`, previously failed in the pre-`5be123b` oracle; hosted CI is cross-evidence only and does not replace developer-local provenance for the current head.
- Open PRs at review time: none.
- Candidate A (future/never-sent ACK atomic rejection) remains closed. Candidate B (mixed generic/queue datagram-drop observability) remains closed.
- `READY_LIVE: none`; release item 3 incomplete; release item 4 incomplete; `RELEASE_CANDIDATE=false`; `PRODUCTION_READY=false`; `FREEZE=false`; `RELEASED=false`.

The external coding agent must synchronize to current `main` and continuously execute every dependency-ready slice below: implementation/review -> focused deterministic tests -> commit -> push -> next slice. Reviewer cadence is only a check frequency and is never a work-ticket length or reason to idle.

## Accepted progress that must remain closed

Do not revert these exact-current repairs without contradictory repository evidence:

- H-R9-040 source repair at `9a113f2`: `acked_frames` is lifecycle-scoped, survives only while an ACKed frame still has sibling packet copies outstanding, and is pruned when the final copy retires on ACK or loss. The single-copy FrameId reuse and sibling-copy regressions directly challenge stale-marker poisoning.
- H-R9-041 implementation ordering at `9a113f2`: post-return and `lab_pump` retransmissions now build/seal first, call `can_send` on exact encoded wire bytes, then commit `on_retransmit_sent` with the same byte count before socket send. **Only the implementation seam is closed; the discriminating refusal regression remains open below.**
- H-R9-042 server side packet-number binding at `c1a22e5`: the R9-3 process test now requires exactly one server `udp_return_packet_ack_sent` and binds its packet number to the expected retransmitted packet number under the current single-retransmit oracle. **The deterministic pre-deadline negative remains open below.**
- H-R9-039 P4-B server-exit assertion at `021d79d`.
- H-R9-038 P4 dual-domain residual diagnostics and exact-oracle strengthening at `236e962` + `9091803` + `021d79d`.
- H-R9-037 reverse-order true three-way packet bind at `d5d5b23`, independently reviewed at `f2529087`.
- H-R9-036 reverse-order exact oracle at `855c679`.
- Post-return ACK arrival-order challenge at `83d10b0` + `424583d` + `855c679` + `d5d5b23`.
- P2 C1-C4 exact closure at `142f0a1` + `8d0ccd5` + `1f8bbb0` + `be03dc3`: TCP replay identity bound on both sides; migration milestones exact cardinality/order; server dual-domain stream/offset/len; positive-path three-way packet-number equality; exactly-one settled cardinality and post-transition order.
- H-R9-034 settlement-continuation accepted-empty classification and late-stale proof at `b801b65`; H-R9-033 initial-loop accepted-empty classification at `7a7c48c`; H-R9-032 packet-number cross-bind/settlement proof at `ddafbb1`; earlier ACK-obligation/classification/identity/malformed/order/count repairs remain accepted.
- Session DeliveryAck and Carrier packet ACK remain separate evidence domains. Do not merge them to make the current R9-3 fixture pass.

# READY_LOCAL 1 — CLOSED at 5be123b

H-R9-043 bounded repeated PTO oracle is repaired at `5be123b`: the fixture accepts >=1 PTO/retransmit, binds the LAST retransmit to final positive retirement + server ACK, preserves separate Session/Carrier domains, and requires `remaining_in_flight=0`.
8. Positive completion still requires Session confirmation complete **and** Recovery `in_flight==0`; exactly one final settlement with `remaining_in_flight=0`.
9. Both processes must succeed.

If a single-retransmission process fixture is preferred, make it a deterministic fixture condition by ensuring/observing the first retransmit Carrier ACK before the next authoritative PTO deadline. Do not alter Recovery semantics and do not use sleep timing as the acceptance oracle.

If this stronger challenge exposes a genuine state-machine contradiction rather than oracle overconstraint, repair the smallest owner + regression and continue immediately.

# READY_LOCAL 2 — CLOSED at 5be123b (oracle repaired); H-R9-041 regression missing — see READY_LOCAL 5

# READY_LOCAL 3 — CLOSED at 5be123b (oracle repaired); missing negatives remain

H-R9-043 bounded repeated PTO oracle is repaired at `5be123b`: the fixture accepts >=1 PTO/retransmit, binds the LAST retransmit to final positive retirement + server ACK, preserves separate Session/Carrier domains, and requires `remaining_in_flight=0`. The H-R9-042 deterministic just-before-deadline negative and H-R9-041 exact-wire refusal regression are still missing — they are READY_LOCAL 4 and 5 below.

# READY_LOCAL 4 — R9-4 ACK-loss + delayed original/reorder

Suppress one legitimate Carrier ACK, force legitimate retransmission, then release delayed original feedback. Require one logical Session delivery, Session dedup authority, fresh packet numbers/nonces, truthful PTO/loss/retransmit evidence and settled Recovery. Late sibling/original feedback may classify accepted-empty/stale after a positive copy retirement but must never manufacture another Session transition, rejection or retransmit.

# READY_LOCAL 4 — H-R9-042 deterministic pre-deadline negative

Add a Recovery/runtime-level deterministic challenge at `deadline_us - 1` (or equivalent injected/query-level time): prove zero PTO probe/retransmit transition before the authoritative deadline. Then at/after the deadline prove the expected probe behavior. Wall-clock sleep is not the oracle. Reuse existing M2 lab granularity/ack-delay inputs already selected by the fixture; do not invent a new PTO policy number.

# READY_LOCAL 5 — H-R9-041 exact-wire refusal regression

Create a deterministic test using existing congestion/accounting state such that the remaining send budget is sufficient for the retained plaintext length but insufficient for the exact encoded/sealed retransmission length. Prove: retransmit admission is refused on exact wire bytes; no retransmission packet ownership is committed; no socket send occurs; no `r9_udp_retransmit_sent` event is emitted; no Session delivery state is fabricated; a paired admitted control commits/sends normally. Do not invent a new cwnd/capacity/security policy value. No decoder/framing change is expected; do not mechanically run fuzz solely for this regression.

# READY_LOCAL 6 — R9-5 adversarial feedback / every Carrier caller

Challenge future/never-sent/stale/duplicate/tampered feedback across every exact-current `UdpAcknowledgement::Carrier` continuation: initial receive, settlement and post-return owners. Rejected feedback must be atomic and fail closed without RTT/PTO/loss/cwnd/Session mutation; accepted-empty must not become transition/rejection. Recheck overlap-copy feedback after H-R9-040/H-R9-043.

# READY_LOCAL 6 — R9-6 ownership/resource boundedness

Every first send and retransmit must consult congestion admission before ownership commit; refusal commits no recovery/logical state. Keep one bounded retransmit-plaintext owner, include H-R9-040 per-frame state in the boundedness challenge, and prove deterministic teardown. Do not add a capacity-pressure benchmark or invent capacity/security values.

# READY_LOCAL 7 — R9-7 process/result truth

Independently challenge that Data, Carrier ACK, Session DeliveryAck, PTO/retransmit, Recovery ACK/loss, Session delivery, malformed budget, accepted-empty feedback, rejected feedback, residual-domain failure and terminal result remain distinct in structured process evidence. Emit diagnostics only after the claimed transition/classification occurred.

# READY_LOCAL 8 — R9-8 warm TCP readiness

Challenge the existing D064 single-active/multi-ready contract: authenticated resume-bound warm standby may carry readiness/control only and no application Data before promotion. Readiness/resource admission remains separate from packet feedback and Session delivery. No architecture change.

# READY_LOCAL 9 — R9-9 health + promotion

Challenge the integrated path: resolved UDP outcomes feed current health/hysteresis; recoverable reliable-UDP loss remains on UDP rather than spuriously promoting TCP; promotion requires existing readiness/health gates; draining/failed UDP receives no new Data ownership.

# READY_LOCAL 10 — R9-10 uncertain replay + dedup + cleanup

Challenge only genuine uncertain Session ranges replaying on promoted TCP; receiver Session dedup remains exactly-once; TCP gets no duplicate UDP packet-ACK layer; shutdown/error negatives clean resources and emit no false success.

# READY_LOCAL 11 — R9-11 lifecycle/terminal invariants

Close remaining lifecycle/terminal invariants for the materially new cross-process reliable-UDP/failover integration. Terminal classification stays distinct from intermediate diagnostics and cleanup is deterministic.

# READY_LOCAL 12 — R9-12 complete cross-process slice + exact-tree gate

Finish remaining R9 implementation/test work and run the clean exact-tree developer-local gate on the final pushed source/test SHA. Preserve source/test/provenance/hosted-CI/live-evidence boundaries; do not rewrite historical live evidence to describe the new local tree.

# READY_LOCAL 13 — dedicated independent R9 review

After READY_LOCAL 1-12 are reachable and gated, perform a dedicated independent bounded challenge of materially new R9 send/admission/recovery/PTO/demux/Session ACK/Carrier ACK/failover/migration/cleanup/diagnostic behavior. A bounded no-finding review is valid item-4 support if scope, inspected owners, commands/tests, exclusions and exact reachable anchor are explicit. Any concrete BLOCKER/HIGH returns immediately to smallest repair -> discriminating regression -> exact-tree local gate.

# READY_LOCAL 14 — Q10/Q11/Q12 factual reconciliation

Only after a reachable independent R9 review anchor exists, factually reconcile `docs/status.md`, `docs/release-security-review-packet.md` and related Q10/Q11/Q12 evidence text with exact reachable implementation/review anchors. Preserve historical evidence boundaries and do not change RC/freeze/release/production authority flags.

## Item-4 / core-surface inventory

Reachable independent bounded review already exists for earlier reliable-UDP engine basics (including future/never-sent ACK guard), `CarrierState`, concurrent Carrier Manager/health/migration-back, FairScheduler/flow accounting, carrier adapters, `SessionRuntime`, observability including mixed queue/generic drop classification, package/reproducibility/operator scripts, dependency/build surface, CLI portability/output, algorithmic boundedness/validators, DeliveryLedger/process codec/datagram/crypto API and wire/parser surfaces.

The materially new cross-process R9 integration remains broad uncovered item-4 work until READY_LOCAL 13. H-R9-043 plus the remaining H-R9-041/H-R9-042 acceptance gaps are concrete dependency-ready local work, so repository-wide queue exhaustion is false.

## VPS opportunity

**Not READY.** Standing authorization remains valid, but authoritative classification is `READY_LIVE: none`. The current exact-head failure and R9 review gaps are local correctness/evidence questions. Do not repeat HY2, warm failover, periodic/soak, package lifecycle, migration-back, endpoint/key migration, IPv6, PLPMTUD or Experimental Track evidence merely because the VPS remains rented.

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