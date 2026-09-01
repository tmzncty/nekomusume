# Reviewer Follow-up B — historical periodic Session VPS evidence and re-port validation

## Scope and provenance

This note preserves a **historical execution** of the periodic TCP Session runner from source commit `d5f0170911bd90cd952625279658313dfa306eec`. It does not claim that the five-minute sample was rerun on the current parent.

- historical source parent: `ed08b644b6cc88ca2b322fd09b3f971d604f791c`
- historical source commit: `d5f0170911bd90cd952625279658313dfa306eec`
- historical release binary SHA-256: `64546b8ab1b5cb53a50070d2de083c111087b7b2212fc4b25fe9044805f2022c`
- re-port parent: `41eace324dd8cf139f3946ab7ace4e6ad726f956`
- endpoints/path: two self-owned Linux hosts over an existing private IPv4 tunnel path; addresses and disposable identities are omitted
- authorization/bounds: standing VPS lab authorization; TCP port 40080; concurrency 1; duration at most 300 s; 60 records at 32 B every 5 s; 1,920 application bytes; no firewall, route, qdisc, namespace, capture, or production-service change

The runner used one TCP connection, one negotiated and Noise-authenticated logical Session `7201`, and stream `1`. Each application record was confirmed by a real encrypted Session `DeliveryAck`; reconnect/resume remained explicitly unsupported and fail-closed.

## Historical five-minute result

The sole actual Session after correcting the sampler output path completed as follows:

```text
client: attempted=60 confirmed=60 missing=0 duplicates=0 reconnects=0
client: p50_confirmation_latency_ms=40 p95_confirmation_latency_ms=41
client: elapsed_ms=295041 application_bytes=1920 cleanup=verified
server: authenticated=true received=60 confirmed=60 duplicates=0 elapsed_ms=295148 cleanup=verified
```

Both processes exited successfully and authenticated Session `7201` / stream `1`. This is an exact `60/60` confirmation result, not a throughput or public-reachability claim.

Direct-child process sampler measurements (296 samples per role at 1 s intervals):

| role | CPU user s | CPU system s | max RSS KiB | peak FD | peak owned sockets |
|---|---:|---:|---:|---:|---:|
| client | 0.143040 | 0.272507 | 10176 | 4 | 0 |
| server | 0.035968 | 0.035968 | 9968 | 5 | 1 |

Both samplers exited 0 without timeout, reaped their direct child, and reported complete cleanup. The first sampler invocation used a relative output path and failed closed **before child start**; that failure was preserved, then the configuration was corrected to a new absolute output path before the sole actual Session. Temporary binaries, identities, raw logs, and sampler JSON were removed; no experiment process or TCP 40080 listener remained.

## Re-port and current-parent validation

The implementation and deterministic tests were reapplied onto exact parent `41eace324dd8cf139f3946ab7ace4e6ad726f956` without altering its health-truth repairs: bounded duplicate admission, explicit event-based health evidence without fabricated telemetry, D064 threshold/reason and CarrierManager decision ownership, and measured cold-fallback timing/classification remain intact.

Validation on the re-port is intentionally separated from the historical execution:

- a minimal current-parent loopback periodic Session smoke/test validates negotiation, Noise authentication, encrypted `DeliveryAck`, delayed/missing/duplicate ACK accounting, bounds, reconnect fail-closed behavior, and signal cleanup;
- the full repository gate validates formatting, all targets, tests, Clippy with warnings denied, repository scripts, and diff hygiene;
- fuzz is not rerun because this re-port does not change wire format or parser behavior.

The historical five-minute sample above was not repeated unchanged.
