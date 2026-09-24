# R-REC-4a — packet-threshold loss + Reno aggregate accounting review

**Reviewed exact repository anchor:** `80542a94f5bb54de6754bc15f8067eea722576ad`

**Result:** no concrete correctness defect found in the inspected source path, but the dedicated owner-spanning positive-loss regression requested by item-4 review support is still missing. This remains a dependency-ready **READY_LOCAL test/evidence slice**, not queue exhaustion and not a policy gate.

## Developer movement reviewed

Since reviewer handoff `4aea0a9f6bd7c8733f02b6cb8a1e26c8e43e0f22`, the current branch contains:

- `312d6bdaa7960042c86eb796c6cb6e76a013b2ed` — H-I4-117 runtime persistent-congestion wiring plus executable threshold tests;
- `39050ae8fe6a20f6c6f58eda7913ee8c68e627cf` — formatting-only follow-up;
- `2ba5b960ebb13f1bfe6c5e4b07438385f5aab673` — orchestration-test adjustment for post-collapse congestion refusal;
- `80542a94f5bb54de6754bc15f8067eea722576ad` — handoff/provenance reconciliation.

H-I4-117 closure is accepted at exact source/test anchor `2ba5b960ebb13f1bfe6c5e4b07438385f5aab673`. Reachable developer-local provenance is `docs/notes/h-i4-117-provenance-2ba5b96-20260924.md`. GitHub combined-status for exact `2ba5b96` had no status entries at review time; do not relabel the developer-local gate as hosted CI.

## Inspected owners

- `crates/neko-reliable/src/lib.rs`
  - `Recovery::on_ack`
  - `Reno::{acked,lost,can_send}`
  - packet-threshold loss regression(s)
- `crates/neko-carrier/src/lib.rs`
  - `PathRecovery::on_sent`
  - `PathRecovery::on_ack`
  - `ReliableUdpRuntime::apply_ack`
  - existing PathRecovery/runtime ACK/loss tests

Applicable boundaries were re-read from `README.md`, `AGENTS.md`, `ROADMAP.md`, `IMPLEMENTATION_PLAN.md`, `SECURITY.md`, `docs/status.md`, `docs/standing-vps-lab-authorization.md`, `docs/vps-rental-window-priority.md`, `docs/decisions.md`, `docs/carrier-architecture.md`, `docs/specs/nekomusume-session-v0.md`, `docs/release-security-review-packet.md`, and `docs/CHATGPT_HANDOFF.md` at the reviewed anchor.

## Challenged invariant

For one authenticated ACK that both positively ACKs packet(s) and causes packet-threshold loss of older packet(s):

1. `Recovery::on_ack` must return disjoint complete `acked_packets` / `lost_packets` sets and stable retransmit frame ownership;
2. `PathRecovery::on_ack` must release each packet's actually charged Reno bytes exactly once;
3. all bytes released as lost in that ACK are aggregated into one `Reno::lost(...)` call, so one ACK/loss transition causes one multiplicative decrease rather than one decrease per lost packet;
4. `bytes_in_flight` after the call must equal the charges still owned by packets left outstanding;
5. a positive loss must tighten admission, while the H-I4-116 zero-loss case remains a no-op for `Reno::lost(0)`.

## Source reasoning

The current implementation satisfies the ownership shape above:

- `Recovery::on_ack` first removes ACKed packets, then computes loss only over the remaining `sent` map. A packet therefore cannot be both ACKed and lost in the same transition.
- `PathRecovery::on_ack` removes the per-packet `charged` entry for each ACKed and lost packet, separately aggregates `released_acked` and `released_lost`, then invokes `reno.acked(released_acked)` once and `reno.lost(released_lost)` once.
- `Reno::lost(0)` is the H-I4-116 no-op; a positive aggregate invokes the window reduction exactly once regardless of how many packets contributed to the aggregate.
- Existing recovery tests prove packet-threshold loss/retransmit selection, and existing Reno tests prove a positive `lost(bytes)` reduces the window. Existing PathRecovery/runtime tests exercise real loss paths for health/retransmit accounting.

No current source path was found that double-removes one packet's charge or invokes Reno loss once per individual lost packet.

## Remaining evidence gap / READY_LOCAL closure

The current tree still lacks one focused deterministic owner-spanning regression that makes the aggregate accounting executable in one place. The coding agent should add a small `PathRecovery` or `ReliableUdpRuntime` test that:

- sends enough equal-sized ack-eliciting packets for a single ACK of the largest packet to declare **multiple** older packets lost by the existing packet threshold;
- asserts the exact `acked_packets`, `lost_packets`, `acked_bytes`/`lost_bytes` (where appropriate), retransmit-frame set, and post-ACK `bytes_in_flight`;
- proves admission tightens after the positive loss without inventing any new congestion-control values;
- includes a positive control that only one aggregate Reno reduction occurs for the one ACK transition (prefer public `can_send`/existing state access already available in the test module; do not add a production-only debug API);
- preserves the H-I4-116 loss-free control and all packet/frame/session evidence separation.

This is test/evidence closure, not a request to redesign Reno or change `PACKET_THRESHOLD`, MSS, ACK semantics, D019, wire/crypto, capacity/security policy, or release flags. No decoder/parser/crypto framing change is involved, so fuzz is not mechanically required.

## Evidence boundary

This review is exact-current GitHub source/control-flow inspection only. No reviewer-local Rust/full-gate, cross-platform execution, fuzz, WAN, benchmark, or performance execution is claimed. Hosted status and developer-local provenance remain separate evidence classes.
