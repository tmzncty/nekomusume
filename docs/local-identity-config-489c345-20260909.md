# Local exact-tree identity/config validation — `489c345`

Developer-run local validation of reachable exact implementation commit `489c345542414aebc726bea6dc5e7e714478061f`.

- start UTC: `2026-09-09T11:12:13Z`
- end UTC: `2026-09-09T11:13:10Z`
- host: `Linux 6.8.0-137-generic x86_64`
- Rust: `rustc 1.98.0 (88d9e12ae 2026-08-18)`
- exact command: `cargo test --workspace --locked -- --test-threads=1`; exit `0`
- exact commands: `cargo fmt --all -- --check`; `git diff --check`; exit `0`
- initial/final source tree: clean
- focused coverage: canonical 32-byte peer-key validation for ordinary server/client; restrictive-umask identity creation/reload; existing identity/symlink/non-regular rejection; deterministic invalid-config identity absence

The serialized test run avoids unrelated fixed-port interference from parallel process tests; it does not alter production behavior or claim network evidence.
