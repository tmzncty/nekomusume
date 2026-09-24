# H-I4-123 — reliable-UDP + DeliveryAck-suppression replay-count mismatch

**Severity:** HIGH — deterministic process-fixture correctness / release-item-4 evidence reliability.

**Reviewed source anchor:** reachable `d3c939ba984301a94e4db2adde63363524aa1ef9`.

**Review class:** exact-current GitHub source/control-flow review only. No reviewer-local Rust/full-gate, hosted CI, WAN, fuzz, cross-platform, or performance execution is claimed.

## Finding

The H-I4-122 wording repair is correct for its intended non-reliable process regression, but the newly precise `--drop-post-auth-delivery-acks` seam remains combinable with the existing `--reliable-udp` mode and exposes a deterministic ownership/count mismatch.

On the client, reliable-UDP owns the first two logical records: both are inserted into the Session-DeliveryAck `outstanding` set. Carrier packet ACKs may settle reliable packet ownership, but the suppression seam withholds Session DeliveryAck messages, so the application deadline path moves **both** logical records into `unacked_retained_records`. Those two records are then inserted into `FailoverController` uncertain ownership. For `count=3`, the normal uncertain partition additionally contributes record 2. After manager promotion, `FailoverController::tcp_resend()` is authoritative and therefore returns all three records; the client sends all three over TCP and waits for a DeliveryAck after each.

The server computes its expected TCP Data count independently. Under reliable-UDP it sets `uncertain_start=2`, so the ordinary uncertain width is one record for `count=3`. The DeliveryAck-suppression adjustment then adds a hard-coded **one** replay record. The server therefore expects only two TCP Data frames even though the client authoritative replay set contains three.

No current argument validation rejects the combination, and the current H-I4-122 process regression does not exercise `--reliable-udp`. A combined focused regression is absent.

## Consequence

For the valid combination `--reliable-udp` + automatic cold failover + `--drop-post-auth-delivery-acks` with at least two initially reliable-owned records, client/server disagree about replay cardinality. The server can finish its expected replay loop after fewer records than the client sends, leaving the client waiting for an ACK for a replay the server did not account for.

This is not a new ACK/PTO policy decision and does not touch H-I4-119. It is an existing ownership-partition mismatch at the process fixture seam.

## Smallest authorized closure

1. Add a focused deterministic process regression combining reliable-UDP, DeliveryAck suppression, automatic cold health failover, and a count that includes both reliable-owned records plus at least one ordinary uncertain record.
2. Derive the server's replay expectation from the same committed logical ownership partition as the client. Under DeliveryAck suppression, every initially reliable-owned record whose Session DeliveryAck was withheld must be included in retained replay ownership; do not assume exactly one retained record.
3. Preserve the existing non-reliable H-I4-122 regression and ordinary reliable-UDP behavior.
4. Assert exact replay cardinality/identity and complete Session delivery without duplicate application delivery.
5. Do not change H-I4-119/core ACK-PTO semantics, D019, policy values, wire/crypto architecture, release flags, or standing authorization.
6. Run the final pushed repaired source SHA through the normal clean exact-tree local gate and persist reachable provenance. No decoder/parser/crypto-framing change is implied; no mechanical fuzz run.

## Queue consequence

H-I4-123 becomes FRONT. H-I4-122's evidence-narrowing source repair remains accepted as a partial closure, but its final evidence closure should use the successor repaired tree/provenance after H-I4-123. Then continue immediately with R-MBOX-REORDER-DELAY and the existing deep queue. `READY_LIVE: none` is unchanged.
