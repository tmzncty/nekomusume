# Independent bounded review — live reliable-UDP integration S1–S9 — exact `94cfd47`

Bounded independent challenge of the new live reliable-UDP integration surface: `PathRecovery` in `crates/neko-carrier/src/lib.rs` (`:4041-4240`), the packet-ACK grammar + packet header in `crates/neko-wire/src/lib.rs` (`:774-940`), and the loopback/auth/loss/failover/observability integration tests, at reachable exact `94cfd4732dca6aae3e88fd188125653d9b44e900`. This is a static + integration-test review of the new surface; not a WAN/live claim, not an independent security/release approval.

## Challenged invariants and result

- **Authentication/evidence-domain separation — holds.** The UDP datagram is the `SecureSession` record: the 8-byte record sequence is the AEAD nonce (which doubles as the packet number) and the ciphertext plaintext is one NK record. A `RecordType::Ack` NK record in that plaintext carries the packet-ACK. `PathRecovery::on_ack` applies an ACK only after `open` succeeds and only for the matching `path_generation`; tampered nonce/ciphertext fails `open` before any RTT/loss/PTO mutation (`reliable_udp_auth`). Packet feedback is structurally unable to mutate Session delivery — `PathRecovery` holds no `SessionRuntime`/`DeliveryLedger`/`confirm_received` reference.
- **Future/unsent/duplicate ACK — holds.** `Recovery::on_ack` rejects `largest > largest_sent` and any ACK on an empty sent set before mutation (already gated at `02b6eaa`/`531c82d`); `PathRecovery` additionally rejects a wrong `path_generation` as `GenerationMismatch`. A duplicate ACK re-ACKs already-retired packets and releases zero bytes (no double-release).
- **Packet/frame exact-once accounting — holds.** `on_sent` charges `reno.sent` + `packets_sent` once per accepted packet; `on_ack` releases `acked`/`lost` bytes exactly once via the engine and `packets_lost`; a re-ACK frees nothing (`ack_retires_bytes_in_flight_exactly_once`, `duplicate_and_old_ack_do_not_double_retire`).
- **Retransmission crypto freshness — holds.** Retransmit output is `FrameId`s; the caller re-encodes the frame into a *fresh* `seal` (new nonce/packet number) — the old encrypted packet image is never resent (`lost_packet_retransmits_frame_level_and_delivers_exactly_once`).
- **Timeout/PTO/loss state transitions — holds.** `on_pto` returns a bounded set of oldest outstanding `FrameId`s for re-encoding (never the raw image); loss is declared only via the engine's reorder/time threshold.
- **Memory/queue bounds — holds.** `Recovery` caps `max_sent_packets`/`max_frames_per_packet`; `AckRanges`/`AckPayload` are hard-bounded (`MAX_ACK_RANGES=32`, varint canonical, over-count/trailing/inversion rejected); `PathRecovery` adds only two u64 counters.
- **Manager/fallback boundary — holds.** `health_sample()` maps packet-level RTT/loss/PTO into the existing `HealthSample`/`HealthObservation` domain — a packet ACK never validates a Path or confirms Session delivery. A measured 625/mille loss degrades UDP through normal hysteresis and drives the existing warm TCP standby promotion + uncertain replay (`reliable_udp_failover`); a clean sample produces no false fallback.
- **Secret-safe observability — holds.** `recovery.*`/`carrier.*` events carry counters/ids only (no secrets/payload); recovery events are distinct from Session delivery (`packet_recovery_events_are_layered`).

## Wire grammar (S2) — holds

`AckPayload`/`AckRangeWire`/`encode_ack`/`decode_ack`/`encode_packet`/`decode_packet`: canonical sorted non-overlapping **non-adjacent** inclusive ranges (`start >= prev_end+2`), `MAX_ACK_RANGES=32`, varint canonical, over-count/inversion/truncation/trailing/`largest_below_ranges` rejected; outer version stays candidate/non-frozen. Decode fuzz extended to `decode_ack`/`decode_packet` — 7M runs clean.

## Evidence

- `cargo test` on exact `94cfd47`: `path_recovery_tests` (7), `reliable_udp_auth` (2), `reliable_udp_exchange` (5), `reliable_udp_failover` (2), `reliable_udp_observe` (1) all pass.
- Exact-tree local provenance: `docs/local-gate-94cfd47-20260913.md` (`check.sh`=0, clippy clean, clean tree, UTC `10:19:23Z`–`10:21:20Z`).
- No code change in this review; no defect found. Loopback/local integration only — no `READY_LIVE` yet; release/governance state unchanged.
