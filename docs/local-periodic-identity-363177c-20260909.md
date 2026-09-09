# Local exact-tree periodic identity/config validation — `363177c`

Developer-run local validation of reachable exact implementation/test commit `363177c18773d52cfa9edc114ed380117e03b562`.

- focused deterministic rejection test: `cargo test -p neko-cli --test probe deterministic_invalid_configuration_does_not_create_identity -- --exact`; exit `0`
- focused authenticated periodic positive: `cargo test -p neko-cli --test probe periodic_session_delayed_confirmations_are_counted_on_one_session -- --exact`; exit `0`
- exact-tree `PYTHONDONTWRITEBYTECODE=1 bash scripts/check.sh`: exit `101` because one existing failover process test (`first_udp_selection_loss_recovers_from_same_peer_duplicate_hello`) hit the known fixed-port/process-start race; all preceding workspace, multistream, and other probe tests passed, and no source-tree residue was left
- `git diff --check`: exit `0`
- host: `Linux 6.8.0-137-generic x86_64`
- Rust: `rustc 1.98.0 (88d9e12ae 2026-08-18)`
- exact-tree gate: `2026-09-09T15:11:39Z` -> `2026-09-09T15:12:35Z`
- initial/final source tree: clean

The implementation moves periodic-server peer-key and bind/port validation before signal setup and identity creation, and routes periodic-client peer-key parsing through the shared exact-32-byte helper before identity creation. The focused positive and rejection tests passed. The full gate result is retained honestly as blocked by the unrelated existing fixed-port failover race; it is not presented as a green release gate.
