# ChatGPT reviewer handoff — H-R9-048 multi-retirement Carrier projection HIGH; R9-3 blocked

## Current repository truth

- Latest developer-owned source/test commit reviewed: exact `d2478ada80bd553210aaab631c7392fcdfc2355c` (`fix(cli): H-R9-047 positive sibling ACK not misclassified accepted-empty`).
- Current independent reviewer anchor: exact `0168e5b4c4a02b099792ac36917dfc56fc7ce6af`, [`docs/reviews/reviewer-r9-3-multi-retirement-projection-d2478ad-20260918.md`](reviews/reviewer-r9-3-multi-retirement-projection-d2478ad-20260918.md).
- **H-R9-048 is open HIGH (process/evidence truth, auto-resolvable under current committed semantics):** `d2478ad` correctly stops using mutable latest `post_pn_client` to decide whether an ACK was positive, but the exact-current post-return projection still collapses non-empty `acked_packets: Vec<u64>` to only `acked_packets.last()`. `Recovery::on_ack` can retire multiple outstanding packets in one canonical `AckRanges` application, so a valid ACK can mutate Recovery for multiple packet identities while CLI evidence names only one. Existing process tests validate every emitted retirement but do not prove completeness against the full typed retirement set.
- H-R9-048 is not a newly established Recovery/Session/wire/crypto architecture defect. `Recovery::on_ack` already returns the complete retired-packet set, and `UdpAcknowledgement::Carrier` already carries it to the projection owner. The smallest repair is truthful per-retired-packet positive evidence plus a deterministic multi-retirement regression.
- H-R9-047 remains closed for its original older-sibling/latest-PN misclassification: any non-empty `acked_packets` result is positive, not accepted-empty. H-R9-048 refines the remaining **completeness** obligation when that positive set has cardinality >1.
- `d4f1ea3` remains accepted for post-retirement retransmit: every observed `r9_udp_retransmit_sent` is strictly before the emitted lifecycle-resolving positive Carrier retirement.
- `0766dd8` remains accepted for repeated-PTO packet identity: every retransmit PN is fresh/non-sentinel/pairwise-distinct and carries stable `frame=48`; every **emitted** positive Carrier retirement is value-bound to a real client retransmit and a matching server `udp_return_packet_ack_sent`. H-R9-048 now challenges whether all actual positive retirement identities are emitted.
- `68e364a` remains accepted for the ordering shape: the single `remaining_in_flight=0` settlement is after the exact Session confirmation and the value-identified emitted positive Carrier retirement.
- `9a113f2` remains the H-R9-040 lifecycle-pruning repair plus the H-R9-041 exact encoded-wire retransmit-admission implementation repair. `c1a22e5` added the server Carrier-ACK packet-number diagnostic used by R9-3 three-way evidence.
- H-R9-043 remains accepted: bounded repeated PTO before the first actual lifecycle-resolving positive Carrier retirement is legal while sibling packet copies remain outstanding; Session DeliveryAck is a separate evidence domain and must not suppress Carrier recovery.
- Still open before R9-3 completion after H-R9-048: H-R9-041 discriminating exact-wire refusal regression and H-R9-042 deterministic just-before-PTO-deadline negative.
- Developer-local clean exact-tree provenance for exact `d2478ad`: `PYTHONDONTWRITEBYTECODE=1 bash scripts/check.sh` exit 0, `git diff --check` exit 0, clean worktree at pushed SHA, 2026-09-17T23:49:37Z → 2026-09-17T23:53:55Z, Linux x86_64, rustc 1.98.0 (88d9e12ae 2026-08-18).
- GitHub-hosted Rust CI for exact `d2478ad`, run `35288334930`, completed successfully: `stable checks` / `bash scripts/check.sh` success and `nightly decode fuzz smoke` success. Hosted CI is cross-evidence only and does not discriminate H-R9-048 because the current oracle does not require completeness of the emitted retirement set.
- Open PRs at review time: none.
- Candidate A (future/never-sent ACK atomic rejection) remains closed. Candidate B (mixed generic/queue datagram-drop observability) remains closed.
- `READY_LIVE: none`; release item 3 incomplete; release item 4 incomplete; `RELEASE_CANDIDATE=false`; `PRODUCTION_READY=false`; `FREEZE=false`; `RELEASED=false`.

