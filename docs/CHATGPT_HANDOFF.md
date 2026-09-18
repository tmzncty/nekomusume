# ChatGPT reviewer handoff — H-R9-054 remains closed; H-R9-042 reopened as H-R9-055 pre-deadline caller-oracle gap; R9-3 blocked

## Current repository truth

- Latest developer-owned source/test commit: exact `f7bc6a19d7e2d0b6051810aa0cb808b879bf0051` (`test(reliable): H-R9-042 deterministic just-before-PTO-deadline negative`), on top of `b69c20e050a5ce777608ffa8a2260fb6a6cf42b2`.
- **H-R9-054 remains closed and independently challenged:** developer regressions at `4761827b7edd4ad149c884f47f1232738ca5377e` + `d995afddc819964a5f994eba960071e840334337` prove packet-number reuse at/below committed watermark is refused without new Recovery/Reno ownership. Independent bounded no-finding review is reachable at `bb988bc35d88ea825d938496c61766cc30bb036d`.
- **H-R9-042 is reopened as HIGH H-R9-055 at reviewer anchor `abafbb6c6619a678476a82f3b2ea9dbbad3322e7`.** Exact `f7bc6a1` proves deterministic deadline arithmetic but does **not** prove the claimed just-before-deadline caller guard: the regression sets `now = deadline - 1` and then directly calls mutating `Recovery::on_pto(4)`, discards `probes_before`, and never executes the production `now_us >= deadline` decision owner. `Recovery::on_pto` increments `pto_count` and may increment persistent-congestion events, so the current oracle performs the transition it claims must not occur pre-deadline.
- Current post-return production source still appears to contain the intended `next_pto_deadline_us(...)` -> `now_us >= deadline` guard before `rt.pto_probe()`. H-R9-055 is therefore presently an **evidence/oracle correctness blocker**, not a demonstrated runtime PTO defect.
- `cli_regression_tests::abandoned_retransmit_restores_committed_sent_watermark` (H-R9-052), `socket_send_failure_rolls_back_retransmit_ownership` (H-R9-051), and `abandoned_retransmit_preserves_retired_committed_watermark` (H-R9-053/054) retained.
- `Recovery::watermark_on_reserve` records pre-reservation committed `largest_sent`; `abandon_sent` restores exact committed watermark; `PathRecovery::abandon_sent` rolls back `packets_sent`; `abandon_retransmit` is the single complete rollback owner.
- Developer-local exact-tree provenance for exact `f7bc6a1` remains factual but is **not a clean passing gate**: `PYTHONDONTWRITEBYTECODE=1 bash scripts/check.sh` exit 101 because `sigterm_after_ready_stops_and_releases_tcp_and_udp_bindings` failed with `server exited before READY` (recorded as a timing flake unrelated to the H-R9-042 source); `git diff --check` exit 0; clean worktree at pushed SHA; 2026-09-18T10:49:11Z -> 2026-09-18T10:52:31Z; Linux x86_64, rustc 1.98.0 (88d9e12ae 2026-08-18). The SIGTERM test passed in isolation and the H-R9-042 unit passed.
- GitHub-hosted Rust CI for exact `f7bc6a1`, run `35336471528`, completed **success** on 2026-09-18. This is cross-evidence only and cannot close a non-discriminating oracle.
- Reviewer-local repository execution was unavailable in the current automation sandbox because outbound DNS for `github.com` failed; reviewer source reasoning therefore does not claim a local gate. The reachable H-R9-055 note records this limitation explicitly.
- Still open before R9-3 completion: **H-R9-055 executable pre-deadline caller discriminator**.
- Open PRs: none at this review.
- Candidate A (future/never-sent ACK atomic rejection) remains closed; H-R9-054 closed the send-side reuse discriminator. Candidate B (mixed queue/generic datagram-drop observability) remains closed.
- `READY_LIVE: none`; release item 3 incomplete; release item 4 incomplete; `RELEASE_CANDIDATE=false`; `PRODUCTION_READY=false`; `FREEZE=false`; `RELEASED=false`.

The external coding agent must synchronize to current `main` and continuously execute every dependency-ready slice below: implementation/review -> focused deterministic tests -> commit -> push -> next slice. Reviewer cadence is only a check frequency and is never a work-ticket length or reason to idle.

## Accepted progress that must remain closed

Do not revert accepted repairs without contradictory current repository evidence:

- H-R9-050 exact-wire executable admission at `c2e263e`: retransmit admission/ownership uses encoded/sealed wire bytes.
- H-R9-051 socket-outcome transaction work at `267604d` + `f954a83`: socket result checks, success-only sent evidence, complete rollback owner, Reno/accounting rollback, partial rollback API removed.
- H-R9-052/H-R9-053 source semantics at `4488d09` + `f954a83` + `9f3786b` + `4821789`: aborted reservation is not ACK-valid; committed high-water survives prior packet retirement; format/clippy gate repaired.
- H-R9-054 send-side committed-watermark reuse discriminator at `4761827` + `d995afd` + formatting-only `b69c20e`, independently rechecked with no finding at `bb988bc`.
- H-R9-040 lifecycle-scoped `acked_frames` pruning and H-R9-041 exact-wire implementation ordering at `9a113f2`.
- H-R9-043 through H-R9-049 repeated-PTO identity/deadline/accepted-empty/retirement/source-projection work at the reachable anchors already recorded in repository history, except the **separate H-R9-042 just-before-deadline acceptance oracle**, now reopened as H-R9-055.
- H-R9-038/H-R9-039 P4 residual/server-exit evidence, H-R9-036/H-R9-037 reverse-order exact oracle, H-R9-032..034 settlement evidence, and P2 C1-C4 remain closed.
- Session DeliveryAck and Carrier packet ACK remain separate evidence domains. Accepted-empty is classification-only and must not describe a positive Recovery retirement.

# READY_LOCAL 1 — CLOSED at b69c20e / independently rechecked at bb988bc (H-R9-054)

H-R9-054 discriminator is complete: Carrier-level coverage rejects reuse at/below a restored committed watermark without changing in-flight ownership and admits a fresh higher packet; the executable CLI/runtime owner rejects committed `N=0` and current watermark `=1` with Recovery/Reno accounting unchanged, then admits fresh `101`. Independent bounded review at `bb988bc35d88ea825d938496c61766cc30bb036d` found no contradictory current evidence. Do not spend another slice reopening it absent new source/test changes or a concrete counterexample.

# READY_LOCAL 2 — H-R9-055 HIGH: executable just-before-PTO-deadline caller discriminator

Exact `f7bc6a1` must **not** be accepted as H-R9-042 closure. Its test calls `Recovery::on_pto(4)` after setting `now = deadline - 1`; that mutates `pto_count` (and can mutate persistent-congestion accounting) while the test claims zero pre-deadline transition. The production caller guard itself is not exercised.

Smallest closure contract, preserving current PTO architecture:

1. Make the actual executable post-return PTO decision owner testable and use that same owner from production. A small helper extraction is acceptable only if it owns both the `now_us >= deadline` decision and the `pto_probe()` invocation; a duplicated boolean helper is not sufficient.
2. Prepare an outstanding frame, obtain the current deadline, and snapshot PTO count, persistent-congestion count, Recovery packet/frame state, and caller-visible retransmit bookkeeping that the decision can mutate.
3. Invoke the production decision owner at `deadline - 1`. Require **no `pto_probe`/`on_pto` call**, no PTO/retransmit transition or diagnostic, and every captured state unchanged.
4. Invoke the same owner at exactly `deadline` (or later). Require one legitimate PTO transition/probe and the expected single increment/state effect.
5. The regression must deterministic-red if the production `now_us >= deadline` guard is removed or weakened. Do not use sleeps/wall-clock races and do not directly call `on_pto` in the pre-deadline negative.
6. Run the normal developer-local exact-tree gate on the final pushed source/test SHA and persist truthful provenance. No decoder/framing change is expected, so fuzz is not mechanically required.
7. Then continue immediately into R9-4 without waiting for reviewer cadence.

Independent finding: `docs/reviews/independent-r9-h042-predeadline-oracle-20260918.md` at `abafbb6c6619a678476a82f3b2ea9dbbad3322e7`.

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

The materially new cross-process R9 integration remains under active independent challenge. H-R9-054 remains independently closed; H-R9-055 is now the current HIGH at the PTO caller boundary, and downstream R9-4..R9-12 plus the dedicated R9 review remain real dependency-ready work. Repository-wide queue exhaustion is therefore false.

## VPS opportunity

**Not READY.** Standing authorization remains valid, but authoritative classification is `READY_LIVE: none`. H-R9-055 and downstream R9 work are deterministic local correctness/evidence questions and create no unresolved real-network hypothesis. Do not repeat HY2, warm failover, periodic/soak, package lifecycle, migration-back, endpoint/key migration, IPv6, PLPMTUD or Experimental Track evidence merely because the VPS remains rented.

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
