# Measurement mode (`NEKO_MEASUREMENT=1`) — limit-relaxation provenance

**Scope:** Option C → A per the 2026-10-07 adjudication (Perlica): ship the
CLI-layer relaxation only, with every ceiling seated exactly at the
`neko_session` library hard limits. Library constant changes (Option B) are
explicitly out of scope this round — they require a fresh security review.

## Research conclusion (recorded as limit evidence)

The protocol's current hard limits **are part of what the limit research is
measuring**, not an obstacle: a single Session runtime's lifetime data
ceiling is 64 MiB, its declared queue-record ceiling is 65,536, and its
per-frame data ceiling is 4,096 bytes. These are enforced in the library and
protected by 460 assertions (era4 resource-limit tests); the CLI cannot
exceed them by argument alone.

### Enforcement chain (all line numbers verified 2026-10-07)

| Library constant (crates/neko-session/src/lib.rs) | Value | CLI derivation that hits it |
|---|---|---|
| `HARD_MAX_RUNTIME_STREAMS` (lib.rs:9) | 4,096 | multistream `max_streams = streams` |
| `HARD_MAX_RUNTIME_QUEUE_RECORDS` (lib.rs:10) | 65,536 | exchange `max_queue_records = count + 2` (main.rs:1798-1799); periodic same (periodic.rs:129); multistream `streams * records + 1` (multistream.rs:321) |
| `HARD_MAX_RUNTIME_QUEUE_BYTES` (lib.rs:11) | 16 MiB | `max_queue_bytes = bytes * (count + 1)` (main.rs:1800; periodic.rs:130) |
| `HARD_MAX_RUNTIME_TOTAL_BYTES` (lib.rs:12) | 64 MiB | periodic `max_total_bytes = bytes * count * 2` (periodic.rs:131); multistream `max_total_bytes = streams * records * bytes` (multistream.rs:323) |
| `HARD_MAX_RUNTIME_RECORD_BYTES` (lib.rs:13) | 1 MiB | multistream `max_record_bytes = bytes` |
| `PROCESS_FRAME_MAX` (lib.rs:1013) | 4,096 | multistream `secure_write` encodes `ProcessMessage::Data` — `PROCESS_DATA_HEADER_LEN` (lib.rs:1023, = 30) + payload must fit |

**Key semantics:** `total_bytes` is a **lifetime accumulator** — it only
increases on queue/receive (lib.rs:1576, lib.rs:1681) and never decreases on
ACK/pop (the `lifetime_bytes` tests at lib.rs:2500/2538 pin this). So "64 MiB
per Session" bounds the whole session's data volume, not a rate. Rate
measurement is unaffected: throughput = total / time, and short sessions can
still saturate the link. A 24 h soak runs at a low traffic rate and records
when `total_bytes` hits 64 MiB — hitting the ceiling is itself a data point.

## Relaxed ceilings (measurement mode only; defaults byte-identical)

| Ceiling | Default | Measurement | Seating |
|---|---|---|---|
| session/workload duration | 30 s / 600 s | 86,400 s (24 h) | pure CLI constant (`MAX_DURATION`/`MAX_WORKLOAD_DURATION`, main.rs:72-73); no library participation |
| `--count` (client/server/failover/endpoint-rebind) | 64 | 13,980 | largest count whose derived limits fit ALL three: `count+2 ≤ 65,536` records, `1200·(count+1) ≤ 16 MiB` queue bytes (13,981 overshoots by 1,184 B), `1200·count ≤ 64 MiB` total — safe for every admissible payload |
| periodic `--count` | 600 | 65,530 | `count + 2 ≤ HARD_MAX_RUNTIME_QUEUE_RECORDS`; pairing guard (below) rejects incompatible payload sizes |
| periodic application bytes | 1 MiB | 32 MiB | runtime derives `max_total_bytes = bytes·count·2`; 32 MiB app = 64 MiB lifetime, exactly at `HARD_MAX_RUNTIME_TOTAL_BYTES` |
| multistream streams | 16 | 256 | review-agreed research value; inside `HARD_MAX_RUNTIME_STREAMS` = 4,096 |
| multistream records/stream | 64 | min(8,192, 65,535/streams) | declared `max_queue_records = streams·records + 1 ≤ 65,536`; at 256 streams this is 255 |
| multistream record bytes | 1,024 | 4,000 | `30 + 4000 ≤ PROCESS_FRAME_MAX = 4096` (66 B headroom) |
| multistream total payload | 1 MiB | 64 MiB | `= HARD_MAX_RUNTIME_TOTAL_BYTES` (lifetime semantics) |

