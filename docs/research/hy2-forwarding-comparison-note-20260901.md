# Hysteria2 application-comparison seam note — 2026-09-01

Status: reviewer research support only. This is not benchmark evidence, a protocol decision, or permission beyond `docs/standing-vps-lab-authorization.md`.

## Official behavior relevant to the rented-VPS comparison

Current Hysteria 2 documentation exposes client-side `tcpForwarding`: a client can listen on a local address and forward that TCP stream through the Hysteria connection to a `remote` host/port reachable from the Hysteria server. Official client configuration reference:

- https://v2.hysteria.network/docs/advanced/Full-Client-Config/

The Hysteria server accepts an explicit listen address/port and normal TLS/auth configuration. Official server configuration references:

- https://v2.hysteria.network/docs/getting-started/Server/
- https://v2.hysteria.network/docs/advanced/Full-Server-Config/

This makes a bounded application-level paired echo experiment possible without touching the existing production Hysteria service:

```text
client workload
  -> temporary HY2 client local TCP forwarding listener
  -> temporary HY2 v2.9.3 client/server connection
  -> temporary loopback TCP echo target on the owned VPS
  -> back through HY2
```

Use a fresh temporary high UDP port for the experimental Hysteria server, a fresh local high TCP forwarding port on the client, and a fresh loopback-only TCP echo port on the VPS. Use only experiment-generated certificate/auth material under a disposable path. Do not read or reuse `/etc/hysteria/server.yaml` secrets and do not stop/reconfigure the existing Hysteria process.

The already documented installed Hysteria artifact remains the comparison target: v2.9.3 with the repository-recorded commit/hash in `docs/bench/hy2-vps-setup-20260830.md`.

## Fairness boundary

This seam gives both implementations the same *application question* (send an exact payload and receive the exact echo) on the same owned client/VPS pair and time window. It does **not** make the protocols cryptographically or architecturally identical:

- Hysteria2 uses QUIC/TLS and its own proxy/forwarding stack;
- Nekomusume uses the repository's authenticated Session/Noise path;
- handshake, framing and proxy-layer overhead are therefore part of the observed system-level result unless a later methodology deliberately separates them.

A valid paired run should fix and record payload bytes + SHA-256, route/time window, MTU metadata, security class (`authenticated encrypted research configuration`, not “same cipher”), stream/load shape, run count and finite timeout. Preserve raw samples and failures. Report median/P95/failures plus process CPU/RSS/FD via the repository resource sampler after that sampler's correctness gate is green. `wire_bytes` remains null unless a bounded capture with trustworthy metadata supplies it.

## Explicit non-options for this comparison

Do not use Hysteria port hopping or Mimic merely to create activity. Port hopping may modify firewall state, and Mimic uses privileged eBPF/XDP behavior; neither is needed for the equal-application-semantics comparison and they are outside the ordinary standing lab path.

Do not weaken the existing loopback-only guard in `scripts/bench/compare-hy2.sh`. Either build a separate self-owned-VPS orchestrator with its own fail-closed target/ownership contract or keep the existing comparison harness as a local-only fixture and reuse only its result schema/methodology.
