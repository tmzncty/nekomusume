# Reviewer finding — H-I4-118 stale ACK can time-threshold newer packets

**Date:** 2026-09-24  
**Exact source anchor reviewed:** `8f48444feb9a8e4fac01c6333ab39873ad46c452`  
**Owner:** `crates/neko-reliable/src/lib.rs` — `Recovery::on_ack` loss eligibility, with downstream `PathRecovery`/Reno accounting  
**Severity:** HIGH correctness / release-item-4 evidence reliability  
**Classification:** independent source/control-flow challenge; no production change in this reviewer commit

## Challenged invariant

An ACK may drive packet- or time-threshold loss only for an outstanding packet that was sent **before** the acknowledged packet-number frontier. A delayed/duplicate ACK for an older, already-retired packet must not fabricate loss/retransmit evidence for newer in-flight packets merely because those newer packets have aged beyond `loss_delay`.

This is also the standard loss-detection ordering rule that the current constants and comments track: RFC 9002 §6.1 requires a lost packet to have been sent prior to an acknowledged packet, and its reference pseudocode skips unacked packet numbers greater than `largest_acked_packet`. See https://www.rfc-editor.org/rfc/rfc9002.html#section-6.1 .

## Exact-current defect

`Recovery::on_ack` first rejects only `ack.largest() > largest_sent`, which correctly closes the never-sent future-ACK case. It then removes packets explicitly covered by the ACK and computes:

```rust
let lost: Vec<u64> = self
    .sent
    .iter()
    .filter_map(|(&n, p)| {
        ((largest.saturating_sub(n) >= PACKET_THRESHOLD)
            || (delay > 0 && now_us.saturating_sub(p.sent_at_us) >= delay))
            .then_some(n)
    })
    .collect();
```

The packet-threshold arm is incidentally protected for `n > largest` by `saturating_sub` yielding zero, but the **time-threshold arm has no `n <= largest` / sent-before-acknowledged-frontier guard**. Therefore a stale ACK with `largest = L` can declare outstanding packets `n > L` lost solely because wall-clock age exceeds `loss_delay`.

A concrete deterministic shape on the current owner:

1. establish a non-zero RTT/loss delay;
2. send/ACK packet 0 so packet 0 retires and `largest_sent` remains a committed high-water;
3. send newer packets 1 and 2 and leave them outstanding;
4. advance `now_us` beyond the loss delay;
5. deliver a duplicate stale ACK for packet 0.

The ACK is legal under the current fail-closed rule (`0 <= largest_sent`) and need not reference a packet still present in `sent`. It newly ACKs nothing. Nevertheless the current time-threshold predicate selects packets 1 and 2, removes them from `sent`, emits `lost_packets`/`lost_bytes`/`retransmit_frames`, and downstream recovery can treat the fabricated loss as congestion evidence.

This is distinct from Candidate A / the future-unsent-ACK repair: the peer ACK number here was genuinely sent and previously acknowledged. It is also distinct from R-REC-2's frame-copy ownership challenge. R-REC-2 remains valid for `acked_frames`/`outstanding_frames` copy-lifetime and retransmit-eligibility ownership, but its no-finding must not be read as coverage of packet-number ordering for time-threshold loss.

## Required closure

Use the smallest current-semantics repair; do not redesign ACK architecture or require every acknowledged packet number to remain in `sent`.

1. Gate loss candidates so packet- and time-threshold loss apply only to packets at/before the acknowledged frontier (in practice outstanding candidates are strictly older than `largest`, because an outstanding packet explicitly ACKed as `largest` is removed first). A narrow `n <= largest`/equivalent guard around loss eligibility is expected if exact-current owners remain unchanged.
2. Add a focused deterministic regression proving a stale duplicate ACK for an already-retired lower packet number:
   - is accepted under the current sent-high-water rule;
   - newly ACKs no packet;
   - does not mark newer packet numbers lost;
   - schedules no retransmit work;
   - does not change RTT/PTO state merely because the ACK is stale;
   - leaves the newer packets in flight.
3. Add/retain a positive time-threshold control proving an actually later acknowledged packet can still cause an older outstanding packet to be declared lost after the delay. Preserve existing packet-threshold behavior.
4. If `PathRecovery` is used for an owner-spanning regression, prove the stale ACK does not release false lost bytes or trigger Reno reduction, while a legitimate positive-loss control still does. Do not introduce new congestion-policy values.
5. Preserve H-I4-116/117 behavior, frame-copy ownership, D019, Session-delivery separation, and current wire/crypto semantics.
6. On the final pushed source/test SHA run the developer-local clean exact-tree gate:
   - `PYTHONDONTWRITEBYTECODE=1 bash scripts/check.sh`
   - `git diff --check`
   - verify clean tree.
   Persist reachable provenance with exact SHA, UTC start/end, exit codes, OS/arch, stable Rust version, and clean-tree state.
7. No decoder/parser/crypto-framing change is expected; do not mechanically run fuzz.

## Evidence boundary

This finding is based on exact-current GitHub source/control-flow review at `8f48444f...` plus standards research used only to challenge the ordering invariant. No reviewer-local Rust/full-gate execution, hosted CI, WAN experiment, cross-platform execution, fuzz result, or performance conclusion is claimed here. `READY_LIVE` remains `none`; this is a local correctness repair.
