# Independent R9-6 ownership / resource boundedness review — exact `788a4dc`

**Classification:** bounded independent no-finding review for release item 4 support. This is not a security audit, release approval, protocol freeze, production authorization, WAN evidence, or a capacity/stress conclusion.

**Exact developer source/test anchor reviewed:** `788a4dcbb4ab802f1444991d7898ec48ed7a9030` (`fix(carrier): H-R9-066 terminal teardown preserves history + gates control plane`), reachable from `main` at the time of review.

## Scope and challenged invariants

This review closes the remaining R9-6 ownership/resource-boundedness lane after H-R9-061 through H-R9-066. I re-read the exact-current owners rather than relying on prior handoff claims:

- `crates/neko-reliable/src/lib.rs`: `Recovery::{on_sent,abandon_sent,on_ack,on_pto,quiesce}`, outstanding-frame copy accounting, committed packet-number high-water restoration and Reno state;
- `crates/neko-carrier/src/lib.rs`: `PathRecovery::{on_sent,abandon_sent,on_ack,on_pto,quiesce}`, `ReliableUdpRuntime::{on_packet_sent,on_packet_received,poll_outgoing_ack,apply_ack,pto_probe,on_retransmit_sent,abandon_sent,abandon_retransmit,teardown,ready_standby,activate_udp,manager_mut}` and retained-plaintext / packet-to-frame ownership;
- `crates/neko-cli/src/main.rs`: exact-wire retransmit admission, first-send socket outcome, retransmit socket rollback, controlled `--drop-r9-data` loss seam and post-return settlement owner;
- current deterministic tests covering committed-watermark restoration/non-reuse, first-send abort, sibling-copy retention, teardown quiescence/history and terminal control-plane gating.

The bounded challenge was whether any refusal, socket abort, ACK/loss outcome, overlapping retransmit copy or teardown can leave a packet, Reno charge, packet-to-frame entry or retained plaintext without a real Recovery/retry owner; whether an aborted packet number becomes ACK-valid/reusable history; whether receiver ACK obligations can escape terminal teardown; and whether preserving lifetime diagnostics re-opens live ownership.

## Result — no new BLOCKER/HIGH in this bounded scope

No concrete current-semantics defect was found in the reviewed R9-6 ownership/resource surface.

1. **First-send admission is transactional.** `ReliableUdpRuntime::on_packet_sent` first bounds congestion and retained plaintext, then commits Recovery/Reno ownership. A Recovery admission failure releases only plaintext newly reserved by that call. Production socket failure is routed through `abandon_sent`, which removes packet-to-frame ownership, reverses Recovery/Reno charge and committed-send accounting, restores the prior committed packet-number watermark, and releases retained plaintext when no sibling packet copy still owns the frame.
2. **Retransmit copies remain tied to stable plaintext.** `on_retransmit_sent` refuses a copy when the stable frame no longer has retained plaintext. A socket-failed retransmit uses `abandon_retransmit`, reversing that packet copy while deliberately retaining the stable plaintext because the executable PTO retransmit owner is reached only from an already-outstanding frame. The current executable callers do not expose a final-copy-loss -> orphaned socket-failed retransmit path without another Recovery/retry owner.
3. **ACK/loss retirement reconciles all owners.** `apply_ack` consumes the Recovery result, removes every retired packet-to-frame mapping, and releases retained plaintext only when no outstanding sibling copy remains and the frame is not scheduled for retransmission. The Recovery outstanding-frame refcount and ACKed-frame marker keep overlapping original/retransmit copies from either premature plaintext release or spurious later retransmission.
4. **Committed packet-number history remains fail-closed.** `Recovery::on_sent` requires strict monotonicity against `largest_sent`; `abandon_sent` restores the exact prior committed high-water using the bounded reservation map; future/never-sent ACKs are rejected before RTT/PTO/loss mutation. Existing focused discriminators cover reuse of the committed `N`/lower values after abort and a fresh `>N` positive control.
5. **Terminal teardown separates live ownership from history.** H-R9-066's `Recovery::quiesce` / `PathRecovery::quiesce` clears live sent/outstanding/ACK-reservation/Reno-charge state while preserving lifetime RTT/PTO/loss/resolved/cwnd diagnostic facts. `ReliableUdpRuntime::teardown` also clears packet-to-frame and retained-plaintext owners; repeated teardown is idempotent. Packet/recovery/ACK/health mutators are fail-closed or inert after teardown, and mutable Carrier-manager access is unavailable.
6. **Receiver ACK ownership is terminal-bounded.** A torn-down runtime refuses new received-packet obligations and `poll_outgoing_ack` is inert, so pending ACK state cannot create post-terminal feedback.
7. **Controlled loss remains distinct from socket failure.** `--drop-r9-data` intentionally keeps the packet Recovery-owned and emits controlled-loss evidence so PTO/retransmission is exercised; ordinary `send_to` failure rolls ownership back and emits no positive sent evidence. The review found no collapse of those two semantics.

## Evidence separation

Developer-persisted exact-tree local provenance for `788a4dc` records:

```text
PYTHONDONTWRITEBYTECODE=1 bash scripts/check.sh   exit 0
git diff --check                                exit 0
clean worktree at pushed SHA
UTC 2026-09-19T00:55:55Z -> 2026-09-19T01:00:41Z
Linux x86_64
rustc 1.98.0
```

GitHub-hosted Rust CI for the same exact SHA is run `35410930506`, completed successfully. Hosted CI is cross-evidence only. This reviewer did **not** execute a reviewer-local clean-tree gate and does not claim one. No wire decoder/parser/crypto framing code changed in the reviewed closure, so a new reviewer fuzz run was not mechanically required.

## Exclusions / non-decisions

This review does not choose or change D019 source-retention/no-reset policy, `SessionRuntime.events` capacity/retention, TTL/LRU/history-size/capacity/security values, adversarial-load suitability, signing/key-custody/SBOM/publication policy, core Session/Carrier/ACK/crypto/wire architecture, destructive migration, release/freeze/production authority, or any WAN/live conclusion.

**R9-6 disposition:** CLOSED, bounded no-finding after H-R9-066 repair. Continue immediately to R9-7 process/result truth; do not wait for reviewer cadence.
