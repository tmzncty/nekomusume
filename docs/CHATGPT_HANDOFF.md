# ChatGPT reviewer handoff — H-R9-051 socket-send commit divergence open; R9-3 blocked

## Current repository truth

- Latest developer-owned source/test commit: exact `c2e263e90ab6ebabfcd829e3012a6f6ef98c26ff` (`fix(cli): H-R9-050 executable retransmit-admission discriminator — admit_retransmit helper on exact wire bytes`).
- Latest independent reviewer finding: exact `1a17cbb098dd6d2a4cfb674b8e77813d6470ef28`, [`docs/reviews/independent-r9-retransmit-send-commit-c2e263e-20260918.md`](reviews/independent-r9-retransmit-send-commit-c2e263e-20260918.md), opens **H-R9-051 HIGH**.
- H-R9-050 remains closed at `c2e263e`: the shared `admit_retransmit(rt, pn, now, wire, frame)` gate makes congestion admission and Recovery ownership accounting use the exact encoded/sealed wire byte count. The focused unit discriminator proves a budget that admits the smaller retained/plaintext-sized control but refuses the larger encoded wire and leaves in-flight ownership unchanged on refusal.
- **H-R9-051 is distinct and does not reopen H-R9-050.** After `admit_retransmit` has committed Recovery ownership, the post-return R9 caller currently discards the result of `u.send_to(&re_sealed, target)`, then unconditionally emits `r9_udp_retransmit_sent` and advances `post_pn_client`. `lab_pump` increments `retransmit_wire_sent` only on `sock.send(&sealed).is_ok()`, but unconditionally inserts the packet into caller `outstanding` after Recovery has already committed it. Therefore an ordinary local socket `Err` can leave a never-accepted datagram represented as in-flight/outstanding and, on the post-return path, can manufacture positive sent evidence.
- This is not the intentional `--drop-r9-data` experiment seam: that seam deliberately commits a packet and suppresses its wire send to model controlled loss. Ordinary socket failure must not silently acquire those semantics.
- Still open before R9-3 completion after H-R9-051: **H-R9-042 deterministic just-before-PTO-deadline negative**.
- Developer-local clean exact-tree provenance for exact `c2e263e` remains developer-reported first-class evidence: `PYTHONDONTWRITEBYTECODE=1 bash scripts/check.sh` exit 0, `git diff --check` exit 0, clean worktree at pushed SHA, 2026-09-18T05:51:46Z → 2026-09-18T05:56:09Z, Linux x86_64, rustc 1.98.0 (88d9e12ae 2026-08-18).
- GitHub-hosted Rust CI for exact `c2e263e`, run `35312324850`, completed successfully. Hosted CI is cross-evidence only; its green result does not challenge the missing injected socket-send failure path.
- Open PRs at review time: none.
- Candidate A (future/never-sent ACK atomic rejection) remains closed. Candidate B (mixed generic/queue datagram-drop observability) remains closed.
- `READY_LIVE: none`; release item 3 incomplete; release item 4 incomplete; `RELEASE_CANDIDATE=false`; `PRODUCTION_READY=false`; `FREEZE=false`; `RELEASED=false`.

The external coding agent must synchronize to current `main` and continuously execute every dependency-ready slice below: implementation/review -> focused deterministic tests -> commit -> push -> next slice. Reviewer cadence is only a check frequency and is never a work-ticket length or reason to idle.

## Accepted progress that must remain closed

Do not revert these repairs without contradictory repository evidence:

- H-R9-050 exact-wire executable-caller discriminator at `c2e263e`: both exact-current retransmit owners share the encoded-wire-byte admission helper. H-R9-051 concerns the following socket-send/commit transaction, not the wire-byte sizing conclusion.
- H-R9-040 lifecycle-scoped `acked_frames` pruning and H-R9-041 exact-wire implementation ordering at `9a113f2` remain accepted.
- H-R9-043 semantic correction at `5be123b`; H-R9-044 per-event deadline / accepted-empty correction at `05b1d7b` + `ebfc620`; H-R9-045 identity/order at `0766dd8` + `68e364a`; H-R9-046 post-positive-retirement negative at `d4f1ea3`; H-R9-047 older-sibling classification at `d2478ad`; H-R9-048 source projection at `1c3bf1d`; H-R9-049 discriminator at `a92d076` + `5f2baf2`.
- H-R9-039 P4-B server-exit assertion at `021d79d`; H-R9-038 P4 residual diagnostics/oracles at `236e962` + `9091803` + `021d79d`.
- H-R9-037 reverse-order three-way packet bind at `d5d5b23`, independently reviewed at `f2529087`; H-R9-036 reverse-order exact oracle at `855c679`.
- P2 C1-C4 exact closure at `142f0a1` + `8d0ccd5` + `1f8bbb0` + `be03dc3`.
- H-R9-034 settlement-continuation accepted-empty classification at `b801b65`; H-R9-033 initial-loop accepted-empty classification at `7a7c48c`; H-R9-032 packet bind/settlement proof at `ddafbb1`.
- Session DeliveryAck and Carrier packet ACK remain separate evidence domains. Accepted-empty is classification-only and must not describe a positive Recovery retirement.

