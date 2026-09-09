# Local exact-tree identity security validation — `749976f`

Developer-run local validation of reachable exact implementation/test commit `749976f89056fd98e2295941471b185cce8b524f`.

## Stable gate

- start UTC: `2026-09-09T07:11:06Z`
- end UTC: `2026-09-09T07:12:57Z`
- host: `Linux 6.8.0-137-generic x86_64`
- Rust: `rustc 1.98.0 (88d9e12ae 2026-08-18)`
- initial source tree: clean
- `PYTHONDONTWRITEBYTECODE=1 bash scripts/check.sh`: exit `0`
- `git diff --check`: exit `0`
- final source tree: clean
- focused regression: new/reloaded secure identity, unchanged permissive-file rejection, symlink rejection without target mutation, and invalid-identity server rejection before READY

## Native packaged-operator smoke

An actual x86_64 package built from the same exact tree was extracted outside the source checkout.

- UTC observation: `2026-09-09T07:13:45Z`
- archive SHA-256: `6492b6ea0a1d85541dfd3598d0bcd50710d750eec4b73d06c0edc595fb58615b`
- packaged binary SHA-256: `9bdd709a98711c865bce95c54ad3e83618ff744fe0f0ab2c0b4e2c1ff422a7de`
- first packaged `keygen --identity <external-temp-path>`: exit `0`, resulting Unix mode `0600`
- second invocation: exit `0`, same public-key output
- permissive existing identity (`0644`): rejected with exit `2`; bytes and mode remained unchanged
- temporary identities: outside the source tree and removed with the temporary worktree
- final source tree: clean

No private or public key material is retained here. This is developer-local CI and packaged-operator evidence only, not independent security review, signing/key custody, RC, release, or production authorization.
