# PLPMTUD changed-hypothesis VPS run plan — exact `a82b5d7` (proposed, not executed)

This note is a **plan for approval**. Nothing here has been executed. It follows the
mainline-1 closed loop (missing runtime seam → local validation → one
changed-hypothesis self-owned VPS run → evidence/status reconciliation) and the
standing authorization recorded in `docs/status.md` (bounded self-owned VPS TCP/UDP
execution; no third-party targets, no public listener, no production exposure).

## Exact identity

- implementation commit: `a82b5d7780ce1cbcb6593551d7e5574f9d59dbf0`
  (gate record `0ad8b5d`, first-attempt green);
- release binary built once locally, SHA-256 recorded on both endpoints before any
  exchange;
- topology: local host → `vps-104` (104.244.74.205), IPv4, unprivileged port 40080
  (inside the fixed 40080-40100 range);
- one deployment, one window, two phases, one client Session per phase;
- both endpoints pass `--plpmtud` (the flag is the opt-in gate; without it nothing
  in the binary changes behaviour).

## Hypotheses under test (the change from the local run)

The local loopback run (`check-gate-a82b5d7`) proved only plumbing: loopback MTU is
65536, so every probe up to the 1500 ceiling succeeded. The changed hypotheses for
a real path are:

1. **Real-path convergence.** Over the WAN path to the VPS, an authenticated
   probe-ACK still arrives for every size the path can carry, and the binary search
   converges to the path's real MTU (1500 if the path is clean; a lower value is
   equally valid evidence, recorded as measured).
2. **No false success under DF.** With `PMTUDISC_PROBE` set, a probe above the
   path MTU is never delivered and never ACKed, so the confirmed size can only
   follow authenticated ACKs. (D067 measured that without DF the second send of an
   oversize datagram falsely succeeds; the VPS run exercises the fixed behaviour.)
3. **Probe loss stays probe-local.** A probe that times out only lowers the search
   bound after its two attempts; the client converges or stops bounded, and no
   congestion or retransmission state is involved (the probe loop is outside every
   Session; periodic is pinned probe-free).

## Phase A — clean path (hypotheses 1 and 2)

- Server on the VPS: `server --transport udp --port 40080 --bind <vps-ip>:40080
  --identity … --client-key … --duration 25 --plpmtud --diagnostic
  --experiment-id plpmtud-vps-a-server`.
- Client locally: `client --transport udp --port 40080 --addr <vps-ip>:40080
  --server-key … --identity … --count 2 --bytes 32 --duration 3 --plpmtud
  --diagnostic --experiment-id plpmtud-vps-a-client`.
- Expected: `plpmtud_converged transport=udp confirmed_mtu=<measured>` with exit 0.
  Diagnostics record `plpmtud_probe_sent` / `plpmtud_probe_acked` per size.

## Phase B — controlled silent black hole (hypothesis 3, self-inflicted)

Immediately after Phase A on the same deployment:

- Local egress only: one `tc` filter on the WAN interface scoped to
  `dst 104.244.74.205` dropping IPv4 total length > 1400 (the D067 silent-blackhole
  pattern; no ICMP is generated). The filter is added after Phase A and removed
  immediately after Phase B; `tc -s` drop counters are recorded as evidence.
- Same commands, experiment ids `plpmtud-vps-b-*`, client `--duration 2` so the
  per-size retries fit inside the 30-second client deadline.
- Expected: every probe above 1400 times out twice (its bound lowers); probes at
  or below 1400 are ACKed. The search converges to **exactly 1400** (the filter
  threshold; the first probe is 1389 < 1400 and is ACKed). Diagnostics must show
  `plpmtud_probe_timeout_retry` / `plpmtud_probe_timeout_lowered_bound` events and
  no `plpmtud_probe_acked` above 1400. The client still exits 0 with a converged
  outcome — probe loss never wedges it.

If Phase B cannot converge inside the deadline, that is recorded as measured, not
retried by reflex; the tc filter is removed either way.

## Evidence and bounds

- Retained: both endpoints' diagnostic stdout, exit codes, the client's
  confirmed-mtu line, tc drop counters, window timestamps, binary SHA-256 — written
  to `docs/notes/plpmtud-vps-a82b5d7-20260929.md` in the migration-back format
  (bounded self-owned observation, no credential/payload/private-diagnostic
  retention), then the status.md R row is reconciled per AGENTS.md §126.
- One VPS deployment, one window; no third-party targets; no ICMP/PTB parsing, raw
  IP, SCTP or IPv6; no release flag, READY_LIVE, H-I4-119 or D019 change; no
  same-class retry.
- Cleanup, verified: no listeners or processes left on the VPS, the tc filter
  removed, no local route/qdisc residue.

## Execution status (2026-09-29, blocked before any connection)

Approved by 85461 (same deployment + same window A+B = one changed-hypothesis run;
three conditions recorded). Execution is **blocked before any VPS contact** by two
facts, both requiring an administrator decision:

1. **Host key change.** `ssh vps-104` now presents ED25519
   `SHA256:ecXqLkw0hzk5SXWZ9WyXRyn0w83dM5XMjgWPVlNEO4`, while the pinned
   known_hosts key is a different ED25519 key (last updated 2026-09-26). TCP/22 is
   reachable. No reinstall is documented in the repo. The new key was **not**
   accepted, known_hosts was **not** modified, and no login was attempted. A
   possible reinstall must be confirmed by the administrator before the key is
   replaced.
2. **All traffic to the VPS is routed through the local `Meta` TUN** (198.18.0.2,
   table 2022, rules 9000/9001/9002), not a direct WAN path. The TUN's egress MTU
   is 9000, so the Phase-B `dst + len > 1400` filter would drop every encapsulated
   datagram (the tunnel's L3 length always exceeds 1400 regardless of payload),
   and Phase A would measure the Meta exit node's path, not the local→VPS path.
   Either outcome would be misleading evidence for the mainline hypothesis.

The release binary is built and frozen for the whole run:
SHA-256 `dfea1d31bf1a37101da21df5918a28cd24a19cef0d8fbb291e8d333bfda591b6`
(1,526,168 bytes, exact tree `a82b5d7`). No gate re-run, no rebuild, no redeploy
across A/B, per the approval conditions.

## Original open question for 85461

Does the two-phase shape (A then B in one deployment and window) count as the one
allowed changed-hypothesis VPS run, or should Phase B be cut and only the clean
path be run? The plan is not executed until this is answered.