# READY_LOCAL 1 — H-R9-051 retransmit socket-send / Recovery transaction repair

Read exact-current `crates/neko-cli/src/main.rs`, the `admit_retransmit` helper, both executable retransmit callers, `ReliableUdpRuntime::{can_send,on_retransmit_sent}`, and the reviewer note at `1a17cbb` before editing.

Required invariant: positive retransmit-send evidence/counters and caller outstanding identity must describe a datagram whose local socket send succeeded. A local socket `Err` must not leave a new packet copy charged/tracked as in-flight or caller-outstanding, and must not consume the stable retained frame/plaintext needed for a later legitimate retry.

Smallest acceptable repair:

1. Preserve H-R9-050 exact-wire admission semantics; do not redesign Session, Carrier ACK, Session DeliveryAck, crypto framing or wire format.
2. Make both post-return R9 and `lab_pump` use an executable retransmit-send transaction that includes the socket outcome, not merely congestion admission.
3. Preserve the opposite safety edge too: a datagram successfully handed to the socket must never become sent-but-untracked. A pre-send Recovery reservation is acceptable only if socket `Err` rolls back **only the newly reserved packet copy**: Recovery/Reno charge, packet->frame map and caller outstanding identity. Stable frame/plaintext ownership must remain available for retry. An equivalent typed reservation/commit shape is acceptable if it has less state/API and remains fail-closed.
4. Emit `r9_udp_retransmit_sent`, increment `retransmit_wire_sent`, advance `post_pn_client`, and add caller `outstanding` only for the successful-send outcome.
5. Add a deterministic injected-send failure regression at the real executable seam: exact-wire admission succeeds, the injected send returns `Err`, and there is no positive sent event/counter, no new in-flight/outstanding packet, no Session transition, and the stable frame remains retryable. Add a paired successful-send control proving exactly one ownership/evidence transition.
6. Apply the regression/repair to both exact-current executable owners. Do not substitute the intentional `--drop-r9-data` loss seam for this ordinary socket-error test.
7. Run focused tests, then the normal clean exact-tree local gate on the final pushed repair SHA. This finding does not itself touch decoder/framing, so do not mechanically fuzz unless the actual repair changes those surfaces.

After closure, continue immediately to READY_LOCAL 2 without waiting for reviewer cadence.

# READY_LOCAL 2 — H-R9-042 deterministic pre-deadline negative

At Recovery/runtime query level, challenge authoritative PTO at `deadline_us - 1` (or equivalent injected deterministic time): zero PTO/retransmit transition before deadline; expected probe at/after deadline. Wall-clock sleep is not the oracle. Reuse current M2 timing inputs; do not select a new PTO policy value.

# READY_LOCAL 3 — R9-4 ACK-loss + delayed original/reorder

Suppress one legitimate Carrier ACK, force legitimate retransmission, then release delayed original feedback. Require one logical Session delivery, Session dedup authority, fresh packet numbers/nonces, truthful PTO/loss/retransmit evidence and settled Recovery. Late sibling/original feedback may classify accepted-empty/stale after positive copy retirement but must never manufacture another Session transition, rejection or retransmit. Retain H-R9-047/H-R9-048/H-R9-049 exact retirement projection semantics and H-R9-051 send/commit truth.

# READY_LOCAL 4 — R9-5 adversarial feedback / every Carrier continuation

Challenge future/never-sent/stale/duplicate/tampered and multi-range feedback across each exact-current `UdpAcknowledgement::Carrier` continuation: initial receive, settlement and post-return owners. Rejected feedback must be atomic and fail closed without RTT/PTO/loss/cwnd/Session mutation; accepted-empty remains classification-only. Recheck positive older-sibling and multi-retirement feedback after H-R9-040/H-R9-046/H-R9-047/H-R9-048/H-R9-049.

# READY_LOCAL 5 — R9-6 ownership/resource boundedness

Every first send and retransmit must consult congestion admission before ownership commit or use an equivalent reservation/commit shape whose failure is atomic. Socket-send failure must not fabricate Recovery/caller ownership after H-R9-051. Keep bounded retransmit plaintext ownership, include H-R9-040 marker lifecycle, and prove deterministic teardown. No capacity-pressure benchmark and no invented capacity/security values.

