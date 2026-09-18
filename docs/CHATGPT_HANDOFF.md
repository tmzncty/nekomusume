# ChatGPT reviewer handoff — H-R9-050 reopens H-R9-041 discriminator; R9-3 remains blocked

## Current repository truth

- Current independent reviewer anchor: exact `08866f30b46f1802efbbc1c700da70c6e46976e4` (`docs(review): reopen H-R9-041 exact-wire discriminator`).
- Latest developer-owned source/test commit reviewed: exact `4a6f52dc0388f901e4da29f7b9e63c19f8cd291d` (`test(carrier): H-R9-041 exact-wire retransmit-admission refusal regression`).
- **H-R9-050 / HIGH evidence-oracle correctness:** `4a6f52d` does not discriminate the executable H-R9-041 exact-wire admission seam. It fills cwnd with synthetic 400-byte Recovery sends and directly checks `!rt.can_send(400)`; it does not encode/seal a retransmission, does not compare retained plaintext length with larger encoded wire length, does not invoke the executable caller/helper, and cannot observe a real socket send or `r9_udp_retransmit_sent`. The test would remain green if `main.rs` regressed from `can_send(re_sealed.len())` back to `can_send(plaintext.len())`.
- The **implementation repair at `9a113f2` remains accepted**: current post-return code builds/seals first, calls `rt.can_send(re_sealed.len() as u64)`, then calls `on_retransmit_sent(..., re_sealed.len() as u64, ...)`, then performs `send_to` and emits `r9_udp_retransmit_sent`; `lab_pump` received the same ordering. Do not rewrite transport architecture to close H-R9-050; this is a missing discriminating regression over an already-correct source seam.
- H-R9-049 remains accepted at `5f2baf2` + `a92d076`: one canonical ACK range may retire multiple Recovery packets and `carrier_retired_fields(acked_packets)` preserves every retired identity without `.last()` collapse.
- H-R9-047/H-R9-048 remain accepted: positive older-sibling ACKs are not accepted-empty and every actual Recovery retirement has positive packet-identity evidence.
- H-R9-046 remains accepted at `d4f1ea3`: no retransmit is emitted after lifecycle-resolving positive Carrier retirement.
- H-R9-045 identity/order work at `0766dd8` + `68e364a` remains accepted: retransmit packet numbers are fresh/pairwise-distinct/stable-frame and final zero-in-flight settlement follows the real Session and Carrier transitions.
- H-R9-043/H-R9-044 semantics remain accepted: repeated PTO before actual lifecycle-resolving Carrier retirement is legal while sibling copies remain outstanding; Session DeliveryAck does not suppress Carrier recovery; accepted-empty sibling/late ACK remains classification-only.
- H-R9-040 lifecycle-scoped `acked_frames` pruning remains accepted at `9a113f2`.
- Still open before R9-3 completion: **H-R9-050 exact-wire caller discriminator**, then **H-R9-042 deterministic just-before-PTO-deadline negative**.
- Developer-local clean exact-tree provenance for exact `4a6f52d` remains a truthful record for that pushed tree: `PYTHONDONTWRITEBYTECODE=1 bash scripts/check.sh` exit 0, `git diff --check` exit 0, clean worktree, 2026-09-18T04:48:41Z → 2026-09-18T04:53:01Z, Linux x86_64, rustc 1.98.0 (88d9e12ae 2026-08-18). It does not by itself cure the oracle gap.
- Candidate A (future/never-sent ACK atomic rejection) remains closed. Candidate B (mixed generic/queue datagram-drop observability) remains closed.
- `READY_LIVE: none`; release item 3 incomplete; release item 4 incomplete; `RELEASE_CANDIDATE=false`; `PRODUCTION_READY=false`; `FREEZE=false`; `RELEASED=false`.

The external coding agent must synchronize to current `main` and continuously execute every dependency-ready slice below: implementation/review -> focused deterministic tests -> commit -> push -> next slice. Reviewer cadence is only a check frequency and is never a work-ticket length or reason to idle.

## Accepted progress that must remain closed

Do not revert these repairs without contradictory repository evidence:

- H-R9-040 source repair at `9a113f2`: `acked_frames` is lifecycle-scoped, survives only while an ACKed frame still has sibling packet copies outstanding, and is pruned when the final copy retires on ACK or loss.
- H-R9-041 **implementation ordering** at `9a113f2`: post-return and `lab_pump` retransmissions build/seal first, call `can_send` on exact encoded wire bytes, then commit `on_retransmit_sent` with that same byte count before socket send. Only the acceptance-grade discriminator is reopened by H-R9-050.
- H-R9-043 semantic correction at `5be123b`; H-R9-044 per-event deadline / accepted-empty correction at `05b1d7b` + `ebfc620`; H-R9-045 identity/order at `0766dd8` + `68e364a`; H-R9-046 post-positive-retirement negative at `d4f1ea3`; H-R9-047 older-sibling classification at `d2478ad`; H-R9-048 source projection at `1c3bf1d`; H-R9-049 discriminator at `a92d076` + `5f2baf2`.
- H-R9-039 P4-B server-exit assertion at `021d79d`; H-R9-038 P4 residual diagnostics/oracles at `236e962` + `9091803` + `021d79d`.
- H-R9-037 reverse-order three-way packet bind at `d5d5b23`, independently reviewed at `f2529087`; H-R9-036 reverse-order exact oracle at `855c679`.
- P2 C1-C4 exact closure at `142f0a1` + `8d0ccd5` + `1f8bbb0` + `be03dc3`.
- H-R9-034 settlement-continuation accepted-empty classification at `b801b65`; H-R9-033 initial-loop accepted-empty classification at `7a7c48c`; H-R9-032 packet bind/settlement proof at `ddafbb1`.
- Session DeliveryAck and Carrier packet ACK remain separate evidence domains. Accepted-empty is classification-only and must not describe a positive Recovery retirement.

