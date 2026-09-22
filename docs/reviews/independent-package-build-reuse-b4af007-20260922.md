# Developer bounded package/build + reproducibility spot re-challenge

**Anchor:** exact source/test tree `5b7b22b` + commits through `75f4d9c`.
**Owners inspected:** `Cargo.toml`, `Cargo.lock`, `crates/*/Cargo.toml`, `scripts/release/`, `scripts/check-release-boundaries.sh`, `scripts/check-era4-protocol-release.sh`, `docs/release-security-review-packet.md` package/provenance claims.

## Owner-diff coverage

| Surface | Status |
|---|---|
| `Cargo.toml`/`Cargo.lock`/`crates/*/Cargo.toml` | unchanged since `5537e93` (`chore(deps): lock neko-reliable as neko-cli dependency for R8 lab`) — covered by prior dependency-surface review (H-I4-085) |
| `scripts/release/` + `check-release-boundaries.sh` + `check-era4-protocol-release.sh` | unchanged — covered by `independent-package-release-8e11de0` + `check-release-boundaries` tests still run by `scripts/check.sh` at `5b7b22b` exit 0 |
| Package/provenance claims in release packet | still accurate — all recent commits are `probe.rs`/`main.rs` test-or-source repairs and docs/review/handoff/provenance notes; no package/operator/build owner touched |
| Signing/SBOM/key-custody | unchanged; no policy invented |

## Verification actually run

- `PYTHONDONTWRITEBYTECODE=1 bash scripts/check.sh` at `5b7b22b` — exit 0 (includes `check-release-boundaries`, era4 protocol release, canonical-vectors validation).

## Exclusions

- No new signing/SBOM/key-custody policy; no package release claimed.
- No re-run of unchanged package lifecycle beyond the committed `scripts/check.sh` gate.

## Result

No package/build/reproducibility defect: all challenged owners are unchanged
under still-valid prior coverage, and the committed `scripts/check.sh` gate
at `5b7b22b` passes.

**READY_LIVE: none** — deterministic local evidence only.
