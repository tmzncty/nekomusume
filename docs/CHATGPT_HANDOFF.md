# ChatGPT reviewer handoff — H-R9-052 closed at 4488d09+f954a83; H-R9-051 discriminator proven; R9-3 blocked on H-R9-042

## Current repository truth

- Latest developer-owned source/test commit: exact `f954a83a5556e77f50871410dc481e5b83d7bbac` (`fix(carrier+cli): H-R9-052 aborted-send ACK watermark + H-R9-051 injected-send discriminator`), on top of `4488d09` (`fix(reliable+carrier): H-R9-052 aborted reservation not ACK-valid; complete rollback owner`).
- **H-R9-052 closed at `4488d09`:** `Recovery::abandon_sent` now restores `largest_sent` to the previous committed maximum when the aborted packet WAS the watermark — an ACK whose `largest` is the aborted number is rejected atomically as never-sent (`Error::InvalidRange`). The earlier ordinary future-ACK guard remains accepted.
- **H-R9-051 discriminator proven at `f954a83`:** `cli_regression_tests::abandoned_retransmit_restores_committed_sent_watermark` proves an ACK of the aborted packet number is rejected atomically after rollback; `socket_send_failure_rolls_back_retransmit_ownership` proves socket `Err` reverses Recovery ownership, Reno charge, and packet->frame map — no positive sent evidence, no in-flight packet, paired control commits exactly once.
- `267604d` socket-result checks retained: both executable retransmit callers check `u.send_to`/`sock.send` result; socket `Err` calls `abandon_retransmit`, skips positive sent evidence, does not advance `post_pn_client`, does not add caller `outstanding`.
- `PathRecovery::abandon_sent` rolls back `packets_sent` counter — socket-failed reservation was never committed.
- Partial-bypass surface removed: `ReliableUdpRuntime::abandon_sent` and `PathRecovery::recovery_mut` deleted; `abandon_retransmit` is the single complete rollback owner (Recovery + Reno + charged + packet_frames).
- GitHub-hosted Rust CI for exact `267604d`, run `35316989919`, completed successfully. Hosted CI is cross-evidence only.
- Developer-local clean exact-tree provenance for exact `f954a83`: `PYTHONDONTWRITEBYTECODE=1 bash scripts/check.sh` exit 0, `git diff --check` exit 0, clean worktree at pushed SHA, 2026-09-18T08:10:18Z → 2026-09-18T08:10:26Z, Linux x86_64, rustc 1.98.0 (88d9e12ae 2026-08-18).
- Still open before R9-3 completion: **H-R9-042 deterministic just-before-PTO-deadline negative**.
- Open PRs at review time: none.
- Candidate A (future/never-sent ACK atomic rejection) remains closed on the ordinary path; H-R9-052 closed the rollback-path variant. Candidate B (mixed generic/queue datagram-drop observability) remains closed.
- `READY_LIVE: none`; release item 3 incomplete; release item 4 incomplete; `RELEASE_CANDIDATE=false`; `PRODUCTION_READY=false`; `FREEZE=false`; `RELEASED=false`.
- Still open before R9-3 completion after H-R9-052/H-R9-051: **H-R9-042 deterministic just-before-PTO-deadline negative**.
- `READY_LIVE: none`; release item 3 incomplete; release item 4 incomplete; `RELEASE_CANDIDATE=false`; `PRODUCTION_READY=false`; `FREEZE=false`; `RELEASED=false`.

The external coding agent must synchronize to current `main` and continuously execute every dependency-ready slice below: implementation/review -> focused deterministic tests -> commit -> push -> next slice. Reviewer cadence is only a check frequency and is never a work-ticket length or reason to idle.

## Accepted progress that must remain closed

Do not revert these repairs without contradictory repository evidence:

- H-R9-050 exact-wire executable-caller discriminator at `c2e263e`: both exact-current retransmit owners share the encoded-wire-byte admission helper. H-R9-052 concerns aborted pre-send reservation state after that admission, not the wire-byte sizing conclusion.
- The useful H-R9-051 source changes in `267604d` are retained: caller socket-result checks, rollback of the new Recovery/Reno packet copy on `Err`, and success-only post-return/lab positive evidence/outstanding insertion. The finding is that the rollback is not yet semantically complete and lacks its required discriminator.
- H-R9-040 lifecycle-scoped `acked_frames` pruning and H-R9-041 exact-wire implementation ordering at `9a113f2` remain accepted.
- H-R9-043 semantic correction at `5be123b`; H-R9-044 per-event deadline / accepted-empty correction at `05b1d7b` + `ebfc620`; H-R9-045 identity/order at `0766dd8` + `68e364a`; H-R9-046 post-positive-retirement negative at `d4f1ea3`; H-R9-047 older-sibling classification at `d2478ad`; H-R9-048 source projection at `1c3bf1d`; H-R9-049 discriminator at `a92d076` + `5f2baf2`.
- H-R9-039 P4-B server-exit assertion at `021d79d`; H-R9-038 P4 residual diagnostics/oracles at `236e962` + `9091803` + `021d79d`.
- H-R9-037 reverse-order three-way packet bind at `d5d5b23`, independently reviewed at `f2529087`; H-R9-036 reverse-order exact oracle at `855c679`.
- P2 C1-C4 exact closure at `142f0a1` + `8d0ccd5` + `1f8bbb0` + `be03dc3`.
- H-R9-034 settlement-continuation accepted-empty classification at `b801b65`; H-R9-033 initial-loop accepted-empty classification at `7a7c48c`; H-R9-032 packet bind/settlement proof at `ddafbb1`.
- Session DeliveryAck and Carrier packet ACK remain separate evidence domains. Accepted-empty is classification-only and must not describe a positive Recovery retirement.

