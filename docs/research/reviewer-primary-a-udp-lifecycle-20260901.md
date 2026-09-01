# Reviewer Primary A — UDP deadline repair and replacement lifecycle evidence

- window (UTC): `2026-09-01T04:47:12Z`–`2026-09-01T04:48:18Z`
- exact authoritative parent: `e7390312e0ca62a088e11fb7c2a6f5060cddcaea`
- tested working tree: this commit's A1/A2 changes; release `neko-cli` SHA-256 `d315e649a4575212052536a50fd947336d07d3e15da9e2234254896333f3e7ee`
- endpoints/path: two self-owned Linux hosts over an existing private IPv4 tunnel path; addresses and disposable identities omitted
- bounds: sequential concurrency 1, ports 40081–40094, count 2, payload 37 B, 74 application bytes per completed cycle, client maximum 4 s, server maximum 6 s

## Deterministic repair evidence

The process sampler now returns a stable `(None, None)` CPU pair when `/proc/<pid>` is unavailable. Its regression imports `read_proc`, uses a guaranteed-nonexistent PID, and proves all unavailable values remain null. Existing collector, validator, schema and timeout/process-group checks pass.

The generic UDP server retains a 100 ms socket poll timeout for shutdown responsiveness but now waits until an explicit post-authentication application deadline derived from bounded `--duration`. `WouldBlock` and `TimedOut` are poll misses before that deadline; peer changes, authentication/payload failures and other socket errors remain terminal. Deterministic process tests prove authenticated first application data delayed 250 ms succeeds and a delay beyond a one-second deadline fails bounded with `data timeout`.

## Replacement lifecycle sample

All 14 scientifically distinct sequential cycles completed: 7 TCP and 7 UDP in strict alternation. Every client exited 0 with `probe_ok`; every server reached READY and then STOPPED with exit 0. Total one-direction application bytes were 1,036 B. No retry occurred within this accepted sample.

Before selecting the accepted path, distinct diagnostic attempts were retained locally: the public TCP path passed while UDP failed before application data, and a first private path intermittently lost a handshake response. Configuration/path changed before the final sample; no failed row was mechanically retried. The old B3 7/8 evidence remains immutable.

Representative sampler observation for the exact binary (short direct-child keygen, not capacity evidence): exit 0; CPU user/system 0.0/0.003434 s; max RSS 9,892 KiB; peak FD 3; sockets unavailable/null because no owned port was supplied; two 10 ms samples; direct-child-only cleanup complete.

Cleanup passed on both hosts: no listener on 40080–40100 and no experiment `neko-cli` process remained. Raw logs, addresses, identities and secrets were not committed.

## Boundary

This proves bounded authenticated cross-host IPv4 TCP/UDP exchange and repaired generic lifecycle behavior on one self-owned path. It does not prove public/general reachability, natural or automatic failover, performance/capacity, production readiness, release readiness, or a broader matrix.
