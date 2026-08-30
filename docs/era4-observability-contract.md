# Era-4 observability contract

Status: **stable v1 contract**, derived from the live M2/M3 implementation at `7d259af`. This freezes diagnostic vocabulary and evidence shapes; it does not claim that every producer is wired yet and does not unblock public or production use.

Machine contracts:

- [`schema/observability-event.v1.json`](../schema/observability-event.v1.json)
- [`schema/health.v1.json`](../schema/health.v1.json)
- [`schema/diagnostic-bundle.v1.json`](../schema/diagnostic-bundle.v1.json)

## 1. Rules

1. A Session is the top correlation scope. Streams and carrier paths are children, never substitutes for Session identity.
2. IDs are opaque, process-generated correlation values. They are not keys, tokens, cookies, addresses, peer public keys, Noise transcripts, packet payloads, or raw endpoint locators.
3. Fields not observed are absent or `null` where the schema permits it. They are never emitted as invented zeroes. Counter zero means the producer observed the complete counter interval and counted zero.
4. Durations use `_us`; sizes use `_bytes`; ratios use `_per_mille`; wall timestamps use Unix `_ms`; monotonic ordering uses `sequence`.
5. Event names and enum values are append-only within v1. Renames, unit changes, changed meaning, required-field removal, or identifier semantic changes require v2.
6. Human summaries are derived, bounded views. JSON fields remain authority.

## 2. Correlation without secrets

Every event has:

- `session_id`: required opaque logical Session correlation ID;
- `stream_id`: optional Session stream ID;
- `carrier_id`: optional carrier instance (`udp`, `tcp`, or future implementation ID is in `data.carrier_kind`, not encoded here);
- `path_id`: optional path instance, local to the Session;
- `connection_id`: optional ephemeral carrier connection correlation only;
- `trace_id`: optional diagnostic capture correlation.

A carrier switch retains `session_id`; old and new carriers use `carrier_id`/`path_id`. A stream retains `stream_id` across carrier changes. Packet numbers and frame IDs are diagnostic transport values, not Session delivery acknowledgements.

Forbidden everywhere in emitted documents: private/identity key material, PSKs, join/auth tokens, cookies, plaintext or ciphertext payload bodies, raw IP addresses, ports, DNS names, full socket tuples, Noise handshake bytes, and environment dumps. Error text is represented by a bounded stable `error_code`, never an arbitrary exception string.

## 3. Stable events

| Event | Required metric/evidence intent |
|---|---|
| `session.started`, `session.stopped` | lifecycle and final reason |
| `stream.opened`, `stream.closed` | stream correlation and priority |
| `carrier.health_sample`, `carrier.health_transition` | RTT/loss/PTO sample and state hysteresis |
| `carrier.switch_started`, `carrier.switch_completed`, `carrier.switch_failed` | from/to carrier/path, reason, recovery latency/outcome |
| `recovery.rtt_updated` | latest/min/smoothed/variance RTT |
| `recovery.loss_detected` | lost packet/byte counts; packet loss is not delivery loss |
| `recovery.pto_fired` | PTO count/deadline and bounded probe frames |
| `recovery.frame_retransmitted` | frame retransmission count/bytes, never “packet replay” |
| `flow.blocked`, `flow.resumed` | stream/session queued bytes and blocking reason |
| `scheduler.dequeued`, `scheduler.starvation_guard` | selected stream/priority, queue and fairness evidence |
| `migration.started`, `migration.validated`, `migration.completed`, `migration.rejected` | generation, validation/hold gates, rejection reason |
| `crypto.key_update_started`, `crypto.key_update_completed`, `crypto.key_update_rejected` | old/new numeric key phase and fail-closed result; never key bytes |
| `resource.limit_hit` | stable resource name, configured limit, observed value |
| `diagnostic.events_dropped` | bounded-buffer eviction count and retained sequence floor |
| `datagram.admitted`, `datagram.dropped` | bounded Session datagram admission and drop decisions; no payload evidence |

