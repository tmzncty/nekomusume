# Local exact-tree cross-command identity/config validation — `aefd49f`

Developer-run local validation of reachable exact implementation/test commit `aefd49fed7414f1cb927ba55dac235e99c1db24a`.

- start UTC: `2026-09-09T12:10:01Z`
- end UTC: `2026-09-09T12:11:54Z`
- host: `Linux 6.8.0-137-generic x86_64`
- Rust: `rustc 1.98.0 (88d9e12ae 2026-08-18)`
- `PYTHONDONTWRITEBYTECODE=1 bash scripts/check.sh`: exit `0`
- `git diff --check`: exit `0`
- native x86_64 package build and `scripts/release/smoke-package.sh`: exit `0`
- archive SHA-256: `e6eb387fbc9bf0ef843469ef32eecb84aba3e061fee7540463fbf2d940c5d546`
- initial/final source tree: clean

The focused table-driven process regression covers ordinary server/client, failover server/client, and endpoint-rebind server/client. Malformed hex, valid-hex 31/33-byte peer keys, and malformed local address/bind syntax all fail with a previously absent identity path still absent. Existing restrictive-umask exact-`0600`, descriptor-bound reload, symlink/non-regular/insecure-mode rejection, and valid runtime tests remained green.

This is developer-local CI and local package evidence only. It is not reviewer-executed CI, VPS/WAN evidence, signing/key custody, independent security review, RC, release, or production authorization.
