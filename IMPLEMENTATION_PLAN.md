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
- [x] **HY2 changed-hypothesis harness repair:** typed failure-row generation retains the first identified attempt without `jq --argjson` diagnostics; payload provenance is explicitly prepared-or-null; missing cleanup observations remain unknown; and sampler-owned process groups are emptied and verified before cleanup completion. Blocked artifacts preserve valid sample prefixes without comparative summaries. This is local deterministic harness evidence only; no new VPS comparison was run.

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
   remain classified as such. Historical failed D064, periodic, and HY2 attempts remain valid negative evidence. Exact `25e0daa` adds a controlled application-level UDP reply-cessation warm fallback: 3/3 records, 48 application bytes, two uncertain/replayed records, duplicate 0, lost 0, and approximately 434 ms failure-decision-to-first-resumed-data; it is not natural degradation or PTO-blackhole evidence. Exact `25e0daa` also adds one approximately five-minute periodic direct-path sample: 60 x 32-byte records, 60/60 confirmed, with no missing, duplicate, or conflicting record; it is one bounded sample only. At exact `f1cb9af`, HY2 ended `BLOCKED_HARNESS` during preflight SSH authentication, with no payload, samples, paired statistics, or comparison. The one-invocation control was violated because the harness was invoked twice; both attempts ended identically with preflight RC2 because the SSH preflight user contract was not explicit and the configured alias resolved to `tmzn`; no root assumption is valid. An unchanged retry was prohibited. At exact `bc38d06`, exactly one substantive changed-hypothesis invocation passed explicit SSH preflight and prepared its 1200-byte payload, then ended `BLOCKED_HARNESS` during setup because `run_client` expanded `impl` before assignment under `set -u` at line 186. It produced zero samples, paired statistics, or comparison. Its validator-valid result SHA-256 is `596ad4b73058143db1918613dd970e44e8e6bf3a1b89602ac0012f911b6d2653`; artifact-recorded cleanup failed with one remote listener remaining, remote process groups not reaped, and remote temp-path removal unknown, while independent manual post-run cleanup subsequently verified no experiment ports, processes, or temporary paths. At exact `3d54585`, exactly one invocation after green exact-head CI prepared the payload and retained a valid ordered two-record prefix (`nekomusume-1` success, `hy2-1` `client_exit`), ending overall `BLOCKED_HARNESS` at `hy2-1-failed`; there are no complete pairs or comparative summary. The tracked validator-valid result SHA-256 is `dc7d4a0887ebc5617dbc34b5146563af7178445ea2ba05d30da05276f4558602`. Automatic cleanup failed solely on `remote_process_groups_reaped=false`; listeners were zero, remote temp removal and local cleanup succeeded, and later serialized double-end postchecks found no experiment ports/processes/temp without rewriting the artifact. IPv6 remains environment-blocked, and the remaining matrix rows, including any HY2 comparison, are still open; this item is
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
