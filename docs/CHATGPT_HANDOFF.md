# ChatGPT reviewer handoff — H-R9-074 closed at d97a536; R9-7 blocked on reviewer closure

## Current repository truth

- Latest developer-owned source/test commit: exact `d97a536414a78ff44ef80b1a6d428ddf96ae6e22` (`test(cli): H-R9-074 mandatory initial ACK control — udp_packet_ack_sent asserted`), on top of `65d30dc8a70a9af0da23042f54147a87241123e7`.
- Latest independent review finding: [`docs/reviews/independent-r9-7-carrier-ack-binding-oracle-3130c91-20260919.md`](reviews/independent-r9-7-carrier-ack-binding-oracle-3130c91-20260919.md), review commit `cd2675521b1e4f94471f8c99d6a4dd2c805195c8`.
- H-R9-068 source repair remains accepted at the currently reviewed owner: only exact ordinary `UDP delivery acknowledgement timeout` continues into PTO/delayed-ACK progress; malformed-bound exhaustion and receive/socket failure are terminal.
- H-R9-069 remains narrowly closed at exact `d8b5ade`: the flood seam sends exactly the existing malformed budget (`3`), the client regression requires exactly three `unexpected_logical_ack` events, and terminal cause is explicitly `UDP delivery acknowledgement malformed bound exceeded`.
- H-R9-070 remains narrowly closed at exact `fd41ea9`: `reliable_udp_post_return_malformed_bound_is_terminal` requires BOTH post-flood feedback domains — Session `udp_return_delivery_ack_sent` and Carrier `udp_return_packet_ack_sent`.
- H-R9-071 source-side socket-outcome defect remains repaired: inspected Carrier ACK owners project `UdpSocket::send_to` truthfully (`Ok` -> positive `*_sent`, `Err` -> typed `*_send_failed`).
- H-R9-072 remains useful process-oracle progress: `reliable_udp_carrier_ack_send_failure_is_typed_not_sent` exercises a real executable server/client path and the delayed/reordered fresh ACK owner.
- **H-R9-073 closed at 3130c91:** dedicated `--fail-r9-return-ack` targets ONLY post-return Carrier ACK owners; initial pre-migration ACKs still succeed.
- **H-R9-074 closed:** `reliable_udp_carrier_ack_send_failure_is_typed_not_sent` now requires `r9_udp_post_return_sent` with fail-closed `packet_number` parse (panic on missing/malformed), asserts `udp_return_delivery_ack_sent` (Session domain independent), and requires initial `udp_packet_ack_sent` — the post-return-only selector cannot silently widen back into an early-owner fault.
- R9-4 / H-R9-060 remain independently closed. R9-5 remains bounded no-finding closed. R9-6 remains independently closed. H-R9-067 remains closed.
- Developer-local clean exact-tree provenance for exact `d97a536`: `PYTHONDONTWRITEBYTECODE=1 bash scripts/check.sh` exit 0, `git diff --check` exit 0, clean worktree at pushed SHA, 2026-09-19T08:50:25Z → 2026-09-19T08:55:13Z, Linux x86_64, rustc 1.98.0. This remains developer-reported local evidence; reviewer-local execution is not claimed.
- GitHub-hosted Rust CI for exact `3130c91`: run `35430362590`, completed `success` on 2026-09-19. Hosted CI is cross-evidence only and cannot make a vacuous process oracle discriminating.
- Earlier accepted findings remain closed absent contradictory current evidence: Candidate A future/never-sent ACK guard; Candidate B mixed datagram-drop observability; H-R9-050 through H-R9-071 subject to the explicit current R9-7 attribution gaps.
- `READY_LIVE: none`; release item 3 remains incomplete; release item 4 remains incomplete; `RELEASE_CANDIDATE=false`; `PRODUCTION_READY=false`; `FREEZE=false`; `RELEASED=false`.

The external coding agent must synchronize to current `main` and continuously execute every dependency-ready slice below: implementation/review -> focused deterministic tests -> commit -> push -> clean exact-tree local gate/provenance -> next slice. Reviewer cadence is only a check frequency and is never a ticket length or reason to idle.

# READY_LOCAL 1 — H-R9-074 post-return Carrier ACK binding / dual-feedback oracle — FRONT HIGH

Close the remaining evidence-attribution gap without redesigning transport semantics.

Current exact facts:

- `3130c91` fixed the earlier stage-target problem: `--fail-r9-return-ack` is scoped to post-return Carrier ACK owners, and the regression now requires at least one typed `udp_return_packet_ack_send_failed`.
- The source-side socket-outcome rule remains correct: positive `udp_return_packet_ack_sent` only follows socket `Ok`; `Err` emits typed `*_send_failed`.
- However, the regression only performs the client/server packet-number equality check inside `if let Some(r9_udp_post_return_sent)`. If the client event is absent, the identity-binding assertion is skipped and the test can still green.
- Packet-number parsing falls back to sentinel values instead of failing closed on missing/malformed evidence.
- The negative fixture does not require `udp_return_delivery_ack_sent`, even though H-R9-073 required the Session-side post-return feedback to be independently present so lack of settlement is attributable to the Carrier half rather than a missing Session confirmation.
- The test comment says initial ACKs succeeded, but does not assert an initial `udp_packet_ack_sent` positive control.

