# ChatGPT reviewer handoff — H-R9-056 blocks R9-4; deep queue remains open

## Current repository truth

- Latest developer-owned source/test commit remains exact `b41e37a03594e040faeedf4d78ed2115fc10e160` (`fix(cli): type alias for due_pto_probe return — clippy type_complexity`), on top of `6221f8d40aaeb01e277ba27700f57fa73be74cc6` (`fix(cli): H-R9-055 executable pre-deadline PTO guard — due_pto_probe helper`). No newer developer-owned implementation/test commit was reachable when this review began.
- **H-R9-054 remains closed and independently challenged:** developer regressions at `4761827b7edd4ad149c884f47f1232738ca5377e` + `d995afddc819964a5f994eba960071e840334337` prove packet-number reuse at/below committed watermark is refused without new Recovery/Reno ownership. Independent bounded no-finding review is reachable at `bb988bc35d88ea825d938496c61766cc30bb036d`.
- **H-R9-055 / R9-3 remain independently closed:** `due_pto_probe(rt, now_us)` owns both the real deadline decision and mutating `rt.pto_probe()` call. Independent bounded no-finding review is reachable at `57bb1826532864310a2170049bb3016c131b53b3`.
- **New HIGH H-R9-056:** exact-current post-return ACK-loss semantics make the server send a freshly authenticated duplicate Session `DeliveryAck` after a PTO retransmission is Session-deduplicated, while client `recv_udp_delivery_ack` classifies that exact already-confirmed logical ACK as `unexpected_logical_ack`, increments the operation-wide malformed budget, and can terminalize repeated legitimate recovery feedback. Source-derived independent finding is reachable at `27b30daf93cdda2b49aa4401ebc5a681ce634eef` (`docs/reviews/independent-r9-h056-duplicate-session-ack-20260918.md`). R9-4 is blocked on the smallest repair + deterministic discriminator below.
- The H-R9-056 path is not crypto-envelope replay: the server produces a fresh authenticated envelope in response to a fresh-PN retransmission of the same stable logical bytes. `SessionRuntime::receive` deliberately accepts the byte-identical retransmission as `DuplicateDedup` without a second application enqueue, and `SessionRuntime::delivery_ack` already treats `end == confirmed_watermark` as zero-delta/idempotent rather than a second window release.
- `cli_regression_tests::abandoned_retransmit_restores_committed_sent_watermark` (H-R9-052), `socket_send_failure_rolls_back_retransmit_ownership` (H-R9-051), and `abandoned_retransmit_preserves_retired_committed_watermark` (H-R9-053/054) remain retained.
- `Recovery::watermark_on_reserve` records pre-reservation committed `largest_sent`; `abandon_sent` restores exact committed watermark; `PathRecovery::abandon_sent` rolls back `packets_sent`; `abandon_retransmit` is the single complete rollback owner.
- Developer-local clean exact-tree provenance for exact `b41e37a`: `PYTHONDONTWRITEBYTECODE=1 bash scripts/check.sh` exit 0, `git diff --check` exit 0, clean worktree at pushed SHA, 2026-09-18T11:54:02Z → 2026-09-18T11:58:23Z, Linux x86_64, rustc 1.98.0 (88d9e12ae 2026-08-18).
- GitHub-hosted Rust CI run `35341890289` for exact `b41e37a` completed success. This is cross-evidence only and does not exercise H-R9-056/R9-4.
- Reviewer-local execution is not claimed for H-R9-056; the finding is a deterministic exact-pushed-source path analysis. Do not rewrite earlier developer-local/hosted evidence as reviewer execution.
- Candidate A (future/never-sent ACK atomic rejection) remains closed; H-R9-054 closed the send-side reuse discriminator. Candidate B (mixed queue/generic datagram-drop observability) remains closed.
- `READY_LIVE: none`; release item 3 incomplete; release item 4 incomplete; `RELEASE_CANDIDATE=false`; `PRODUCTION_READY=false`; `FREEZE=false`; `RELEASED=false`.

The external coding agent must synchronize to current `main` and continuously execute every dependency-ready slice below: implementation/review -> focused deterministic tests -> commit -> push -> next slice. Reviewer cadence is only a check frequency and is never a work-ticket length or reason to idle.

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
- Session DeliveryAck and Carrier packet ACK remain separate evidence domains. Accepted-empty is classification-only and must not describe a positive Recovery retirement.

## Closed R9-3 reviewer gate

