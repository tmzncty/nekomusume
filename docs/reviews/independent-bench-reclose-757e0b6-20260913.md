# Independent bounded benchmark re-close — exact `757e0b6`

Bounded independent re-close of `crates/neko-bench/src/main.rs` (`stat`/`sample`/`emit`, harness `main`) plus `docs/era4-i-performance.md`, at reachable exact `757e0b6c056f0bac39c1fb00e24d54cab55f3f72`, against the `era4-i-performance.v1` output shape and `scripts/bench/run-netns.sh`'s existing P95 convention. Re-closes the benchmark measurement-semantics MEDIUM. Not a WAN/superiority claim, not an independent security/release approval.

## Challenged invariants and result

- **Requested iteration clamp vs successful sample count — holds.** `NEKO_BENCH_ITERS` is clamped to `1..=10000` (`:81`); `sample` runs exactly `iters` ops; `emit`'s `iterations` reports `n` = successful samples (equal to requested count when `failures == 0`).
- **Median/P95 exact order-statistic for small/large n — holds.** `median = xs[n/2]`, `p95 = xs[round((n-1)*0.95)]` — now identical to `run-netns.sh`. `p95_order_statistic_matches_repository_convention` pins the index for `n ∈ {1,2,20,100,1000}` (e.g. n=100→94, n=1000→949).
- **Failure exclusion + empty/all-failure — holds.** Only successful ops feed `xs`; `failures` is counted separately; an empty distribution reports `n=0`, median/p95=0 without panic (`empty_distribution_reports_zero_not_panic`, `failed_operations_are_excluded_from_latency_distribution`).
- **Crypto setup/nonce across repeated samples — holds.** `crypto_seal` reuses one session and advances the nonce each call (valid repeated samples); `crypto_open` pre-generates `iters` sealed records and opens them with increasing sequence so the replay window accepts each in order — samples are genuine per-op decrypts, not a reused nonce.
- **Units/names match the timed boundary — holds.** `encode_decode`, `outer_record_encode_decode`, `crypto_seal`, `crypto_open`, `scheduler_next_frame`, `recovery_ack_loss`, `instrumentation_counter` each time exactly the named operation; `unit` is `nanoseconds per operation`.
- **Scheduler/recovery microbench rows do not imply end-to-end/WAN — holds.** Each is a local single-op fixture; the docstring and `resource_note` disclaim end-to-end/CPU-memory/WAN claims.
- **Output cannot be read as HY2 superiority — holds.** The header prints `"claims":"bounded local evidence only; no superiority claim"` and `docs/era4-i-performance.md` repeats the boundary.

## Evidence

- `cargo test -p neko-bench` on exact `757e0b6`: 3 tests pass.
- Exact-tree local provenance: `docs/local-gate-757e0b6-20260913.md` (`cargo test`, `check.sh`, `git diff --check`, clean tree, exit 0, UTC `02:49:15Z`–`02:51:15Z`).
- No remaining defect in this scope. No new `READY_LIVE` question; release/governance state unchanged.