Required smallest repair:

1. Preserve the dedicated post-return-only injection seam, current socket-outcome source rule, ACK ranges, Recovery, Session, crypto/wire, D064, D019, malformed budget and all policy values.
2. Make the target client evidence mandatory: require the intended `r9_udp_post_return_sent`, parse its `packet_number` fail-closed, and fail if event/field is missing or malformed. Remove the optional `if let` skip and sentinel fallbacks.
3. Match a server `udp_return_packet_ack_send_failed` by that exact target packet number; do not rely on “first failure line”. Prove there is no positive `udp_return_packet_ack_sent` for the same target identity.
4. Require the independent Session feedback half: assert `udp_return_delivery_ack_sent` for the same post-return logical record before accepting the Carrier-negative result. Session DeliveryAck alone is not complete dual feedback.
5. Require at least one initial pre-migration `udp_packet_ack_sent` positive control so the dedicated selector remains mutation-sensitive and cannot silently widen into an early-owner fault seam.
6. Preserve the client-side negative: no Carrier retirement/application for the failed target attempt. If the persistent injection mode is deterministically terminal under current committed semantics, pin its actual process result/terminal cause and forbid final summary/ordered-success evidence; if current Recovery deliberately permits later distinct recovery, assert the later successful ACK attempt with distinct identity instead of inventing a terminal rule.
7. Reuse or add a paired normal positive control for the same post-return owner: socket `Ok` -> `udp_return_packet_ack_sent` -> Carrier retirement -> dual-domain settlement.
8. Keep delayed-original and fresh-current reordered ACK attempts independently attributable in the later R9-7 sweep; one branch's evidence must not satisfy the other.
9. Do not satisfy this with helper-only coverage or random OS socket failure.

On the final pushed repair SHA run and persist:

```text
PYTHONDONTWRITEBYTECODE=1 bash scripts/check.sh
git diff --check
```

Record exact reachable pushed SHA, UTC start/end, exit codes, OS/arch, stable Rust version and clean-tree state. Fuzz is not mechanically required unless decoder/parser/crypto framing changes.

After closure continue immediately to READY_LOCAL 2 without waiting for reviewer cadence.

# READY_LOCAL 2 — R9-7 remaining process / result truth

Independently challenge that each structured diagnostic corresponds to the typed state transition/classification it claims. Keep distinct:

- Data accepted/deduplicated;
- Carrier ACK positive retirement vs accepted-empty/stale vs rejected;
- Session DeliveryAck positive confirmation vs exact duplicate accepted-empty vs unexpected/malformed;
- PTO fired vs retransmit actually socket-sent;
- Recovery loss/ACK;
- successful vs failed socket send / aborted reservation;
- retained-ownership release vs packet-copy retirement vs terminal teardown;
- terminal lifetime-history snapshot vs current live-owner zero;
- readiness/warm/active/promotion control evidence;
- residual/intermediate timeout-only classification;
- malformed-bound/socket/receive terminal error vs final process success/failure.

Re-challenge H-R9-062 through H-R9-074 together at current exact owner paths. Mutation of any source/test success projection should turn the appropriate oracle red. If no further defect appears, persist a scope-exact bounded no-finding note with exact inspected owners, challenged invariants, focused tests/commands, exclusions, reachable anchor and evidence class, then continue immediately.

# READY_LOCAL 3 — R9 mid-slice factual reconciliation

After R9-4/R9-5/R9-6/R9-7 are coherently closed, perform one small factual reconciliation rather than rewriting the entire release packet.

Check only current facts:

- exact reachable R9 implementation/review anchors and evidence classes;
- release item 3 remains incomplete unless repository truth independently changes;
- item 4 remains incomplete until later dedicated R9 review/reconciliation;
- `READY_LIVE` remains `none` unless new code/instrumentation creates a specific unresolved real-network question;
- RC/production/freeze/released flags remain unchanged.

# READY_LOCAL 4 — R9-8 warm TCP readiness

Challenge D064 single-active / multi-ready semantics on the cross-process integration:

- authenticated resume-bound warm standby carries readiness/control only;
- no application Data is owned/sent on TCP before promotion;
- readiness is not inferred from TCP connect, UDP packet feedback or Session DeliveryAck;
- readiness admission/generation/session/epoch binding remains exact;
- failed/incomplete readiness cannot emit positive promotion evidence or steal active ownership;
- a torn-down UDP runtime cannot manufacture new readiness/control evidence.

No policy-value changes.

# READY_LOCAL 5 — R9-9 health + promotion

Challenge integrated resolved reliable-UDP outcomes -> Carrier health/hysteresis -> promotion:

