# N8 self-owned endpoint and loopback audit — 2026-08-31

## Scope and provenance

- Exact authoritative parent and audit HEAD before this report: `5e2ac795747582f5f1f36ad537234305c300b47f`.
- Isolated detached worktree: `/tmp/nekomusume-n8` on the administrator-owned host reached as `192.168.122.1`.
- No third-party target, scanning, firewall change, route change, packet capture, privileged fault injection, or repository identity use.
- `neko-server.identity` was never present in the worktree. The authoritative checkout's identity was read only once for a SHA-256 preservation record; its contents are not stored.
- The `192.168.122.1` rows exercise a self-owned non-loopback host address from the same host. They are endpoint/runtime evidence, **not** independent public-WAN path evidence and not NAT traversal evidence.

Raw evidence is under `artifacts/n8-20260831/`; `sha256sums.txt` covers the evidence files existing before the report.

## Supported runtime matrix

All rows use the release binary recorded in `binary-sha256.txt`, fresh temporary server/client identities, an explicit client allowlist, a Noise-authenticated handshake, four authenticated 32-byte request/echo records, and bounded durations.

| Target class | Transport | Address | Result | Evidence |
|---|---|---|---|---|
| Loopback | TCP | `127.0.0.1:40080` | PASS | `tcp-loopback-{server,client}.log` |
| Loopback | UDP | `127.0.0.1:40081` | PASS | `udp-loopback-{server,client}.log` |
| Self-owned host endpoint | TCP | `192.168.122.1:40082` | PASS | `tcp-vps-{server,client}.log` |
| Self-owned host endpoint | UDP | `192.168.122.1:40083` | PASS | `udp-vps-{server,client}.log` |

The CLI emits `probe_ok` only after all requested authenticated echoes complete. Each server log records readiness and terminal `lifecycle_state=STOPPED readiness=false`.

### Multi-record and multistream

- Multi-record is covered in every matrix row with `--count 4`.
- The supported experimental multistream CLI is TCP-only. Loopback TCP with 3 streams × 4 records × 17 bytes passed (12 records, 204 payload bytes), with ordered JSON evidence in `multistream-{client,server}.log`.
- UDP multistream: **N/A / unsupported by this CLI**, so it was not represented by a substitute fixture.

### Shutdown and reconnect

- Natural completion after the bounded exchange reached `STOPPED` in every server row.
- Explicit SIGTERM shutdown reached `STOPPED`; evidence: `lifecycle-sigterm.log`.
- Fresh-process reconnect/rebind passed twice on the same TCP port and twice on the same UDP port, with two authenticated records per cycle; evidence: `reconnect.tsv` and `reconnect-*-{client,server}.log`.
- Post-run `listeners-after.txt` is empty: no audit listener remained.

## Unsupported dimensions and claim boundaries

| Dimension | Classification | Reason |
|---|---|---|
| UDP-to-TCP failover | **NOT_RUN in N8 runtime matrix** | The CLI has an experimental deterministic UDP-blackhole/TCP-resume command, but this run had no real independently controlled WAN fault path. Existing unit/integration coverage is not WAN evidence. |
| NAT rebinding / address migration | **N/A / unsupported runtime CLI** | No CLI mechanism or independent NAT path was available; the self-owned host-address row is not NAT traversal. |
| Migration-back / multipath | **N/A / unsupported runtime CLI** | State-machine fixtures/tests exist, but no supported live endpoint command proves this dimension. |
| PMTUD / PLPMTUD | **N/A / unsupported runtime CLI** | Deterministic tests exist; no live CLI exposure and no controlled MTU path were available. |
| Runtime key update | **N/A / unsupported runtime CLI** | `key-update` is reported as a fixture capability, not a live peer command. It was not promoted to endpoint evidence. |
| Public-WAN traversal | **BLOCKED / not evidenced** | This execution had one host and no independently observed public remote path. Prior checked-in VPS datasets were not relabeled as results of this run. |

## Verification and cleanup

- Baseline full test gate: `cargo test --workspace --locked` passed before evidence generation.
- Release build: `cargo build --workspace --locked --release` passed; capability output is in `capabilities.json`.
- Because this audit adds documentation/evidence only, the full repository gate was rerun after the change (see `full-gate.log`).
- Temporary identities were removed. Ports `40080`–`40087` had no remaining listeners.
- The authoritative checkout's pre-existing untracked `neko-server.identity` was not modified; before/after SHA-256 records match.
