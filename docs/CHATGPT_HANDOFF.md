# ChatGPT reviewer handoff — H-R9-053 reopens aborted-send committed high-water; R9-3 blocked

## Current repository truth

- Current `main` before this handoff update: reviewer-owned docs commit `c919523659ad5a056d84c8f9b3ffd600fc9d06c0` (`docs(review): reopen aborted-send committed high-water as H-R9-053`).
- Latest developer-owned source/test commit remains exact `f954a83a5556e77f50871410dc481e5b83d7bbac`, on top of `4488d09a79ed150f4a1c7d732f4f0ecffedfb381`.
- **Accepted H-R9-051 progress remains valid:** both executable retransmit callers check socket outcome; socket `Err` routes through complete `ReliableUdpRuntime::abandon_retransmit`, reverses Recovery/Reno/packet->frame ownership, skips positive send evidence/counters/caller outstanding, does not advance `post_pn_client`, and retained frame/plaintext ownership remains available for retry. The partial `ReliableUdpRuntime::abandon_sent` / `PathRecovery::recovery_mut` bypass is removed and `packets_sent` rolls back.
- **H-R9-053 HIGH reopens R9-3:** exact-current `Recovery::abandon_sent` restores `largest_sent` with `self.sent.keys().next_back().copied()`. That is only the largest packet still *in flight*, not the previous committed-to-socket high-water. If committed packet `N` was already ACKed/lost and removed from `sent`, then higher reservation `M` is aborted, rollback can lower `largest_sent` below `N` or to `None`. This can reject a legitimate late/duplicate ACK for actually-sent `N` and can allow packet-number reuse/regression `<= N`, weakening the authenticated record-sequence/AEAD-nonce monotonicity invariant.
- Independent reviewer note: `docs/reviews/independent-r9-abort-highwater-history-f954a83-20260918.md` at exact reachable reviewer commit `c919523659ad5a056d84c8f9b3ffd600fc9d06c0`.
- The existing `f954a83` regression covers only the narrow case where the previous committed packet remains present in `sent`; it does not first retire the previous high-water and therefore does not discriminate H-R9-053.
- Developer-reported local exact-tree provenance for exact `f954a83` remains a separate fact: `PYTHONDONTWRITEBYTECODE=1 bash scripts/check.sh` exit 0, `git diff --check` exit 0, clean worktree, 2026-09-18T08:10:18Z → 2026-09-18T08:10:26Z, Linux x86_64, rustc 1.98.0 (88d9e12ae 2026-08-18).
- **Hosted cross-evidence is red, not green:** GitHub-hosted Rust CI run `35323003081` for exact `f954a83` failed `stable checks` in `scripts/check.sh` at `cargo fmt --check` under stable rustc/rustfmt 1.98.1; its nightly decode fuzz-smoke job passed. The docs-only descendant `dcb4fb` also has failed hosted run `35323142232` because it inherits the same source tree. This does not erase the developer-local provenance, but the current source formatting must be repaired in the next source/test slice and hosted CI must not be described as green.
- Open PRs at review time: none.
- Candidate A ordinary future/never-sent ACK rejection remains accepted on the ordinary path, but the abort/rollback historical-watermark variant is open as H-R9-053. Candidate B mixed queue/generic datagram-drop observability remains closed.
- `READY_LIVE: none`; release item 3 incomplete; release item 4 incomplete; `RELEASE_CANDIDATE=false`; `PRODUCTION_READY=false`; `FREEZE=false`; `RELEASED=false`.

The external coding agent must synchronize to current `main` and continuously execute every dependency-ready slice below: implementation/review -> focused deterministic tests -> commit -> push -> next slice. Reviewer cadence is only a check frequency and is never a work-ticket length or reason to idle.

## Accepted progress that must remain closed

Do not revert accepted repairs without contradictory current repository evidence:

- H-R9-050 exact-wire executable admission at `c2e263e`: retransmit admission/ownership uses encoded/sealed wire bytes, not retained plaintext length.
- H-R9-051 socket-outcome transaction work at `267604d` + `f954a83`: caller socket result checks, success-only positive sent evidence, complete rollback owner, Reno/accounting rollback, and removed partial rollback API.
- H-R9-040 lifecycle-scoped `acked_frames` pruning and H-R9-041 exact-wire implementation ordering at `9a113f2`.
- H-R9-043 semantic correction at `5be123b`; H-R9-044 per-event deadline/accepted-empty correction at `05b1d7b` + `ebfc620`; H-R9-045 identity/order at `0766dd8` + `68e364a`; H-R9-046 post-positive-retirement negative at `d4f1ea3`; H-R9-047 older-sibling classification at `d2478ad`; H-R9-048 source projection at `1c3bf1d`; H-R9-049 discriminator at `a92d076` + `5f2baf2`.
- H-R9-039 P4-B server-exit assertion at `021d79d`; H-R9-038 P4 residual diagnostics/oracles at `236e962` + `9091803` + `021d79d`.
- H-R9-037 reverse-order three-way packet bind at `d5d5b23`, independently reviewed at `f2529087`; H-R9-036 reverse-order exact oracle at `855c679`.
- P2 C1-C4 exact closure at `142f0a1` + `8d0ccd5` + `1f8bbb0` + `be03dc3`.
- H-R9-034 settlement-continuation accepted-empty classification at `b801b65`; H-R9-033 initial-loop accepted-empty classification at `7a7c48c`; H-R9-032 packet bind/settlement proof at `ddafbb1`.
- Session DeliveryAck and Carrier packet ACK remain separate evidence domains. Accepted-empty is classification-only and must not describe a positive Recovery retirement.

