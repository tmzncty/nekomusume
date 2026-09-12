# Independent bounded recovery/observability review — exact `e6a0377`

**Scope:** reviewer static/source review of exact reachable `e6a03776c20c5b32b4676d992cf7fcc613cff2e2`. This is not a developer-local test gate, security audit, WAN result, release decision, or production approval.

## Finding R1 — MEDIUM correctness: strictly-future ACK can create false loss/retransmit evidence

Owner: `crates/neko-reliable/src/lib.rs`, `Recovery::on_ack`.

Current state already tracks `largest_sent: Option<u64>` and rejects non-monotonic local sends in `on_sent`. `on_ack`, however, obtains `largest = ack.largest()` and immediately uses that peer-provided value for packet-threshold loss detection:

```text
largest.saturating_sub(n) >= PACKET_THRESHOLD
```

There is no fail-closed check that `largest <= largest_sent` before RTT/loss processing. Therefore an ACK range whose largest packet number is strictly greater than anything this `Recovery` instance has ever sent can cause real in-flight packets to be classified lost and removed from `sent`, with their FrameIds exposed for retransmission.

This is a current-semantics contradiction, not a new ACK design question. `docs/spec/m2-udp-recovery.md` defines monotonic sender packet numbers and bounded sent-packet recovery state; peer ACK evidence must not manufacture packet-threshold history beyond the sender's own packet-number history.

Required repair boundary:

- add a focused regression first: a strictly-future/never-sent largest ACK must fail closed;
- rejection must be atomic: no RTT mutation, no removal from `sent`, no lost/retransmit output, no PTO reset;
- do **not** require every ACKed packet number to remain present in `sent`, because already-retired/duplicate ACK history is a separate valid case;
- the mechanically determined invalid boundary is only `largest > largest_sent` (and no-sent-state with a nonempty ACK);
- reuse an existing error if semantically adequate; do not redesign ACK encoding or recovery architecture.

After repair: focused `neko-reliable` tests, then clean exact-pushed-tree `scripts/check.sh` + `git diff --check` + provenance.

## Finding O1 — MEDIUM evidence integrity: mixed datagram drop reasons are collapsed to `queue_full`

Owners:

- `crates/neko-session/src/lib.rs`, `DatagramRuntime` / `DatagramCounters`;
- `crates/neko-observe/src/lib.rs`, `Producer::record_datagrams`.

The source relationship is explicit:

- terminal send after close increments `dropped` only;
- queue-full send increments both `queue_dropped` and `dropped`;
- oversize increments `rejected_oversize` and is tracked separately.

Therefore `queue_dropped` is a subset of the generic `dropped` count, not a boolean reason for the whole observation window.

`record_datagrams` currently computes both deltas, but then emits `dropped` events using:

```text
if queue > 0 { "queue_full" } else { "terminal" }
```

for **every** generic drop in the window. If one queue-full drop and one terminal drop occur between observations, both events are reported as `queue_full`. This corrupts structured evidence while leaving transport behavior unchanged.

Required repair boundary:

- add a mixed-delta regression that creates/feeds at least one queue-full generic drop and one terminal generic drop in the same observation interval;
- preserve exact event count and emit queue-full only for the queue-drop subset, with the remaining generic dropped delta classified terminal under current `DatagramRuntime` semantics;
- keep oversize reporting separate as today;
- handle impossible/inconsistent externally supplied counter tuples fail-closed or conservatively without inventing a new transport state/policy;
- do not change `DatagramRuntime` transport semantics or counter definitions merely to simplify projection.

After repair: focused `neko-observe`/`neko-session` tests as appropriate, then clean exact-pushed-tree full local gate + provenance.

## No broader severity claim

These findings do not establish cryptographic compromise, unbounded memory, RSEC-001 capacity suitability, D019 resolution, item-3 completion, or release readiness. They are bounded correctness/evidence defects in already-implemented local state/projection logic.

The remaining high-throughput item-4 review queue in `docs/CHATGPT_HANDOFF.md` remains valid after these two repairs and should be consumed continuously rather than waiting for reviewer cadence.