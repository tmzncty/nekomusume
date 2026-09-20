# Reviewer finding H-I4-085 — dependency/build review misclassifies production `snow`

**Severity:** HIGH — release-item-4 evidence / dependency-surface truth. This is **not** a claim that `snow` is unsafe or that the current crypto implementation is incorrect.

**Reviewed head:** `11678b2ff59bedfeac16f51e0b68518595fbe5a9`.
**Relevant executable source/test anchor:** exact `8cbd9af39a600f4ddd5fbc50208791e8584c6d74`; subsequent reachable commits through the reviewed head are documentation/review/navigation only.

## Concrete contradiction

The developer I4-BLD note at `e11c1d282625c1e72d6c8e3ce74500f1ba4d5307` says, under feature/default-feature reachability, that:

- `snow` is a **dev-dependency of `neko-crypto` only**; and
- **no production crate depends on `snow`**.

Those statements are false on both exact `8cbd9af` and current `main`.
`crates/neko-crypto/Cargo.toml` contains:

```toml
[dependencies]
snow = { version = "0.10.0", default-features = true }
```

and the committed lockfile records `neko-crypto -> snow` in the normal resolved package graph. The same lockfile also records `snow`'s resolved transitive packages. Therefore the I4-BLD no-finding note cannot currently be accepted as truthful item-4 dependency/build evidence.

This finding is deliberately narrower than the review note's other claims. In particular, lockfile presence alone is not enough to decide which optional/default features, build scripts, or native hooks are active in the exact normal build. Those facts must be checked from Cargo metadata/tree output rather than guessed.

## Closure contract

1. Re-read the exact-current workspace and all crate manifests plus `Cargo.lock`.
2. On the exact current source/test tree, inspect the actual dependency classes with locked Cargo tooling, at minimum:
   - `cargo metadata --locked`;
   - `cargo tree --locked -e normal`;
   - `cargo tree --locked -e build`;
   - `cargo tree --locked -e dev` (or an equivalent feature/classification view).
3. Correct/supersede the I4-BLD review note so it accurately distinguishes:
   - direct normal/runtime dependencies;
   - dev-only dependencies;
   - transitive normal dependencies;
   - active build/native hooks and feature/default-feature reachability.
4. Specifically determine the current `snow` / default-feature / transitive build surface from the tooling output. Do **not** change dependencies or features merely to make the old prose true. If the corrected graph exposes a concrete correctness/security/build defect whose answer is already fixed by current policy, make the smallest repair and regression; otherwise documentation/evidence repair is sufficient.
5. Preserve the existing governance boundary: dependency license/signing/SBOM/publication policy, new crypto/library selection, and any policy/value choice remain separate maintainer gates.
6. For the final pushed closure SHA, run and persist the normal clean exact-tree gate required by the repository contract: `PYTHONDONTWRITEBYTECODE=1 bash scripts/check.sh`, `git diff --check`, clean tree, UTC start/end, OS/arch, Rust stable version, and exact reachable SHA. Classify it as developer-local provenance only.

No decoder/parser/framing code is implicated by this finding, so fuzz is not mechanically required. `READY_LIVE: none`; no WAN run is relevant.

## Queue effect

Reopen only **I4-BLD**. The already completed R9/H-R9 findings and the other refill lanes are not reopened by this contradiction. While H-I4-085 is open, it stays at the front; after closure continue the existing independent item-4 review queue without waiting for reviewer cadence.
