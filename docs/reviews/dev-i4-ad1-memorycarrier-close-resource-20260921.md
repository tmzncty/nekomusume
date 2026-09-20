# Developer bounded I4-AD1 review — MemoryCarrier close/error/resource semantics

**Anchor:** exact source/test tree `8cbd9af` + reviewer/dev commits through `051f588`.
**Owners inspected:** `neko-carrier` `MemoryCarrier` pair send/recv/close ordering,
idempotence, peer/owner release, post-close failures, false-success negatives.

## Per-invariant coverage

| Invariant | Coverage |
|---|---|
| send/recv/close ordering and idempotence | `close_is_idempotent_and_preserves_queued_data` proves close is a repeatable no-op, queued pre-close data remains drainable by the peer, and post-close `send` fails `Closed`/`PeerClosed` rather than succeeding |
| peer/owner release | closing one side yields `PeerClosed` on the peer's `send` and `None` on `recv` after drain — no dangling owner or false delivery |
| post-close failures / false-success negatives | `send` after close is `Closed`; `send` to a closed peer is `PeerClosed`; neither records delivery |
| bounded concurrency | `concurrent_bidirectional_send_completes_without_deadlock` proves bidirectional concurrent sends complete within bounded time with no stuck worker join |
| opaque/evidence-free contract | `contract_is_opaque_and_evidence_free` proves `CarrierKind::Other`, non-reliable ordered properties, and committed `CarrierLimits` — no hidden evidence channel |

## Reachable regressions named

- `close_is_idempotent_and_preserves_queued_data`
- `concurrent_bidirectional_send_completes_without_deadlock`
- `contract_is_opaque_and_evidence_free`
- `memory_roundtrip_fifo_and_empty` (record-framing boundary)

## Result

No concrete defect found across MemoryCarrier close/error/resource semantics.
Close is idempotent and does not drop queued data; post-close sends fail
`Closed`/`PeerClosed` without recording false delivery; concurrent
bidirectional sends complete bounded; the carrier contract is opaque and
evidence-free.

**READY_LIVE: none** — deterministic local evidence only.
