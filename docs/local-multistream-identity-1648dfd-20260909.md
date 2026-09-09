# Local exact-tree multistream identity validation — `1648dfd`

Developer-run local validation of reachable exact implementation/test commit `1648dfdbf6731f55bc31598a120bd42c1de1d8a7`.

- focused command: `cargo test -p neko-cli --test multistream`; exit `0` (6/6)
- secure existing owner-only identity reaches the existing authenticated multistream positive path
- Unix `0644` identity is rejected without byte or mode mutation
- final-component symlink identity is rejected without target mutation
- malformed address and wrong-length peer key fail before network I/O and leave an absent identity path absent
- multistream continues to require an existing identity; it does not auto-generate one
- exact-tree `PYTHONDONTWRITEBYTECODE=1 bash scripts/check.sh`: exit `0`
- `git diff --check`: exit `0`
- exact-tree gate: `2026-09-09T14:17:53Z` -> `2026-09-09T14:19:45Z`
- host: `Linux 6.8.0-137-generic x86_64`
- Rust: `rustc 1.98.0 (88d9e12ae 2026-08-18)`
- initial/final source tree: clean

The repair reuses the CLI's descriptor-bound existing-identity reader and changes no Session, Carrier, ACK, Noise, wire, or multistream delivery semantics. This is developer-local CI evidence only, not reviewer-executed CI, VPS/WAN evidence, independent security review, RC, release, or production authorization.
