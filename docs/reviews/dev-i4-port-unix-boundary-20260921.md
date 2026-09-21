# Developer bounded I4-PORT-UNIX review — platform-boundary challenge

**Anchor:** exact source/test tree `0dbb931` + reviewer commits through `3577489`.
**Owners inspected:** probe.rs signal/snapshot/socket fixtures, Linux sampler.

## Coverage

- **Linux-only measurement** — `process_resource_snapshot` (`/proc`) is
  `#[cfg(target_os = "linux")]`; `owned_port_sockets_present`/`read_proc`
  live only in the Linux-specific sampler. No `/proc` claim is made for
  non-Linux Unix.
- **POSIX signal/process semantics** — `signal_term` (external `kill`) is
  `#[cfg(unix)]`, so it runs on Linux/macOS where `kill` exists; it does not
  claim Windows evidence. Process-group/`setsid` semantics live only in the
  Linux sampler, not in portable CLI tests.
- **Genuinely portable socket/lifecycle** — `Child::kill()` (`std::process`)
  and `UdpSocket`/`TcpStream`/`TcpListener` (`std::net`) callers are
  un-gated: `ready_failover_server`, `reliable_udp_incomplete_settlement`,
  `ready_endpoint_rebind_server` are portable.
- **No conflation** — a `cfg(unix)` fixture may exercise the UDP churn on
  macOS but skips the Linux-only `/proc` snapshot (`cfg(linux)`); Linux
  resource evidence is never promoted into macOS/non-Linux evidence.
- **No macOS/non-Linux execution claim** — this host is Linux; the notes
  assert Linux evidence only and label `cfg(unix)` as POSIX scope, not as
  proof of non-Linux execution.

## Result

Linux-only measurement, POSIX signal semantics, and portable socket/lifecycle
behavior are correctly separated; no platform-boundary conflation, no
unsupported execution claim.

**READY_LIVE: none** — deterministic local evidence only.
