# Independent R9-7 review — H-R9-073 post-return Carrier ACK target-oracle gap

**Reviewed implementation/test anchor:** exact reachable pushed source/test commit `d2e78a7beea34b0248b4964970d6f9a5a8b12fd2`, with current handoff-only head `6a57d926dcff2edbdadb8898e29a7714cbdd27ef`.

**Classification:** HIGH — process/evidence acceptance-oracle correctness. H-R9-071's source-side socket-outcome projection remains materially repaired, and `d2e78a7` adds real process coverage, but the new regression does not prove that its injected failure actually reaches the required post-return Carrier ACK owner. This is not a wire, ACK-architecture, Recovery, Session-delivery, crypto, D064, retention-capacity, or policy decision.

## Scope inspected

- developer-owned commit `d2e78a7beea34b0248b4964970d6f9a5a8b12fd2` and handoff-only closure `6a57d926dcff2edbdadb8898e29a7714cbdd27ef`;
- `crates/neko-cli/src/main.rs` initial reliable-UDP Carrier ACK owner plus post-return normal and delayed/reordered ACK owners;
- `crates/neko-cli/tests/probe.rs::reliable_udp_carrier_ack_send_failure_is_typed_not_sent`;
- prior H-R9-072 acceptance contract in `docs/reviews/independent-r9-7-carrier-ack-failure-oracle-20863a4-20260919.md`;
- current `README.md`, `AGENTS.md`, `ROADMAP.md`, `IMPLEMENTATION_PLAN.md`, `SECURITY.md`, `docs/status.md`, standing VPS authorization/priority, decisions, carrier architecture, Session v0 entry point, release/security packet, and current handoff.

Reviewer-local execution is **not** claimed. Developer-reported exact-tree provenance for `d2e78a7` is retained separately. GitHub-hosted Rust CI run `35427731874` completed `success` for exact `d2e78a7`; hosted CI is cross-evidence only and cannot repair a non-discriminating oracle.

## What `d2e78a7` does repair

The delayed/reordered fresh post-return ACK owner now honors the injected failure seam and projects the actual send outcome as `udp_return_packet_ack_sent` vs `udp_return_packet_ack_send_failed`. The new process test also exercises a real executable server/client fixture rather than a helper-only primitive.

Those are valid improvements and H-R9-071's source-side socket-outcome rule remains accepted.

## Concrete acceptance-oracle gap

The new regression launches the server with a global `--fail-r9-ack` flag. That same flag is consumed by the **initial** reliable-UDP Carrier ACK owner as well as the post-return owners. The test then explicitly accepts either

```text
udp_packet_ack_send_failed
OR
udp_return_packet_ack_send_failed
```

and its own comment says `Typed send-failure emitted (initial or post-return owner)`.

This directly contradicts the H-R9-072 closure requirement that the regression deterministically reach the **real post-return Carrier ACK owner**. The test does not require a post-return failure event at all.

The remaining assertions do not recover that missing precondition:

- they globally forbid both initial and post-return positive ACK events rather than proving a successful initial ACK path followed by a targeted post-return failure;
- `srv_status` is intentionally discarded;
- no server-side post-return progression evidence such as `udp_recovery_validated` / `udp_return_delivery_ack_sent` is required before the asserted failure;
- no target `packet_number` is extracted and matched against the failed post-return ACK;
- the test only checks absence of `r9_udp_return_packet_ack_applied`, `r9_udp_post_return_settled`, and `failover_client_ok`; it does not prove those absences are caused by the intended post-return socket failure rather than an earlier poisoned ACK path or unrelated terminal condition.

Therefore a mutation/implementation that fails only an early initial Carrier ACK, never reaches the post-return ACK owner, and exits without settlement can still satisfy the current regression. The process test is real, but it is not target-discriminating.

This is exactly the false-green class the prior H-R9-072 review required the stage/one-shot seam to avoid.

## Required smallest repair

1. Preserve the current truthful socket-outcome projection and all current ACK ranges, Recovery, Session, crypto/wire, D064, malformed budget, and policy values.
2. Introduce the smallest test-only stage/one-shot selector needed so the negative fixture can keep the **initial reliable-UDP Carrier ACK path successful** and inject `Err` only at a chosen post-return Carrier ACK owner. A dedicated post-return-only flag is sufficient; do not build a general fault-injection framework.
3. The negative process regression must establish the intended owner was reached before accepting the result:
   - at least one initial reliable-UDP `udp_packet_ack_sent` positive control occurred before the injected stage;
   - migration-back/post-return processing reached the server owner;
   - the Session-side post-return feedback path is independently observed, e.g. `udp_return_delivery_ack_sent`, so the missing Carrier half is the discriminating condition;
   - a typed `udp_return_packet_ack_send_failed` exists for the intended post-return packet identity;
   - no `udp_return_packet_ack_sent` exists for that same packet identity;
   - the client does not apply Carrier retirement for that failed attempt.
4. Do **not** over-specify the whole operation as terminal if current Recovery semantics legitimately recover from a single lost/failed ACK via retransmission. What must be forbidden is success evidence attributable to the failed ACK attempt itself. If the operation later recovers through a distinct successful retransmission/ACK, assert that recovery path explicitly and keep packet identities distinct. If the selected injected mode is intentionally persistent/terminal, then assert the exact nonzero/terminal cause and forbid final success explicitly.
5. Preserve a paired normal positive control proving the same post-return owner emits `udp_return_packet_ack_sent` on socket `Ok` and settlement/retirement remains correct.
6. Cover the delayed/reordered owner with the same stage-aware principle: delayed-original and fresh-current ACK attempts must be independently attributable; one branch's failure must not masquerade as coverage of the other.
7. Keep initial-owner socket-evidence truth separately mutation-sensitive if needed, but do not use an initial failure as proof of the post-return oracle.
8. On the final pushed repair SHA run and persist:

```text
PYTHONDONTWRITEBYTECODE=1 bash scripts/check.sh
git diff --check
```

Record exact reachable pushed SHA, UTC start/end, exit codes, OS/arch, stable Rust version, and clean-tree state. Fuzz is not mechanically required unless decoder/parser/crypto framing changes.

## Queue consequence

H-R9-072 cannot yet be accepted as a post-return production-owner closure. Treat `d2e78a7` as useful partial process-oracle progress and reopen the remaining target-attribution gap as H-R9-073. R9-7 stays blocked on this HIGH. After repair, continue immediately through the remaining R9-7 process/result truth sweep and the existing deep R9 queue without waiting for reviewer cadence.

`READY_LIVE` remains `none`: this is deterministic local correctness/evidence work and creates no unresolved real-network question.
