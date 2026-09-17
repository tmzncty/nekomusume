# ChatGPT reviewer handoff — finish H-R9-040, then close R9-3 exact recovery proof

## Current repository truth

- Current independent reviewer anchor: exact `85c58c3a7ae2bb015350cc77596575c0adffe714` (`docs(review): keep R9-3 blocked on prune and retransmit admission`).
- Latest developer-owned source/test commit reviewed: exact `92d5b8d80da645c122d679a33a806fd0c6e59ce8` (`feat(cli): R9-3 data-loss recovery — PTO retransmit + monotonic clock + acked_frames prune`). Its parent developer repair is `be50ed5955b7eaba88275cbcbd0be3addf1dd43f` (H-R9-040 positive-ACK retransmit gate).
- Independent review note: [`docs/reviews/reviewer-r9-3-head-92d5b8d-20260917.md`](reviews/reviewer-r9-3-head-92d5b8d-20260917.md).
- GitHub-hosted Rust CI for exact `92d5b8d` is green at run `35244453584`: `stable checks` / `bash scripts/check.sh` success and `nightly decode fuzz smoke` success. Hosted CI is cross-evidence only.
- No reachable developer-local clean exact-tree provenance for exact `92d5b8d` was found in the repository at this review. Preserve the older exact-tree provenance records for their own SHAs; do not relabel them as `92d5b8d` evidence.
- Open PRs at review time: none.
- Candidate A (future/never-sent ACK atomic rejection) remains closed. Candidate B (mixed generic/queue datagram-drop observability) remains closed.
- `READY_LIVE: none`; release item 3 incomplete; release item 4 incomplete; `RELEASE_CANDIDATE=false`; `PRODUCTION_READY=false`; `FREEZE=false`; `RELEASED=false`.

The external coding agent must synchronize to current `main` and continuously execute every dependency-ready slice below: implementation/review -> focused deterministic tests -> commit -> push -> next slice. Reviewer cadence is only a check frequency and is never a work-ticket length or reason to idle.

## Accepted progress that must remain closed

Do not revert these without contradictory repository evidence:

- H-R9-039 P4-B server-exit assertion at `021d79d`.
- H-R9-038 P4 dual-domain residual diagnostics and exact-oracle strengthening at `236e962` + `9091803` + `021d79d`.
- H-R9-037 reverse-order true three-way packet bind at `d5d5b23`, independently reviewed at `f2529087`.
- H-R9-036 reverse-order exact oracle at `855c679`.
- Post-return ACK arrival-order challenge at `83d10b0` + `424583d` + `855c679` + `d5d5b23`.
- P2 C1-C4 exact closure at `142f0a1` + `8d0ccd5` + `1f8bbb0` + `be03dc3`.
- H-R9-034 settlement-continuation accepted-empty classification and late-stale proof at `b801b65`; H-R9-033 initial-loop accepted-empty classification at `7a7c48c`; H-R9-032 packet-number cross-bind/settlement proof at `ddafbb1`; earlier ACK-obligation/classification/identity/malformed/order/count repairs remain accepted.
- `be50ed5` correctly establishes the **positive-ACK retransmit-eligibility gate** for overlapping packet copies: once any copy of stable frame `F` is Carrier-ACKed, PTO/later sibling loss must not schedule `F` again while that overlap lifecycle remains open. Do not remove this behavior while finishing H-R9-040 bounded-state pruning.
- `92d5b8d` correctly moves authenticated Carrier ACK observation time to a monotonic operation clock after receive/decrypt/decode, adds the read-only PTO deadline query path, recovery-owned post-return data-drop seam, fresh-packet retransmit path and a useful cross-process recovery fixture. The findings below narrow closure; they do not justify reverting the whole slice.
- Session DeliveryAck and Carrier packet ACK remain separate evidence domains. Do not merge them.

# READY_LOCAL 1 — finish H-R9-040: prune positive-ACK frame state

**HIGH correctness/resource blocker.** Exact `92d5b8d` still leaves H-R9-040 incomplete.

Current `Recovery` has:

- `outstanding_frames: BTreeMap<FrameId, usize>` for packet-copy lifetime;
- `acked_frames: BTreeSet<FrameId>` for positive Carrier receive evidence.

The positive-ACK gate is correct, but `acked_frames` is never pruned after the final packet copy retires. That violates the prior repair contract and creates both retained-state growth and stale semantic poisoning: after all old copies of `F` are gone, a later legal lifecycle reusing `F` can be incorrectly filtered from PTO because the old ACK marker survives.

### Minimal repair invariant

