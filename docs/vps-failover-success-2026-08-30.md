# Bounded VPS surviving-session failover — 2026-08-30

## Authorization and scope

The administrator authorized use of the dedicated experimental VPS
`192.144.192.215` for bounded Nekomusume development and network experiments.
This run used only the project VPS and the controlled client; no third-party
system was contacted or modified. Temporary listeners, identities, logs and
binaries were cleaned after the run.

## Candidate and bounds

- Candidate parent commit: `b32815acd0748fa1db9654ee9ec9ee82451edeec`.
- The exact locally built `neko-cli` binary was copied to the VPS.
- TCP port: `40080`; UDP port: `40081`.
- Eight records, 64-byte payloads, 15-second bounded duration.
- Server listener was started first; `ss` verified both intended listeners
  before the client began.
- Server used the generated client public key in its allowlist. This corrects
  the prior changed-TCP baseline configuration mistake where the authorization
  key was not passed correctly.

## Result

The run succeeded with client exit code `0` and server exit code `0`:

1. UDP socket bind and client send/receive stages completed.
2. UDP authenticated setup completed for stable Session `7001`, generation 0.
3. The bounded client emitted `udp_blackhole_injected` after the first UDP
   application record/ACK step.
4. A fresh TCP handshake and `tcp_resume_guard` completed for the same Session
   `7001`, generation 1.
5. Eight ordered records were sent over TCP; the server reported
   `failover_server_ok`, `records=8`, `bytes_hex` containing 64 bytes of `0x78`,
   and `duplicates=0`.
6. The client reported `failover_client_ok`, `count=8`, `bytes=64`, and
   `udp_blackhole=true`.

The captured event order was:

```text
udp_authenticated -> udp_blackhole_injected -> tcp_resume_guard
-> ordered_records_complete
```

Server carrier events were:

```text
udp_authenticated -> tcp_resumed
```

## Evidence boundary

This is the first retained bounded evidence of an established-session UDP
failure followed by TCP resume on the dedicated VPS, with ordered delivery and
zero reported duplicates for this exact candidate, route, endpoint and bounds.
It is not a production-readiness, broad reachability, NAT-rebinding, IPv6,
long-soak, performance-superiority or security-approval claim. No packet
payloads, keys or unrestricted packet capture were retained. The command output
also showed the intended listener readiness and final cleanup verification.

The prior failed attempt at commit `125bbcf` remains valid historical evidence
for its different candidate/configuration and is not erased by this success.
