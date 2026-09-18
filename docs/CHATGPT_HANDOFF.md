# ChatGPT reviewer handoff — H-R9-059/H-R9-058 block R9-4; deep queue remains open

## Current repository truth

- Latest developer-owned source/test commit is exact `1bbf7e8939ba2c6a73015a6a57b0c63ac6706018` (`style(cli): underscore unused recv result — clippy`), directly above substantive repair `08cbdaf63fa24152eb3387cb9eae98a34e9d21c3` (`fix(session+cli): H-R9-057 one-shot delay/reorder seam; H-R9-058 bounded-exact confirmed range`).
- Latest independent reviewer finding is reachable at `4a85b360d9ba4a1fafee3d38e3e231e777520b50`, `docs/reviews/independent-r9-h059-bounded-session-ack-witness-20260919.md`.
- **H-R9-057 source defect is closed.** `delay_reorder_done` makes `--delay-r9-ack-reorder` genuinely one-shot; exact-head hosted run `35365100701` now passes `reliable_udp_ack_loss_delayed_original_reorder_settles`. Do not re-open the alternating-ACK defect absent contradictory current evidence.
- **Exact-head stable gate is still red.** Hosted Rust CI run `35365100701` for exact `1bbf7e8` completed `failure`: nightly decode fuzz smoke succeeded, while `stable checks` failed in `bash scripts/check.sh`. The only observed process failures are the two older post-return negative tests that still demand `residual.len() == 1`; current bounded recovery correctly emits repeated residual diagnostics while continuing PTO/recovery until the operation deadline. Repair the oracles, not the runtime semantics.
- **H-R9-058 remains open.** The old broad `confirmed_watermark >= end` predicate was removed, but current tests only prove a positive exact-duplicate case. There is still no discriminator proving authenticated interior/subrange and zero-length feedback below the watermark stay malformed if the broad predicate is restored.
- **H-R9-059 HIGH — the H-R9-058 witness is retained Session-lifetime ACK history.** `SessionRuntime.confirmed_ranges: BTreeMap<StreamId, BTreeSet<(u64,u64)>>` inserts one historical tuple for every positive `delivery_ack` and clears only with the whole runtime. Its B-tree metadata is not charged to queue/window/total-byte accounting and grows with historical confirmations rather than current operation ownership. This violates the prior no-unbounded-history repair contract and the repository's per-connection/global resource-bound security requirement. Do not patch it with a new TTL/LRU/history-size/capacity number.
- H-R9-056 semantic direction remains accepted but not yet closed: a **fresh authenticated exact duplicate** Session DeliveryAck for an exact already-confirmed current bounded record is classification-only accepted-empty; identical authenticated-envelope replay remains crypto replay rejection; semantically unadmitted feedback remains malformed/fail-closed.
- R9-3, H-R9-054 and H-R9-055 remain independently closed. Earlier H-R9-050 exact-wire admission, H-R9-051 socket-send transaction rollback, H-R9-052/H-R9-053 committed ACK-valid watermark restoration, H-R9-043..049 repeated-PTO identity/deadline/retirement/source-projection work, H-R9-040 lifecycle-scoped `acked_frames`, and prior settlement/reverse-order findings remain retained.
- Candidate A (future/never-sent ACK atomic rejection) remains closed. Candidate B (mixed queue/generic datagram-drop observability) remains closed.
- Reviewer-local execution is **not** claimed for this pass. Exact pushed source/test inspection plus GitHub-hosted run `35365100701` are cross-evidence only. There is no accepted exact-tree developer-local provenance for `1bbf7e8` recorded by this reviewer.
- `READY_LIVE: none`; release item 3 incomplete; release item 4 incomplete; `RELEASE_CANDIDATE=false`; `PRODUCTION_READY=false`; `FREEZE=false`; `RELEASED=false`.

