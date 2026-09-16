# Reviewer R9 P3 applied-evidence checkpoint — exact `966eb49`

**Reviewed exact tree:** `966eb49775296a46d59e6326404ea012b29eea7b`

**Scope:** latest developer-owned R9-2 repair after the prior reviewer handoff: exact `966eb49` (`fix(cli): P3 malformed->CarrierACK->malformed ordering + typed bound evidence (H-R9-021)`), plus the current reliable receive/demux owner and built-binary P3 regression. This is a bounded reviewer checkpoint, not a security approval, release decision, protocol freeze, WAN result, or production claim.

## Current tree / CI facts

- `main` is exact `966eb49775296a46d59e6326404ea012b29eea7b` at review time.
- Hosted Rust CI succeeds on exact `966eb49`: `stable checks` runs `bash scripts/check.sh` successfully, and `nightly decode fuzz smoke` also succeeds. Hosted CI remains cross-evidence only.
- Open PRs: none.
- No new WAN/VPS experiment is present in this sequence.
- Developer-local final R9-2 clean exact-tree provenance is still absent.

## Accepted progress — H-R9-021 source ordering repair

The server-side test seam now defers the already-canonical Carrier packet ACK under `--malformed-budget-test` and emits datagrams in the intended source order:

```text
malformed #1
malformed #2
Carrier packet ACK
Session DeliveryAck
malformed #3
```

The ordinary non-test path still sends the Carrier ACK immediately. This closes the **server source-ordering** part of H-R9-021. The Session DeliveryAck remains a separate evidence domain and is not being used as the Carrier ACK substitute.

## H-R9-022 — P3 still does not prove the Carrier ACK was applied between malformed #2 and #3

**Severity: HIGH for R9-2 evidence correctness. Mechanically repairable under current semantics; no maintainer policy or architecture decision required.**

The current built-binary regression still cannot distinguish the required success condition from an invalid/rejected/missed Carrier ACK:

1. The test discards the server result/log (`let (_st, _sl) = finish_server(server);`), so it does not mechanically assert the server Carrier-ACK milestone/order.
2. On the client, `recv_udp_delivery_ack` returns `UdpAcknowledgement::Carrier { applied }`, but the outer reliable receive loop only increments `packet_ack_applied` / `packet_ack_rejected`. The applied counter is emitted later in `r9_udp_packet_ack_outcomes`, after logical-confirmation settlement. P3 intentionally terminates on malformed #3 before reaching that later output, so no client-visible event proves `applied=true` occurred between malformed #2 and #3.
3. Therefore the P3 test would still pass if the interleaved packet ACK were decoded but rejected (`applied=false`), or if the acceptance path regressed so the required valid Carrier feedback were never applied, as long as malformed #3 still produced a terminal error.
4. The terminal assertion is still broad (`client_err.contains("malformed") || client_err.contains("bound")`) rather than pinning the existing malformed-budget terminal error. A different malformed failure could satisfy it.
5. The test forbids two settled markers, but does not mechanically forbid downstream health/failover continuation as required by the current R9-2 acceptance contract.

This means exact `966eb49` repairs the send-order implementation but does **not** yet prove the actual invariant P3 exists to challenge: the operation-wide malformed counter survives one **successfully applied** valid Carrier ACK between malformed #2 and malformed #3.

## Required smallest repair

Do not change `MAX_POST_HANDSHAKE_MALFORMED`; do not redesign Session/Carrier/ACK/wire semantics; do not create a second receive owner.

1. Keep the server ordering from `966eb49` unchanged.
2. Add the smallest client diagnostic at the point where the existing outer receive owner handles `UdpAcknowledgement::Carrier { applied }`. For P3 it must expose enough to prove:
   - `applied=true`;
   - the persistent operation-wide malformed counter is already exactly `2` when that Carrier ACK is handled.
   A diagnostic-only field/event is sufficient; this is not protocol evidence and must not change transport semantics.
3. Tighten the built-binary P3 regression so it requires exactly one interleaved Carrier-ACK applied event with `malformed_seen=2` (or equivalent exact fields), then requires the existing exact malformed-budget terminal error on the next malformed input.
4. Require the client to exit nonzero unconditionally and require the specific existing error text `UDP delivery acknowledgement malformed bound exceeded` (or the exact stable terminal marker the implementation already emits). Do not accept a generic `malformed || bound` disjunction.
5. Require no rejected Carrier-ACK outcome for the selected interleaved ACK.
6. Require no `r9_udp_in_flight_settled`, no `r9_udp_post_return_settled`, and no downstream health/failover/migration continuation after the malformed bound terminates the operation.
7. Run the focused built-binary regression, commit/push, then continue immediately to the still-open R9-2 queue. No decoder/framing fuzz is required solely for this diagnostic/test repair; the final source/test tree still needs the ordinary clean exact-tree local gate before R9-2 closure.

## Preserved accepted progress

- H-R9-020 failover accounting remains closed absent a contradictory reproducer.
- The count=4 P2 fixture and its existing causal milestones remain useful but exact C1-C4 closure is still open.
- Both P4 acknowledgement-domain suppression seams remain useful partial evidence and must be tightened before R9-2 closure.
- Earlier candidates A/B remain closed on the current tree: future/never-sent ACK rejection is atomic, and mixed datagram drop reasons remain separated.

## Remaining dependency-ordered R9-2 work

After H-R9-022:

1. exact P2 C1-C4 identity/cardinality/order closure in the existing count=4 fixture;
2. post-return Session-ACK -> Carrier-ACK and Carrier-ACK -> Session-ACK arrival-order seam using the same bounded owner/deadline;
3. automatic-health replay exact-identity challenge against `FailoverController::tcp_resend()` ownership;
4. tighten both P4 suppression cases so they explicitly forbid all success/continuation evidence;
5. persist developer-local clean exact-tree provenance on the final pushed R9-2 SHA with `PYTHONDONTWRITEBYTECODE=1 bash scripts/check.sh`, `git diff --check`, exact SHA, UTC start/end, Linux OS/arch, Rust stable version, exit codes, and clean initial/final tree.

Then continue directly into R9-3..R9-12 and the preserved Q10/Q11/Q12 queue without waiting for reviewer cadence.

## Release/live boundary

R9-2 is not closed; R9-3 must not expand the cross-process surface while H-R9-022 remains open. `READY_LIVE` remains none. Standing VPS authorization is valid; current blockers are local correctness/evidence plus the later dedicated independent R9 integration review, not permission.
