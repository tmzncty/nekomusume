# ADR: M2 UDP packetization PMTUD semantics

**Status:** Decision for implementation planning; no socket or wire implementation in this change.
**Date:** 2026-08-30
**Scope:** Nekomusume UDP Carrier paths, including the authorized Linux VPS as an evidence target.
**Parent:** `7d259af1829a45ace7bffe551834e49ced37193a`

## Decision

Nekomusume will use packetization-layer PMTUD (DPLPMTUD, RFC 8899) as the
portable protocol behavior, with Linux kernel PMTUD/EMSGSIZE and validated ICMP
as useful input rather than as the sole source of truth. Every concrete path
has three distinct limits:

1. **Interface MTU**: the local link/device MTU, an operating-system fact. It
   is an upper bound on a locally emitted IP packet, not proof of end-to-end
   delivery. The recorded VPS has `eth0=1500` and an IPv6 route MTU of `1464`
   (`artifacts/environment/vps-2026-08-29.json`).
2. **Path MTU (PMTU)**: the largest IP packet that traverses the current path
   without IP fragmentation. It is path-, address-, route- and potentially
   traffic-class-specific, and can change after routing/path migration.
3. **Application ceiling (MPS)**: the largest complete Nekomusume UDP datagram
   the packetizer is willing to construct, including Nekomusume and crypto
   overhead. It MUST be `min(local interface/route limit, confirmed PLPMTU,
   configured protocol ceiling)`, after subtracting all outer headers. It is
   not a claim about the interface MTU or PMTU.

The packetizer sends one complete datagram per UDP write. It never relies on
IPv4 or IPv6 fragmentation for normal traffic. Records larger than the current
MPS are fragmented at the Nekomusume/session layer (or rejected by the API if
the record type is atomic); IP fragmentation is not a substitute for record
fragmentation.

Initial MPS is a conservative 1200-byte **IP packetization target** unless a
future protocol revision explicitly negotiates another base. The application
payload ceiling is therefore lower by UDP/IP, Nekomusume framing and crypto
overhead. The 1200 value is a starting policy, not a measured PMTU claim.

## Kernel and standards facts

* Linux `udp(7)` documents that UDP datagrams are subject to PMTU discovery by
  default; a write larger than the known path MTU returns `EMSGSIZE`, and the
  application should reduce its packet size. Disabling discovery permits IPv4
  fragmentation but is not recommended for performance/reliability.
* Linux `IP_MTU_DISCOVER(2const)` documents `IP_PMTUDISC_DO` (DF on all outgoing
  packets and `EMSGSIZE` above known PMTU), `IP_PMTUDISC_PROBE` (DF while
  ignoring the cached PMTU for deliberate probes), and `IP_MTU`/error-queue
  retrieval. These are Linux adapter details, not wire semantics.
* RFC 1191 defines IPv4 PMTUD using DF and ICMP Destination Unreachable,
  Fragmentation Needed (Type 3, Code 4). IPv4 routers may fragment only when
  DF is clear; Nekomusume will keep DF behavior enabled and will not choose
  fragmentation fallback.
* RFC 8201 defines IPv6 PMTU and ICMPv6 Packet Too Big. IPv6 forwarding
  routers do not fragment packets: a sender must emit a packet no larger than
  PMTU (an IPv6 source may use a Fragment header, but this design does not).
  IPv6's minimum link MTU is 1280, so an IPv6 path below the configured base is
  not usable by this UDP slice without a separately specified fallback.
* RFC 8085 §3.2 says large UDP messages normally require IP fragmentation,
  fragmentation harms reliability/efficiency, and applications should choose
  appropriate sizes. It also warns that UDP has no inherent congestion control.
* RFC 8899 defines DPLPMTUD for datagram packetization layers: probe packet
  delivery must be confirmed at the packetization layer; it handles black holes,
  reduces PLPMTU on loss, and cautiously searches upward. PTB messages may
  accelerate reduction but need validation. RFC 8899 §8 requires resource and
  amplification bounds to be part of the design.

