# ChatGPT reviewer handoff — H-R9-044 reopens R9-3 repeated-PTO acceptance

## Current repository truth

- Latest developer-owned source/test commit remains exact `5be123b8aee40a8487e995927dde23283f97b20d` (`test(cli): H-R9-043 bounded repeated PTO — accept legal repeat probes before first Carrier retirement`).
- Current `main` includes the independent reviewer note [`docs/reviews/reviewer-r9-3-hosted-repeat-pto-5be123b-20260918.md`](reviews/reviewer-r9-3-hosted-repeat-pto-5be123b-20260918.md), which reopens acceptance as H-R9-044. No source/test owner changed after `5be123b`.
- `9a113f2` remains the H-R9-040 lifecycle-pruning repair plus the H-R9-041 exact encoded-wire retransmit-admission implementation repair. `c1a22e5` added the server Carrier-ACK packet-number side of the R9-3 oracle.
- H-R9-043's **semantic conclusion remains accepted**: bounded repeated PTO before the first positive Carrier retirement is legal while sibling copies remain outstanding; Session DeliveryAck stays a separate evidence domain and must not suppress Carrier recovery.
- H-R9-044 is **HIGH evidence/correctness**: exact `5be123b` still asserts zero `r9_udp_return_packet_ack_accepted_empty`, even though legal repeated PTO can create sibling copies whose late ACK is correctly classified accepted-empty. The same fixture also says all PTOs obey their deadlines but checks only `pto_ev[0]`.
- GitHub-hosted Rust CI for exact `5be123b`, run `35260878111`, failed `stable checks -> bash scripts/check.sh`; nightly decode fuzz smoke passed. The failing trace contains two legal PTO/retransmits, exact Session ACK, one accepted-empty sibling Carrier ACK, one positive Carrier retirement, and final `remaining_in_flight=0` settlement.
- The docs-only descendant `1d90f82` reran the unchanged source/test tree in hosted run `35262066403` and failed the same fixture with the same legal two-PTO / accepted-empty / final-zero pattern. Therefore current acceptance is not closed.
- Developer-local clean exact-tree provenance previously recorded for exact `5be123b` remains truthful local provenance for that execution (`scripts/check.sh` exit 0, `git diff --check` exit 0, clean tree), but it does not erase the contradictory hosted cross-evidence or make the current process oracle acceptance-grade.
- Still open before R9-3 completion after H-R9-044: H-R9-041 discriminating exact-wire refusal regression and H-R9-042 deterministic just-before-PTO-deadline negative.
- Candidate A (future/never-sent ACK atomic rejection) remains closed. Candidate B (mixed generic/queue datagram-drop observability) remains closed.
- `READY_LIVE: none`; release item 3 incomplete; release item 4 incomplete; `RELEASE_CANDIDATE=false`; `PRODUCTION_READY=false`; `FREEZE=false`; `RELEASED=false`.

The external coding agent must synchronize to current `main` and continuously execute every dependency-ready slice below: implementation/review -> focused deterministic tests -> commit -> push -> next slice. Reviewer cadence is only a check frequency and is never a work-ticket length or reason to idle.

## Accepted progress that must remain closed

Do not revert these repairs without contradictory repository evidence:

- H-R9-040 source repair at `9a113f2`: `acked_frames` is lifecycle-scoped, survives only while an ACKed frame still has sibling packet copies outstanding, and is pruned when the final copy retires on ACK or loss. Single-copy FrameId reuse and sibling-copy regressions challenge stale-marker poisoning.
- H-R9-041 implementation ordering at `9a113f2`: post-return and `lab_pump` retransmissions build/seal first, call `can_send` on exact encoded wire bytes, then commit `on_retransmit_sent` with that same byte count before socket send. **Only the implementation seam is closed; the discriminating refusal regression remains open.**
- H-R9-043 semantic correction at `5be123b`: do not force exactly one PTO/retransmit and do not merge Session DeliveryAck with Carrier packet recovery. **Its process-oracle acceptance is reopened by H-R9-044.**
- H-R9-039 P4-B server-exit assertion at `021d79d`; H-R9-038 P4 residual diagnostics/oracles at `236e962` + `9091803` + `021d79d`.
- H-R9-037 reverse-order three-way packet bind at `d5d5b23`, independently reviewed at `f2529087`; H-R9-036 reverse-order exact oracle at `855c679`.
- P2 C1-C4 exact closure at `142f0a1` + `8d0ccd5` + `1f8bbb0` + `be03dc3`.
- H-R9-034 settlement-continuation accepted-empty classification at `b801b65`; H-R9-033 initial-loop accepted-empty classification at `7a7c48c`; H-R9-032 packet bind/settlement proof at `ddafbb1`.
- Session DeliveryAck and Carrier packet ACK remain separate evidence domains. Accepted-empty is classification-only and must not become rejection or Session transition.

