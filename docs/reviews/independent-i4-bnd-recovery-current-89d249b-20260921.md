# Independent bounded I4-BND Recovery challenge — current tree

**Review anchor:** exact reachable `89d249b9f2b64a6cac5be9639743a86348dcb8bd`.
**Current executable/source-test tree:** exact `42be53917ee58359ad86532a6aab0136f75047b9`; commits after it through the review anchor are documentation-only.

## Scope and invariant challenged

This closes the `neko-reliable` portion that the developer I4-BND note did not explicitly inspect. Owners read on the exact-current tree:

- `crates/neko-reliable/src/lib.rs` (`AckRanges`, `Recovery`, RTT/PTO, Reno);
- the reachable ACK construction/application seam in `crates/neko-carrier/src/lib.rs`;
- the executable ACK decode/application seam in `crates/neko-cli/src/main.rs`;
- `crates/neko-wire/src/lib.rs` only to establish the existing peer-controlled ACK-range input bound.

Challenge: peer-controlled packet numbers/ranges and retained recovery ownership must not create iteration or retained-state growth beyond already committed limits. This review does **not** choose new capacity values and does not run an adversarial/capacity benchmark.

## Result — no concrete defect found

- `Recovery::sent` is admission-bounded by `max_sent_packets`; constructor validation caps that configured value at `HARD_MAX_SENT_PACKETS`. `watermark_on_reserve` is created/removed with packet ownership and is cleared by `quiesce`.
- Per-packet frame count is admission-bounded by `max_frames_per_packet` with `HARD_MAX_FRAMES_PER_PACKET`; `outstanding_frames` and `acked_frames` are derived from copies owned by bounded `sent` packets and are retired/pruned with those copies. `quiesce` clears the live maps/sets rather than retaining packet/frame ownership indefinitely.
- `AckRanges` retains at most its explicit `max_ranges` (itself hard-capped). Range membership walks the retained range vector; `Recovery::on_ack` walks live `sent` ownership, not every packet number represented by a numeric interval. A huge inclusive range therefore does not expand into one element per packet number.
- The executable peer-input seam is tighter than the generic hard cap: `neko-wire::MAX_ACK_RANGES == 32`, `decode_ack` rejects a larger count before allocating the decoded range vector, and executable callers convert that bounded vector with `AckRanges::from_ranges(32, ...)`. Thus `from_ranges`' caller-slice clone is not a current unbounded peer-controlled path.
- `Recovery::on_ack` rejects no-sent/future-largest ACKs before RTT/loss/PTO or packet-state mutation, so malformed future numbers cannot force loss/retransmit work over fabricated state.
- RTT/PTO/Reno state is scalar; loss/retransmit scans are over bounded live ownership. `quiesce` preserves only scalar lifetime history while releasing live packet/frame/high-water ownership.

## Evidence boundary

This was a reviewer source-level bounded challenge; **no reviewer-local command execution is claimed**. The current executable/source-test tree `42be539` has separate developer-reported clean exact-tree provenance in `docs/notes/i4-port-01-provenance-42be539-20260921.md` (`scripts/check.sh` exit 0, `git diff --check` exit 0, clean tree). That provenance is not reclassified as reviewer execution or hosted CI.

No decoder/parser/framing code changed in this review, so no fuzz run is triggered. No WAN/live or performance conclusion follows. `SessionRuntime.events` retained-history capacity and D019 remain maintainer/security policy gates; no value is invented here.
