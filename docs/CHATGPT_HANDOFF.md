# ChatGPT reviewer handoff — H-R9-040 first, then R9-3 PTO integration

## Current repository truth

- Current `main` independent reviewer anchor before this handoff update: exact `22fb01809998854e65324bc83444a62674a2e5ee` (`docs(review): block R9-3 on overlap ACK retransmit eligibility`).
- Latest developer-owned source/test commit reviewed remains exact `021d79d88dadc9dbc7cf749745b2395c06602b29` (`test(cli): P4-B assert server success (H-R9-039)`). No newer developer source/test commit exists at this review.
- New concrete finding: **H-R9-040 — HIGH correctness/resource blocker** in overlapping reliable-UDP packet copies. Current `Recovery` uses the same per-frame outstanding-copy count as retransmit eligibility. After one copy of stable `FrameId F` is positively Carrier-ACKed, another overlapping copy may still cause `on_pto` to probe `F`, or its later loss may emit `F` again in `retransmit_frames`. This can retain retransmit plaintext or schedule a spurious retransmission even though packet-recovery receive evidence already exists for `F`.
- Independent finding note: [`docs/reviews/reviewer-r9-3-overlap-ack-retransmit-021d79d-20260917.md`](reviews/reviewer-r9-3-overlap-ack-retransmit-021d79d-20260917.md), reachable at exact `22fb018`.
- Existing R9-3 navigation remains valid behind this repair:
  - [`docs/reviews/reviewer-r9-3-pto-timebase-navigation-021d79d-20260917.md`](reviews/reviewer-r9-3-pto-timebase-navigation-021d79d-20260917.md), reachable at `319c6bb`;
  - [`docs/reviews/reviewer-r9-3-ack-observation-time-021d79d-20260917.md`](reviews/reviewer-r9-3-ack-observation-time-021d79d-20260917.md), reachable at `938e290`;
  - [`docs/reviews/reviewer-r9-3-pto-api-ownership-021d79d-20260917.md`](reviews/reviewer-r9-3-pto-api-ownership-021d79d-20260917.md), reachable at `bb96fab`.
- H-R9-039 and H-R9-038 remain closed at `021d79d` / `9091803`; P2 C1-C4 and H-R9-037 through earlier accepted findings remain closed. Candidate A (future/never-sent ACK) and candidate B (mixed datagram-drop observability) remain closed.
- Developer-local clean exact-tree provenance for exact `021d79d` remains: `PYTHONDONTWRITEBYTECODE=1 bash scripts/check.sh` exit 0, `git diff --check` exit 0, clean worktree, 2026-09-17T06:54:00Z → 2026-09-17T06:57:45Z, Linux x86_64, rustc 1.98.0 (88d9e12ae 2026-08-18). One first-run failure of `sigterm_after_ready_stops_and_releases_tcp_and_udp_bindings` passed an isolated rerun at the same SHA; preserve that provenance truthfully.
- GitHub-hosted Rust CI for exact `021d79d` is separately green (run `35191357800`); hosted CI is cross-evidence, not a substitute for developer-local provenance.
- Open PRs at review time: none.
- `READY_LIVE: none`; release item 3 incomplete; release item 4 incomplete; `RELEASE_CANDIDATE=false`; `PRODUCTION_READY=false`; `FREEZE=false`; `RELEASED=false`.

The external coding agent must synchronize to current `main` and continuously execute every dependency-ready slice below: implementation/review -> focused deterministic tests -> commit -> push -> next slice. Reviewer cadence is only a check frequency and is never a work-ticket length or reason to idle.

## Accepted progress that must remain closed

Do not revert these exact-current repairs without contradictory evidence:

