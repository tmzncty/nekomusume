# ADR M3: Concurrent carrier semantics — UDP primary, warm TCP fallback

- **Status:** Accepted design decision; implementation and WAN gates remain separate
- **Date:** 2026-08-30
- **Scope:** Carrier Manager policy and Session safety semantics

## Decision

Use **single-active, multi-ready** operation. “Concurrent” means overlapping
candidate establishment, authentication, probing, and standby maintenance—not
striping application data across carriers.

```text
                 readiness / maintenance
          UDP primary <------------> TCP fallback
                 \                    /
                  +-- Carrier Manager
                           |
                    one active Session path
```

UDP is preferred. TCP may remain connected and `warm` while UDP is active, but
warm/standby paths carry control/readiness only. At most one path owns new
Session data. Per-packet TCP/UDP striping, aggregation, and multipath scheduling
remain disabled by the existing M4 gate.

A path identity is `(CarrierKind, PathId, PathGeneration)`; reconnect/address
replacement creates a new generation. Session identity, delivery epoch, and
logical stream offsets survive carrier changes.

## Operational states

| State | Meaning | New Session data | Legal next states |
|---|---|---:|---|
| `standby` | Candidate exists but is probing, backing off, or below hysteresis | No | `warm`, `failed` |
| `warm` | Authenticated, validated, flow-control-admitted, bounded control exchange ready | No | `active`, `draining`, `failed` |
| `active` | Sole owner of new Session data | Yes | `draining`, `failed` |
| `draining` | Retiring path; flushes assigned work only | No | `failed` or removed at deadline |

`failed` is terminal for a generation and is not a fifth operational service
state. A new generation starts at `standby`.

- New paths start `standby`.
- `standby -> warm` requires authenticated readiness and resource admission.
- Promotion requires readiness plus policy gates. During migration the old
  active path enters `draining` before the new path becomes active; two active
  owners never exist.
- PTO/lost probes are health evidence only. A hard-failure detector may move
  an active path to failure; one timeout alone cannot do so.
- Draining never accepts new application data.

## Readiness and anti-amplification

Readiness is a separate evidence domain from carrier packet feedback and Session
delivery. The challenge/response must be authenticated and bound to Session
identity, path generation, and delivery epoch. TCP connect/write completion and
UDP packet ACK are not readiness or Session-delivery proof.

Before address/path validation:

```text
bytes_sent_unvalidated <= 3 * bytes_received_unvalidated
```

The responder additionally enforces absolute `max_unvalidated_bytes`,
`max_probe_payload`, and `max_probe_rate` limits. Responses, errors, padding,
and retry text all consume the same budget. Timeout yields `probe_timeout`, not
peer-closed evidence. Promotion requires `k_ready` consecutive successful
probes (default 3); probes cannot starve active Session data. Post-validation
flow-control and rate limits still apply.

## Recovery measurement

Every switch records `failure_decided_at`, `new_active_at`, `recovery_class`,
`switch_reason`, old/new path generations, `recovery_latency_ms`, and
`uncertain_bytes`, `replayed_bytes`, `duplicate_bytes`, `confirmed_bytes`,
`lost_bytes`, and `success`.

- **Warm recovery:** an eligible TCP path was already `warm`; latency starts at
  failure decision and ends after resume exchange plus first accepted Session
  data.
- **Cold recovery:** no eligible warm path existed; latency includes candidate
  creation, handshake, validation, resume, and first accepted data.

Warm and cold results are reported separately (median/P95). If a warm candidate
cannot resume, retain the failed warm-attempt record and classify the eventual
new-generation recovery as cold.

## Stable switch reason codes

`udp_blackhole`, `udp_path_degraded`, `tcp_ready_preferred`, `address_change`,
`operator_request`, `drain_deadline`, `resume_rejected`, `carrier_error`, and
`shutdown`. A switch event includes `from`, `to`, reason, active epoch, and
path-generation values. Unknown wire values are rejected or represented as an
explicit `other(u16)` without weakening safety behavior.

## Dwell and anti-flap defaults

These are candidate policy defaults, not production tuning claims:

- probe interval: 1 second;
- `k_ready = 3` consecutive successes;
- `k_failure = 3` consecutive failed probes;
- minimum active dwell: 5 seconds;
- post-switch cooldown: 10 seconds;
- voluntary promotion requires candidate score at least 20% above active score.

A hard close/failure may bypass dwell/cooldown. A single failed probe never
switches. Cooldown suppresses voluntary reverse migration and a failed
 generation cannot be immediately promoted again.

## Drain and uncertain Session data

Session delivery remains authoritative across carriers:

```text
UNSENT -> IN_FLIGHT -> CONFIRMED
                  \\-> UNCERTAIN -> replay -> CONFIRMED
```

Only explicit logical Session delivery proof is confirmation. At drain start,
all assigned data without that proof becomes `UNCERTAIN`, regardless of TCP
write acceptance or UDP packet ACK. The sender retains uncertain ranges under
bounded bytes/ranges/age limits; exhaustion fails closed rather than dropping
bytes. A drain deadline closes the old path and replays remaining uncertain
ranges on the active path.

Replay identity is stable authenticated Session/stream/offset (or the final
approved DataId). Identical duplicates are idempotent; conflicting bytes at one
identity are rejected. Watermarks never move backwards, and ordered consumers
do not observe later offsets before preceding ranges are accepted according to
the Session contract. If resume authentication, epoch, or receiver state is
ambiguous, fail closed and replay; never infer delivery from carrier success.
Application side-effect idempotency remains above Session.

## Alternatives and recommendation

1. **Cold fallback only:** simplest and cheapest, but starts recovery after
   failure and gives the worst outage latency. Rejected.
2. **Single-active, warm standby (chosen):** overlaps TCP preparation with UDP
   service, preserves one ordering owner, and keeps uncertain-data replay
   auditable. It costs bounded idle resources and policy complexity.
3. **Concurrent striping/aggregation:** potential capacity benefit, but requires
   cross-path sequence/reordering, congestion coupling, retransmission ownership,
   fairness, and stronger memory/safety contracts. Deferred behind the existing
   M4 gate.

## Implementation gates

Before implementation is promoted: deterministic state/illegal-transition,
generation, reason-code, dwell/cooldown tests; anti-amplification and absolute
probe-budget tests; Session drain/uncertain/dedup/conflict/bounded-retention
tests; and loopback/netns warm/cold recovery experiments with the metrics above.
Run `./scripts/check.sh`, `git diff --check`, and fuzz smoke when wire/parser
fields change. This ADR adds no wire fields, crypto choice, runtime, listener,
WAN authorization, or production threshold approval.
