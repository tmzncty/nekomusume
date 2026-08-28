# Crypto dependency review — 2026-08-29

## Scope

Bounded local/loopback research implementation only. This review does not
constitute a security audit, protocol freeze, or production approval.

## Candidate selection

| Dependency | Version observed | License | Use | Decision |
|---|---:|---|---|---|
| `snow` | 0.10.0 | Apache-2.0 OR MIT | Noise Framework handshake and cipher state | Select for the research slice; keep the exact pattern and wire contract in repository code/tests |
| `chacha20poly1305` | 0.11.0 | Apache-2.0 OR MIT | Direct AEAD record protection if needed outside Noise | Do not add yet; avoid duplicate crypto surface until the Noise cipher API seam is exercised |
| `blake3` | 1.8.7 | CC0-1.0 OR Apache-2.0 OR Apache-2.0 WITH LLVM-exception | Not required by the selected Noise candidate | Do not add |

`cargo info` on the DATA5 host reported Rust 1.85 minimum for `snow` and
`chacha20poly1305`; the host compiler is Rust 1.98. `snow` 0.10.0 exposes the
required default resolver with Curve25519, ChaChaPoly and SHA-256 support and
is maintained upstream at `github.com/mcginty/snow`. The project license
expression `MIT OR Apache-2.0` is compatible with the selected dependency's
license expression. The lockfile is committed and `--locked` checks remain
mandatory.

## Constraints carried into implementation

- Select one explicit Noise pattern and test both roles; no ad-hoc handshake.
- Keep prologue and per-record AAD as separate byte domains.
- Keep authentication separate from authorization.
- Keep nonce direction, replay, epoch/key phase and resource limits explicit.
- Do not expose a non-loopback listener or add production runtime behavior.
