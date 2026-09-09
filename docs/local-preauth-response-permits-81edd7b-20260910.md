# Local exact-tree pre-auth response-permit validation — `81edd7b`

Developer-run validation of reachable exact implementation/test commit `81edd7ba358562e5e6013f889b76e3243dff44fe`.

- response permits are bound to a process-local monotonic controller identity, state identity, and monotonic response-attempt identity; overflow fails closed
- each state admits at most one pending response attempt; completion, suppression, abandon, expiry and release invalidate ownership without refunding response accounting
- effective send deadline is the earliest of the configured 100 ms ceiling, state idle/lifetime boundary, and an already-earlier observable socket write timeout; reusable sockets restore their prior timeout
- intentional failover loss seams settle the charged attempt as suppressed rather than leaving pending ownership
- endpoint-rebind UDP negotiation/Noise responses now use the same exact-charge bounded wrapper; responder inventory is 8 surfaces / 7 admission sites
- focused ownership/deadline, ordinary TCP/UDP, failover-loss and endpoint-rebind positives passed
- exact-tree `PYTHONDONTWRITEBYTECODE=1 bash scripts/check.sh`: exit `0`
- `git diff --check`: exit `0`
- gate: `2026-09-09T17:29:17Z` -> `2026-09-09T17:31:09Z`
- host: `Linux 6.8.0-137-generic x86_64`
- Rust: `rustc 1.98.0 (88d9e12ae 2026-08-18)`
- initial/final source tree: clean

This is developer-local correctness/CI evidence, not reviewer-executed or hosted CI, adversarial-load evidence, independent security review, RC, release, public-listener approval or production authorization. RSEC-001 and D019 remain open at their existing boundaries.
