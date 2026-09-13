# Independent bounded DatagramRuntime review — exact `9697ee7`

Bounded independent review of `DatagramRuntime`/`DatagramCounters`/`SessionDatagram`/`DatagramError` in `crates/neko-session/src/lib.rs` (`:1683-1763`), against `docs/spec/m2-unreliable-datagram.md`, at reachable exact `9697ee7a0045b39c6ef46c1951fefea486ccf13b`. This reviews the unreliable datagram runtime itself — the observability `record_datagrams` mixed-drop fix is a downstream projection and is not treated as a substitute. Not a WAN claim, not an independent security/release approval.

## Challenged invariants and result

- **Counter relationships — holds.** `send` always `offered++` (`:1733`), then exactly one outcome: closed → `dropped++` + `Terminal` (`:1734-1736`); oversize → `rejected_oversize++` + `Oversize` (`:1738-1740`, note oversize is tracked separately, not added to `dropped`); queue-full → `queue_dropped++` + `dropped++` + `QueueFull` (`:1742-1745`); success → push + `admitted++` (`:1750`). Invariant `offered == admitted + dropped + rejected_oversize` holds, and `queue_dropped` is a subset of `dropped` (queue-full drops also increment `dropped`), so downstream `dropped - queue_dropped = terminal/other` attribution stays truthful. `admission_send_receive_and_drop_are_bounded` pins the exact `{3,1,1,1,1,1}` outcome.
- **Closed-state behavior and queue release — holds.** `close` sets `closed` and `queue.clear()` (`:1728-1730`); post-close `send` is `Terminal` (counted as `dropped`) and `receive` is `Terminal`. `close_drops_without_ack_or_retransmission` confirms queued data is released and `opened` is not advanced.
- **Payload/queue bounds before allocation — holds.** `payload.len() > max_payload → Oversize` before `to_vec` (`:1738` vs `:1748`); `queue.len() >= max_queue → QueueFull` before `push_back` (`:1742` vs `:1747`). Constructor rejects `max_queue == 0 || max_payload == 0`.
- **No ACK/retransmit/order/delivery-evidence promotion — holds.** `DatagramRuntime` has no ACK, retransmission, ordering, or delivery ledger; `receive` is a plain `pop_front` that increments `opened` only on a real pop.
- **Mixed reliable/unreliable resource separation — holds.** `DatagramRuntime` is a self-contained queue+counters; it shares no state with `SessionRuntime`'s reliable send/recv/window surface.
- **Saturating counters cannot fabricate a stronger claim — holds.** Every counter uses `saturating_add`; on overflow a counter can only *under*-report (fail-safe), never inflate admitted/opened or hide drops.

## Evidence

- `cargo test -p neko-session` on exact `9697ee7`: `admission_send_receive_and_drop_are_bounded`, `close_drops_without_ack_or_retransmission` pass.
- No code change; no defect found. No new `READY_LIVE` question; release/governance state unchanged.
