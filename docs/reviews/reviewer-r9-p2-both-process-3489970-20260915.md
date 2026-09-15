# Reviewer R9 P2 both-process checkpoint — exact `3489970`

**Reviewed source/test anchor:** `348997060579336ba846125ee0f9aadaa8accd97`.

**Classification:** bounded reviewer/evidence checkpoint. This is not a security approval, release decision, WAN result, protocol freeze, or production claim.

## Repository truth reviewed

- Exact `3489970` is the latest developer-owned source/test commit at this checkpoint.
- It changes only `crates/neko-cli/src/main.rs` and `crates/neko-cli/tests/probe.rs`.
- GitHub-hosted Rust CI for exact `3489970` completed successfully: both `stable checks` (`bash scripts/check.sh`) and `nightly decode fuzz smoke` passed. Hosted CI remains cross-evidence only and does not replace the final developer-local clean exact-tree provenance required at R9-2 closure.
- Open PRs: none.
- No new WAN/VPS experiment was introduced by this commit.
- `READY_LIVE: none`; release items 3 and 4 remain incomplete; release/freeze/production flags remain false.

## Accepted progress — do not revert

### M-R9-008 reliable-settlement scope correction — CLOSED

`r9_udp_post_return_settled` is now emitted only when a `ReliableUdpRuntime` owner exists. The prior `map_or(0, ...)` shape could make owner absence look like a successful zero-in-flight reliable settlement; exact `3489970` removes that ambiguity without changing Session/Carrier/ACK semantics.

### Positive P2 process completion — ACCEPT

The existing `reliable_udp_migration_back_reserves_final_record` fixture now requires both client and server exit success from the same bounded count=4 migration-back run. This closes the previous vacuous-success allowance where server status was discarded and client nonzero exit could still pass the evidence fixture.

The current source path also still preserves the post-return ordering invariant: the server calls `SessionRuntime::receive` successfully before it constructs/sends the Session DeliveryAck and before it registers/polls/sends the Carrier packet ACK; on the client, the dedicated `r9_udp_post_return_settled` event remains after exact logical retirement and `ReliableUdpRuntime::in_flight()==0`.

## Evidence claim correction — P2 is still not closed

The commit message says that the test "asserts the server-side causal chain (`udp_recovery_owner_started`, `udp_recovery_validated`, `udp_return_delivery_ack_sent`, `udp_return_packet_ack_sent`)". The current test does **not** inspect `server_log` for those milestones. It only requires `server_status.success()` and then continues with client-log assertions. Therefore that commit-message sentence is not executable evidence and must not be used to close the P2 causal chain.

This is an evidence-integrity gap, not a reproduced transport correctness defect. Do not rewrite the historical commit message; close the gap in the existing fixture.

## READY_LOCAL first action — finish exact P2 causal evidence in the existing fixture

Tighten `reliable_udp_migration_back_reserves_final_record`; do not create a parallel scenario. From the same count=4 run require:

1. client success and server success (already present);
2. exactly one TCP application replay for stream 1 / offset 32, with no TCP replay of offsets 0, 16, or reserved 48;
3. client `udp_recovery_challenge_sent` before client `udp_recovery_validated` and `udp_migrated_back`;
4. server `udp_recovery_owner_started` before server `udp_recovery_validated`;
5. server post-return Session receive success for stream 1 / offset 48 is the gating branch for acknowledgement emission;
6. server `udp_return_delivery_ack_sent` for offset 48 before `udp_return_packet_ack_sent` for that post-return owner;
7. exactly one client `r9_udp_post_return_sent` for stream 1 / offset 48;
8. exact client Session confirmation for offset 48;
9. client `r9_udp_return_packet_ack` with `applied=true`;
10. client `r9_udp_post_return_settled` for stream 1 / offset 48 / `remaining_in_flight=0`.

Use exact structured-event lines and order. Existing TCP replay diagnostics currently expose positional `seq` rather than the full logical identity; add the smallest `stream`/`offset` fields to the existing `tcp_delivery_ack_validated` / `tcp_delivery_ack_sent` events if needed so the test proves ownership rather than inferring it from cardinality. Likewise, add the smallest identity field to an existing post-return event when necessary; do not invent a redundant evidence channel. The server ACK-sent branch is already source-gated on successful `SessionRuntime::receive`, so a separate duplicate "data accepted" event is unnecessary unless the current event fields cannot make the causal branch unambiguous.

Correct the stale P2 test comment that says "With 3 records" while the fixture uses count=4.

## Continue R9-2 immediately after P2 tightening

1. **Post-return ACK-order seam:** exercise both `Session DeliveryAck -> Carrier ACK` and `Carrier ACK -> Session DeliveryAck` through the same bounded owner and require the same terminal settlement event.
2. **Automatic-health replay exact-identity regression:** compare actual replay identities/payloads with exact `FailoverController::tcp_resend()` ownership. Current code still reduces `tcp_resend()` to `.len()` before reconstructing a contiguous slice; prove exact equivalence or consume the exact returned ownership.
3. **P3 persistent malformed budget:** one operation must drive `malformed #1 -> malformed #2 -> canonical Carrier ACK -> malformed #3`; malformed #3 must hit the existing bound and the valid ACK must not reset it.
4. **P4 incomplete settlement negatives:** suppress each acknowledgement domain separately; require typed nonzero termination and no settled-success event while logical or Carrier ownership remains outstanding.
5. **Final R9-2 exact-tree local provenance:** on the final pushed source/test SHA run `PYTHONDONTWRITEBYTECODE=1 bash scripts/check.sh`, `git diff --check`, verify clean initial/final tree, and persist exact SHA, UTC start/end, Linux OS/arch, stable Rust version, exit codes and clean-tree state. Run pinned decoder fuzz only if decoder/parser/crypto framing changed.

Then continue R9-3 through R9-12 and Q10/Q11/Q12 without waiting for reviewer cadence.

## No new BLOCKER/HIGH in this bounded checkpoint

No reproduced correctness/security BLOCKER/HIGH was found in the exact `3489970` delta. R9-3 remains dependency-blocked on finishing the R9-2 evidence/negative matrix and exact-tree local provenance, not on WAN permission.
