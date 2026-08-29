# Bounded VPS failover attempt — 2026-08-29

## Authorization and bounds

The administrator authorized exactly one bounded observation against the dedicated VPS `192.144.192.215`. The candidate was commit `35f6593`, with TCP port `40080`, UDP port `40081`, three records of 16 bytes, and a maximum duration of 10 seconds. No second attempt, scan, proxy, tunnel, or production operation was performed.

## Candidate integrity

The release binary SHA-256 was `bce999e3d64f42d76eb99f8fcfe0fabd67a62d8b687e1949481b08a1fe2dbbe7` on the controlled client and VPS.

## Result

- Start: `2026-08-29T21:48:18+08:00` client; `2026-08-29T21:48:22+08:00` VPS.
- Client exit code: `2`.
- Client stderr: `neko: UDP handshake timeout`.
- VPS server stderr: `neko: failover timeout`.
- No successful cross-carrier WAN failover is claimed.

## Cleanup

The VPS listener check during the run showed only the intended `0.0.0.0:40080/tcp` and `0.0.0.0:40081/udp` listeners. After termination, `ss` showed no listener on either port and no test process remained. Temporary runtime files and identities were removed from the client and VPS after evidence capture. No firewall changes were made.

This is a failed closed observation, not a reachability, interoperability, performance, security, or production result. Public WAN retries require fresh explicit authorization.
