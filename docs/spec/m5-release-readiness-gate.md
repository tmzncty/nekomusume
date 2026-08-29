# Release-readiness and real-WAN gate

**Status: blocked; research candidates are not a release.**

The repository contains bounded local/loopback implementations and isolated
netem evidence for authenticated UDP echo, TCP fallback, reliable recovery,
Carrier Manager, key update, PLPMTUD, unreliable datagrams, disabled FEC
candidate, and disabled concurrent/multipath gates. These artifacts prove
selected state-machine and harness properties only. They do not prove public
reachability, sustained WAN operation, NAT/endpoint-change behavior, production
resource safety, protocol interoperability, security approval, or superiority
against Hysteria2.

The following remain deliberately blocked:

- public or non-loopback listeners;
- VPS IPv4/IPv6 reachability and sustained bidirectional WAN tests;
- NAT/endpoint-change experiments outside an isolated authorized lab;
- same-server/same-route/same-MTU/same-load HY2 comparison;
- production tunnel/proxy deployment, migration or replacement;
- any claim of security audit, production readiness, protocol freeze or
  performance superiority.

A future release gate requires an explicit scope decision, isolated test
accounts/endpoints, reproducible metadata and rollback, independent security
review, canonical interoperability vectors, sustained repeated WAN results,
resource/abuse limits, and a reviewed comparison protocol. Until all are
present, `docs/status.md` `reachability` and `production` rows remain blocked.
No command in this repository should silently widen that boundary.


## Bounded probe runtime boundary

The candidate CLI probe runtime is a one-exchange authenticated echo intended
for the isolated test VPS. It is not a service: it permits only TCP/UDP ports
40080–40100, 1–1200-byte payloads, 1–30-second duration, and one server-side
exchange. It performs no forwarding, routing, proxying or tunnel behavior and
must be stopped and cleaned after each experiment.
