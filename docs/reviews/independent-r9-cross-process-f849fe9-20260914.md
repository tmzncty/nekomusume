# Independent bounded R9 cross-process reliable-UDP review — exact `f849fe9`

**Reviewed revision:** `f849fe9c3f82fb3e560bfe4c79f36afc725e8e23` (`main` at review time).

**Sequence reviewed:** `aa483694b83e554fd802bee1250c32162e20106e` (first cross-process reliable-UDP seam), developer provenance `ec0319362753c67b3957753bf919c4cc50bc0f94`, `f0178e3d460694508b1da9924c17dfe45ea8a221` (second reliable record), and style-only `f849fe9`.

**Scope:** `crates/neko-cli/src/main.rs` failover server/client post-auth UDP path, `recv_udp_delivery_ack`, `crates/neko-cli/tests/probe.rs` R9 process regression, current R9 handoff contract, and current hosted checks. This is a bounded correctness/evidence review, not a security audit, WAN result, RC decision, protocol freeze, or release approval.

## Current evidence

- `aa48369` has a recorded developer-local clean exact-tree gate in `docs/local-gate-aa48369-20260914.md` (`scripts/check.sh`, `git diff --check`, clean initial/final tree).
- Current exact `f849fe9` hosted `stable checks` and `nightly decode fuzz smoke` are green. Hosted checks remain cross-evidence, not developer-local exact-tree provenance for `f0178e3`/`f849fe9`.
- No open PR was present at review time.

## Verdict

R9 has genuinely started and the existing failover process seam is the correct integration owner. The `aa48369` source is useful forward progress and should not be reverted. However **R9-2 is not closed at `f849fe9`**. Three correctness/ownership HIGH findings and one evidence MEDIUM must be repaired before R9-3 fault injection or any `READY_LIVE` classification.

## H-R9-001 — multi-record Session confirmation is swallowed by the R9 receive demux

**Severity:** HIGH.

`recv_udp_delivery_ack` has one `expected: &OutboundRecord`. It returns only when the authenticated plaintext is a Session `DeliveryAck` matching that one record. When `rt` is present, every other authenticated plaintext is treated as a possible Carrier ACK; if it is not a canonical `RecordType::Ack`, the function still `continue`s without classifying it as malformed or as another Session acknowledgement.

After `f0178e3`, record 2 is sent through `ReliableUdpRuntime`, so the server emits both a Carrier packet ACK and a Session `DeliveryAck` for record 2. If record-2 Session acknowledgement arrives before the expected record-1 Session acknowledgement, it is consumed and discarded by this demux. After the helper returns, the client calls `delivery.delivery_ack(...)` only for record 1. The later settlement loop decodes only Carrier ACKs and does not apply Session `DeliveryAck`s. Therefore current code cannot demonstrate independent logical confirmation for both reliable records.

The same branch also discards `ReliableUdpRuntime::apply_ack` errors (`let _ = ...`) and, in R9 mode, authenticated non-ACK garbage no longer increments the existing malformed bound.

### Required repair

Replace the single-record R9 receive behavior with one bounded authenticated demux that distinguishes:

1. canonical Carrier `RecordType::Ack` -> decode -> `apply_ack`; rejection is typed/observable and must not mutate recovery;
2. Session `ProcessMessage::DeliveryAck` -> validate against a bounded set/map of outstanding logical `(stream, offset, len)` records and apply `SessionRuntime::delivery_ack` exactly once;
3. unexpected authenticated Session/control/malformed plaintext -> bounded rejection/diagnostic rather than silent success-path consumption.

Do not merge Carrier ACK and Session DeliveryAck semantics.

## H-R9-002 — record 2 has two concurrent UDP owners in `--reliable-udp` mode

**Severity:** HIGH.

`f0178e3` sends `records[1]` through `ReliableUdpRuntime` with cwnd admission, `on_packet_sent`, packet number ownership and a real UDP send. Later the unchanged legacy failover path again takes `records.get(1)`, seals it with a fresh AEAD packet number and sends it directly as `udp_uncertain_range_sent`, outside `ReliableUdpRuntime`.

