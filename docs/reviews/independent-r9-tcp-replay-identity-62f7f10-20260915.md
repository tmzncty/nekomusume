# Independent bounded R9 TCP replay identity review — exact `62f7f10`

**Reviewed tree:** `62f7f108ff28f171675e773eded2f9644f32f57f` (`main`).

**Latest source/test ancestor relevant to this finding:** `422c16b106b5568cb87f81419b45941508c2d4a4` (`fix(cli): controlled-fallback TCP replay uses ownership partition (H-R9-017)`). Later commits through the reviewed tree are review/handoff documentation only.

**Scope:** the cross-process failover client's transition from accepted uncertain ownership into TCP replay, especially the current `reliable_udp + migration_back` P2 shape. This is a bounded correctness/evidence review, not a release/security approval and not a policy decision.

## Verdict

**HIGH — H-R9-019: replay cardinality now follows the ownership partition, but replay identity still does not.**

The client correctly computes:

```text
uncertain_start = reliable_udp ? 2 : 1
uncertain_end   = recovery_enabled ? count - 1 : count
controlled_tcp_replay = uncertain_end - uncertain_start
```

and the server now expects the same replay count. However the actual client send loop still selects records positionally:

```rust
for record in records.into_iter().skip(1).take(tcp_records) {
    ... send record over TCP ...
}
```

It does **not** start at `uncertain_start`, and in automatic-health mode it uses only `failover.tcp_resend().unwrap().len()` while discarding the exact `(DataId, payload)` ownership returned by `tcp_resend()`.

Therefore the count can be correct while the identity is wrong.

### Deterministic P2 example

The current handoff correctly proposes changing the positive migration-back fixture to `count=4`, `bytes=16`, `reliable_udp=true`, `recovery_enabled=true`. Under the committed ownership partition:

```text
records[0] offset 0  -> reliable UDP owned
records[1] offset 16 -> reliable UDP owned
records[2] offset 32 -> uncertain; the one TCP replay
records[3] offset 48 -> reserved for post-migration reliable UDP
uncertain_start = 2
uncertain_end   = 3
tcp_records     = 1
```

But the current `skip(1).take(1)` loop sends `records[1]` / offset 16 over TCP, not `records[2]` / offset 32. The real uncertain record is never replayed, while an already reliable-UDP-owned record is replayed across the Carrier boundary.

So changing P2 from count 3 to count 4 **without first repairing replay selection cannot produce valid positive P2 evidence**.

### Automatic-health mode has the same ownership problem

`FailoverController::tcp_resend()` already returns the exact uncertain ownership as `Vec<(DataId, Vec<u8>)>`. The client currently reduces that exact set to `.len()` and then chooses the first positional records after index 0. A future exact uncertain subset that is not that positional prefix can therefore be replaced with unrelated logical records while retaining the same cardinality.

This contradicts the existing Session/Carrier ownership boundary: TCP replay must carry the exact uncertain logical ranges selected by failover state, not an equal-sized positional approximation.

## Smallest repair contract

Do not change Session, Carrier, ACK, wire, timing or policy semantics. Repair only replay selection.

1. Build an explicit bounded `tcp_replay_records` sequence before the TCP send loop.
2. In controlled/non-automatic mode, select exactly `records[uncertain_start..uncertain_end]` (or the equivalent iterator using `skip(uncertain_start).take(uncertain_count)`).
3. In automatic-health mode, preserve the exact identities returned by `failover.tcp_resend()` rather than using only its length. Map each returned `DataId` back to exactly one original logical record and require payload equality; missing, duplicate or payload-mismatched ownership must fail closed.
4. Iterate the resulting exact replay sequence. Do not fall back to `skip(1)` or another guessed prefix.
5. Preserve the reserved final record exclusion and the already-closed H-R9-017 replay-count partition.

No new TTL/LRU/capacity/security value is required.

## Required focused regressions

Before advancing P2:

- **P2 identity:** with `count=4`, reliable UDP + migration-back, assert exactly one TCP replay and that it is offset 32; offsets 0, 16 and reserved 48 must not appear as TCP replay.
- **Controlled reliable fallback identity:** cover a non-migration reliable case where `uncertain_start=2`; the first TCP replay must be the record at index 2, not index 1.
- **Automatic-health identity:** arrange an exact tracked uncertain subset and assert the TCP Data records correspond to the exact `tcp_resend()` DataIds/payloads, not merely an equal count.

After this repair is green, continue the existing P2 positive evidence contract (`count=4`), then P3/P4 and the preserved R9 queue.

## Evidence / exclusions

Reviewed exact current `crates/neko-cli/src/main.rs` ownership calculation and TCP replay loop, current positive P2 fixture, and the existing FailoverController `tcp_resend()` exact-ownership API. Current hosted `stable checks` and `nightly decode fuzz smoke` on exact `62f7f10` are green, so this is not a red-tree or CI availability blocker.

Not performed: code repair, WAN/VPS execution, performance/capacity testing, cryptanalysis, D019 or other policy decisions.