Keep the current positive-ACK gate and packet-copy accounting. Whenever `release_frame(...)` reports that the **last** outstanding copy of a frame retired, remove that frame's positive-ACK marker. Apply this on both ACK and loss retirement paths. Do not eagerly erase sibling packet copies.

### Required deterministic regressions

1. single-copy `F` ACKed -> `frame_outstanding(F)==false`; later higher-numbered fresh packet reuses `F`; before any ACK for that new lifecycle, PTO may select `F` normally;
2. same-call sibling ACK + time-loss -> ACKed sibling suppresses retransmit, final ownership empties and marker is pruned;
3. split-call sibling ACK then later loss -> PTO suppressed while old sibling remains; final loss emits no retransmit and prunes marker;
4. all-copies-lost control still schedules `F` exactly once;
5. future/never-sent ACK rejection remains atomic.

Commit/push this repair and continue immediately to READY_LOCAL 2. No decoder/framing change is expected; do not mechanically run fuzz solely for this repair.

# READY_LOCAL 2 — H-R9-041 exact encoded-byte congestion admission for retransmit

**HIGH correctness/resource blocker.** Exact `92d5b8d` currently calls `rt.can_send(plaintext.len() as u64)` before building/sealing the retransmission, then commits `rt.on_retransmit_sent(..., re_sealed.len() as u64, ...)` using a larger/different byte count.

The already-decided retransmission order is:

`build ProcessMessage -> seal -> can_send(exact re_sealed.len) -> on_retransmit_sent(exact same len) -> socket send`.

Repair only that ordering/byte-accounting seam. Do not add a second cwnd ledger or invent a capacity value.

Add a deterministic refusal regression using existing cwnd/accounting state so the remaining budget falls between plaintext length and sealed length. Refusal must produce no retransmission packet ownership, no socket send and no `r9_udp_retransmit_sent` event. Keep a normal admitted control proving one commit/send.

Then continue immediately to READY_LOCAL 3.

# READY_LOCAL 3 — H-R9-042 / R9-3 exact acceptance closure

**HIGH evidence/oracle blocker.** The current built-binary R9-3 fixture proves much of the recovery path but not all claims recorded in the intended acceptance.

Required closure after READY_LOCAL 1-2:

- exactly one suppressed original `r9_udp_post_return_sent` is Recovery-owned before intentional wire drop;
- authoritative PTO deadline is queried before `pto_probe`; no retransmit transition may occur before that deadline;
- add a deterministic **just-before-deadline** challenge (`deadline_us - 1` or equivalent injectable/query-level check) proving zero PTO/retransmit transition without sleep-based timing as the oracle;
- at/after deadline exactly one expected stable-frame probe fires;
- retransmission keeps Session/stream/offset/len/payload logical identity and stable `FrameId`, but uses a fresh packet number/nonce;
- exactly one retransmission client send event, exactly one server `udp_return_packet_ack_sent`, exactly one positive client Carrier retirement; bind all three packet numbers equal and different from the suppressed original;
- exactly one `stream=1 offset=48 len=16` Session confirmation;
- zero rejected/accepted-empty substitution on the positive recovery path;
- suppressed original is removed by committed loss rules, final Recovery `in_flight==0`, and exactly one settlement occurs only after Session confirmation + Recovery zero;
- positive completion requires both client and server success.

The current test comment says “three-way bind” but only binds retransmit packet number to client retirement; add the server ACK identity/cardinality explicitly.

After this closure, run the normal clean exact-tree developer-local gate on the resulting final pushed source/test SHA and persist provenance: `PYTHONDONTWRITEBYTECODE=1 bash scripts/check.sh`, `git diff --check`, clean tree, exact SHA, UTC start/end, OS/arch, stable Rust version and exit codes. Decoder fuzz is only required if the actual repair changes decoder/parser/crypto framing.

# READY_LOCAL 4 — R9-4 ACK-loss + delayed original/reorder

Suppress one legitimate Carrier ACK, force a legitimate retransmission, then release the delayed original ACK. Require one logical Session delivery, Session dedup authority, fresh packet numbers/nonces, truthful PTO/loss/retransmit evidence and settled Recovery. A late original may classify accepted-empty/stale but must never manufacture another transition, rejection or retransmit after positive frame receipt. Include overlap-copy feedback behavior after the H-R9-040 pruning repair.

# READY_LOCAL 5 — R9-5 adversarial feedback / every Carrier caller

