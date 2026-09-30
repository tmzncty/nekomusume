# R / live PLPMTUD — minimal integration plan (2026-09-29)

Owner: 祀 / Session 75590 (admin direction relayed by Session 85461).
Status of R before this plan: `BLOCKED_IMPLEMENTATION` (as of 2026-09-30 the local
integration is complete and locally validated; the remaining VPS evidence is
`BLOCKED_ENVIRONMENT`, retired vps-104 + fixed Meta TUN — see
`plpmtud-vps-run-plan-a82b5d7-20260929.md`) — `neko_reliable::Plpmtud`
is a socket-free state model that nothing in the carrier or CLI calls.

Inputs read: `docs/spec/m2-plpmtud.md`, `docs/adr/m2-pmtud-semantics.md`, D030/D047,
`docs/reviews/independent-plpmtud-757e0b6-20260913.md`, `AGENTS.md` §3.1/§3.3,
`neko_carrier::ReliableUdpRuntime`, `neko_wire::Frame`.

## Invariants every slice must hold

1. **Opt-in, default unchanged.** With PLPMTUD not enabled, every existing API behaves
   byte-for-byte as before, and no probe can be produced.
2. **A probe is not data.** A probe never enters `Recovery`, Reno bytes-in-flight,
   `RetransmitBuffer` or the packet→frame map. So its loss cannot be congestion
   loss, cannot move the health loss counters, and cannot schedule a Session (or
   carrier) retransmission.
3. **A probe is not in the ACK packet-number space.** The receiver must never feed a
   probe's packet number to `PacketAckTracker`. If it did, a later ACK could name a
   packet `Recovery` never saw, and `Recovery::on_ack` would reject the whole ACK
   (`largest > largest_sent`). Probes are acknowledged only by a dedicated,
   authenticated probe-ACK that carries `(probe id, path generation, exact size)`.
4. **Fail closed.** A torn-down runtime refuses every probe operation. A refused
   admission mutates nothing and does not consume a probe id or probe budget.
5. **Only an authenticated probe-ACK raises the confirmed size** (ADR). ICMP/PTB
   stays out of scope. EMSGSIZE can only lower the search bound.

## Shapes considered

| Shape | Idea | Verdict |
|---|---|---|
| A. Probe as a non-ack-eliciting `SentPacket` in `Recovery` | reuse loss detection | **Rejected**: its loss lands in `packets_lost` / `resolved_lost` (violates 2), and the receiver would have to put it in the ACK space (violates 3). |
| B. Probe outside `Recovery`, owned by an optional `Plpmtud` inside `ReliableUdpRuntime`; timeout driven by the model | smallest new state; the invariants hold by construction | **Chosen** |
| C. Separate `PmtuProber` object next to the runtime | no coupling | Rejected: the teardown/generation fail-closed rules would have to be duplicated and kept in sync by every caller. |

## Slices

1. **Carrier seam (this change, socket-free).** Add an optional `Plpmtud` to
   `ReliableUdpRuntime`: `enable_plpmtud`, `start_pmtu_probe`, `on_pmtu_probe_ack`,
   `on_pmtu_probe_timeout`, `on_pmtu_emsgsize`, `pmtu_confirmed_mtu`. Admission is
   checked against the congestion window before any mutation. Negative tests cover
   invariants 1–4.
2. **Authenticated wire frames.** `PmtuProbe { id, generation, size }` (padded to
   `size` inside the sealed record, so the padding is ours, never attacker-chosen)
   and `PmtuProbeAck { id, generation, size }` (small, never reflects padding).
   Both are ignorable-bit frames, so an old peer drops them instead of failing.
   Golden vectors plus decode negatives.
3. **Live CLI path behind an explicit flag** (for example `--plpmtud`) on the
   existing reliable-UDP client/server. It sends probes only after authentication,
   answers with a probe-ACK only, and emits bounded diagnostic events. Tests: the
   deterministic harness plus a real-process loopback case (the loopback MTU is
   65536, so the local run proves the plumbing, not a path MTU).
4. **One changed-hypothesis self-owned VPS run**: does the confirmed size converge
   under the real path's 1500 MTU, and does a probe loss above it stay out of the
   congestion and loss counters? Then reconcile `docs/status.md` (R) from the
   evidence. No same-class retry.

## Policy values — candidates only, NOT decided here (need 85461 / maintainer)

The ADR lists these as "candidates for the implementation gate". This plan does not
choose them. Until they are decided, slice 3 uses the ADR candidate values, and only
behind the explicit opt-in flag:

- probe size ceiling (`max_mtu`; ADR example 1500, and `PathMtuLimits` header
  accounting of 28 / 48 bytes);
- attempts per size (ADR candidate 2) and total probes per generation (candidate 32);
- probe cadence / token bucket (candidate: at most one probe per RTT and ≤1% of path
  bytes) and the black-hole cooldown;
- **whether a probe is charged to bytes-in-flight** or only admission-checked. Slice
  1 only checks admission (`can_send(size)`) and does not charge. That keeps
  invariant 2 trivially true, but the ADR says probes "consume the same carrier
  pacing/congestion budget". Charging would need a charge/release path owned outside
  `Recovery`. This is flagged as a decision point.

## Out of scope

ICMP / PTB parsing, `IP_MTU_DISCOVER` socket options, IPv6 (no environment),
READY_LIVE, release flags, H-I4-119 / D019.
