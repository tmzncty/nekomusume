# Independent R9-6 review — H-R9-065 narrow terminal guard accepted; flag H-R9-066 terminal-history/control-plane escape

Review anchor: exact reachable `main` tree `86981addcb1799dbf77369a9dfb06ac55dfbde43`. Latest developer-owned source/test tree reviewed is exact `b5e7bc90599adca19c1f40e7e7be87b0728c8980` (`fix(carrier): H-R9-065 teardown is terminal — every mutator fail-closed/inert`), on top of exact `dcbcf55160e918180c3a5e96259bffcefef34e44`.

This is bounded release-item-4 review support. It is not release/security approval, protocol freeze, production authorization, WAN evidence, or a performance conclusion.

## Repository truth re-read

This pass re-read current default-branch/recent-commit truth, `README.md`, `AGENTS.md`, `ROADMAP.md`, `IMPLEMENTATION_PLAN.md`, `SECURITY.md`, `docs/status.md`, `docs/standing-vps-lab-authorization.md`, `docs/vps-rental-window-priority.md`, `docs/decisions.md`, `docs/carrier-architecture.md`, `docs/specs/nekomusume-session-v0.md`, `docs/release-security-review-packet.md`, `docs/CHATGPT_HANDOFF.md`, exact-current `ReliableUdpRuntime`, `PathRecovery`, the H-R9-064/H-R9-065 regressions, and exact-`b5e7bc9` hosted CI.

New reachable commits since the prior reviewer anchor are classified as follows:

- `dcbcf55160e918180c3a5e96259bffcefef34e44` — implementation + deterministic unit-test repair for H-R9-064 live Recovery/Reno ownership cleanup.
- `62bab736bca59393e6a16d4eaf2b9a59acd71e03` — handoff documentation only.
- `ef80586abe1ebaff63093ce19b349e3f3652cafa` — independent reviewer note flagging H-R9-065.
- `5587b819edceea2b78b9c675c8074a8620a2c6c4` — handoff documentation only.
- `b5e7bc90599adca19c1f40e7e7be87b0728c8980` — implementation + deterministic unit-test repair for H-R9-065 terminal data/feedback guards.
- `86981addcb1799dbf77369a9dfb06ac55dfbde43` — handoff documentation only.

Developer-persisted local provenance reports exact `b5e7bc9` clean under `PYTHONDONTWRITEBYTECODE=1 bash scripts/check.sh`, `git diff --check`, and a clean worktree at the pushed SHA. GitHub-hosted Rust CI run `35407172293` for exact `b5e7bc9` completed successfully: stable `scripts/check.sh` and nightly pinned decode fuzz smoke both succeeded. Hosted CI remains cross-evidence only. Reviewer-local execution is not claimed in this pass.

## H-R9-065 — narrow terminal data/feedback guard accepted at exact `b5e7bc9`

The concrete H-R9-065 escape identified in the previous independent review is repaired in its narrow data/feedback domain:

- `torn_down=true` prevents new `can_send` eligibility and rejects `on_packet_sent`, `on_retransmit_sent`, `on_packet_received`, and `apply_ack`;
- `poll_outgoing_ack` cannot emit a pending pre-teardown Carrier ACK;
- `pto_probe` remains empty after sender ownership is quiesced;
- `poll_health` is inert after teardown.

The new regression would fail the old implementation for those paths. Preserve this repair.

## H-R9-066 — HIGH correctness/evidence: terminal teardown still erases lifetime history and leaves control-plane mutation escape hatches

The H-R9-065 closure claim is broader than the exact implementation/test. Two independent terminal invariants remain false.

### A. `teardown()` destroys public lifetime diagnostic history

`PathRecovery::new(...)` initializes `packets_sent=0`, `packets_lost=0`, RTT/PTO/recovery state, resolved-outcome counters and health epochs from scratch. `ReliableUdpRuntime::teardown()` still replaces `self.recovery` with exactly such a fresh `PathRecovery` before setting `torn_down=true`.

But the public runtime accessors `packets_sent()` and `packets_lost()` delegate directly to the replacement `PathRecovery`, and `recovery_engine()` exposes the replacement engine used for observability projection. Therefore a deterministic sequence

1. commit at least one real packet (and optionally resolve loss/PTO history),
2. observe nonzero lifetime diagnostic state,
3. call `teardown()`,
4. read terminal diagnostics,

reports the fresh owner's zeroed history rather than the facts that occurred before teardown.

This directly contradicts both the source comment that historical resolved counters remain history and the previous H-R9-065 acceptance contract: terminal quiescence must separate current live ownership from historical facts rather than rewriting “historically sent/lost/PTO-observed” into “never happened.” It is an evidence-boundary defect even though already-emitted external events remain immutable.

The current H-R9-065 regression does not capture any historical diagnostic state before teardown and asserts none afterward, so it remains green while lifetime accessors lie.

