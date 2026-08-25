# M0 candidate cross-layer integration gates

**Status: candidate model gates, not frozen normative behavior.**

`neko-carrier/tests/integration_gates.rs` exercises the boundary between the
pure `neko-carrier` path model and the pure `neko-session` delivery ledger. It
opens no socket, selects no runtime, performs no cryptography, and creates no
real TCP/UDP/failover connection.

The gates enforce these separations:

- `packet_feedback`, `SessionDelivery`, and `ChallengeValidated` are distinct
  evidence domains; none silently promotes Session delivery or path validation.
- A late old-generation path event is rejected before it can affect the
  delivery watermark; an old delivery epoch ACK is also rejected without
  changing the watermark.
- Duplicate `DataId` bytes are idempotent and conflicting bytes are not
  delivered a second time. This is a candidate integration assertion; the
  project still needs a normative DataId allocation decision.
- PTO is health/probe evidence only. It is not failure and not delivery proof.
- A path becomes active only after `ChallengeValidated`, the matching
  generation, and the configured success/dwell hysteresis gate.
- Carrier and Session resource limits remain independently bounded by their
  respective candidate models.

The candidate delivery proof remains `received`/transport acceptance only; it
never claims application `delivered`, `effect`, or side-effect commit.
