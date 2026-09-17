# Independent reviewer checkpoint — H-R9-037 reverse-order server binding

Date: 2026-09-17

## Classification

**HIGH — evidence/oracle correctness.** This checkpoint does **not** establish a new Session/Recovery/Carrier runtime state-machine defect. It reopens the reverse post-return ACK-order closure because the exact-current process regression is weaker than the closure claim recorded in `docs/CHATGPT_HANDOFF.md`.

## Exact reviewed anchors

- Developer source/test anchor: `855c679a4085fded7382d081299ad1f6b3d915e9` (`test(cli): H-R9-036 reverse-order exact oracle — three-way packet bind + cardinality + Session exact`).
- Handoff that incorrectly closes the claim: `e670619a04d08c9b8b77297064ea299daeaf29bd`.
- Exact owners inspected:
  - `crates/neko-cli/tests/probe.rs::reliable_udp_post_return_reversed_ack_order_settles`
  - `crates/neko-cli/src/main.rs` post-return server `udp_return_packet_ack_sent` diagnostic owner
  - `crates/neko-cli/src/main.rs` client `r9_udp_return_packet_ack` / rejected / accepted-empty classification owner
  - the already-closed positive P2 fixture in `crates/neko-cli/tests/probe.rs`, which already contains the exact server/client packet-number binding pattern needed here.
- Applicable evidence boundary re-read: `docs/specs/nekomusume-session-v0.md` keeps Session delivery and Carrier packet feedback as separate evidence domains; `docs/carrier-architecture.md` requires the same separation.

## Claim challenged

The prior handoff required the reverse-order built-binary regression to prove all of:

1. both processes succeed;
2. exactly one client post-return send, exactly one server Carrier ACK, and exactly one client positive Carrier retirement;
3. exact three-way equality of client-send / server-ACK / client-retirement `packet_number`;
4. exactly one client Session transition with `stream=1 offset=48 len=16`;
5. zero rejected / accepted-empty post-return Carrier classification;
6. exactly one post-return settlement with `remaining_in_flight=0`;
7. Carrier retirement before Session transition, and settlement strictly after both.

## Finding

Exact `855c679` closes only part of that contract:

- it requires exactly one positive client `r9_udp_return_packet_ack` and checks `applied=true, retired=true`;
- it requires exactly one exact Session transition and checks `stream=1 offset=48 len=16`;
- it excludes rejected / accepted-empty classifications;
- it requires exactly one settlement and checks Carrier-before-Session and settlement-after-both ordering.

However:

1. The client post-return send is obtained with `.find(...)`; its cardinality is **not** required to be exactly one.
2. The server log is never searched for `udp_return_packet_ack_sent` in this test; server Carrier-ACK cardinality is therefore unproved.
3. The test compares only `client_send.packet_number == client_retirement.packet_number`. It never parses the server Carrier ACK packet number. The handoff phrase “three-way packet-number equality (client send == server ACK == client retire)” is therefore factually false at this tree.
4. The settlement vector is required to have length one, but the settlement line is **not** required to contain `remaining_in_flight=0`, despite the nearby comment stating that condition.

This is discriminating: a run with duplicate client-send diagnostics, a missing/duplicate/wrong-identity server ACK diagnostic, or a non-zero settlement marker can still satisfy the exact-current reverse-order test. Hosted green CI therefore proves only the weaker current oracle.

## Existing instrumentation means no runtime redesign is needed

No new protocol event is required. The exact-current runtime already emits `udp_return_packet_ack_sent` with `packet_number`, and the positive P2 regression already demonstrates the intended test pattern:

- collect exactly one `r9_udp_post_return_sent`;
- collect exactly one server `udp_return_packet_ack_sent`;
- collect exactly one positive `r9_udp_return_packet_ack` retirement;
- parse all three with the existing `packet_number(...)` helper;
- require client-send == server-ACK == client-retirement.

The same existing positive P2 pattern also shows how to require the settlement marker itself to contain `remaining_in_flight=0`.

## Smallest READY repair contract

Treat this as test/oracle repair first. In `reliable_udp_post_return_reversed_ack_order_settles` require:

1. exactly one client `r9_udp_post_return_sent`;
2. exactly one server `udp_return_packet_ack_sent`;
3. exactly one positive client `r9_udp_return_packet_ack` with `applied=true, retired=true`;
4. exact equality of all three `packet_number` values;
5. exactly one `r9_udp_return_delivery_ack` with `stream=1 offset=48 len=16`;
6. zero `r9_udp_return_packet_ack_rejected` and zero `r9_udp_return_packet_ack_accepted_empty`;
7. exactly one `r9_udp_post_return_settled`, and that exact line contains `remaining_in_flight=0`;
8. positive Carrier retirement < Session transition < settlement.

Only change runtime code if this stronger oracle exposes a real contradiction. Do not change Session/Carrier/ACK/wire/crypto architecture, deadline values, capacity values, D019 policy, or release authority.

After the focused regression passes, run the normal developer-local clean exact-tree gate on the final pushed source/test SHA:

```text
PYTHONDONTWRITEBYTECODE=1 bash scripts/check.sh
git diff --check
```

Record pushed reachable SHA, UTC start/end, OS/arch, Rust stable version, exit codes and clean-tree state. This should be test-only; do not mechanically run decoder fuzz unless decoder/parser/crypto framing actually changes.

## Evidence/provenance boundary

GitHub-hosted Rust CI for exact `855c679` completed successfully (`stable checks` and `nightly decode fuzz smoke`). The handoff also records developer-local clean exact-tree provenance for `855c679`. Both remain valid evidence **for that exact weak test tree**; neither repairs the missing server binding/cardinality/settlement assertion.

`READY_LIVE` remains `none`. No new real-network question is created by this finding. Release item 3 and item 4 remain incomplete; release/production/freeze flags remain unchanged.
