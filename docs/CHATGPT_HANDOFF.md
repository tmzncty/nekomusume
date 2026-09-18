# ChatGPT reviewer handoff — H-R9-057/H-R9-058 block R9-4; deep queue remains open

## Current repository truth

- Latest developer-owned source/test commit is exact `c585727d5a8b4bc4958b69cdacbfdcf75226b99f` (`style(cli): slice for queue_send — clippy useless_vec`), on top of `cded9a97e1eea54a04ded7fc94b4c5c6f4ab9f`, `14dcc09ffc72f958efa32c00a568a34ee753aa76`, and substantive H-R9-056/R9-4 implementation `458713c478d3d30086e9799aaac568f454b660aa`.
- Latest independent reviewer finding is reachable at `69370caf30504717c4a24f1bd7e58b5737230ae1` (`docs/reviews/independent-r9-h057-h058-r9-4-closure-20260918.md`). It records two current HIGH findings and exact hosted evidence boundaries.
- **H-R9-057 HIGH — R9-4 delay/reorder seam is not one-shot.** Current `--delay-r9-ack-reorder` uses `delayed_post_ack.is_none()` as both “first ACK has never been delayed” and “previous delayed ACK was already released”. After packet 2 takes the delayed value back to `None`, packet 3 is incorrectly treated as another first packet and withheld. Exact `c585727` hosted run `35359163020` proves the failure: PN 4 and PN 5 retire, PN 6 remains in flight, repeated PTO fires, and the positive R9-4 fixture times out instead of settling.
- **The exact-current hosted stable gate is red.** GitHub-hosted Rust CI run `35359163020` for exact `c585727` completed `failure`: `stable checks` failed inside `bash scripts/check.sh`; the nightly decode fuzz smoke passed. The stable job had three failing process tests: the new `reliable_udp_ack_loss_delayed_original_reorder_settles` plus existing `reliable_udp_post_return_carrier_ack_withheld_fails` and `reliable_udp_post_return_session_ack_withheld_fails`. The latter two expose stale “exactly one residual diagnostic” assumptions after the client timeout path was changed to keep recovering until the bounded deadline. Do not restore premature first-timeout failure just to satisfy those old assertions.
- **H-R9-058 HIGH — H-R9-056 accepted-empty witness is too broad.** `recv_udp_delivery_ack` currently accepts any authenticated logical ACK with `confirmed_watermark(stream) >= offset + len` as benign accepted-empty. A watermark is prefix high-water, not proof that `(stream, offset, len)` is an exact admitted/confirmed record. Interior/subrange ACKs can therefore bypass `unexpected_logical_ack`/malformed accounting; `len == 0, offset == 0` can even classify accepted-empty at watermark 0 although `SessionRuntime::delivery_ack` rejects zero-length ACKs. Preserve the exact-duplicate positive behavior but replace the watermark-only witness with bounded exact current-operation ownership/context.
- **H-R9-056 direction is only partially accepted, not closed:** a freshly authenticated exact duplicate Session DeliveryAck for the already-confirmed current bounded record should indeed be classification-only and not malformed. The present implementation proves that positive direction but over-accepts semantically unadmitted ACKs; H-R9-058 must close that safety boundary before H-R9-056/R9-4 can be accepted.
- R9-3, H-R9-054 and H-R9-055 remain independently closed. Do not reopen them absent new contradictory source/test evidence.
- Earlier H-R9-050 exact-wire admission, H-R9-051 socket-send transaction rollback, H-R9-052/H-R9-053 committed ACK-valid watermark restoration, H-R9-043..049 repeated-PTO identity/deadline/retirement/source-projection work, H-R9-040 lifecycle-scoped `acked_frames`, and prior settlement/reverse-order findings remain retained.
- Candidate A (future/never-sent ACK atomic rejection) remains closed. Candidate B (mixed queue/generic datagram-drop observability) remains closed.
- Reviewer-local execution is **not** claimed for this pass. Evidence is exact pushed source/test inspection plus GitHub-hosted run `35359163020`. Do not relabel hosted evidence as reviewer-local or developer-local CI.
- There is no accepted exact-tree developer-local provenance for `c585727` recorded by this reviewer. A green historical gate for an ancestor does not override the current exact-head red hosted gate.
- `READY_LIVE: none`; release item 3 incomplete; release item 4 incomplete; `RELEASE_CANDIDATE=false`; `PRODUCTION_READY=false`; `FREEZE=false`; `RELEASED=false`.

