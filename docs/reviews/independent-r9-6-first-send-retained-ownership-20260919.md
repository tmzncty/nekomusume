# Independent R9 review — close H-R9-062; H-R9-063 retained first-send ownership

Review anchor: exact reachable `main` tree `e0638d76593a87cc3259bf5b33910c9d346c99d0`. Latest developer-owned source/test tree reviewed is exact `ad2296db56230a6ec8665273b15b8aa82536f9fb`, on top of `f8670a5a0ec95c4159e046fa12c4eec2177c63ff`.

This is bounded release-item-4 review support. It is not release/security approval, protocol freeze, production authorization, WAN evidence, or a performance conclusion.

## Repository truth re-read

This pass re-read the current default-branch state and the required repository sources: `README.md`, `AGENTS.md`, `ROADMAP.md`, `IMPLEMENTATION_PLAN.md`, `SECURITY.md`, `docs/status.md`, `docs/standing-vps-lab-authorization.md`, `docs/vps-rental-window-priority.md`, `docs/decisions.md`, `docs/carrier-architecture.md`, `docs/specs/nekomusume-session-v0.md`, `docs/release-security-review-packet.md`, `docs/CHATGPT_HANDOFF.md`, plus exact-current `ReliableUdpRuntime`, `PathRecovery` / `neko-reliable::Recovery`, the three executable reliable-UDP first-send call sites, the post-return PTO loop, and the new process regression.

New reachable commits reviewed since reviewer anchor `5cc11dbd9e25b4943d5f66058ab629de37340f07`, by class:

- `ad2296db56230a6ec8665273b15b8aa82536f9fb` — test + production fault-injection seam for H-R9-062 in `crates/neko-cli/src/main.rs` and `crates/neko-cli/tests/probe.rs`.
- `e0638d76593a87cc3259bf5b33910c9d346c99d0` — handoff documentation only.

## H-R9-062 — independently closed at `ad2296d`

The new `--fail-r9-first-send` branch is placed after the post-return reliable-UDP path has already completed exact-wire cwnd admission, secure-record sealing/packet-number allocation and `ReliableUdpRuntime::on_packet_sent`. It then feeds an injected `io::Error` into the same `match` branch a real `UdpSocket::send_to` error takes. The executable owner therefore calls `abandon_sent`, emits `r9_udp_post_return_send_failed`, never emits `r9_udp_post_return_sent`, and cannot truthfully emit `r9_udp_post_return_settled` for the nonexistent socket send.

The lower-level `first_send_abandon_rolls_back_full_ownership` regression from `f8670a5` remains complementary evidence for the rollback primitive itself: Recovery/Reno packet ownership is removed, `packets_sent` is reversed, the committed PN watermark is restored and the aborted PN is not ACK-valid. Exact-current source inspection also confirms the initial, second-record and post-return executable first-send branches all route socket `Err` through `ReliableUdpRuntime::abandon_sent`, with positive sent diagnostics only under `Ok`.

The earlier review suggested consolidating all three callers behind one shared production helper. `ad2296d` did not perform that refactor; however, no current correctness divergence exists merely because the simple call-site transaction remains duplicated. The actual missing production-owner discriminator now exists on the materially new post-return R9 owner, while the rollback primitive and the other two call sites remain directly inspectable. Avoiding a helper-only refactor here is consistent with the no-filler rule. H-R9-062 is therefore closed as bounded evidence at `ad2296d`.

## H-R9-063 — HIGH correctness/resource ownership: first-send abort leaves plaintext with no retry owner

The R9-6B ownership challenge finds a concrete current defect in the same transaction.

`ReliableUdpRuntime::on_packet_sent` first stores stable frame plaintext in `RetransmitBuffer`, then records the packet in Recovery and `packet_frames`. On a first-send socket failure, `ReliableUdpRuntime::abandon_sent` removes only `packet_frames[packet_number]` and calls `PathRecovery::abandon_sent`. `Recovery::abandon_sent` removes the packet and decrements/removes that frame from `outstanding_frames`; therefore after an only-copy first-send abort, `Recovery::frame_outstanding(frame)` is false and `Recovery::on_pto` has no frame from which to schedule a retry.

But `ReliableUdpRuntime::abandon_sent` does **not** release that newly retained stable plaintext from `RetransmitBuffer`. The buffer comment says retained plaintext survives for a later legitimate retry, while the exact-current recovery scheduler can only produce PTO work from `outstanding_frames`. For an only-copy first-send abort there is no outstanding packet copy and no separate persisted retry queue/owner. The retained entry is therefore orphaned until whole-runtime teardown/drop.

This is not only cosmetic accounting. `RetransmitBuffer` is intentionally bounded (currently 64 frames / 8192 bytes). Repeated distinct first-send socket errors on a long-lived runtime can accumulate unreachable retained entries until later valid `on_packet_sent` calls fail the existing retransmit-capacity admission even though Recovery reports no packet in flight for those aborted sends. This is bounded memory, but stale ownership can poison valid future transport work inside the existing bound. No new capacity value is needed to fix it.

