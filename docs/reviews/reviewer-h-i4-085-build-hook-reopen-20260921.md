# Reviewer continuation H-I4-085 — active transitive build/native surface is still misclassified

**Severity:** HIGH — release-item-4 evidence / dependency-build-surface truth. This is **not** a finding that `snow` or `ring` is unsafe, nor a request to replace the selected crypto library.

**Reviewed head:** `9adf1d2181d1765325436ae1336614eee00c7a87`.
**Relevant executable source/test anchor:** exact `8cbd9af39a600f4ddd5fbc50208791e8584c6d74`; later commits through the reviewed head are review/evidence/navigation documentation only.

## New commits reviewed

- `c4925b67eb3f8425c77e53076248ab8e569f2d6e` — evidence correction: correctly changes `snow` from dev-only to a normal production dependency of `neko-crypto`.
- `803bee32c40a676612e89c9616797c600453eb19` — evidence/tooling note: records locked Cargo tree commands and the resolved normal `snow` path.
- `9adf1d2181d1765325436ae1336614eee00c7a87` — independent I4-FS1 wording correction (`drained/inert`, not a nonexistent scheduler close API). Useful later-lane support; unrelated to this HIGH.

The direct `snow` classification error is repaired, but H-I4-085 is **not closed** because the corrected I4-BLD note still makes a contradictory build/native-hook claim.

## Concrete remaining contradiction

Current `crates/neko-crypto/Cargo.toml` has one direct normal dependency:

```toml
[dependencies]
snow = { version = "0.10.0", default-features = true }
```

The corrected developer note then states both:

- `cargo tree --locked -e build -p neko-crypto` has "no build dependencies"; and
- "no build/native hooks exist" / the build/native surface is absent apart from a short proc-macro/version-check description.

That conclusion is false for the active resolved production graph.

The exact committed lockfile resolves:

```text
neko-crypto -> snow 0.10.0
snow -> ... ring 0.17.14 ... rustc_version 0.4.1 ...
ring -> ... cc 1.4.4 ...
```

The resolved upstream manifests/source establish the edge classes and hooks that the filtered command missed:

1. `snow 0.10.0` has `[build-dependencies.rustc_version] version = "0.4"` and ships `build.rs`; its build script calls `rustc_version::version_meta()`.
2. `snow 0.10.0` default features include `std`, and `std` includes `ring/std`; current normal-tree evidence already shows `ring` active under `snow`.
3. resolved `ring 0.17.14` declares `build = "build.rs"`, `links = "ring_core_0_17_14_"`, and a build dependency on `cc`; its package includes C/assembly/native build inputs.

Public upstream exact-version references for reproducibility:

- <https://docs.rs/crate/snow/0.10.0/source/Cargo.toml>
- <https://docs.rs/crate/snow/0.10.0/source/build.rs>
- <https://docs.rs/crate/ring/0.17.14/source/Cargo.toml>

Therefore an edge-filtered `cargo tree -e build -p neko-crypto` result cannot by itself justify "no build/native hooks" for the full normal production closure. The evidence must traverse the active normal path and then classify build edges/hooks of those resolved packages.

This remains an **evidence-truth** finding. The presence of a build script, `links`, C/assembly inputs, or `cc` does not itself establish a correctness/security defect.

## Required closure

1. Re-run exact-current locked dependency inspection with a view that preserves the normal path into transitive packages while exposing their build edges, for example:
   - `cargo metadata --locked --format-version 1` (without `--no-deps`, or an equivalent full resolved-node view);
   - `cargo tree --locked -p neko-crypto -e normal,build`;
   - `cargo tree --locked -p neko-crypto -e features,normal,build` (or equivalent feature-resolved output);
   - a dev-edge view sufficient to keep dev-only dependencies separate.
2. Correct/supersede `docs/reviews/dev-i4-bld-dependency-build-surface-20260921.md` so it truthfully distinguishes:
   - direct normal dependencies;
   - transitive normal dependencies;
   - dev-only dependencies;
   - active default/features;
   - build-script/build-dependency edges;
   - native/`links` surface.
3. Explicitly record the current exact facts for `snow` and the active `ring` path: `snow` normal/default-feature use, `snow` build script + `rustc_version`, and `ring` build script + `links` + `cc`/native build surface.
4. Keep workspace-member facts separate from transitive-dependency facts. "No workspace member has a local `build.rs`" may be true; "no build/native hooks exist in the active production closure" is not.
5. Do **not** change dependencies, default features, crypto selection, or build policy merely to make the earlier prose true. If the corrected inspection exposes a separate concrete correctness/security/build defect whose answer is already fixed by current policy, repair only that defect with focused regression; otherwise this HIGH closes by truthful evidence repair.
6. Final pushed closure SHA gets the normal developer-local clean exact-tree gate/provenance required by the repository contract: `PYTHONDONTWRITEBYTECODE=1 bash scripts/check.sh`, `git diff --check`, clean tree, exact reachable SHA, UTC start/end, OS/arch, Rust stable version. Keep that classification distinct from reviewer-local or hosted CI.

No decoder/parser/framing change is implicated, so fuzz is not mechanically required. `READY_LIVE: none`; no WAN run is relevant.

## Queue effect

Keep H-I4-085 at the front until the build/native-surface contradiction and exact-tree provenance are closed. Preserve the already prepared I4-FS1/FS2/AD1/AD2/PORT notes as later-lane inputs; do not discard them, but do not let them make the HIGH disappear. After closure, immediately continue the existing dependency-ready item-4 queue without waiting for reviewer cadence.
