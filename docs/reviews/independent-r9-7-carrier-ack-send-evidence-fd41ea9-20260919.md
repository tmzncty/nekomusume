# Independent R9-7 review — H-R9-071 Carrier ACK socket-outcome evidence truth

**Reviewed implementation/test anchor:** exact reachable pushed source/test commit `fd41ea9ff9ec772951c61fb4ed5162fbc1977d95`.

**Classification:** HIGH — process/evidence correctness. This is not a wire, ACK-architecture, crypto, Session-delivery, retention-capacity, or policy decision.

## Scope inspected

- `crates/neko-cli/src/main.rs`
  - executable reliable-UDP server Carrier packet-ACK send owners on the initial path;
  - post-return/migration-back Carrier packet-ACK send owners, including normal and delayed/reordered branches;
  - H-R9-068 malformed-bound terminal continuation;
- `crates/neko-cli/tests/probe.rs::reliable_udp_post_return_malformed_bound_is_terminal` at exact `fd41ea9`;
- H-R9-070 change at exact `fd41ea9` requiring both post-flood feedback domains;
- current Session-vs-Carrier evidence separation and release-review boundaries.

Reviewer-local execution is **not** claimed. Developer-local exact-tree provenance for `fd41ea9` is retained in the current handoff. GitHub-hosted Rust CI run `35422199161` for exact `fd41ea9` completed successfully; hosted CI is cross-evidence only.

## Challenged invariant

A structured diagnostic whose event name says `*_sent` is positive evidence that the corresponding datagram was accepted by the socket send operation. An OS/socket `Err` must never be projected as successful Carrier feedback.

This matters directly to H-R9-070: its resurrection discriminator now requires both `udp_return_delivery_ack_sent` and `udp_return_packet_ack_sent` to establish that complete valid Session + Carrier feedback was sent after the malformed flood. The Carrier half therefore must be backed by a successful socket send, not merely an attempted send.

## Concrete counterexample

The current executable server has multiple Carrier ACK owners shaped as:

```rust
let _ = udp.send_to(&sealed_ack_or_pack, peer);
emit_diagnostic(..., "udp_packet_ack_sent" | "udp_return_packet_ack_sent", ...);
```

The `send_to` result is discarded. If the socket send returns `Err`, execution still emits the positive `*_sent` event. The same pattern exists on the normal post-return ACK path and delayed/reordered ACK release paths; the initial reliable-UDP ACK owner also has the same evidence bug.

The post-return Session DeliveryAck path does not have this exact false-positive shape: it requires its socket send before the `udp_return_delivery_ack_sent` event. The mismatch means H-R9-070 can still report its two-domain precondition even when the Carrier ACK never entered the socket.

This is an evidence defect even if a later client timeout eventually prevents final success. R9-7 explicitly requires structured diagnostics to correspond to the typed/socket outcome they claim.

## Required smallest repair

1. Audit every executable server Carrier packet-ACK send owner that can emit `udp_packet_ack_sent` or `udp_return_packet_ack_sent`.
2. Emit the positive `*_sent` diagnostic **only after `UdpSocket::send_to` returns `Ok`**.
3. On `Err`, emit a typed failure classification such as `udp_packet_ack_send_failed` / `udp_return_packet_ack_send_failed`, or an equivalently precise existing error event. Do not relabel the failure as sent.
4. Do not redesign ACK ranges, Recovery, Session delivery, crypto, wire format, D064, or any policy value. Whether the operation may later recover through a fresh packet/ACK opportunity should remain governed by current recovery semantics; this finding only requires truthful socket-outcome projection unless the deterministic process regression exposes a separate correctness defect.
5. Add a deterministic test seam on the **real executable post-return Carrier ACK owner** so the send outcome can be forced to `Err` after ACK construction/sealing. Do not rely on random OS failure and do not satisfy this with a helper-only unit test.
6. Negative process regression must prove for the injected target ACK:
   - typed ACK-send failure is observable;
   - no positive `udp_return_packet_ack_sent` is emitted for that failed send/packet identity;
   - no false H-R9-070 “complete dual feedback sent” premise can be established from the failed Carrier channel;
   - final client success/settlement cannot be inferred solely from the failed ACK attempt.
7. Preserve a paired normal-path positive control proving a successful Carrier ACK send still emits the positive event and the existing successful settlement path remains green.
8. Cover the same evidence rule on initial `udp_packet_ack_sent` and delayed/reordered post-return ACK owners in the same coherent repair, preferably with one minimal shared socket-outcome helper only if that reduces duplicated error-prone logic. Do not introduce framework/schema filler.

## Validation contract

On the final pushed developer SHA:

```text
PYTHONDONTWRITEBYTECODE=1 bash scripts/check.sh
git diff --check
```

Persist exact reachable pushed SHA, UTC start/end, exit codes, OS/arch, stable Rust version, and clean-tree state. Decoder/parser/crypto framing is not implicated by this finding, so fuzz is not mechanically required unless the repair changes those surfaces.

## Queue consequence

H-R9-070 remains narrowly closed as a test condition requiring both feedback domains, but R9-7 is **blocked on H-R9-071** because the Carrier `*_sent` domain is not yet a truthful socket-success projection. After repair and independent closure, continue immediately through the remaining R9-7 process/result truth sweep; do not wait for reviewer cadence.