Sources (retrieved 2026-08-30):
[udp(7)](https://man7.org/linux/man-pages/man7/udp.7.html),
[IP_MTU_DISCOVER(2const)](https://man7.org/linux/man-pages/man2/IP_MTU_DISCOVER.2const.html),
[RFC 1191](https://www.rfc-editor.org/rfc/rfc1191),
[RFC 8201](https://www.rfc-editor.org/rfc/rfc8201),
[RFC 8085](https://www.rfc-editor.org/rfc/rfc8085),
[RFC 8899](https://www.rfc-editor.org/rfc/rfc8899).

## Error and PTB handling

`EMSGSIZE` is a local send result, not proof that the peer or path is dead.
The carrier reports it with the attempted datagram size and path generation;
it MUST NOT enqueue, partially send, or mark Session delivery. The packetizer
reduces its usable ceiling to a value strictly below the failing size (or to a
kernel-reported MTU when that value is trustworthy), retransmits the logical
record in a smaller packet, and records the event. Repeated `EMSGSIZE` at the
base is a hard local configuration/path incompatibility, not a black-hole
verdict.

ICMPv4 Fragmentation Needed and ICMPv6 Packet Too Big are advisory path
feedback. Accept only a message that can be associated with a currently active
path and a recently transmitted authenticated packet (quoted tuple/flow and,
where available, packet identity). Reject stale, impossible, unsolicited, or
increasing MTU reports. Clamp any accepted value to protocol minima, the local
route/interface limit, and the configured ceiling. ICMP alone never advances
Session delivery or path validation.

## State machine and black-hole behavior

The per-path state is `Base -> Probing -> Confirmed`, with `Degraded` as a
health annotation; it is independent of the existing carrier/path state and
must carry path generation. Maintain `confirmed`, `upper_bound`, at most one
outstanding probe, bounded retries per size, and a bounded total probe budget.
Only an authenticated packetization-layer acknowledgement containing the exact
probe ID, path generation, and size can raise `confirmed`.

Normal data at or below `confirmed` supplies reachability evidence. A probe
above `confirmed` is not application data and is never delivered. On timeout,
retry only within the per-size budget; then lower `upper_bound` to below the
failed size and continue or converge. A timeout is not by itself path failure.

A black hole is declared only after repeated loss of traffic at or below the
confirmed size, with the ordinary liveness/recovery evidence required by the
carrier. On black-hole fallback, atomically reset the usable MPS to the safe
base, cancel the outstanding probe, and stop upward probing for a cooldown.
Keep the path `Degraded`; do not claim MTU causality or immediately fail the
Session. If base-sized authenticated liveness also fails through the carrier's
normal bounded failure gate, the existing failover state machine may select a
previously validated alternate path. The alternate path starts a new PMTUD
generation and inherits no old size evidence.

Loss of a large probe is ambiguous: it may be congestion, filtering, or MTU.
Therefore it reduces only the PMTU search bound, while confirmed-size loss is
needed for black-hole fallback. Successful ordinary traffic resets the
black-hole counter. A path may probe upward only infrequently and only while
healthy; route/path-generation changes restart from base.

## Probe amplification and resource bounds

Probes consume the same carrier pacing/congestion budget as data. Before peer
address validation, PMTUD probes are forbidden (or limited to the existing
address-validation response budget); an unauthenticated request cannot cause a
large response. After validation:

* one outstanding probe per path;
* fixed retry count per size (default candidate: 2 attempts);
* fixed total probe count per path generation (default candidate: 32);
* one probe response is no larger than the triggering probe and contains only
  the authenticated acknowledgement, never reflected payload;
* a cooldown and token bucket bound upward probes (initial candidate: at most
  one probe per RTT and no more than 1% of path bytes over a rolling interval);
* global concurrent paths, bytes, timers and memory remain bounded by the M3
  limits; each limit has deterministic exhaustion behavior.

These numbers are candidates for the implementation gate, not yet a claim that
production tuning is complete. The amplification invariant is strict:
PMTUD processing must not create more unauthenticated egress bytes than the
bounded request/validation budget permits, and it must never reflect attacker
chosen padding.

## Consequences and non-decisions

This decision does not implement socket options, `recvmsg(MSG_ERRQUEUE)`, an
ICMP parser, a wire probe frame, a runtime, or WAN probing. It does not change
the existing socket-free `neko-reliable::Plpmtud` candidate; that candidate's
`base_mtu`/`max_mtu` are packetization values and must be documented as such
before integration. In particular, the current candidate's `u16` fields cannot
by themselves represent IP-version header overhead, interface MTU, or an
application MPS.

The next implementation gate must define: exact probe/ack authenticated wire
fields, IPv4/IPv6 header accounting (including extensions/tunnels), PTB quote
validation, timer/cooldown constants, record fragmentation semantics, and
observable error/event schemas. Tests must cover local `EMSGSIZE`, PTB decrease,
stale/increasing PTB rejection, IPv4 DF/no-fragment policy, IPv6 no-router-
fragmentation policy, black-hole fallback, path-generation reset, and every
amplification/resource boundary. Until that gate is accepted, no implementation
or public listener is authorized.
