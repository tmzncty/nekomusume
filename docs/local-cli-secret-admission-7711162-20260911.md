# Local CLI secret/admission surface review — `7711162`

Historical developer review using pre-rebase labels `7711162` / `48bca29`; those objects are not repository-reachable, so this file is quarantined / not accepted as exact-tree evidence. Reachable `96af2be` introduced the report; current accepted navigation uses reachable `4034f86`.

## Review result

The nine current pre-auth responder surfaces across seven admission sites remain correctly ordered and accounted for. No private key, plaintext payload, identity file contents, Noise transcript bytes, raw secret material or arbitrary peer text is emitted through diagnostics. Output keys are public/peer keys only, and no private material is printed.

No unauthenticated input can mutate Session delivery/path/ACK state, create persistent identity, bypass exact 32-byte peer-key validation, or produce READY/success evidence before required configuration, trust, negotiation, Noise, transcript binding, resume readiness and data-admission gates complete. Deterministic invalid configuration, malformed pre-auth input and incomplete trust/config remain before secret creation and network readiness.

Focused positives confirm ordinary authenticated TCP/UDP probes reach READY only after prerequisites; malformed UDP churn from eight asserted-distinct local source ports is rejected and a later authenticated exchange completes. Invalid command/config process regressions preserve absent identity paths. Clippy and responder-inventory checks pass.

Boundaries remain bounded research implementation only: no adversarial-load capacity suitability, public listener, production daemon, D019 source-retention answer, independent security audit, release or RC claim.

## Verification

- authenticated TCP/UDP READY positive: passed
- deterministic invalid configuration / absent identity matrix: passed
- eight distinct-source malformed UDP recovery process observation: passed
- all six `multistream` tests: passed
- `cargo clippy -p neko-cli --all-targets -- -D warnings`: passed
- `python3 scripts/check-preauth-responder-inventory.py`: passed (`9 surfaces`, `7 admission sites`)
- `PYTHONDONTWRITEBYTECODE=1 bash scripts/check.sh`: exit `0`
- `git diff --check`: exit `0`
- gate: `2026-09-10T20:34:41Z` -> `2026-09-10T20:36:36Z`
- host: `Linux 6.8.0-137-generic x86_64`
- Rust: `rustc 1.98.0 (88d9e12ae 2026-08-18)`
- initial/final source tree: clean
