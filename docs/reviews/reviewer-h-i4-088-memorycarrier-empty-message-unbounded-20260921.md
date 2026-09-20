# H-I4-088 — MemoryCarrier empty-message queue bypasses the configured resource bound

**Severity:** HIGH — bounded resource/accounting correctness.

**Reachable review anchor:** exact `982ee6da427698ac73e7e199650873255ecef4bb`.

**Owner:** `crates/neko-carrier/src/lib.rs` — `MemoryPair`, `MemoryEndpoint::send/recv`, `MemoryState::{queues,queue_bytes}`, and `memory_pair_tests`.

## Concrete source-decided counterexample

`MemoryLimits` exposes `max_message_bytes` and `max_queue_bytes`; the `Carrier` projection reports the latter as `max_buffered_bytes`. `MemoryEndpoint::send` bounds a peer queue only by:

```text
queue_bytes[peer] + message.len() <= max_queue_bytes
```

and then unconditionally pushes `message.to_vec()` into the peer `VecDeque`.

Empty messages are an explicitly supported current behavior: `bidirectional_fifo_boundaries_and_input_copy` sends `&[]` and expects the peer to receive `Some(Vec::new())`.

For `message.len() == 0`, the byte-accounting value never changes. Therefore a live peer can execute:

```text
send([]) -> Ok
send([]) -> Ok
send([]) -> Ok
... without a queue-length bound
```

Each operation appends another `Vec`/`VecDeque` element while `queue_bytes[peer]` remains zero. The advertised/configured queue-byte bound therefore does **not** bound MemoryCarrier-owned queue memory or record count for a valid message shape. This contradicts the bounded MemoryPair/resource claim and the repository security requirement that connection/resource state be bounded.

The earlier developer no-finding note `docs/reviews/dev-i4-ad1-memorycarrier-close-resource-20260921.md` did not challenge zero-length record multiplicity; its close/order/concurrency observations may remain useful, but its overall resource no-finding conclusion is superseded until this defect is closed.

## Required bounded repair

Preserve the existing public empty-message behavior unless a separately approved contract change says otherwise. Do **not** invent a new public capacity/security policy value merely to close this finding.

A minimal source-local repair may add an internal queued-record ceiling derived from an already committed nonzero limit (for example, never permitting the number of queued records to exceed `max_queue_bytes`), while retaining `queue_bytes` as exact payload-byte accounting. An equivalent implementation is acceptable if it satisfies all of these invariants:

1. repeated zero-length sends cannot grow the peer queue without bound;
2. nonempty payloads still obey the existing `max_queue_bytes` byte cap exactly;
3. rejected empty/nonempty admission is atomic — no queue element or accounting mutation;
4. receiving an empty queued record releases whatever internal record-cap ownership was consumed, so subsequent admission can resume;
5. close/peer-close ordering and the existing pre-close queued-data drain semantics remain unchanged;
6. no Session-delivery/path evidence is introduced.

Add a focused mutation-sensitive regression using small existing limits: fill the derived record bound with empty messages, prove the next empty send returns `BufferFull`, receive one empty record, then prove one new empty send succeeds. Keep the existing empty-message FIFO test and byte-cap tests.

No decoder/parser/crypto framing changes are required, so fuzz is not mechanically required. After the repair, run focused deterministic tests and the final pushed exact-tree local gate (`PYTHONDONTWRITEBYTECODE=1 bash scripts/check.sh`, `git diff --check`, clean tree) and persist provenance.

## Evidence boundary

This finding is from exact-current source/test reasoning; no reviewer-local test execution is claimed here. It is local-only and creates no WAN/live question. It does not choose D019, TTL/LRU/history-size, a new public capacity value, or any release/production state.
