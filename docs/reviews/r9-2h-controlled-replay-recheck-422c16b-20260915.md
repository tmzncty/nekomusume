# R9-2H independent bounded recheck — controlled fallback replay ownership at `422c16b`

**Reviewed source/test revision:** exact reachable `422c16b106b5568cb87f81419b45941508c2d4a4` (`fix(cli): controlled-fallback TCP replay uses ownership partition (H-R9-017)`).

**Review scope:** the H-R9-017 client-side controlled TCP replay-count repair, its server/client ownership-partition consistency, current process-test evidence relevant to the previously red controlled reliable-UDP fallback path, and current governance/evidence boundaries. This is not a complete R9-2H acceptance, WAN experiment, release/security approval, or protocol freeze.

## Verdict

**H-R9-017 CLOSED for the reviewed source shape.** No new BLOCKER/HIGH was found in the one-line ownership repair itself.

The non-automatic client no longer replays `count - 1`; it now uses `uncertain_end.saturating_sub(uncertain_start)`, the same ownership partition already used by the server. Therefore reliable-UDP-owned records and a migration-back-reserved final record are excluded from controlled TCP replay. Automatic-health mode remains based on the actual `failover.tcp_resend()` tracked uncertain set and is not changed by this repair.

For the pinned `count=3` ownership table, the committed formulas yield the intended controlled replay cardinalities:

- no reliable UDP, no recovery reservation -> 2;
- no reliable UDP, recovery reservation -> 1;
- reliable UDP, no recovery reservation -> 1;
- reliable UDP, recovery reservation -> 0.

This closes the exact client/server replay-count drift that produced the extra reliable-owned offset and `BrokenPipe` after the server completed its smaller expected replay set.

## Cross-evidence

GitHub-hosted checks on exact `422c16b` are both green:

- `stable checks` -> success;
- `nightly decode fuzz smoke` -> success.

Hosted checks are only cross-evidence. No new developer-local exact-tree provenance file was added by `422c16b`; final R9-2H closure still requires the repository-prescribed clean exact pushed-tree `PYTHONDONTWRITEBYTECODE=1 bash scripts/check.sh`, `git diff --check`, clean initial/final tree and provenance record on the final source/test SHA.

## Remaining acceptance-critical front

R9-2H is **not closed**. P2 remains conditional/vacuous in the current test source: `reliable_udp_migration_back_reserves_final_record` always enforces the negative pre-promotion reservation invariant, but the positive post-return reliable-ownership/dual-settlement assertions run only when the log already contains `r9_udp_post_return_sent`. The test therefore does not yet prove that the built binary successfully reaches migration-back, sends the reserved offset 32 through reliable-UDP ownership, obtains both exact Session DeliveryAck and independent Carrier ACK, and reaches `remaining_in_flight=0` before success.

Next dependency-safe work is therefore the already-specified **positive P2** closure, followed by P3 persistent malformed-budget evidence, P4 incomplete-settlement terminal evidence, final exact-tree provenance, then R9-3 onward. No VPS/live run is READY yet.

## Boundaries preserved

- Session DeliveryAck and UDP Carrier packet ACK remain distinct evidence domains.
- No TCP packet-ACK layer is introduced.
- No Session/Carrier/ACK/wire architecture is changed by H-R9-017.
- No TTL/LRU/history-size/capacity/security value is selected.
- Item 3 and item 4 remain incomplete; release/production/freeze/released flags remain false.
- `READY_LIVE: none` remains the authoritative live classification until the cross-process R9 path is independently closed and Q11 creates a specific changed-hypothesis row.
