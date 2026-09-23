# H-I4-110 — canonical failover client remains an unbounded process owner

**Severity:** HIGH — release/item-4 test-harness correctness

**Reviewed repository anchor:** `f4916c52aa61ee9b0da7bb17195ba14c0f4de011`

**Exact inspected owner:** `crates/neko-cli/tests/probe.rs` (blob `3d6c445f6ba61b6a5940e0275f280edb2907035f` on the reviewed `main`)

## Finding

H-I4-107/109 hardened the bounded process-output helper, and H-I4-108 converted many real networked `client` / `failover-client` process owners to `bounded_client_output`. The exact-current owner sweep is not complete, however.

`executable_loopback_controlled_udp_stop_tcp_resume` still has this shape:

```text
spawn real failover server with piped output
  -> prove server ready
  -> Command("failover --role client ... --duration 3").output()
  -> only after client returns: finish_server(server)
```

The direct synchronous `.output()` is an independent process owner. If the canonical failover client regresses and remains live during connect, negotiation, Noise/authentication, framed I/O, DeliveryAck/resume, or shutdown, the test thread cannot reach the already-bounded `finish_server` cleanup. The product `--duration 3` is part of the behavior under test; it is not an outer harness deadline and cannot prove that a lifecycle regression terminates.

This gives a concrete counterexample to the broad H-I4-108 closure claim: a client process can remain live with the test blocked inside `.output()`, while both server cleanup and failure assertions are unreachable.

This is **not** a production Session/Carrier/ACK/crypto/wire semantic defect. It is a test-process ownership defect that can turn a release/item-4 negative into an indefinitely hung gate.

## Closure contract

1. Convert the canonical `failover --role client` in `executable_loopback_controlled_udp_stop_tcp_resume` to the existing exact-current `bounded_client_output` owner rather than introducing another process framework.
2. Preserve the current product arguments, protocol assertions, diagnostics, server readiness path, and `finish_server` semantics. The local harness deadline must exceed the product `--duration`; the current H-I4-108 rule is product duration plus bounded slack (for this owner, `3s + 5s` is the direct existing pattern), without creating a new repository-wide policy value.
3. Continue an owner-by-owner causal sweep of remaining direct `.output()` / `wait` / `wait_with_output`, socket blocking reads/accepts, drain threads and joins in process tests. Convert only real networked/live owners lacking an independent bound. Do **not** mechanically rewrite keygen, help/capabilities, invalid-argument/invalid-configuration or other source-proven fail-fast commands.
4. A second generic `sleep 30` regression is not required merely for coverage: `bounded_client_output_fails_when_client_never_exits` already proves the shared helper's ordinary timeout path. Add a focused regression only if this owner exposes a distinct failure shape.
5. No decoder/parser/crypto-framing implementation is implicated; do not run fuzz mechanically.
6. On the final pushed source/test SHA, run `PYTHONDONTWRITEBYTECODE=1 bash scripts/check.sh`, `git diff --check`, confirm a clean tree, and persist developer-local exact-tree provenance with reachable SHA, UTC start/end, exit codes, OS/arch and stable Rust.
7. Continue immediately to the remaining ownership sweep and then the retained rolling queue; this finding is not repository-wide queue exhaustion.

## Evidence boundary

This note is reviewer source/control-flow inspection only. It does not claim reviewer-local Rust execution, a full local gate, hosted CI, cross-platform execution, fuzz, WAN evidence or performance evidence. The developer-reported exact-tree provenance for H-I4-107/108/109 at `4c445ce36aa06edc1b3d4952036073865e3f20d0` remains a distinct evidence class; this finding narrows only the completeness of the H-I4-108 owner inventory.

`READY_LIVE` remains `none`; release items 3 and 4 remain incomplete; no release/governance flag or policy value is changed.
