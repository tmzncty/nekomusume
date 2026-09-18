# Independent bounded R9 send-rollback review — exact `267604d`

**Classification:** `HIGH` — H-R9-052 Recovery/evidence correctness; H-R9-051 remains open pending its required failure-path discriminator and exact-tree provenance.

**Reviewed source/test anchor:** exact reachable pushed developer SHA `267604d43fe7cd1bd15cd900d8fa84ba7f8ff353`.

**Review scope:** exact-current `Recovery::{on_sent,on_ack,abandon_sent}`, `PathRecovery::{on_sent,abandon_sent}`, `ReliableUdpRuntime::{on_retransmit_sent,abandon_retransmit,abandon_sent}`, the shared executable retransmit admission path, the post-return R9 PTO caller, and `lab_pump`. The review also rechecked the already-accepted future/never-sent ACK invariant and H-R9-051 socket-outcome contract. No live/WAN experiment was run and no protocol/wire architecture was changed.

## Accepted progress

Exact `267604d` does fix the original H-R9-051 caller divergence on the ordinary happy/error branch: after exact-wire admission, both executable retransmit callers now check the socket result; a socket `Err` invokes `ReliableUdpRuntime::abandon_retransmit`, skips the positive `r9_udp_retransmit_sent`/`retransmit_wire_sent` evidence, does not advance `post_pn_client`, and does not insert caller `outstanding`. `PathRecovery::abandon_sent` also removes the new Recovery packet copy and reverses the Reno charged bytes.

This progress must be retained. The finding below is about rollback state that the new source repair leaves behind and about the missing discriminator required before closure.

## H-R9-052 finding — aborted reservation remains ACK-valid through `largest_sent`

`Recovery::on_ack` deliberately rejects a peer ACK whose `largest` exceeds `largest_sent`; the comment states this is the atomic future/never-sent guard because otherwise a fabricated largest ACK can drive packet-threshold loss/retransmit over real in-flight packets.

The new `Recovery::abandon_sent`, however, removes the failed socket packet from `sent` and reverses its frame-copy count **while explicitly leaving `largest_sent` unchanged**. After a socket-send failure, that packet number was never accepted by the socket and therefore is a never-sent identity for Carrier feedback purposes, yet it remains at or below the ACK-valid high-water mark.

A deterministic counterexample does not require WAN behavior: keep an older real packet in flight, create enough monotonic packet-number gap (for example by later reservations/refusals/aborted reservations), reserve packet `N`, abort it as a socket failure, then feed an ACK whose largest is `N`. Because `largest_sent == N`, the current guard accepts the feedback even though `N` is absent from `sent`; the same `largest=N` is then used by packet-threshold loss detection and can retire a real older packet as lost. At minimum, the never-sent ACK is no longer rejected atomically; with a threshold-sized gap it can also mutate loss/retransmit state. This reopens the candidate-A invariant on the rollback path.

The repair must not solve this by requiring every ACK number to remain in `sent`: ACKs for genuinely sent packets may arrive after that packet has already been retired and accepted-empty is an intentional classification. The distinction needed here is **successfully committed-to-socket send history versus an aborted pre-send reservation**, not `sent`-map membership.

## Secondary API/state divergence introduced by the same patch

`ReliableUdpRuntime::abandon_retransmit` correctly routes through `PathRecovery::abandon_sent`, which reverses Recovery + Reno charge and drops the packet->frame map. But exact `267604d` also exposes a separate public `ReliableUdpRuntime::abandon_sent` that directly calls `self.recovery.recovery_mut().abandon_sent(number)`. That bypasses `PathRecovery::charged`/Reno rollback and `packet_frames` cleanup. The newly exposed `PathRecovery::recovery_mut` exists only to enable that bypass in the reviewed diff.

This public partial-rollback surface can create `in_flight()==0` while `bytes_in_flight` and packet->frame bookkeeping remain charged/stale. It should be removed, made non-bypassable, or routed through the same complete transactional rollback owner. No exact-current executable caller needs the partial variant.

