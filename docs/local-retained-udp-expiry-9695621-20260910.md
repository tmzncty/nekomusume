# Local retained UDP expiry closure — `9695621`

Developer-run exact-tree validation first passed at reachable implementation/test commit `9695621049acc22f3556c04f1d92f08b7548e514`. After a concurrent equivalent process regression was retained and the duplicate test removed, replacement exact `fe4c85a2213cd57f7ca6d1d3a6cd1b7a48bfc4ee` passed the same full gate.

## Terminal expiry evidence

- when `preauth.expire()` consumes the retained failover UDP admission before first application Data, the server now clears the admission, cached first-Noise response, secure transport object, and pre-progress ResumeGuard before any receive/classification path runs
- the focused process regression completes version negotiation and Noise, then delays first Data for 1,200 ms beyond the existing 1,000 ms pre-auth idle deadline
- Data from that expired secure context does not obtain a DeliveryAck or failover success
- the same bounded server process subsequently accepts a fresh admitted negotiation/Noise exchange and exactly one authenticated 16-byte application record
- the server emits one `udp_preprogress_expired` event with aggregate boolean teardown facts; exactly one DeliveryAck is emitted, for the fresh session only
- first-Noise-response-loss recovery, exact-once retained input charging, selection retry and responder inventory (nine surfaces / seven admission sites) remain green
- the stale selection-loss comment no longer claims a removed late post-auth hello assertion

The `--test-first-data-delay-ms` option is a bounded test seam (`0..=2000` ms); it does not introduce a protocol timeout or alter the existing D019 candidate idle/lifetime values.

## Exact-tree gate

- `PYTHONDONTWRITEBYTECODE=1 bash scripts/check.sh`: exit `0`
- `git diff --check`: exit `0`
- final replacement gate: `2026-09-09T22:20:14Z` -> `2026-09-09T22:22:10Z`
- host: `Linux 6.8.0-137-generic x86_64`
- Rust: `rustc 1.98.0 (88d9e12ae 2026-08-18)`
- initial/final source tree: clean

This is developer-local bounded lifecycle/correctness evidence. It does not resolve D019 terminal source-retention policy, prove adversarial-load or production-capacity suitability, constitute independent security review, approve a public listener, or change RC/release/production flags.
