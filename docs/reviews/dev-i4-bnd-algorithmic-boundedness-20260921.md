# Developer bounded I4-BND review — algorithmic resource boundedness

**Anchor:** exact source/test tree `26aa4e8` + reviewer/dev commits through `536d59a`.
**Scope:** `SessionRuntime` maps/queues/counters, `neko-carrier` `CarrierHealth`/`CarrierHealthEvidence`/`FailoverController`/`FairScheduler`/`MemoryCarrier`, `neko-cli` CLI loops/maps, and retained/live-state capacity under existing committed limits.

## Per-surface coverage

| Surface | Bounded state | Limit enforced |
|---|---|---|
| `SessionRuntime.send`/`recv` | `VecDeque<OutboundRecord>`/`VecDeque<InboundRecord>` | `max_queue_records` aggregate cap (H-I4-087) + `max_queue_bytes` |
| `SessionRuntime.received`/`confirmed`/`sent` | `BTreeMap<(StreamId,u64),Vec<u8>>`/`BTreeMap<StreamId,u64>` | `max_streams` caps stream count; per-stream byte windows cap stored payload |
| `SessionRuntime.send_inflight`/`recv_window_used` | `BTreeMap<StreamId,usize>` | `max_streams` caps entries; `max_stream_window`/`max_session_window` cap bytes |
| `SessionRuntime.streams` | `BTreeMap<StreamId,RuntimeStream>` | `max_streams` cap; released on terminal via `clear_runtime_state` |
| `SessionRuntime.events` | `Vec<RuntimeEvent>` | **retained-history capacity is a maintainer/security policy gate** — intentionally not capped here; D019 owns the decision |
| `CarrierHealth.records` | `BTreeMap<PathId,HealthRecord>` | `max_paths` cap |
| `CarrierHealthEvidence.samples`/`events`/`transitions` | `Vec<…>` FIFO | `max_samples` cap with `remove(0)` eviction |
| `FailoverController.uncertain`/`received` | `BTreeMap<DataId,Vec<u8>>` | `max_uncertain_entries`/`max_uncertain_bytes` cap |
| `FairScheduler.streams`/`order`/`session_bytes` | `BTreeMap`/`Vec`/`usize` | `max_streams`/`max_session_bytes`/`max_stream_bytes` cap |
| `MemoryCarrier.queues`/`queue_bytes` | `[VecDeque; 2]`/`[usize; 2]` | `max_queue_bytes` byte cap + record-count cap (H-I4-088) |
| `neko-cli` `matrix_probe`/`authenticated_probe`/`failover`/`periodic` loops | `Vec<CommandResult>`/`Vec<ProbeResult>` | iteration count bounded by CLI arg (`--count`/`--duration`/`--matrix-size`) — no peer-controlled unbounded growth |

## Explicitly excluded policy gates

- `SessionRuntime.events` retained-history capacity — D019/maintainer gate, no cap chosen here.
- `SessionRuntime.total_bytes`/`last_activity_ms`/`cancelled`/`next_event` — cumulative lifetime facts that survive terminalization by design.

## Reachable regressions named

- `aggregate_queue_cap_send_then_receive` / `aggregate_queue_cap_receive_then_send` / `aggregate_queue_cap_pop_frees_slot` (H-I4-087)
- `empty_messages_are_bounded_by_record_count` (H-I4-088)
- `interactive_burst_is_bounded_and_order_is_repeatable` / `flow_limits_are_atomic`
- `health_evidence_bounded_samples_and_transitions` / `health_evidence_window_evicts_oldest`
- `failover_track_uncertain_capacity` / `failover_received_capacity`
- `receive_ordering_and_session_resource_limits_apply_across_streams`
- `idle_timeout_releases_all_runtime_owned_state`

## Result

No concrete defect found in algorithmic resource boundedness under existing
committed limits. All runtime-owned live/queued state is bounded by an
existing cap; `SessionRuntime.events` retained-history capacity is explicitly
classified as a maintainer/security policy gate, not a defect.

**READY_LIVE: none** — deterministic local evidence only.
