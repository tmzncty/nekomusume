# M0 carrier path state (candidate)

**Candidate only; not a frozen protocol and not real failover.** `neko-carrier` is a pure synchronous state model. It opens no socket and does not implement network, routing, tunnel, or QEMU behavior.

## Identity and states

`PathId`, `CarrierKind`, and `PathGeneration` are distinct types. A path has validation (`candidate -> validating -> validated`) and operational state (`candidate`, `validating`, `active`, `degraded`, `draining`, `failed`). At most one path is active in an `active_epoch`; activation is explicit and increments that epoch.

## Evidence domains

`PacketFeedback`, `PathValidated`, and `SessionDelivery` are distinct API types. ACK/loss/reordering can affect health and success counters, but cannot validate a path. Only an explicit `ChallengeValidated` event reaches `validated`. Session delivery is a separate observation; TCP Connected/WriteAccepted are intentionally not represented as delivery evidence.

PTO expiry marks an active path `degraded` only. It never marks a path failed or valid. Failure requires an explicit `Fail` event from degraded or draining. Old generations and late old-path evidence are rejected.

## Candidate failover gates

A validated candidate becomes eligible for explicit `Activate` only after configurable `k_successes` and `min_dwell_events` hysteresis thresholds. These are deterministic state-machine gates, not an automatic failover implementation. Resource limits and illegal transitions return deterministic errors.
