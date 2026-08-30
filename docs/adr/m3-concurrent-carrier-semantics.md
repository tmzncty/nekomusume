# ADR M3: UDP-primary carrier manager with warm TCP fallback

- **Status:** Accepted design contract; implementation, WAN and release gates remain separate
- **Date:** 2026-08-30
- **Scope:** carrier-manager concurrency, readiness, failover, migration, and Session-data safety
- **Repository anchor:** `86a6b0956ee03fa49530239fabc58b62107151a9`

## Decision

M3 uses **single-active, multi-ready** carrier semantics:

```text
        readiness / maintenance (overlapped)
 UDP primary  <---------------------------->  TCP fallback
       |                                           |
       +---------------- Carrier Manager ---------+
                              |
                    one active Session owner
```

UDP is preferred for new Session data. TCP may be established, authenticated,
validated, flow-control-admitted, and maintained while UDP is active, but a TCP
standby does not receive new application data. “Concurrent” means overlapped
candidate preparation and probing; it does **not** mean per-packet striping,
aggregation, or heterogeneous multipath scheduling. Those remain disabled by
the M4 gate.

Carrier/path identity is `(CarrierKind, PathId, PathGeneration)`. Reconnect,
address replacement, or a new validation attempt uses a new generation. Session
identity, delivery epoch, and logical stream offsets survive carrier changes.

## Carrier Manager contract

The manager is the sole owner of active-path selection. Session owns logical
delivery and ordering; carriers expose bounded I/O and path observations; the
manager converts only explicitly permitted observations into path state. No
carrier-local ACK, TCP write completion, or socket connection event may confirm
Session delivery.

### Operational states

| State | Meaning | New Session data | Legal transitions |
|---|---|---:|---|
| `standby` | Candidate exists but is probing, backing off, or below hysteresis | No | `warm`, `failed` |
| `warm` | Authenticated, independently validated, admitted, and ready for bounded control/resume | No | `active`, `draining`, `failed` |
| `active` | Sole owner of new Session data | Yes | `draining`, `failed` |
| `draining` | Retiring owner; may finish already-assigned work only | No | `failed` or removal at deadline |

`failed` is terminal for a path generation, not a service state. A replacement
generation starts at `standby`.

Required invariants:

1. There is at most one `active` path per Session active epoch.
2. A new path starts `standby`; promotion requires readiness and policy gates.
3. Migration is an ordered ownership change: old `active -> draining`, then new
   `warm -> active`. The manager never exposes two active owners.
4. `draining` rejects new application data. At its deadline, the old path is
   closed and unresolved logical ranges are replayed on the current active path.
5. A stale or mismatched generation is rejected before it can mutate active
   state, switch counters, or Session evidence.
6. Rejection is atomic. The only allowed non-activation mutation is a bounded
   hold counter when the documented hysteresis gate explicitly requires it.

### State-machine sketch

```text
new generation
      |
      v
 standby --validated + k_ready--> warm --promotion--> active
    |                                 |                 |
    +--------------fail---------------+                 +--graceful--> draining
                                                        |                 |
                                                        +--hard fail------> failed
```

The existing migration-back contract remains a separate guarded transition:
`TCP_ACTIVE -> TCP_HOLD -> UDP_ACTIVE`; it requires current-generation,
independent validation, healthy samples, score margin, and hold events. It is
not an implicit response to one successful probe.

## Readiness probes and amplification bounds

Readiness is a separate evidence domain from carrier packet feedback and Session
delivery. A readiness exchange must be authenticated and bound to Session
identity, path generation, and delivery epoch. TCP connect/write success and UDP
packet ACK are insufficient. A timeout is `probe_timeout`, not peer-closed proof.

Before address/path validation, the responder enforces:

```text
bytes_sent_unvalidated <= 3 * bytes_received_unvalidated
```

Every response—including errors, retry text, padding, and challenge material—
consumes that same budget. Absolute `max_unvalidated_bytes`,
`max_probe_payload`, `max_probe_rate`, and in-flight probe limits apply as well.
A rejected budget charge does not mutate probe, path, or Session state. After
validation, normal per-path and Session flow-control/rate limits still apply;
probe traffic cannot starve active data.

A candidate enters `warm` only after resource admission and `k_ready` consecutive
successful authenticated readiness observations (initial contract default:
`k_ready = 3`). Consecutive failure and hard-close detection are distinct:
one lost probe never switches the active path.