The external coding agent must synchronize to current `main` and continuously execute every dependency-ready slice below: implementation/review -> focused deterministic tests -> commit -> push -> exact-tree local gate/provenance -> next slice. Reviewer cadence is only a check frequency and is never a work-ticket length or reason to idle.

## Accepted progress that must remain closed

Do not revert accepted repairs without contradictory current repository evidence:

- H-R9-050 exact-wire executable admission at `c2e263e`: retransmit admission/ownership uses encoded/sealed wire bytes.
- H-R9-051 socket-outcome transaction work at `267604d` + `f954a83`: socket result checks, success-only sent evidence, complete rollback owner, Reno/accounting rollback, partial rollback API removed.
- H-R9-052/H-R9-053 source semantics at `4488d09` + `f954a83` + `9f3786b` + `4821789`: aborted reservation is not ACK-valid; committed high-water survives prior packet retirement; format/clippy gate repaired.
- H-R9-054 send-side committed-watermark reuse discriminator at `4761827` + `d995afd` + formatting-only `b69c20e`, independently rechecked with no finding at `bb988bc`.
- H-R9-055 executable pre-deadline PTO decision owner at `6221f8d` + `b41e37a`, independently rechecked with no finding at `57bb182`.
- H-R9-040 lifecycle-scoped `acked_frames` pruning and H-R9-041 exact-wire implementation ordering at `9a113f2`.
- H-R9-043 through H-R9-049 repeated-PTO identity/deadline/accepted-empty/retirement/source-projection work at their reachable repository anchors.
- H-R9-038/H-R9-039 P4 residual/server-exit evidence, H-R9-036/H-R9-037 reverse-order exact oracle, H-R9-032..034 settlement evidence, and P2 C1-C4 remain closed.
- Session DeliveryAck and Carrier packet ACK remain separate evidence domains. Session accepted-empty and Carrier accepted-empty are separate classification-only outcomes and neither may fabricate a positive transition in the other domain.

## Closed R9-3 reviewer gate

R9-3 no longer blocks downstream work. The H-R9-055 independent closure at `57bb1826532864310a2170049bb3016c131b53b3` found no contradictory current evidence. Do not spend another slice reopening R9-3 absent a new source/test change or concrete counterexample.

# READY_LOCAL 1 — H-R9-057 one-shot R9-4 Carrier-ACK delay/reorder seam + exact-head gate restoration

Read exact-current server post-return ACK branch, client post-return dual-settlement/PTO loop, and all three currently failing `probe.rs` tests before editing.

Smallest repair contract:

1. Make `--delay-r9-ack-reorder` explicitly one-shot with bounded local seam state (a bool or tiny enum is enough):
   - first post-return Carrier ACK only -> withhold;
   - next post-return packet -> release delayed original **before** current packet ACK;
   - every later post-return packet -> ACK normally; never automatically re-arm the delay.
2. Do not change Recovery/Session/wire architecture to repair a test fixture.
3. Preserve scheduler realism: the client may produce more than one fresh retransmission before it processes delayed feedback. Every actually sent later copy still needs a path to ordinary Carrier ACK/settlement.
4. Reconcile the two existing negative process tests with the newer recovery loop instead of reintroducing first-timeout terminalization:
   - Carrier ACK suppressed: Session may confirm and PTO/retransmit may repeat, but no positive Carrier retirement or settlement; final/terminal residual must have `session_outstanding == 0` and `remaining_in_flight > 0`.
   - Session ACK suppressed: Carrier may settle, but no positive Session confirmation or settlement; final/terminal residual must have `remaining_in_flight == 0` and `session_outstanding > 0`.
   Intermediate residual diagnostics may repeat on bounded receive wakeups; bind the oracle to final residual/terminal truth and positive-transition cardinalities rather than `residual.len() == 1`.
5. The positive R9-4 test must end `in_flight == 0` and cannot accept a timeout as success.

