# Independent bounded I4-FS2 review — Session multi-stream / flow-control accounting

**Reachable review anchor:** exact `11cdb08e7bc56e105dadb0092c7636a4ca346b3d`.

**Executable owner history relevant to this review:** H-I4-086 source repair exact `ca629d702cf832c12bf74bdfc5f9d07ebb77ab17` plus companion tests exact `3acba049fee5f9297cc8a4c3867250635c255717`; H-I4-087 source repair begins at exact `14e2f520a0fc9d41bcfcb1bd480758008a1dd4ea` and dedicated regression/provenance source-test anchor is exact `d819d38559d60b6bd99df8a2453321809c046116`.

## Scope and inspected owners

This is the independent remainder re-challenge requested after H-I4-086 and H-I4-087. I read exact-current:

- `crates/neko-session/src/lib.rs` — `RuntimeLimits`, `SessionRuntime::{queue_send,pop_send,receive,pop_receive,delivery_ack,clear_runtime_state}`, per-stream/session accounting and current deterministic regressions;
- `docs/specs/nekomusume-session-v0.md` — Session delivery-evidence boundary;
- `docs/carrier-architecture.md` — Session-delivery versus Carrier/packet-feedback separation;
- `SECURITY.md` — per-connection/global resource-bound requirement;
- `docs/reviews/dev-i4-fs2-multistream-flow-control-20260921.md` — developer bounded review, with the H-I4-086/H-I4-087 portions explicitly superseded by the later repairs.

The challenged claims were: stream/session byte-window interaction, aggregate queued-record/byte accounting, queued versus drained/sent-unacked ownership, reject-before-mutate behavior, duplicate and terminal ownership, cross-stream isolation, receive-window ownership/release, and separation of Session delivery evidence from Carrier packet feedback.

## Independent challenge

### Outbound ownership and ACK release

`queue_send` charges both the per-stream `send_inflight` and aggregate `session_send_inflight` before enqueue. `pop_send` removes only queue ownership (`queued_bytes`) and advances the per-stream drained/sent end; it deliberately does **not** release delivery-window credit. `delivery_ack` rejects a forward gap, rejects any end beyond the drained/sent boundary, and releases only the newly confirmed prefix from both the per-stream and Session inflight counters. This closes the pre-H-I4-086 ACK-before-drain counterexample without promoting local queue admission or Carrier packet feedback into Session delivery evidence.

Current reachable regressions specifically challenge ACK-before-drain, a first-record-drained/second-record-still-queued prefix, cross-stream drain isolation, resumed drain-before-ACK ordering, and terminal release of the new sent bookkeeping.

### Aggregate queue ownership

`queued_records()` is `send.len() + recv.len()`. Both `queue_send` and `receive` now reject before mutation when that aggregate count reaches `max_queue_records`; queue bytes are likewise one shared `queued_bytes` budget. H-I4-087 regressions cover both admission orders and the positive case where `pop_send` / `pop_receive` frees an aggregate record slot for the opposite direction.

The rejected paths occur before offset/watermark advancement, queue insertion, byte/window accounting and success-event emission for the challenged aggregate-cap cases. The dedicated regressions also assert no new event for those rejected admissions.

### Receive-window ownership

`receive` verifies exact next-offset ordering, total/window/queue limits and only then advances `next_receive`, stores duplicate evidence, enqueues the record and charges per-stream plus Session receive-window usage. `pop_receive` releases the queued bytes and both receive-window counters. An exact retained duplicate below `next_receive` is idempotent and does not re-charge queue/window bytes; a conflicting old range or forward gap fails closed.

### Cross-stream and terminal isolation

Per-stream send/receive accounting remains keyed by `StreamId`, while Session counters are intentionally aggregate. Draining stream A does not make stream B ACK-eligible. Existing multi-stream tests challenge independent stream closure, stream IDs 0/1/2+, ordering, and shared Session resource limits. Terminal paths clear send/recv queues, dedup state, confirmation/sent maps, per-stream window maps, aggregate Session window counters, queued bytes, stream runtime state and close deadline.

### Evidence-domain separation

The provisional Session contract defines delivery confirmation as peer acceptance of logical bytes for transport delivery, distinct from application delivery/effect. The carrier architecture separately keeps UDP packet ACK/RTT/loss feedback below Session delivery state. Nothing in the repaired `SessionRuntime` turns Carrier feedback into `delivery_ack` evidence.

## Result

**No additional concrete source-decided defect found in the bounded I4-FS2 remainder after H-I4-086 and H-I4-087.** The earlier developer no-finding note is not accepted as-is for its falsified pre-repair statements; this note supersedes those portions and anchors the current bounded conclusion to the repaired reachable tree.

## Evidence boundary / exclusions

- This was a source/spec/test review. I did **not** execute reviewer-local tests in this review environment.
- Developer-reported exact-tree provenance for `d819d38559d60b6bd99df8a2453321809c046116` is retained in `docs/notes/h-i4-087-provenance-d819d38-20260921.md`: clean worktree, `PYTHONDONTWRITEBYTECODE=1 bash scripts/check.sh` exit 0 on the recorded rerun, `git diff --check` exit 0, Linux x86_64, rustc 1.98.0 stable. That remains developer-local evidence, not reviewer-local or hosted CI.
- No GitHub-hosted status/workflow run was visible for exact `d819d38`; absence of hosted evidence is not interpreted as failure.
- No decoder/parser/crypto framing changed in this review lane; no fuzz claim is made.
- Whether a successful DeliveryAck or duplicate-only observation should refresh idle-liveness time is not decided by the current I4-FS2 accounting contract and is not invented here.
- The already-known retained `SessionRuntime.events` capacity question remains the existing maintainer/security policy gate; this review does not choose a TTL/LRU/history-size/capacity value.
- No WAN/live, performance, release, production, freeze or security-approval claim follows. `READY_LIVE: none` remains unchanged for this lane.