R9-3 no longer blocks downstream work. The H-R9-055 independent closure at `57bb1826532864310a2170049bb3016c131b53b3` found no contradictory current evidence. Do not spend another slice reopening R9-3 absent new source/test changes or a concrete counterexample.

# READY_LOCAL 1 — H-R9-056 exact duplicate Session ACK repair + R9-4 discriminator

Read the exact-current server post-return continuation, `SessionRuntime::receive`, client `recv_udp_delivery_ack`, `SessionRuntime::delivery_ack`, `ReliableUdpRuntime::{poll_outgoing_ack,apply_ack,pto_probe,on_retransmit_sent,abandon_retransmit}`, and the R9 process fixture before editing.

The concrete defect to close is narrow: after the original post-return Data is accepted and its Session DeliveryAck logically confirms the current bounded record, loss of only the Carrier ACK forces a fresh-PN retransmission. The server Session runtime byte-deduplicates that retransmission and returns `Ok(())`, but the surrounding real owner sends a freshly authenticated exact Session DeliveryAck again. The client has already removed the logical record from `outstanding`, so the same exact current-range ACK is presently labeled `unexpected_logical_ack` and charged to `malformed`.

Smallest repair contract:

1. preserve crypto replay rejection: replaying the same authenticated envelope remains invalid;
2. preserve fail-closed treatment for wrong session/stream/offset/length or otherwise unadmitted logical ACKs;
3. classify a **freshly authenticated exact ACK for the already-confirmed current bounded logical range** as accepted-empty/stale Session feedback (or equivalent zero-transition class), not malformed and not a second positive logical confirmation;
4. prefer the existing `SessionRuntime` confirmation state or an equivalent bounded exact current-operation record context; do not add unbounded Session-ACK history, TTL/LRU/capacity policy, or a second delivery architecture;
5. do not weaken the separation between Session DeliveryAck and Carrier packet ACK evidence.

The focused cross-process regression must use the real post-return owners and deterministically:

- deliver the original post-return Data;
- withhold **exactly the first legitimate Carrier ACK** long enough to force a real PTO + fresh packet-number/AEAD-sequence retransmission while still allowing the first Session DeliveryAck through;
- have the server deduplicate the retransmitted stable logical bytes exactly once at the application layer and emit the fresh duplicate Session DeliveryAck generated by the real owner;
- release the delayed original and retransmission/sibling Carrier feedback in a deterministic reorder rather than suppressing all packet feedback;
- prove exactly one positive Session confirmation transition for stream 1 / the tested range and exactly-once application delivery;
- prove the exact duplicate Session DeliveryAck is classification-only, does not increment the malformed budget, and cannot substitute for Carrier settlement;
- prove every positive Carrier retirement names a packet actually retired by Recovery; stale/duplicate Carrier feedback may classify accepted-empty only when current committed semantics allow it;
- prove no false rejection, no additional loss/retransmit after lifecycle resolution, final Recovery `in_flight == 0`, and retained retransmit plaintext releases exactly once;
- make the regression deterministic red if the current exact-duplicate `unexpected_logical_ack -> malformed += 1` behavior is restored.

The current `reliable_demux_bounds_an_authenticated_unexpected_logical_ack` test is not the discriminator: it uses an unrelated offset (`4096`) and repeated identical ciphertexts. Keep that negative intact while adding the exact confirmed-range case.

No wire/parser/crypto-framing change is expected; fuzz is not mechanically required. After the final pushed developer SHA, run the normal clean exact-tree local gate, persist truthful provenance, and continue immediately into R9-5 without waiting for reviewer cadence.

# READY_LOCAL 2 — R9-5 adversarial feedback / every Carrier continuation

Challenge future/never-sent/stale/duplicate/tampered and multi-range feedback across each current `UdpAcknowledgement::Carrier` continuation: initial receive, settlement and post-return owners. Rejected feedback must be atomic/fail closed without RTT/PTO/loss/cwnd/Session mutation. Distinguish genuinely sent-and-retired historical identities from aborted pre-send reservation identities. Include the newly repaired Session duplicate classification in the cross-domain audit so it cannot be confused with Carrier accepted-empty.

# READY_LOCAL 3 — R9-6 ownership/resource boundedness

Every first send and retransmit must consult congestion admission before ownership commit or use an equivalent bounded reservation/commit transaction whose failure is atomic. Socket-send failure must not fabricate Recovery/caller ownership or ACK-valid send history. Keep bounded retransmit plaintext ownership and prove deterministic teardown. No capacity-pressure benchmark and no invented capacity/security values.

# READY_LOCAL 4 — R9-7 process/result truth

