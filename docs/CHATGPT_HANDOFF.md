# ChatGPT reviewer handoff — H-R9-060 blocks full R9-4 closure; deep queue remains open

## Current repository truth

- Current `main` includes independent reviewer finding `dd3e552f57d83ddd2024bfe5b6ea6a462c70aad1`, `docs/reviews/independent-r9-h060-session-accepted-empty-evidence-20260919.md`. Latest developer-owned source/test commit remains exact `8d2eea9ec66fb4260c93a86e683d77f47faa9923`, directly above `5f99a51a91b477b58f0b41d9b63e562ae2d60dea`.
- **H-R9-059 source/resource repair is independently accepted.** `SessionRuntime.confirmed_ranges` is removed; the exact-duplicate Session ACK witness is caller-owned current-operation `BTreeSet<(stream,offset,len)>`, populated only when an authenticated Session DeliveryAck exact-matches an admitted `outstanding` record. It dies with the operation and adds no Session-lifetime history/TTL/LRU/capacity policy.
- **H-R9-058 predicate repair is independently accepted.** Current deterministic negatives reject a fresh authenticated interior/subrange tuple and `len == 0`; broad `confirmed_watermark >= end` acceptance is no longer the current source semantics.
- **Exact-head stable negative-oracle rebind is accepted.** The two post-return withheld-ACK negatives bind to the terminal (last) residual and allow repeated same-range Session ACK emissions during bounded PTO/recovery rather than restoring premature first-timeout termination.
- **H-R9-057 remains closed.** `delay_reorder_done` makes `--delay-r9-ack-reorder` one-shot; do not re-open alternating ACK suppression absent contradictory current evidence.
- **New H-R9-060 — HIGH, evidence/oracle correctness.** The real post-return owner currently passes `&mut |_| {}` into `recv_udp_delivery_ack`, so the helper's `accepted_empty_logical_ack` classification for a fresh exact duplicate Session DeliveryAck is discarded. `reliable_udp_ack_loss_delayed_original_reorder_settles` therefore cannot prove that the real retransmission exercised the repaired current-operation accepted-empty path, cannot prove zero malformed charge for that duplicate, and cannot prove `unexpected_logical_ack` was absent. Its unit positive pre-populates `confirmed_acks`, so it proves only lookup semantics, not real witness population through the current-operation exact-match path.
- Exact pushed developer source/test `8d2eea9` has repository-persisted developer-local provenance: clean detached worktree, `PYTHONDONTWRITEBYTECODE=1 bash scripts/check.sh` exit 0, `git diff --check` clean, clean tree, around 2026-09-19 01:05–01:10 UTC+8. Keep this evidence class distinct from reviewer execution.
- GitHub-hosted Rust CI run `35372825479` for exact `8d2eea9` is green: `stable checks` success and `nightly decode fuzz smoke` success. Hosted CI is cross-evidence only.
- Reviewer-local execution is **not** claimed for this pass. The independent review is exact pushed source/test inspection plus repository-persisted developer provenance and hosted cross-evidence.
- H-R9-056 semantic direction remains: a **fresh authenticated exact duplicate** Session DeliveryAck for an exact already-confirmed current bounded record is classification-only accepted-empty; identical authenticated-envelope replay remains crypto replay rejection; semantically unadmitted feedback remains malformed/fail-closed.
- R9-3, H-R9-054 and H-R9-055 remain independently closed. Earlier H-R9-050 exact-wire admission, H-R9-051 retransmit socket-send transaction rollback, H-R9-052/H-R9-053 committed ACK-valid watermark restoration, H-R9-043..049 repeated-PTO identity/deadline/retirement/source-projection work, H-R9-040 lifecycle-scoped `acked_frames`, and prior settlement/reverse-order findings remain retained.
- Candidate A (future/never-sent ACK atomic rejection) remains closed. Candidate B (mixed queue/generic datagram-drop observability) remains closed.
- `READY_LIVE: none`; release item 3 incomplete; release item 4 incomplete; `RELEASE_CANDIDATE=false`; `PRODUCTION_READY=false`; `FREEZE=false`; `RELEASED=false`.

