# Post-auth blackhole seam rename — H-I4-121 closure note

Developer-owned companion to reviewer notes
`docs/reviews/h-i4-121-rootless-hard-loss-evidence-boundary-20260924.md`
(H-I4-121, HIGH) and
`docs/reviews/independent-r-mbox-rootless-loss-62d5914-20260924.md`.

## What changed and why

H-I4-121 correctly flagged that the `62d5914...` slice overstated the
`--drop-all-udp` process seam as a full "reply blackhole from the first
datagram" while the implementation only withheld post-authenticated
application-level UDP replies. Two defects were found and repaired in one
bounded developer pass:

1. **Test-support semantics defect (the gate failure).** Under the original
   flag the client's application record loop hit its bounded deadline waiting
   for the first Session DeliveryAck and failed closed before the health
   observation window could classify the degradation — hard loss never
   reached the failover it was built to exercise. Repair: an ordinary bounded
   receive timeout in the application record loop is a non-terminal
   continuation under `--automatic-health-failover` (H-R9-068 precedent);
   unconfirmed outstanding records are retained as uncertain ownership and
   replayed over TCP after manager promotion. Every other receive error
   stays fail-closed; without the flag the behavior is unchanged. The
   failover-server's expected TCP replay count includes the retained record 0
   under the seam; SessionRuntime exact-duplicate dedup keeps already
   delivered bytes single-counted.

2. **Contract narrowing (H-I4-121 item 1).** The seam is renamed to its
   provable semantics:

   - flag `--drop-all-udp` → `--drop-post-auth-udp-replies`
   - machine-readable mode label `hard_udp_loss` → `post_auth_udp_reply_blackhole`
   - test `executable_loopback_hard_udp_loss_drives_tcp_failover` →
     `executable_loopback_post_auth_udp_reply_blackhole_drives_tcp_failover`

   Comments now state explicitly: negotiation and Noise handshake replies are
   NOT affected; the impairment starts only at the authenticated application
   boundary; the seam is not a directional network-topology model. Startup
   semantics are unchanged (H-I4-121 item 4: no silent startup/negotiation
   change merely to match the old label).

## Evidence boundary (unchanged)

- `FaultInjectCarrier { loss_percent: 100 }` remains the deterministic
  Carrier-level hard-loss evidence, explicitly separated from process/
  topology evidence (H-I4-121 item 2).
- The process test proves: UDP negotiation/authentication succeeds; every
  post-authentication application-level UDP reply is withheld from the first
  application record; health evidence (`udp_health_failed`, 3 observation
  windows, `authenticated_delivery_ack_timeout`) drives the cold TCP
  fallback; all three records/48 bytes complete over TCP; no fabricated
  duplicate metric (H-I4-121 item 3).
- This supersedes the specific "from first datagram / server never replies /
  hard black hole" claims in the earlier reviewer note's reading of the
  process seam (H-I4-121 item 6). The earlier notes are historical evidence
  and are not rewritten.
- `READY_LIVE: none` unchanged. No release flag, D019, H-I4-119, or standing
  authorization was touched.
