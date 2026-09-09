# Local exact-tree identity/config validation — `899b190`

Developer-run local validation of reachable exact implementation/test commit `899b190379c7a1dd213bfb520eff221ef15cfe39`.

- start UTC: `2026-09-09T08:28:09Z`
- end UTC: `2026-09-09T08:30:00Z`
- host: `Linux 6.8.0-137-generic x86_64`
- Rust: `rustc 1.98.0 (88d9e12ae 2026-08-18)`
- initial source tree: clean
- `PYTHONDONTWRITEBYTECODE=1 bash scripts/check.sh`: exit `0`
- `git diff --check`: exit `0`
- source tree after stable gate: clean
- native x86_64 package build outside the source tree: exit `0`
- package archive SHA-256: `dde4acc5659e128e3ffb6cced6ced13c90fcfb59986d7f2220398fce3e99db1f`
- packaged binary SHA-256: `a9e69df5264229d571d2e7030dbdbebb669f5cccf5f9216713cbb8769cadaded`
- packaged client with deterministic malformed address and a nonexistent external identity path: exit `2`; identity remained absent
- final source tree: clean

The implementation parses deterministic server trust/bind configuration and client peer/address/payload configuration before automatic long-term identity creation. Network bind/connect failures remain a separate runtime boundary. No identity or key material is retained. This is developer-local CI and packaged-operator evidence only, not independent security review, signing/key custody, RC, release, or production authorization.
