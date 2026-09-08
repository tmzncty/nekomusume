# Current-tree package/operator closure — exact `f19ad28`

## Scope and boundary

This is an isolated local package/operator rehearsal for exact `f19ad280b7363b6f4a86c7c34dc50e2b12b92ea3`. It revalidates the materially changed current executable against the existing package contract. It is not VPS/WAN evidence, a distinct-version upgrade compatibility result, RC, production readiness, security approval, or release authorization.

The historical N5 exact A→B→A distinct-version result remains authoritative for version-switch rollback and external-state retention. This run does not manufacture a second package version from one tree; it only exercises an isolated immutable-directory symlink switch and rollback using two identical current-tree installs.

## Build provenance and reproducibility

- Target: `x86_64-unknown-linux-gnu`
- Rust: `rustc 1.98.0 (88d9e12ae 2026-08-18)`
- Cargo: `cargo 1.98.0 (797e8a9bc 2026-08-05)`
- `SOURCE_DATE_EPOCH`: `1788893163`
- Archive: `nekomusume-0.1.0-x86_64-unknown-linux-gnu.tar.gz`
- Two independent invocations of `scripts/release/build-package.sh` produced the same archive SHA-256: `47576920cfce7dffc20f02ce6b3e3b3a00367faae35d72127cf55fdcbaf4579a`.
- Both emitted identical build evidence; build-evidence SHA-256: `953cf79de2f1f018f25071f261e2db9fe3ba47c8023cd552663c76ef3b5b604a`.
- Packaged `neko-cli` SHA-256: `6cf74e8f1f027c10b843fc98ff0f31e12b7f13bb3495eb5b5dbc1bbb0320d7e2`.

`smoke-package.sh` passed archive path/traversal checks, embedded `SHA256SUMS`, executable/document modes, native execution, capability schema, and `secret_free=true`.

## Isolated operator observations

All work used temporary directories and loopback port 40100. Fresh temporary identities were stored outside both package directories.

- Authenticated TCP package smoke: two 32-byte exchanges completed; client reported `probe_ok transport=tcp bytes=32`.
- Authenticated UDP package smoke: two 32-byte exchanges completed; client reported `probe_ok transport=udp bytes=32`.
- Both listeners were absent after their bounded server exited.
- A temporary external-state marker retained mode `0600` across an immutable-directory symlink switch and rollback.
- Final symlink pointed back to the first current-tree install.
- Temporary installation, state, identities, logs, and listeners were removed after observation.

## Remaining gates

The current-tree package behavior above is bounded local evidence only. Native aarch64 execution, signed checksums/SBOM/provenance publication, distinct-version state-format compatibility, service-manager sandboxing, independent release/security review, WAN/reachability, and production authorization remain open. `RELEASE_CANDIDATE=false`, `PRODUCTION_READY=false`, `FREEZE=false`, and `RELEASED=false` are unchanged.
