# M3 WAN failover CLI gate

The `neko failover`, `neko failover-server`, and `neko failover-client`
commands now implement an executable **bounded loopback runner**. They validate `count` (1–64), payload size (1–1200 bytes), duration
(1–30 seconds), and ports (40080–40100), and default to loopback-safe operation. The runner uses real UDP primary and TCP
standby sockets; it is still a research candidate and must not be exposed publicly.

The runner preserves logical SessionId 7001, authenticates a fresh TCP Noise
resume, validates `ResumeGuard`, ACKs duplicate data, and bounds all loops and
frames. It binds only addresses explicitly supplied by the operator; public WAN
use remains prohibited pending review.

## Next executable seam

Implement a separate bounded runner around carrier adapters and
`SessionRuntime`, with UDP and TCP addresses supplied independently but both
restricted to 40080–40100. The runner must keep Session socket-free, use a
fresh Noise handshake plus validated `ResumeBinding`/`ResumeGuard`, resend only
uncertain logical ranges, and terminate after bounded count/duration. Add a
loopback process-level test before any authorized isolated WAN observation.

No proxy, tunnel, 0-RTT, concurrent multipath, public listener, or WAN test is
enabled by this gate.

## Process boundary wire (candidate)

`neko-session::ProcessMessage` is the socket-free seam for a future runner.
Frames are capped at `PROCESS_FRAME_MAX = 4096` bytes and use the fixed `NK`
magic plus version `1`. The candidate message set is:

- `Data`: logical Session ID, stream, byte offset, and bounded payload;
- `Resume`: logical Session ID plus delivery/key/path generation, expiry, and
  opaque resume token;
- `DeliveryAck`: logical Session ID, stream, offset, and acknowledged length.

Encoding and decoding are exact-length and fail closed on truncation, unknown
version/type, overflow, empty data, or frames above the cap. This protocol is
only an in-process/test boundary today; it does not open sockets, bind WAN
listeners, or claim cross-process failover. A Carrier runner must authenticate
transport records separately, validate the resume claim with `ResumeGuard`, and
feed decoded data/ACKs into the same `SessionRuntime` instance before any WAN
experiment is reconsidered.
