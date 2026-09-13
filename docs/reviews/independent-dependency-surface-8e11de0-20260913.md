# Independent bounded dependency/build-surface review — exact `8e11de0`

Bounded independent review of the workspace `Cargo.toml`, all eight member manifests, `Cargo.lock`, the `unsafe_code = "forbid"` workspace lint, build scripts, and feature surface, at reachable exact `8e11de0e5915efa43591069172b97f311aae2c6e`. Not a CVE/audit or release decision — dependency-health/release-gating belongs to existing project governance; this only challenges the reachable manifest/lockfile/build surface for contradictions with committed policy.

## Challenged invariants and result

- **No unexpected/renamed/mismatched dependency or feature surface — holds.** Direct external deps are minimal and well-known: `snow 0.10` (crypto), `signal-hook 0.3`, `sha2 0.10`, `getrandom 0.2`, `libc 0.2` (CLI), and `serde`/`serde_json` only as `neko-wire` dev-deps. No renamed, typosquat-shaped, or stray dependency appears in any manifest.
- **No build-script/proc-macro exposure beyond declared — holds.** No `build.rs` exists in the workspace root or any member (`git ls-files | grep build.rs` → none). The only proc-macro crate reachable is `serde_derive`, pulled exclusively by `neko-wire` **dev-dependencies** (tests only — it does not enter the release `neko-cli` binary). `curve25519-dalek-derive`/`fiat-crypto` are internal transitive deps of `snow`, not authored surface.
- **Workspace member containment + unsafe forbid — holds.** All 8 members use `[lints] workspace = true`, so `unsafe_code = "forbid"` applies workspace-wide; `grep -rn 'unsafe ' crates/*/src` returns zero hits. `resolver = "3"`, `panic = "abort"` in both profiles.
- **Foreign/unsafe artifacts cannot silently change trust — holds.** `Cargo.lock` is tracked; all 76 resolved packages carry registry checksums and `source = "registry"` — no `git`/path/`vendor` source substitutes trust. All builds/tests use `--locked`.
- **No hidden second protocol or supply-chain surprise — holds.** `getrandom` appears as `0.2.17` (direct CLI dep) and `0.3.4` (transitive via `snow`) — a normal duplicate-version resolution, not a conflicting semantic. No dep enables a second wire protocol.

## Minor observation (not a defect)

`Cargo.lock` resolves two `getrandom` majors (0.2 + 0.3). Both are registry-pinned with checksums and are used for distinct consumers; this is expected multi-version resolution, not an unbounded or unsafe surface.

## Evidence

- `Cargo.lock` committed and registry-only; `cargo build --locked`/`scripts/check.sh` use `--locked`.
- No code change; no defect found. No new `READY_LIVE` question; release/governance state unchanged.
