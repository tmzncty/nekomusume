# ChatGPT reviewer handoff — H-R9-071 Carrier ACK socket-outcome evidence HIGH; R9-7 blocked

## Current repository truth

- Latest developer-owned source/test commit: exact `fd41ea9ff9ec772951c61fb4ed5162fbc1977d95` (`test(cli): H-R9-070 require BOTH post-flood feedback channels`), on top of `d8b5adefc0309923be8540c928e4d435b85e1a52`.
- Current independent review finding: [`docs/reviews/independent-r9-7-carrier-ack-send-evidence-fd41ea9-20260919.md`](reviews/independent-r9-7-carrier-ack-send-evidence-fd41ea9-20260919.md), review commit `8eeccfb364f5da8156ae628468a3ba66d11fb964`.
- H-R9-068 source repair remains accepted at the currently reviewed owner: only exact ordinary `UDP delivery acknowledgement timeout` continues into PTO/delayed-ACK progress; malformed-bound exhaustion and receive/socket failure are terminal.
- H-R9-069 remains narrowly closed at exact `d8b5ade`: the flood seam sends exactly the existing malformed budget (`3`), the client regression requires exactly three `unexpected_logical_ack` events, and terminal cause is explicitly `UDP delivery acknowledgement malformed bound exceeded`.
- H-R9-070 remains narrowly closed at exact `fd41ea9`: `reliable_udp_post_return_malformed_bound_is_terminal` requires BOTH post-flood feedback domains — Session `udp_return_delivery_ack_sent` and Carrier `udp_return_packet_ack_sent` — so one channel alone no longer satisfies the oracle.
- **New HIGH H-R9-071:** the executable server still discards `UdpSocket::send_to` results for several Carrier packet-ACK paths and emits positive `udp_packet_ack_sent` / `udp_return_packet_ack_sent` diagnostics unconditionally afterward. A socket `Err` can therefore be projected as `*_sent`. This directly weakens H-R9-070 because its Carrier-side “valid feedback actually sent” premise trusts `udp_return_packet_ack_sent`.
- R9-4 / H-R9-060 remain independently closed. R9-5 remains bounded no-finding closed. R9-6 remains independently closed. H-R9-067 remains closed.
- Developer-local clean exact-tree provenance for exact `fd41ea9`: `PYTHONDONTWRITEBYTECODE=1 bash scripts/check.sh` exit 0, `git diff --check` exit 0, clean worktree at pushed SHA, 2026-09-19T04:50:01Z → 2026-09-19T04:55:00Z, Linux x86_64, rustc 1.98.0.
- GitHub-hosted Rust CI for exact `fd41ea9`: run `35422199161`, completed `success` on 2026-09-19. Hosted CI is cross-evidence only; it does not exercise an injected server Carrier-ACK socket failure and therefore cannot close H-R9-071.
- Reviewer-local execution is not claimed.
- Earlier accepted findings remain closed absent contradictory current evidence: Candidate A future/never-sent ACK guard; Candidate B mixed datagram-drop observability; H-R9-050 through H-R9-070 except the newly opened H-R9-071 boundary described here.
- `READY_LIVE: none`; release item 3 remains incomplete; release item 4 remains incomplete; `RELEASE_CANDIDATE=false`; `PRODUCTION_READY=false`; `FREEZE=false`; `RELEASED=false`.

The external coding agent must synchronize to current `main` and continuously execute every dependency-ready slice below: implementation/review -> focused deterministic tests -> commit -> push -> clean exact-tree local gate/provenance -> next slice. Reviewer cadence is only a check frequency and is never a ticket length or reason to idle.

# READY_LOCAL 1 — H-R9-071 Carrier ACK socket-outcome evidence truth — FRONT HIGH

Repair the R9-7 process/evidence owner before claiming complete malformed-terminality closure.

Current concrete counterexample:

- Multiple executable server Carrier packet-ACK owners use the shape `let _ = udp.send_to(...); emit_diagnostic(..., "*_packet_ack_sent", ...)`.
- Because the socket result is discarded, `Err` still emits a positive `sent` event.
- The affected surface includes the initial reliable-UDP `udp_packet_ack_sent` owner and normal / delayed-reorder post-return `udp_return_packet_ack_sent` owners.
- The post-return Session DeliveryAck positive event is not the same defect: its ordinary owner requires the socket send before emitting `udp_return_delivery_ack_sent`.
- H-R9-070 now correctly requires both feedback-domain event names, but the Carrier half can still be false-positive if the socket rejected the ACK datagram.

Required smallest repair:

1. Audit every executable server Carrier packet-ACK send owner that can emit `udp_packet_ack_sent` or `udp_return_packet_ack_sent`.
2. Emit positive `*_sent` only after `UdpSocket::send_to` returns `Ok`.
3. On `Err`, emit a typed failure classification such as `udp_packet_ack_send_failed` / `udp_return_packet_ack_send_failed`, or an equally precise existing event. Never relabel the failed attempt as sent.
4. Preserve current ACK ranges, Recovery, Session delivery, crypto/wire, D064, malformed budget and policy values. Do not redesign whether a later fresh packet/ACK opportunity may recover; this finding is about truthful socket-outcome projection unless the deterministic process regression exposes a separate concrete correctness defect.
5. Add a deterministic failure seam on the **real executable post-return Carrier ACK owner** after ACK construction/sealing, so tests can force a send `Err`. Do not rely on random OS failure and do not satisfy this with helper-only unit coverage.
6. The negative process regression must prove for the injected target ACK:
   - typed ACK-send failure exists;
   - no positive `udp_return_packet_ack_sent` exists for the failed send / packet identity;
   - H-R9-070 cannot infer complete dual feedback from the failed Carrier channel;
   - no final client success/settlement is fabricated solely from the failed ACK attempt.
7. Preserve a paired normal-path positive control proving successful ACK send still emits the positive event and ordinary settlement remains green.
8. Cover the same evidence rule for initial `udp_packet_ack_sent` and delayed/reordered post-return ACK owners in this coherent repair. A minimal shared outcome helper is acceptable if it reduces duplicated error-prone logic; do not create checker/schema/framework filler.

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

Re-challenge H-R9-062 through H-R9-071 together at current exact owner paths. Mutation of any source/test success projection should turn the appropriate oracle red. If no further defect appears, persist a scope-exact bounded no-finding note with exact inspected owners, challenged invariants, focused tests/commands, exclusions, reachable anchor and evidence class, then continue immediately.

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

**Not READY. `READY_LIVE: none`.** Standing authorization remains valid, but H-R9-071 and the current R9 queue are deterministic local correctness/evidence work and create no unresolved real-network question that loopback/process evidence cannot answer. Do not repeat HY2, warm failover, periodic/soak, package lifecycle, migration-back, endpoint/key migration, IPv6, PLPMTUD or Experimental Track evidence merely because the VPS remains rented.

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
