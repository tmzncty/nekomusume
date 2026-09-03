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

### Exact-`c6ab8fd` repeated warm-failover typed negative (2026-09-04)

The exact-`c6ab8fd` one-invocation result is retained at
`artifacts/repeated-warm-failover/c6ab8fd-typed-negative/result.json`.
It is a schema-valid 0/6 `invalid_cycle_evidence` negative at the
orchestration/evidence-collection boundary, with no retry. The private
collector diagnostic records only `missing JSON event: start`; whether that
reflects remote event absence, output framing, or early exit is indeterminate.
It is not a reachability-matrix row and supplies no runtime failover,
WAN/public reachability, or production evidence. Zero-row limitations and
separate external cleanup are documented in the artifact README.

### Final exact-`4a2129e` repeated-failover boundary (2026-09-04)

The final current-line attempt is retained at `artifacts/repeated-warm-failover/4a2129e-final-typed-negative/result.json` (SHA-256 `8af7aab8bb5f8a70c024fd6ef28ac8bc59a53147d86a2bd82a2cf23987fe8d3d`). Exactly one outer invocation, with no retry, retained 0/6 rows and stopped at cycle 1 after 1,165 ms with `invalid_cycle_evidence`; the bounded diagnostic says the server exited before its structured start event. The repeated lane is therefore `BLOCKED_ORCHESTRATION_CURRENT_LINE`. This is not a reachability row or runtime failover failure. Separate later cleanup found zero residue. Historical negatives remain unchanged.

The exact-`60cd40d` periodic attempt is retained at `artifacts/periodic-session/60cd40d-start-timeout/result.json` (SHA-256 `c5a6576f3131f8e3a6bd120b8192fdaed40e50846ace6e124b2d07741ca5f9b2`). Its single no-retry invocation ended at `start_timeout` with SSH exit 255 before client start and before application metrics. Artifact cleanup failed during collection; a later cleanup-only observation separately found zero residue. It is not Session runtime or reachability evidence.
