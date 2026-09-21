# Developer bounded I4-BND synthesis — algorithmic resource boundedness across owners

**Anchor:** exact source/test tree `42be539` + reviewer/dev commits through `5e9b4b2`.
**Independent inputs:** `independent-i4-bnd-recovery-current-89d249b-20260921.md` (reviewer `neko-reliable` source-level boundedness), `dev-i4-bnd-algorithmic-boundedness-20260921.md` (developer Session/Carrier/CLI boundedness), `dev-i4-fs1-fair-scheduler-20260920.md` (FairScheduler), `dev-i4-fs2-multistream-flow-control-20260921.md` (Session windows), `dev-i4-ad1/2a/2b` (adapter close/resource).

## Cross-owner synthesis

| Owner | Bounded state | Independent challenge | Result |
|---|---|---|---|
| `neko-reliable` `Recovery`/`AckRanges` | `sent`/`watermark_on_reserve`/`outstanding_frames`/`acked_frames`/`ranges` | `independent-i4-bnd-recovery-current-89d249b` — admission caps (`max_sent_packets`, `HARD_MAX_SENT_PACKETS`, `max_frames_per_packet`, `HARD_MAX_FRAMES_PER_PACKET`, `max_ranges`, `MAX_ACK_RANGES`); peer-controlled ACK ranges bounded by `neko-wire` decode cap | no defect — all live/queued state bounded |
| `SessionRuntime` | `send`/`recv`/`received`/`confirmed`/`sent`/`send_inflight`/`recv_window_used`/`streams` | `dev-i4-bnd-algorithmic-boundedness` — `max_queue_records` (aggregate, H-I4-087), `max_queue_bytes`, `max_total_bytes`, `max_record_bytes`, `max_session_window`, `max_stream_window`, `max_streams` | no defect — all runtime-owned state bounded; `events` retained-history is D019/maintainer gate |
| `neko-carrier` `CarrierHealth`/`FailoverController`/`FairScheduler` | `records`/`samples`/`events`/`transitions`/`uncertain`/`received`/`streams`/`order` | `dev-i4-bnd-algorithmic-boundedness` + `dev-i4-fs1`/`dev-i4-fs2` — `max_paths`, `max_samples`, `max_uncertain_entries`/`max_uncertain_bytes`, `max_streams`, `max_session_bytes`, `max_stream_bytes` | no defect — all bounded |
| `neko-carrier` `MemoryCarrier`/`UdpLoopback`/`TcpLoopback` | `queues`/`queue_bytes`/`socket`/`stream`/`closed` | `dev-i4-ad1`/`dev-i4-ad2a`/`dev-i4-ad2b` — `max_queue_bytes` byte + record cap (H-I4-088), `max_datagram_bytes`, `max_frame_bytes`, close/drop semantics | no defect — bounded and truthful |
| `neko-cli` probe/failover/periodic/matrix | `Vec<CommandResult>`/`Vec<ProbeResult>`/`Vec<CycleResult>` | `dev-i4-bnd-algorithmic-boundedness` + `independent-i4-cli-machine` — iteration count bounded by CLI arg; no peer-controlled unbounded growth | no defect — bounded by argument shape |

## Explicitly excluded

- `SessionRuntime.events` retained-history capacity — D019/maintainer/security policy gate; no cap chosen.
- `SessionRuntime.total_bytes`/`last_activity_ms`/`cancelled`/`next_event` — cumulative lifetime facts surviving terminalization by design.
- No capacity-pressure/adversarial-load benchmark was run; BND is semantic boundedness under existing committed limits only.

## Result

No concrete defect found in algorithmic resource boundedness across
`neko-reliable`, `SessionRuntime`, `neko-carrier`, and `neko-cli` owners under
existing committed limits. Every runtime-owned live/queued/map/counter state
is bounded by a committed limit or explicitly classified as a policy gate.

**READY_LIVE: none** — deterministic local evidence only.