# READY_LOCAL 1 — H-R9-053 historical committed high-water repair + source-format gate

Read exact-current `crates/neko-reliable/src/lib.rs` (`Recovery::{on_sent,on_ack,abandon_sent}`), `crates/neko-carrier/src/lib.rs` (`PathRecovery::{on_sent,abandon_sent}`, `ReliableUdpRuntime::{on_retransmit_sent,abandon_retransmit}`), `crates/neko-cli/src/main.rs` H-R9-051/052 tests/callers, and reviewer note `c919523` before editing.

Required invariants:

- `largest_sent` / equivalent ACK-valid send history must represent the highest **successfully committed-to-socket** packet number, not merely `max(currently_in_flight)`;
- aborting a pre-send reservation may invalidate that aborted packet identity but must not erase history for older packets that were genuinely sent and later ACKed/lost/retired;
- legitimate late/duplicate ACKs for actually-sent-and-retired packets retain current accepted-empty/late semantics and cannot be rejected merely because the packet is no longer in `sent`;
- packet-number monotonicity must not regress after an abort; a later `on_sent` must still reject numbers at or below the prior committed high-water;
- ACK of the aborted reservation remains fail-closed and atomic before RTT/PTO/loss/cwnd/Session mutation;
- no unbounded abandoned-PN history, no invented TTL/LRU/history-size/capacity/security policy.

Smallest accepted repair/discriminator:

1. Preserve H-R9-050 exact-wire admission and H-R9-051 socket-result transaction behavior. Do not redesign Session, Carrier ACK, Session DeliveryAck, crypto framing or wire format.
2. Use a bounded reservation/commit/abort shape or equivalent local state that can restore the actual pre-reservation committed high-water even if that prior committed packet has already been removed from `sent` by ACK/loss.
3. Add a deterministic regression: commit packet `N`; ACK/retire `N` completely; reserve higher `M`; abort `M`; then prove (a) duplicate/late ACK for `N` remains accepted under existing empty/late semantics without unrelated RTT/PTO/loss/cwnd/Session mutation, (b) ACK for aborted `M` is rejected atomically, (c) `on_sent` still rejects packet numbers `<= N`, and (d) a valid retry number `> N` remains possible.
4. Add or retain a variant with an older lower packet still outstanding so rollback cannot silently lower committed history to `max(sent.keys())`.
5. Fix the current `cargo fmt --check` delta in the same source/test slice; do not paper over it with docs.
6. Run focused tests, then on the final pushed source/test SHA run `PYTHONDONTWRITEBYTECODE=1 bash scripts/check.sh`, `git diff --check`, confirm clean tree, and persist exact SHA, UTC start/end, OS/arch, stable Rust version and exit codes. Hosted CI remains cross-evidence; do not wait for it before continuing if the local exact-tree gate is clean. No decoder/framing change is required by this finding, so fuzz only if the actual repair touches those surfaces.

After closure, continue immediately to READY_LOCAL 2 without waiting for reviewer cadence.

# READY_LOCAL 2 — H-R9-042 deterministic just-before-PTO-deadline negative

At Recovery/runtime query level, challenge authoritative PTO at `deadline_us - 1` (or equivalent injected deterministic time): zero PTO/retransmit transition before deadline; expected probe at/after deadline. Wall-clock sleep is not the oracle. Reuse current M2 timing inputs; do not select a new PTO policy value. Preserve all accepted repeated-PTO identity/retirement semantics from H-R9-043 through H-R9-049 and the final H-R9-051/H-R9-053 send/commit truth.

# READY_LOCAL 3 — R9-4 ACK-loss + delayed original/reorder

Suppress one legitimate Carrier ACK, force legitimate retransmission, then release delayed original feedback. Require one logical Session delivery, Session dedup authority, fresh packet numbers/nonces, truthful PTO/loss/retransmit evidence and settled Recovery. Late sibling/original feedback may classify accepted-empty/stale after positive copy retirement but must never manufacture another Session transition, rejection or retransmit.

# READY_LOCAL 4 — R9-5 adversarial feedback / every Carrier continuation

Challenge future/never-sent/stale/duplicate/tampered and multi-range feedback across each exact-current `UdpAcknowledgement::Carrier` continuation: initial receive, settlement and post-return owners. Rejected feedback must be atomic and fail closed without RTT/PTO/loss/cwnd/Session mutation; accepted-empty remains classification-only. Explicitly distinguish: (a) genuinely sent-and-retired historical packet identity, whose late/duplicate feedback may remain accepted-empty, from (b) aborted pre-send reservation identity, whose feedback must be rejected under H-R9-053.

# READY_LOCAL 5 — R9-6 ownership/resource boundedness

Every first send and retransmit must consult congestion admission before ownership commit or use an equivalent bounded reservation/commit shape whose failure is atomic. Socket-send failure must not fabricate Recovery/caller ownership or ACK-valid send history. Keep bounded retransmit plaintext ownership, include H-R9-040 marker lifecycle, and prove deterministic teardown. No capacity-pressure benchmark and no invented capacity/security values.

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

The materially new cross-process R9 integration remains under active independent challenge. H-R9-053 is a concrete rollback-path correctness defect, so repository-wide queue exhaustion is false even before the downstream R9-4..R9-12 lanes.

## VPS opportunity

**Not READY.** Standing authorization remains valid, but authoritative classification is `READY_LIVE: none`. H-R9-053, H-R9-042 and downstream R9 work are deterministic local correctness/evidence questions and create no unresolved real-network hypothesis. Do not repeat HY2, warm failover, periodic/soak, package lifecycle, migration-back, endpoint/key migration, IPv6, PLPMTUD or Experimental Track evidence merely because the VPS remains rented.

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