The external coding agent must synchronize to current `main` and continuously execute every dependency-ready slice below: implementation/review -> focused deterministic tests -> commit -> push -> exact-tree local gate/provenance -> next slice. Reviewer cadence is only a check frequency and is never a work-ticket length or reason to idle.

## Accepted progress that must remain closed

Do not revert accepted repairs without contradictory current repository evidence:

- H-R9-057 one-shot ACK-delay/reorder server seam at `08cbdaf6`; the exact-head positive R9-4 process discriminator is hosted-green.
- H-R9-050 exact-wire executable retransmit admission at `c2e263e`.
- H-R9-051 socket-outcome transaction work at `267604d` + `f954a83`.
- H-R9-052/H-R9-053 committed ACK-valid watermark work at `4488d09` + `f954a83` + `9f3786b` + `4821789`.
- H-R9-054 send-side committed-watermark reuse discriminator at `4761827` + `d995afd`, independently rechecked at `bb988bc`.
- H-R9-055 executable pre-deadline PTO owner at `6221f8d` + `b41e37a`, independently rechecked at `57bb182`.
- H-R9-040 lifecycle-scoped `acked_frames`; H-R9-043..H-R9-049 repeated-PTO identity/deadline/accepted-empty/retirement/source-projection; H-R9-038/H-R9-039 residual/server-exit; H-R9-036/H-R9-037 reverse-order exact oracle; H-R9-032..034 settlement; P2 C1-C4.
- Session DeliveryAck and Carrier packet ACK are separate evidence domains. Session accepted-empty and Carrier accepted-empty are separate classification-only outcomes and cannot fabricate a positive transition in the other domain.

# READY_LOCAL 1 — H-R9-059 + H-R9-058: bounded exact current-operation Session ACK witness

Read exact-current `SessionRuntime`, `recv_udp_delivery_ack`, the current R9 client operation owner and the review note at `4a85b36` before editing.

Smallest repair contract:

1. Remove Session-lifetime `confirmed_ranges` / general historical exact-range retention. Do not invent TTL, LRU, history-size, capacity or security-policy values.
2. Put the duplicate witness at the current R9 operation owner, where admitted logical records are already finitely bounded. A minimal shape is a caller-owned exact `(session, stream, offset, len)` collection populated only when the first genuine Session ACK exact-matches an admitted outstanding record. Cardinality must derive from the operation's existing bounded admitted records, not Session lifetime.
3. Fresh authenticated exact duplicate of one of those current-operation confirmed records -> accepted-empty classification only: malformed budget unchanged, no second `delivery_ack`, no second window release/`AckReleased`/`Resumed`, no Carrier substitution.
4. Same-session authenticated feedback not naming an exact admitted current-operation record -> fail closed/malformed even when its end is below the confirmed watermark. `len == 0` must never become accepted-empty.
5. Identical authenticated-envelope replay stays crypto replay rejection; the accepted-empty case is a newly sealed envelope with identical Session-ACK semantics.
6. Operation-owned witness clears with operation/lifecycle completion and must not become general Session history.

Required regressions must be red if broad watermark acceptance returns:

- positive exact freshly sealed duplicate;
- fresh authenticated interior/subrange with `end <= confirmed_watermark` -> malformed, not accepted-empty;
- fresh authenticated `len == 0` -> malformed, not accepted-empty;
- retain wrong stream/offset/length and identical-envelope replay negatives.

Do not use an arbitrary direct `SessionRuntime::delivery_ack` call alone as proof that a range was an admitted operation record unless the same test binds it to real current-operation ownership.

# READY_LOCAL 2 — exact-head stable-gate restoration for post-return negative oracles

Do not reintroduce first-timeout terminalization.

- Carrier ACK suppressed: Session may confirm and PTO/retransmit may repeat. Bind the oracle to the final/terminal residual: `session_outstanding == 0`, `remaining_in_flight > 0`; zero positive Carrier settlement/retirement for the suppressed domain; nonzero terminal result; no settled marker. Intermediate residual count may be greater than one.
- Session ACK suppressed: Carrier may settle. Bind the oracle to final/terminal residual: `remaining_in_flight == 0`, `session_outstanding > 0`; zero positive Session confirmation for the suppressed domain; nonzero terminal result; no settled marker. Intermediate residual count may be greater than one.
- Preserve existing exact identity/cardinality checks for actual positive transitions and server fault-injection truth.