`PathRecovery::on_sent` also increments the diagnostics `packets_sent` counter before the socket outcome, while `abandon_sent` does not reverse it. The existing API/documentation names this a sent-packet counter, not a reservation-attempt counter. The H-R9-051 failure-path regression should therefore pin the intended current semantic and prevent a socket `Err` from manufacturing a positive sent counter unless the repository explicitly changes that diagnostic contract.

## Missing H-R9-051 acceptance discriminator

Exact `267604d` changes only source owners; it adds no deterministic injected-send failure regression and no paired successful-send control. Therefore the required H-R9-051 oracle is still absent even before considering H-R9-052. GitHub-hosted Rust CI run `35316989919` is green, but existing tests do not inject the post-admission socket-error path and cannot prove rollback/evidence/retained-frame behavior.

## Smallest accepted repair contract

Keep H-R9-050 exact-wire admission and the useful socket-result checks from `267604d`. Do not redesign Session delivery, Carrier ACK, crypto framing, or wire format.

1. Make pre-send retransmit reservation/rollback preserve a truthful **committed sent high-water**. An aborted socket reservation must not become ACK-valid future/never-sent history. A typed reservation/commit token, or an equivalent local transactional shape that restores the previous committed high-water on abort and cannot regress past genuinely sent/retired packets, is acceptable. Do not add an unbounded abandoned-packet history or invent a TTL/LRU/capacity policy.
2. Add a focused Recovery/runtime regression: after an admitted reservation is aborted, ACK of the aborted packet number must return the existing invalid/fail-closed classification and leave RTT, PTO count, sent/in-flight packet state, loss/retransmit result and Reno/cwnd accounting unchanged. Include a threshold-sized packet-number gap so the test deterministically fails if the aborted number is still used as packet-threshold `largest`.
3. Preserve legitimate late/duplicate ACK semantics for packet numbers that were actually handed to the socket and later retired; do not implement the fix as `ACK number must currently exist in sent`.
4. Remove the public partial `ReliableUdpRuntime::abandon_sent`/`PathRecovery::recovery_mut` bypass or route it through the same complete rollback owner so Recovery, Reno `charged`, packet->frame map and caller-facing accounting cannot diverge.
5. Complete the original H-R9-051 injected-send regression at the real executable seam for **both** post-return R9 and `lab_pump`: exact-wire admission succeeds; injected socket send returns `Err`; no positive sent event/counter, no new Recovery/caller outstanding identity, no Session transition; stable frame/plaintext remains retryable. Paired success produces exactly one ownership/evidence transition.
6. Pin diagnostics truth: if `packets_sent` retains its current documented meaning, socket `Err` must not increment it after rollback. If a different reservation-attempt metric is desired, that is a separate explicit evidence-semantic change and must not silently redefine existing `packets_sent`.
7. Run focused tests, then on the final pushed source/test SHA run `PYTHONDONTWRITEBYTECODE=1 bash scripts/check.sh`, `git diff --check`, and confirm a clean tree with exact SHA/UTC/OS-arch/stable-Rust provenance. No decoder/framing change is required by this finding, so fuzz is not mechanical unless the repair actually touches those surfaces.

## Evidence boundary

- GitHub-hosted Rust CI for exact `267604d`, run `35316989919`, completed successfully; it is hosted cross-evidence only and does not exercise the missing injected send-failure/aborted-ACK discriminator.
- This reviewer did not claim a separate local exact-tree run. A container clone attempt was unavailable because the review runtime had no DNS access; the finding is therefore source-level plus existing hosted-CI evidence, not reviewer-executed Rust evidence.
- `READY_LIVE` remains `none`: this is a deterministic local Recovery/process-truth question.
- D019, resource-policy values, signing/SBOM/key custody, release/freeze/production authority and core Session/Carrier/crypto/wire architecture are untouched.
