# Isolated VPS candidate experiment — 2026-08-29

## Scope

Target: dedicated `neko-test`, `VM-0-6-ubuntu`, public IPv4 `192.144.192.215`.
The experiment used only the repository candidate binary from commit
`a6e1437` and ports `40080/tcp`, `40081/udp`. One authenticated encrypted echo
was run per transport from the controlled client `192.168.122.1`; payload was
32 bytes and runtime duration was bounded to 5 seconds on the client and 15
seconds on the server. No proxy, forwarding, tunnel, privileged protocol,
third-party scan, or production system was involved.

## Results

- TCP IPv4: authenticated handshake and encrypted echo succeeded; observed
  client-reported elapsed time `94 ms`.
- UDP IPv4: authenticated handshake and encrypted echo succeeded; observed
  client-reported elapsed time `45 ms`.
- IPv6: not tested as a positive path. VPS has no global IPv6 on `eth0` (only
  ULA/link-local addresses), so there was no valid IPv6 endpoint to test.
- The candidate server was one-exchange and was stopped after each probe.

These are reachability observations for this exact host, route, binary and
payload. They are not sustained WAN, NAT/endpoint-change, interoperability,
security, performance-superiority or production evidence.

## Cleanup

The temporary `/root/nekomusume-test` runtime, identities, logs and listeners
were removed. A final listener check found no process bound to ports
`40080-40100`; no firewall changes were made. The VPS remains SSH-only.


## Authenticated IPv4/IPv6 baseline matrix — 2026-08-29

The dedicated client-to-VPS key path was used; no private key was copied between hosts. Each case was a bounded single authenticated echo, three repetitions per payload size (1, 32, 1200 bytes), with temporary listeners on TCP/40080 or UDP/40081 and cleanup after each case.

| Family | Carrier | Payloads | Success | Wall-clock median (ms) |
|---|---|---:|---:|---:|
| IPv4 | TCP | 1/32/1200 | 9/9 | 93–95 |
| IPv4 | UDP | 1/32/1200 | 9/9 | 51 |
| IPv6 | TCP | 1/32/1200 | 9/9 | 82–87 |
| IPv6 | UDP | 1/32/1200 | 9/9 | 42–43 |

Raw machine-readable output: [`docs/data/vps-wan-baseline-2026-08-29.jsonl`](data/vps-wan-baseline-2026-08-29.jsonl). This is an authorized isolated observation, not a throughput result, broad reachability claim, interoperability claim, or production/security approval.


## Repeated baseline and negative-path checks

A second bounded run used five repetitions for each IPv4/IPv6 × TCP/UDP × 32/1200-byte combination: 40/40 authenticated echoes succeeded. Observed wall-clock medians/P95 were IPv4 TCP 91–96/93–98 ms, IPv4 UDP 50/52–53 ms, IPv6 TCP 83–84/85–86 ms, and IPv6 UDP 41–44/45–48 ms. These are small availability samples, not throughput or superiority evidence.

Negative paths failed closed: an unallowlisted client identity was rejected (`rc=2`, 52 ms) and a probe with no listener failed within the bounded client timeout (`rc=2`, 4120 ms). Raw evidence: [`docs/data/vps-wan-negative-2026-08-29.jsonl`](data/vps-wan-negative-2026-08-29.jsonl).

The current public CLI performs one authenticated echo and exits. It therefore cannot honestly establish a long-lived Session, live WAN UDP→TCP failover, NAT rebinding, or continuous key-update stability. Those remain unchecked until a bounded multi-exchange Session runner exists. The existing deterministic and loopback failover evidence remains separate.


## Authenticated multi-exchange WAN smoke — 2026-08-29

The new bounded `--count 8` runner was deployed to the dedicated VPS. One Noise handshake carried eight sequential authenticated exchanges for each IPv4/IPv6 × TCP/UDP case; all four cases succeeded. This validates bounded multi-record use over one carrier, not cross-carrier failover. Raw summary: [`docs/data/vps-multiexchange-2026-08-29.jsonl`](data/vps-multiexchange-2026-08-29.jsonl).

The First Surviving Session gate remains open: the present CLI still creates one transport-bound `SecureSession` per invocation, and does not yet preserve that cryptographic Session across a UDP-to-TCP carrier replacement. No claim of live WAN failover or DELIVERY_ACK is made.
