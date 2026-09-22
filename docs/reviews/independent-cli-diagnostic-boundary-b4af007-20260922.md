# Developer bounded independent CLI diagnostic/machine-output boundary review

**Anchor:** exact source/test tree `b4af007` + commits through `75f4d9c`.
**Owners inspected:** `crates/neko-cli/src/main.rs` `emit_diagnostic`/`diagnostic_mode`/`diagnostic_id`, all `emit_diagnostic` call sites incl. `malformed_or_unadmitted`, `udp_hello_received`, `udp_selection_retried`, `udp_selection_dropped`, `udp_noise_response_dropped`, `udp_noise_response_retried`, `server_recv`; `crates/neko-cli/tests/probe.rs` diagnostic barrier consumers (`malformed_classification_barrier`, `\"event\":\"malformed_or_unadmitted\"` matching); `docs/carrier-architecture.md`, `docs/status.md` diagnostic prose.

## Per-invariant coverage

| Invariant | Coverage |
|---|---|
| `malformed_or_unadmitted` scope is server-side malformed/rejected classification only | Emitted only in the pre-auth `admit_carrier`/`charge_input`/`server_accept_hello` failure paths of `failover_server` and the malformed-UDP churn test path — never on authenticated/session/carrier success. |
| No JSON/human output collision | Diagnostic lines are emitted only under `--diagnostic` (`diagnostic_mode`), a distinct flag from `--json` and human modes; probe/reachability `{"reachable":true}` and human `pass:`/`fail:` lines are unaffected. The `{"experiment_id","role","event","seq",...}` schema is a separate diagnostic envelope. |
| Diagnostic parsing stability | Tests match the literal `\"event\":\"malformed_or_unadmitted\"` substring inside the emitted JSON line; the event name is a compile-time literal at every call site, not a constructed string. |
| Diagnostic-only — not authentication/Delivery/Path/ACK/wire/release evidence | `malformed_or_unadmitted` is a server-side diagnostic counter for the bounded churn barrier; it is never written into Session state, Carrier readiness, packet ACK, wire framing, or release artifacts. H-I4-091 keeps it post-`preauth.release` so it is a cleanup-complete observation, not a delivery signal. |
| `diagnostic_id` validation | `--experiment-id` is required with `--diagnostic`; length 8–72, alphanumeric + `.`/`_`/`-` only — bounded and validated before emission. |

## Commands/tests actually run

- `cargo test -p neko-cli --test probe` — malformed-churn + barrier tests consume the diagnostic stream; all pass.
- `cargo clippy -p neko-cli --all-targets -- -D warnings` — clean.

## Exclusions

- No fuzz — diagnostic emission is `println!`, not decoder/parser/crypto framing.
- No release promotion — diagnostic events remain internal test evidence.

## Result

No concrete defect found in the challenged CLI diagnostic/machine-output
boundary. `malformed_or_unadmitted` is scoped, literal, diagnostic-mode-only,
and not promoted into protocol/release semantics.

**READY_LIVE: none** — deterministic local evidence only.
