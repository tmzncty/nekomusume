# ChatGPT reviewer handoff — H-R9-062 independently closed; H-R9-063 retained first-send ownership HIGH

## Current repository truth

- Current reviewed default-branch HEAD before this handoff update: reviewer commit `4920ed27ed89132dcf852c88dc78c9b209f00ff1`, containing [`docs/reviews/independent-r9-6-first-send-retained-ownership-20260919.md`](reviews/independent-r9-6-first-send-retained-ownership-20260919.md).
- Latest developer-owned source/test commit: exact `ad2296db56230a6ec8665273b15b8aa82536f9fb` (`test(cli): H-R9-062 production-owner first-send socket-failure discriminator`), on top of `f8670a5a0ec95c4159e046fa12c4eec2177c63ff`.
- **H-R9-062 is independently closed as bounded evidence at exact `ad2296d`.** `--fail-r9-first-send` injects `Err` after post-return reliable-UDP admission into the same executable first-send owner and same error branch used by real `UdpSocket::send_to`. The process regression proves `r9_udp_post_return_send_failed`, absence of `r9_udp_post_return_sent` and `r9_udp_post_return_settled`, while the lower-level rollback regression retains the Recovery/Reno/watermark/ACK-validity discriminator. Current source inspection also confirms the initial, second-record and post-return reliable-UDP first-send branches all route socket `Err` through `ReliableUdpRuntime::abandon_sent` and emit positive sent evidence only on `Ok`.
- **New HIGH H-R9-063: first-send socket abort leaves stable plaintext with no schedulable retry owner.** `ReliableUdpRuntime::on_packet_sent` retains plaintext in `RetransmitBuffer`. `ReliableUdpRuntime::abandon_sent` removes `packet_frames` and calls `PathRecovery::abandon_sent`; Recovery removes the aborted packet and its only-copy frame from `outstanding_frames`, so `frame_outstanding(frame)==false`, `next_pto_deadline_us()==None` when no other packet remains, and `on_pto` cannot schedule that frame. But `abandon_sent` does not release the retained plaintext. Repeated distinct first-send socket errors can therefore accumulate unreachable retained entries inside the existing bounded buffer and eventually poison later valid transport admission. No new capacity value or policy is needed to repair this.
- H-R9-061's socket-outcome source repair remains the baseline: all three executable first-send call sites check real socket outcome; `--drop-r9-data` remains intentionally Recovery-owned controlled loss and must not be converted to abort semantics.
- Developer-local clean exact-tree provenance for exact `ad2296d`: `PYTHONDONTWRITEBYTECODE=1 bash scripts/check.sh` exit 0, `git diff --check` exit 0, clean worktree at pushed SHA, 2026-09-18T20:50:24Z → 2026-09-19T04:55:00+08:00, Linux x86_64, rustc 1.98.0.
- GitHub-hosted Rust CI for exact `ad2296d`, run `35393332734`, is green: `stable checks` passed `bash scripts/check.sh`; `nightly decode fuzz smoke` also passed pinned `cargo fuzz build decode` and `cargo fuzz run decode -- -max_total_time=30 -max_len=8192`. Hosted CI is cross-evidence only and does not inspect retained plaintext ownership after the injected abort.
- Reviewer-local execution is **not claimed**. A fresh clone attempt failed before checkout because this reviewer environment could not resolve `github.com`. Exact pushed source/test inspection, developer-persisted local provenance and hosted CI remain distinct evidence classes.
- **R9-4 / H-R9-060 remain independently closed** at source/test tree `95d9388`; **R9-5 remains closed no-finding** at `b9f0dc5467771f2398d06f2f6a66ebb3a36c1111`.
- Earlier accepted findings remain closed absent contradictory current evidence: Candidate A future/never-sent ACK guard; Candidate B mixed datagram-drop observability; H-R9-050 exact-wire retransmit admission; H-R9-051 retransmit socket rollback; H-R9-052/H-R9-053 committed ACK-valid watermark restoration; H-R9-054 committed-watermark reuse discriminator; H-R9-055 executable pre-deadline PTO owner; H-R9-057 one-shot ACK-delay/reorder; H-R9-058 exact Session ACK witness; H-R9-059 operation-owned bounded witness; H-R9-060 typed R9-4 accepted-empty evidence; H-R9-061 first-send socket transaction; H-R9-062 production-owner fault-injection oracle.
- `READY_LIVE: none`; release item 3 incomplete; release item 4 incomplete; `RELEASE_CANDIDATE=false`; `PRODUCTION_READY=false`; `FREEZE=false`; `RELEASED=false`.

