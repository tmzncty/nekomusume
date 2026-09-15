# Reviewer R9 P2 causal-evidence checkpoint — exact `2d3809e`

**Reviewed developer source/test revision:** `2d3809ef385c003692aa25b634cc5885535b4b91`

**Review scope:** the single developer-owned test-only commit after reviewer handoff `63ed73c`, current `reliable_udp_migration_back_reserves_final_record`, and the corresponding post-return server/client implementation in `crates/neko-cli/src/main.rs`. This is bounded reviewer/navigation evidence, not a release, security, WAN, performance, or protocol-freeze decision.

## Repository / CI truth

- `2d3809e` changes only `crates/neko-cli/tests/probe.rs` (+21/-2); no runtime source changed.
- GitHub-hosted `stable checks` completed successfully for exact `2d3809e`.
- GitHub-hosted `nightly decode fuzz smoke` completed successfully for exact `2d3809e`.
- Hosted CI remains cross-evidence only; final R9-2 developer-local clean exact-tree provenance is still a separate closure requirement.
- No open PR and no new VPS/WAN run were observed in this review.

## Accepted progress

1. The positive P2 fixture still requires **both client and server process success**.
2. The stale count comment is corrected to the actual count=4 ownership layout: reliable UDP owns offsets 0/16, TCP uncertain replay is offset 32, reserved post-return record is offset 48.
3. The fixture now at least checks presence of the server-side milestones `udp_recovery_owner_started`, `udp_recovery_validated`, `udp_return_delivery_ack_sent`, and `udp_return_packet_ack_sent`.
4. Current source order is consistent with the intended server causal chain: post-return authenticated `ProcessMessage::Data` must pass `SessionRuntime::receive`; the server then records the Carrier packet owner, sends the Session DeliveryAck, and only afterwards emits/sends the Carrier packet ACK. No source correctness contradiction was found in that bounded path.

## M-R9-009 — server P2 assertions are presence-only, not causal/exact evidence

**Severity:** MEDIUM evidence-integrity / regression-coverage gap. This does not by itself establish a runtime correctness defect.

The new assertions use independent `server_log.contains(...)` checks. They do **not** prove:

- exactly one occurrence of each milestone in this bounded run;
- `udp_recovery_owner_started < udp_recovery_validated < udp_return_delivery_ack_sent < udp_return_packet_ack_sent` in observed event order;
- that the post-return DeliveryAck is the reserved stream 1 / offset 48 record rather than merely an event with the right name;
- that the Carrier packet ACK event belongs to that same post-return owner.

The current server source makes the intended ordering plausible, but the stated P2 acceptance contract deliberately asks the built-binary fixture to pin the causal/evidence boundary so a later refactor cannot silently reorder or broaden it.

### Required smallest follow-up

Tighten the existing count=4 fixture rather than creating a parallel scenario:

1. collect exact server event lines for the four milestones and require one occurrence each;
2. assert their strict order;
3. require `udp_return_delivery_ack_sent` to identify the reserved record (`seq=3`, corresponding to offset 48 in this fixed 16-byte fixture); if current fields are insufficient, add only the smallest `stream`/`offset` diagnostic fields;
4. bind the packet-ACK milestone to the same bounded post-return branch using existing event ordering/uniqueness, or add the smallest contextual field if the current line cannot prove it.

Do not introduce a new protocol tag, ACK architecture, deadline, policy number, or second receive owner.

## M-R9-010 — the same P2 fixture still does not pin the full ownership chain

**Severity:** MEDIUM evidence-integrity / regression-coverage gap.

The current positive P2 assertions still leave several handoff requirements unpinned:

- **TCP replay ownership:** require exactly one client `tcp_delivery_ack_validated` for the uncertain record at offset 32 / fixed-fixture `seq=2`, and no replay confirmation for offsets 0, 16, or reserved 48. The server side should likewise observe exactly one corresponding `tcp_delivery_ack_sent` for that same logical record.
- **Recovery order:** require client `udp_recovery_challenge_sent` before client `udp_recovery_validated`, and validation before `udp_migrated_back`.
- **Post-return send cardinality:** require exactly one `r9_udp_post_return_sent` for offset 48, rather than only substring presence.
- **Logical confirmation identity:** `udp_return_delivery_ack_validated` currently lacks an explicit stream/offset assertion in the fixture. The terminal `r9_udp_post_return_settled` line already pins stream 1 / offset 48 / `remaining_in_flight=0`; either connect the confirmation to that exact record with existing structured fields/order or add the smallest diagnostic identity fields.

The existing exact terminal settlement assertion and `r9_udp_return_packet_ack ... applied=true` assertion are useful and should remain.

## Still-open R9-2 continuation

After exact P2 causal/identity closure, continue without reviewer wait:

1. acknowledgement-order seam: Session DeliveryAck -> Carrier ACK and Carrier ACK -> Session DeliveryAck must converge to the same terminal settlement;
2. automatic-health replay identity: compare actual `(DataId,payload)` controller ownership against the replayed records rather than trusting `.len()` reconstruction;
3. persistent malformed budget: `malformed #1 -> malformed #2 -> canonical Carrier ACK -> malformed #3` must hit the same operation-wide finite bound;
4. incomplete-settlement negative: suppress each acknowledgement domain separately and forbid any settled/success continuation while logical confirmation or Recovery ownership remains outstanding;
5. developer-local clean exact-tree `scripts/check.sh` + `git diff --check` provenance on the final pushed R9-2 source/test SHA.

Then proceed directly into the existing R9-3..R9-12 and Q10/Q11/Q12 queue. `READY_LIVE` remains none until the cross-process integration and its dedicated independent review make a specific changed-hypothesis WAN question truthful.
