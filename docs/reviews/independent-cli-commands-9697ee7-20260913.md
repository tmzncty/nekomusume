# Independent bounded CLI subcommand truth-boundary review — exact `9697ee7`

Bounded independent spot review of command-specific factual boundaries across `crates/neko-cli/src/main.rs` runners — `capabilities`, `health_observe`, `server`, `client`, `failover_*`, `endpoint_rebind_*`, `workload`, `matrix_probe`, `key_update_fixture`, `scheduler_fairness`, `lab` — at reachable exact `9697ee7a0045b39c6ef46c1951fefea486ccf13b`. The earlier `independent-cli-portability-8e11de0` review covered portability/exit/JSON shape; this pass challenges command-specific truth boundaries (READY-before-complete, partial-result loss, exit consistency, secret-bearing paths, validation-before-side-effect). Not a live run, not an independent security/release approval.

## Challenged invariants and result

- **Config validation before side effects — holds.** `workload` validates `duration`/`concurrency`/`records`/`bytes` bounds (`:3596-3607`) **before** spawning workers (`:3610`); `matrix_probe`/`health_observe`/`capabilities` parse and bound all args before any socket work. No runner performs a side effect then validates.
- **Success/READY cannot precede actual completion — holds.** `workload` asserts `confirmed_watermark == records*bytes` per worker before reporting `cleanup=verified` (`:3629-3657`); `matrix_probe` derives `reachable` from the real `reachability::run` result and only then prints/exits. `lifecycle_state=STOPPED readiness=false` strings reflect the actual lifecycle after `emit_signal_shutdown`.
- **Partial results are not silently lost — holds.** Worker `join` failures call `fail` (`:3641-3643`) rather than reporting a partial total as success; `matrix_probe` exits `1` on any non-reachable artifact.
- **Exit-code consistency — holds.** `fail` is uniformly `stderr + exit(2)`; success paths exit `0`; the gate/`matrix_probe` exit `0`/`1` matches the reported `reachable` truth.
- **Secret-bearing debug/error paths — holds.** Private key material only reaches the owner-only identity file; `println!`/`eprintln!` surfaces carry public keys, counters, and stage labels — never secrets.
- **Machine vs human truth — holds.** `--json` paths emit pure machine artifacts; human `喵~！`/`喵呜呜呜呜…` markers are confined to the non-JSON branch and never contaminate machine output or the exit code.

## Evidence

- Static review of the runner/output surfaces on exact `9697ee7`; `scripts/check.sh` keeps the CLI runner regressions green (`failover_gate_arguments_are_bounded_and_loopback_explicit`, `workload_duration_boundary_is_strictly_bounded`, `health_observe_arguments_are_bounded_and_parseable`).
- No code change; no defect found. No new `READY_LIVE` question; release/governance state unchanged.
