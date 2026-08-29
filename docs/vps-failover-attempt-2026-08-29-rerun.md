# Bounded VPS failover rerun — 2026-08-29

## Authorization and bounds

The administrator explicitly authorized one correctly serialized rerun against VPS `192.144.192.215`: TCP `40080`, UDP `40081`, three records of 16 bytes, maximum duration 10 seconds. No second rerun, scan, proxy, tunnel, or production operation was performed.

## Candidate integrity

The release binary was commit `35f6593` and SHA-256 `bce999e3d64f42d76eb99f8fcfe0fabd67a62d8b687e1949481b08a1fe2dbbe7` on both client and VPS.

## Serialization evidence

The VPS server was started first and `ss` confirmed:

- `0.0.0.0:40081/udp`
- `0.0.0.0:40080/tcp`

Listener readiness was recorded at `2026-08-29T22:31:40+08:00`. The client started at `2026-08-29T22:32:03+08:00`, after readiness confirmation.

## Result

- Client exit code: `2`.
- Client stderr: `neko: UDP handshake timeout`.
- VPS server stderr: `neko: failover timeout`.
- No cross-carrier WAN failover succeeded.

Unlike the earlier attempt, this run is validly serialized and therefore is evidence of a bounded failure for this exact candidate, route, endpoint and parameter set. It does not establish a general reachability claim, interoperability claim, performance result, security result, or production readiness.

## Cleanup

After the client ended, the VPS server was terminated and waited for. A final `ss` check found no listener on TCP `40080` or UDP `40081`; no test process remained. Temporary binary, identities, logs and runtime directories were removed from both hosts. No firewall changes were made.

Further WAN retries require fresh explicit authorization.

## Frozen WAN evidence boundary

This evidence is frozen: candidate `125bbcf` failed during the UDP handshake and never entered an established-session failover transition. No automatic WAN retry was performed or enabled. Any future WAN attempt requires fresh explicit authorization; this worktree adds diagnostics only and does not touch public WAN.
