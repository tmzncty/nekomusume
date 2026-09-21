# Repository-wide 13-surface refill check — post-H-I4-089/090

**Anchor:** exact source/test tree `0dbb931` + all reviewer/dev commits through `b6cffc4`.

## 13-surface inventory — refill status

| # | Surface | Status | Evidence |
|---|---|---|---|
| 1 | `neko-reliable` Recovery | covered | `independent-i4-bnd-recovery-current-89d249b` |
| 2 | `CarrierState` | covered | R9-11C/final R9 |
| 3 | Carrier Manager / health / migration-back | covered | R9-11C/final R9 |
| 4 | FairScheduler / multi-stream / flow-control | covered | `dev-i4-fs1` + `dev-i4-fs2` + H-I4-086/087 |
| 5 | Carrier adapters (Memory/UDP/TCP) | covered | `dev-i4-ad1`/`ad2a`/`ad2b` + H-I4-088 |
| 6 | `SessionRuntime` lifecycle/resource/window/DeliveryAck | covered | H-I4-086/087 + `dev-i4-bnd` |
| 7 | `neko-observe` projection/counters | covered | unchanged owner since `8e11de0` |
| 8 | Package/reproducibility/operator scripts | covered | unchanged owner since 2026-09-13 |
| 9 | Dependency/build surface | covered | H-I4-085 |
| 10 | Cross-platform CLI/process-test | covered | `26a2c40` + `0dbb931` + `c3c1664`/`1c8eacc`/`5e9b4b2` |
| 11 | CLI exit/JSON/human contract | covered | `independent-i4-cli-machine` + `dev-i4-cli-h` + `dev-i4-cli-h-independent` + `dev-i4-cli-h-recheck` |
| 12 | Algorithmic resource boundedness | covered | `dev-i4-bnd` + `independent-i4-bnd-recovery` + `dev-i4-bnd-synthesis` |
| 13 | Release-packet factual consistency | covered | `dd166b3` + `7197abf` + `588ca37` + `b6cffc4` |

## Material owner changes since last inventory

- `crates/neko-cli/tests/probe.rs` — `26a2c40` (H-I4-089 `#[cfg(target_os = "linux")]` /proc scoping) + `0dbb931` (H-I4-090 pre-churn baseline ordering). Both are test-only platform/ordering repairs; no runtime/product owner changed.

## Refill conclusion

No surface requires refill: every implemented core owner has either a dedicated
current bounded independent challenge or an unchanged owner with valid prior
coverage. The `probe.rs` cfg-gate repairs are scoped test-evidence truth, not
a material runtime owner change.

**Queue is genuinely empty of dependency-ready work.** `READY_LIVE: none`;
release items 3/4 remain incomplete; D019/policy gates untouched.
