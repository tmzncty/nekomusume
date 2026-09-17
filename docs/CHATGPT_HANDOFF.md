# ChatGPT reviewer handoff — H-R9-046 reopened; R9-3 blocked on H-R9-046/041/042

## Current repository truth

- Latest developer-owned source/test commit: exact `68e364a45466ce958892d536aeea450678919401` (`test(cli): H-R9-045 order/terminal — settlement after Session ACK + lifecycle-resolving Carrier retirement`).
- Independent reviewer checkpoint `c46577e384b8b6244c643349cec36be17599710a` reopens one remaining R9-3 evidence gap as **H-R9-046**: exact `68e364a` proves final settlement occurs after both the exact Session confirmation and the first positive Carrier retirement bound to the retransmit set, but the fixture still does **not** prove that no `r9_udp_retransmit_sent` occurs after that lifecycle-resolving retirement. The previous handoff's stronger closure sentence was therefore false.
- `0766dd8` remains accepted for the packet-identity half: every retransmit packet number is fresh/non-sentinel/pairwise-distinct and carries stable `frame=48`; every positive Carrier retirement is bound by packet-number value to a real client retransmit and a matching server `udp_return_packet_ack_sent`.
- `68e364a` remains accepted for settlement ordering: the single `remaining_in_flight=0` settlement occurs after both the exact Session confirmation and the identified positive Carrier retirement. It does not by itself establish the no-post-retirement-retransmit invariant.
- `9a113f2` remains the H-R9-040 lifecycle-pruning repair plus the H-R9-041 exact encoded-wire retransmit-admission implementation repair. `c1a22e5` added the server Carrier-ACK packet-number diagnostic needed for the R9-3 three-way oracle.
- H-R9-043 remains accepted: bounded repeated PTO before the first lifecycle-resolving positive Carrier retirement is legal while sibling packet copies remain outstanding; Session DeliveryAck is a separate evidence domain and must not suppress Carrier recovery.
- Still open before R9-3 completion: H-R9-046 post-retirement retransmit negative, H-R9-041 discriminating exact-wire refusal regression, and H-R9-042 deterministic just-before-PTO-deadline negative.
- Developer-reported local exact-tree provenance for exact `68e364a`: `PYTHONDONTWRITEBYTECODE=1 bash scripts/check.sh` exit 0, `git diff --check` exit 0, clean worktree at pushed SHA, 2026-09-17T21:48:47Z → 2026-09-17T21:52:58Z, Linux x86_64, rustc 1.98.0 (88d9e12ae 2026-08-18). This is developer-local provenance, not reviewer execution.
- GitHub-hosted Rust CI for exact `68e364a`, run `35278461883`, completed successfully. Hosted CI is cross-evidence only and cannot prove an assertion absent from the test.
- Open PRs at review time: none.
- Candidate A (future/never-sent ACK atomic rejection) remains closed. Candidate B (mixed generic/queue datagram-drop observability) remains closed.
- `READY_LIVE: none`; release item 3 incomplete; release item 4 incomplete; `RELEASE_CANDIDATE=false`; `PRODUCTION_READY=false`; `FREEZE=false`; `RELEASED=false`.

The external coding agent must synchronize to current `main` and continuously execute every dependency-ready slice below: implementation/review -> focused deterministic tests -> commit -> push -> next slice. Reviewer cadence is only a check frequency and is never a work-ticket length or reason to idle.

## Accepted progress that must remain closed

Do not revert these repairs without contradictory repository evidence:

- H-R9-040 source repair at `9a113f2`: `acked_frames` is lifecycle-scoped, survives only while an ACKed frame still has sibling packet copies outstanding, and is pruned when the final copy retires on ACK or loss. Single-copy FrameId reuse and sibling-copy regressions challenge stale-marker poisoning.
- H-R9-041 implementation ordering at `9a113f2`: post-return and `lab_pump` retransmissions build/seal first, call `can_send` on exact encoded wire bytes, then commit `on_retransmit_sent` with that same byte count before socket send. **Only the implementation seam is closed; the discriminating refusal regression remains open.**
- H-R9-043 semantic correction at `5be123b`: do not force exactly one PTO/retransmit and do not merge Session DeliveryAck with Carrier packet recovery.
- H-R9-044 repair at `05b1d7b` + `ebfc620`: every observed `r9_udp_pto_fired` is checked against its own deadline; typed Carrier rejection remains zero; accepted-empty sibling/late ACK remains legal and classification-only.
- H-R9-045 packet-identity half at `0766dd8`: every retransmit PN is fresh/non-sentinel/pairwise-distinct with stable `frame=48`; every positive retirement packet number belongs to the client retransmit set and has a matching server ACK by value.
- H-R9-045 settlement-order half at `68e364a`: the single zero-in-flight settlement is after both the exact Session confirmation and the value-identified positive Carrier retirement. **The no-post-retirement-retransmit negative remains open as H-R9-046.**
- H-R9-039 P4-B server-exit assertion at `021d79d`; H-R9-038 P4 residual diagnostics/oracles at `236e962` + `9091803` + `021d79d`.
- H-R9-037 reverse-order three-way packet bind at `d5d5b23`, independently reviewed at `f2529087`; H-R9-036 reverse-order exact oracle at `855c679`.
- P2 C1-C4 exact closure at `142f0a1` + `8d0ccd5` + `1f8bbb0` + `be03dc3`.
- H-R9-034 settlement-continuation accepted-empty classification at `b801b65`; H-R9-033 initial-loop accepted-empty classification at `7a7c48c`; H-R9-032 packet bind/settlement proof at `ddafbb1`.
- Session DeliveryAck and Carrier packet ACK remain separate evidence domains. Accepted-empty is classification-only and must not become rejection or Session transition.

