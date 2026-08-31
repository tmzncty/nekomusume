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

Standing authorization in `docs/standing-vps-lab-authorization.md` already
permits bounded, temporary execution between administrator-controlled clients
and VPS endpoints, including ordinary TCP/UDP listeners, probes, failover,
benchmarks and cleanup within its limits. Such an experiment needs no repeated
per-run permission. Authorization to execute is not release evidence, security
approval, permission for third-party targets, or production authorization.

The following release-evidence and deployment claims remain deliberately
blocked:

- treating a temporary self-owned listener or one bounded run as a public
  service, global reachability result, or sustained bidirectional WAN proof;
- complete independently controlled IPv4/IPv6, failover, long-lived and
  NAT/endpoint-change evidence across the declared release matrix;
- reviewed same-server/same-route/same-MTU/same-load HY2 comparison evidence;
- experiments outside the standing authorization boundary without new explicit
  authorization;
- production tunnel/proxy deployment, migration or replacement;
- any claim of security audit, production readiness, protocol freeze or
  performance superiority.

A future release gate requires an explicit scope decision, reproducible
metadata and rollback, independent security review, canonical interoperability
vectors, sustained repeated WAN results, resource/abuse limits, and a reviewed
comparison protocol. Standing authorization allows the bounded work needed to
collect some of that evidence; it does not satisfy those criteria by itself.
Until all are present, `docs/status.md` `reachability` and `production` rows
remain blocked. No command in this repository should silently widen that
boundary.


## Bounded probe runtime boundary

The candidate CLI probe runtime is a one-exchange authenticated echo intended
for the isolated test VPS. It is not a service: it permits only TCP/UDP ports
40080–40100, 1–1200-byte payloads, 1–30-second duration, and one server-side
exchange. It performs no forwarding, routing, proxying or tunnel behavior and
must be stopped and cleaned after each experiment.