The external coding agent must synchronize to current `main` and continuously execute every dependency-ready slice below: implementation/review -> focused deterministic tests -> commit -> push -> next slice. Reviewer cadence is only a check frequency and is never a work-ticket length or reason to idle.

## Accepted progress that must remain closed

Do not revert these repairs without contradictory repository evidence:

- H-R9-040 source repair at `9a113f2`: `acked_frames` is lifecycle-scoped, survives only while an ACKed frame still has sibling packet copies outstanding, and is pruned when the final copy retires on ACK or loss. Single-copy FrameId reuse and sibling-copy regressions challenge stale-marker poisoning.
- H-R9-041 implementation ordering at `9a113f2`: post-return and `lab_pump` retransmissions build/seal first, call `can_send` on exact encoded wire bytes, then commit `on_retransmit_sent` with that same byte count before socket send. **Only the implementation seam is closed; the discriminating refusal regression remains open.**
- H-R9-043 semantic correction at `5be123b`: do not force exactly one PTO/retransmit and do not merge Session DeliveryAck with Carrier packet recovery.
- H-R9-044 repair at `05b1d7b` + `ebfc620`: every observed `r9_udp_pto_fired` is checked against its own deadline; typed Carrier rejection remains zero; genuinely accepted-empty sibling/late ACK remains legal and classification-only.
- H-R9-045 packet-identity work at `0766dd8` and ordering work at `68e364a` remain useful and must not be reverted.
- H-R9-046 no-post-positive-retirement retransmit oracle is closed at `d4f1ea3`.
- H-R9-047 older-sibling ACK classification is closed at `d2478ad`: a positive ACK that retires an older retransmit sibling is not mislabeled accepted-empty. H-R9-048 is a separate completeness defect for one ACK retiring multiple packet identities.
- H-R9-039 P4-B server-exit assertion at `021d79d`; H-R9-038 P4 residual diagnostics/oracles at `236e962` + `9091803` + `021d79d`.
- H-R9-037 reverse-order three-way packet bind at `d5d5b23`, independently reviewed at `f2529087`; H-R9-036 reverse-order exact oracle at `855c679`.
- P2 C1-C4 exact closure at `142f0a1` + `8d0ccd5` + `1f8bbb0` + `be03dc3`.
- H-R9-034 settlement-continuation accepted-empty classification at `b801b65`; H-R9-033 initial-loop accepted-empty classification at `7a7c48c`; H-R9-032 packet bind/settlement proof at `ddafbb1`.
- Session DeliveryAck and Carrier packet ACK remain separate evidence domains. Accepted-empty is classification-only and must not describe a positive Recovery retirement.

# READY_LOCAL 1 — H-R9-048 complete multi-retirement Carrier projection

Repair the exact-current post-return `UdpAcknowledgement::Carrier` projection without changing Recovery/Session/wire/crypto architecture:

- `rejected=true` remains typed rejection;
- otherwise, any non-empty `acked_packets` is positive Carrier retirement evidence;
- represent **every** packet identity in `acked_packets`, preferably one `r9_udp_return_packet_ack` diagnostic per retired packet; do not collapse the vector to `.last()` and do not silently omit actual Recovery retirements;
- only `applied=false && rejected=false` may emit accepted-empty.

Add a focused deterministic regression that has at least two simultaneously outstanding packet identities and applies one canonical ACK range/range-set that retires at least two in one owner transition. Require exact set equality between actual retired packet identities and positive diagnostics, zero rejection/accepted-empty for that transition, and exactly-once Session delivery semantics. Preserve the singleton case, H-R9-046 post-positive no-retransmit behavior, and all existing rejected/stale behavior. A helper/unit seam may supplement the process fixture, but the observable projection must be tested. No fuzz run is required solely for this projection/test repair unless decoder/parser/crypto framing changes.