**Measurement-only derived guard** (periodic config): even under measurement
mode, `count + 2 ≤ 65,536` and `bytes·(count+1) ≤ 16 MiB` are enforced
(measurement.rs/periodic.rs config), because the periodic client queues a
record before awaiting each ack, so the queue holds `bytes·(count+1)` in
steady state. Large counts therefore pair with small payloads; the pairing
rejection is explicit, not a runtime panic.

**Off-by-one notes vs. the adjudication numbers:** count 13,980 (not 65,530
for `--count`: the adjudication's 65,530 is the *periodic* count ceiling;
the generic `--count` ceiling must additionally fit the 16 MiB queue-byte
derivation at the full 1,200 B payload, which caps it at 13,980); multistream
records `min(8192, 65536/streams)` corrected to `min(8_192, 65_535/streams)`
(the declared value is `streams·records + 1`, so `s·r ≤ 65,535`).

## Switch contract

- `NEKO_MEASUREMENT=1` (exact string) enables; anything else (unset, `0`,
  `true`, …) is off. Default is off; no CLI flag equivalent (the env shape
  matches the preauth experimental tooling convention).
- Every relaxed command prints `measurement_mode=true relaxed_limits` as its
  first stdout line (client, server, failover server/client,
  endpoint-rebind server/client, periodic server/client, multistream,
  workload). Non-relaxing commands (probe/matrix, lab, capabilities,
  keygen, scheduler-fairness, key-update, health-observe) print nothing.
- `capabilities --json` gains `"measurement_mode":<bool>` after
  `secret_free` and `"periodic_total_bytes_max":<n>` in `limits`;
  default-mode output numbers are unchanged (`count_max:64`,
  `duration_seconds_max:30`, `workload_duration_seconds_max:600`,
  `periodic_total_bytes_max:1048576`). The exact-published-line pin test was
  updated to carry the two new fields.
- USAGE notes the variable with a research-only warning.

## Tests

`crates/neko-cli/tests/measurement_mode.rs` (8 tests, all subprocess,
loopback-only):

1. `default_windows_reject_at_their_old_edges` — duration 31 / count 65 /
   workload 601 / periodic count 601 / multistream streams 17 & records 65
   all still reject with the exact old messages.
2. `default_capabilities_and_stdout_carry_no_measurement_trace` — default
   JSON reports `measurement_mode:false` with the old numbers; a default
   command prints no marker.
3. `measurement_mode_relaxes_duration_and_count_and_marks_output` —
   duration 700 + count 100 accepted (fails later at the unreachable
   target, exit 2, no "outside"); the marker line prints; 86,401 / 13,981 /
   workload 86,401 reject at the new ceilings.
4. `measurement_mode_relaxes_periodic_ceilings_up_to_the_library_limits` —
   count 65,530 accepted & 65,531 rejected; duration 86,401 rejected;
   1,200 B × 13,980 = 32 MiB accepted (marker printed) & 13,981 rejected by
   the application/queue guard.
5. `measurement_mode_relaxes_multistream_ceilings_up_to_the_library_limits` —
   256×255×1,000 (65,280,000 B ≤ 64 MiB) accepted; streams 257, bytes
   4,001, records-at-256-streams 256, and the 261 MB total all rejected.
6. `measurement_capabilities_report_the_active_limits` — JSON reflects the
   active ceilings.
7. `measurement_periodic_exchange_runs_above_the_default_duration_window` —
   full loopback periodic exchange at duration 700 s (window relaxed; the
   3-record exchange itself completes in ms), marker on both ends,
   clean exits.
8. `measurement_multistream_exchange_runs_above_the_default_stream_window` —
   full loopback multistream exchange with 17 streams, clean exits.

Plus `crates/neko-cli/src/measurement.rs` unit tests: every ceiling's
derivation pinned against the library constants (and a `const _` block
fail-the-build pin), so a library change that moves a hard limit breaks here.

## Not changed

- No default-window number moved; no existing test assertion was weakened
  (the two capability-pin templates gained exactly the two new default
  fields).
- No network behavior added; no governance flags touched; no library
  constants touched (Option B deferred).
- Loopback-only testing; no VPS/WAN/sudo.
