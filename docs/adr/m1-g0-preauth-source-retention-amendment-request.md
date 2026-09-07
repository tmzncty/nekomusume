# ADR Amendment Request: D019 Terminal Source Retention

- **Date:** 2026-09-07
- **Status:** Review required — not approved and not implementation authorization
- **Parent:** [`m1-g0-preauth-resource-budget.md`](m1-g0-preauth-resource-budget.md)

## Conflict

D019 states that pre-auth counters are never reset by retry, reconnect, carrier
change, identity change, or error, and that input/response counters are charged
for the source over the state lifetime. It also requires bounded pre-auth state,
source and global memory, queue, and concurrency accounting.

The current implementation uses a bounded in-process `sources` map whose source
entry is removed when its last live state is released. A later retry,
reconnect, or carrier change can therefore create a fresh source entry and
start fresh source-lifetime counters. The C1 carrier-aware source projection
prevents TCP/UDP aliasing, but does not solve persistence after terminal cleanup.

This is a semantic policy conflict, not an implementation bug that can be
resolved honestly by inventing a retention number.

## Why the conflict matters

- Resetting on cleanup permits an attacker to rotate attempts over time and
  evade D019's literal no-reset source lifetime accounting.
- Retaining every historical source forever permits attacker-driven source-map
  growth and conflicts with D019's bounded-memory requirement.
- Carrier separation must remain explicit: TCP and UDP source domains must not
  alias while any approved retention policy is applied.
- A terminally rejected source must not regain protocol success merely because
  accounting state was discarded.

## Policy shapes requiring maintainer/reviewer selection

No numeric TTL, LRU size, epoch, or history bound is proposed here. Possible
shapes to evaluate are:

1. **Bounded retention:** retain terminal source accounting under an explicitly
   approved bounded eviction/expiry policy, with eviction semantics that do not
   silently turn prior rejection into success.
2. **Stable external/accounting identity:** move source-lifetime accounting to a
   separately bounded authority whose identity and capacity semantics are
   explicitly specified.
3. **Weaken D019:** redefine “source lifetime” to mean only the lifetime of a
   live admission state and explicitly accept reset on terminal cleanup,
   reconnect, and carrier change.
4. **Reject the literal requirement for this implementation scope:** preserve
   bounded in-process cleanup and record the resulting limitation as a release
   blocker rather than implying D019 compliance.

## Required decision evidence

Before closure, the selected shape needs deterministic tests for same-source
retry/reconnect, carrier transition, concurrent source/global limits, terminal
rejection non-revival, cleanup, and bounded memory. It also needs a security
review confirming that eviction/reset cannot create an admission or evidence
bypass and that TCP/UDP projections remain non-colliding.

Until an approved policy resolves this conflict, this lane remains blocked at a
policy checkpoint. No release, production, freeze, or public-network claim is
changed by this request.