Independently challenge Data, Carrier ACK, Session DeliveryAck, PTO/retransmit, Recovery ACK/loss, successful/failed socket send, aborted reservation, Session delivery, malformed budget, Session accepted-empty/stale, Carrier accepted-empty/stale, rejected feedback, residual-domain failure and terminal result as distinct structured evidence. Diagnostics must follow the typed transition/classification they claim.

# READY_LOCAL 5 — R9-8 warm TCP readiness

Challenge D064 single-active/multi-ready: authenticated resume-bound warm standby carries readiness/control only and no application Data before promotion. Resource/readiness evidence remains separate from packet feedback and Session delivery.

# READY_LOCAL 6 — R9-9 health + promotion

Challenge integrated resolved-UDP-outcome -> health/hysteresis behavior: recoverable reliable-UDP loss remains on UDP rather than spuriously promoting TCP; promotion requires existing readiness/health gates; draining/failed UDP receives no new Data ownership.

# READY_LOCAL 7 — R9-10 uncertain replay + dedup + cleanup

Only genuinely uncertain Session ranges replay on promoted TCP; receiver Session dedup stays exactly-once; TCP receives no duplicate UDP packet-ACK layer; shutdown/error negatives clean resources and emit no false success.

# READY_LOCAL 8 — R9-11 lifecycle/terminal invariants

Close remaining lifecycle/terminal invariants for the materially new cross-process reliable-UDP/failover integration. Terminal classification remains distinct from intermediate diagnostics and cleanup is deterministic.

# READY_LOCAL 9 — R9-12 complete cross-process slice + final exact-tree provenance

After R9-4 through R9-11 close, run `PYTHONDONTWRITEBYTECODE=1 bash scripts/check.sh`, `git diff --check`, confirm clean tree on the final pushed source/test SHA, and persist exact SHA, UTC start/end, OS/arch, stable Rust and exit codes without rewriting historical live evidence.

# READY_LOCAL 10 — dedicated independent R9 review

Perform an independent bounded challenge of materially new R9 send/admission/reservation/socket-outcome/recovery/PTO/demux/Session ACK/Carrier ACK/failover/migration/cleanup/diagnostic behavior. A bounded no-finding note is valid item-4 support when scope, inspected owners, focused commands/tests, exclusions and exact reachable anchor are explicit. Any concrete BLOCKER/HIGH returns immediately to smallest repair -> regression -> exact-tree gate.

# READY_LOCAL 11 — Q10/Q11/Q12 factual reconciliation

Only after a reachable independent R9 review anchor exists, reconcile `docs/status.md`, `docs/release-security-review-packet.md` and related Q10/Q11/Q12 evidence text with exact reachable implementation/review anchors. Preserve evidence boundaries and do not change RC/freeze/release/production authority.

# READY_LOCAL 12 — repository-wide item-4 inventory refill if reconciliation remains blocked

If Q10/Q11/Q12 cannot yet close because item 4 still lacks independent coverage, perform one repository-wide inventory against the 13 mandated core surfaces and enqueue only genuinely unreviewed implemented surfaces. Do not create filler/checker/schema churn. Queue exhaustion is allowed only if broad inventory finds no concrete defect, no READY_LOCAL review-support, no READY_LIVE question, and every remaining item is truly policy/environment/release-authority gated.

## Item-4 / core-surface inventory

Earlier reachable independent bounded review covers reliable-UDP engine basics, `CarrierState`, concurrent Carrier Manager/health/migration-back, FairScheduler/flow accounting, carrier adapters, `SessionRuntime`, observability including mixed queue/generic drop classification, package/reproducibility/operator scripts, dependency/build surface, CLI portability/output, algorithmic boundedness/validators, DeliveryLedger/process codec/datagram/crypto API and wire/parser surfaces.

The materially new cross-process R9 integration remains under active independent challenge. H-R9-054 and H-R9-055/R9-3 remain independently closed; H-R9-056 is now the concrete R9-4 blocker, followed by R9-5 through R9-12 plus the dedicated R9 review. Repository-wide queue exhaustion is therefore false.

## VPS opportunity

**Not READY.** Standing authorization remains valid, but authoritative classification is `READY_LIVE: none`. H-R9-056/R9 work is deterministic local correctness/evidence work and creates no unresolved real-network hypothesis. Do not repeat HY2, warm failover, periodic/soak, package lifecycle, migration-back, endpoint/key migration, IPv6, PLPMTUD or Experimental Track evidence merely because the VPS remains rented.

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
