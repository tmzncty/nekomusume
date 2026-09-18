# Independent R9-6 review — close H-R9-063; flag H-R9-064 terminal teardown ownership

Review anchor: exact reachable `main` tree `b017700582398eb78942ca8ee205c6d50e439341`. Latest developer-owned source/test tree reviewed is exact `2b4abdfe23f1beffc0f72a8d551d4f6b8e08276d`, on top of `ad2296db56230a6ec8665273b15b8aa82536f9fb`.

This is bounded release-item-4 review support. It is not release/security approval, protocol freeze, production authorization, WAN evidence, or a performance conclusion.

## Repository truth re-read

This pass re-read the current default-branch state, recent developer/reviewer commits, `README.md`, `AGENTS.md`, `ROADMAP.md`, `IMPLEMENTATION_PLAN.md`, `SECURITY.md`, `docs/status.md`, `docs/standing-vps-lab-authorization.md`, `docs/vps-rental-window-priority.md`, `docs/decisions.md`, `docs/carrier-architecture.md`, `docs/specs/nekomusume-session-v0.md`, `docs/release-security-review-packet.md`, `docs/CHATGPT_HANDOFF.md`, plus exact-current `ReliableUdpRuntime`, `PathRecovery`, `neko_reliable::Recovery`, R9-6 ownership tests, and exact `2b4abdf` hosted CI.

New reachable commits since the previous reviewer handoff are classified as follows:

- `ad2296db56230a6ec8665273b15b8aa82536f9fb` — production-owner fault injection + process regression for H-R9-062.
- `e0638d76593a87cc3259bf5b33910c9d346c99d0` — handoff documentation only.
- `4920ed27ed89132dcf852c88dc78c9b209f00ff1` — independent reviewer note closing H-R9-062 and opening H-R9-063.
- `57568f44f174dcfcc6d39f0290385721844df7ce` — handoff documentation only.
- `2b4abdfe23f1beffc0f72a8d551d4f6b8e08276d` — implementation/tests repairing H-R9-063 retained first-send plaintext ownership.
- `b017700582398eb78942ca8ee205c6d50e439341` — handoff documentation only.

## H-R9-063 — independently closed at exact `2b4abdf`

The source repair is consistent with the current ownership model. `ReliableUdpRuntime::abandon_sent` now removes the aborted packet's `packet_frames`, rolls back the packet through `PathRecovery::abandon_sent`, and releases each affected stable plaintext entry only when `Recovery::frame_outstanding(frame)` reports that no packet copy still owns it. This closes the concrete orphan identified by the prior review without changing packet-number, ACK, wire, crypto, Session, or capacity policy.

The focused sibling discriminator is meaningful: two outstanding packet copies may carry the same stable frame; aborting only one keeps the retained plaintext while the sibling remains in Recovery, while a single-copy abort releases its plaintext. The earlier full-rollback test also now checks that an aborted frame is absent from the retransmit owner after rollback. The production `--fail-r9-first-send` regression was strengthened to require zero residual Recovery in-flight and no PTO/retransmit evidence for the failed first send.

The repair does not conflate deliberate controlled loss with socket abort and does not change retransmit-abort semantics. H-R9-063 is therefore closed as bounded correctness/resource evidence at exact `2b4abdf`.

Developer-persisted exact-tree provenance for `2b4abdf` reports `PYTHONDONTWRITEBYTECODE=1 bash scripts/check.sh` exit 0, `git diff --check` exit 0 and a clean pushed worktree on Linux x86_64 with rustc 1.98.0. GitHub-hosted Rust CI run `35398511814` for exact `2b4abdf` also completed successfully. Hosted CI remains cross-evidence only. Reviewer-local execution is not claimed.

## H-R9-064 — HIGH correctness/resource/evidence: teardown clears bytes but leaves live Recovery ownership

The remainder of R9-6 finds a concrete terminal-lifecycle defect in the exact-current runtime.

`ReliableUdpRuntime::teardown()` currently clears only `packet_frames` and `RetransmitBuffer`. It does not retire/reset the owned `PathRecovery` state. `PathRecovery` still contains the `neko_reliable::Recovery` sent map / outstanding-frame copy counts, Reno `bytes_in_flight`, per-packet `charged` bookkeeping, PTO state, and health outcome epochs.

A deterministic counterexample already exists inside the current teardown unit-test setup:

