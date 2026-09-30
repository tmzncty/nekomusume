# PLPMTUD real-path survey on tmzn-hk — plan for review (not executed)

**Status: proposed. Nothing in this plan has been executed beyond read-only
reconnaissance (ssh `show`-class queries, `ip route get`, `tracepath`, `ss`).**

This is the changed-hypothesis run replacing the retired-vps-104 plan
(`plpmtud-vps-run-plan-a82b5d7-20260929.md`, now `BLOCKED_ENVIRONMENT` →
superseded in scope by this plan on the administrator-approved `tmzn-hk`
endpoint). One frozen binary, one window, multiple cells = the one
changed-hypothesis invocation.

## Changed hypothesis

The local netns+tc matrix (48+18 cells) proved convergence on synthetic
topologies. The changed hypothesis: **on real heterogeneous overlay paths with
path-specific PMTUs (1500 direct, 1500 tunnel inner, ~1000 through a small-MTU
wg link), the frozen binary still converges to each path's true PMTU or stops
bounded — never a false success.** Phase B (tc silent blackhole) is **cancelled**
per the hard boundary: no tc anywhere. The 1000-MTU link is a natural
blackhole candidate.

## Endpoint facts (recon, read-only, 2026-09-30)

- `tmzn-hk` (ssh alias, passwordless sudo — not needed for this run), 1c/2G,
  uptime 406d. Ports 40080–40100 free. `tracepath` present.
- HK addresses (aliases only in repo, real values in the local non-repo map):
  `hk-public` (public, reachable from here via a direct exception route on the
  physical NIC, bypassing the Meta TUN — verified `ip route get`), `hk-ovl-a`
  (tun-hk tunnel peer, MTU 1500), `hk-ovl-b` (tun-qd/tun-hk-direct side, MTU
  1500), `hk-ovl-c` (tun-rafa side; from here routed via wg-hy-102, **MTU 1000**),
  `hk-ecmp` (eth0 addr; OSPF-equal-cost over tun-hk + tun-hk-direct, per-flow
  hash).
- Local egress MTUs: physical 1500; tun-hk/tun-hk-direct 1500; wg-hy-102 1000.
- tracepath oracles (prechecked, read-only): ovl-a → 1500 (1 hop); ovl-b → 1500
  (1 hop); **ovl-c → pmtu 1000, 30 hops no reply** (UDP probes die in the small
  tunnel — expected for >1000-byte probes; the cell is still valid for the
  binary, which probes authenticated sizes and accepts timeout+bound-lowering;
  precheck noted as such); hk-public → tracepath timed out (public path filters
  UDP probe replies; oracle = `ip route get` + python IP_MTU probe only).
- The probe client sets `IP_MTUDISC_PROBE` (DF on) on the **Session socket**
  (`pmtu_socket.rs`; probes ride the Session socket). With DF on and a local
  egress MTU of 1000, any probe >1000B sent on the ovl-c cell returns
  **EMSGSIZE at send time** — the binary lowers the search bound on
  `plpmtud_probe_emsgsize_retry` and, if the v4 base 1278 exceeds the local
  MTU, exits 2 with `plpmtud_failed reason=base_incompatible` (code-verified
  branch). Both outcomes are recorded, not retried, and are exactly the
  "BASE unreachable" behavior 85461 flagged as likely-unhandled — it IS
  handled (exit 2, fail-closed), and this run exercises it on a real path.
- `kernel_path_mtu` exists in the library but the client emits no such
  diagnostic; the third oracle is a userspace python probe (connect UDP socket
  to the cell destination, read `IP_MTU`) — no binary change.

## Cells (5, one client session each; aliases only)

| cell | server bind (HK) | client --addr | expected oracle | question |
|---|---|---|---|---|
| C1 hk-public | hk-public:40090 | hk-public:40090 | ~1500 (route+IP_MTU) | clean WAN direct path convergence |
| C2 hk-ovl-a | hk-ovl-a:40091 | hk-ovl-a:40091 | 1500 | tunnel inner path |
| C3 hk-ovl-b | hk-ovl-b:40092 | hk-ovl-b:40092 | 1500 | second overlay, different tunnel |
| C4 hk-ovl-c | hk-ovl-c:40093 | hk-ovl-c:40093 | **1000** | small-MTU real blackhole; BASE-vs-MTU boundary (base 1278 > 1000 → expected fail-closed exit 2, `plpmtud_probe_emsgsize_base_incompatible`) |
| C5 hk-ecmp | hk-ecmp:40094 | hk-ecmp:40094 | 1500 (both members) | per-flow hash path selection; repeated sessions may hash to different tunnels — convergence must hold on both; drift is recorded as ECMP evidence, not failure |