The external coding agent must synchronize to current `main` and continuously execute every dependency-ready slice below: implementation/review -> focused deterministic tests -> commit -> push -> clean exact-tree local gate/provenance -> next slice. Reviewer cadence is only a check frequency and is never a work-ticket length or reason to idle.

# READY_LOCAL 1 — H-R9-063 / R9-6B retained first-send ownership — FRONT HIGH

Repair the concrete orphan-retained-plaintext defect without inventing new retention policy or changing core transport architecture.

Required contract:

1. Preserve exact-wire congestion admission, `SecureSession` sequence/nonce as the single packet-number source, current Recovery/Reno rollback, and packet-number monotonicity. An aborted PN remains non-ACK-valid and is never reused.
2. Make **first-send socket abort ownership complete**. After removing the aborted packet copy from Recovery, stable plaintext for each affected frame must be released when no other Recovery packet copy still owns that frame and no already-existing retry owner can schedule it.
3. Preserve retransmit-abort semantics. A failed retransmission must not discard stable plaintext while another legitimate packet copy remains outstanding or an already-existing retry lifecycle still requires it. Do not blindly `clear()` the retransmit buffer and do not conflate first-send abort with retransmit abort.
4. Add a focused deterministic `ReliableUdpRuntime` discriminator:
   - `on_packet_sent(frame A)` -> `abandon_sent` leaves `in_flight == 0`, Reno bytes-in-flight zero for that copy, no packet->frame ownership, rolled-back sent count/watermark, ACK(aborted PN) rejected, and **zero retained plaintext ownership for frame A**;
   - several distinct first-send aborts do not accumulate retained frame/byte ownership. This is an ownership regression, not a capacity-pressure benchmark;
   - paired success retains plaintext while a packet copy is genuinely outstanding and releases it through the existing ACK/loss lifecycle.
5. Add a sibling/retransmit negative: if frame F still has another legitimate outstanding packet copy, aborting one packet copy must leave F resealable/retransmittable until the surviving lifecycle settles. This is the discriminator against over-release.
6. Strengthen the existing `--fail-r9-first-send` process regression only where it binds the real owner: terminal residual should show `remaining_in_flight:0`; no positive `r9_udp_post_return_sent`, `r9_udp_retransmit_sent`, `r9_udp_pto_fired`, settlement, Session-delivery or Carrier-retirement evidence may be fabricated from the failed first send.
7. Re-audit the initial, second-record and post-return executable first-send callers after the runtime cleanup repair. Prefer one correct `ReliableUdpRuntime::abandon_sent` ownership owner rather than three divergent caller-side cleanup implementations.
8. Preserve `--drop-r9-data` as deliberate Recovery-owned loss. It must keep retained plaintext and PTO/retransmission semantics.
9. No new TTL/LRU/history-size/capacity/security values, no D019 change, no ACK redesign, no wire/crypto architecture change, no release-authority change.

After the final pushed repair SHA, run and persist the ordinary exact-tree developer gate:

```text
PYTHONDONTWRITEBYTECODE=1 bash scripts/check.sh
git diff --check
```

Record exact reachable pushed SHA, UTC start/end, exit codes, OS/arch, stable Rust version and clean-tree state. The finding requires no decoder/parser/crypto-framing change, so fuzz is not mechanically required unless the actual repair touches those surfaces.

On closure, continue immediately to READY_LOCAL 2 without waiting for reviewer cadence.

# READY_LOCAL 2 — R9-6 remaining ownership / resource boundedness completion

After H-R9-063 closes, finish R9-6 as an independent bounded challenge rather than assuming the retained-plaintext repair closes every ownership question.

Minimum scope:

