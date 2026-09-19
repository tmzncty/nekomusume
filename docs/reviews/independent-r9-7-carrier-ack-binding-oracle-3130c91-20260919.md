# Independent R9-7 review — H-R9-074 post-return Carrier ACK binding / dual-feedback oracle gap

**Reviewed implementation/test anchor:** exact reachable pushed source/test commit `3130c914b3fe2c8722a5802a4728e509b6ad27d5`, with current handoff-only head `bf9004ea0653f7669ee6a191af87757a005f87f1`.

**Classification:** HIGH — process/evidence acceptance-oracle correctness. The new `--fail-r9-return-ack` source seam is materially better scoped than the prior global seam and does reach post-return Carrier ACK owners, but the regression still does not make the claimed packet-identity binding or “Carrier half is the only missing feedback domain” premise mutation-sensitive. This is not a wire, ACK-architecture, Recovery, Session-delivery, crypto, D064, D019, retention-capacity, or policy decision.

## Scope inspected

- developer-owned commit `3130c914b3fe2c8722a5802a4728e509b6ad27d5` and handoff closure `bf9004ea0653f7669ee6a191af87757a005f87f1`;
- `crates/neko-cli/src/main.rs` post-return normal and delayed/reordered Carrier ACK send owners and the Session DeliveryAck owner;
- `crates/neko-cli/tests/probe.rs::reliable_udp_carrier_ack_send_failure_is_typed_not_sent`;
- prior H-R9-073 acceptance contract in `docs/reviews/independent-r9-7-carrier-ack-target-oracle-d2e78a7-20260919.md`;
- current `README.md`, `AGENTS.md`, `ROADMAP.md`, `IMPLEMENTATION_PLAN.md`, `SECURITY.md`, `docs/status.md`, standing VPS authorization/priority, decisions, carrier architecture, Session v0 entry point, release/security packet, and current handoff.

Reviewer-local execution is **not** claimed. Developer-reported exact-tree provenance for `3130c91` is retained separately. GitHub-hosted Rust CI run `35430362590` completed `success` for exact `3130c91`; hosted CI is cross-evidence only and cannot make a vacuous oracle discriminating.

## What `3130c91` does repair

The dedicated `--fail-r9-return-ack` selector is consumed by post-return Carrier ACK owners while the initial pre-migration Carrier ACK owner keeps the old `--fail-r9-ack` selector. The negative fixture now requires at least one typed `udp_return_packet_ack_send_failed` and globally forbids positive `udp_return_packet_ack_sent` under the persistent failure mode. This closes the earlier “an initial ACK failure can masquerade as post-return coverage” source-seam defect.

The source-side rule also remains correct: `udp_return_packet_ack_sent` is emitted only on real socket `Ok`, while injected/socket `Err` emits `udp_return_packet_ack_send_failed`.

## Concrete remaining oracle gaps

### 1. The claimed packet-number binding is optional

The regression says it binds the failed server ACK to the client’s real `r9_udp_post_return_sent` packet number, but the client-side event is guarded by:

```text
if let Some(sent_ev) = ...find(r9_udp_post_return_sent) {
    ... assert_eq!(failed_pn, client_pn) ...
}
```

If `r9_udp_post_return_sent` disappears, is renamed, is emitted after the searched window, or a future mutation stops that positive owner evidence while the server still reaches a post-return packet and emits `udp_return_packet_ack_send_failed`, the identity assertion is skipped entirely and the test may still pass. The closure claim “binds the real packet_number” is therefore not mutation-sensitive.

The numeric extraction also falls back to sentinels (`u64::MAX` / `0`) instead of failing the test on malformed/missing `packet_number`. For an evidence oracle, missing/ill-formed target identity must be a hard test failure, not a substitute value.

### 2. The Session-side positive control required by H-R9-073 is still absent

The current server owner sends a valid Session `DeliveryAck` independently of the Carrier packet ACK. H-R9-073 explicitly required the negative fixture to observe Session-side post-return feedback (for example `udp_return_delivery_ack_sent`) so that lack of settlement is attributable to the Carrier half rather than an unrelated missing Session confirmation.

