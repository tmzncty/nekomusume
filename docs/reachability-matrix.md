# Reachability matrix artifact

`neko probe` supports a machine-readable local-only probe mode:

```text
neko probe --matrix --target 127.0.0.1:40080 --transport tcp --ip-version ipv4 --timeout-ms 500 --bytes 32 --json
neko probe --matrix --target '[::1]:40080' --transport udp --ip-version ipv6 --json
```

The artifact is defined in `schema/reachability-matrix.v1.json`. Every artifact explicitly records `scope`, `privileged`, `raw_protocol`, and `third_party_scan`. Targets must be loopback, timeout is bounded to 1–5000 ms, and payload is bounded to 1–1200 bytes. TCP and ordinary UDP only are supported; raw sockets, ICMP, and privileged protocols are intentionally excluded. The command never performs public-WAN probing or third-party scanning.

Exit status remains 0 for a reachable case, 1 for a completed failed case, and 2 for invalid arguments. Existing authenticated `probe` behavior is unchanged unless `--matrix` is supplied.


## Cross-host failover evidence boundary

The exact-`9fd2411` one-run repeated warm-failover result is retained at `artifacts/repeated-warm-failover/9fd2411-typed-negative/result.json`. It is a 0/6 `invalid_cycle_evidence` typed negative at the orchestration/evidence boundary, not a reachability matrix row and not evidence of WAN failover success, runtime correctness, public/general reachability, or production exposure. The separate six-row dry preflight was synthetic and lacked `endpoint_provenance`; it is not counted here.
