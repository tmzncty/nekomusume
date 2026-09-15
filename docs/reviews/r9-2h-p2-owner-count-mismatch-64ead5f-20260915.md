# R9-2H P2 independent bounded recheck — exact `64ead5f`

**Reviewed revision:** `64ead5fb157d7c96da9994e593e8ef67b24a881f`.
**Review scope:** the reverted P2 recovery-retry experiment, current `failover-client`/`failover-server` ownership arithmetic, and `reliable_udp_migration_back_reserves_final_record`.
**Not in scope:** WAN execution, new retry policy, Session/Carrier/ACK architecture changes, D019, capacity policy, RC/release authority.

## Verdict

- `64ead5f` **ACCEPT** as disposition of M-R9-009: it exactly removes the guessed 500 ms / six-attempt production retry introduced by `802ef95`. No new retry semantics are accepted.
- P2 remains **OPEN**, but the current stall is now mechanically explained by a cross-process ownership-count mismatch rather than missing UDP retransmission.
- **H-R9-016 — HIGH correctness/evidence:** under `--reliable-udp + --migration-back`, the client has zero TCP replay records while the server waits for one TCP application record before starting the UDP recovery owner. The server therefore blocks on a frame the client will never send, so `udp_recovery_owner_started` is reached too late or not at all within the client recovery wait.

## Exact source argument

For the P2 fixture (`count=3`, `reliable_udp=true`, `recovery_enabled=true`):

1. Client ownership bounds use:
   - `uncertain_end = records.len() - 1 = 2` because the final record is reserved for post-migration UDP;
   - `uncertain_start = 2` because records 0 and 1 are reliable-UDP owned;
   - therefore `uncertain_count = uncertain_end - uncertain_start = 0` and `FailoverController::track_uncertain` receives no record.
2. After TCP promotion the client sets `tcp_records = failover.tcp_resend().unwrap().len()`, therefore the client sends **0** TCP application Data records in this mode.
3. The server independently computes `tcp_records = count.saturating_sub(if recovery_enabled { 2 } else { 1 })`, therefore with `count=3` it waits for **1** TCP application Data record.
4. The server enters `udp_recovery_owner_started` only **after** that TCP Data loop.

Thus P2 has a deterministic producer/consumer count mismatch: client 0, server 1. A UDP recovery challenge can already be queued on the bound UDP socket, but the server never reaches its recovery receive owner because it is blocked waiting for nonexistent TCP application Data. This explains why resending the same authenticated recovery ciphertext did not provide a principled fix.

## Required smallest repair

Do not add retries, sleeps, new deadlines, capacity values, or protocol messages. Make the server's expected TCP replay count derive from the same existing ownership partition as the client:

```text
uncertain_end   = recovery_enabled ? count - 1 : count
uncertain_start = reliable_udp ? 2 : 1
tcp_records     = saturating_sub(uncertain_end, uncertain_start)
```

Equivalent factoring into a small shared/helper calculation is preferred if it prevents future drift, but do not create a generalized framework.

The deterministic mode table should be pinned:

| reliable UDP | migration/recovery | count=3 expected TCP replay |
|---|---|---:|
| false | false | 2 |
| false | true  | 1 |
| true  | false | 1 |
| true  | true  | 0 |

This is not a new Session/Carrier semantic decision; it reconciles the server with the already-committed client ownership boundaries.

## P2 positive acceptance after repair

`reliable_udp_migration_back_reserves_final_record` must stop being conditional:

- require both client and server process success;
- require `udp_recovery_owner_started`, server `udp_recovery_validated`, client `udp_recovery_validated`, successful migration-back, and exactly one `r9_udp_post_return_sent` for stream 1 / offset 32;
- prove promotion precedes post-return reliable ownership/send;
- prove accepted server Session receive precedes creation/emission of both Session DeliveryAck and Carrier ACK;
- prove exact Session confirmation for offset 32 and independent Carrier ACK application;
- require a post-return-scoped terminal Recovery result with `remaining_in_flight=0`;
- run both Session-ACK-first and Carrier-ACK-first using a bounded test-only send-order seam, not timing sleeps;
- preserve the negative invariant that reserved offset 32 never uses the legacy uncertain pre-promotion path.

After positive P2, continue immediately to P3/P4 and the exact-tree local provenance gate; do not wait for reviewer cadence.

## Evidence truth

Exact `64ead5f` hosted `stable checks` and `nightly decode fuzz smoke` are green. These are supplementary hosted checks, not a substitute for the required developer-local clean exact-tree gate after the P2-P4 source/test closure.

No VPS/WAN result follows. `READY_LIVE: none`, item 3 incomplete, item 4 incomplete, and all release/production/freeze flags remain unchanged.