# READY_LOCAL 1 — H-R9-050 exact-wire executable-caller discriminator

Keep `9a113f2` source semantics. Add a focused deterministic regression against the real retransmit admission owner (the executable path itself or a minimal helper actually shared by it) such that existing values produce `retained_plaintext_len < remaining_send_budget < encoded_wire_len`. The real path must build/seal first and refuse on the encoded/sealed length. On refusal prove: no `on_retransmit_sent` ownership/state commit, no datagram send, no `r9_udp_retransmit_sent`, no Session delivery transition. A paired control with enough budget must proceed and use the same encoded byte count for admission and Recovery accounting. The regression must deterministically fail if the caller is changed back to plaintext-length admission. Do not invent a new cwnd/capacity/security policy value; do not redesign wire/crypto/recovery architecture.

# READY_LOCAL 2 — H-R9-042 deterministic pre-deadline negative

At Recovery/runtime query level, challenge authoritative PTO at `deadline_us - 1` (or equivalent injected deterministic time): zero PTO/retransmit transition before deadline; expected probe at/after deadline. Wall-clock sleep is not the oracle. Reuse current M2 timing inputs; do not select a new PTO policy value.

# READY_LOCAL 3 — R9-4 ACK-loss + delayed original/reorder

Suppress one legitimate Carrier ACK, force legitimate retransmission, then release delayed original feedback. Require one logical Session delivery, Session dedup authority, fresh packet numbers/nonces, truthful PTO/loss/retransmit evidence and settled Recovery. Late sibling/original feedback may classify accepted-empty/stale after positive copy retirement but must never manufacture another Session transition, rejection or retransmit. Retain H-R9-047/H-R9-048/H-R9-049 exact retirement projection semantics.

# READY_LOCAL 4 — R9-5 adversarial feedback / every Carrier continuation

Challenge future/never-sent/stale/duplicate/tampered and multi-range feedback across each exact-current `UdpAcknowledgement::Carrier` continuation: initial receive, settlement and post-return owners. Rejected feedback must be atomic and fail closed without RTT/PTO/loss/cwnd/Session mutation; accepted-empty remains classification-only. Recheck positive older-sibling and multi-retirement feedback after H-R9-040/H-R9-046/H-R9-047/H-R9-048/H-R9-049.

# READY_LOCAL 5 — R9-6 ownership/resource boundedness

Every first send and retransmit must consult congestion admission before ownership commit; refusal commits no Recovery/logical state. Keep bounded retransmit plaintext ownership, include H-R9-040 marker lifecycle, and prove deterministic teardown. H-R9-050 is a prerequisite for treating exact-wire refusal evidence as closed. No capacity-pressure benchmark and no invented capacity/security values.

# READY_LOCAL 6 — R9-7 process/result truth

Independently challenge Data, Carrier ACK, Session DeliveryAck, PTO/retransmit, Recovery ACK/loss, Session delivery, malformed budget, accepted-empty, rejected feedback, residual-domain failure and terminal result as distinct structured evidence. Diagnostics must follow the actual typed transition/classification they claim.

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

Perform an independent bounded challenge of materially new R9 send/admission/recovery/PTO/demux/Session ACK/Carrier ACK/failover/migration/cleanup/diagnostic behavior. A bounded no-finding note is valid item-4 support when scope, inspected owners, focused commands/tests, exclusions and exact reachable anchor are explicit. Any concrete BLOCKER/HIGH returns immediately to smallest repair -> regression -> exact-tree local gate.

# READY_LOCAL 13 — Q10/Q11/Q12 factual reconciliation

Only after a reachable independent R9 review anchor exists, reconcile `docs/status.md`, `docs/release-security-review-packet.md` and related Q10/Q11/Q12 evidence text with exact reachable implementation/review anchors. Preserve evidence boundaries and do not change RC/freeze/release/production authority.

# READY_LOCAL 14 — repository-wide item-4 inventory refill if reconciliation remains blocked

If Q10/Q11/Q12 cannot yet close because item 4 still lacks independent coverage, perform one repository-wide inventory against the 13 mandated core surfaces and enqueue only genuinely unreviewed implemented surfaces. Do not create filler/checker/schema churn. Queue exhaustion is allowed only if broad inventory finds no concrete defect, no READY_LOCAL review-support, no READY_LIVE question, and every remaining item is truly policy/environment/release-authority gated.

## Item-4 / core-surface inventory

Earlier reachable independent bounded review already covers reliable-UDP engine basics including future/never-sent ACK guard, `CarrierState`, concurrent Carrier Manager/health/migration-back, FairScheduler/flow accounting, carrier adapters, `SessionRuntime`, observability including mixed queue/generic drop classification, package/reproducibility/operator scripts, dependency/build surface, CLI portability/output, algorithmic boundedness/validators, DeliveryLedger/process codec/datagram/crypto API and wire/parser surfaces.

The materially new cross-process R9 integration remains uncovered until READY_LOCAL 12. H-R9-050 and H-R9-042 are concrete dependency-ready local work, so repository-wide queue exhaustion is false.

## VPS opportunity

**Not READY.** Standing authorization remains valid, but authoritative classification is `READY_LIVE: none`. H-R9-050/H-R9-042 and downstream R9 work are local correctness/evidence questions and create no unresolved real-network hypothesis. Do not repeat HY2, warm failover, periodic/soak, package lifecycle, migration-back, endpoint/key migration, IPv6, PLPMTUD or Experimental Track evidence merely because the VPS remains rented.

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
