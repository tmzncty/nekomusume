# Independent bounded resource-boundedness + result-validator cross-sweep — exact `9697ee7`

Bounded independent cross-sweep for (RL9) static algorithmic boundedness across implemented core owners and (RL10) deterministic/netns/result-validator evidence correctness, at reachable exact `9697ee7a0045b39c6ef46c1951fefea486ccf13b`. Not a pressure/capacity test, not an independent security/release approval; no new policy value is chosen.

## RL9 — algorithmic boundedness cross-sweep — no new defect

Grepped every `.push`/`.insert`/loop site in `crates/neko-*/src`. Every retained collection is governed by a declared hard limit checked **before** mutation:

- `SessionRuntime` send/recv/received/confirmed/inflight/window — `max_streams`/`max_queue_records`/`max_queue_bytes`/`max_total_bytes`/`max_*_window` (`checked_add`/`checked_sub`).
- `DeliveryLedger` segments/bytes/streams — `max_connection_bytes`/`max_streams`/`max_offset_jump`/`max_reorder`.
- `neko-carrier` `uncertain`/`received`/`ranges`/`paths`/`samples`/`readiness_observation_ids` — `max_uncertain_entries`/`max_uncertain_bytes`/`max_uncertain_ranges`/`max_paths`/`max_samples`.
- `FairScheduler` streams/order/queues — `max_streams`/`max_stream_bytes`/`max_session_bytes`.
- `neko-observe` ring — `capacity` clamp (emission capped per event, overflow evicts bounded).
- `neko-crypto` `ReplayWindow`/`NonceManager` — bounded window + non-wrapping nonce.

External-magnitude loops (`for _ in 0..n`, `while`, `loop`) are either test/fixture-scoped or bounded by a declared limit (`max_states_global`, `emit_admitted`, ring `capacity`, `frames.len()`). **The single unbounded retained surface remains `SessionRuntime.events` — already classified `POLICY_BLOCKED_RESOURCE_BOUND` (maintainer/security capacity choice; no cap invented here).** No new defect.

## RL10 — result validators / deterministic-netns evidence — no defect

- `scripts/bench/validate-process-resource.py` is a fail-closed validator: `exact` field-set matching, bounded/finite/nullable `number` checks, `paired` non-null consistency, ISO-`Z` timestamps, `cleanup` matched against the exact expected dict, and `application_bytes`/`sampling`/identity bounds. It cannot accept a partial or extra-field artifact.
- `run-netns.sh` summary computes `median_rtt_ms`/`p95_rtt_ms` from `avg_rtt_ms != null` samples only — null-RTT (e.g. 100%-loss) scenarios are excluded rather than zero-filled, and `summary.failures` counts `pass==false` independently.
- `run-isolated.py` (deterministic fixture) reports per-scenario `failures` plus `all_frames_delivered`; the deterministic samples are synthetic fixture output (no network mutation), consistent with its declared mode.
- Partial-prefix/negative artifacts: harnesses retain typed negatives (`run-repeated-warm-failover*`, `run-live-warm-failover-cycle`) rather than rewriting them; `trap`-based cleanup and `cleanup`-dict validation preserve truthful cleanup evidence.

## Evidence

- Static review on exact `9697ee7`; `scripts/check.sh` keeps validator/bench regressions green.
- No code change; no defect found beyond the already-deferred `SessionRuntime.events` policy bound. No new `READY_LIVE` question; release/governance state unchanged.
