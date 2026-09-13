# Item-4 core-surface inventory — reachable exact `3352463`

Repository-wide inventory of every implemented core crate and release tool against the independent-review index, after the 2026-09-13 deep item-4 sweep. This inventory reports coverage status; it does **not** self-declare item 4 complete — the final exhaustion decision and any independent security/release approval remain with the reviewer/maintainer.

## Implemented core surfaces and their dedicated bounded challenge

| Surface | Owner | Dedicated bounded challenge | Result |
|---|---|---|---|
| Reliable UDP recovery / ACK | `neko-reliable` | `independent-reliable-udp-02b6eaa` | future/no-sent ACK rejected (`531c82d`/`02b6eaa`) |
| Carrier state / generation / hysteresis | `neko-carrier` | `independent-carrier-state-8f93b93` | no defect |
| CarrierManager margin/hold/migration-back | `neko-carrier` | `independent-carrier-manager-a14cf47` | margin overflow+negative repaired (`a14cf47`/`91f8cd8`) |
| FairScheduler / flow accounting | `neko-carrier` | `independent-fair-scheduler-91f8cd8` | no defect |
| Carrier adapters close/error/resource | `neko-carrier` | `independent-carrier-adapters-8e11de0` | no defect |
| DeliveryLedger overlap/context/watermark | `neko-session` | `independent-delivery-ledger-9697ee7` | no defect |
| SessionRuntime lifecycle/resource/cleanup | `neko-session` | `independent-session-runtime-8e11de0` (+ superseded note) | terminal-cleanup repaired (`9697ee7`); `events` bound `POLICY_BLOCKED_RESOURCE_BOUND` |
| ProcessMessage/Resume/readiness codec | `neko-session` | `independent-process-codec-9697ee7` | no defect |
| DatagramRuntime counters/closed-state | `neko-session` | `independent-datagram-runtime-9697ee7` | no defect |
| Observability buffer/sequence/drop/schema | `neko-observe` | `independent-observability-*` + re-close | O1–O6 repaired (`8f93b93`/`8a4e465`/`ece67b8`/`8e11de0`) |
| Wire parser/decoder | `neko-wire` | `independent-wire-review-4034f86` | no defect (reviewer) |
| Crypto Noise-IK integration/API | `neko-crypto` | `independent-crypto-api-9697ee7` | no defect (not cryptanalysis) |
| CLI portability/exit/JSON/subcommands | `neko-cli` | `independent-cli-portability-8e11de0`, `independent-cli-commands-9697ee7` | no defect |
| Bench evidence semantics | `neko-bench` | `independent-boundedness-validators-9697ee7` + `era4-i` fix | failure-excluded median/P95 (`01b24a0`) |
| Package/reproducibility scripts | `scripts/release/` | `independent-package-release-8e11de0` | no defect |
| Dependency/build surface | workspace | `independent-dependency-surface-8e11de0` | no defect |
| Result validators / netns/deterministic | `scripts/bench/` | `independent-boundedness-validators-9697ee7` | no defect |
| Resource/abuse responder surfaces | `neko-crypto`/CLI | `independent-resource-abuse-review-f184cf2` | no defect (reviewer) |

## Remaining open items — all policy/environment/authority-gated

- `SessionRuntime.events` retained-state bound — `POLICY_BLOCKED_RESOURCE_BOUND` (needs a maintainer/security capacity/retention choice; no numeric cap invented here).
- D019 persistent-replay/source-retention — maintainer policy.
- RSEC-001 adversarial-load/capacity suitability — unestablished; not converted to a local pressure test.
- Signing/key-custody/SBOM/publication trust and frozen-release interop — separate release-authority gates.
- All `READY_LIVE` items — none; WAN/HY2/warm-failover/periodic/soak/package-lifecycle/migration-back/endpoint-migration/key-update/IPv6/PMTUD/Experimental Track remain frozen/absent.

## Status

Every substantial implemented core surface now has a dedicated bounded independent challenge or is explicitly subsumed; no concrete repairable defect is open in this inventory; the only remaining items are genuinely policy/environment/release-authority gates. This is a coverage report for the reviewer — item-4 closure itself is not self-declared.
