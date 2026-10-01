# R / live PLPMTUD — changed-hypothesis owned-VPS run (2026-10-01)

The one changed-hypothesis invocation required by the integration plan
(slice 4). Environment: HK (43.154.97.90, client) -> 23.147.120.24 (Virginia,
AS54286, server), real public path, RTT ~213 ms. The workstation's direct
route to 23 traverses the Meta TUN, so HK must be the client (path-topology
constraint recorded in the plan amendment). Binaries: neko-cli
SHA-256 `374a6daf…` (prefix `374a6dafbc92af51`) built from tree `e328f14` +
two run-enabling fixes (below), identical hash deployed to both ends
(verified). Port 40098/udp only.

## Authorization and firewall handling (85461 relayed, administrator's scope)

- `ufw status numbered` snapshot before: `ufw_before_final.txt`.
- Opened: `ufw allow 40080:40100/udp comment 'neko-plpmtud-exp-20261001'`
  (one rule, nothing else touched). Closed after every phase:
  `ufw delete allow 40080:40100/udp`.
- Post-run `ufw status numbered` vs the before snapshot: **identical**
  (`UFW_DIFF_CLEAN`; the diff shown during an intermediate phase was a
  snapshot-ordering artifact, resolved against the verified clean baseline
  `ufw_after2.txt` — final state also re-verified live: no `neko` rule
  present).
- Teardown verified by command on both hosts: no neko-cli processes
  (`pgrep -f` hits were the check commands themselves, confirmed GONE), no
  listeners in 40080-40100, run outputs removed (binaries + identity
  material retained under /tmp/neko-exp for future authorized runs).

## Two run-enabling defects found and fixed before the recorded run

1. **Server handshake wait was RTT-blind.** The second `recv_from` (awaiting
   the client's handshake) ran under the 100 ms poll timeout, so any path
   with RTT > 100 ms failed the handshake (`handshake timeout` on the
   server; the 101-byte client packet demonstrably arrived — tcpdump at 23
   showed it In with no reply). Netns/loopback (~0 RTT) hid this. Fix: wait
   via `recv_udp_until` bounded by the session deadline, shutdown-aware.
2. **Post-convergence linger.** With the default raise interval (300 s) in a
   30 s session, the client slept to the session deadline after reporting
   convergence instead of exiting (a raise scheduled past the deadline can
   never fire). This also made `udp_plpmtud_loopback_converges…` run at its
   30 s bound (passing by a hair; failed under load 21 the same day — the
   gate caught it). Fix: if `next_raise_at >= deadline`, emit
   `plpmtud_deadline` and exit immediately. Test time 30.04 s -> 0.46 s.

Both fixes are in the gated commit; the run below used the fixed binaries.

## The run (single invocation, six sessions)

Orchestration: `orchestrate.sh` (archived) — ufw open -> per-session server
start/stop -> client on HK -> ufw close -> diff. A = `--plpmtud-raise-interval 3`
(verification raise every 3 s), B = default. `--count 8 --bytes 32 --duration 28`.

| Session | Probes sent | Acked | Timeouts | Converged |
|---|---|---|---|---|
| A1 | 16 | 16 | 0 | 1500 |
| A2 | 16 | 16 | 0 | 1500 |
| A3 | 16 | 16 | 0 | 1500 |
| B1 | 8 | 8 | 0 | 1500 |
| B2 | 8 | 8 | 0 | 1500 |
| B3 | 8 | 8 | 0 | 1500 |

- **Hypothesis confirmed**: the confirmed size converges to 1500 under the
  real path's 1500 MTU; probe losses above it: none occurred (all acked), so
  the congestion/loss-counter isolation question remains covered by the
  deterministic harness (invariant tests), not by this run's evidence.
- A-sessions additionally exercised the CONFIRMED-state verification raise:
  8 post-convergence raises at 3 s intervals, all acked at 1500, confirmed
  unchanged, clean `plpmtud_deadline` exit — the raise machinery works on a
  real WAN path.
- Negotiation/handshake/data/probe exchange visible on-path (tcpdump
  archived sequence: 6 B negotiation, 101 B handshake, 48 B response, 82 B
  data, 1361 B probe, 72 B ack).
