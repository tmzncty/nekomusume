# Local bounded pre-auth response observation — `f74b823`

Developer-run exact-tree validation of reachable implementation/test commit `f74b8231ed8a088ceba20d4367993094c3ccbae5`.

- an exact charged response attempt was explicitly abandoned and remained counted at 3 bytes / 1 packet
- the next response operation was the first over-limit operation at the unchanged test candidate boundary and rejected atomically; aggregate response accounting remained 3 bytes / 1 packet and never exceeded the configured ceiling
- this deterministic observation exercises retained charging after abandonment and max-plus-one response refusal; it does not select or validate production capacity values
- TCP bounded response writing now restores the prior socket timeout on the injected write-error path as well as success; permit documentation states that dropping alone leaves ownership pending until release/expiry
- focused pre-auth ownership/resource and timeout-error regressions passed
- exact-tree `PYTHONDONTWRITEBYTECODE=1 bash scripts/check.sh`: exit `0`
- `git diff --check`: exit `0`
- gate: `2026-09-09T18:17:36Z` -> `2026-09-09T18:19:28Z`
- host: `Linux 6.8.0-137-generic x86_64`
- Rust: `rustc 1.98.0 (88d9e12ae 2026-08-18)`
- initial/final source tree: clean

This is one bounded deterministic local observation at exact existing test limits. It is not adversarial-load capacity/suitability evidence, independent security review, D019 resolution, public-listener approval, RC, release or production authorization.
