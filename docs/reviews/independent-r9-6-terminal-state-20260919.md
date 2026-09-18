# Independent R9-6 review — H-R9-064 live-owner repair accepted; flag H-R9-065 terminal-state escape

Review anchor: exact reachable `main` tree `62bab736bca59393e6a16d4eaf2b9a59acd71e03`. Latest developer-owned source/test tree reviewed is exact `dcbcf55160e918180c3a5e96259bffcefef34e44` (`fix(carrier): H-R9-064 teardown quiesces live Recovery ownership`).

This is bounded release-item-4 review support. It is not release/security approval, protocol freeze, production authorization, WAN evidence, or a performance conclusion.

## Repository truth re-read

This pass re-read current default-branch/recent-commit truth, `README.md`, `AGENTS.md`, `ROADMAP.md`, `IMPLEMENTATION_PLAN.md`, `SECURITY.md`, `docs/status.md`, `docs/standing-vps-lab-authorization.md`, `docs/vps-rental-window-priority.md`, `docs/decisions.md`, `docs/carrier-architecture.md`, `docs/specs/nekomusume-session-v0.md`, `docs/release-security-review-packet.md`, `docs/CHATGPT_HANDOFF.md`, exact-current `ReliableUdpRuntime`, `PathRecovery`, `PacketAckTracker`, `neko_reliable::Recovery`, the H-R9-064 regression, and exact-`dcbcf55` hosted CI.

New reachable commits since the prior reviewer anchor are classified as follows:

- `dcbcf55160e918180c3a5e96259bffcefef34e44` — implementation + deterministic unit-test repair for H-R9-064 sender-side live Recovery/Reno ownership.
- `62bab736bca59393e6a16d4eaf2b9a59acd71e03` — handoff documentation only.

GitHub-hosted Rust CI run `35403232907` for exact `dcbcf55` completed successfully: stable `bash scripts/check.sh` and nightly pinned decode fuzz smoke both succeeded. This is cross-evidence only. Developer-persisted local provenance reports a clean exact-tree gate for `dcbcf55`. Reviewer-local execution is **not claimed**: the reviewer container could not resolve `github.com`, so no checkout/test run was fabricated.

## H-R9-064 — narrow live-owner defect repaired at exact `dcbcf55`

The concrete defect from the previous review is fixed in the narrow sense: `teardown()` now removes packet->frame/plaintext ownership and replaces sender `PathRecovery`, so `in_flight()` and Reno `bytes_in_flight` become zero; `pto_probe()` also returns before mutating PTO state when no packets are in flight. The strengthened test would fail the old implementation that left live Recovery/Reno ownership behind.

That repair is useful and should be preserved. It is **not sufficient to close the terminal contract**, because the runtime is not actually terminal after `teardown()`.

## H-R9-065 — HIGH correctness/evidence: teardown is quiescent only for sender Recovery, not terminal runtime state

`ReliableUdpRuntime::teardown()` currently clears `packet_frames` and retained plaintext, then replaces only `PathRecovery` with a fresh owner for the **same path and generation**. It does not mark the runtime terminal and does not clear/gate receiver-side `PacketAckTracker`; `health` and `manager` also remain live objects. Public send/receive/ACK methods remain callable.

Two deterministic counterexamples follow directly from exact-current source.

### A. Pre-teardown receiver ACK obligation escapes after terminal teardown

1. Call `on_packet_received(7, true)` before teardown. `PacketAckTracker::observe_packet` records packet 7 and sets `pending_ack=true`.
2. Call `teardown()`. The method does not touch `self.acks`.
3. Call `poll_outgoing_ack(0)` after teardown. `PacketAckTracker::take_ack` consumes the still-pending obligation and returns a positive `AckPayload` for the terminated path generation.

This manufactures fresh Carrier ACK evidence after a method documented and queued as a terminal/quiescent transition. The previous H-R9-064 contract explicitly required post-teardown ACK-related paths to be fail-closed or observationally inert; the new test covers only PTO.

### B. Fresh same-generation Recovery re-opens packet admission after teardown

`PathRecovery::new` creates a fresh `neko_reliable::Recovery::default()`. `Recovery::new` sets `largest_sent=None`; `Recovery::on_sent` rejects packet-number reuse only when `largest_sent` contains a committed high-water. Therefore:

1. send/commit packet number `0` before teardown;
2. call `teardown()`;
3. call `on_packet_sent(0, ...)` again on the same `ReliableUdpRuntime`;
4. the fresh Recovery owner has forgotten the committed high-water and can accept `0` again, recreating Recovery/Reno/plaintext ownership after the terminal transition.