## Recovery classes and measurement

Each switch is an immutable event with old/new path generations, active epoch,
reason, timestamps, and bounded counters:

- `failure_decided_at`
- `new_active_at`
- `recovery_class`
- `recovery_latency_ms`
- `uncertain_bytes`, `replayed_bytes`, `duplicate_bytes`
- `confirmed_bytes`, `lost_bytes`, `success`

**Warm recovery** means an eligible TCP path was already `warm` before failure
decision. The interval ends only after resume validation and the first accepted
logical Session data on the new active path.

**Cold recovery** means no eligible warm fallback existed. Its interval includes
candidate creation, handshake, validation, resume, and first accepted logical
data. Warm and cold distributions must be reported independently using median
and P95. A failed warm resume is retained as a failed warm attempt; the eventual
new-generation recovery is cold.

These measurements are observations, not performance or production claims.

## Stable switch reason codes

The manager emits one stable reason code per ownership switch:

- `udp_blackhole`
- `udp_path_degraded`
- `tcp_ready_preferred`
- `address_change`
- `operator_request`
- `drain_deadline`
- `resume_rejected`
- `carrier_error`
- `shutdown`

The event also records `from`, `to`, active epoch, and generation values. Unknown
external values are rejected or represented as explicit `other(u16)`; they must
not weaken the safety policy.

## Dwell and anti-flap

Initial policy defaults are deterministic candidate values, not production tuning:

- probe interval: 1 second;
- `k_ready = 3` consecutive successes;
- `k_failure = 3` consecutive failed probes for degraded/failover suspicion;
- minimum active dwell: 5 seconds;
- post-switch voluntary-migration cooldown: 10 seconds;
- voluntary promotion requires candidate score at least 20% above active score.

A hard close or independently established hard failure may bypass dwell and
cooldown. A single probe failure may not. Cooldown suppresses voluntary reverse
migration, and a failed generation cannot be immediately promoted again. Every
counter and timer is bounded and checked for overflow.

## Drain and uncertain Session-data safety

Session delivery remains authoritative across carrier changes:

```text
UNSENT -> IN_FLIGHT -> CONFIRMED
                  \\-> UNCERTAIN -> replay -> CONFIRMED
```

At drain start, every assigned range without explicit logical Session delivery
proof becomes `UNCERTAIN`, regardless of TCP write acceptance or UDP packet ACK.
The sender retains uncertain ranges under byte, range-count, and age limits;
capacity exhaustion fails closed rather than discarding data. At the drain
deadline, all remaining uncertain ranges are replayed on the active path.

Replay identity is the authenticated stable Session/stream/offset tuple (or a
later approved DataId contract). Exact duplicate bytes are idempotent; conflicting
bytes at one identity are rejected. Delivery watermarks never decrease, and
ordered consumers cannot observe a later offset before preceding ranges are
accepted under the Session contract. Ambiguous resume authentication, epoch, or
receiver state is a fail-closed replay decision—not an inference of delivery.
Application side-effect idempotency remains above Session.

## Alternatives

1. **Cold fallback only:** lowest resource and implementation cost, but recovery
   starts after failure and has the worst outage latency. Rejected for M3.
2. **Single-active, warm standby (chosen):** overlaps TCP preparation with UDP
   service, preserves one ordering owner, and makes replay auditable. It costs
   bounded idle resources and explicit probe policy.
3. **Concurrent striping/aggregation:** possible capacity benefit, but requires
   cross-path sequence/reordering, congestion coupling, retransmission ownership,
   fairness, and stronger bounded-memory rules. Deferred behind the existing M4
   gate until controlled evidence justifies the complexity.

## Implementation gates

Before any broader implementation or WAN claim, add deterministic tests for all
state/illegal transitions, generation checks, reason codes, readiness evidence,
amplification and absolute probe budgets, dwell/cooldown, drain deadlines,
uncertain replay, duplicate/conflict handling, and bounded retention. Run the
loopback/process and netns warm/cold recovery experiments with the metrics above.
Run `./scripts/check.sh`, `git diff --check`, and fuzz smoke when wire/parser
fields change. This ADR adds no code, dependencies, wire fields, listeners,
WAN authorization, or release/security approval.