Run focused tests first, then the normal exact-tree local gate on the pushed developer SHA. No wire/parser/crypto-framing change is expected, so do not mechanically add a special fuzz requirement; hosted CI may still run its normal fuzz job.

# READY_LOCAL 2 — H-R9-058 exact bounded witness for duplicate Session ACK accepted-empty

Keep the narrow H-R9-056 positive semantics but remove the watermark-only false witness.

Required invariants:

1. A **fresh authenticated exact duplicate** of a genuinely confirmed current-operation `(session, stream, offset, len)` is classification-only accepted-empty: malformed budget unchanged, no second `delivery_ack`, no second window release / `AckReleased` / `Resumed`, and no Carrier substitution.
2. Same-session authenticated ACKs that are not an exact admitted current-operation record remain fail closed even if their `end <= confirmed_watermark`.
3. `len == 0` remains invalid and must never become accepted-empty merely because the watermark is 0 or higher.
4. Wrong stream/offset/length and otherwise unadmitted logical ACKs remain on the bounded malformed path.
5. Identical authenticated-envelope replay remains crypto replay rejection; a fresh envelope carrying the same exact Session ACK is the H-R9-056 case.
6. Use bounded current-operation ownership/context already available to the caller (or an equivalently bounded exact record set). Do **not** add unbounded ACK history, TTL/LRU/history-size policy, or a new delivery architecture.

Add deterministic positive + negative regressions that would turn red if the current `confirmed_watermark >= end` broad predicate is restored.

# READY_LOCAL 3 — R9-4 full ACK-loss + delayed original/reorder process closure

Only after H-R9-057 and H-R9-058 are green, finish the real cross-process R9-4 discriminator with the actual owners:

- deliver original post-return Data;
- let its first Session DeliveryAck through;
- withhold exactly the first legitimate Carrier ACK until a real PTO/fresh-PN retransmission occurs;
- server Session runtime byte-deduplicates retransmitted stable logical bytes and emits a fresh exact duplicate Session DeliveryAck;
- release delayed original + sibling/current Carrier feedback in deterministic order, with later packets ACKed normally;
- prove exactly-once application delivery / one positive Session confirmation transition for the tested range;
- prove the fresh exact duplicate Session ACK traverses the repaired accepted-empty path without malformed charge;
- prove every positive Carrier retirement names a packet actually retired by Recovery; stale Carrier feedback can only be accepted-empty when current committed semantics allow it;
- prove no false rejection, no retransmit after lifecycle resolution, final `in_flight == 0`, and retained plaintext/frame ownership releases correctly;
- make the test deterministic red if either alternating-ACK suppression or broad/non-exact Session accepted-empty behavior returns.

After a pushed exact-tree local gate/provenance is recorded, continue immediately into R9-5; do not wait for reviewer cadence.

# READY_LOCAL 4 — R9-5 adversarial feedback / every Carrier continuation

Challenge future/never-sent/stale/duplicate/tampered and multi-range feedback across each current `UdpAcknowledgement::Carrier` continuation: initial receive, settlement and post-return owners. Rejected feedback must be atomic/fail closed without RTT/PTO/loss/cwnd/Session mutation. Distinguish genuinely sent-and-retired historical identities from aborted pre-send reservation identities. Include the repaired Session duplicate classification in the cross-domain audit so it cannot be confused with Carrier accepted-empty.

# READY_LOCAL 5 — R9-6 ownership/resource boundedness

Every first send and retransmit must consult congestion admission before ownership commit or use an equivalent bounded reservation/commit transaction whose failure is atomic. Socket-send failure must not fabricate Recovery/caller ownership or ACK-valid send history. Keep bounded retransmit plaintext ownership and prove deterministic teardown. No capacity-pressure benchmark and no invented capacity/security values.

# READY_LOCAL 6 — R9-7 process/result truth

Independently challenge Data, Carrier ACK, Session DeliveryAck, PTO/retransmit, Recovery ACK/loss, successful/failed socket send, aborted reservation, Session delivery, malformed budget, Session accepted-empty/stale, Carrier accepted-empty/stale, rejected feedback, residual-domain failure and terminal result as distinct structured evidence. Diagnostics must follow the typed transition/classification they claim.