This means the clean R9-2 path has one logical Session range carried once by the reliable recovery owner and again by an untracked direct UDP send before a fault requires replay. It makes recovery ownership non-authoritative, can create Session duplicates in the nominal no-loss case, and would confound later Data-loss/PTO evidence.

### Required repair

In `--reliable-udp` mode, every logical range selected for the R9 reliable subset must have exactly one Carrier recovery owner. For the first bounded R9-2 slice, route at least two records through `ReliableUdpRuntime` and exclude those same records from the legacy untracked `udp_uncertain_range_sent` path. Preserve historical behavior when the flag is absent. Remaining non-R9 records may continue through the existing failover ownership until later R9 slices migrate them deliberately.

## H-R9-003 — receiver records packet ACK obligation before valid R9 Data classification

**Severity:** HIGH.

On the server, after `SecureSession::open_unreliable` succeeds, current code immediately derives the outer AEAD sequence and calls `server_rt.on_packet_received(pn, true)` before `ProcessMessage::decode`, before proving the message is `Data`, and before checking the expected Session identity.

The packet ACK is polled/sent only inside the later valid-Data branch, but the ACK tracker has already retained any earlier authenticated non-Data/control/malformed packet. A subsequent valid Data packet can therefore cause the outgoing ACK range to include a packet that never entered the R9 reliable-Data path. This broadens ACK obligation and risks ACK-of-ACK/control feedback when the process path grows.

### Required repair

Commit the receiver packet number to R9 recovery only after authentication **and** structurally valid `ProcessMessage::Data` classification with the expected Session identity. Session duplicate/conflict semantics remain Session-owned. Do not make arbitrary authenticated control/ACK frames ack-eliciting R9 Data packets unless a later explicit wire/architecture decision defines such a packet space.

## M-R9-004 — the current process regression does not prove the new multi-record claim

**Severity:** MEDIUM evidence-integrity.

The existing `reliable_udp_failover_settles_packet_acks_to_zero_in_flight` regression still checks only:

- process success;
- presence of `r9_udp_in_flight_settled`;
- `remaining_in_flight=0`;
- at least one server packet-ACK event;
- at least one server Session-DeliveryAck event.

`f0178e3` changes source only; it adds no regression asserting two unique Session logical records, two independent logical confirmations, zero no-loss duplicate/conflict, or that both packet ACKs were applied. Therefore the commit-message statement that multi-record once-delivery is proved is stronger than current durable evidence.

### Required regression

On the repaired no-loss R9-2 process path assert, at minimum:

- at least two distinct byte offsets are carried through `ReliableUdpRuntime`;
- two distinct logical records are first-delivered exactly once by Session;
- both corresponding Session `DeliveryAck`s are independently validated/applied;
- Carrier packet ACKs retire both packet owners and final authoritative `in_flight()==0`;
- zero PTO/retransmit in the no-loss case;
- zero Session conflict and preferably zero duplicate in this clean baseline;
- listener/process/identity cleanup remains valid.

## Queue consequence

Do **not** advance to R9-3 Data-loss recovery until H-R9-001..003 and M-R9-004 are repaired and the repaired R9-2 exact pushed tree is gated. After that, continue immediately through the already pre-authorized R9-3..R9-12 queue; do not wait for reviewer cadence.

The next coherent gate after repairs should be run on the final pushed source/test SHA in a clean checkout/worktree:

```bash
PYTHONDONTWRITEBYTECODE=1 bash scripts/check.sh
git diff --check
```

Record exact SHA, UTC start/end, OS/arch, rustc, exit codes, and clean initial/final tree. No wire/parser grammar change is requested by this review, so no extra fuzz requirement is introduced beyond existing policy.

`READY_LIVE` remains none until cross-process R9 recovery/fault/promotion work and independent R9 closure actually land.