# Repository-wide reuse boundaries + 13-surface refill — post-I4-CLI-PROC-096/BND

**Anchor:** exact source/test tree `5b7b22b` + reviewer/dev commits through `67541a8`.

## Material owner changes since prior dedicated reviews

- `crates/neko-cli/tests/probe.rs` — H-I4-089..095 causal oracles, I4-CLI-PROC-096 bounded readiness, I4-BND one-item channel bound. All test-only; each closed on its own exact-tree provenance.
- `crates/neko-cli/src/main.rs` — H-I4-091 `malformed_or_unadmitted` emit ordering only (post-`preauth.release`). No decoder/parser/crypto/wire owner changed.
- `docs/preauth-responder-inventory.v1.json` — anchor realignment to post-release emit order.

## 13-surface refill inventory

| # | Surface | Status | Evidence |
|---|---|---|---|
| 1 | `neko-reliable` Recovery | covered | `independent-i4-bnd-recovery-current-89d249b` — owner unchanged |
| 2 | `CarrierState` | covered | R9-11C/final R9 — owner unchanged |
| 3 | Carrier Manager / health / migration-back | covered | R9-11C/final R9 — owner unchanged |
| 4 | FairScheduler / flow accounting | covered | `dev-i4-fs1` + `dev-i4-fs2` + H-I4-086/087 — owner unchanged |
| 5 | Carrier adapters (Memory/UDP/TCP) | covered | `dev-i4-ad1`/`ad2a`/`ad2b` + H-I4-088 — owner unchanged |
| 6 | `SessionRuntime` lifecycle/resource/DeliveryAck | covered | H-I4-086/087 + `dev-i4-bnd` — owner unchanged; `events` policy-gated |
| 7 | `neko-observe` projection/counters | covered | unchanged owner since `8e11de0` |
| 8 | Package/reproducibility/operator | covered | `independent-package-build-reuse-b4af007` — unchanged owners under committed `scripts/check.sh` gate |
| 9 | Dependency/build surface | covered | H-I4-085 — unchanged owners |
| 10 | Cross-platform CLI/process-test | covered | `independent-cross-platform-cli-process-b4af007` — all readiness/process helpers bounded, target-gated, ownership convergent |
| 11 | CLI machine/human + diagnostic contract | covered | `independent-cli-diagnostic-boundary-b4af007` + prior `independent-i4-cli-machine`/`dev-i4-cli-h` |
| 12 | Algorithmic resource boundedness | covered | `independent-i4-bnd-reconciliation-5b7b22b` — one concrete bound repaired (`sync_channel(64)→(1)`), no remaining defect |
| 13 | Release-packet factual consistency | covered | `67541a8` — new group row added, evidence classes distinct, no release promotion |

## Refill conclusion

Every implemented core surface is either currently independently challenged or
unchanged under still-valid dedicated coverage. The only concrete defect found
in this sweep (`sync_channel(64)` unexplained buffer) was repaired at
`5b7b22b`. Queue exhaustion is legal at this anchor: no uncovered implemented
core surface, no READY review-support, no READY live question.

**READY_LIVE: none**; release items 3/4 remain incomplete; D019/policy gates
untouched.
