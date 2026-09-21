# Final item-4 technical sweep — repository-wide conclusion

**Anchor:** exact source/test tree `42be539` + all reviewer/dev commits through `f61b201`.

## 13-surface inventory — final status

| # | Surface | Status | Evidence |
|---|---|---|---|
| 1 | `neko-reliable` Recovery | covered | `independent-i4-bnd-recovery-current-89d249b` — bounded `sent`/`AckRanges`/RTT/PTO |
| 2 | `CarrierState` | covered | R9-11C/final R9 — generation/validation/hysteresis/single-active |
| 3 | Carrier Manager / health / migration-back | covered | R9-11C/final R9 — uncertain retention/replay/validation |
| 4 | FairScheduler / multi-stream / flow-control | covered | `dev-i4-fs1` + `dev-i4-fs2` + H-I4-086/087 repairs |
| 5 | Carrier adapters (Memory/UDP/TCP) | covered | `dev-i4-ad1`/`ad2a`/`ad2b` + H-I4-088 repair |
| 6 | `SessionRuntime` lifecycle/resource/window/DeliveryAck | covered | H-I4-086/087 + `dev-i4-bnd` — sent-drain boundary, aggregate queue cap |
| 7 | `neko-observe` projection/counters | covered | unchanged owner since `8e11de0` deep-sweep repair |
| 8 | Package/reproducibility/operator scripts | covered | unchanged owner since 2026-09-13 independent review |
| 9 | Dependency/build surface | covered | H-I4-085 — `snow`/`ring` build/native surface truthfully classified |
| 10 | Cross-platform CLI/process-test | covered | I4-PORT-01 `42be539` + `5e9b4b2` independent closure |
| 11 | CLI exit/JSON/human contract | covered | `independent-i4-cli-machine` + `dev-i4-cli-h` + `dev-i4-cli-h-independent` |
| 12 | Algorithmic resource boundedness | covered | `dev-i4-bnd` + `independent-i4-bnd-recovery` + `dev-i4-bnd-synthesis` |
| 13 | Release-packet factual consistency | covered | `dd166b3` + `7197abf` + `588ca37` + this note |

## Refill lanes closed

- **I4-FS1** — `dev-i4-fs1-fair-scheduler-20260920` + wording repair `9adf1d2` + independent `reviewer-i4-fs1-fair-scheduler-44a0073` — **no defect**
- **I4-FS2** — `dev-i4-fs2` superseded by H-I4-086/087 repairs — **defects repaired**
- **I4-AD1** — `dev-i4-ad1` superseded by H-I4-088 repair — **defect repaired**
- **I4-AD2a/AD2b** — `dev-i4-ad2a`/`ad2b` — **no defect**
- **I4-BLD** — `dev-i4-bld` superseded by H-I4-085 corrections — **defect repaired**
- **I4-PORT** — `dev-i4-port` + `dev-i4-port-rechallenge` + `independent-i4-port-followup` + `5e9b4b2` — **narrow repair `42be539` + independent closure**
- **I4-CLI-M** — `dev-i4-cli-m` + `independent-i4-cli-machine` — **no defect**
- **I4-CLI-H** — `dev-i4-cli-h` + `dev-i4-cli-h-independent` — **no defect**
- **I4-BND** — `dev-i4-bnd` + `independent-i4-bnd-recovery` + `dev-i4-bnd-synthesis` — **no defect**

## Defects repaired this closure group

- **H-I4-085** — `snow`/`ring` build/native surface truthfully classified (`c4925b6`/`250c4d2`/`97e282d`/`38379b1` provenance)
- **H-I4-086** — `delivery_ack` rejects `end > sent` (`ca629d7`/`3acba04`)
- **H-I4-087** — aggregate `max_queue_records` cap (`d819d38`/`14e2f52`)
- **H-I4-088** — `MemoryCarrier` record-count bound (`26aa4e8`)
- **I4-PORT-01** — `#[cfg(unix)]` SIGTERM/process-test scope (`42be539`)

## Final classification

- **Item 4 remains incomplete** — release items 3 and 4 are open; `RELEASE_CANDIDATE`/`PRODUCTION_READY`/`FREEZE`/`RELEASED` are unchanged.
- **D019 / policy gates remain maintainer-owned** — no cap, TTL, LRU, or capacity value was invented.
- **READY_LIVE: none** — no live/WAN question was created; no VPS authorization was consumed.
- **Evidence classes are distinct** — developer-local executable provenance, developer-local bounded review, reviewer-source-review, reviewer-local execution (none claimed), hosted CI (none), live WAN (none), and performance (none) are separately labelled.
- **Queue is not exhausted by a narrow sweep** — this sweep covered all 13 surfaces against exact-current owners; the queue is genuinely empty of dependency-ready work.