- H-R9-039 P4-B server-exit assertion at `021d79d`.
- H-R9-038 P4 dual-domain residual diagnostics and exact-oracle strengthening at `236e962` + `9091803` + `021d79d`.
- H-R9-037 reverse-order true three-way packet bind at `d5d5b23`, independently reviewed at `f2529087`.
- H-R9-036 reverse-order exact oracle at `855c679`.
- Post-return ACK arrival-order challenge at `83d10b0` + `424583d` + `855c679` + `d5d5b23`.
- P2 C1-C4 exact closure at `142f0a1` + `8d0ccd5` + `1f8bbb0` + `be03dc3`: TCP replay identity bound on both sides; migration milestones exact cardinality/order; server dual-domain stream/offset/len; positive-path three-way packet-number equality; exactly-one settled cardinality and post-transition order.
- H-R9-034 settlement-continuation accepted-empty classification and late-stale proof at `b801b65`; H-R9-033 initial-loop accepted-empty classification at `7a7c48c`; H-R9-032 packet-number cross-bind/settlement proof at `ddafbb1`; earlier ACK-obligation/classification/identity/malformed/order/count repairs remain accepted.
- Session DeliveryAck and Carrier packet ACK remain separate evidence domains. Do not merge them to solve H-R9-040 or R9-3.

# READY_LOCAL 1 — H-R9-040 overlap ACK / retransmit-eligibility repair

Repair the current reliable-UDP engine before extending R9-3 cross-process recovery.

### Current defect to reproduce

`Recovery` currently keeps `outstanding_frames: BTreeMap<FrameId, usize>` as both packet-copy lifetime and PTO/loss retransmit eligibility. For original `P0(F)` plus retransmitted `P1(F)`:

- ACKing `P1` decrements the count but does not remember that `F` has positive Carrier receive evidence;
- if old `P0` is time-lost in that same `on_ack`, final-copy removal emits `F` in `retransmit_frames` even though `P1` was ACKed;
- if `P0` is not lost yet, `on_pto` may still select `F` from the remaining outstanding-frame key even though `P1` was ACKed;
- `ReliableUdpRuntime::apply_ack` intentionally retains plaintext for scheduled retransmit frames, so the defect can become stale retained ownership or a spurious retransmission.

### Repair invariant

Separate **packet-copy lifetime** from **frame retransmit eligibility** while preserving existing packet accounting. Exact internal type/API shape is not normative. A bounded per-frame state such as `{ outstanding_copies, carrier_acked }` or an equivalently bounded ACK marker is acceptable.

Required behavior:

1. any positively ACKed packet containing stable frame `F` marks `F` positively received in the Carrier recovery domain;
2. other packet copies of `F` may remain tracked until ACK/loss resolution for packet-level RTT/loss/cwnd accounting;
3. retained plaintext may continue to follow the current final-copy ownership rule, but **PTO must not probe `F` after any copy was positively ACKed**;
4. later loss of another copy may emit `F` in `retransmit_frames` only if no copy of `F` was ever positively ACKed in that overlap lifecycle;
5. when the last packet copy retires, prune the per-frame positive-ACK state so state remains bounded by Recovery ownership;
6. do not eagerly erase old packet copies merely because a sibling copy ACKed;
7. do not change Session ACK, Carrier ACK encoding, wire/crypto, packet-number semantics, D019, capacity/security numbers or release policy.

### Focused deterministic regressions

- **same-call ACK + time-loss:** old `P0(F)`, fresh `P1(F)`; ACK `P1` at a timestamp that time-losses `P0`. Require `acked_packets=[P1]`, `lost_packets=[P0]`, `retransmit_frames=[]`, final `frame_outstanding(F)==false`; runtime retained plaintext becomes releasable and subsequent `pto_probe` is empty.
- **split-call ACK then later loss:** ACK `P1` while `P0` stays outstanding. Final-copy retention may keep `frame_outstanding(F)==true`, but `pto_probe` must not schedule `F`. Later loss of `P0` must still emit no retransmit frame and final retained plaintext must release.
- **all-copies-lost control:** if no copy of `F` was ever positively ACKed, final loss closure schedules `F` exactly once.
- Preserve future/never-sent ACK atomic rejection, packet loss/cwnd accounting and stable FrameId semantics.

After focused tests, commit/push and run the normal clean exact-tree developer gate. This repair does not touch decoder/parser/crypto framing, so do not mechanically run decode fuzz unless the actual implementation expands into those surfaces. Then continue immediately into READY_LOCAL 2; do not wait for the next reviewer cadence.

# READY_LOCAL 2 — R9-3 Data-loss recovery / real PTO clock

With H-R9-040 closed, implement the complete current M2 recovery contract on the cross-process R9 path.

### One authoritative monotonic recovery clock