- first-send and retransmit packet-copy ownership cannot survive refusal/abort/terminal teardown incorrectly;
- retransmit exact-wire admission remains truthful;
- committed packet numbers are never reused; aborted reservations do not become ACK-valid history;
- retained plaintext/frame ownership stays tied to a real Recovery/retry owner and releases deterministically on ACK/loss/final teardown;
- packet-to-frame maps, Reno bytes-in-flight, Recovery sent/outstanding state and retained plaintext reconcile after each positive/negative outcome;
- retransmit socket-abort with overlapping sibling copies preserves exactly the needed stable plaintext, no more and no less;
- terminal teardown deterministically clears residual packet/frame/plaintext ownership without rewriting historical failure evidence;
- historical classification metadata remains bounded/accounted by existing owners.

If a concrete correctness/security/evidence defect appears, repair the smallest current-semantics seam with discriminating positive/negative tests and exact-tree gate. If no defect appears, persist a precise bounded no-finding note and continue immediately.

# READY_LOCAL 3 — R9-7 process / result truth

Independently challenge that structured diagnostics correspond to the typed state transition/classification they claim. Keep these domains distinct:

- Data accepted/deduplicated;
- Carrier ACK positive retirement;
- Carrier accepted-empty/stale;
- Carrier rejected feedback;
- Session DeliveryAck positive confirmation;
- Session accepted-empty exact duplicate;
- Session unexpected/malformed feedback;
- PTO fired / retransmit sent;
- Recovery loss/ACK;
- successful vs failed socket send / aborted reservation;
- retained-ownership release vs packet-copy retirement;
- residual-domain failure;
- terminal result/settlement.

H-R9-062/H-R9-063's production socket-success/failure and retained-ownership boundaries must be included. No diagnostic may manufacture evidence in another domain. Add only discriminators that would actually fail if the production owner regressed; do not generate schema/checker filler.

# READY_LOCAL 4 — R9 mid-slice factual reconciliation

After the coherent R9-4/R9-5/R9-6/R9-7 group is closed, perform one **small factual reconciliation** of release/item-4 state before continuing. Do not rewrite the full release packet yet.

Check only current facts:

- exact reachable R9 anchors and evidence classes;
- item 3 remains incomplete unless repository truth changed independently;
- item 4 remains incomplete until the later dedicated R9 review/reconciliation;
- `READY_LIVE` remains `none` unless new code creates a concrete unresolved real-network question;
- release/freeze/production flags remain unchanged.

# READY_LOCAL 5 — R9-8 warm TCP readiness

Challenge D064 single-active / multi-ready semantics on the materially new cross-process integration:

- authenticated resume-bound warm standby carries readiness/control only;
- no application Data is owned/sent on TCP before promotion;
- readiness is not inferred from TCP connect, UDP packet feedback or Session DeliveryAck;
- readiness resource admission/generation/session/epoch binding remains exact;
- failed/incomplete readiness cannot emit positive promotion evidence or steal active ownership.

No policy-value changes.

# READY_LOCAL 6 — R9-9 health + promotion

Challenge integrated resolved reliable-UDP outcomes -> Carrier health/hysteresis -> promotion:

- recoverable reliable-UDP loss/PTO that successfully settles must stay on UDP rather than spuriously promote TCP;
- actual promotion requires existing readiness + health/hysteresis gates;
- draining/failed UDP cannot receive new Session Data ownership;
- single-active invariant holds through transition;
- health/decision evidence remains distinct from packet feedback and Session delivery evidence.

# READY_LOCAL 7 — R9-10 uncertain replay + dedup + cleanup

Challenge failover after a genuinely unresolved UDP logical range:

- only genuinely uncertain Session ranges replay on promoted TCP;
- stable Session/stream/offset identity is preserved;
- receiver dedup remains exactly-once and conflicting bytes fail closed;
- TCP path does not grow a duplicate UDP packet-ACK layer;
- resolved UDP ranges do not replay;
- shutdown/error/partial-promotion negatives clean retained ownership and emit no false success.

Do not change core replay architecture. Any ambiguity requiring such a change is a maintainer/architecture gate while independent local lanes continue.

# READY_LOCAL 8 — R9-11 lifecycle / terminal invariants

Close remaining lifecycle and terminal invariants for the cross-process reliable-UDP/failover integration:

- exactly one terminal success/failure classification per bounded operation;
- intermediate timeout/residual diagnostics are not terminal success/failure;
- no post-terminal retransmit, delivery confirmation, promotion or new Data ownership;
- listener/socket/session/recovery/retained-plaintext state is deterministically cleaned on normal and error exits;
- cleanup observations do not rewrite historical failure evidence.