### Switch reasons

`pto_threshold`, `health_failed`, `path_unavailable`, `carrier_error`, `migration_preferred`, `probe_validated`, `operator_requested`, `resource_pressure`, `recovery_hysteresis`.

A reason says **why selection changed**, not whether it succeeded. Result is expressed by event name and `outcome`. Existing M3 UDP→TCP failover maps the hard-failure PTO threshold to `pto_threshold`; validated TCP→UDP migration maps to `probe_validated` or `migration_preferred`.

## 4. Metrics

The v1 schemas cover these live implementation facts:

- RTT: `latest_us`, `min_us`, `smoothed_us`, `variance_us` from `RttEstimator`;
- loss: packets/bytes and per-mille window estimate;
- PTO: current count, total firings and probe frames;
- retransmission: frames and bytes (frames are retransmitted, not packet identities);
- flow: Session/stream queue bytes, stream count, blocked count and configured maxima from `FlowLimits`;
- scheduler: dequeue totals by priority, starvation-guard activations and maximum observed wait/dequeue gap;
- migration/switch: attempts, validation/rejections, completions, switches, recovery latency, duplicate and delivered bytes;
- key update: numeric phase, started/completed/rejected counts and last result;
- resources: sent-packet/frame bounds, uncertain data, health paths, event capacity/use/drop totals and process memory where observed.

Metrics are monotonic counters unless named `current_*`, `latest_*`, `min_*`, `max_*`, or documented as gauges. Saturation must emit `resource.limit_hit`; silently wrapping is forbidden.

## 5. Health JSON and human summary

`nekomusume.health.v1` is a point-in-time, secret-free snapshot. `schema_version` is integer `1` in addition to the string discriminator. Backward-compatible optional additions retain v1; incompatible interpretation requires a new discriminator and endpoint/content negotiation.

`status` is `ok`, `degraded`, `failed`, or `unknown`. `unknown` is correct when evidence is insufficient. `human_summary` is one line, at most 512 characters, with no endpoint locator or arbitrary error text. Recommended rendering:

```text
session=<id> status=degraded carrier=tcp streams=4 rtt=24.1ms loss=12‰ pto=2 switches=1 events=128/128 dropped=7
```

The JSON values, not this text, are authoritative.

## 6. Bounded event buffer

The producer declares `capacity` in `[1,1024]`. Events are ordered by strictly increasing `sequence`. On overflow it evicts oldest-first, increments `dropped_total`, advances `oldest_sequence`, and emits/coalesces `diagnostic.events_dropped`. Event documents are capped at 16 KiB after UTF-8 JSON encoding; diagnostic bundles are capped at 8 MiB. Buffering must not block transport progress.

Health exposes capacity, retained count, oldest/newest sequence and drops. A gap or drop is evidence of incomplete diagnostics, not evidence that no protocol event occurred.

## 7. Diagnostic bundle and negative evidence

`nekomusume.diagnostic-bundle.v1` contains one health snapshot, a bounded ordered event slice, capture bounds, build/source identity, redaction report and explicit negative evidence. It must be created with restrictive permissions, use exclusive creation, and be safe to delete. It must not include identity files, packet captures, payloads, environment variables, command lines, or host network configuration.

Negative evidence distinguishes:

- `observed_absent`: complete observation over a declared interval and zero matching events;
- `not_observed`: producer or metric was unavailable;
- `buffer_gap`: relevant evidence may have been evicted;
- `not_applicable`: condition did not apply.

Each entry names the claim, interval, producer, completeness, and reason. Only `observed_absent` with `completeness=complete` supports “did not happen.” Missing events, empty arrays, `unknown`, and zero after a buffer gap do not.

## 8. Compatibility examples

Adding an optional `ecn_marked_packets` counter is v1-compatible. Changing loss from per-mille to percent, exposing a hashed raw socket tuple as `path_id`, changing `sequence` ordering scope, or treating PTO as Session failure is incompatible and requires a new contract plus migration notes.
