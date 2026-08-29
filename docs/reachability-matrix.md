# Reachability matrix artifact

`neko probe` supports a machine-readable local-only probe mode:

```text
neko probe --matrix --target 127.0.0.1:40080 --transport tcp --ip-version ipv4 --timeout-ms 500 --bytes 32 --json
neko probe --matrix --target '[::1]:40080' --transport udp --ip-version ipv6 --json
```

The artifact is defined in `schema/reachability-matrix.v1.json`. Every artifact explicitly records `scope`, `privileged`, `raw_protocol`, and `third_party_scan`. Targets must be loopback, timeout is bounded to 1–5000 ms, and payload is bounded to 1–1200 bytes. TCP and ordinary UDP only are supported; raw sockets, ICMP, and privileged protocols are intentionally excluded. The command never performs public-WAN probing or third-party scanning.

Exit status remains 0 for a reachable case, 1 for a completed failed case, and 2 for invalid arguments. Existing authenticated `probe` behavior is unchanged unless `--matrix` is supplied.