The post-return injected-failure process path also exposes the scheduling contradiction: after `abandon_sent`, `remaining_in_flight` is expected to be zero, `next_pto_deadline_us` is `None`, and no Recovery PTO can rediscover the orphan frame; the caller only waits for its bounded Session-side settlement deadline and then terminates. That is not a real retry owner for the retained plaintext.

### Smallest dependency-ready repair contract

Current committed architecture is sufficient. Do not redesign Session/Carrier/ACK/crypto/wire semantics and do not invent TTL/LRU/history/capacity policy.

1. Make first-send socket-abort ownership complete: after removing the aborted packet copy from Recovery, stable plaintext for each affected frame must be released when no other Recovery packet copy still owns that frame and no already-existing persisted retry owner can schedule it.
2. Preserve retransmit-abort semantics. A failed retransmission must **not** drop stable plaintext while another legitimate packet copy remains outstanding or an already-existing retry owner still requires it. Do not blindly `clear()` the retransmit buffer.
3. Keep packet-number / nonce rules unchanged: aborted PN remains non-ACK-valid and is never reused; later secure-record allocation remains fresh and monotonic.
4. Add a focused deterministic unit discriminator against `ReliableUdpRuntime`:
   - first-send `on_packet_sent(frame A)` -> `abandon_sent` leaves `in_flight == 0`, Reno bytes-in-flight zero for that copy, no packet-frame mapping, rolled-back sent count/watermark, ACK(A's PN) rejected, **and zero retained plaintext ownership for frame A**;
   - several distinct first-send aborts must not accumulate retained frame/byte ownership; do not turn this into a capacity-pressure benchmark;
   - paired success still retains plaintext while in flight and releases it through the existing ACK/loss lifecycle.
5. Add a sibling/retransmit negative proving the repair does not over-release: if frame F still has another legitimate outstanding packet copy, aborting one copy leaves F resealable/retransmittable until the surviving lifecycle settles.
6. Strengthen the existing `--fail-r9-first-send` process regression only where useful to bind the production owner: require the terminal residual to show `remaining_in_flight:0`, and require absence of positive `r9_udp_post_return_sent`, `r9_udp_retransmit_sent`, `r9_udp_pto_fired`, settlement, Session-delivery and Carrier-retirement evidence caused by the failed send. Do not add schema/checker filler.
7. Audit the initial, second-record and post-return first-send callers after the repair. A single correct `ReliableUdpRuntime::abandon_sent` owner is preferable to three divergent cleanup implementations.
8. `--drop-r9-data` remains controlled Recovery-owned loss and must retain plaintext/PTO semantics; never reclassify that seam as socket abort.

After the final pushed repair SHA, persist the ordinary developer exact-tree gate:

```text
PYTHONDONTWRITEBYTECODE=1 bash scripts/check.sh
git diff --check
```

Record exact reachable pushed SHA, UTC start/end, exit codes, OS/arch, stable Rust version and clean-tree state. No decoder/parser/crypto-framing change is requested by this finding, so fuzz is not mechanically required for the repair.

On closure, continue immediately through the remainder of R9-6 ownership/resource boundedness and then R9-7 without waiting for reviewer cadence.

## Evidence truth

Developer-persisted exact-tree provenance in the current handoff reports `PYTHONDONTWRITEBYTECODE=1 bash scripts/check.sh` exit 0, `git diff --check` exit 0, clean tree for pushed `ad2296d`, Linux x86_64, rustc 1.98.0.

GitHub-hosted Rust CI for exact `ad2296d`, run `35393332734`, completed successfully: `stable checks` passed `bash scripts/check.sh`, and `nightly decode fuzz smoke` also passed. This is hosted cross-evidence only; it does not contradict H-R9-063 because the new process regression does not inspect retained plaintext ownership.

Reviewer-local execution is not claimed. A fresh clone attempt in this reviewer environment failed before checkout because `github.com` DNS could not be resolved. Exact pushed source/test inspection, developer-persisted local provenance and hosted CI remain separate evidence classes.

## Queue / live effect

H-R9-063 is the front HIGH for R9-6B. The external coding agent should repair retained ownership, run focused tests + exact-tree gate, push, and continue the remaining R9-6 / R9-7 queue immediately.

The deeper queue remains: R9-6 retained ownership closure -> remaining R9-6 bounded ownership audit -> R9-7 process/result truth -> small factual reconciliation -> R9-8 warm TCP readiness -> R9-9 health/promotion -> R9-10 uncertain replay/dedup/cleanup -> R9-11 lifecycle/terminal invariants -> R9-12 final exact-tree provenance -> dedicated independent R9 review -> Q10/Q11/Q12 factual reconciliation -> repository-wide item-4 refill if still needed.

`READY_LIVE: none` remains authoritative. H-R9-063 is deterministic local correctness/resource work and creates no unresolved real-network question. D019, retained-state capacity policy values, signing/key-custody/SBOM/publication policy, core architecture and release/freeze/production authority remain untouched.