### B. Carrier-manager control mutators are still live after terminal teardown

The new terminal checks cover the listed packet/recovery methods, but the same `ReliableUdpRuntime` still exposes ungated control-plane mutators:

- `ready_standby(&mut self, now_ms)` continues to feed positive readiness observations into the embedded `ConcurrentCarrierManager`;
- `activate_udp(&mut self, now_ms)` continues to invoke manager activation;
- `manager_mut(&mut self) -> &mut ConcurrentCarrierManager` exposes the mutable manager directly with no terminal guard.

A torn-down runtime can therefore still acquire new readiness/activation/control state in the embedded manager even though data-plane/recovery methods are terminal. That violates the current R9-6/R9-11 invariant that no new Carrier success/control evidence or active ownership state may appear after terminal teardown. The H-R9-065 regression never challenges these owners.

This review does **not** require a redesign of D064 or Carrier Manager semantics. It requires the terminal owner to make its own control surface consistently terminal.

## Smallest dependency-ready repair contract

Current committed semantics already decide the answer: `ReliableUdpRuntime::teardown()` is terminal for this runtime/path generation. Repair the smallest owner boundary without changing Session/Carrier/ACK/crypto/wire architecture and without inventing new policy values.

1. Preserve H-R9-064/H-R9-065 live-owner cleanup and terminal packet/recovery guards.
2. Separate terminal historical diagnostics from current live Recovery ownership. After teardown, live `in_flight`, Reno bytes/charge, packet->frame, retained plaintext and pending ACK ownership remain zero, while pre-teardown lifetime sent/lost/PTO/RTT/resolved facts exposed as diagnostics remain truthful. A terminal summary must not become indistinguishable from “nothing ever happened.”
3. Do not restore a live `PathRecovery` merely to preserve counters. Historical diagnostics must not make packet-number admission, ACK validity, PTO, retransmit, health or cwnd live again.
4. Make the runtime's Carrier-manager control surface terminal-aware. After teardown, readiness observation, activation/promotion setup and any mutable-manager access reachable through `ReliableUdpRuntime` must be fail-closed or observationally inert. Prefer one runtime-owned terminal gate rather than caller-side duplicated flags. If `manager_mut` cannot preserve the invariant safely in its current form, narrow/replace that exposure with the smallest typed/gated API needed by existing callers; do not change D064 policy values or state meanings.
5. Add one deterministic discriminator on the real runtime owner:
   - before teardown, commit a packet and create nonzero lifetime diagnostic state; create at least one manager/readiness state that makes post-terminal mutation observable;
   - capture lifetime diagnostics and manager state;
   - teardown;
   - assert all current live transport ownership is zero and packet/recovery methods remain terminal;
   - assert lifetime diagnostic facts remain equal to the pre-teardown historical facts (not reset to a fresh-zero history);
   - attempt post-terminal `ready_standby`, activation/promotion-related control and mutable-manager access through the supported public surface; assert no new readiness/active/switch state or positive evidence appears;
   - assert repeated teardown is idempotent and does not rewrite history again.
6. Add a paired non-terminal control proving ordinary readiness/activation plus packet recovery still function on a live runtime.
7. Preserve committed packet-number non-reuse, first-send/retransmit socket rollback, retained-plaintext ownership, controlled `--drop-r9-data` loss semantics, and all earlier H-R9 closures.
8. Re-audit executable terminal/error exits for terminal summary reads: they must consume truthful historical diagnostics while never generating fresh terminal runtime evidence.
9. No D019 change, no `SessionRuntime.events` retention/capacity decision, no new TTL/LRU/history-size/security/capacity number, no core architecture change, and no release-authority transition.
10. On the final pushed repair SHA run and persist:

```text
PYTHONDONTWRITEBYTECODE=1 bash scripts/check.sh
git diff --check
```

Record exact reachable pushed SHA, UTC start/end, exit codes, OS/arch, stable Rust version and clean-tree state. Fuzz is not mechanically required unless the repair actually changes wire decoder/parser/crypto framing.

## Queue / live effect

H-R9-066 is now the front HIGH inside R9-6. After it closes, continue the remaining R9-6 ownership/resource bounded challenge, then R9-7 process/result truth, one small factual reconciliation, R9-8 through R9-12, dedicated independent R9 review, Q10/Q11/Q12 reconciliation, and repository-wide item-4 refill if still needed. Do not wait for reviewer cadence between slices.

`READY_LIVE: none` remains authoritative. This is deterministic local correctness/evidence work and creates no unresolved real-network question. Release item 3 and item 4 remain incomplete; `RELEASE_CANDIDATE=false`, `PRODUCTION_READY=false`, `FREEZE=false`, and `RELEASED=false`. D019, `SessionRuntime.events` capacity policy, signing/key-custody/SBOM/publication policy, core architecture and release authority remain untouched.