At exact `021d79d`, current cross-process reliable sends and Carrier ACK application use recovery time `0`. Use one bounded local `Instant` origin for the complete reliable operation and convert `elapsed().as_micros()` deterministically into the existing `u64` recovery domain. Pass this time consistently to all relevant first-send `on_packet_sent`, retransmit `on_retransmit_sent`, and Carrier `apply_ack` calls.

`recv_udp_delivery_ack` may block. Do **not** sample `now_us` before entering it and reuse that stale scalar. Thread the origin/equivalent tiny time source into the existing shared authenticated ACK owner and sample elapsed time only after the Carrier ACK has actually been received/decrypted/decoded, immediately before `apply_ack`.

### Authoritative PTO deadline ownership

Prefer a read-only query through `Recovery` -> `PathRecovery` -> `ReliableUdpRuntime`, conceptually:

`next_pto_deadline_us(granularity_us, max_ack_delay_us) -> Option<u64>`

computed from the authoritative oldest outstanding ack-eliciting `SentPacket.sent_at_us` plus current `rtt.pto_us(..., pto_count)`. Do not expose the mutable sent map or create a second mutable timer/packet ledger in `failover_client`.

The existing bounded M2 lab baseline `LAB_PTO_GRANULARITY_US = 1_000` and `LAB_MAX_ACK_DELAY_US = 0` may be reused for this fixture. They are not a new global policy.

### Send/retransmit order

For the intentionally suppressed first wire send:

`seal/current packet number -> can_send(exact encoded bytes) -> on_packet_sent -> test-only one-shot socket-send suppression`.

For retransmission:

- drain authenticated ACKs first;
- query deadline before calling `pto_probe` because `pto_probe` mutates `pto_count`;
- never fire before deadline;
- at/after deadline require exactly the expected stable `FrameId` probe;
- re-seal the same logical Data under a fresh packet number/nonce;
- call `can_send(exact encoded bytes)` before `on_retransmit_sent`;
- commit retransmission ownership before socket send;
- apply its Carrier ACK at observation time from the same monotonic origin.

### R9-3 acceptance

- selected first packet is Recovery-owned before intentional wire suppression;
- deterministic just-before-deadline negative proves zero PTO/retransmit event; at/after deadline exactly one firing with `fired_at_us >= deadline_us`;
- retransmission preserves stable FrameId and stream/offset/len/payload logical identity but uses fresh packet number/nonce;
- every first/retransmit send proves congestion admission before Recovery ownership commit;
- receiver Session logical delivery remains exactly once;
- Session DeliveryAck and Carrier packet ACK remain separately observable;
- fresh retransmission ACK is a real positive retirement, not accepted-empty/rejected substitution;
- the suppressed original is removed by committed loss rules without H-R9-040-style resurrection, and final Recovery `in_flight==0` exactly;
- positive completion requires Session confirmation complete **and** Recovery zero;
- bounded negatives report actual residual domain and emit no false settlement/failover success.

If this reveals another concrete current-semantics defect, repair the smallest owner + discriminating regression, gate it, and continue.

# READY_LOCAL 3 — R9-4 ACK-loss + delayed original/reorder

Suppress one legitimate Carrier ACK, force a legitimate retransmission, then release the delayed original ACK. Require one logical Session delivery, Session dedup authority, fresh packet numbers/nonces, truthful PTO/loss/retransmit evidence and settled Recovery. A late original may classify accepted-empty/stale but must never manufacture another transition, rejection or retransmit after positive frame receipt.

# READY_LOCAL 4 — R9-5 adversarial feedback / every Carrier caller

Challenge future/never-sent/stale/duplicate/tampered feedback across every exact-current `UdpAcknowledgement::Carrier` continuation: initial receive, settlement and post-return owners. Rejected feedback must be atomic and fail closed without RTT/PTO/loss/cwnd/Session mutation; accepted-empty must not become transition/rejection. Recheck overlap-copy feedback in light of H-R9-040.

# READY_LOCAL 5 — R9-6 ownership/resource boundedness

Every first send and retransmit must consult congestion admission before ownership commit; refusal commits no recovery/logical state. Keep one bounded retransmit-plaintext owner, include H-R9-040 per-frame state in the boundedness challenge, and prove deterministic teardown. Do not add a capacity-pressure benchmark or invent capacity/security values.

