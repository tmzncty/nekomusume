# Reviewer Follow-up B — process-resource sampler measured use

- window (UTC): `2026-09-01T03:32:06Z`–`2026-09-01T03:32:29Z`
- source/binary identity supplied to the sampler: GitHub `main` parent `33310fc50a89eaf366b3804f6435e95ca69a2591`; fresh Linux x86_64 `neko-cli` build
- profile: self-owned client to self-owned VPS, IPv4 TCP, unprivileged port 40097, concurrency 1, count=3, payload=37 B, 111 one-direction application bytes, maximum 10 s
- privacy/scope: addresses, identity material, raw logs and payload omitted; no capture, route, firewall, qdisc, namespace or production-service change

Both roles passed authenticated application echo and exited 0. The schema and
dependency-free validator accepted both samples.

| role | elapsed | CPU user/system (`wait4`) | max RSS | peak FD | owned experimental listener/socket peak | application bytes | cleanup |
|---|---:|---:|---:|---:|---:|---:|---|
| server | 0.846251 s | 0.038705 / 0.003055 s | 10,228 KiB | 5 | 1 | 111 | process reaped; owned socket after exit 0 |
| client | 0.633882 s | 0.032660 / 0.003958 s | 9,996 KiB | 4 | null (no owned port supplied) | 111 | process reaped |

RSS is `max(sampled /proc/<pid>/status VmRSS, wait4 ru_maxrss)` with Linux KiB
units. FD is sampled from `/proc/<pid>/fd`. The server socket count intersects
that process's socket FDs with the one caller-owned port; the client value is
truthfully unavailable/null rather than zero because no client-owned port was
supplied.

Cleanup PASS: disposable binary, identities, sampler copies, validator copy,
raw logs and raw JSON were removed from both hosts; no experiment listener on
port 40097 and no process using the disposable VPS binary remained.

This is one small non-performance resource observation. It is **not** a
capacity, stress, superiority, production-readiness or public-reachability
claim. It does not erase or reinterpret the earlier B3 lifecycle result: that
sample remains exactly 7/8 successful with its failed UDP cycle retained as
negative evidence.
