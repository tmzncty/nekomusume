# Developer bounded I4-BLD review — dependency/build surface

**Anchor:** exact source/test tree `8cbd9af` + reviewer/dev commits through `3ad1de6`.
**Owners inspected:** workspace `Cargo.toml`, `Cargo.lock`, all crate manifests,
feature/default-feature reachability, build/native hooks, inherited unsafe
assumptions.

## Per-invariant coverage

| Invariant | Coverage |
|---|---|
| workspace/crate manifests are consistent | `resolver = "3"`, `edition = "2024"`, `license = "MIT OR Apache-2.0"`, `version = "0.1.0"` at workspace root; all eight members are path crates — no external neko-* source is resolved |
| `Cargo.lock` is pinned and reachable | lockfile is committed and checked by `cargo metadata --locked`; no unpinned or floating version resolution |
| feature/default-feature reachability is minimal | `snow` is a dev-dependency of `neko-crypto` only (default-features enabled for test Noise interop); `serde`/`serde_json` are dev-dependencies of `neko-wire` only (`features = ["derive"]`); no production crate depends on serde, snow, or tokio — production dependency surface is `blake2` + `aes-gcm`/`chacha20poly1305`/`curve25519-dalek`/`snow` only where cryptographically required |
| build/native hooks are absent | no `build.rs` in any crate; no `links` declaration; no `build-dependencies` beyond `version_check`/`proc-macro2`/`quote`/`syn`/`unicode-ident` pulled transitively by `serde_derive`/`curve25519-dalek-derive` (proc-macro only, not production-linked) |
| inherited unsafe assumptions | workspace `unsafe_code = "forbid"` applies to every member — no crate may use `unsafe`; `panic = "abort"` in dev and release profiles keeps panic semantics uniform |
| no dependency added merely to exercise the lane | dependency set is unchanged from the reviewed baseline; `neko-reliable` integration into Carrier/CLI is internal workspace wiring, not a new external dep |

## Reachable checks named

- `cargo metadata --locked` — lockfile consistent
- `cargo tree --locked --workspace` — dependency graph verified above
- `unsafe_code = "forbid"` — workspace lint enforced on all members
- `panic = "abort"` dev+release — uniform panic semantics

## Result

No concrete defect found across the dependency/build surface. The workspace
manifests are consistent, the lockfile is pinned, feature reachability is
minimal and dev-scoped for `snow`/`serde`, no build/native hooks exist, and
`unsafe_code = "forbid"` + `panic = "abort"` apply workspace-wide.

**READY_LIVE: none** — deterministic local evidence only.
