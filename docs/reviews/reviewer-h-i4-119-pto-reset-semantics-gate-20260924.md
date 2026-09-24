# H-I4-119 reconciliation — PTO-reset semantics require maintainer/spec choice

**Classification:** MAINTAINER / CORE ACK-PTO SEMANTICS GATE. Do not auto-repair as an ordinary READY_LOCAL correctness slice.

**Exact repository anchor reviewed:** `b2a8ba7b1a7316eee880ac045eea55f90b1215a2`.

**Supersedes the execution classification, not the historical finding:** `docs/reviews/reviewer-h-i4-119-non-ack-eliciting-ack-pto-reset-20260924.md` remains an immutable record of the challenged sequence. This note corrects the claim that its proposed repair is already uniquely determined by committed repository semantics.

## Why the previous FRONT is not safe to implement automatically

Exact-current `Recovery::on_ack` resets `pto_count` whenever an ACK newly retires at least one sent packet:

```text
if !out.acked_packets.is_empty() {
    self.pto_count = 0;
}
```

`SentPacket::ack_eliciting` is real implemented state, but the current repository gives it at least two already-committed meanings that do **not** by themselves decide the PTO-backoff-reset rule:

- `PathRecovery::on_sent` charges Reno bytes only for `ack_eliciting=true` packets;
- `PacketAckTracker` uses `ack_eliciting` to decide whether receipt creates a pending ACK obligation, avoiding ACK-of-ACK ping-pong.

The repository documents reviewed in this pass (`README.md`, `AGENTS.md`, `ROADMAP.md`, `IMPLEMENTATION_PLAN.md`, `SECURITY.md`, `docs/status.md`, `docs/decisions.md`, `docs/carrier-architecture.md`, `docs/specs/nekomusume-session-v0.md`, `docs/release-security-review-packet.md`) do not state the stronger rule that **only** newly acknowledging an ack-eliciting packet may reset `pto_count`.

The external standards reference also does not make the previously proposed repair an obvious QUIC-derived correctness requirement. RFC 9002 Appendix A.7 first computes `newly_acked_packets`; if that set is nonempty, it later resets `pto_count` after ACK/loss processing (subject to the address-validation exception). Its `IncludesAckEliciting(newly_acked_packets)` predicate guards RTT sampling, not the `pto_count` reset. Section 6.2 separately says a sender should restart the PTO timer when an ack-eliciting packet is sent or acknowledged, while describing the PTO backoff factor as reset when an acknowledgment is received. Those are related but not identical state transitions.

Nekomusume is not required to copy QUIC exactly, and its current simplified `pto_count` is also reused as health/persistent-congestion evidence. That makes the intended reset boundary an actual project semantic choice rather than a reviewer-safe one-line bug fix.

## Maintainer/spec decision required

Choose and document one of these semantics (or another explicit reviewed rule) before changing code:

1. **Any newly acknowledged sent packet resets `pto_count`** — close H-I4-119 as no code defect in the present reset condition, while retaining tests/documentation that distinguish RTT sampling, ACK obligation, Reno charge, and PTO reset.
2. **Only newly acknowledged ack-eliciting packets reset `pto_count`** — explicitly adopt that project rule, then the previously proposed focused repair/regression becomes READY_LOCAL.

The choice should also state how this project's PTO-count-based persistent-congestion/health evidence is intended to relate to the chosen reset rule. Do not invent a new threshold or capacity value while resolving it.

## Immediate execution consequence

- H-I4-119 is **not** a coding-agent FRONT until the semantic choice is made.
- Do not patch `Recovery::on_ack` merely because the earlier review note proposed `newly_acked_ack_eliciting` gating.
- Do not let this gate idle the coding agent. Continue the dependency-ready queue outside this semantic seam: residual Recovery review that does not decide PTO reset, pre-auth owner-diff reuse/challenge, broad 13-surface moved-owner inventory, moved-owner challenges, release-packet/security-boundary reconciliation, and other unchanged READY_LOCAL review support.
- `READY_LIVE: none` remains unchanged; this question is deterministic local protocol semantics and creates no new real-network hypothesis.
- Release items 3 and 4 remain incomplete; `RELEASE_CANDIDATE=false`, `PRODUCTION_READY=false`, `FREEZE=false`, and `RELEASED=false` remain unchanged.

## Evidence boundary

This reconciliation is repository source/spec review plus standards research. It is not reviewer-local Rust/full-gate execution, hosted CI, cross-platform execution, fuzz, WAN evidence, performance evidence, or a release/security approval.
