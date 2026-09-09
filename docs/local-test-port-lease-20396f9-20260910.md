# Local exact-tree test port lease validation — `20396f9`

Developer-run local validation of reachable exact test-fixture commit `20396f91918342f34e94654edacb74bc310f5152`.

- replaces probe-then-release test-port selection with a `PortLease` that owns both TCP and UDP bindings while selecting ports inside the unchanged `40080..=40100` operator range
- the failover fixture acquires two distinct leases and releases them immediately before spawning the server; periodic fixtures acquire one lease and release it at the child-spawn handoff
- this narrows but cannot make the OS API handoff mathematically atomic; it removes the unbounded selection/use gap without changing production bind policy or runtime semantics
- focused selection-loss failover and authenticated periodic positives: exit `0`
- exact-tree `PYTHONDONTWRITEBYTECODE=1 bash scripts/check.sh`: exit `0`
- `git diff --check`: exit `0`
- exact-tree gate: `2026-09-09T17:35:28Z` -> `2026-09-09T17:37:18Z`
- host: `Linux 6.8.0-137-generic x86_64`
- Rust: `rustc 1.98.0 (88d9e12ae 2026-08-18)`
- initial/final source tree: clean

This is developer-local test-fixture/CI evidence. It is not reviewer-executed CI, hosted CI, VPS/WAN evidence, failover reliability evidence, independent security review, RC, release, or production authorization.