The repaired test does not assert `udp_return_delivery_ack_sent` at all. Therefore a mutation that breaks/suppresses Session DeliveryAck while preserving a post-return Carrier-ACK send failure can still satisfy:

- typed `udp_return_packet_ack_send_failed` exists;
- no positive `udp_return_packet_ack_sent` exists;
- client applies no Carrier ACK;
- no `r9_udp_post_return_settled` / `failover_client_ok` appears.

That is a false-green attribution: the test would no longer prove “Session feedback succeeded and only Carrier retirement was missing.”

### 3. The initial success control is documented but not asserted

The test comment says initial pre-migration ACKs succeeded, but there is no assertion requiring an initial `udp_packet_ack_sent`. The dedicated flag currently makes this true by construction, so this is not a new source defect; however the regression does not protect the seam against a future accidental widening or an unrelated early-path regression. Since H-R9-073 required this as a positive control, it should be made explicit rather than left as prose.

## Required smallest repair

1. Preserve the dedicated post-return-only injection seam and all current ACK ranges, Recovery, Session, crypto/wire, D064, D019, malformed budget and policy values.
2. Make the client post-return send evidence mandatory: require exactly the intended `r9_udp_post_return_sent` event for the target attempt (or otherwise deterministically select it), parse `packet_number` fail-closed, and fail the test if the event or numeric field is missing/malformed. Do not use `if let` to skip the binding assertion and do not use sentinel fallbacks.
3. Match the server failure by the **target packet number**, not merely by “first failure line”: locate a typed `udp_return_packet_ack_send_failed` carrying that same packet identity. Then prove no positive `udp_return_packet_ack_sent` exists for that same identity.
4. Require the independent Session half to have succeeded: assert server `udp_return_delivery_ack_sent` for the same post-return logical record before accepting absence of settlement. Keep Session DeliveryAck and Carrier ACK as separate evidence domains.
5. Require at least one initial pre-migration `udp_packet_ack_sent` positive control so the post-return-only selector remains mutation-sensitive and cannot silently widen back into an early-owner fault seam.
6. Preserve the client-side negative: no Carrier retirement may be applied for the failed target attempt. Do not treat a Session DeliveryAck alone as complete dual feedback.
7. If the persistent `--fail-r9-return-ack` mode is deterministically terminal under current committed semantics, pin the actual process result / terminal cause and forbid final summary / ordered-success evidence. If current Recovery semantics deliberately permit a later distinct recovery path, do not invent a terminal requirement; instead assert that any later success is tied to a distinct successful ACK attempt and not to the failed target identity.
8. Reuse or add a paired normal positive control for the same post-return owner: socket `Ok` -> `udp_return_packet_ack_sent` -> Carrier retirement -> dual-domain settlement. Do not satisfy this with helper-only coverage or random OS failure.
9. Keep delayed-original and fresh-current reordered ACK attempts independently attributable in the later R9-7 sweep; one branch’s evidence must not satisfy the other branch’s oracle.
10. On the final pushed repair SHA run and persist:

```text
PYTHONDONTWRITEBYTECODE=1 bash scripts/check.sh
git diff --check
```

Record exact reachable pushed SHA, UTC start/end, exit codes, OS/arch, stable Rust version and clean-tree state. Fuzz is not mechanically required unless decoder/parser/crypto framing changes.

## Queue consequence

Accept `3130c91` as useful source-seam progress and keep H-R9-071 source truth closed, but do **not** accept H-R9-073 as fully closed. Reopen the remaining evidence-attribution gap as **H-R9-074** and keep R9-7 blocked until the negative oracle makes target identity, initial-path success, and Session-vs-Carrier feedback separation mandatory.

After repair, continue immediately through the remaining R9-7 process/result truth sweep and the existing deep R9 queue without waiting for reviewer cadence. `READY_LIVE` remains `none`: this is deterministic local correctness/evidence work and creates no unresolved real-network question.
