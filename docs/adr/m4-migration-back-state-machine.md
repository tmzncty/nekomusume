# M4: guarded local migration-back state machine

**Status: Candidate design — local/loopback implementation gate only; not a WAN,
production, security, or protocol approval.**

## Scope and invariant

`CarrierManager::migrate_back_to_udp` is a proposal boundary while TCP is
active. It may promote only a previously observed UDP candidate; it must never
change Session delivery state, acknowledge data, or infer path validation from
health samples. Rejection is fail-closed and must leave the active path,
generation, switch counter, and candidate health unchanged (the hold counter is
the sole intentional exception for a hold-gate rejection).

## States and transitions

The guarded local state machine is:

```text
TCP_ACTIVE(g) --candidate rejected--> TCP_ACTIVE(g)
TCP_ACTIVE(g) --eligible proposal--> TCP_HOLD(g, udp, n)
TCP_HOLD(g, udp, n < H) --eligible proposal--> TCP_HOLD(g, udp, n+1)
TCP_HOLD(g, udp, n >= H) --eligible proposal--> UDP_ACTIVE(g)
UDP_ACTIVE(g) --new explicit set_active_tcp--> TCP_ACTIVE(g')
```

`g` is the active path generation, `H` is `min_hold_events`, and `n` resets
to zero on activation or an explicit TCP activation. There is no implicit
UDP recovery transition and no concurrent TCP/UDP striping transition in this
contract.

## Ordered guards

A proposal is eligible only when all guards pass, in this order:

1. TCP is active and the candidate is a different path (`NotTcp` otherwise).
2. Candidate generation equals the active generation. An older generation is
   rejected as `OldGeneration`; any other mismatch is `GenerationMismatch`.
3. Explicit independent path validation is present (`validated == true`).
4. Candidate health is healthy (`pto < 3` and `loss_per_mille < 500`).
5. The observed active TCP sample is healthy and the candidate score exceeds
   it by `switch_margin`.
6. The candidate has remained eligible for `min_hold_events` prior proposals.

Only guard 6 mutates state without activating (it increments the bounded hold
counter). Activation resets the hold counter and increments `switches` with
saturating arithmetic. Candidate input is bounded by the manager's observed
path limit; no network I/O is implied by this state machine.

## Evidence and test gate

The current unit test `migration_back_requires_validation_generation_health_margin_and_hold`
proves each rejection class, atomicity of the failed guards, the hold sequence,
activation, and post-activation `NotTcp`. Any future implementation change must
retain tests for stale/future generation, absent validation, unhealthy samples,
insufficient margin, repeated eligible observations, and activation counter
bounds. A loopback/process-level test is required before any separately reviewed
WAN observation; this ADR itself authorizes neither.

This document records the missing transition/invariant contract around the
existing gates. It does not add code, sockets, dependencies, wire fields, or
runtime behavior.
