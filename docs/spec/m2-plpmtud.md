# Bounded packetization-layer PMTU discovery candidate

**Status: deterministic socket-free research model; not live path probing.**

`neko-reliable::Plpmtud` follows a packetization-layer approach: only an
explicit authenticated acknowledgement bound to probe ID, path generation and
exact size can raise the confirmed MTU. Unauthenticated ICMP is not trusted and
ordinary loss is not path-failure evidence.

The default design starts from a configured safe base (normally 1200 bytes),
keeps an explicit upper bound, and uses bounded binary search. Exactly one probe
may be outstanding. Each size has a bounded retry count and the entire search
has a probe limit. Exhausting retries lowers the upper bound; it does not fail
the path. ACKs for stale generations/IDs, wrong sizes and duplicates are rejected.
A new path generation discards all prior probe evidence and restarts from base.

Repeated loss at or below the currently confirmed size can trigger conservative
blackhole fallback to the configured base after a threshold. Progress resets the
counter. Fallback does not claim that MTU caused the loss or that the path failed.
Tests cover exact convergence for 1200/1201/1280/1499/1500, timeout/retry,
reorder/stale/duplicate/wrong ACKs, one-outstanding/resource bounds, invalid
configuration and blackhole fallback.

No socket, DF bit, route, listener, ICMP parser, public endpoint or benchmark is
introduced. Probe messages must later travel inside the authenticated carrier
record before this model may be connected to a live path.
