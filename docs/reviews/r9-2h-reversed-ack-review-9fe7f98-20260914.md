# R9-2H reversed-order Session DeliveryAck review — exact `9fe7f98`

**Reviewed source/test revision:** `9fe7f982aa254d6550fbd191b70268a719b985c9` (`fix(cli): R9-2 reversed-order ACK handling + --reverse-ack-order seam (M-R9-008 partial)`).

**Parent reviewer handoff:** `9f698de2e996076a69c77e9a469b00833ae5b741`.

**Review type:** bounded correctness/evidence review of the new reversed-order logical-ACK seam and its interaction with the existing `SessionRuntime::delivery_ack` contract. This is not a cryptographic audit, WAN run, release approval, or protocol freeze.

## Repository/check truth

- `9fe7f98` is exactly one source commit ahead of `9f698de`; the only changed file is `crates/neko-cli/src/main.rs` (+57/-11). No process-test file or local-provenance file changed in this commit.
- GitHub-hosted `stable checks`: **success** on exact `9fe7f98`.
- GitHub-hosted `nightly decode fuzz smoke`: **success** on exact `9fe7f98`.
- Hosted checks are supplementary cross-evidence only. Developer-local exact-tree provenance for the eventual R9-2H closure tree is still required.
- Open PRs at review time: none.

## Accepted partial progress

The server now has a bounded test-only `--reverse-ack-order` seam that can defer the first reliable-owned logical record's Session DeliveryAck and emit the second record's ACK first. This is the right kind of discriminating fault injection for P1 and does not by itself change wire grammar or crypto framing.

The client also no longer panics immediately when the earlier ACK arrives after the later ACK advanced the Session watermark. That prevents one prior failure mode, but the current treatment is not a correct P1 closure.

## HIGH — H-R9-011: later exact DeliveryAck can manufacture earlier Session confirmation

### Current behavior

`SessionRuntime::delivery_ack(stream, offset, len, ...)` is watermark-based:

```text
end = offset + len
current = confirmed_watermark(stream)
delta = end - current
if delta <= send_inflight: release `delta`, set confirmed=end
```

It does **not** require `offset == current` before advancing the watermark.

For two 16-byte records on the same stream:

```text
record 0: offset 0,  len 16
record 1: offset 16, len 16
current confirmed watermark = 0
send_inflight = 32
```

If record 1's exact Session DeliveryAck arrives first, current `delivery_ack(16,16)` computes `end=32`, `delta=32`, accepts it, releases all 32 in-flight bytes and advances the confirmed watermark to 32. That is a cumulative confirmation of record 0 despite no record-0 DeliveryAck having arrived.

When record 0's ACK arrives later, `end=16 < current=32`, so `SessionRuntime::delivery_ack` returns `RuntimeError::Protocol`.

The new CLI branch treats that `Protocol` as proof the earlier range is already covered and `continue`s. Meanwhile `recv_udp_delivery_ack` has already removed the matching record from `outstanding` before the caller applies Session confirmation. The outstanding set can therefore become empty even though the exact two logical ranges were not independently confirmed in the evidence order required by R9-2H P1.

### Why this is HIGH

R9-2H P1 explicitly requires:

- both exact `(stream, offset, len)` logical ranges confirm exactly once;
- order reversal must not lose/swallow the later record **or manufacture confirmation for the earlier record**;
- Carrier packet ACK remains separate from Session confirmation.

The current source violates the second requirement and can over-promote Session delivery evidence. This is mechanically repairable and does not require a maintainer policy decision, so R9-3 remains blocked on R9-2H rather than escalating to architecture approval.

## Required smallest repair — keep Session core semantics unchanged

Do **not** redefine `SessionRuntime::delivery_ack`, wire grammar, or ACK architecture in this repair.

Instead, make the reliable CLI receive owner retain a **bounded pending logical-ACK set** whose maximum cardinality is the already-bounded reliable-owned `outstanding` set.

Required ownership/order contract:

1. Authentication/demux may recognize an exact Session DeliveryAck in any arrival order.
2. Recognition of an out-of-order ACK records it as pending evidence; it does **not** yet advance `SessionRuntime` confirmation and does not remove its logical range as confirmed.
3. Only a pending ACK whose exact `offset == SessionRuntime::confirmed_watermark(stream)` may be applied to `delivery_ack`.
4. After one exact contiguous ACK applies successfully, drain any now-contiguous pending ACK(s) in order.
5. Remove an outstanding logical record only after its exact ACK has been successfully applied by SessionRuntime. Do not use `RuntimeError::Protocol` as a generic success/covered signal.
6. Pending ACK storage must be bounded by the original outstanding reliable-owned records; duplicates/stale/unmatched ACKs stay typed bounded negatives and consume the existing operation-wide malformed policy where applicable. Do not invent a new capacity value.
7. Carrier packet ACK handling remains independent and must not advance Session confirmation.

This preserves the current Session implementation while making the integration layer respect exact ACK evidence.

## P1 process regression still required

`9fe7f98` changed only `main.rs`; it did **not** add the required built-binary P1 regression. The eventual test must run real `failover-server` / `failover-client` with the reversed-order seam and prove at minimum:

- record 1 ACK is observed before record 0 ACK;
- record 1 is buffered, not used to confirm record 0;
- record 0 exact confirmation applies once;
- then record 1 exact confirmation applies once;
- final logical outstanding set is empty only after both exact confirmations;
- Carrier packet ACK remains Carrier-local;
- Recovery settles to zero on the successful run;
- no duplicate/conflict application delivery is created.

Prefer exact offset-bearing client diagnostics so the test discriminates the two logical confirmations instead of inferring them from counts.

## Remaining R9-2H closure work

After H-R9-011/P1 is repaired, the previous handoff's other discriminating process requirements remain live unless a later reachable source/test commit proves them:

- **P2:** `--reliable-udp + migration-back` reserved final record is neither Recovery-tracked nor legacy UDP-sent before post-promotion ownership.
- **P3:** operation-wide malformed budget survives `malformed -> malformed -> canonical Carrier ACK -> malformed` and fails on the third malformed input.
- **P4:** `reliable_udp_incomplete_settlement_fails_not_settled` remains green.
- Final pushed source/test tree needs developer-local `PYTHONDONTWRITEBYTECODE=1 bash scripts/check.sh`, `git diff --check`, clean initial/final tree, exact SHA, UTC start/end, OS/arch and Rust stable provenance.

No wire decoder/parser/crypto framing change is required for H-R9-011; do not mechanically add a fuzz obligation solely for this CLI ownership repair.

## Verdict

**ACCEPT partial seam progress; REJECT R9-2H/P1 closure at exact `9fe7f98`.**

Open correctness finding: **H-R9-011 HIGH**.

R9-3 remains blocked until the exact-confirmation ordering bug, P1-P3 process evidence and final local provenance are closed on one reachable source/test tree. The existing deep R9-3..R9-12 and Q10/Q11/Q12 queue remains valid behind this front; do not idle after R9-2H closes.