# READY_LOCAL 6 — R9-7 process/result truth

Independently challenge that Data, Carrier ACK, Session DeliveryAck, PTO/retransmit, Recovery ACK/loss, Session delivery, malformed budget, accepted-empty feedback, rejected feedback and terminal result remain distinct in structured process evidence. Emit/accept diagnostics only after the claimed transition/classification occurred.

# READY_LOCAL 7 — R9-8 warm TCP readiness

Challenge the existing D064 single-active/multi-ready contract: authenticated resume-bound warm standby may carry readiness/control only and no application Data before promotion. Readiness/resource admission remains separate from packet feedback and Session delivery. No architecture change.

# READY_LOCAL 8 — R9-9 health + promotion

Challenge the integrated path: resolved UDP outcomes feed current health/hysteresis; recoverable reliable-UDP loss remains on UDP rather than spuriously promoting TCP; promotion requires existing readiness/health gates; draining/failed UDP receives no new Data ownership.

# READY_LOCAL 9 — R9-10 uncertain replay + dedup + cleanup

Challenge only genuine uncertain Session ranges replaying on promoted TCP; receiver Session dedup remains exactly-once; TCP gets no duplicate UDP packet-ACK layer; shutdown/error negatives clean resources and emit no false success.

# READY_LOCAL 10 — R9-11 lifecycle/terminal invariants

Close remaining lifecycle/terminal invariants for the materially new cross-process reliable-UDP/failover integration. Terminal classification stays distinct from intermediate diagnostics and cleanup is deterministic.

# READY_LOCAL 11 — R9-12 complete cross-process slice + exact-tree gate

Finish remaining R9 implementation/test work and run the clean exact-tree developer-local gate on the final pushed source/test SHA. Preserve source/test/provenance/hosted-CI/live-evidence boundaries; do not rewrite historical live evidence to describe the new local tree.

# READY_LOCAL 12 — dedicated independent R9 review

After READY_LOCAL 1-11 are reachable and gated, perform a dedicated independent bounded challenge of materially new R9 send/admission/recovery/PTO/demux/Session ACK/Carrier ACK/failover/migration/cleanup/diagnostic behavior. A bounded no-finding review is valid item-4 support if scope, inspected owners, commands/tests, exclusions and exact reachable anchor are explicit. Any concrete BLOCKER/HIGH returns immediately to smallest repair -> discriminating regression -> exact-tree local gate.

# READY_LOCAL 13 — Q10/Q11/Q12 factual reconciliation

Only after a reachable independent R9 review anchor exists, factually reconcile `docs/status.md`, `docs/release-security-review-packet.md` and related Q10/Q11/Q12 evidence text with exact reachable implementation/review anchors. Preserve historical evidence boundaries and do not change RC/freeze/release/production authority flags.

## Item-4 / core-surface inventory

Reachable independent bounded review already exists for earlier reliable-UDP engine basics (including future/never-sent ACK guard), `CarrierState`, concurrent Carrier Manager/health/migration-back, FairScheduler/flow accounting, carrier adapters, `SessionRuntime`, observability including mixed queue/generic drop classification, package/reproducibility/operator scripts, dependency/build surface, CLI portability/output, algorithmic boundedness/validators, DeliveryLedger/process codec/datagram/crypto API and wire/parser surfaces.

H-R9-040 is a newly discovered overlap-copy recovery defect within the materially new R9 challenge. The **cross-process R9 integration remains broad uncovered item-4 work** until READY_LOCAL 12. Repository-wide queue exhaustion is false.

## VPS opportunity

**Not READY.** Standing authorization remains valid, but authoritative classification is `READY_LIVE: none`. Current work is local recovery correctness/evidence and the later independent cross-process R9 review. Do not repeat HY2, warm failover, periodic/soak, package lifecycle, migration-back, endpoint/key migration, IPv6, PLPMTUD or Experimental Track evidence merely because the VPS remains rented.

Only create a new `READY_LIVE` row if later code/instrumentation/hypothesis/path conditions produce a concrete unresolved real-network question that local/loopback evidence cannot answer.

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
