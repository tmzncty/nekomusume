# ChatGPT reviewer handoff — H-R9-054 reopens R9-3 send-reuse discriminator

## Current repository truth

- Latest developer-owned source/test commit: exact `482178973185c081b6b99b417490bf36287b612c`, on top of `9f3786bab18a0e792028d6a4f367be17920d9adb`.
- Reviewer note: `b9ebbdc4a49f09af6c3aacf71d006533b840a1c4` (`docs/reviews/independent-r9-h053-discriminator-20260918.md`).
- **H-R9-053 source repair remains accepted in shape:** `Recovery::watermark_on_reserve` records the committed `largest_sent` preceding each reservation; `abandon_sent` restores that exact committed high-water even if the prior committed packet has already ACKed/retired out of `sent`.
- **New HIGH H-R9-054:** the closure overstates the deterministic regression. Current source rejects `p.number <= largest_sent`, but neither H-R9-053 regression actually attempts packet-number reuse at/below the restored committed high-water. A regression that accidentally weakens send-side monotonicity while preserving ACK behavior can remain green. Because reliable-UDP packet number is also the authenticated record sequence / nonce source, the requested reuse-rejection discriminator is release/security evidence, not cosmetic checker churn.
- Exact `4821789` developer-local gate remains recorded as `PYTHONDONTWRITEBYTECODE=1 bash scripts/check.sh` exit 0, `git diff --check` exit 0, clean worktree, Linux x86_64, rustc 1.98.0, 2026-09-18T09:05:38Z → 09:09:46Z.
- Hosted Rust CI for exact `4821789`: run `35327703324`, SUCCESS. Hosted CI is cross-evidence only.
- Candidate A future/never-sent ACK atomic rejection remains closed at source level; H-R9-054 is specifically the missing send-side packet-number reuse discriminator after aborted reservation rollback. Candidate B mixed queue/generic datagram-drop observability remains closed.
- Open PRs: none at this review.
- `READY_LIVE: none`; release item 3 incomplete; release item 4 incomplete; `RELEASE_CANDIDATE=false`; `PRODUCTION_READY=false`; `FREEZE=false`; `RELEASED=false`.

The external coding agent must synchronize to current `main` and continuously execute every dependency-ready slice below: implementation/review -> focused deterministic tests -> commit -> push -> next slice. Reviewer cadence is only a check frequency and is never a work-ticket length or reason to idle.

## Accepted progress that must remain closed

Do not revert accepted repairs without contradictory current repository evidence:

- H-R9-050 exact-wire executable admission at `c2e263e`: retransmit admission/ownership uses encoded/sealed wire bytes.
- H-R9-051 socket-outcome transaction work at `267604d` + `f954a83`: socket result checks, success-only sent evidence, complete rollback owner, Reno/accounting rollback, partial rollback API removed.
- H-R9-052/H-R9-053 source semantics at `4488d09` + `f954a83` + `9f3786b` + `4821789`: aborted reservation is not ACK-valid; committed high-water survives prior packet retirement; format/clippy gate repaired.
- H-R9-040 lifecycle-scoped `acked_frames` pruning and H-R9-041 exact-wire implementation ordering at `9a113f2`.
- H-R9-043 through H-R9-049 repeated-PTO identity/deadline/accepted-empty/retirement/source-projection work at the reachable anchors already recorded in repository history.
- H-R9-038/H-R9-039 P4 residual/server-exit evidence, H-R9-036/H-R9-037 reverse-order exact oracle, H-R9-032..034 settlement evidence, and P2 C1-C4 remain closed.
- Session DeliveryAck and Carrier packet ACK remain separate evidence domains. Accepted-empty is classification-only and must not describe a positive Recovery retirement.

# READY_LOCAL 1 — HIGH H-R9-054 send-side committed-watermark discriminator

Keep the accepted H-R9-053 implementation shape; do not redesign Recovery, Session, Carrier ACK, crypto framing or wire format.

Required deterministic discriminator on the real `ReliableUdpRuntime` send owner:

1. Commit packet `N`, then ACK/retire `N` completely.
2. Reserve higher packet `M`, then abort `M` as pre-send/socket-failure rollback.
3. Prove late/duplicate ACK(`N`) remains accepted under current historical/accepted-empty semantics.
4. Prove ACK(`M`) is rejected atomically and does not mutate RTT/PTO/loss/retransmit/Reno/Session state.
5. **Attempt a new first-send/retransmit using packet number `N` (and, if the API naturally permits, one lower value); require deterministic rejection with no new Recovery/Reno/frame ownership.** This is the currently missing H-R9-053 discriminator.
6. Paired control: a genuinely fresh packet number `> N` succeeds.
7. Regression must deterministic-red if `Recovery::on_sent` is weakened to permit reuse at/below the restored committed watermark while leaving ACK behavior unchanged.
8. Run focused tests and then the developer-local clean exact-tree gate on the final pushed source/test SHA; persist exact SHA, commands, UTC start/end, exit codes, OS/arch, stable Rust and clean-tree state. No decoder/framing change is required; do not run fuzz mechanically.

After closure, continue immediately to READY_LOCAL 2 without waiting for reviewer cadence.

# READY_LOCAL 2 — H-R9-042 deterministic just-before-PTO-deadline negative

At authoritative Recovery/runtime query level, challenge PTO at `deadline_us - 1` (or equivalent injected deterministic time): zero PTO/retransmit transition before deadline, expected probe at/after deadline. Wall-clock sleep is not the oracle. Reuse current M2 timing inputs; do not select a new PTO policy value. Preserve all accepted repeated-PTO identity/retirement semantics and H-R9-051/H-R9-053 send/commit truth.

