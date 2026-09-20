# Independent bounded I4-AD1 MemoryCarrier re-challenge — exact `d0f4c55`

**Reachable owner anchor:** `d0f4c5519434aa9fb963de96dc4177e18d42d857` (`main` at review start).

**Scope:** exact-current `MemoryPair` / `MemoryEndpoint` close, error, queue accounting, record/resource boundedness, concurrency and Rust ownership/drop semantics in `crates/neko-carrier/src/lib.rs`, against D011 in `docs/decisions.md`, `SECURITY.md`, the current `Carrier` contract, developer note `docs/reviews/dev-i4-ad1-memorycarrier-close-resource-20260921.md`, H-I4-088 repair `26aa4e8036d61da7924d9fc5e587081d8a0f448f`, and its developer-local provenance note.

This is a bounded independent source/test challenge. It is not WAN evidence, not a security audit/release approval, and not reviewer-local CI provenance.

## H-I4-088 closure check

Accepted for this bounded surface. Exact `26aa4e8` adds a record-count admission guard before queue mutation while retaining the existing checked payload-byte accounting. Empty messages therefore consume one bounded record slot even though they consume zero payload bytes; `recv()` removes the `VecDeque` entry, so a received empty record releases that slot. The dedicated regression fills the derived record ceiling, proves the next empty send fails `BufferFull`, receives one empty record, and proves one further empty send succeeds.

The exact-tree gate recorded in `docs/notes/h-i4-088-provenance-26aa4e8-20260921.md` is developer-reported local evidence only: `scripts/check.sh` exit 0, `git diff --check` exit 0, clean worktree, Linux x86_64, rustc 1.98.0. GitHub exposes no hosted status/workflow run for that SHA; absence of hosted evidence is not interpreted as failure.

## Challenged invariants

- **Admission is fail-closed and pre-mutation — holds.** `send()` rejects oversize messages before locking; under the single pair mutex it then rejects local close, peer close, checked-add overflow, payload-byte capacity, and record-count capacity before `push_back` or `queue_bytes` mutation.
- **Byte and record accounting remain coherent — holds.** Non-empty payload bytes are tracked exactly in `queue_bytes`; record ownership is independently finite through the H-I4-088 guard. `recv()` removes one queue record and checked-subtracts exactly that record's payload bytes. Empty-record dequeue leaves byte accounting unchanged while releasing the record slot.
- **Pre-close queued-data drain does not admit post-close new data — holds.** Closing side A only flips A's shared logical closed flag and is idempotent. Data A queued to B before close remains drainable by B. After A closes, A cannot send (`Closed`) and B cannot enqueue any new record toward A (`PeerClosed`). A `recv()` after its own close can only observe records that were already in A's queue before the close; this matches D011's explicit queued-data-drain candidate semantics and is not a new post-close enqueue/delivery success.
- **Peer/local error precedence is deterministic — holds.** Local `Closed` is checked before peer `PeerClosed`; both precede capacity mutation. No close/error path creates SessionDelivery, PathValidated, ACK or other evidence-domain promotion.
- **Concurrency is serialized without a second ownership channel — holds.** Both directions share one `Mutex<MemoryState>`; the existing bidirectional concurrency regression exercises simultaneous sends. There is no worker/background queue consumer whose lifetime can outlive the pair state.
- **Mutex poison fails closed — holds by source inspection.** Every stateful operation acquires the same mutex through `lock()` and maps poison to `StatePoisoned` before further state mutation in that operation.
- **Rust `Clone` / `Drop` semantics are bounded and distinct from logical `close()` — no defect found.** Clones intentionally share the same logical side/closed bit through `Arc<MemoryShared>`. Dropping an endpoint handle does not synthesize `close()` and the current D011/Carrier contract does not claim RAII-close semantics. If the peer remains while the other side's handles are dropped without logical close, any future peer sends are still bounded by the payload-byte and record-count caps rather than producing unbounded retained state. This distinction should not be promoted into a socket/process-lifetime claim.

## Focused tests/evidence inspected

Current reachable tests inspected include `empty_messages_are_bounded_by_record_count`, `limits_and_queue_full_are_atomic`, `close_is_idempotent_and_preserves_queued_data`, `concurrent_bidirectional_send_completes_without_deadlock`, `bidirectional_fifo_boundaries_and_input_copy`, and `contract_is_opaque_and_evidence_free`.

Reviewer-local command execution was not available in this review environment, so no reviewer-local test/gate claim is made. The accepted executable evidence for exact `26aa4e8` remains the separately labelled developer-local clean exact-tree provenance above.

## Result

**No additional concrete source-decided MemoryCarrier defect found after H-I4-088. I4-AD1 is CLOSED as a bounded independent current-owner challenge at exact `d0f4c55`.**

Exclusions: UDP/TCP adapters, SessionRuntime semantics, real socket/process cleanup, production memory sizing, D019/history-cap policy, WAN evidence, release/security/production authority. `READY_LIVE: none`; release items 3 and 4 remain incomplete.