# READY_LOCAL 1 — H-R9-044 repeated-PTO sibling-ACK oracle + exact-tree gate repair

Repair the exact-current `reliable_udp_post_return_data_loss_recovers_via_pto_retransmit` oracle without changing Recovery/Session architecture.

1. Preserve `>=1` bounded PTO/retransmit before first positive Carrier retirement.
2. Parse **every** `r9_udp_pto_fired`; for each event require `fired_at_us >= deadline_us`. Do not claim all deadlines from `pto_ev[0]` only.
3. Every retransmit must use a fresh packet number distinct from the original and sibling retransmits while preserving stable logical/frame identity (`frame=48`; Session stream 1 / offset 48 / len 16).
4. Exactly one Session delivery transition for that logical range.
5. Carrier rejection remains zero. **Do not require accepted-empty to be zero.** A late/sibling ACK may legitimately classify accepted-empty; if present it must be classification-only and may not fabricate another Session transition, rejection, or retransmission ownership transition.
6. Identify the packet copy that actually receives positive Carrier retirement and bind that packet number to an actual client retransmit and corresponding server `udp_return_packet_ack_sent`. Do not assume `last retransmit == last server ACK == last retirement` unless the fixture deterministically establishes that order.
7. No retransmit may occur after the first positive Carrier retirement that resolves the frame lifecycle.
8. Exactly one final settlement with `remaining_in_flight=0`, after Session confirmation plus Recovery zero.
9. Both processes must succeed.
10. Run focused test first, then the developer-local clean exact-tree gate on the final pushed source/test SHA. Hosted CI remains cross-evidence, not the waiting condition.

If this stronger oracle reveals a genuine state-machine contradiction, immediately switch to the smallest owner repair + discriminating regression. Otherwise keep this test/oracle-only. No decoder/framing change is expected; do not mechanically fuzz solely for this slice.

# READY_LOCAL 2 — H-R9-041 exact-wire refusal regression

Using existing congestion/accounting values, construct a deterministic case where remaining send budget admits retained plaintext length but rejects the larger encoded/sealed retransmission length. Prove exact-wire admission refusal commits no retransmission packet ownership, performs no socket send, emits no `r9_udp_retransmit_sent`, and fabricates no Session delivery state; paired admitted control commits/sends normally. Do not invent a new cwnd/capacity/security policy value.

# READY_LOCAL 3 — H-R9-042 deterministic pre-deadline negative

At Recovery/runtime query level, challenge authoritative PTO at `deadline_us - 1` (or equivalent injected deterministic time): zero PTO/retransmit transition before deadline; expected probe at/after deadline. Wall-clock sleep is not the oracle. Reuse current M2 timing inputs; do not select a new PTO policy value.

# READY_LOCAL 4 — R9-4 ACK-loss + delayed original/reorder

Suppress one legitimate Carrier ACK, force legitimate retransmission, then release delayed original feedback. Require one logical Session delivery, Session dedup authority, fresh packet numbers/nonces, truthful PTO/loss/retransmit evidence and settled Recovery. Late sibling/original feedback may classify accepted-empty/stale after positive copy retirement but must never manufacture another Session transition, rejection or retransmit.

# READY_LOCAL 5 — R9-5 adversarial feedback / every Carrier continuation

Challenge future/never-sent/stale/duplicate/tampered feedback across each exact-current `UdpAcknowledgement::Carrier` continuation: initial receive, settlement and post-return owners. Rejected feedback must be atomic and fail closed without RTT/PTO/loss/cwnd/Session mutation; accepted-empty must remain classification-only. Recheck overlap-copy feedback after H-R9-040/H-R9-044.

# READY_LOCAL 6 — R9-6 ownership/resource boundedness

