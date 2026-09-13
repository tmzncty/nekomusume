# Independent bounded package/reproducibility/operator review — exact `8e11de0`

Bounded independent static review of `scripts/release/build-package.sh`, `scripts/release/check-clean-source.sh`, `scripts/release/smoke-package.sh`, and `scripts/release/reproducibility-test.sh`, at reachable exact `8e11de0e5915efa43591069172b97f311aae2c6e`. This is a deterministic implementation review of the packaging scripts — **not** a repeat VPS rehearsal, not a live install/upgrade/rollback run, not signing/key-custody/SBOM policy, not an independent security/release approval.

## Challenged invariants and result

- **Clean-source enforcement timing — holds.** `check-clean-source.sh` runs before any build metadata query or output creation (`build-package.sh:6`), and it fails closed on unstaged tracked changes, staged changes, or non-ignored untracked files. Source identity (git HEAD) is fixed before the build.
- **Archive path/type/root/mode/checksum validated before extraction/use — holds.** `smoke-package.sh` lists members, enforces a strict `nekomusume-*-{x86_64,aarch64}-unknown-linux-gnu` root whitelist, rejects absolute paths, `..` segments and over-deep members, diffs the full member set against the expected layout, and validates each member's type/mode via `tarfile` **before** `tar -xzf`. Extraction uses `--no-same-owner --no-same-permissions` so hostile modes cannot be re-applied.
- **Reproducibility inputs and source/binary identity — holds.** `build-package.sh` exports `SOURCE_DATE_EPOCH` (integer-validated), `CARGO_INCREMENTAL=0`, `--locked`, `--target`, `tar --sort=name --owner=0 --group=0 --numeric-owner --mtime`, and `gzip -n`; the build evidence JSON records `git_commit`, `version`, `target`, `archive_sha256`, `binary_sha256`.
- **Dedicated install-path containment — holds.** Staging is a `mktemp -d` with a `trap` cleanup; the archive root is the single `nekomusume-VERSION-TARGET` directory.
- **External identity/state retention boundaries — holds.** Output goes only to `$OUT` (default `dist/`); the atomic `mv -f` from `*.tmp` avoids partial archives.
- **Rollback selects intended binary / no untested schema-compat claim — n/a for these scripts.** Rollback lives in the operator helpers, not in `build-package.sh`/`smoke-package.sh`; no schema-compatibility claim is asserted here.
- **Cleanup/listener/process assertions — holds.** These scripts make no "service started/listening" claims before observation; `smoke-package.sh` only runs `neko-cli capabilities --json` on the matching host target and asserts `secret_free:true`.
- **Deterministic failure surface — holds.** `set -eu` throughout; unsupported target/`SOURCE_DATE_EPOCH`/malformed manifest each fail closed with a distinct message rather than silently proceeding.

## Minor observation (not a defect)

`build-package.sh` derives `VERSION` via `cargo metadata | sed`; if the `neko-cli` manifest were ever renamed, `VERSION` could come out empty and produce a `nekomusume--TARGET` archive name. `set -eu` does not catch an empty sed substitution. This is a cosmetic robustness note, not a contract defect — the produced archive would still be internally consistent and `smoke-package.sh` would still validate it. Recorded for awareness only.

## Evidence

- Static review of all four scripts on exact `8e11de0`; `scripts/check-release-boundaries.sh`, `check-clean-source-test.sh`, and `smoke-package-test.sh` are the owning regression harnesses and pass in the shared `scripts/check.sh` gate.
- No code change; no defect found. No new `READY_LIVE` question; release/governance state unchanged.