# READY_LOCAL 1 — H-R9-046 post-retirement retransmit negative

In `reliable_udp_post_return_data_loss_recovers_via_pto_retransmit`, keep the existing packet-number value binding and identify the lifecycle-resolving positive Carrier retirement. Add a discriminating assertion that **every** `r9_udp_retransmit_sent` log position is strictly before that retirement; zero retransmit may occur after the lifecycle-resolving positive ACK. Preserve all existing exact identity/deadline/Session/settlement assertions and the legality of accepted-empty sibling ACKs. Test-only first. If the strengthened oracle exposes runtime contradiction, make the smallest semantics-preserving repair. Then run the developer-local clean exact-tree gate and persist reachable provenance.

# READY_LOCAL 2 — H-R9-041 exact-wire refusal regression

Using existing congestion/accounting values, construct a deterministic case where remaining send budget admits retained plaintext length but rejects the larger encoded/sealed retransmission length. Prove exact-wire admission refusal commits no retransmission packet ownership, performs no socket send, emits no `r9_udp_retransmit_sent`, and fabricates no Session delivery state; paired admitted control commits/sends normally. Do not invent a new cwnd/capacity/security policy value.

# READY_LOCAL 3 — H-R9-042 deterministic pre-deadline negative

At Recovery/runtime query level, challenge authoritative PTO at `deadline_us - 1` (or equivalent injected deterministic time): zero PTO/retransmit transition before deadline; expected probe at/after deadline. Wall-clock sleep is not the oracle. Reuse current M2 timing inputs; do not select a new PTO policy value.

# READY_LOCAL 4 — R9-4 ACK-loss + delayed original/reorder

Suppress one legitimate Carrier ACK, force legitimate retransmission, then release delayed original feedback. Require one logical Session delivery, Session dedup authority, fresh packet numbers/nonces, truthful PTO/loss/retransmit evidence and settled Recovery. Late sibling/original feedback may classify accepted-empty/stale after positive copy retirement but must never manufacture another Session transition, rejection or retransmit.

# READY_LOCAL 5 — R9-5 adversarial feedback / every Carrier continuation

Challenge future/never-sent/stale/duplicate/tampered feedback across each exact-current `UdpAcknowledgement::Carrier` continuation: initial receive, settlement and post-return owners. Rejected feedback must be atomic and fail closed without RTT/PTO/loss/cwnd/Session mutation; accepted-empty must remain classification-only. Recheck overlap-copy feedback after H-R9-040/H-R9-046.

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

# READY_LOCAL 15 — repository-wide item-4 inventory refill if reconciliation remains blocked

If Q10/Q11/Q12 cannot yet close because item 4 still lacks independent coverage, perform one repository-wide inventory against the 13 mandated core surfaces and enqueue only genuinely unreviewed implemented surfaces. Do not create filler/checker/schema churn. Queue exhaustion is allowed only if broad inventory finds no concrete defect, no READY_LOCAL review-support, no READY_LIVE question, and every remaining item is truly policy/environment/release-authority gated.

## Item-4 / core-surface inventory

Earlier reachable independent bounded review already covers reliable-UDP engine basics including future/never-sent ACK guard, `CarrierState`, concurrent Carrier Manager/health/migration-back, FairScheduler/flow accounting, carrier adapters, `SessionRuntime`, observability including mixed queue/generic drop classification, package/reproducibility/operator scripts, dependency/build surface, CLI portability/output, algorithmic boundedness/validators, DeliveryLedger/process codec/datagram/crypto API and wire/parser surfaces.

The materially new cross-process R9 integration remains uncovered until READY_LOCAL 13. H-R9-046 plus H-R9-041/H-R9-042 are concrete dependency-ready work, so repository-wide queue exhaustion is false.

## VPS opportunity

**Not READY.** Standing authorization remains valid, but authoritative classification is `READY_LIVE: none`. H-R9-046/H-R9-041/H-R9-042 and downstream R9 work are local correctness/evidence questions. Do not repeat HY2, warm failover, periodic/soak, package lifecycle, migration-back, endpoint/key migration, IPv6, PLPMTUD or Experimental Track evidence merely because the VPS remains rented.

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
