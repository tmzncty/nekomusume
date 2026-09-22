# Developer bounded independent cross-platform CLI/process review

**Anchor:** exact source/test tree `b4af007` + reviewer/dev commits through `75f4d9c`.
**Owners inspected:** `crates/neko-cli/tests/probe.rs` (`ReadyServer`, `ReadyProof`, `BarrierProof`, `wait_for_ready_marker`, `bounded_reap_or_kill`, `malformed_classification_barrier`, `signal_term`, process launch/finish helpers, target-gated tests); `crates/neko-cli/src/main.rs` (`signal_hook`/`libc` signal registration, `/proc/self/fd` benchmark diagnostic, `read_identity`/`load_or_generate` Unix permission paths); `crates/neko-cli/Cargo.toml`; portability/release claims in `docs/status.md`, `IMPLEMENTATION_PLAN.md`, `docs/release-security-review-packet.md`, prior independent CLI portability notes.

## Per-invariant coverage

| Invariant | Coverage |
|---|---|
| Linux-only `/proc` measurements fully `cfg(target_os = "linux")`, fail affirmative on Linux | `process_resource_snapshot`/`gated_resource_snapshot` and their callers' snapshot blocks are `#[cfg(target_os = "linux")]`; on Linux the snapshots must be `Some` (fail closed on `None`). `main.rs` `/proc/self/fd` benchmark diagnostic is a runtime-only `fd_count` field: on non-Linux `fs::read_dir` returns `Err` → `fail("benchmark FD inventory unavailable")` → exit 1 (fail closed, not a silent pass). First-RC target is x86_64 Linux; no non-Linux execution claim is made. |
| Unix-specific commands/signals/shell syntax target-gated | `Command::new("sleep")`, `Command::new("sh")` (`exec 1>&-`), `Command::new("kill")` and `signal_term` are all inside `#[cfg(unix)]` tests; `wait_for_ready_marker`/`bounded_reap_or_kill`/`ReadyProof`/`BarrierProof` are target-neutral (pure `std::process`/`mpsc`/`BufReader`, no Unix API) — name-resolvable on non-Unix. |
| Child ownership converges on success + every failure shape | `wait_for_ready_marker` kills+reaps via `bounded_reap_or_kill` on timeout/EOF/read-error/disconnect; `malformed_classification_barrier` kills+reaps on timeout/disconnect/EOF and joins the reader; `finish_server` waits only after stdout drained. No unbounded `wait()` on an unproven-exited child; no reader left owning the only pipe endpoint. |
| Shared readiness helpers preserve exact markers/log semantics | `wait_for_ready_marker(child, marker, timeout)` takes the exact per-helper marker string (`lifecycle_state=READY readiness=true`, `"event":"start"`, `endpoint_rebind_server_ready`) — refactor cannot weaken a marker into a different readiness. |
| Negative process tests bounded, platform-honest | `ready_endpoint_rebind_bounded_when_child_is_silent` / `..._closes_stdout_but_stays_alive` / `start_server_for_bounded_when_binary_exits_silently` are `#[cfg(unix)]` with `elapsed < 10s` asserts — bounded and honest about Unix assumptions. |
| No Windows/macOS/BSD execution inferred | Only Linux x86_64 provenance is claimed; first-RC target remains x86_64 Linux. |

## Commands/tests actually run

- `cargo test -p neko-cli --test probe` — 15 process/readiness/malformed tests pass (incl. three new negative regressions)
- `cargo clippy -p neko-cli --all-targets -- -D warnings` — clean
- `PYTHONDONTWRITEBYTECODE=1 bash scripts/check.sh` at `b4af007` — exit 0 (persisted in `docs/notes/i4-cli-proc-096-provenance-b4af007-20260922.md`)

## Exclusions

- No macOS/Windows/BSD execution claim; no hosted CI claim.
- `/proc/self/fd` runtime diagnostic unchanged — first-RC Linux target keeps the fail-closed `Err` path; no `cfg` widening is a defect under current committed target policy.
- No WAN/performance/security approval; no release-flag change; no D019/TTL/LRU/capacity value.

## Result

No concrete defect found in the challenged cross-platform CLI/process-test
scope. Linux `/proc` gates are correctly `cfg(target_os = "linux")`, Unix
process fixtures are correctly `cfg(unix)`, proof markers are target-neutral,
child ownership converges on all failure shapes, and negative regressions are
bounded.

**READY_LIVE: none** — deterministic local evidence only.