- recoverable reliable-UDP loss/PTO that settles stays on UDP rather than spuriously promoting TCP;
- terminal/torn-down UDP state cannot generate fresh health/readiness/promotion input;
- actual promotion requires existing readiness plus health/hysteresis gates;
- draining/failed UDP cannot receive new Session Data ownership;
- single-active invariant holds;
- health/decision evidence stays distinct from packet feedback and Session delivery evidence.

# READY_LOCAL 6 — R9-10 uncertain replay + dedup + cleanup

Challenge failover after a genuinely unresolved UDP logical range:

- only genuinely uncertain Session ranges replay on promoted TCP;
- stable Session/stream/offset identity is preserved;
- receiver dedup remains exactly-once and conflicting bytes fail closed;
- TCP path does not grow duplicate UDP packet-ACK semantics;
- resolved UDP ranges do not replay;
- shutdown/error/partial-promotion negatives clean Recovery/packet/plaintext ownership and emit no false success.

Do not change core replay architecture. Architecture ambiguity becomes a separate maintainer gate while independent local lanes continue.

# READY_LOCAL 7 — R9-11 lifecycle / terminal invariants

Close remaining lifecycle invariants for the cross-process reliable-UDP/failover integration:

- exactly one terminal success/failure classification per bounded operation;
- intermediate timeout/residual diagnostics are not terminal success/failure;
- malformed-bound/socket/receive terminal errors cannot be downgraded into timeout continuation;
- no post-terminal retransmit, delivery confirmation, Carrier ACK emission, readiness mutation, health promotion, or new Data ownership;
- listener/socket/session/recovery/Reno/ACK-tracker/retained-plaintext state cleans deterministically on normal/error exits;
- cleanup observations do not rewrite historical failure/recovery evidence.

# READY_LOCAL 8 — R9-12 complete cross-process slice + final exact-tree provenance

After R9-7 through R9-11 close, run on the final pushed developer SHA in a safe clean checkout/worktree:

```text
PYTHONDONTWRITEBYTECODE=1 bash scripts/check.sh
git diff --check
```

Confirm clean tree and persist exact reachable pushed SHA, UTC start/end, OS/arch, stable Rust version, exit codes and clean-tree state. Keep developer-local provenance distinct from hosted CI and reviewer execution. Run pinned decode fuzz only if accumulated R9 work actually changes wire decoder/parser/crypto framing.

# READY_LOCAL 9 — dedicated independent R9 review

Perform a fresh independent bounded challenge of the materially new R9 send/admission/reservation/socket-outcome/recovery/PTO/demux/Session-ACK/Carrier-ACK/failover/migration/cleanup/diagnostic behavior at the final R9 implementation anchor.

A no-finding review is valid item-4 support only when it names inspected owners, challenged invariants, focused deterministic tests/commands, exclusions, exact reachable anchor and evidence class. Any concrete correctness/security/evidence BLOCKER/HIGH returns immediately to smallest repair -> regression -> exact-tree gate.

# READY_LOCAL 10 — Q10/Q11/Q12 factual reconciliation

Only after a reachable independent R9 review anchor exists, reconcile `docs/status.md`, `docs/release-security-review-packet.md`, `IMPLEMENTATION_PLAN.md` and related Q10/Q11/Q12 evidence text against exact reachable implementation/review anchors.

Preserve evidence boundaries:

- release item 3 remains incomplete unless repository truth independently closes it;
- item 4 closes only to the extent supported by reachable independent review evidence;
- no historical WAN artifact rewrite;
- no automatic RC/freeze/release/production transition;
- policy/authority gates stay separate.

# READY_LOCAL 11 — repository-wide item-4 inventory refill if reconciliation remains blocked

If item 4 remains incomplete after R9 independent review/reconciliation, inventory all mandatory core surfaces:

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
12. algorithmic resource boundedness;
13. release packet factual consistency/evidence boundary.

Earlier reachable independent reviews cover established core implementations broadly; refill only genuinely implemented surfaces that still lack a dedicated reachable challenge or where R9 materially changed the owner. No checker/schema/framework/docs filler.

Repository-wide `queue exhausted` is permitted only when this broad inventory finds no concrete defect, no READY_LOCAL review/support lane, no READY_LIVE question, and every remaining item is truly policy/environment/release-authority gated.

## VPS opportunity

**Not READY. `READY_LIVE: none`.** Standing authorization remains valid, but H-R9-074 and the current R9 queue are deterministic local correctness/evidence work and create no unresolved real-network question that loopback/process evidence cannot answer. Do not repeat HY2, warm failover, periodic/soak, package lifecycle, migration-back, endpoint/key migration, IPv6, PLPMTUD or Experimental Track evidence merely because the VPS remains rented.

Only create a new READY_LIVE row if later code/instrumentation/hypothesis/path conditions produce a concrete unresolved real-network question inside `docs/standing-vps-lab-authorization.md`.

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

A policy gate in one lane does not block independent READY_LOCAL review/support lanes.
