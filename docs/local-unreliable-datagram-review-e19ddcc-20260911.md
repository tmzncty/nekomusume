# Local unreliable-datagram bounded review — `e19ddcc`

Developer-run bounded source/test review and exact-tree validation of reachable commit `e19ddccde4a82426593a4fb020a256522c2cde07`.

## Result

Focused crypto and Session tests pass for the current T-row candidate boundary:

- authenticated unreliable record round-trip and replay rejection;
- exact 1200-byte payload cap and 1201-byte fail-closed rejection;
- envelope oversize rejection before replay-window mutation, proven by later acceptance of the original valid record;
- bounded queue admission, queue-full drop, oversize accounting, FIFO dequeue and terminal close;
- close clears queued datagrams without ACK, retransmission or inferred receipt;
- reliable `ProcessMessage::Data`/`DeliveryAck` remains a distinct codec/evidence path.

The bounded inspection found no concrete contradiction with the provisional semantic contract. `seal_unreliable`/`open_unreliable` authenticate payload, context, direction, nonce and replay state, but do not create Session delivery/effect evidence. `DatagramRuntime` supplies bounded admission/drop counters only and has no ACK, retransmission, ordering guarantee or flow/congestion claim. No user-level datagram CLI is added.

Remaining limitations are explicit: shared-carrier fairness/isolation is not claimed by this row; the 1200-byte payload cap is not a measured PMTU or carrier envelope limit; no WAN, congestion, throughput, reliability, public listener, release or production evidence follows.

## Verification

Focused tests:

- authenticated unreliable roundtrip/replay/size test: passed
- oversize replay-atomicity test: passed
- Session datagram runtime tests: passed (`2`)
- reliable process-boundary separation tests: passed (`2`)
- Session/Crypto clippy with warnings denied: passed
- `git diff --check`: passed

Exact-tree gate:

- `PYTHONDONTWRITEBYTECODE=1 bash scripts/check.sh`: exit `0`
- `git diff --check`: exit `0`
- gate: `2026-09-10T18:30:36Z` -> `2026-09-10T18:32:33Z`
- host: `Linux 6.8.0-137-generic x86_64`
- Rust: `rustc 1.98.0 (88d9e12ae 2026-08-18)`
- initial/final source tree: clean

This closes only the bounded local T-row question, not the wider experimental/release gates.