Challenge future/never-sent/stale/duplicate/tampered feedback across every exact-current `UdpAcknowledgement::Carrier` continuation: initial receive, settlement and post-return owners. Rejected feedback must be atomic and fail closed without RTT/PTO/loss/cwnd/Session mutation; accepted-empty must not become transition/rejection. Include packet-copy overlap and reused-FrameId lifecycles.

# READY_LOCAL 6 — R9-6 ownership/resource boundedness

Every first send and retransmit must consult congestion admission on **exact encoded bytes** before packet ownership commit; refusal commits no packet/logical ownership. Prove one bounded retransmit-plaintext owner, bounded/pruned H-R9-040 per-frame state, deterministic teardown and no stale frame marker after lifecycle closure. Do not add a capacity-pressure benchmark or invent capacity/security values.

# READY_LOCAL 7 — R9-7 process/result truth

Independently challenge that Data, Carrier ACK, Session DeliveryAck, PTO/retransmit, Recovery ACK/loss, Session delivery, malformed budget, accepted-empty feedback, rejected feedback, residual-domain failure and terminal result remain distinct in structured process evidence. Emit diagnostics only after the claimed transition/classification occurred.

# READY_LOCAL 8 — R9-8 warm TCP readiness

Challenge the existing D064 single-active/multi-ready contract: authenticated resume-bound warm standby may carry readiness/control only and no application Data before promotion. Readiness/resource admission remains separate from packet feedback and Session delivery. No architecture change.

# READY_LOCAL 9 — R9-9 health + promotion

Challenge the integrated path: resolved UDP outcomes feed current health/hysteresis; recoverable reliable-UDP loss remains on UDP rather than spuriously promoting TCP; promotion requires existing readiness/health gates; draining/failed UDP receives no new Data ownership.

# READY_LOCAL 10 — R9-10 uncertain replay + dedup + cleanup

Challenge only genuine uncertain Session ranges replaying on promoted TCP; receiver Session dedup remains exactly-once; TCP gets no duplicate UDP packet-ACK layer; shutdown/error negatives clean resources and emit no false success.

# READY_LOCAL 11 — R9-11 lifecycle/terminal invariants

Close remaining lifecycle/terminal invariants for the materially new cross-process reliable-UDP/failover integration. Terminal classification stays distinct from intermediate diagnostics and cleanup is deterministic.

# READY_LOCAL 12 — R9-12 complete cross-process slice + exact-tree gate

Finish remaining R9 implementation/test work and run the clean exact-tree developer-local gate on the final pushed source/test SHA. Preserve source/test/provenance/hosted-CI/live-evidence boundaries; do not rewrite historical live evidence to describe the new local tree.

# READY_LOCAL 13 — dedicated independent R9 review

After READY_LOCAL 1-12 are reachable and gated, perform a dedicated independent bounded challenge of materially new R9 send/admission/recovery/PTO/demux/Session ACK/Carrier ACK/failover/migration/cleanup/diagnostic behavior. A bounded no-finding review is valid item-4 support if scope, inspected owners, commands/tests, exclusions and exact reachable anchor are explicit. Any concrete BLOCKER/HIGH returns immediately to smallest repair -> discriminating regression -> exact-tree local gate.

# READY_LOCAL 14 — Q10/Q11/Q12 factual reconciliation

Only after a reachable independent R9 review anchor exists, factually reconcile `docs/status.md`, `docs/release-security-review-packet.md` and related Q10/Q11/Q12 evidence text with exact reachable implementation/review anchors. Preserve historical evidence boundaries and do not change RC/freeze/release/production authority flags.

## Item-4 / core-surface inventory

Reachable independent bounded review already exists for earlier reliable-UDP basics (including future/never-sent ACK guard), `CarrierState`, concurrent Carrier Manager/health/migration-back, FairScheduler/flow accounting, carrier adapters, `SessionRuntime`, observability including mixed queue/generic drop classification, package/reproducibility/operator scripts, dependency/build surface, CLI portability/output, algorithmic boundedness/validators, DeliveryLedger/process codec/datagram/crypto API and wire/parser surfaces.

The materially new cross-process R9 integration remains broad uncovered item-4 work until READY_LOCAL 13. H-R9-040/H-R9-041/H-R9-042 are concrete dependency-ready local work, so repository-wide queue exhaustion is false.

## VPS opportunity

**Not READY.** Standing authorization remains valid, but authoritative classification remains `READY_LIVE: none`. The current blockers are local recovery correctness/evidence and the later independent cross-process R9 review. Do not repeat HY2, warm failover, periodic/soak, package lifecycle, migration-back, endpoint/key migration, IPv6, PLPMTUD or Experimental Track evidence without a new code/instrumentation/hypothesis/path-condition question.
