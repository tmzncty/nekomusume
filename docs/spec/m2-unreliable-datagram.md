# Authenticated unreliable datagram candidate

**Status: candidate/provisional semantic contract; not a wire or API freeze.**

This document defines the meaning of the existing bounded unreliable-record
experiment. It does not authorize a socket, runtime, WAN, or production
service. `seal_unreliable` and `open_unreliable` are authenticated record APIs:
they bind a bounded payload to the established Noise record context, direction,
nonce and replay state. They are not a second reliable Session delivery path.

## Reliable Session delivery is a different contract

Reliable Session delivery is the path whose application-visible result is
ordered, loss-recovered delivery (subject to the separate Session contract).
It may use the reliable carrier/recovery machinery and its delivery evidence is
accounted for independently. An unreliable datagram has only a bounded,
individually authenticated record outcome: accepted/opened, or absent/rejected.
A successfully opened record is not an application delivery receipt, and a
carrier observation is not a Session delivery receipt.

In particular, the current APIs **must not downgrade** an inner reliable
`ProcessMessage::Data` message. If such a message is carried by a reliable
Session path, it retains the reliable Session semantics. Calling an
unreliable-record API around bytes does not convert that inner message into an
unreliable message, nor does it authorize bypassing reliable ordering,
retransmission, flow control, or delivery accounting. A future datagram-shaped
Session message would require an explicit reviewed API and contract; it is not
implied here.

## Size and admission boundary

The 1200-byte value is the candidate **payload cap** for the unreliable API. It
is not the maximum size of an authenticated record envelope, UDP/IP packet, or
carrier write. Authentication tag, record header, nonce/sequence metadata and
any carrier framing consume additional envelope/carrier budget. Consequently,
implementations must distinguish:

* payload admission (`payload_len <= 1200`),
* authenticated-record envelope limits, and
* carrier/path packetization limits (including any configured PMTU target).

An envelope or carrier limit can reject a record even when its payload is at
most 1200 bytes; conversely, a 1200-byte payload cap must not be presented as a
measured path MTU. Oversize policy rejection remains atomic: it occurs before
generic open/replay mutation, so rejection does not consume a nonce or replay
window entry.

Admission is fail-closed and bounded. Malformed, tampered, replayed, envelope-
oversize and carrier-unadmittable records are rejected. A record lost before
admission, dropped at admission, or absent at the receiver is caller-visible
absence; no inferred receipt is created.

## Loss, acknowledgement and ordering

There is no retransmission, datagram-level recovery, delivery ACK, Session-ledger
promotion, or carrier-ACK promotion in this contract. A packet ACK or path
health event, if observed by another subsystem, does not become unreliable
application delivery evidence. There is no ordering guarantee: records can be
lost, duplicated at the carrier boundary, or observed out of send order; replay
protection may reject a duplicate without creating a delivery event. Callers
that need ordering or recovery must use the reliable Session contract.

## Flow, congestion and mixed traffic

This candidate supplies no independent flow-control or congestion-control
algorithm. It must not assume that authentication or a payload cap prevents
queue growth, receiver overload, or path congestion. Implementations and
experiments must keep bounded admission/queues and expose rejection/drop
outcomes rather than silently converting them to reliable delivery.

When reliable and unreliable traffic share a Session, carrier, or scheduler,
they remain semantically separate. Unreliable traffic must not consume the
resources reserved for reliable delivery, and reliable traffic must not be
silently discarded because an unreliable burst occupies the queue. Scheduling
must provide bounded admission and a fairness/starvation policy (or record
that the policy is not implemented); no claim of fairness, isolation, or
congestion safety is made by this document. A mixed-traffic experiment must
report its queue, drop and reliable-delivery outcomes separately.

Candidate counters for bounded research observability are: `offered`,
`admitted`, `opened`, `dropped`, `rejected_oversize`, `rejected_malformed`,
`rejected_auth`, `rejected_replay`, `carrier_unadmitted`, and
`queue_dropped`. They are diagnostic counters only. They do not establish
receipt, ordering, fairness, throughput, congestion response, or application
effect, and counter names/formats are not frozen.

## CLI and scope decision

Retain the existing bounded probe and its fixed limits as research tooling.
Do not add a user-level datagram CLI at this stage. No wire format, public API,
implementation, runtime, dependency, socket, WAN, or production behavior is
changed by this semantic contract.
