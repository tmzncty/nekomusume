# Local exact-tree native package smoke — `5e68ad6`

Developer-run local validation of the reachable exact CLI implementation commit `5e68ad6ee20654e5e3b1c8d62857b72d129d1870`.

- target: `x86_64-unknown-linux-gnu`
- command chain: `PYTHONDONTWRITEBYTECODE=1 bash scripts/check.sh`; external-temporary-output `scripts/release/build-package.sh`; `scripts/release/smoke-package.sh` on the produced native archive
- start/end date: `2026-09-09` UTC; each command exited `0`
- host: Linux x86_64
- Rust: `rustc 1.98.0 (88d9e12ae 2026-08-18)`
- archive: `nekomusume-0.1.0-x86_64-unknown-linux-gnu.tar.gz`
- archive SHA-256: `b55aff6f44dc512297108835f085c348a97c661c2dd465ed73ba60ff63eb83fe`
- packaged binary SHA-256: `913c59095a4b1ac8825417de622504854f5555076587da5324332a3b1dcfd073`
- native packaged `capabilities --json`: passed through `package_smoke_ok target=x86_64-unknown-linux-gnu`
- source checkout: clean before and after validation

This is developer local release-tool evidence. It is not VPS/WAN evidence, signing or publication trust, independent security review, RC, release, or production authorization.
