# R9-3 reviewer navigation — PTO time base and retransmission ownership

**Reviewed source/test anchor:** exact `021d79d88dadc9dbc7cf749745b2395c06602b29` (reachable from `main` at review time).

**Classification:** reviewer/navigator support for the already-queued R9-3 implementation slice. This note does **not** reopen H-R9-039, does not prove a defect in the current no-loss accepted path, and does not change Session/Carrier/ACK/crypto/wire architecture or any policy value.

## Scope inspected

- `crates/neko-cli/src/main.rs`
  - `failover_client` reliable-UDP initial sends and post-return reliable owner;
  - `recv_udp_delivery_ack` Carrier ACK application;
  - existing `lab_pump` PTO/retransmission loop used by the bounded R8 reliable-UDP lab.
- `crates/neko-carrier/src/lib.rs`
  - `ReliableUdpRuntime::{can_send,on_packet_sent,pto_probe,on_retransmit_sent,apply_ack}` and Recovery ownership.
- `crates/neko-reliable/src/lib.rs`
  - `Recovery::{on_sent,on_ack,on_pto,frame_outstanding}`.
- Current `docs/CHATGPT_HANDOFF.md` READY_LOCAL R9-3 contract.

## Exact-current timing seam that R9-3 must resolve

The current cross-process R9 path is internally consistent for its existing no-loss test surface because both first sends and ACK application use recovery time `0`:

- the reliable initial sends and the post-return reliable send call `on_packet_sent(..., sent_at_us = 0, ...)`;
- `recv_udp_delivery_ack` applies an authenticated Carrier ACK with `apply_ack(&ranges, 0, ack_delay_us)`.

That is **not** sufficient for the queued R9-3 PTO/retransmission slice. If R9-3 gives a retransmitted packet a positive `sent_at_us` while the shared ACK owner continues calling `apply_ack(..., now_us = 0, ...)`, an ACK whose largest acknowledged packet is that fresh retransmission can reach Recovery RTT sampling with `0.checked_sub(sent_at_us)` and fail with `Error::Arithmetic`. Conversely, keeping every timestamp at zero cannot prove that PTO fires only after a real deadline.

Therefore R9-3 needs one bounded monotonic relative recovery clock for the entire cross-process reliable operation. The smallest current-semantics shape is:

1. establish one local `Instant` origin for the reliable operation;
2. pass `elapsed().as_micros()` (checked/saturated to the existing `u64` recovery time domain) to every relevant `on_packet_sent`, `on_retransmit_sent`, and Carrier `apply_ack` call owned by that operation;
3. thread current `now_us` into `recv_udp_delivery_ack` (or an equivalently small wrapper) rather than creating a second ACK parser/demux owner;
4. keep Session DeliveryAck evidence independent from Carrier packet ACK evidence.

Do not introduce wall-clock timestamps, a second recovery engine, or a new ACK architecture.

## Reuse the committed PTO algorithm instead of inventing another one

The existing bounded `lab_pump` already demonstrates the M2 scheduling shape R9-3 should reuse conceptually:

- drain authenticated ACKs;
- compute PTO deadline from the oldest outstanding send timestamp and `recovery_engine().rtt.pto_us(...)` with the existing recovery PTO count;
- before the deadline, do **not** call `pto_probe`;
- at/after the deadline, call `pto_probe`;
- for each `(stable FrameId, plaintext)` probe, run `can_send` **before** committing retransmission ownership;
- re-encode/re-seal the same logical Data under a fresh secure envelope, yielding a fresh packet number / nonce;
- call `on_retransmit_sent(fresh_packet_number, now_us, ..., stable_frame_id)` before socket send;
- keep original and retransmission packet copies separately accountable until ACK/loss resolution makes the stable frame no longer outstanding.

R9 uses `SecureSession::seal_unreliable(ProcessMessage::Data)` on this cross-process path. Do not copy the R8 lab's extra `neko_wire::RecordType::Data` wrapper into R9 merely because the PTO loop is a useful reference; preserve the exact-current R9 framing path.

## Fault-seam placement

For the discriminating R9-3 regression, suppress exactly one reliable-owned Data **after** congestion admission and Recovery ownership commit:

`can_send -> seal/current packet number -> on_packet_sent -> [test-only socket-send suppression]`.

The fault seam must not skip `on_packet_sent`; otherwise it would test an unowned unsent record rather than data loss after congestion admission. Suppression should be one-shot and scoped to the dedicated process fixture so ordinary P2/P4 semantics remain unchanged.

## Required focused proof

A minimal acceptance-grade R9-3 process regression should prove all of the following on exact identities/cardinalities rather than broad substring presence:

- the selected first packet is recovery-owned before intentional wire suppression;
- no PTO/retransmission event occurs before the computed deadline;
- after the deadline, exactly the expected probe is scheduled;
- retransmission preserves `stream`, `offset`, payload/logical identity and stable `FrameId`, but uses a different packet number / crypto nonce from the suppressed original;
- receiver Session delivery for that logical range occurs exactly once;
- Session DeliveryAck transition and Carrier ACK retirement are separately observable and neither substitutes for the other;
- an authenticated ACK for the retransmission is applied with the same monotonic recovery time base and is not spuriously typed as rejected;
- final success requires Recovery `in_flight == 0` and Session logical confirmation complete;
- any bounded terminal negative must report the actual residual domain and must not emit false settled/failover success.

If strengthening this proof exposes a current runtime contradiction, then convert it to the smallest code repair + positive/negative regression. Otherwise keep the work within the already-authorized R9-3 slice.

## Validation / evidence boundary

This note itself ran no developer-local gate and makes no CI claim. The current handoff separately records developer-local clean exact-tree provenance for `021d79d`; hosted CI is separate cross-evidence.

If R9-3 changes only runtime orchestration/tests and does not change decoder/parser/crypto framing, do not mechanically run decode fuzz. If it changes a wire decoder/parser or framing surface, apply the repository's pinned fuzz-toolchain rule.

`READY_LIVE` remains `none`: this is local cross-process correctness/evidence work and creates no new real-network question by itself.
