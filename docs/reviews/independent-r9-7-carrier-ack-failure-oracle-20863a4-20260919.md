# Independent R9-7 review — H-R9-072 Carrier ACK failure-oracle gap

**Reviewed implementation/test anchor:** exact reachable pushed source/test commit `20863a4cf392dc0d6d978d424c5d21cd2fba4a57`.

**Classification:** HIGH — process/evidence acceptance-oracle correctness. The source-side H-R9-071 socket-outcome projection is materially repaired, but its required production-owner negative regression is not present. This is not a wire, ACK-architecture, crypto, Session-delivery, retention-capacity, or policy decision.

## Scope inspected

- developer-owned commits after the prior reviewer handoff: `31ef45aa94f08ea1f5b4a373a978cee47dedd097`, `20863a4cf392dc0d6d978d424c5d21cd2fba4a57`, and the handoff-only `5965b52bee48ad996c3d08bacadfb5acfe95d42e`;
- `crates/neko-cli/src/main.rs` initial reliable-UDP Carrier ACK owners and post-return normal/seam/delayed-reorder owners;
- `crates/neko-cli/tests/probe.rs` current R9 process tests, including malformed-bound, reversed-order, withheld-domain, ACK-loss/reorder, first-send socket failure, and future-ACK regressions;
- `docs/reviews/independent-r9-7-carrier-ack-send-evidence-fd41ea9-20260919.md` acceptance contract;
- current handoff and release/governance boundaries.

Reviewer-local execution is **not** claimed. The developer reports a clean exact-tree local gate for exact `20863a4`; this review treats it as developer-reported provenance, not reviewer execution. No decoder/parser/crypto-framing change is implicated.

## What is repaired

The source rule itself is now materially correct at the inspected owners: Carrier ACK positive diagnostics are selected from the actual `UdpSocket::send_to` result, with `Ok` -> `*_sent` and `Err` -> typed `*_send_failed`. The normal post-return owner also has a deterministic `--fail-r9-ack` seam after ACK construction/sealing.

That closes the original unconditional-positive projection defect at the source level.

## Concrete acceptance-oracle gap

The previous H-R9-071 review explicitly required a deterministic negative **process regression** on the real executable post-return Carrier ACK owner, plus paired normal-path control. Between the pre-repair reviewer handoff `59b5983eb5967b81c5b4667ceb4ae5c0d4b5efa2` and current `5965b52bee48ad996c3d08bacadfb5acfe95d42e`, the only changed implementation/test path is `crates/neko-cli/src/main.rs`; `crates/neko-cli/tests/probe.rs` did not change.

Therefore the new `--fail-r9-ack` seam is not exercised by a newly added regression, and the required discriminators are not repository-enforced:

- typed `udp_return_packet_ack_send_failed` on the injected target ACK;
- absence of positive `udp_return_packet_ack_sent` for that same failed packet identity;
- absence of a false H-R9-070 complete-dual-feedback premise when the Carrier half fails;
- no fabricated client settlement/final success from the failed ACK attempt;
- paired successful post-return ACK send still producing positive evidence and settlement.

There is an additional seam-coverage issue in the delayed/reordered branch: the delayed ACK release honors `--fail-r9-ack`, but the fresh current ACK immediately sent later in that same branch still calls `udp.send_to(...)` directly. Its positive event remains correctly conditional on the real socket result, so this is not the original false-positive source defect; however, the deterministic failure seam does not cover every executable delayed/reordered ACK owner as the H-R9-071 closure claim states.

A green full gate cannot substitute for this missing discriminator: without an injected owner-level regression, a later mutation that removes the rollback/failure classification, reintroduces premature positive evidence, or bypasses one branch can remain green if no ordinary socket error occurs.

## Required smallest repair

1. Keep the current socket-outcome source fix; do not redesign ACK, Recovery, Session, crypto/wire, D064, or any policy value.
2. Add a deterministic **process test** that reaches the real post-return Carrier ACK owner and injects an `Err` after ACK construction/sealing.
3. The test must bind the failed ACK by packet identity and prove:
   - typed `udp_return_packet_ack_send_failed` exists for that target;
   - no positive `udp_return_packet_ack_sent` exists for that same target;
   - Session DeliveryAck evidence alone does not satisfy complete dual feedback;
   - client does not report `r9_udp_post_return_settled`, `failover_client_ok`, final `summary`, or `ordered_records_complete` solely from the failed ACK attempt;
   - terminal/nonzero behavior, if current semantics require it for that injected operation, is asserted explicitly rather than inferred from an unrelated timeout/socket failure.
4. Preserve a paired ordinary positive-control process test: successful post-return Carrier ACK send emits `udp_return_packet_ack_sent`, binds the same packet identity as client send/retirement, and normal settlement remains green.
5. Make the deterministic seam cover the delayed/reordered post-return ACK owners as well. A small stage/one-shot selector is preferable to a blanket failure that also poisons earlier unrelated ACKs if that is needed to reach the intended owner deterministically.
6. Cover initial `udp_packet_ack_sent` evidence truth with focused deterministic coverage if it is not already mutation-sensitive; do not duplicate large fixtures merely for line coverage.
7. On the final pushed repair SHA run and persist:

```text
PYTHONDONTWRITEBYTECODE=1 bash scripts/check.sh
git diff --check
```

Record exact reachable pushed SHA, UTC start/end, exit codes, OS/arch, stable Rust version, and clean-tree state. Fuzz is not mechanically required unless decoder/parser/crypto framing changes.

## Queue consequence

H-R9-071 is accepted only for the **source-side socket-outcome projection**. R9-7 remains blocked on H-R9-072 until the production-owner failure oracle is repository-enforced. After closure, continue immediately through the remaining R9-7 process/result truth sweep and then the existing deep R9 queue; do not wait for reviewer cadence.
