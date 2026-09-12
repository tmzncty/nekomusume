# Local dependency/safety factual review — `8d7c147`

Historical developer factual review for pre-rebase local label `8d7c147`. That object is not repository-reachable, so its exact-tree gate claim is **quarantined / not accepted as exact-tree evidence**. Reachable commit `2e92b0c` introduced this report. The dependency inventory remains a bounded observation, not accepted CI provenance.

## Findings

- Workspace lint configuration is `workspace.lints.rust.unsafe_code = "forbid"`, and all current workspace members inherit it. The repository-level check is an implementation/source policy fact, not a compiler or dependency proof.
- Current direct crypto/security-sensitive crate dependencies remain:
  - `neko-crypto`: `snow 0.10.0` (Noise IK transport and AEAD), `getrandom 0.2.17`, `libc 0.2.189` for constrained platform hooks;
  - `neko-cli`: `sha2 0.10.9`, `getrandom 0.2.17`, `libc 0.2.189`, `signal-hook 0.3.18`, plus workspace `neko-*` crates;
  - `neko-bench`: `serde`/`serde_json` and `zmij`, `libc`, `signal-hook` for diagnostics/observation;
  - other crates carry no additional direct crypto/security dependency.
- `Cargo.lock` is committed and `cargo tree --locked` confirms the same direct lockfile versions used by focused tests. `cargo metadata --locked` resolves the workspace without fetching policy changes.
- No signing, key-custody, SBOM, publication-trust, dependency-upgrade, toolchain-policy or service decision is made by this review. Dependency maintenance/suitability and cryptographic review remain separate release gates.

The check did not find a current executable dependency graph contradiction or unsafe-code policy drift requiring a code change or scanning framework.

## Verification

- direct `Cargo.toml` and `Cargo.lock` inspection: consistent
- `cargo metadata --locked --format-version 1 --no-deps`: resolved workspace packages
- `cargo tree --locked -p neko-crypto --depth 1`: `snow`, `getrandom`, `libc`
- `cargo tree --locked -p neko-cli --depth 1`: `sha2`, `getrandom`, `libc`, `signal-hook`
- `PYTHONDONTWRITEBYTECODE=1 bash scripts/check.sh`: exit `0`
- `git diff --check`: exit `0`
- gate: `2026-09-12T11:13:46Z` -> `2026-09-12T11:15:42Z`
- host: `Linux 6.8.0-137-generic x86_64`
- Rust: `rustc 1.98.0 (88d9e12ae 2026-08-18)`
- initial/final source tree: clean

This is bounded dependency/source-policy evidence only, not dependency security approval, independent cryptographic review, signing/key custody, SBOM, RC, release, public-listener or production authorization.
