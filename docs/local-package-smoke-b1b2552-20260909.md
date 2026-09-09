# Local exact-tree CLI and native package validation — `b1b2552`

Developer-run local validation of reachable exact implementation/test commit `b1b2552181a995d0d9b73def7bcde6d71418a344`.

- start UTC: `2026-09-09T06:16:52Z`
- end UTC: `2026-09-09T06:18:44Z`
- host: `Linux 6.8.0-137-generic x86_64`
- Rust: `rustc 1.98.0 (88d9e12ae 2026-08-18)`
- initial source tree: clean
- `PYTHONDONTWRITEBYTECODE=1 bash scripts/check.sh`: exit `0`
- `git diff --check`: exit `0`
- source tree after stable gate: clean
- `scripts/release/reproducibility-test.sh`: exit `0`; both same-tree archives matched
- external-temporary-output `scripts/release/build-package.sh`: exit `0`
- `scripts/release/smoke-package.sh` on the produced native archive: exit `0`; packaged `capabilities --json` executed successfully
- archive: `nekomusume-0.1.0-x86_64-unknown-linux-gnu.tar.gz`
- archive SHA-256: `01319fc37dcfdc08a9fb8da92f882f42cdf7803213795340adf762fdd8ffb9b4`
- packaged binary SHA-256: `f897d39967a33898733faedf4a9e6dda1eea1ed49c1e1c809320271e373490c6`
- final source tree: clean

The first local exact-`342e65c` attempt exposed a fixed-port collision in `response_send_restores_socket_write_timeouts`; exact `b1b2552` replaces that test listener with an OS-assigned loopback port before this successful full-chain run. This is developer-local CI and release-tool evidence only, not VPS/WAN evidence, signing/publication trust, independent security review, RC, release, or production authorization.