After repair: focused tests -> `PYTHONDONTWRITEBYTECODE=1 bash scripts/check.sh` -> `git diff --check` -> clean exact pushed SHA/provenance -> immediately continue READY_LOCAL 2.

# READY_LOCAL 2 — H-R9-041 exact-wire refusal regression

Using existing congestion/accounting values, construct a deterministic case where remaining send budget admits retained plaintext length but rejects the larger encoded/sealed retransmission length. Prove exact-wire admission refusal commits no retransmission packet ownership, performs no socket send, emits no `r9_udp_retransmit_sent`, and fabricates no Session delivery state; paired admitted control commits/sends normally. Do not invent a new cwnd/capacity/security policy value.

# READY_LOCAL 3 — H-R9-042 deterministic pre-deadline negative

At Recovery/runtime query level, challenge authoritative PTO at `deadline_us - 1` (or equivalent injected deterministic time): zero PTO/retransmit transition before deadline; expected probe at/after deadline. Wall-clock sleep is not the oracle. Reuse current M2 timing inputs; do not select a new PTO policy value.

# READY_LOCAL 4 — R9-4 ACK-loss + delayed original/reorder

Suppress one legitimate Carrier ACK, force legitimate retransmission, then release delayed original feedback. Require one logical Session delivery, Session dedup authority, fresh packet numbers/nonces, truthful PTO/loss/retransmit evidence and settled Recovery. Late sibling/original feedback may classify accepted-empty/stale after positive copy retirement but must never manufacture another Session transition, rejection or retransmit. Explicitly retain H-R9-047/H-R9-048: every ACK that actually retires one or more Recovery packets is positive evidence for every retired packet identity, regardless of which sibling is latest.

# READY_LOCAL 5 — R9-5 adversarial feedback / every Carrier continuation

Challenge future/never-sent/stale/duplicate/tampered and multi-range feedback across each exact-current `UdpAcknowledgement::Carrier` continuation: initial receive, settlement and post-return owners. Rejected feedback must be atomic and fail closed without RTT/PTO/loss/cwnd/Session mutation; accepted-empty must remain classification-only. Recheck overlap-copy feedback after H-R9-040/H-R9-046/H-R9-047/H-R9-048, including positive older-sibling ACKs and one ACK retiring multiple copies; none may be mislabeled or silently omitted.

# READY_LOCAL 6 — R9-6 ownership/resource boundedness

Every first send and retransmit must consult congestion admission before ownership commit; refusal commits no Recovery/logical state. Keep bounded retransmit plaintext ownership, include H-R9-040 frame-marker lifecycle in the boundedness challenge, and prove deterministic teardown. No capacity-pressure benchmark and no invented capacity/security values.

# READY_LOCAL 7 — R9-7 process/result truth

Independently challenge Data, Carrier ACK, Session DeliveryAck, PTO/retransmit, Recovery ACK/loss, Session delivery, malformed budget, accepted-empty, rejected feedback, residual-domain failure and terminal result as distinct structured evidence. Diagnostics must follow the actual typed transition/classification they claim; H-R9-047/H-R9-048 are required premises of this lane.

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

The materially new cross-process R9 integration remains uncovered until READY_LOCAL 13. H-R9-048 plus H-R9-041/H-R9-042 are concrete dependency-ready work, so repository-wide queue exhaustion is false.

## VPS opportunity

**Not READY.** Standing authorization remains valid, but authoritative classification is `READY_LIVE: none`. H-R9-048/H-R9-041/H-R9-042 and downstream R9 work are local correctness/evidence questions. Do not repeat HY2, warm failover, periodic/soak, package lifecycle, migration-back, endpoint/key migration, IPv6, PLPMTUD or Experimental Track evidence merely because the VPS remains rented.

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
