# M3 WAN failover CLI gate

The `neko failover`, `neko failover-server`, and `neko failover-client`
commands implement an executable bounded real-socket research runner. They
validate `count` (1–64), payload size (1–1200 bytes), duration (1–30 seconds),
and ports (40080–40100), default to loopback-safe operation, and use UDP primary
plus TCP standby sockets.

Both UDP admission and TCP resume perform canonical version negotiation before
a fresh Noise handshake and bind the exact negotiation transcript into the
authenticated prologue. The runner preserves logical Session ID 7001, validates
the TCP `ResumeBinding` with `ResumeGuard`, resends uncertain logical ranges,
deduplicates delivery through one `SessionRuntime`, and cryptographically opens
and semantically checks Session/stream/offset/length before accepting
`DeliveryAck`. UDP packet ACK and Session delivery acknowledgement remain
separate concepts.

Bounded temporary runs between administrator-controlled endpoints are permitted
by `docs/standing-vps-lab-authorization.md`; they are not prohibited pending
another per-run review. Operators must stay within that authorization's limits,
use explicit self-owned endpoints, retain actual parameters/evidence, and clean
up listeners, identities, logs, and processes. Third-party targets, public or
general reachability claims, production exposure, and release/security approval
remain blocked.

## Retry and state boundary

The server keeps bounded same-peer caches for negotiation selection and the
first Noise response. Duplicate/late client hello replays only the selected
version response while negotiation is pending. After authentication, a duplicate
Noise first-message replays only the exact cached Noise response. Deterministic
process tests drop the first selection and, separately, the first Noise response;
they assert that replay does not renegotiate or reauthenticate and does not reset
the selected transcript, `ResumeGuard` binding, Session ID, path generation, or
Session delivery state.

## Evidence boundary and next gate

The existing real-socket scenario performs an explicit controlled application
UDP stop. It proves the negotiated/authenticated runner and controlled resume
path, not automatic detection of a natural UDP blackhole, configured health/PTO
threshold crossing, or production failover. Pre-`f680702` evidence retains valid
negotiation, authenticated admission/resume, ordered server receive, and cleanup
facts, but its old plaintext/opaque ACK wording is superseded; only post-fix runs
may establish authenticated exact-semantic Session `DeliveryAck` evidence.

The next release-matrix gate remains a bounded self-owned replacement run of the
post-fix binary, followed separately by threshold-driven natural-degradation
integration/evidence. Neither gate is checked by this document update.

No proxy, tunnel, 0-RTT, concurrent multipath, public listener, general WAN
reachability, release candidate, or production claim is enabled by this gate.

## Process boundary wire (candidate)

`neko-session::ProcessMessage` remains the socket-free logical boundary. Frames
are capped at `PROCESS_FRAME_MAX = 4096` bytes and use fixed `NK` magic plus
version `1`:

- `Data`: logical Session ID, stream, byte offset, and bounded payload;
- `Resume`: logical Session ID plus delivery/key/path generation, expiry, and
  opaque resume token;
- `DeliveryAck`: logical Session ID, stream, offset, and acknowledged length.

Encoding and decoding are exact-length and fail closed on truncation, unknown
version/type, overflow, empty data, or frames above the cap. Carrier transports
must authenticate records separately, validate resume with `ResumeGuard`, and
feed decoded data/acknowledgements into the same `SessionRuntime`.