`PathRecovery` explicitly documents the packet number as the authenticated `SecureSession` record sequence / AEAD-nonce identity and the single source of truth. This review does **not** claim a demonstrated cryptographic nonce-reuse exploit in the executable caller, because the external `SecureSession` may independently keep its sequence monotonic. It does establish that the runtime's committed-number non-reuse guard is lost and that terminal teardown can be followed by new live ownership under the same path generation, contrary to the current R9-6/R9-11 invariants.

The same reset also zeros `PathRecovery` lifetime diagnostics (`packets_sent`, `packets_lost`, RTT/PTO/outcome counters) rather than separating historical facts from current live ownership. Already-emitted external diagnostics remain immutable, but any terminal summary that consults this runtime after teardown can no longer distinguish “historically sent/lost, now quiescent” from “never happened.” That is an evidence-boundary risk and should be addressed by the same terminal-state repair rather than by adding ad-hoc caller snapshots.

### Why the current oracle misses H-R9-065

The H-R9-064 regression proves only sender live-owner zero and post-teardown PTO inertness. It does not create a pending receiver ACK before teardown, does not call `poll_outgoing_ack` afterward, and does not attempt either reused or fresh packet admission after teardown. It therefore stays green even though the runtime can emit stale ACK feedback and become live again.

## Smallest dependency-ready repair contract

Current architecture already decides the semantics: teardown is terminal for this runtime/path generation. Do not redesign Session/Carrier/ACK/crypto/wire behavior and do not invent TTL/LRU/history/capacity/security values.

1. Give `ReliableUdpRuntime` one authoritative terminal/quiesced state (or an equivalent single owner mechanism) rather than modeling terminal teardown as “fresh same-generation runtime.”
2. On teardown, reconcile all **current live ownership** to zero: Recovery sent/outstanding state, Reno charge/bytes-in-flight, packet->frame map, retained plaintext, and any pending receiver ACK obligation from the terminated runtime.
3. Preserve historical facts separately from current live ownership where existing public diagnostics expose lifetime history. Do not obtain quiescence by silently turning historical `packets_sent` / loss / PTO observations into “never happened.”
4. After teardown, all data/feedback mutators for this runtime must be fail-closed or observationally inert. At minimum challenge `can_send`/`on_packet_sent`, `on_retransmit_sent`, `on_packet_received`, `poll_outgoing_ack`, `apply_ack`, `pto_probe`, and `poll_health`. No new packet/frame/plaintext ownership, ACK emission, PTO/loss/health transition, or success evidence may appear.
5. Add a focused deterministic terminal discriminator using the production owners:
   - before teardown, commit at least one sender packet and create one pending receiver ACK obligation;
   - record nonzero live ownership and relevant historical counters;
   - teardown;
   - assert zero current ownership;
   - assert `poll_outgoing_ack` cannot emit the pre-teardown ACK;
   - assert reused **and fresh** packet-number sends cannot recreate ownership on the terminated runtime;
   - assert post-teardown receive/ACK/PTO/health calls create no new evidence/state;
   - if lifetime diagnostic accessors remain public, assert historical facts remain truthful.
6. Keep a paired normal-lifecycle control showing a non-terminal runtime still admits fresh strictly increasing packet numbers, emits receiver ACKs, settles ACK/loss, and releases ownership normally.
7. Preserve H-R9-061/H-R9-062/H-R9-063 semantics and the repaired H-R9-064 sender-owner quiescence. Controlled `--drop-r9-data` remains deliberate Recovery-owned loss; socket abort remains transactional; committed packet numbers are never reused; aborted reservations are not ACK-valid.
8. Re-audit executable terminal/error exits that call `teardown()` or otherwise retain a `ReliableUdpRuntime` afterward. They must not consult a terminal runtime for new success/health/recovery evidence.
9. On the final pushed repair SHA, run and persist:

```text
PYTHONDONTWRITEBYTECODE=1 bash scripts/check.sh
git diff --check
```

Record exact reachable pushed SHA, UTC start/end, exit codes, OS/arch, stable Rust version and clean-tree state. Fuzz is not mechanically required unless the actual repair changes wire decoder/parser/crypto framing.

## Queue / live effect

H-R9-065 is now the front HIGH within R9-6. After it closes, continue the remaining R9-6 ownership/resource bounded challenge, then R9-7 process/result truth, one small factual reconciliation, R9-8 through R9-12, dedicated independent R9 review, Q10/Q11/Q12 reconciliation, and repository-wide item-4 refill if still needed. Do not wait for reviewer cadence between slices.

`READY_LIVE: none` remains authoritative. This is deterministic local correctness/evidence work and creates no unresolved real-network question. Release item 3 and item 4 remain incomplete; `RELEASE_CANDIDATE=false`, `PRODUCTION_READY=false`, `FREEZE=false`, and `RELEASED=false`. D019, `SessionRuntime.events` capacity policy, RSEC-001 conditions, signing/key-custody/SBOM/publication policy, core architecture and release authority remain untouched.
