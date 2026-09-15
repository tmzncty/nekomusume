# Reviewer R9 positive-P2 checkpoint — exact `ee21564`

**Reviewed source/test anchor:** `ee2156420cf98bb0638c18b56ad3f32077ede225`.

**Classification:** bounded reviewer/evidence checkpoint. This is not a security approval, release decision, WAN result, protocol freeze, or production claim.

## What changed

Exact `ee21564` changes only `crates/neko-cli/tests/probe.rs`. The runtime source remains the exact source inherited from `6e19a08`. The migration-back P2 fixture now unconditionally requires client success, client-side UDP recovery validation and migration-back, the post-return reliable send at offset 48, one Session DeliveryAck observation, one applied Carrier packet ACK observation, and a zero in-flight diagnostic somewhere in the client log.

GitHub-hosted checks on exact `ee21564` are green: `stable checks` SUCCESS and `nightly decode fuzz smoke` SUCCESS. These are cross-evidence only; no new persisted developer-local clean exact-tree provenance was added by this commit.

## Accepted progress

The old fully conditional `if client_log.contains("r9_udp_post_return_sent")` acceptance is gone. The fixture can no longer pass merely because post-return execution never happened: client success and the named client-side post-return milestones are now required.

This is useful positive progress and does not require reverting the `count=4` ownership partition or H-R9-019 replay-identity repair.

## R9-2 / P2 is still open

The current test does **not** yet satisfy the reviewer handoff's positive-P2 evidence contract:

1. `finish_server(server)` is still destructured into `_server_status` / `_server_log`; the test therefore does not require server success and does not inspect the server-side causal chain.
2. The fixture does not assert the exact TCP replay identity: offset 32 must be the sole TCP application replay and offsets 0, 16 and 48 must not be replayed over TCP.
3. It does not require server `udp_recovery_owner_started`, server `udp_recovery_validated`, authenticated Session acceptance of offset 48 before acknowledgement emission, `udp_return_delivery_ack_sent`, or `udp_return_packet_ack_sent`.
4. The source still has no dedicated `r9_udp_post_return_settled` terminal diagnostic. The test's broad `"remaining_in_flight":0` substring is not a post-return-specific evidence anchor. The runtime control flow currently waits for both logical completion and `rt.in_flight()==0` before returning success, so this is an evidence-boundary gap, not a reproduced runtime correctness defect.
5. No bounded post-return acknowledgement-order seam exists yet for both `Session DeliveryAck -> Carrier ACK` and `Carrier ACK -> Session DeliveryAck`.
6. The automatic-health exact replay-identity regression, P3 persistent malformed-budget sequence, and P4 incomplete-settlement negatives remain outstanding.

There is no new BLOCKER/HIGH in this bounded checkpoint. R9-3 remains dependency-blocked on completing R9-2 evidence and its final clean exact-tree provenance.

## Required continuation

Continue without waiting for reviewer cadence:

- finish positive P2 with both process exit statuses and exact server/client structured causal evidence;
- add one post-return terminal settlement event emitted only after exact logical confirmation and Recovery `in_flight==0`;
- add the two acknowledgement arrival orders through one bounded test-only seam;
- add the automatic replay exact-identity regression;
- execute P3 and P4;
- persist the developer-local clean exact-tree gate on the final pushed R9-2 source/test SHA;
- then continue the existing R9-3 through R9-12 and Q10/Q11/Q12 queue.

`READY_LIVE` remains none until the dependency chain reaches an independently reviewed cross-process reliable-UDP + real TCP promotion surface and Q11 creates a specific changed-hypothesis live question.