Per cell: 2 client runs (short) + 1 long, matching the local matrix's session
shape; per run the client records the full plpmtud event stream (probe_sent /
acked / timeout_retry / emsgsize_retry / converged / converged_by_timeout /
failed).

## Oracles per cell (all userspace, read-only, run before the binary)

1. `tracepath -n <dst>` (where it answers);
2. `ip route get <dst>` → egress interface + its MTU;
3. python `IP_MTU` probe on a connected UDP socket (the value the kernel gives
   the binary);
4. recorded into the cell's evidence JSON as `oracle_tracepath`,
   `oracle_route_mtu`, `oracle_kernel_ip_mtu`.

## Command template (aliases; real values resolved from the local map at run time)

- Deploy (once): `scp` frozen `neko-cli` (sha `efe5d922e50d…` recorded in
  `check-gate-0522e14-20260929.md`) to `tmzn-hk:~/neko-lab/<run-id>/`;
  `sha256sum` verified on both ends; `chmod 700`; no other files.
- Server (per cell): `~/neko-lab/<run-id>/neko-cli server --transport udp
  --bind <cell-addr>:<port> --port <port> --identity <srv-identity>
  --client-key <pub> --count <count> --duration <dur> --plpmtud` under
  `nohup setsid`, bound to the specific cell address, `--duration` capped
  (short cells 60s, long 180s), stdout→log; readiness = log
  `lifecycle_state=READY readiness=true`.
- Client (per run): `neko-cli client --transport udp --addr <cell-addr>:<port>
  --port <port> --identity <cli-identity> --server-key <pub> --count <n>
  --bytes 32 --duration <dur> --plpmtud --diagnostic --json
  --experiment-id plpmtud-hk-<cell>-<run>` (fresh identity pair per cell).
- One cell at a time, sequential; server for cell N is killed and reaped
  before cell N+1 starts.

## Hard boundaries honored

- User-space processes only on both ends: no iptables/nft/ufw/tc/sysctl/ip
  rule/route/WireGuard/FRR changes, no service restarts, no sudo use at all.
- No tc on the physical NIC (Phase B cancelled; natural MTUs instead).
- Ports within 40080–40100 (40090–40094 chosen, all free now), servers bound
  to specific addresses, `--duration` caps, binary under `~/neko-lab/<run-id>/`.
- Minimum standing-authorization load: 5 cells × 3 runs × ≤64-byte payloads
  plus probe padding (≤1500B each, ≤ ~40 probes per run).
- Repo notes use aliases only; the real map stays in a local non-repo file.
- If the cloud `YJ-FIREWALL-INPUT` chain blocks UDP reachability on hk-public,
  the cell is recorded `BLOCKED_ENVIRONMENT` and the firewall is not touched.

## Cleanup verification (per cell, recorded)

- HK: `pgrep -f neko-lab/<run-id>` empty; `ss -H -lntup | grep :<port>` empty;
  `rm -rf ~/neko-lab/<run-id>` verified absent at the end of the whole run.
- Local: client processes exited (each run is bounded); no local listeners in
  40080–40100 at the end.
- Per-cell cleanup evidence goes into the cell JSON before the next cell starts
  (same discipline as the owned-lab harness).

## Failure classification (recorded, not retried)

- `converged_exact` — confirmed == oracle PMTU;
- `converged_lower` — confirmed < oracle but consistent (e.g. tunnel inner
  MTU smaller than route MTU) — compared against all three oracles;
- `emsgsize_base_incompatible` (exit 2) — C4-expected: base > real path MTU,
  fail-closed is correct behavior, recorded as the measured BASE-unreachable
  outcome (RFC 8899 error state; fix, if wanted, is a follow-up slice);
- `deadline_bounded` — search did not finish within the session budget;
- `handshake_or_exchange_failed` — reachability/filtered (incl. potential
  hk-public firewall block → `BLOCKED_ENVIRONMENT`);
- `false_success` (FAIL) — confirmed > real path MTTU: must not happen; any
  occurrence aborts the remaining cells and is reported immediately.
- ECMP drift (C5): runs converging to different-but-each-correct values across
  hashed paths are recorded as ECMP evidence, not failure.

## Evidence & write-back

Per-cell JSON + client/server logs under
`artifacts/plpmtud-hk-survey/<run-id>/` (aliases only); summary MD table with
all three oracles per cell; evidence note + status/ledger row update (live PMTUD
`BLOCKED_ENVIRONMENT` → whatever the evidence supports: per-path convergence on
real overlays, with the VPS-shaped question re-scoped to this endpoint) after
85461 review of the raw results.