1. `on_packet_sent(0, ..., FrameId(9), ...)` creates one Recovery packet copy, Reno charge and retained plaintext;
2. `on_retransmit_sent(1, ..., FrameId(9))` creates a second outstanding packet copy and second congestion charge;
3. `teardown()` clears `packet_frames` and plaintext only;
4. `in_flight()` therefore remains nonzero and `recovery_bytes_in_flight()` remains charged even though the runtime claims path/generation teardown;
5. calling `pto_probe()` after teardown still enters `PathRecovery::on_pto` / `Recovery::on_pto`, advances PTO state and a fresh recovery outcome epoch, and internally selects the stale outstanding frame; only the outer `filter_map` turns the returned public probe list into empty because the plaintext owner was already cleared;
6. a subsequent health poll can consume that post-teardown outcome epoch as fresh Carrier health evidence, while congestion admission can remain constrained by phantom Reno bytes.

The existing regression `retransmit_requires_retained_ownership_and_teardown_is_deterministic` checks only that `pto_probe().is_empty()` after teardown. That oracle is therefore insufficient: it passes because plaintext was erased, not because Recovery ownership was quiesced. It can hide stale sent packets while the very assertion mutates PTO/recovery evidence after teardown.

This violates the current R9-6 contract already written in the handoff: terminal teardown must deterministically clear residual packet/frame/plaintext ownership without rewriting historical failure evidence. It also violates the source comment that presents `teardown()` as path/generation teardown rather than merely a retransmit-buffer clear.

### Smallest dependency-ready repair contract

Current architecture is sufficient; do not redesign Session/Carrier/ACK/crypto/wire semantics and do not invent new TTL/LRU/history/capacity/security values.

1. Make `ReliableUdpRuntime::teardown` a true quiescent terminal ownership transition for this runtime/path generation. After it returns, current live ownership must reconcile to zero: Recovery in-flight packets, `packet_frames`, Reno bytes-in-flight / per-packet charge, and retained plaintext must not survive.
2. Preserve historical diagnostic facts where the existing model treats them as history. Do not erase or rewrite already-emitted loss/failure evidence merely to get zero ownership. Separate current live ownership from historical counters/evidence rather than resetting indiscriminately.
3. Post-teardown calls must fail closed or remain observationally inert: `pto_probe`, recovery ACK application, health polling, or similar paths must not create new PTO/loss/ACK/Carrier-health/success evidence derived from packets that belonged to the terminated path generation.
4. Replace/strengthen the current misleading teardown regression with a deterministic discriminator. Before teardown, assert nonzero `in_flight` and congestion bytes with overlapping original/retransmit copies. After teardown, assert zero current Recovery ownership, zero congestion bytes, zero retained plaintext, no packet->frame ownership, and prove a post-teardown probe/feedback call cannot mutate PTO/health state or manufacture a new positive transition.
5. Include a paired normal-lifecycle control showing ACK/loss settlement still releases exactly the existing owners and does not depend on teardown to mask leaks.
6. Re-audit executable terminal/error exits that own a `ReliableUdpRuntime`: each path must either perform the correct terminal cleanup or drop the owner without later consulting it for success/health/recovery evidence. Do not create a separate framework or duplicate cleanup logic merely for the test.
7. Preserve H-R9-061/H-R9-062/H-R9-063 behavior: first-send abort remains transactional, deliberate `--drop-r9-data` remains Recovery-owned controlled loss, retransmit socket abort preserves stable plaintext while another legitimate retry owner exists, committed packet numbers are never reused and aborted reservations remain non-ACK-valid.
8. Run the ordinary exact-tree developer gate on the final pushed repair SHA:

```text
PYTHONDONTWRITEBYTECODE=1 bash scripts/check.sh
git diff --check
```

Persist exact reachable pushed SHA, UTC start/end, exit codes, OS/arch, stable Rust version and clean-tree state. No fuzz is mechanically required unless the repair actually touches wire decoder/parser/crypto framing.

After H-R9-064 closes, continue immediately through the rest of the R9-6 bounded ownership/resource challenge, then R9-7, without waiting for reviewer cadence.

## Queue / live effect

H-R9-064 is the front HIGH. The deeper dependency-ordered queue remains R9-6 remainder -> R9-7 process/result truth -> small factual reconciliation -> R9-8 warm TCP readiness -> R9-9 health/promotion -> R9-10 uncertain replay/dedup/cleanup -> R9-11 lifecycle/terminal invariants -> R9-12 final exact-tree provenance -> dedicated independent R9 review -> Q10/Q11/Q12 factual reconciliation -> repository-wide item-4 refill if still needed.

`READY_LIVE: none` remains authoritative. This finding is deterministic local correctness/resource/evidence work and creates no unresolved real-network question. Release item 3 and item 4 remain incomplete; `RELEASE_CANDIDATE=false`, `PRODUCTION_READY=false`, `FREEZE=false`, and `RELEASED=false`. D019, `SessionRuntime.events` capacity policy, RSEC-001 conditions, signing/key-custody/SBOM/publication policy, core architecture and release authority remain untouched.
