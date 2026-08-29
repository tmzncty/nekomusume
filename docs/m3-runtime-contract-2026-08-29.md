# M3-alpha runtime seam audit and contract

Status: design/acceptance contract only; no runtime behavior or release approval.

## Current seam

`neko-session` currently provides a synchronous `DeliveryLedger`: bounded
segment insertion, per-stream watermarks, monotonic delivery/key/path context,
range state transitions, and exact overlap/conflict handling. It does not own a
queue, clock, socket, carrier, stream lifecycle, cancellation token, or close
state. `neko-carrier` provides carrier-neutral state models plus separate
loopback UDP/TCP adapters, a deterministic `FaultInjectCarrier`, and a
carrier-local `FailoverController`. The CLI is a bounded one-exchange adapter
around `neko-crypto`; it is not yet a Session runtime.

The runtime must therefore be a new orchestration layer. Session owns identity,
stream/record lifecycle, delivery ledger, bounded queues, deadlines, idle and
close state. Carrier owns socket/address/path mechanics and exposes only
bounded send/receive observations. Crypto owns authenticated record operations
and context; Session selects context but never handles sockets. Failover owns
carrier/path evidence and uncertain ranges, while Session decides application
ordering and delivery confirmation.

## M3 contract

- One Session ID is generated once and remains stable across carrier/path changes.
- One stream is required for alpha; stream IDs are bounded and lifecycle is
  `Open -> HalfClosedLocal/HalfClosedRemote -> Closed`, with reset/error as a
  terminal path.
- Send and receive queues are byte-bounded and record-count-bounded. Enqueue is
  atomic: a rejected record changes no ledger, queue, watermark or close state.
- Session limits include maximum streams, records, total application bytes,
  in-flight/uncertain bytes, and peers. Every limit has deterministic errors.
- All operations accept a monotonic clock timestamp. Deadline and idle timeout
  are checked before mutation; cancellation is terminal and idempotent.
- Graceful CLOSE drains already accepted outbound data, emits one CLOSE, and
  becomes terminal after peer CLOSE/ack or bounded close deadline. Error CLOSE
  is immediate, bounded, and never exposes unauthenticated detail.
- Carrier errors are observations, not application delivery. A carrier may be
  replaced without changing Session ID or stream ordering.
- Application receives each byte range exactly once in offset order. Duplicate
  authenticated records are idempotent; conflicting overlap is a stable error.
- Runtime emits versioned events with a monotonic sequence and virtual timestamp;
  event emission is observational and cannot change protocol state.

## Acceptance tests before WAN

1. Open/close lifecycle: session and one stream open, send/receive ordered
   records, graceful close is idempotent and cleanup leaves zero queued bytes.
2. Bounds: queue byte/record, total bytes, stream, peer and uncertain limits are
   atomic and deterministic at zero, maximum and maximum-plus-one boundaries.
3. Time: virtual-clock tests cover operation deadline, idle timeout, close
   deadline and cancellation; no post-terminal mutation is possible.
4. Errors: malformed/unauthorized records produce error CLOSE without leaking
   peer detail; duplicate records do not duplicate application delivery;
   conflicting overlap closes with stable error.
5. Carrier independence: the same Session test suite runs over Memory,
   FaultInjectCarrier and loopback TCP/UDP adapters without Session importing
   socket/address types.
6. Failover: after bounded UDP death, uncertain records are resent on a
   validated TCP path, deduplicated, DELIVERY_ACK advances watermark, and the
   application byte stream has no gaps, corruption or duplicates.
7. Cleanup: every test asserts no queued bytes/records, no live stream, no
   retained uncertain range and no temporary listener after terminal close.

## Non-goals and gates

M3-alpha does not enable 0-RTT, concurrent multipath, FEC, proxy/tunnel
forwarding, production listeners, or a security/release claim. Real WAN is
permitted only after simulator acceptance and remains an isolated observation.
