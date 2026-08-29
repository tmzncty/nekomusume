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
