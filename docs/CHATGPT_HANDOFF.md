# ChatGPT reviewer handoff — current-head R9-3 reopened on repeated-PTO gate failure

## Current repository truth

- Latest developer-owned source/test commit reviewed: exact `c1a22e5095c81132786da7b4f3298adf1f560970` (`test(cli): H-R9-042 three-way cross-process retransmit bind — server Carrier ACK packet_number matches retransmit`).
- Its developer parent `9a113f2e95bdc1003995d72e49cf90b71b3c0d10` implements H-R9-040 lifecycle pruning and H-R9-041 exact encoded-wire retransmit admission.
- Current independent reviewer anchor: exact `d50bec9924eab48b9a45ccb24716bb2c4c154540`, [`docs/reviews/reviewer-r9-3-current-head-c1a22e5-20260918.md`](reviews/reviewer-r9-3-current-head-c1a22e5-20260918.md).
- GitHub-hosted Rust CI for exact `c1a22e5`, run `35254921227`, is **FAILED**: `stable checks -> bash scripts/check.sh` failed in `reliable_udp_post_return_data_loss_recovers_via_pto_retransmit`; `nightly decode fuzz smoke` succeeded. Hosted CI is cross-evidence only and this failure is a real current-tree gate blocker.
- The failed trace shows two legitimate PTO/retransmit copies before the first positive Carrier retirement: original PN 4 suppressed; PTO sends PN 5; exact Session DeliveryAck arrives; a later PTO sends PN 6; a late sibling Carrier ACK classifies accepted-empty; PN 6 receives positive Carrier retirement; final settlement still reaches `remaining_in_flight=0`. The test currently fails because it requires exactly one PTO/retransmit.
- No final developer-local clean exact-tree provenance is accepted for exact `c1a22e5`. The older provenance recorded for exact `92d5b8d` remains historical evidence for that SHA only and must not be relabeled as current-head evidence.
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

# READY_LOCAL 1 — H-R9-043: repair the repeated-PTO R9-3 oracle / exact-tree failure

**HIGH evidence/correctness gate blocker.** Exact `c1a22e5` fails its full hosted stable gate.

The current failing trace is important repository truth, not flaky noise. Before any positive Carrier retirement, the authoritative Recovery still has the frame outstanding. A second PTO can therefore select the same stable frame and create another fresh packet copy. The Session DeliveryAck arriving between those PTOs is a different evidence domain and must not be used as a substitute Carrier ACK.

Do **not** “fix” this by merging Session delivery confirmation into packet recovery or by suppressing Carrier recovery solely because Session bytes were confirmed.

### Smallest repair/challenge contract

Use current committed Recovery semantics and make the process oracle deterministic:

1. Preserve separate Session DeliveryAck and Carrier packet-ACK ownership.
2. If no positive Carrier retirement has occurred, bounded repeated PTO probes are legal. Do not require exactly one retransmit merely because the first retransmit has been sent.
3. Every retransmission must keep stable FrameId + stream/offset/len/payload logical identity and use a fresh packet number/nonce.
4. Bind the **actually positive-retired packet copy** to its matching server `udp_return_packet_ack_sent` and client retransmit event. Do not assume it must be the first retransmit if another PTO became due first.
5. A late ACK for a sibling copy after another copy has resolved the frame may classify accepted-empty. It must not become rejection, second Session transition, or new retransmit after positive frame retirement.
6. Receiver Session delivery/confirmation remains exactly once for `stream=1 offset=48 len=16` despite overlapping packet copies.
7. Once a positive Carrier ACK closes the frame lifecycle, no later PTO/retransmit for that frame is allowed.
8. Positive completion still requires Session confirmation complete **and** Recovery `in_flight==0`; exactly one final settlement with `remaining_in_flight=0`.
9. Both processes must succeed.

If a single-retransmission process fixture is preferred, make it a deterministic fixture condition by ensuring/observing the first retransmit Carrier ACK before the next authoritative PTO deadline. Do not alter Recovery semantics and do not use sleep timing as the acceptance oracle.

If this stronger challenge exposes a genuine state-machine contradiction rather than oracle overconstraint, repair the smallest owner + regression and continue immediately.

# READY_LOCAL 2 — finish H-R9-041 exact-wire refusal regression

The code ordering is repaired at `9a113f2`, but the required discriminating regression is still missing.

Create a deterministic test using existing congestion/accounting state such that the remaining send budget is sufficient for the retained plaintext length but insufficient for the exact encoded/sealed retransmission length. Prove:

- retransmit admission is refused on exact wire bytes;
- no retransmission packet ownership is committed;
- no socket send occurs;
- no `r9_udp_retransmit_sent` event is emitted;
- no Session delivery state is fabricated;
- a paired admitted control commits/sends normally.

Do not invent a new cwnd/capacity/security policy value. No decoder/framing change is expected; do not mechanically run fuzz solely for this regression.

# READY_LOCAL 3 — finish H-R9-042 / R9-3 pre-deadline acceptance

After READY_LOCAL 1-2, close the remaining deadline oracle gap.

Add a deterministic **just-before-deadline negative** at the Recovery/runtime query level: at `deadline_us - 1` (or equivalent injected/query-level time), prove zero PTO transition and zero retransmit transition. Then at/after the authoritative deadline prove the expected probe behavior. Wall-clock sleep is not the oracle.

Reconcile the positive process test with H-R9-043 so its cardinality/accepted-empty claims match actual overlap-copy semantics rather than assuming a single total PTO. Preserve:

- recovery-owned suppressed first packet;
- monotonic observation-time clock;
- authoritative read-only PTO deadline;
- exact encoded-byte admission before ownership commit;
- stable logical/frame identity + fresh packet numbers/nonces;
- exact once Session confirmation;
- truthful Carrier positive/accepted-empty/rejected classification;
- final Recovery zero + one settlement after both evidence domains complete.

Then run the developer-local clean exact-tree gate on the final pushed source/test SHA and persist exact provenance: `PYTHONDONTWRITEBYTECODE=1 bash scripts/check.sh`, `git diff --check`, clean tree, exact SHA, UTC start/end, OS/arch, stable Rust version, exit codes. If actual changes touch decoder/parser/crypto framing, additionally use the pinned fuzz toolchain; otherwise do not mechanically run fuzz.

# READY_LOCAL 4 — R9-4 ACK-loss + delayed original/reorder

Suppress one legitimate Carrier ACK, force legitimate retransmission, then release delayed original feedback. Require one logical Session delivery, Session dedup authority, fresh packet numbers/nonces, truthful PTO/loss/retransmit evidence and settled Recovery. Late sibling/original feedback may classify accepted-empty/stale after a positive copy retirement but must never manufacture another Session transition, rejection or retransmit.

# READY_LOCAL 5 — R9-5 adversarial feedback / every Carrier caller

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