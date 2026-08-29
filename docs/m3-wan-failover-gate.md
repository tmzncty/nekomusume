# M3 WAN failover CLI gate

The `neko failover`, `neko failover-server`, and `neko failover-client`
commands currently implement an explicit **bounded admission gate**, not a WAN
runner. They validate `count` (1–64), payload size (1–1200 bytes), duration
(1–30 seconds), and ports (40080–40100), then require `--loopback-only` and emit
a machine-readable simulator-gate result. Without that flag they fail closed.

This is intentional: the existing `server`/`client` commands each own one
transport-bound `SecureSession`; they do not yet preserve one logical Session
across two listeners or carry `ResumeBinding`/`DELIVERY_ACK` state between
processes. Enabling WAN orchestration before that seam is implemented would
create a misleading reachability claim and risk unbounded listener behavior.

## Next executable seam

Implement a separate bounded runner around carrier adapters and
`SessionRuntime`, with UDP and TCP addresses supplied independently but both
restricted to 40080–40100. The runner must keep Session socket-free, use a
fresh Noise handshake plus validated `ResumeBinding`/`ResumeGuard`, resend only
uncertain logical ranges, and terminate after bounded count/duration. Add a
loopback process-level test before any authorized isolated WAN observation.

No proxy, tunnel, 0-RTT, concurrent multipath, public listener, or WAN test is
enabled by this gate.
