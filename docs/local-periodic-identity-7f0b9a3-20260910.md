# Local exact-tree periodic identity/config closure — `7f0b9a3`

Developer-run replacement local validation of reachable exact implementation/test commit `7f0b9a326a90254b4e37810012a37186880dd997`.

- focused periodic deterministic rejection matrix: exit `0`; includes malformed/31/33-byte peer keys, malformed address/bind, and address/bind port mismatch with absent identity paths remaining absent
- focused authenticated periodic positive remained green
- the previously red failover selection-loss process test now selects two currently available ports inside the unchanged operator range `40080..=40100`; focused test exit `0`
- diagnosis: direct reproduction retained `neko: TCP bind failed`; socket inspection showed TCP `TIME-WAIT` on fixed port `40092`, and a direct server invocation on that pair failed before the diagnostic start event. The repair is test-only port selection and does not change production port policy or failover runtime semantics.
- exact-tree `PYTHONDONTWRITEBYTECODE=1 bash scripts/check.sh`: exit `0`
- `git diff --check`: exit `0`
- exact-tree gate: `2026-09-09T16:28:16Z` -> `2026-09-09T16:30:08Z`
- host: `Linux 6.8.0-137-generic x86_64`
- Rust: `rustc 1.98.0 (88d9e12ae 2026-08-18)`
- initial/final source tree: clean

This supersedes the red result only as the latest green tested tree; [`local-periodic-identity-363177c-20260909.md`](local-periodic-identity-363177c-20260909.md) remains unchanged historical evidence of the earlier failed full gate. This is developer-local CI/security-behavior evidence, not reviewer-executed CI, hosted CI, VPS/WAN evidence, independent security review, RC, release or production authorization.