The external coding agent must synchronize to current `main` and continuously execute every dependency-ready slice below: implementation/review -> focused deterministic tests -> commit -> push -> exact-tree local gate/provenance -> next slice. Reviewer cadence is only a check frequency and is never a work-ticket length or reason to idle.

## Accepted progress that must remain closed

Do not revert accepted repairs without contradictory current repository evidence:

- H-R9-059 caller-owned bounded exact ACK witness at `5f99a51` + `8d2eea9`; no Session-lifetime historical range store.
- H-R9-058 exact-range negative discriminators at `5f99a51` + `8d2eea9`: subrange/interior and zero-length feedback stay fail-closed.
- H-R9-057 one-shot ACK-delay/reorder server seam at `08cbdaf6`.
- H-R9-050 exact-wire executable retransmit admission at `c2e263e`.
- H-R9-051 retransmit socket-outcome transaction work at `267604d` + `f954a83`.
- H-R9-052/H-R9-053 committed ACK-valid watermark work at `4488d09` + `f954a83` + `9f3786b` + `4821789`.
- H-R9-054 send-side committed-watermark reuse discriminator at `4761827` + `d995afd`, independently rechecked at `bb988bc`.
- H-R9-055 executable pre-deadline PTO owner at `6221f8d` + `b41e37a`, independently rechecked at `57bb182`.
- H-R9-040 lifecycle-scoped `acked_frames`; H-R9-043..H-R9-049 repeated-PTO identity/deadline/accepted-empty/retirement/source-projection; H-R9-038/H-R9-039 residual/server-exit; H-R9-036/H-R9-037 reverse-order exact oracle; H-R9-032..034 settlement; P2 C1-C4.
- Session DeliveryAck and Carrier packet ACK are separate evidence domains. Session accepted-empty and Carrier accepted-empty are separate classification-only outcomes and cannot fabricate a positive transition in the other domain.

# READY_LOCAL 1 — DONE at `5f99a51` + `8d2eea9`: bounded exact current-operation Session ACK witness

Independent source/resource review found no remaining H-R9-058/H-R9-059 defect in the repaired ownership shape. Do not reintroduce Session-lifetime exact-range history or broad watermark-based duplicate acceptance.

# READY_LOCAL 2 — DONE at `8d2eea9`: exact-head stable-gate restoration for post-return negative oracles

Do not restore `residual.len() == 1` or terminate at the first receive timeout merely to satisfy historical cardinality. Bounded PTO/recovery may emit repeated residuals / same-range Session ACKs until the operation deadline; final truth is the terminal residual and final result.

# READY_LOCAL 3 — H-R9-060 + full R9-4 ACK-loss / delayed-original-reorder closure

Treat this as one coherent implementation/evidence slice, not a chain of docs-only microtickets.

### H-R9-060 smallest repair

1. Preserve the caller-owned exact `confirmed_acks` witness and current Session/Carrier/ACK semantics.
2. The real post-return caller must not discard the helper's exact-duplicate Session classification. Replace the no-op diagnostic sink with caller-visible typed/structured evidence for the post-return exact duplicate path. A small diagnostic projection is enough; transport semantics need not change.
3. Bind the duplicate evidence to the exact current-operation `(stream, offset, len)` and expose enough state to demonstrate **zero malformed charge** and **zero second Session mutation**. Reuse existing state/counters; do not invent capacity/security policy.
4. The positive cross-process regression must populate `confirmed_acks` through the first genuine exact Session ACK, not by pre-filling the set. A later freshly sealed same-range Session ACK from the retransmission must visibly classify accepted-empty.
5. Preserve negatives: fresh authenticated interior/subrange/wrong tuple/zero-length feedback remains malformed; identical authenticated-envelope replay remains crypto replay rejection.

### Finish the existing R9-4 contract in the same process test

Using real cross-process owners:

- deliver original post-return Data;
- allow its first Session DeliveryAck;
- withhold **exactly** its first legitimate Carrier ACK until a real PTO/fresh-PN retransmission;
- server Session runtime byte-deduplicates stable logical bytes and freshly seals another exact Session DeliveryAck for that same range;
- client observes the fresh exact duplicate via the repaired current-operation Session accepted-empty path with zero malformed charge and no `unexpected_logical_ack`;
- exactly one positive `r9_udp_return_delivery_ack` / Session confirmation transition occurs for the range;
- release delayed original + sibling/current Carrier feedback deterministically; all later packets ACK normally;
- parse original and retransmit packet identities and prove every positive client Carrier retirement names an actually transmitted Recovery-owned identity; server delayed/sibling ACK identities must bind to that same transmitted set; stale feedback is accepted-empty only where current committed semantics permit;
- prove exactly-once application delivery / no second logical delivery side effect;
- prove there is no retransmit event after terminal lifecycle resolution / `r9_udp_post_return_settled`;
- final `in_flight == 0` and retained plaintext/frame ownership are released;
- the regression must deterministic-red if the Session duplicate is silently swallowed, charged as malformed, produces a second Session transition, broad/non-exact accepted-empty returns, or ACK suppression becomes alternating again.

After source/tests are pushed, run the normal exact developer-tree gate:

```text
PYTHONDONTWRITEBYTECODE=1 bash scripts/check.sh
git diff --check
```

Record exact pushed SHA, UTC start/end, OS/arch, stable Rust, exit codes and clean-tree state. No wire decoder/parser/crypto-framing change is required by H-R9-060, so do not mechanically run extra local fuzz solely for this repair; any automatic hosted fuzz remains cross-evidence.

Then continue immediately to R9-5 without waiting for the next reviewer cadence.

# READY_LOCAL 4 — R9-5 adversarial feedback / every Carrier continuation

Challenge future/never-sent/stale/duplicate/tampered and multi-range feedback across every current `UdpAcknowledgement::Carrier` continuation: initial receive, settlement and post-return owners. Rejected feedback is atomic/fail-closed without RTT/PTO/loss/cwnd/Session mutation. Distinguish sent-and-retired historical identities from aborted pre-send reservation identities. Audit Session duplicate classification separately so it cannot substitute for Carrier accepted-empty.

# READY_LOCAL 5 — R9-6 ownership/resource boundedness

Every first send/retransmit must consult congestion admission before ownership commit or use equivalent bounded reservation/commit with atomic failure. Socket-send failure cannot fabricate Recovery/caller ownership, ACK-valid history, or positive process evidence. Retained retransmit plaintext ownership must be bounded and deterministically released. Include the H-R9-059 lesson: historical classification metadata must remain accounted/bounded by existing owners. No capacity-pressure benchmark and no invented policy values.

Explicitly challenge the post-return **first-send** owner as well as retransmit owners: Recovery ownership / `r9_udp_post_return_sent` evidence must not survive a real socket-send failure unless current committed semantics deliberately define a reservation event separately from a successful send. Do not silently assume H-R9-051's retransmit rollback covers first-send ordering.

# READY_LOCAL 6 — R9-7 process/result truth

Independently challenge Data, Carrier ACK, Session DeliveryAck, PTO/retransmit, Recovery ACK/loss, successful/failed socket send, aborted reservation, Session delivery, malformed budget, Session accepted-empty/stale, Carrier accepted-empty/stale, rejected feedback, residual-domain failure and terminal result as distinct structured evidence. Diagnostics must follow the typed transition/classification claimed. H-R9-060's post-return Session accepted-empty projection is part of this audit surface, not a substitute for it.

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

The materially new cross-process R9 integration remains under active independent challenge. H-R9-057/H-R9-058/H-R9-059 are repaired; H-R9-060 is the current HIGH evidence blocker, followed by coherent R9-4 closure, R9-5..R9-12 and dedicated R9 review. Repository-wide queue exhaustion is false.

## VPS opportunity

**Not READY.** Standing authorization remains valid, but authoritative classification is `READY_LIVE: none`. H-R9-060 and current R9 work are deterministic local correctness/evidence work and create no unresolved real-network hypothesis. Do not repeat HY2, warm failover, periodic/soak, package lifecycle, migration-back, endpoint/key migration, IPv6, PLPMTUD or Experimental Track evidence merely because the VPS remains rented.

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