Run both focused process negatives first. Then continue directly into full R9-4 rather than waiting for reviewer cadence.

# READY_LOCAL 3 — R9-4 full ACK-loss + delayed original/reorder closure

Using real cross-process owners:

- deliver original post-return Data;
- allow its first Session DeliveryAck;
- withhold exactly its first legitimate Carrier ACK until a real PTO/fresh-PN retransmission;
- server Session runtime byte-deduplicates stable logical bytes and emits a freshly sealed exact duplicate Session DeliveryAck;
- release delayed original + sibling/current Carrier feedback deterministically; all later packets ACK normally;
- prove exactly-once application delivery and exactly one positive Session confirmation transition for the range;
- prove the fresh exact duplicate Session ACK traverses the repaired current-operation accepted-empty path with zero malformed charge;
- prove every positive Carrier retirement names a packet actually retired by Recovery; stale feedback only accepted-empty where current committed semantics allow it;
- prove no retransmit after lifecycle resolution, final `in_flight == 0`, and retained plaintext/frame ownership releases correctly;
- test must deterministic-red if alternating ACK suppression or broad/non-exact Session accepted-empty behavior returns.

After a pushed exact-tree local gate/provenance, continue immediately to R9-5.

# READY_LOCAL 4 — R9-5 adversarial feedback / every Carrier continuation

Challenge future/never-sent/stale/duplicate/tampered and multi-range feedback across every current `UdpAcknowledgement::Carrier` continuation: initial receive, settlement and post-return owners. Rejected feedback is atomic/fail-closed without RTT/PTO/loss/cwnd/Session mutation. Distinguish sent-and-retired historical identities from aborted pre-send reservation identities. Audit Session duplicate classification separately so it cannot substitute for Carrier accepted-empty.

# READY_LOCAL 5 — R9-6 ownership/resource boundedness

Every first send/retransmit must consult congestion admission before ownership commit or use equivalent bounded reservation/commit with atomic failure. Socket-send failure cannot fabricate Recovery/caller ownership or ACK-valid history. Retained retransmit plaintext ownership must be bounded and deterministically released. Include the H-R9-059 lesson: historical classification metadata must remain accounted/bounded by existing owners. No capacity-pressure benchmark and no invented policy values.

# READY_LOCAL 6 — R9-7 process/result truth

Independently challenge Data, Carrier ACK, Session DeliveryAck, PTO/retransmit, Recovery ACK/loss, successful/failed socket send, aborted reservation, Session delivery, malformed budget, Session accepted-empty/stale, Carrier accepted-empty/stale, rejected feedback, residual-domain failure and terminal result as distinct structured evidence. Diagnostics must follow the typed transition/classification claimed.

# READY_LOCAL 7 — R9-8 warm TCP readiness

Challenge D064 single-active/multi-ready: authenticated resume-bound warm standby carries readiness/control only and no application Data before promotion. Resource/readiness evidence stays separate from packet feedback and Session delivery.

# READY_LOCAL 8 — R9-9 health + promotion

Challenge integrated resolved-UDP-outcome -> health/hysteresis behavior: recoverable reliable-UDP loss stays on UDP rather than spuriously promoting TCP; promotion requires existing readiness/health gates; draining/failed UDP receives no new Data ownership.

# READY_LOCAL 9 — R9-10 uncertain replay + dedup + cleanup

Only genuinely uncertain Session ranges replay on promoted TCP; receiver Session dedup remains exactly-once; TCP receives no duplicate UDP packet-ACK layer; shutdown/error negatives clean resources and emit no false success.

# READY_LOCAL 10 — R9-11 lifecycle/terminal invariants

