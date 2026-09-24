# H-I4-119 — non-ack-eliciting packet ACK resets PTO streak for still-unresolved ack-eliciting traffic

**Severity:** HIGH correctness / item-4 recovery-evidence reliability within the reusable `Recovery` / `PathRecovery` seam.

**Exact source/control-flow anchor:** `5d9d16a238d1f368f61c163a6d3d439f56e4f31a`.

**Review class:** independent GitHub source/control-flow challenge. No reviewer-local Rust execution, hosted CI, WAN, fuzz, or performance claim is made here.

## Owners inspected

- `crates/neko-reliable/src/lib.rs::Recovery::{on_sent,on_ack,on_pto}`;
- `crates/neko-reliable/src/lib.rs::SentPacket::ack_eliciting`;
- `crates/neko-carrier/src/lib.rs::PathRecovery::{on_sent,on_ack,on_pto,fresh_health_sample,persistent_congestion}`;
- `ReliableUdpRuntime::{on_packet_sent,on_retransmit_sent,pto_probe}`;
- exact-current non-ack-eliciting accounting tests in the PathRecovery test module;
- R-REC-1/2/3/4a/4b and H-I4-116/117/118 boundaries.

## Concrete defect

`Recovery::on_ack` currently resets the PTO streak with:

```text
if !out.acked_packets.is_empty() {
    self.pto_count = 0;
}
```

but it does not retain whether any **newly acknowledged packet was ack-eliciting** before removing those packets from `sent`.

That contradicts the exact-current meaning already attached to `SentPacket::ack_eliciting` by the higher owner:

- `PathRecovery::on_sent` charges Reno bytes only for `ack_eliciting=true` packets;
- its comments/tests explicitly state that a non-ack-eliciting packet is uncharged because it cannot elicit an ACK by itself;
- nevertheless such a packet is a legal committed `SentPacket` and can later be acknowledged as part of an ACK range.

Therefore this deterministic sequence is currently possible in the public recovery seam:

1. send ack-eliciting packet 0 and leave it unresolved;
2. send non-ack-eliciting packet 1 (zero Reno charge);
3. fire one or more PTOs while packet 0 remains unresolved;
4. receive a legal ACK that newly retires only packet 1;
5. `Recovery::on_ack` observes a nonempty `acked_packets` vector and resets `pto_count` to zero even though the outstanding ack-eliciting packet made no acknowledged progress.

That can postpone or suppress the already committed persistent-congestion threshold and changes the health bridge's `pto` evidence even though the ACK did not newly acknowledge ack-eliciting traffic. It is distinct from H-I4-116 (zero-loss Reno), H-I4-117 (threshold effect wiring), and H-I4-118 (stale-ACK loss eligibility).

## Existing-test gap

Exact-current tests do exercise non-ack-eliciting packets, but only assert that acknowledging them does not drain bytes charged to a different packet. They do **not** challenge PTO reset. Existing PTO reset tests use ordinary ack-eliciting packets, so they do not distinguish these cases.

## Required narrow repair

Current committed semantics already decide the result; no architecture/policy choice is needed.

1. In `Recovery::on_ack`, determine whether at least one **newly retired ACKed packet** is `ack_eliciting=true` before removing it from `sent`.
2. Reset `pto_count` only for that positive ack-eliciting acknowledgement. Do not require the peer ACK number to remain present after retirement and do not change the H-I4-118 `largest <= largest_sent` / `n <= largest` rules.
3. Add a focused negative regression: unresolved ack-eliciting packet + newly ACKed non-ack-eliciting packet after PTO(s) must leave the PTO streak unchanged and leave the real ack-eliciting packet in flight.
4. Add/retain a positive control that newly ACKing an ack-eliciting packet resets the PTO streak.
5. Preserve Reno charged-byte accounting, RTT sampling, loss/retransmit eligibility, persistent-congestion threshold/value, frame ownership, and Session/Carrier evidence separation.

An equivalent minimal guard is acceptable if it proves the same semantic boundary for all committed callers. Do not redesign ACK architecture or add a new congestion/policy number.

## Important runtime boundary

Current `ReliableUdpRuntime::on_packet_sent` and `on_retransmit_sent` construct `SentPacket { ack_eliciting: true, ... }`, so this finding does **not** claim that the current canonical runtime data-send path already emits a false reset from its own ACK-only records. The defect is in the reusable implemented `Recovery`/`PathRecovery` semantic surface, which explicitly accepts and tests non-ack-eliciting sent packets and is part of release item-4 review scope.

## Validation / evidence contract

After the repair, the developer should run the focused deterministic tests and then on the final pushed source SHA:

- `PYTHONDONTWRITEBYTECODE=1 bash scripts/check.sh`
- `git diff --check`
- clean-tree verification

Persist exact reachable provenance with SHA, UTC start/end, exit codes, OS/arch and stable Rust version. This is not a decoder/parser/crypto-framing change; do not mechanically run fuzz.

`READY_LIVE: none` — this is deterministic recovery state-machine work and introduces no new real-network question.
