# HY2 comparison setup — dedicated VPS (2026-08-30)

## Scope and safety boundary

This record covers only the authorized dedicated VPS `192.168.122.1`. It does
not authorize or describe production-network traffic, third-party probes, WAN
scanning, route/firewall changes, or modification of the existing service.
The existing Hysteria process and `/etc/hysteria/server.yaml` were inspected
read-only and were not restarted, reconfigured, or stopped.

Any experiment must use a newly allocated high port, a temporary config under
`/tmp`, a generated test certificate/authentication secret, a finite timeout,
and an explicit cleanup trap. Do not copy credentials from the existing
service. Bind only the dedicated VPS address; do not bind `0.0.0.0` or alter
firewall/NAT rules.

## Exact HY2 artifact selected

- Hysteria2: **v2.9.3**
- Build date: `2026-06-27T06:53:09Z`
- Commit: `2d973f9513ef661d1922d6d14acb37945caef47d`
- Toolchain: `go1.26.4 linux/amd64`
- SHA-256: `66dbdb0608f25f3057b433afe975a9fc1af2ca8e512479e294988b3ef363d6c1`
- Path: `/usr/local/bin/hysteria`

This is an explicit installed artifact, not an unpinned `latest` download.
No package or binary was installed by this setup because the selected artifact
was already present. The VPS also already has `iperf3 3.16-1build2` and
`socat 1.8.0.0-4ubuntu0.1`; these are inventory facts, not comparison runs.

## Comparability contract

The repository workload contract is `scripts/bench/compare-hy2.sh` plus
`docs/bench/hy2-comparison-workload.md` at baseline
`aa5dd93e8616dd4fdba026dd3ebab419eb959046`.

A valid comparison requires both implementations to consume the same
zero-filled payload, exact byte count and SHA-256, target, timeout, run count,
server/route/MTU/security/load metadata, and to emit the same JSON fields
(`application_bytes`, `fd_count`; `wire_bytes` is nullable). It requires 3–100
runs, loopback-only target, explicit isolated-lab and command-evaluation
consent, `jq`, and GNU `time`. Results with a failed/incomplete exchange are
not performance evidence.

Recommended HY2-only experimental profile (to be recorded with every run):

```text
version=v2.9.3 sha256=66dbdb0608f25f3057b433afe975a9fc1af2cae512479e294988b3ef363d6c1
transport=HY2/QUIC; target=dedicated-VPS-only; port=<fresh temporary high port>
payload=1200 zero bytes; runs=5; per-run-timeout=30s
security=TLS certificate generated for this experiment; unique random auth;
         no reuse of /etc/hysteria/server.yaml credentials
load=single stream, one client, no background generator
MTU/server/route=<measured and recorded, unchanged for both implementations>
```

The profile is a setup contract, not evidence: the required equivalent
Nekomusume application benchmark command and a matching HY2 application
exchange were not available in this inspection.

## Inspection evidence and commands

Executed as `tmzn` with non-interactive sudo where needed:

```sh
hostname; uname -a; cat /etc/os-release
dpkg-query -W -f='${Package}\t${Version}\n' | grep -Ei 'hysteria|iperf|netperf|ncat|socat'
/usr/local/bin/hysteria version
sha256sum /usr/local/bin/hysteria
ss -lntup
pgrep -af hysteria
sudo stat /etc/hysteria/server.yaml
```

Observed host: Ubuntu Linux, kernel `6.8.0-137-generic`, x86_64. Existing
process: `/usr/local/bin/hysteria server -c /etc/hysteria/server.yaml` (PID
observed during inspection); existing config listens on `:38525` and contains
TLS/auth/forwarding settings. Secret values were deliberately not recorded.

## Execution status and cleanup

No TCP/UDP comparison was executed. The fail-closed harness correctly limits
targets to loopback and there was no equivalent HY2/Nekomusume command pair
with equal application semantics. Therefore no throughput, latency, loss, or
fairness claim may be made from this setup.

No temporary listener, route, firewall rule, package, config, certificate,
credential, or capture was created. Cleanup verification was the unchanged
listener/process inventory after read-only inspection. If a future bounded
run is approved, use `trap` cleanup, verify the selected temporary port is
absent from `ss -lntup`, kill only the recorded experiment PID, and remove
only files created beneath `/tmp/neko-hy2-*`.
