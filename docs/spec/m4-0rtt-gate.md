# 0-RTT gate decision

**Status: explicitly disabled; no implementation authorized by this gate.**

The current candidate has authenticated Noise IK handshakes, bounded replay for
established records, key-phase update, trust/scope authorization, and bounded
pre-auth budgets. That is not sufficient for 0-RTT. Early application data would
need replay-safe resumption identity, persistent anti-replay state across
restart/rollback, freshness and ticket policy, authorization binding before any
side effect, bounded duplicate handling, and independent review of all failure
and privacy behavior.

Until those invariants have a concrete design, canonical vectors, persistence
and rollback tests, and review evidence, early application/session data remains
rejected. No early `Delivery`, path, ACK, authorization, or state-mutating
control evidence may be created. The existing `SessionRejected` boundary is the
expected outcome.

This is a governance closure, not a claim that 0-RTT is impossible or secure.
No code, wire format, ticket, resumption key, public listener, production
configuration, or performance comparison is introduced.
