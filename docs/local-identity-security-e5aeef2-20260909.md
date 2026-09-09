# Local exact-tree identity path validation — `e5aeef2`

Developer-run local validation of reachable exact implementation/test commit `e5aeef23f21c3577bb5b273311382d704baf96c6`.

- start UTC: `2026-09-09T08:12:33Z`
- end UTC: `2026-09-09T08:14:22Z`
- host: `Linux 6.8.0-137-generic x86_64`
- Rust: `rustc 1.98.0 (88d9e12ae 2026-08-18)`
- initial source tree: clean
- `PYTHONDONTWRITEBYTECODE=1 bash scripts/check.sh`: exit `0`
- `git diff --check`: exit `0`
- final source tree: clean
- focused identity boundary: existing identity metadata and bytes are read from the same descriptor; Unix opens use `O_NOFOLLOW|O_CLOEXEC`; the file must be regular and owner-only; group/world-writable identity parents are rejected before creation; secure reload, unchanged `0644` rejection, symlink rejection without target mutation, and server no-READY rejection remain covered.

No identity or key material is retained. This is developer-local CI evidence only, not independent security review, signing/key custody, RC, release, or production authorization.
