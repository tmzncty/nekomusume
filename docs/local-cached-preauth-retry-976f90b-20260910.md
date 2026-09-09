# Local cached pre-auth retry closure — `976f90b`

Developer-run exact-tree validation of reachable implementation/test commit `976f90b738315aa7d925b54282a000bcdb139007`.

## Ownership and focused evidence

- failover UDP retains the original source-owned admission after server-side Noise completion while the cached first-Noise response remains retryable
- every matching retry charges the received bytes/work and exact cached response bytes through the existing bounded UDP response permit/deadline helper; it does not create a fresh admission or use raw `send_to`
- the retained owner and cached response are removed on first valid authenticated application Data; the opt-in delayed post-auth duplicate seam is explicitly suppressed rather than emitting pre-auth material after authenticated progress
- a focused wrapper regression completes/suppresses four charged response attempts in one source domain, rejects the first packet beyond the existing four-packet ceiling, proves the state is terminal, and releases it with zero live states
- the first-Noise-response-loss process positive still recovers through one charged cached retry without renegotiating, reauthenticating, changing path generation, or resetting delivery state
- responder inventory now declares nine surfaces / seven admission sites, including the cached failover UDP responder

## Malformed-process correction

- the eight malformed five-byte UDP senders are all pre-bound and kept alive for the complete phase; their eight local source ports are asserted unique before sending
- temporary server/client identity removals are asserted and both paths are absent afterward
- the workload remains one bounded local recovery observation, not same-source saturation or capacity evidence

## Exact-tree gate

- `PYTHONDONTWRITEBYTECODE=1 bash scripts/check.sh`: exit `0`
- `git diff --check`: exit `0`
- gate: `2026-09-09T20:24:29Z` -> `2026-09-09T20:26:22Z`
- host: `Linux 6.8.0-137-generic x86_64`
- Rust: `rustc 1.98.0 (88d9e12ae 2026-08-18)`
- initial/final source tree: clean

This is developer-local bounded correctness/process evidence. It is not D019 source-retention resolution, adversarial-load or production-capacity suitability, independent security review, public-listener approval, RC, release or production authorization.
