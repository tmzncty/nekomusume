# Reviewer Follow-up D — bounded VPS negotiated failover evidence

- experiment_id: `followup-d-vps-20260901`
- endpoint labels: `self-owned-client` ↔ `self-owned-vps` (addresses intentionally redacted)
- authorization: standing VPS lab authorization; temporary unprivileged high ports only
- start (UTC): `2026-09-01T00:46:41Z`
- end (UTC): `2026-09-01T00:46:42Z`
- duration observed: ~1s; configured max duration 5s
- GitHub `main` at setup: `12e918affba7a999dfa14c98d5391521c30c6f46`
- binary: workspace `neko-cli`, package version `0.1.0`, Linux x86_64; built with `cargo build --locked --package neko-cli --bin neko-cli`
- parameters: count=3; payload=16 bytes/record; total application payload=48 bytes; UDP/TCP temporary high ports in authorized 40080–40100 range (actual labels redacted); max_seconds=5; concurrency=1
- capture: metadata-only; no pcap, keys, identity material, or payload bytes committed

## Real-socket result

PASS. The bounded real-socket run completed with client exit code 0 and server completion. Structured output recorded:

- client: `udp_authenticated`, one UDP datagram sent and one ACK observed;
- client: explicit `controlled_udp_stop` with reason `bounded_application_fault_injection` (not a claim of natural WAN blackhole/PTO detection);
- client: `tcp_resume_guard`, `ordered_records_complete count=3`, and `failover_client_ok ... payload_bytes=48`;
- server: `udp_negotiated version=0`, `udp_authenticated`, one UDP ACK sent;
- server: `tcp_negotiated version=0`, `tcp_resumed`, `failover_server_ok records=3 payload_bytes=48`;
- server summary: `classification=A`, `records=3`.

This demonstrates canonical negotiation and authenticated admission on initial UDP, then fresh canonical negotiation plus authenticated ResumeBinding/ResumeGuard on TCP resume in the executable controlled endpoint-fault path. It does **not** demonstrate natural WAN loss detection or a configured ACK/PTO threshold: the current runner exposes the truthful controlled-stop seam.

## Negative and regression evidence

All five targeted tests passed on this exact worktree/binary source:

- unsupported selected version rejected before Noise/data;
- malformed/unsupported/duplicate negotiation rejected before echo;
- negotiation transcript mismatch rejected before application echo;
- UDP handshake timeout reports the last successful stage;
- deterministic UDP diagnostic stages.

The existing executable loopback controlled-stop/resume test also passed with non-default count=3, bytes=16, UDP/TCP ports 40089/40090, and bounded durations. The process-boundary exactly-once test was not re-run because its source/test contract is unchanged and the current VPS run already exercised the bounded ordered completion path; no unchanged failure is being retried.

## Environment and cleanup

- host observed: Linux 6.8 x86_64; unprivileged user `tmzn`.
- pre-existing listeners were inspected; no production listener was altered.
- unprivileged user/network namespace setup was attempted and is unavailable: `unshare -Urn true` failed with `Operation not permitted` (exact local capability blocker); no netns/veth claim is made.
- temporary identity files, logs, and process were removed by a trap cleanup path.
- post-run socket check found no listener on the experiment ports; no experimental process remained.
- no code or production configuration was changed.

## Evidence integrity

Only this small redacted summary is committed. Raw stdout/logs and identity files were disposable and were not copied into the repository. No secrets, IP addresses, packet captures, or raw payloads are included.