Close remaining lifecycle/terminal invariants for the materially new cross-process reliable-UDP/failover integration. Terminal classification remains distinct from intermediate diagnostics and cleanup is deterministic.

# READY_LOCAL 11 — R9-12 complete cross-process slice + final exact-tree provenance

After R9-4 through R9-11 close, run on the final pushed developer SHA:

```text
PYTHONDONTWRITEBYTECODE=1 bash scripts/check.sh
git diff --check
```

Confirm clean tree and persist exact SHA, UTC start/end, OS/arch, stable Rust and exit codes. No historical live evidence rewrite. Run the pinned decode fuzz only if this work actually changes wire decoder/parser/crypto framing.

# READY_LOCAL 12 — dedicated independent R9 review

Perform an independent bounded challenge of the materially new R9 send/admission/reservation/socket-outcome/recovery/PTO/demux/Session-ACK/Carrier-ACK/failover/migration/cleanup/diagnostic behavior. A bounded no-finding note is valid item-4 support when scope, inspected owners, focused commands/tests, exclusions and exact reachable anchor are explicit. Any concrete BLOCKER/HIGH returns immediately to smallest repair -> regression -> exact-tree gate.

# READY_LOCAL 13 — Q10/Q11/Q12 factual reconciliation

Only after a reachable independent R9 review anchor exists, reconcile `docs/status.md`, `docs/release-security-review-packet.md` and related Q10/Q11/Q12 evidence text against exact reachable implementation/review anchors. Preserve evidence boundaries. Do not change RC/freeze/release/production authority.

# READY_LOCAL 14 — repository-wide item-4 inventory refill if reconciliation remains blocked

If item 4 still lacks independent coverage, perform one repository-wide inventory against the 13 mandated core surfaces and enqueue only genuinely unreviewed implemented surfaces. Do not create checker/schema/framework/docs filler. Queue exhaustion is allowed only if broad inventory finds no concrete defect, no READY_LOCAL review-support, no READY_LIVE question, and every remaining item is truly policy/environment/release-authority gated.

## Core-surface inventory

Earlier reachable independent bounded review covers reliable-UDP engine basics, `CarrierState`, concurrent Carrier Manager/health/migration-back, FairScheduler/flow accounting, carrier adapters, `SessionRuntime`, observability including mixed queue/generic drop classification, package/reproducibility/operator scripts, dependency/build surface, CLI portability/output, algorithmic boundedness/validators, DeliveryLedger/process codec/datagram/crypto API and wire/parser surfaces.

The materially new cross-process R9 integration remains under active independent challenge. H-R9-057 is repaired; H-R9-058/H-R9-059 are current blockers, followed by exact-head negative-oracle gate restoration, full R9-4, R9-5..R9-12 and dedicated R9 review. Repository-wide queue exhaustion is false.

## VPS opportunity

**Not READY.** Standing authorization remains valid, but authoritative classification is `READY_LIVE: none`. H-R9-058/H-R9-059 and current R9 work are deterministic local correctness/resource/evidence work and create no unresolved real-network hypothesis. Do not repeat HY2, warm failover, periodic/soak, package lifecycle, migration-back, endpoint/key migration, IPv6, PLPMTUD or Experimental Track evidence merely because the VPS remains rented.

Only create a new READY_LIVE row if later code/instrumentation/hypothesis/path conditions produce a concrete unresolved real-network question that local/loopback evidence cannot answer.

## Separate non-blocking policy / authority gates

Do not invent or change while executing this queue:

- `SessionRuntime.events` retention/capacity policy;
- D019 source-retention/no-reset policy;
- RSEC-001 adversarial-load/capacity suitability conditions;
- signing/key custody/SBOM/publication policy;
- previous-frozen-release policy;
- core Session/Carrier/ACK/crypto/wire architecture;
- destructive/canonical-meaning migration;
- RC/freeze/release/production authority.

These do not block independent dependency-ready local work above.