# READY_LOCAL 1 — H-R9-052 + H-R9-051 transactional rollback closure

Read exact-current `crates/neko-reliable/src/lib.rs` (`Recovery::{on_sent,on_ack,abandon_sent}`), `crates/neko-carrier/src/lib.rs` (`PathRecovery::{on_sent,abandon_sent}`, `ReliableUdpRuntime::{on_retransmit_sent,abandon_retransmit,abandon_sent}`), both executable retransmit callers in `crates/neko-cli/src/main.rs`, and the reviewer note at `a4af03b` before editing.

Required invariants:

- a socket `Err` after pre-send admission must not create a Carrier packet that feedback is later allowed to treat as successfully sent;
- future/never-sent ACK rejection must remain atomic on the rollback path and must not mutate RTT/PTO/loss/retransmit/cwnd/Session state;
- a successful socket handoff must still never become sent-but-untracked;
- rollback must be one coherent owner: Recovery, Reno charge, packet->frame/caller outstanding and positive process evidence must not diverge;
- stable retained frame/plaintext survives an aborted copy and remains retryable.

Smallest acceptable repair:

1. Preserve H-R9-050 exact-wire admission and the useful socket-result checks from `267604d`; do not redesign Session, Carrier ACK, Session DeliveryAck, crypto framing or wire format.
2. Replace the misleading aborted-reservation high-water behavior with a typed reservation/commit or equivalent local transaction that preserves a truthful **committed sent high-water**. A socket-aborted packet number must not remain valid as a never-sent ACK `largest`. Do not solve this by requiring every ACK number to remain in `sent`: legitimate late/duplicate feedback for actually sent-and-retired packets must keep current accepted-empty semantics.
3. Do not introduce an unbounded abandoned-PN history or invent TTL/LRU/history-size/capacity policy. Prefer reservation/commit/undo state local to the in-progress send.
4. Add a deterministic Recovery/runtime regression that creates an older real in-flight packet plus a threshold-sized number gap, reserves a fresh retransmit, aborts it as socket failure, then ACKs the aborted PN. Required result: fail-closed invalid/future classification; RTT/PTO/loss/retransmit packet state and Reno/cwnd accounting unchanged; the real old packet remains in flight.
5. Remove the public partial `ReliableUdpRuntime::abandon_sent` / `PathRecovery::recovery_mut` bypass or route it through the same complete transaction owner. There must not be an API path that drops `Recovery.sent` while leaving Reno `charged`/`bytes_in_flight` and packet->frame bookkeeping stale.
6. Complete the original H-R9-051 executable failure-path discriminator for **both** post-return R9 and `lab_pump`: exact-wire admission succeeds; injected send returns `Err`; no `r9_udp_retransmit_sent`, no `retransmit_wire_sent`, no new Recovery/caller outstanding identity, no Session transition; retained frame/plaintext remains retryable. Paired success must commit exactly once and emit exactly one positive ownership/evidence transition.
7. Pin diagnostics truth. If existing `packets_sent` keeps its documented sent-packet meaning, the socket-error rollback must not leave it incremented. Do not silently change its semantic class.
8. Run focused tests, then on the final pushed repair SHA run `PYTHONDONTWRITEBYTECODE=1 bash scripts/check.sh`, `git diff --check`, confirm clean tree, and persist exact SHA, UTC start/end, OS/arch, stable Rust version and exit codes. This finding does not itself require decoder/framing changes; fuzz only if the actual repair touches those surfaces.

After closure, continue immediately to READY_LOCAL 2 without waiting for reviewer cadence.

# READY_LOCAL 2 — CLOSED at f954a83 (H-R9-051/052 discriminator proven)

H-R9-051 socket-send transaction + H-R9-052 aborted-send ACK watermark are both repaired: `Recovery::abandon_sent` restores `largest_sent` on abort; `abandon_retransmit` is the single complete rollback owner; `packets_sent` rolls back; deterministic tests prove aborted-ACK rejection and ownership rollback.

