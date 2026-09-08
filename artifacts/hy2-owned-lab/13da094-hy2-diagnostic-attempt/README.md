# Exact `13da094` HY2 diagnostic-seam attempt

- Invocation count: exactly one, after exact `13da094` passed exact-head CI run `34249324406`.
- Scope: self-owned client and VPS only; temporary pinned Hysteria v2.9.3 process; existing production Hysteria service was not touched.
- Contract: 1200-byte payload, payload SHA-256 `655a3ef0465a9f30fddf25f4dde0c19a05c6f9069b83961800c1944165955273`, five-run bound, fresh transport/resource-sampler lifecycle.
- Valid prefix: `nekomusume-1` succeeded with 1200 application bytes; `hy2-1` exited nonzero before application bytes.
- Result: `BLOCKED_HARNESS` at `hy2-1-failed`; no complete pair, summary, comparative statistic, performance or superiority claim.
- HY2 typed diagnostic: category `unknown`, baseline `last_success_stage=client_started`, `last_success_source=harness`, bounded private bundle 699 bytes, SHA-256 `bad8734ebcd73637d64ac10c155b22110d5dd9931463f19f801b825b786673c3`.
- Result artifact SHA-256: `dffab760a660b3ce9addcbd13bedb15f0b0a8f58624088da000c178de0de14cb`.
- Cleanup: verified; local/remote listeners 0, local processes and remote process groups reaped, remote temporary path removed.

The `unknown` category and baseline stage are an evidence boundary, not a root-cause diagnosis. Stderr prose did not prove TLS/auth/path success. This consumes the one materially changed hypothesis attempt; no same-class retry is permitted without a new material code/config/path hypothesis. Private diagnostic content is not tracked.