# READY_LOCAL 7 — R9-8 warm TCP readiness

Challenge D064 single-active/multi-ready: authenticated resume-bound warm standby carries readiness/control only and no application Data before promotion. Resource/readiness evidence remains separate from packet feedback and Session delivery.

# READY_LOCAL 8 — R9-9 health + promotion

Challenge integrated resolved-UDP-outcome -> health/hysteresis behavior: recoverable reliable-UDP loss remains on UDP rather than spuriously promoting TCP; promotion requires existing readiness/health gates; draining/failed UDP receives no new Data ownership.

# READY_LOCAL 9 — R9-10 uncertain replay + dedup + cleanup

Only genuinely uncertain Session ranges replay on promoted TCP; receiver Session dedup stays exactly-once; TCP receives no duplicate UDP packet-ACK layer; shutdown/error negatives clean resources and emit no false success.

# READY_LOCAL 10 — R9-11 lifecycle/terminal invariants

Close remaining lifecycle/terminal invariants for the materially new cross-process reliable-UDP/failover integration. Terminal classification remains distinct from intermediate diagnostics and cleanup is deterministic.

# READY_LOCAL 11 — R9-12 complete cross-process slice + final exact-tree provenance

After R9-4 through R9-11 close, run `PYTHONDONTWRITEBYTECODE=1 bash scripts/check.sh`, `git diff --check`, confirm clean tree on the final pushed source/test SHA, and persist exact SHA, UTC start/end, OS/arch, stable Rust and exit codes without rewriting historical live evidence.

# READY_LOCAL 12 — dedicated independent R9 review

Perform an independent bounded challenge of materially new R9 send/admission/reservation/socket-outcome/recovery/PTO/demux/Session ACK/Carrier ACK/failover/migration/cleanup/diagnostic behavior. A bounded no-finding note is valid item-4 support when scope, inspected owners, focused commands/tests, exclusions and exact reachable anchor are explicit. Any concrete BLOCKER/HIGH returns immediately to smallest repair -> regression -> exact-tree gate.

# READY_LOCAL 13 — Q10/Q11/Q12 factual reconciliation

Only after a reachable independent R9 review anchor exists, reconcile `docs/status.md`, `docs/release-security-review-packet.md` and related Q10/Q11/Q12 evidence text with exact reachable implementation/review anchors. Preserve evidence boundaries and do not change RC/freeze/release/production authority.

# READY_LOCAL 14 — repository-wide item-4 inventory refill if reconciliation remains blocked

If Q10/Q11/Q12 cannot yet close because item 4 still lacks independent coverage, perform one repository-wide inventory against the 13 mandated core surfaces and enqueue only genuinely unreviewed implemented surfaces. Do not create filler/checker/schema churn. Queue exhaustion is allowed only if broad inventory finds no concrete defect, no READY_LOCAL review-support, no READY_LIVE question, and every remaining item is truly policy/environment/release-authority gated.

## Item-4 / core-surface inventory

Earlier reachable independent bounded review covers reliable-UDP engine basics, `CarrierState`, concurrent Carrier Manager/health/migration-back, FairScheduler/flow accounting, carrier adapters, `SessionRuntime`, observability including mixed queue/generic drop classification, package/reproducibility/operator scripts, dependency/build surface, CLI portability/output, algorithmic boundedness/validators, DeliveryLedger/process codec/datagram/crypto API and wire/parser surfaces.

The materially new cross-process R9 integration remains under active independent challenge. H-R9-054 and H-R9-055/R9-3 remain independently closed; H-R9-057/H-R9-058 are now concrete R9-4 blockers, followed by the R9-4 full closure, R9-5 through R9-12, and dedicated R9 review. Repository-wide queue exhaustion is therefore false.

## VPS opportunity

**Not READY.** Standing authorization remains valid, but authoritative classification is `READY_LIVE: none`. H-R9-057/H-R9-058/R9 work is deterministic local correctness/evidence work and creates no unresolved real-network hypothesis. Do not repeat HY2, warm failover, periodic/soak, package lifecycle, migration-back, endpoint/key migration, IPv6, PLPMTUD or Experimental Track evidence merely because the VPS remains rented.

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