# READY_LOCAL 9 — R9-12 complete cross-process slice + final exact-tree provenance

After R9-6 through R9-11 close, run on the final pushed developer SHA in a safe clean checkout/worktree:

```text
PYTHONDONTWRITEBYTECODE=1 bash scripts/check.sh
git diff --check
```

Confirm clean tree and persist exact reachable pushed SHA, UTC start/end, OS/arch, stable Rust version, exit codes and clean-tree state. Keep developer-local provenance distinct from hosted CI and reviewer execution. Run pinned decode fuzz only if accumulated R9 work actually changes wire decoder/parser/crypto framing.

Do not rewrite historical live evidence.

# READY_LOCAL 10 — dedicated independent R9 review

Perform a fresh independent bounded challenge of the materially new R9 send/admission/reservation/socket-outcome/recovery/PTO/demux/Session-ACK/Carrier-ACK/failover/migration/cleanup/diagnostic behavior at the final R9 implementation anchor.

A no-finding review is valid item-4 support only when it names inspected owners, challenged invariants, focused deterministic tests/commands, exclusions, exact reachable anchor and evidence class. Any concrete correctness/security/evidence BLOCKER/HIGH returns immediately to smallest repair -> regression -> exact-tree gate.

# READY_LOCAL 11 — Q10/Q11/Q12 factual reconciliation

Only after a reachable independent R9 review anchor exists, reconcile `docs/status.md`, `docs/release-security-review-packet.md`, `IMPLEMENTATION_PLAN.md` and related Q10/Q11/Q12 evidence text against exact reachable implementation/review anchors.

Preserve evidence boundaries:

- release item 3 remains incomplete unless repository truth independently closes it;
- item 4 closes only to the extent supported by reachable independent review evidence;
- no historical WAN artifact rewrite;
- no automatic RC/freeze/release/production transition;
- policy/authority gates stay separate.

# READY_LOCAL 12 — repository-wide item-4 inventory refill if reconciliation remains blocked

If item 4 remains incomplete after R9 independent review/reconciliation, perform one repository-wide inventory against the mandatory core surfaces:

1. `neko-reliable` recovery;
2. `CarrierState`;
3. concurrent Carrier Manager / health / migration-back;
4. FairScheduler / flow accounting;
5. carrier adapters;
6. `SessionRuntime`;
7. observability;
8. package/reproducibility/operator scripts;
9. dependency/build surface;
10. cross-platform CLI/process tests;
11. CLI exit/JSON/human-output contract;
12. algorithmic boundedness;
13. release packet factual consistency/evidence boundary.

Earlier reachable independent reviews cover established core implementations broadly; refill only genuinely implemented surfaces that still lack a dedicated reachable challenge or where R9 materially changed the owner. No checker/schema/framework/docs filler.

Repository-wide `queue exhausted` is permitted only when this broad inventory finds no concrete defect, no READY_LOCAL review/support lane, no READY_LIVE question, and every remaining item is truly policy/environment/release-authority gated.

## VPS opportunity

**Not READY. `READY_LIVE: none`.** Standing authorization remains valid, but H-R9-063 and the current R9 queue are deterministic local correctness/evidence work and create no unresolved real-network question that loopback/process evidence cannot answer. Do not repeat HY2, warm failover, periodic/soak, package lifecycle, migration-back, endpoint/key migration, IPv6, PLPMTUD or Experimental Track evidence merely because the VPS remains rented.

Only create a new READY_LIVE row if later code/instrumentation/hypothesis/path conditions produce a concrete unresolved real-network question and the run stays inside `docs/standing-vps-lab-authorization.md`.

## Separate non-blocking maintainer / policy / authority gates

Do not decide or modify while executing this queue:

- `SessionRuntime.events` retention/capacity policy;
- D019 source-retention/no-reset policy or numeric candidate values;
- RSEC-001 adversarial-load/capacity suitability conditions;
- signing/key-custody/SBOM/publication policy;
- previous frozen-release policy;
- core Session/Carrier/ACK/crypto/wire architecture;
- destructive/canonical-meaning migration;
- RC/freeze/release/production authority.

These gates do not block the independent dependency-ready local work above.
