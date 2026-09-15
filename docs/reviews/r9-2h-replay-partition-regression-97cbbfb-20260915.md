# R9-2H bounded recheck — exact `97cbbfb`

**Reviewed revision:** `97cbbfbc55adacf28724d37f38f8cfd98eb42a57`.
**Review scope:** H-R9-016 server replay-count repair, the current client/server reliable-UDP/TCP ownership partition, the two hosted stable-check failures, and the P2 migration-back evidence gate.
**Not in scope:** WAN/VPS execution, new retry/timing policy, Session/Carrier/ACK/wire architecture changes, D019, capacity/security values, RC/release authority.

## Verdict

- H-R9-016 is **partially repaired**: the server now uses the intended `uncertain_start`/`uncertain_end` ownership partition and no longer waits for a nonexistent TCP application record in `--reliable-udp + recovery/migration-back` (`count=3` => zero TCP replay expected).
- Exact `97cbbfb` is **not acceptable as a closure tree** because hosted `stable checks` fails in `scripts/check.sh`; nightly decode fuzz smoke passes and is only supplementary evidence.
- **H-R9-017 — HIGH correctness / exact-tree-gate blocker:** the H-R9-016 server repair exposed a pre-existing client-side ownership inconsistency in the non-automatic controlled fallback path. The server now expects only the current uncertain partition, while the client still sends `count - 1` TCP records whenever `automatic_health_failover == false`, replaying a record that was already reliable-UDP-owned and Session-confirmed. This causes the server to finish after its smaller expected replay set and the client to hit `BrokenPipe` on the extra TCP write.
- The previous reviewer H-R9-016 repair brief was incomplete for the non-automatic path. This review supersedes the implication that changing only the server was sufficient; the developer implemented the requested server formula correctly.
- P2 remains **OPEN**. Its current test is still conditional/vacuous with respect to positive post-return evidence and does not require client/server success.

## Reproduced hosted failure boundary

On exact `97cbbfb`, GitHub-hosted `stable checks` reaches `bash scripts/check.sh` and fails in `neko-cli` process tests:

1. `reliable_udp_failover_settles_packet_acks_to_zero_in_flight`
2. `reliable_udp_reversed_ack_order_confirms_in_order`

Both are `--reliable-udp` controlled-fallback runs without `--automatic-health-failover` and without migration/recovery. Their logs first prove the two reliable UDP records have separate Session confirmations and Carrier settlement reaches `remaining_in_flight=0`, then the client enters TCP fallback and panics on a `BrokenPipe` during the extra TCP replay. The nightly decode fuzz smoke passes; this does not override the stable/full-gate failure.

## Source argument for H-R9-017

Current ownership before TCP is already explicit on the client:

```text
uncertain_end   = recovery_enabled ? records.len() - 1 : records.len()
uncertain_start = reliable_udp ? 2 : 1
uncertain_count = saturating_sub(uncertain_end, uncertain_start)
```

The server at `97cbbfb` now uses the equivalent partition for `tcp_records`.

The client, however, chooses TCP replay using two different rules:

```text
automatic_health_failover == true  => failover.tcp_resend().len()
automatic_health_failover == false => count - 1
```

The second rule ignores reliable-UDP ownership. For `count=3`, `reliable_udp=true`, `recovery_enabled=false`:

```text
uncertain_start = 2
uncertain_end   = 3
server expected TCP replay = 1
client controlled-fallback TCP replay = 2   <-- wrong
```

The extra client replay includes offset 16 even though the same path has already classified records 0 and 1 as reliable-UDP-owned and required their exact logical confirmations before Carrier settlement. This is an ownership contradiction, not a socket timing problem.

## Required smallest repair

Do **not** revert the server formula and do not add sleeps/retries/deadlines. Make the client non-automatic replay count use the same already-committed ownership partition, preferably through one tiny shared calculation used by server expectation and client controlled fallback:

```text
uncertain_start = reliable_udp ? 2 : 1
uncertain_end   = recovery_enabled ? count - 1 : count
owned_tcp_replay_count = saturating_sub(uncertain_end, uncertain_start)
```

Then:

```text
automatic-health path: tcp_records = failover.tcp_resend().len()
controlled fallback:   tcp_records = owned_tcp_replay_count
```

For the automatic path, add a deterministic assertion/test that the tracked uncertain set agrees with the same ownership partition for the covered fixture modes; do not invent a second policy.

Pin the existing `count=3` mode table in a small deterministic test:

| reliable UDP | recovery/migration reservation | expected TCP replay |
|---|---|---:|
| false | false | 2 |
| false | true  | 1 |
| true  | false | 1 |
| true  | true  | 0 |

This is a current-semantics ownership repair. It changes no Session/Carrier/ACK/wire architecture and introduces no policy value.

## Immediate acceptance after H-R9-017

Before expanding R9 work, the developer must make the exact source/test tree green and at minimum rerun the two red process tests plus the repository local gate. The full final R9-2H provenance still belongs after P2-P4.

The repaired tree must preserve:

- P1 reversed logical ACK acceptance at exact structured identities and Recovery zero settlement;
- no TCP packet-ACK layer;
- no replay of reliable-UDP-owned, already logically confirmed offset 16 as uncertain TCP data;
- non-reliable legacy controlled fallback still replays the correct two records for `count=3`.

## P2 remains mandatory and non-conditional

After the tree is green, convert `reliable_udp_migration_back_reserves_final_record` from conditional evidence into positive acceptance:

1. require both client and server process success;
2. require server `udp_recovery_owner_started` before server/client `udp_recovery_validated` and migration-back promotion;
3. exactly one post-return reliable send for stream 1 / offset 32, never on the pre-promotion uncertain path;
4. promotion precedes reliable ownership/send;
5. accepted server Session receive for offset 32 precedes creation/emission of both Session DeliveryAck and Carrier ACK;
6. exact Session confirmation for offset 32 is applied;
7. independent post-return Carrier ACK is applied;
8. post-return Recovery terminates with `remaining_in_flight=0` before success;
9. exercise Session-ACK-first and Carrier-ACK-first using a bounded test-only ordering seam, not sleeps;
10. neither acknowledgement domain substitutes for the other.

Then continue directly through P3 persistent malformed-budget evidence, P4 incomplete-settlement negative, clean exact-tree provenance, and R9-3 onward without waiting for reviewer cadence.

## Evidence / release boundary

No WAN/VPS result follows from this local repair. `READY_LIVE: none`; item 3 and item 4 remain incomplete; `RELEASE_CANDIDATE=false`, `PRODUCTION_READY=false`, `FREEZE=false`, `RELEASED=false`.