# READY_LOCAL 3 — R9-4 ACK-loss + delayed original/reorder

Suppress one legitimate Carrier ACK, force legitimate retransmission, then release delayed original feedback. Require one logical Session delivery, Session dedup authority, fresh packet numbers/nonces, truthful PTO/loss/retransmit evidence and settled Recovery. Late sibling/original feedback may classify accepted-empty/stale after positive copy retirement but must never manufacture another Session transition, rejection or retransmit.

# READY_LOCAL 4 — R9-5 adversarial feedback / every Carrier continuation

Challenge future/never-sent/stale/duplicate/tampered and multi-range feedback across each current `UdpAcknowledgement::Carrier` continuation: initial receive, settlement and post-return owners. Rejected feedback must be atomic/fail closed without RTT/PTO/loss/cwnd/Session mutation. Distinguish genuinely sent-and-retired historical identities from aborted pre-send reservation identities.

# READY_LOCAL 5 — R9-6 ownership/resource boundedness

Every first send and retransmit must consult congestion admission before ownership commit or use an equivalent bounded reservation/commit transaction whose failure is atomic. Socket-send failure must not fabricate Recovery/caller ownership or ACK-valid send history. Keep bounded retransmit plaintext ownership and prove deterministic teardown. No capacity-pressure benchmark and no invented capacity/security values.

# READY_LOCAL 6 — R9-7 process/result truth

Independently challenge Data, Carrier ACK, Session DeliveryAck, PTO/retransmit, Recovery ACK/loss, successful/failed socket send, aborted reservation, Session delivery, malformed budget, accepted-empty, rejected feedback, residual-domain failure and terminal result as distinct structured evidence. Diagnostics must follow the typed transition/classification they claim.

# READY_LOCAL 7 — R9-8 warm TCP readiness

Challenge D064 single-active/multi-ready: authenticated resume-bound warm standby carries readiness/control only and no application Data before promotion. Resource/readiness evidence remains separate from packet feedback and Session delivery.

# READY_LOCAL 8 — R9-9 health + promotion

Challenge integrated resolved-UDP-outcome -> health/hysteresis behavior: recoverable reliable-UDP loss remains on UDP rather than spuriously promoting TCP; promotion requires existing readiness/health gates; draining/failed UDP receives no new Data ownership.

# READY_LOCAL 9 — R9-10 uncertain replay + dedup + cleanup

Only genuinely uncertain Session ranges replay on promoted TCP; receiver Session dedup stays exactly-once; TCP receives no duplicate UDP packet-ACK layer; shutdown/error negatives clean resources and emit no false success.

# READY_LOCAL 10 — R9-11 lifecycle/terminal invariants

Close remaining lifecycle/terminal invariants for the materially new cross-process reliable-UDP/failover integration. Terminal classification remains distinct from intermediate diagnostics and cleanup is deterministic.

# READY_LOCAL 11 — R9-12 complete cross-process slice + final exact-tree provenance

After R9-3 through R9-11 close, run `PYTHONDONTWRITEBYTECODE=1 bash scripts/check.sh`, `git diff --check`, confirm clean tree on the final pushed source/test SHA, and persist exact SHA, UTC start/end, OS/arch, stable Rust and exit codes without rewriting historical live evidence.

# READY_LOCAL 12 — dedicated independent R9 review

Perform an independent bounded challenge of materially new R9 send/admission/reservation/socket-outcome/recovery/PTO/demux/Session ACK/Carrier ACK/failover/migration/cleanup/diagnostic behavior. A bounded no-finding note is valid item-4 support when scope, inspected owners, focused commands/tests, exclusions and exact reachable anchor are explicit. Any concrete BLOCKER/HIGH returns immediately to smallest repair -> regression -> exact-tree gate.

# READY_LOCAL 13 — Q10/Q11/Q12 factual reconciliation

Only after a reachable independent R9 review anchor exists, reconcile `docs/status.md`, `docs/release-security-review-packet.md` and related Q10/Q11/Q12 evidence text with exact reachable implementation/review anchors. Preserve evidence boundaries and do not change RC/freeze/release/production authority.

# READY_LOCAL 14 — repository-wide item-4 inventory refill if reconciliation remains blocked

If Q10/Q11/Q12 cannot yet close because item 4 still lacks independent coverage, perform one repository-wide inventory against the 13 mandated core surfaces and enqueue only genuinely unreviewed implemented surfaces. Do not create filler/checker/schema churn. Queue exhaustion is allowed only if broad inventory finds no concrete defect, no READY_LOCAL review-support, no READY_LIVE question, and every remaining item is truly policy/environment/release-authority gated.

## Item-4 / core-surface inventory

Earlier reachable independent bounded review covers reliable-UDP engine basics, `CarrierState`, concurrent Carrier Manager/health/migration-back, FairScheduler/flow accounting, carrier adapters, `SessionRuntime`, observability including mixed queue/generic drop classification, package/reproducibility/operator scripts, dependency/build surface, CLI portability/output, algorithmic boundedness/validators, DeliveryLedger/process codec/datagram/crypto API and wire/parser surfaces.

The materially new cross-process R9 integration remains under active independent challenge. H-R9-054 is a concrete acceptance-oracle defect, so repository-wide queue exhaustion is false even before downstream R9-4..R9-12 lanes.

## VPS opportunity

**Not READY.** Standing authorization remains valid, but authoritative classification is `READY_LIVE: none`. H-R9-054, H-R9-042 and downstream R9 work are deterministic local correctness/evidence questions and create no unresolved real-network hypothesis. Do not repeat HY2, warm failover, periodic/soak, package lifecycle, migration-back, endpoint/key migration, IPv6, PLPMTUD or Experimental Track evidence merely because the VPS remains rented.

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
