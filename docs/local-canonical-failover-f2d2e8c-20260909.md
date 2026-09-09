# Local exact-tree canonical failover validation — `f2d2e8c`

Developer-run local validation of reachable exact implementation/test commit `f2d2e8cb8c0bd15a47e673e143fadede7d39ea7c`.

- exact end-to-end regression: `executable_loopback_controlled_udp_stop_tcp_resume`
- canonical launch: `failover --role server` and `failover --role client`
- bounded authenticated UDP -> TCP resume succeeded with the existing ordered-record and delivery-ack assertions
- exact-tree `PYTHONDONTWRITEBYTECODE=1 bash scripts/check.sh`: exit `0`
- `git diff --check`: exit `0`
- exact-tree gate: `2026-09-09T13:10:53Z` -> `2026-09-09T13:12:44Z`
- host: `Linux 6.8.0-137-generic x86_64`
- Rust: `rustc 1.98.0 (88d9e12ae 2026-08-18)`
- initial/final source tree: clean

Actual native x86_64 package smoke from the same exact tree:

- archive SHA-256: `52cb533fb1b4375a406654c130fc18172fb85c9929d572d7371e4ea86250c63c`
- packaged binary SHA-256: `f0ea8decadba842a09871bf594075307703723e2717d9f6a0200b14114350ce3`
- archive smoke: exit `0`
- packaged `failover --role unknown --identity <external-temp-path>`: exit `2`; identity remained absent
- package/source smoke tree: clean

The retained evidence is developer-local executable/package evidence only. It is not reviewer-executed CI, hosted CI, VPS/WAN evidence, failover-performance evidence, independent security review, signing/publication trust, RC, release, or production authorization.
