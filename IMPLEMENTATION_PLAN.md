# Nekomusume executable implementation and RC plan

This is the agent execution queue required by `AGENTS.md`. [`ROADMAP.md`](ROADMAP.md)
describes milestones; [`docs/status.md`](docs/status.md) is the authoritative
capability/governance ledger. Checkmarks here mean that the bounded repository
slice exists and its local gate passed, not that a release, security audit,
protocol freeze, public reachability or production approval exists.

## Fixed governance state

- `IMPLEMENTATION_COMPLETE=true`
- `RELEASE_CANDIDATE=false`
- `PRODUCTION_READY=false`
- `FREEZE=false`
- `RELEASED=false`

Standing authorization permits bounded temporary experiments on self-owned
client/VPS endpoints. It is an execution authorization, not an RC prerequisite
or release/security/production evidence. A self-owned host-address result must
not be promoted to public-WAN reachability.

## Completed bounded N slices

- [x] **N0 governance separation:** implementation completion is independent of
  RC, production readiness, freeze and release state.
- [x] **N1 negotiation primitive:** bounded fail-closed version negotiation.
- [x] **N2 executable candidate vectors:** structure validation plus Rust
  execution against current implementation; this described the pre-N9 candidate state.
- [x] **N3 compatibility harness:** current/current acceptance and unsupported /
  future rejection. Previous/current remains inapplicable until a prior frozen
  release exists.
- [x] **N4 lifecycle readiness repair:** incomplete readiness is fatal; bounded
  TCP/UDP evidence reaches READY before traffic and STOPPED on signal. Historical
  N8 raw logs remain immutable and their old FAILED readiness subclaim is
  superseded only by the repair evidence.
- [x] **HY2 changed-hypothesis harness repair:** typed failure-row generation retains the first identified attempt without `jq --argjson` diagnostics; ordered local/remote process-group reaping precedes runtime deletion, and cleanup blockers remain truthful artifacts. This is local deterministic harness evidence only; no new VPS comparison was run.

- [x] **N5 package lifecycle:** reproducible x86_64 package install A -> upgrade B
  -> rollback A with retained external state and cleanup evidence.
- [x] **N6 architecture audit:** first-RC target, if later approved, is
  x86_64-unknown-linux-gnu only; aarch64 remains a candidate target.
- [x] **N7-11 TCP multistream authenticated admission:** exact N1 transcript is
  bound into Noise; executable mismatch and unsupported-only peers fail before
  Session data admission. This does not negotiate probe, failover/resume or UDP.
- [x] **N8 self-owned endpoint matrix:** bounded authenticated TCP/UDP
  multi-record exchange only; no independently observed public-WAN, NAT or live
  failover conclusion.

## Next executable work (strict dependency order)

1. [x] **N9 canonical-corpus freeze.** Independently reviewed bytes, semantics,
   exclusions and oracle-specific implementation mapping are frozen as corpus v1
   with `freeze=true` and a content-addressed identity. This corpus-specific fact
   does not imply RC, security approval or release, and does not freeze Noise,
   ciphertext, carrier packetization, failover/resume or the global protocol.
2. [x] **Negotiation path completion.** Generic TCP/UDP probes and the bounded
   UDP-primary -> TCP resume path use canonical negotiation before fresh Noise,
   authenticate the exact transcript, enforce the same selected Session version,
   and preserve ResumeGuard replay/resource boundaries before data admission.
   The process fixture truthfully reports controlled application fault injection;
   it is not evidence of packet-level ACK/PTO blackhole detection.
3. [ ] **Bounded release evidence matrix.** Under standing authorization, collect
   reproducible independently controlled IPv4/IPv6, UDP degradation -> TCP
   fallback, long-lived and NAT/endpoint-change evidence with actual parameters,
   endpoint ownership, cleanup and negative results. Self-owned same-host paths
   remain classified as such. The D064 pre-failure seam now uses three bounded authenticated peer request/response observations, fail-closed responder admission, a one-second per-probe and three-second whole-sequence readiness policy additionally bounded by remaining experiment duration, deterministic reset-on-failure manager policy, live runtime admission, negative process tests, and local warm/cold tests, but current-exact-head VPS warm recovery remains absent: the retained changed-path run failed closed at challenge 3. The periodic implementation now separates a bounded 5000 ms default setup deadline (maximum 10000 ms, connect through Noise, with a compatible accepted-server deadline) from the per-record ACK deadline; a current-tree periodic VPS row remains absent pending changed-path evidence. The owned-lab fair-pair adapter and exact-payload seam are implemented and fully gated; its HY2 server now requires a distinct, remotely verified local `LAB_REMOTE_BIND_ADDRESS` and generates no wildcard listener, but a temporary HY2 QUIC/UDP path timeout prevented all comparative samples, statistics and superiority claims. IPv6 remains environment-blocked, and the remaining matrix rows are still open; this item is
   intentionally not complete.
4. [ ] **Independent release/security review.** Resolve findings, verify resource
   and abuse limits, compatibility policy, package rollback, operator lifecycle,
   canonical vectors and comparison methodology.
5. [ ] **RC decision.** Only a reviewed decision after items 1-4 may change
   `RELEASE_CANDIDATE`. Production readiness, protocol freeze and release remain
   separate decisions and must never follow automatically.

Agents select the first unchecked item whose explicit dependencies and required
environment are available. A blocked network row does not block independent
local review or test work; record the blocker and continue with the earliest
independent READY item. Every completed item must update evidence and
`docs/status.md` together where status changes.

## Experimental Track C — later enhancement gates

These markers mirror `ROADMAP.md` for the plan-sync gate; they are closed
candidate decisions, not release claims.

- [x] bounded authenticated PLPMTUD state
- [x] bounded XOR FEC candidate (disabled; evidence does not select it)
- [x] bounded authenticated unreliable datagram API
- [x] bounded synchronized key update
- [x] 0-RTT gate closed (disabled; no early data)
- [x] concurrent UDP + TCP gate closed (disabled; no striping)
- [x] heterogeneous multipath aggregation gate closed (disabled; no aggregation)
