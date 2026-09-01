# Follow-up B bounded process evidence (2026-09-01)

> **Supersession note (2026-09-01):** This pre-`f680702` run retains its valid canonical-negotiation, authenticated-admission/resume, ordered server-receive, bounded-execution, and cleanup facts. Its acknowledgement wording did **not** prove authenticated, exact-semantic Session `DeliveryAck`; that claim is disavowed. `f680702` is the implementation repair, and only post-fix evidence may support authenticated Session-delivery acknowledgement claims.

This is a small, redacted evidence summary. It intentionally contains no
identity material, keys, payload bytes, raw logs, addresses beyond loopback,
or packet capture.

## Provenance and limits

- Authoritative remote check: `git ls-remote origin refs/heads/main` returned
  `12e918affba7a999dfa14c98d5391521c30c6f46`.
- Evidence worktree: fresh detached worktree at that exact parent.
- Binary: `neko-cli`, built from that exact parent.
- Profile: loopback only; UDP port `40091`, TCP port `40090`; `count=3`,
  `bytes=37` per record, `duration=8s` maximum. Total application payload was
  111 bytes, within standing authorization limits.
- Diagnostic experiment id: `fub-process-20260901` (only the id is retained).

## Process/loopback result

A real server process and client process were run concurrently with generated
throwaway identities. Both exited after the bounded scenario and the listener
check showed no remaining listener on either port.

Redacted diagnostic/event summary, in observed order:

1. Both roles emitted `start` with the requested bounded parameters.
2. Client/server emitted `udp_hello_sent` / `udp_hello_received`.
3. Server emitted `udp_negotiated` and `udp_authenticated`; client emitted
   `udp_authenticated`.
4. One UDP data exchange and acknowledgement completed.
5. Client emitted `controlled_udp_stop` with the explicit bounded fault-injection
   reason (not a claim that the UDP socket failed spontaneously).
6. Client/server completed `tcp_negotiated` and `tcp_resume_guard` /
   `tcp_resumed` at generation 1.
7. Three ordered records completed; both summaries classified the run `A` and
   reported 3 records / 111 payload bytes.
8. Client emitted metadata-only capture diagnostics with payload and keys
   excluded; no capture file was retained.

The server's success line also reported the same negotiated/authenticated /
controlled-stop/resumed event chain. No secret or raw payload is retained here.

## Negative pre-data and unit/process evidence

Passed on this exact parent:

- `cargo test -p neko-carrier --test process_runner --test resumed_session -- --nocapture`
  (4 tests passed, including process-boundary UDP blackhole/TCP resume and
  ResumeGuard exactly-once behavior).
- `cargo test -p neko-cli --test probe -- --nocapture`
  (10 tests passed, including real loopback controlled stop/resume and
  diagnostics).
- Targeted negative tests for unsupported selected version, malformed /
  duplicate negotiation, and transcript mismatch (all passed). These assert
  rejection before Noise/application echo.
- `./scripts/check.sh` (passed).

## Namespace attempt and cleanup

`unshare -n true` was attempted once and was blocked by the host with
`Operation not permitted`; no namespace/veth was created. This is an exact
environment blocker, not a protocol result. Loopback was therefore the
strongest supported process-level evidence in this run. Temporary identities,
processes, listeners, and runtime files were removed or terminated; a final
socket check found no listener on ports 40090/40091.
