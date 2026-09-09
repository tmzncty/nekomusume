# Local exact-tree test port lease validation — `acbd4bf`

Developer-run local validation of reachable exact test-fixture commit `acbd4bf44a9a7c13e615cd9266aa3caf897e8417`.

- replaces probe-then-release test-port selection with a `PortLease` that owns both TCP and UDP bindings while selecting ports inside the unchanged `40080..=40100` operator range
- all dual-port failover process fixtures acquire two distinct leases and releases them immediately before spawning the server; periodic fixtures acquire one lease and release it at the child-spawn handoff
- this narrows but cannot make the OS API handoff mathematically atomic; it removes the unbounded selection/use gap without changing production bind policy or runtime semantics
- focused selection-loss failover and authenticated periodic positives: exit `0`
- exact-tree `PYTHONDONTWRITEBYTECODE=1 bash scripts/check.sh`: exit `0`
- `git diff --check`: exit `0`
- exact-tree gate: `2026-09-09T16:43:36Z` -> `2026-09-09T16:45:28Z`
- host: `Linux 6.8.0-137-generic x86_64`
- Rust: `rustc 1.98.0 (88d9e12ae 2026-08-18)`
- initial/final source tree: clean

This is developer-local test-fixture/CI evidence. It is not reviewer-executed CI, hosted CI, VPS/WAN evidence, failover reliability evidence, independent security review, RC, release, or production authorization.

The intermediate exact `7d26330` gate remained red because only one failover fixture had adopted the lease; `executable_loopback_controlled_udp_stop_tcp_resume` still used fixed ports and failed before start. Exact `acbd4bf` extends the lease to every dual-port failover process fixture and is the first green exact implementation tree for this repair.
