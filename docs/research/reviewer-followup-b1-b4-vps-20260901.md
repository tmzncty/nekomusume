# Reviewer Follow-up B1–B4 — replacement bounded self-owned VPS evidence

- experiment window (UTC): `2026-09-01T02:54:00Z`–`2026-09-01T03:05:37Z`
- source / binary identity: GitHub `main` and fresh detached worktree at `4ecd29979e633cea02c17abb8e448e1e37ab8ad7`; `neko-cli` 0.1.0, Linux x86_64, SHA-256 `7bd5561e011c78d6dcc83d47d11a010e4d09d072128bd0aa94424b920968fb9d`
- endpoints: two self-owned hosts; IPv4; addresses and disposable identity material intentionally omitted
- authorization/bounds: standing VPS lab authorization; unprivileged ports 40080–40096; concurrency 1; no firewall, route, qdisc, namespace, capture, or production-service change
- selected canonical version: 0 in negotiated failover events; generic rows completed negotiation and authenticated Noise before application echo

## B1 — separate generic TCP/UDP current/current sanity

PASS on real cross-host sockets.

| row | parameters | result |
|---|---|---|
| TCP | count=3, payload=37 B, port=40092, max=10 s | client/server exit 0; authenticated application echo 111 B; server STOPPED |
| UDP | count=3, payload=37 B, port=40093, max=10 s | client/server exit 0; authenticated application echo 111 B; server STOPPED |
| malformed TCP negotiation | 3 malformed bytes in one bounded framed hello, port=40094, max=5 s | fail closed; server nonzero; no application success/STOPPED row |

One initial path candidate accepted TCP but did not pass this probe's UDP negotiation and was abandoned; the independent owned-host path above passed both transports. This is behavior/reachability evidence, not performance evidence.

## B2 — controlled endpoint-stop failover/resume

PASS on real self-owned UDP and TCP sockets: count=3, payload=37 B/record, 111 application bytes, ports 40095/40096, max=10 s, concurrency=1.

Observed authenticated/structured order:

1. UDP canonical negotiation selected version 0 and UDP Noise authenticated.
2. First logical record completed; `udp_delivery_ack_validated` reported encrypted acknowledgement ciphertext length 86 B.
3. The next logical range remained unconfirmed at explicit `controlled_udp_stop` (`bounded_application_fault_injection`).
4. TCP canonical negotiation selected version 0; fresh Noise-authenticated resume passed `tcp_resume_guard` / server `tcp_resumed`.
5. Resumed unconfirmed ranges produced two `tcp_delivery_ack_validated` events, each encrypted ciphertext length 86 B.
6. Client reported ordered completion of 3 records / 111 application bytes; server reported exactly 3 records / 111 application bytes. Real final state therefore showed no missing or duplicate application delivery.

Classification: **real self-owned TCP/UDP sockets + controlled application endpoint stop**; explicitly **not** natural blackhole/PTO detection and **not** automatic `FailoverController` threshold evidence.

## B3 — small repeated lifecycle sample

Eight sequential, concurrency-1 generic open/exchange/close cycles were attempted, alternating TCP/UDP, with count=2 and payload=16 B (32 application bytes per successful cycle), max=6 s each.

- successful cycles: 7 (4 TCP, 3 UDP)
- failed cycles: 1 (UDP cycle 6: negotiation/authentication completed, then client `echo timeout` and server `data timeout`)
- successful one-direction application bytes: 224
- duplicates/missing on successful real completion state: 0 observed
- failure retained as negative evidence; it was not mechanically rerun

This is a bounded lifecycle sample, not capacity/stress evidence.

## B4 — IPv6 blocker

Not run. The client-side owned host had a global IPv6 address but no IPv6 default route; the server-side owned VPS had neither a global IPv6 address nor an IPv6 default route. Therefore no actual end-to-end owned IPv6 path existed. IPv4 B1–B3 were not stopped. (Separately, the failover client's `addr:port` concatenation is not truthful IPv6 socket syntax; B4 requested only the smallest generic B1 row, so path absence is the operative blocker.)

## Cleanup and evidence handling

PASS after all rows: disposable binaries, identity files and raw logs were removed from both endpoints; no experiment listener on ports 40080–40100 and no process using the disposable experiment path remained. No secrets, addresses, raw payload/log, or capture are committed.