Every first send and retransmit must consult congestion admission before ownership commit; refusal commits no Recovery/logical state. Keep bounded retransmit plaintext ownership, include H-R9-040 frame-marker lifecycle in the boundedness challenge, and prove deterministic teardown. No capacity-pressure benchmark and no invented capacity/security values.

# READY_LOCAL 7 — R9-7 process/result truth

Independently challenge Data, Carrier ACK, Session DeliveryAck, PTO/retransmit, Recovery ACK/loss, Session delivery, malformed budget, accepted-empty, rejected feedback, residual-domain failure and terminal result as distinct structured evidence. Diagnostics must follow the transition/classification they claim.

# READY_LOCAL 8 — R9-8 warm TCP readiness

Challenge D064 single-active/multi-ready: authenticated resume-bound warm standby carries readiness/control only and no application Data before promotion. Resource/readiness evidence remains separate from packet feedback and Session delivery.

# READY_LOCAL 9 — R9-9 health + promotion

Challenge integrated resolved-UDP-outcome -> health/hysteresis behavior: recoverable reliable-UDP loss remains on UDP rather than spuriously promoting TCP; promotion requires existing readiness/health gates; draining/failed UDP receives no new Data ownership.

# READY_LOCAL 10 — R9-10 uncertain replay + dedup + cleanup

Only genuinely uncertain Session ranges replay on promoted TCP; receiver Session dedup stays exactly-once; TCP receives no duplicate UDP packet-ACK layer; shutdown/error negatives clean resources and emit no false success.

# READY_LOCAL 11 — R9-11 lifecycle/terminal invariants

Close remaining lifecycle/terminal invariants for the materially new cross-process reliable-UDP/failover integration. Terminal classification remains distinct from intermediate diagnostics and cleanup is deterministic.

# READY_LOCAL 12 — R9-12 complete cross-process slice + final exact-tree provenance

After R9-3 through R9-11 close, run the clean exact-tree developer-local gate on the final pushed source/test SHA: `PYTHONDONTWRITEBYTECODE=1 bash scripts/check.sh`, `git diff --check`, clean tree, exact SHA, UTC start/end, OS/arch, stable Rust version and exit codes. Persist provenance without rewriting historical live evidence.

# READY_LOCAL 13 — dedicated independent R9 review

Perform an independent bounded challenge of materially new R9 send/admission/recovery/PTO/demux/Session ACK/Carrier ACK/failover/migration/cleanup/diagnostic behavior. A bounded no-finding note is valid item-4 support when scope, inspected owners, focused commands/tests, exclusions and exact reachable anchor are explicit. Any concrete BLOCKER/HIGH returns immediately to smallest repair -> regression -> exact-tree local gate.

# READY_LOCAL 14 — Q10/Q11/Q12 factual reconciliation

Only after a reachable independent R9 review anchor exists, reconcile `docs/status.md`, `docs/release-security-review-packet.md` and related Q10/Q11/Q12 evidence text with exact reachable implementation/review anchors. Preserve evidence boundaries and do not change RC/freeze/release/production authority.

## Item-4 / core-surface inventory

Earlier reachable independent bounded review already covers reliable-UDP engine basics including future/never-sent ACK guard, `CarrierState`, concurrent Carrier Manager/health/migration-back, FairScheduler/flow accounting, carrier adapters, `SessionRuntime`, observability including mixed queue/generic drop classification, package/reproducibility/operator scripts, dependency/build surface, CLI portability/output, algorithmic boundedness/validators, DeliveryLedger/process codec/datagram/crypto API and wire/parser surfaces.

The materially new cross-process R9 integration remains uncovered until READY_LOCAL 13. H-R9-044 plus H-R9-041/H-R9-042 are concrete dependency-ready work, so repository-wide queue exhaustion is false.

## VPS opportunity

**Not READY.** Standing authorization remains valid, but authoritative classification is `READY_LIVE: none`. H-R9-044 and the remaining R9 work are local correctness/evidence questions. Do not repeat HY2, warm failover, periodic/soak, package lifecycle, migration-back, endpoint/key migration, IPv6, PLPMTUD or Experimental Track evidence merely because the VPS remains rented.

Only create a new READY_LIVE row if later code/instrumentation/hypothesis/path conditions produce a concrete unresolved real-network question that local/loopback evidence cannot answer.

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