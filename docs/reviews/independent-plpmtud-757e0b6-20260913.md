# Independent bounded PLPMTUD candidate review — exact `757e0b6`

Bounded independent review of `Plpmtud`/`PlpmtudConfig`/`Probe`/`ProbeTimeout`/`PmtuSendOutcome`/`PmtuSocketCeiling` in `crates/neko-reliable/src/lib.rs` (`:814-1045`), against `docs/spec/m2-plpmtud.md`, at reachable exact `757e0b6c056f0bac39c1fb00e24d54cab55f3f72`. This challenges only the socket-free candidate state model — it does not connect PLPMTUD to live sockets/ICMP and creates no live experiment. Not a WAN claim, not an independent security/release approval.

## Challenged invariants and result

- **Base/max MTU + config validation — holds.** `new` rejects `base_mtu < 576`, `max_mtu < base_mtu`, `attempts_per_size == 0`, `max_probes == 0`, `blackhole_threshold == 0` as `InvalidConfig` (`:908-914`).
- **Binary-search arithmetic near u16 edges — holds.** `start_probe` picks `size = confirmed + (upper - confirmed).div_ceil(2)` (`:952-953`); since `confirmed <= upper <= u16::MAX` always, the u16 addition `confirmed + ceil(gap/2)` cannot overflow — the result stays `<= upper`.
- **One outstanding probe + total probe-limit ordering — holds.** `start_probe` rejects `ProbeOutstanding`, `NoProbeNeeded` (converged), then `ProbeLimit` in that order (`:943-951`).
- **Checked probe-id exhaustion + atomic failure — holds.** `next_probe_id.checked_add(1)` returns `ProbeLimit` before `probes_started += 1` (`:960-964`), so a failed start does not consume budget.
- **ACK binding to exact generation/id/size — holds.** `acknowledge` requires `generation == self.generation == probe.path_generation`, `id == probe.id`, and `size == probe.size`; `StaleAck`/`WrongSize`/`NoOutstandingProbe` all reject before `confirmed`/`outstanding` mutation (`:975-984`).
- **Stale/duplicate/wrong-size rejection without mutation — holds.** All rejections precede state change; a duplicate ACK after completion returns `NoOutstandingProbe`. Covered by `stale_reordered_duplicate_and_wrong_ack_are_rejected`.
- **Timeout/retry/upper-bound reduction — holds.** `timeout` decrements `attempts_left` for `Retry`, else sets `upper = (probe.size - 1).max(confirmed)` and reports `ReducedUpperBound`/`Converged` (`:988-1001`) — probe-local loss evidence only.
- **Generation reset discards stale evidence — holds.** `reset_generation` resets generation, `confirmed`/`upper`, `outstanding`, `probes_started`, `base_loss_run` (`:1037-1044`).
- **Blackhole fallback is not path-failure evidence — holds.** `observe_confirmed_size_loss` only counts `packet_size <= confirmed` losses and, after `blackhole_threshold`, conservatively drops `confirmed` to `base_mtu` — it returns a bool, never declares the path failed (`:1003-1018`).
- **`on_emsgsize` cannot raise confirmed or fabricate probe evidence — holds.** It only lowers `upper` toward `ceiling.max(confirmed)` and returns `BaseIncompatible`/`RetryAt(confirmed)`; `confirmed` is never raised by an EMSGSIZE hint (`:1019-1031`).

## Evidence

- `cargo test -p neko-reliable` on exact `757e0b6`: `converges_across_path_mtus`, `timeout_retries_then_reduces_without_path_failure`, `stale_reordered_duplicate_and_wrong_ack_are_rejected`, `only_one_probe_and_resource_limit`, `blackhole_fallback_is_bounded_and_progress_resets_counter` pass.
- No code change; no defect found. PLPMTUD remains a non-live candidate; no new `READY_LIVE` question; release/governance state unchanged.