# READY_LOCAL 6 — R9-7 process/result truth

Independently challenge Data, Carrier ACK, Session DeliveryAck, PTO/retransmit, Recovery ACK/loss, successful/failed socket send, Session delivery, malformed budget, accepted-empty, rejected feedback, residual-domain failure and terminal result as distinct structured evidence. Diagnostics must follow the actual typed transition/classification they claim.

# READY_LOCAL 7 — R9-8 warm TCP readiness

Challenge D064 single-active/multi-ready: authenticated resume-bound warm standby carries readiness/control only and no application Data before promotion. Resource/readiness evidence remains separate from packet feedback and Session delivery.

# READY_LOCAL 8 — R9-9 health + promotion

Challenge integrated resolved-UDP-outcome -> health/hysteresis behavior: recoverable reliable-UDP loss remains on UDP rather than spuriously promoting TCP; promotion requires existing readiness/health gates; draining/failed UDP receives no new Data ownership.

# READY_LOCAL 9 — R9-10 uncertain replay + dedup + cleanup

Only genuinely uncertain Session ranges replay on promoted TCP; receiver Session dedup stays exactly-once; TCP receives no duplicate UDP packet-ACK layer; shutdown/error negatives clean resources and emit no false success.

# READY_LOCAL 10 — R9-11 lifecycle/terminal invariants

Close remaining lifecycle/terminal invariants for the materially new cross-process reliable-UDP/failover integration. Terminal classification remains distinct from intermediate diagnostics and cleanup is deterministic.

# READY_LOCAL 11 — R9-12 complete cross-process slice + final exact-tree provenance

After R9-3 through R9-11 close, run the clean exact-tree developer-local gate on the final pushed source/test SHA: `PYTHONDONTWRITEBYTECODE=1 bash scripts/check.sh`, `git diff --check`, clean tree, exact SHA, UTC start/end, OS/arch, stable Rust version and exit codes. Persist provenance without rewriting historical live evidence.

# READY_LOCAL 12 — dedicated independent R9 review

Perform an independent bounded challenge of materially new R9 send/admission/socket-outcome/recovery/PTO/demux/Session ACK/Carrier ACK/failover/migration/cleanup/diagnostic behavior. A bounded no-finding note is valid item-4 support when scope, inspected owners, focused commands/tests, exclusions and exact reachable anchor are explicit. Any concrete BLOCKER/HIGH returns immediately to smallest repair -> regression -> exact-tree local gate.

# READY_LOCAL 13 — Q10/Q11/Q12 factual reconciliation

Only after a reachable independent R9 review anchor exists, reconcile `docs/status.md`, `docs/release-security-review-packet.md` and related Q10/Q11/Q12 evidence text with exact reachable implementation/review anchors. Preserve evidence boundaries and do not change RC/freeze/release/production authority.

# READY_LOCAL 14 — repository-wide item-4 inventory refill if reconciliation remains blocked

If Q10/Q11/Q12 cannot yet close because item 4 still lacks independent coverage, perform one repository-wide inventory against the 13 mandated core surfaces and enqueue only genuinely unreviewed implemented surfaces. Do not create filler/checker/schema churn. Queue exhaustion is allowed only if broad inventory finds no concrete defect, no READY_LOCAL review-support, no READY_LIVE question, and every remaining item is truly policy/environment/release-authority gated.

## Item-4 / core-surface inventory

Earlier reachable independent bounded review already covers reliable-UDP engine basics including future/never-sent ACK guard, `CarrierState`, concurrent Carrier Manager/health/migration-back, FairScheduler/flow accounting, carrier adapters, `SessionRuntime`, observability including mixed queue/generic drop classification, package/reproducibility/operator scripts, dependency/build surface, CLI portability/output, algorithmic boundedness/validators, DeliveryLedger/process codec/datagram/crypto API and wire/parser surfaces.

The materially new cross-process R9 integration remains uncovered until READY_LOCAL 12. H-R9-051 and H-R9-042 are concrete dependency-ready local work, so repository-wide queue exhaustion is false.

## VPS opportunity

**Not READY.** Standing authorization remains valid, but authoritative classification is `READY_LIVE: none`. H-R9-051/H-R9-042 and downstream R9 work are local correctness/evidence questions and create no unresolved real-network hypothesis. Do not repeat HY2, warm failover, periodic/soak, package lifecycle, migration-back, endpoint/key migration, IPv6, PLPMTUD or Experimental Track evidence merely because the VPS remains rented.

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
