# R-PKG/BLD-DIFF — package / reproducibility / dependency-build owner-diff review

**Exact source/control-flow anchor:** `5135d8e835ea84d02f0c254111a03482fa57a75d`

**Review class:** bounded independent GitHub source/owner-diff review. This note records reviewer inspection only; it is not reviewer-local Rust execution, developer-local exact-tree provenance for this anchor, hosted CI, WAN evidence, fuzz evidence, or a performance conclusion.

## Owners inspected

- `Cargo.toml`, `Cargo.lock`, and every current `crates/*/Cargo.toml`;
- `scripts/release/build-package.sh`;
- `scripts/release/check-clean-source.sh` and `scripts/release/check-clean-source-test.sh`;
- `scripts/release/reproducibility-test.sh`;
- `scripts/release/smoke-package.sh` and `scripts/release/smoke-package-test.sh`;
- `scripts/check.sh`;
- repository code-search for Rust `build.rs` / repository-owned build hooks;
- current package/reproducibility and dependency/build evidence boundaries in `docs/release-security-review-packet.md` and `docs/CHATGPT_HANDOFF.md`.

Reuse baselines challenged against current HEAD:

- package/reproducibility review commit `71de29cfcbc0a0ddd4db26cc4cec6be2500e72d8`;
- dependency/build review anchor `e11c1d282625c1e72d6c8e3ce74500f1ba4d5307`.

## Invariants challenged

1. **Manifest / feature / lock drift must not silently invalidate the previous dependency/build review.**
2. **Workspace unsafe policy must still be inherited by current crates rather than locally bypassed.**
3. **No new repository-owned native/build hook may have entered the build surface without dedicated review.**
4. **Release package construction must still bind a clean source identity before metadata/build/output mutation and must remain `--locked`.**
5. **Archive layout/type/mode/checksum validation must still occur before package use/extraction, with adversarial regressions retained in the ordinary gate.**
6. **Reproducibility evidence must remain deterministic-source/build metadata evidence, not signing/SBOM/key-custody/publication evidence.**
7. **Recent release-packet wording must not promote package/operator evidence into RC, security approval, production readiness, or a claim that one SHA ran all historical gates.**

## Owner-diff result

**No finding in this bounded lane.**

- `71de29cf..5135d8e` changes only carrier/recovery/CLI-test owners plus review/provenance documentation and a conservative release-packet index addition. It does **not** change the root manifest, lockfile, crate manifests, `scripts/release/*`, or `scripts/check.sh`.
- `e11c1d28..5135d8e` likewise has no `Cargo.toml`, `Cargo.lock`, crate-manifest, release-package-script, or build-hook change. Current crate manifests continue to opt into workspace lints; root `unsafe_code = "forbid"` remains the inherited Rust unsafe policy.
- Repository code search found no `build.rs`, so there is no new repository-owned Rust build script/native hook to review in this lane.
- `build-package.sh` still runs the clean-source guard before Cargo metadata/output creation, restricts release targets to the existing Linux GNU targets, uses `cargo build --release --locked --target`, fixes archive ordering/ownership/mtime, uses `gzip -n`, and records source/archive/binary identity.
- `check-clean-source.sh` still rejects unstaged tracked, staged, and non-ignored untracked source state; its deterministic regression remains wired into `scripts/check.sh`.
- `smoke-package.sh` still validates the full member set, root, member type/mode, and checksum-manifest shape before extraction/use; `smoke-package-test.sh` retains traversal/link/special-member/layout/checksum/mode negative cases.
- `reproducibility-test.sh` still builds the same source twice outside the repository output tree with a fixed source epoch, compares archive SHA-256 values, and checks that the source tree stayed clean.
- The release packet remains an evidence index with RC/security/production/freeze/release boundaries explicit; the recent recovery/CarrierState review cluster does not alter package semantics.

## Evidence boundaries / exclusions

- The historical developer-local `scripts/check.sh` pass cited by the previous package review belongs to its own exact tested tree. This review did **not** rerun or reattribute it to `5135d8e`.
- No signing policy, key-custody policy, SBOM/publication policy, release target expansion, capacity/security number, or release authority decision is made here.
- No wire decoder/parser/crypto-framing owner changed in this lane; fuzz is not mechanically requested.
- No new real-network question arises from this owner-diff. **`READY_LIVE: none`.**

## Classification

`R-PKG/BLD-DIFF: CLOSED — bounded no-finding / prior evidence reusable for unchanged semantic owners.`

Continue immediately to the next dependency-ready review lane; this closure is not repository-wide queue exhaustion and release items 3/4 remain incomplete.
