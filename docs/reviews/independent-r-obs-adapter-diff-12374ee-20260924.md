# R-OBS/ADAPTER-DIFF — observability + Memory/UDP/TCP adapter owner-diff review

**Exact source/control-flow anchor:** `12374ee99f5bf5bdd5096e8d5e24a88d9046d188` (source tree differs from `5135d8e` only by the preceding review note).

**Review class:** bounded independent GitHub source/owner-diff review. No reviewer-local Rust execution, hosted CI, WAN run, fuzz run, or performance conclusion is claimed.

## Reuse baselines

- observability stable-v1 re-close: `18f7c58fa9504c9f0c0bd4e4bb1f17017083d09a`;
- Memory adapter challenge chain including H-I4-088 / I4-AD1, reused through `8e905163` and the later dedicated 2026-09-21 MemoryCarrier rechallenge;
- UDP adapter no-finding: `5f24cbffcbe13386e4016059f8a8253a5fd1a434`;
- TCP adapter no-finding: `e8fbc65e1ccd8766e000d48054411d53fcf94555`.

## Owners and invariants challenged

1. `crates/neko-observe/src/lib.rs`: event projection must remain bounded/secret-free; datagram generic-drop and `queue_dropped` deltas must preserve mixed reason attribution; retransmit/PTO/scheduler totals and resource high-water projection must not promote diagnostic observations into transport/session evidence.
2. `MemoryEndpoint`: byte + record-count bounds, atomic close/peer-close/capacity/enqueue ordering, queued-data drain after local close, empty-message boundedness, and carrier-neutral error mapping.
3. `UdpLoopbackEndpoint`: size rejection before send, nonblocking `WouldBlock`, local idempotent close, no fabricated peer-close/session-delivery semantics, and bounded receive allocation.
4. `TcpLoopbackEndpoint`: frame bound, finite loopback read/write timeout, exact length-prefix read, truncation/error mapping, and idempotent shutdown/closed-state behavior.
5. Current `Carrier` abstraction must not make changed Recovery/CarrierState/runtime-manager code retroactively alter these adapter semantics.

## Owner-diff result

**No finding in this bounded lane.**

- `18f7c58f..12374ee` does not modify `crates/neko-observe/src/lib.rs`. The exact-current observability owner therefore reuses the stable-v1 review boundary. Candidate B remains closed: current `record_datagrams` explicitly computes `queue_full = queue.min(dropped)` and classifies the generic-drop remainder as `terminal`, while bounding per-batch emission by producer capacity.
- Exact-current Memory/UDP/TCP adapter source was compared against the dedicated 2026-09-21 adapter-review tree. The adapter contracts and their owning tests are unchanged in the reviewed sections. Later edits to `crates/neko-carrier/src/lib.rs` are in separately challenged Recovery/Reno/PTO, CarrierState, manager/health/migration and test-support owners rather than these adapter implementations.
- Memory still bounds zero-length records by queue element count as well as bytes, uses one pair mutex to linearize close/peer-close/capacity/enqueue, preserves pre-close queued data for peer drain, and maps checked accounting failure fail-closed.
- UDP still allocates at most `max_datagram_bytes + 1`, distinguishes `WouldBlock`, rejects oversize, and treats local close as local terminal state without inventing peer-close evidence.
- TCP still validates `0 < max_frame_bytes <= u32::MAX`, configures one-second loopback read/write timeouts, maps `UnexpectedEof` to truncation, rejects oversize frames before payload allocation, and makes close idempotent through shutdown + local closed state.

## Evidence boundaries / exclusions

- This is source/control-flow reuse, not a claim that the current anchor ran the historical adapter/observability tests.
- No transport semantics, Session delivery state, resource-policy number, D019 decision, or release/security policy is changed.
- Candidate B is not reopened because its semantic owner did not move.
- No decoder/parser/crypto-framing change is made; fuzz is not requested.
- No materially new real-network question is created. **`READY_LIVE: none`.**

## Classification

`R-OBS/ADAPTER-DIFF: CLOSED — bounded no-finding / prior dedicated reviews remain reusable for unchanged semantic owners.`

Continue immediately to `R-FS/SESSION-DIFF`; this is not repository-wide queue exhaustion and release items 3/4 remain incomplete.