At Recovery/runtime query level, challenge authoritative PTO at `deadline_us - 1` (or equivalent injected deterministic time): zero PTO/retransmit transition before deadline; expected probe at/after deadline. Wall-clock sleep is not the oracle. Reuse current M2 timing inputs; do not select a new PTO policy value.

# READY_LOCAL 3 — R9-4 ACK-loss + delayed original/reorder

Suppress one legitimate Carrier ACK, force legitimate retransmission, then release delayed original feedback. Require one logical Session delivery, Session dedup authority, fresh packet numbers/nonces, truthful PTO/loss/retransmit evidence and settled Recovery. Late sibling/original feedback may classify accepted-empty/stale after positive copy retirement but must never manufacture another Session transition, rejection or retransmit. Retain H-R9-047/H-R9-048/H-R9-049 exact retirement projection semantics and the final H-R9-051/H-R9-052 send/commit truth.

# READY_LOCAL 4 — R9-5 adversarial feedback / every Carrier continuation

Challenge future/never-sent/stale/duplicate/tampered and multi-range feedback across each exact-current `UdpAcknowledgement::Carrier` continuation: initial receive, settlement and post-return owners. Rejected feedback must be atomic and fail closed without RTT/PTO/loss/cwnd/Session mutation; accepted-empty remains classification-only. Explicitly include an aborted pre-send reservation identity after H-R9-052 so Candidate A stays closed under the new transaction model.

# READY_LOCAL 5 — R9-6 ownership/resource boundedness

Every first send and retransmit must consult congestion admission before ownership commit or use an equivalent reservation/commit shape whose failure is atomic. Socket-send failure must not fabricate Recovery/caller ownership or ACK-valid send history after H-R9-051/H-R9-052. Keep bounded retransmit plaintext ownership, include H-R9-040 marker lifecycle, and prove deterministic teardown. No capacity-pressure benchmark and no invented capacity/security values.

# READY_LOCAL 6 — R9-7 process/result truth

Independently challenge Data, Carrier ACK, Session DeliveryAck, PTO/retransmit, Recovery ACK/loss, successful/failed socket send, aborted reservation, Session delivery, malformed budget, accepted-empty, rejected feedback, residual-domain failure and terminal result as distinct structured evidence. Diagnostics must follow the actual typed transition/classification they claim.

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

Perform an independent bounded challenge of materially new R9 send/admission/reservation/socket-outcome/recovery/PTO/demux/Session ACK/Carrier ACK/failover/migration/cleanup/diagnostic behavior. A bounded no-finding note is valid item-4 support when scope, inspected owners, focused commands/tests, exclusions and exact reachable anchor are explicit. Any concrete BLOCKER/HIGH returns immediately to smallest repair -> regression -> exact-tree local gate.

# READY_LOCAL 13 — Q10/Q11/Q12 factual reconciliation

Only after a reachable independent R9 review anchor exists, reconcile `docs/status.md`, `docs/release-security-review-packet.md` and related Q10/Q11/Q12 evidence text with exact reachable implementation/review anchors. Preserve evidence boundaries and do not change RC/freeze/release/production authority.

# READY_LOCAL 14 — repository-wide item-4 inventory refill if reconciliation remains blocked

If Q10/Q11/Q12 cannot yet close because item 4 still lacks independent coverage, perform one repository-wide inventory against the 13 mandated core surfaces and enqueue only genuinely unreviewed implemented surfaces. Do not create filler/checker/schema churn. Queue exhaustion is allowed only if broad inventory finds no concrete defect, no READY_LOCAL review-support, no READY_LIVE question, and every remaining item is truly policy/environment/release-authority gated.

## Item-4 / core-surface inventory

Earlier reachable independent bounded review covers reliable-UDP engine basics, `CarrierState`, concurrent Carrier Manager/health/migration-back, FairScheduler/flow accounting, carrier adapters, `SessionRuntime`, observability including mixed queue/generic drop classification, package/reproducibility/operator scripts, dependency/build surface, CLI portability/output, algorithmic boundedness/validators, DeliveryLedger/process codec/datagram/crypto API and wire/parser surfaces.

The ordinary future/never-sent ACK guard was previously covered, but H-R9-052 creates a materially new aborted-reservation path that can bypass its semantic premise. That path is now dependency-ready local repair/review work; repository-wide queue exhaustion is false. The materially new cross-process R9 integration remains uncovered until READY_LOCAL 12.

## VPS opportunity

**Not READY.** Standing authorization remains valid, but authoritative classification is `READY_LIVE: none`. H-R9-052/H-R9-051/H-R9-042 and downstream R9 work are local correctness/evidence questions and create no unresolved real-network hypothesis. Do not repeat HY2, warm failover, periodic/soak, package lifecycle, migration-back, endpoint/key migration, IPv6, PLPMTUD or Experimental Track evidence merely because the VPS remains rented.

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
