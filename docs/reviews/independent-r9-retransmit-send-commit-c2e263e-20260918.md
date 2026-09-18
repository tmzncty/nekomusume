# Independent bounded R9 retransmit send/commit review — exact `c2e263e`

**Classification:** `HIGH` — H-R9-051 process/evidence + Recovery ownership correctness.

**Reviewed source/test anchor:** exact reachable pushed developer SHA `c2e263e90ab6ebabfcd829e3012a6f6ef98c26ff`.

**Review scope:** the exact-current R9 retransmit caller seam in `crates/neko-cli/src/main.rs`, the shared `admit_retransmit` helper introduced by H-R9-050, `ReliableUdpRuntime::{can_send,on_retransmit_sent}`, the post-return R9 settle loop, and `lab_pump`. This review did not change Session/Carrier/ACK/wire semantics and did not run a live/WAN experiment.

## Challenged invariant

A positive retransmit-send event/counter must describe a datagram that the local socket accepted for transmission. If the socket send returns an error, that packet identity must not remain falsely committed as an in-flight Recovery packet or as caller `outstanding`, and no `*_sent` evidence may be emitted for it. The stable retained frame/plaintext must remain available for a later legitimate retry.

This is distinct from the intentional `--drop-r9-data` loss seam: that seam deliberately commits a packet to Recovery and suppresses the wire send as the experiment's controlled loss. An ordinary OS/socket error is not that seam and must not silently masquerade as a successful retransmit.

## Finding

H-R9-050 correctly centralizes congestion admission and Recovery ownership accounting on exact encoded/sealed wire bytes, but the helper stops before the actual socket operation. Two exact-current executable callers can therefore diverge from real send outcome:

1. In the post-return R9 PTO loop, after `admit_retransmit(...)` succeeds, the code executes `let _ = u.send_to(&re_sealed, target);`, discards the result, unconditionally emits `r9_udp_retransmit_sent`, and advances `post_pn_client = rpn`. A local send error is therefore reported as a successful retransmit while Recovery already owns the packet.
2. In `lab_pump`, `sock.send(&sealed).is_ok()` gates only `retransmit_wire_sent`; `st.outstanding.insert(rpn, st.now_us)` still happens unconditionally after `admit_retransmit(...)` has committed Recovery ownership. A failed socket write can therefore leave a never-accepted datagram tracked as outstanding/in-flight.

The H-R9-050 regression only discriminates exact-wire congestion refusal versus admission. It does not inject a socket-send failure after admission, so this divergence is not challenged by that test. Hosted CI being green does not answer this missing failure path.

## Impact

A local socket error can manufacture positive `r9_udp_retransmit_sent` evidence, move the process-side active packet identity to a packet that was never accepted by the socket, and/or keep Recovery/caller outstanding state charged for a never-transmitted datagram. Subsequent PTO/health/settlement behavior can then be driven by synthetic in-flight state rather than an actual transmission attempt accepted by the OS. This violates the repository's process/result-truth boundary and is release-item-4 relevant.

## Smallest accepted repair contract

Keep the current exact-wire H-R9-050 ordering and do **not** redesign Session, Carrier ACK, Session DeliveryAck, crypto framing, or wire format. Repair the local transactional seam instead:

1. Both post-return R9 and `lab_pump` must share an executable retransmit-send path whose outcome includes the socket operation, not only congestion admission.
2. Preserve the safety property that a datagram cannot be successfully handed to the socket and then become `sent-but-untracked`. A pre-send reservation is acceptable, but a socket `Err` must roll back **only the new packet copy's** Recovery/Reno charge, packet->frame map and caller outstanding identity while retaining the stable frame/plaintext for retry. An equivalent typed reservation/commit design is also acceptable.
3. Emit `r9_udp_retransmit_sent` / increment `retransmit_wire_sent` / advance any active retransmit packet identity only after the socket call returns success.
4. Add a deterministic injected-send regression: exact-wire admission succeeds, the send seam returns `Err`, and the oracle requires no positive sent event/counter, no newly committed in-flight/outstanding packet, unchanged Session state, and retained frame ownership still available for a later retry. Add the paired successful-send control proving exactly one commit/evidence transition.
5. Apply the repair to both executable owners before closing H-R9-051. Do not use the intentional `--drop-r9-data` experimental suppression path as the socket-error oracle.
6. Run the ordinary exact-tree local gate on the final pushed repair SHA. No decoder/framing change is implied by this finding, so fuzz is not mechanically required unless the repair actually touches those surfaces.

## Exclusions / boundaries

- No claim that the current happy-path retransmit is broken when the socket send succeeds.
- No WAN/live question is created; `READY_LIVE` remains `none`.
- No new capacity/TTL/LRU/security number is selected.
- D019, release authority, signing/SBOM/key custody and production policy are untouched.
- H-R9-050's exact-wire admission conclusion remains accepted; H-R9-051 is the next caller-side transaction boundary.

## Reviewer verification boundary

This is a source-level independent bounded challenge. The reviewer inspected exact-current source and tests and the successful GitHub-hosted Rust CI for exact `c2e263e`; the reviewer did not claim a separate local exact-tree execution. Developer-local provenance recorded in the handoff remains developer evidence, not reviewer execution.
