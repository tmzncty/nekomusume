# Developer bounded I4-PORT owner-diff review — process cleanup/signal + cross-platform + CLI + BND + release-packet

**Anchor:** exact source/test tree `0dbb931` + reviewer/dev commits through `3577489`.
**Owner-diff basis:** H-I4-089 (`26a2c40`) and H-I4-090 (`0dbb931`) changed `crates/neko-cli/tests/probe.rs` only — `process_resource_snapshot`/`signal_term`/`udp_listener_rejects` cfg gates. No other executable owner changed.

## Lane 3 — process cleanup/signal bounded challenge

`ready_failover_server`/`ready_endpoint_rebind_server` use `std::process::Child::kill()` (cross-platform Rust API); `signal_term` uses POSIX `kill -TERM` under `#[cfg(unix)]`. `finish_server` waits on `Child` with `wait()` and asserts exit. No orphan/listener leak: `endpoint_rebind_real_sockets_promote_new_source_and_reject_stale_old_source` proves stale-source rejection and new-source promotion; `sigterm_after_ready` proves SIGTERM releases TCP/UDP bindings. H-R9-082/083/084 seams (sampler exit status, `/proc/net` table parse, process-group reap, result oracle) remain closed — no new counterexample.

**Result:** no defect — process cleanup/signal paths unchanged by H-I4-089/090.

## Lane 4 — cross-platform CLI/process-test semantics owner-diff

`probe.rs` changed only in the `process_resource_snapshot`/`signal_term`/`pid`/`resource-assert` cfg gates. `#[cfg(unix)]` on POSIX signal tests is correct; `#[cfg(target_os = "linux")]` on `/proc` observation is correct. No other platform seam changed; first-RC Linux target and release policy are not broadened.

**Result:** no material owner change beyond the repaired H-I4-089/090 scope.

## Lane 5 — CLI machine-contract owner-diff

`matrix_probe`/`reachability::run`/`failover_gate`/`fail()`/`print_json` unchanged since `independent-i4-cli-machine-1b1b8c8` — exit-code/JSON/stderr contract is closed. `probe.rs` cfg-gate changes do not touch the machine contract.

**Result:** no owner change — machine contract remains closed.

## Lane 6 — CLI human-output owner-diff

`matrix_probe`/`failover_gate`/`fail()`/`print_json` unchanged since `dev-i4-cli-h-independent`/`dev-i4-cli-h-recheck`/`5d8600e` — `pass:`/`fail:` prefixed lines and exit consistency are reconciled and closed.

**Result:** no owner change — human contract remains closed.

## Lane 7 — algorithmic/resource boundedness owner-diff

`SessionRuntime`/`neko-carrier`/`neko-reliable`/`neko-cli` resource ownership unchanged since `dev-i4-bnd`/`independent-i4-bnd-recovery`/`dev-i4-bnd-synthesis` — all committed caps hold; `SessionRuntime.events` retained-history remains policy-gated.

**Result:** no owner change — boundedness remains closed.

## Lane 8 — release-packet factual-consistency challenge

`docs/release-security-review-packet.md` rows added by `dd166b3`/`7197abf`/`588ca37` label developer-local provenance distinctly from reviewer/live/performance evidence. `docs/status.md` item rows remain `candidate`/`blocked` with no promotion of Linux-only resource observation into broader Unix/security-audit/capacity/hosted-CI/production/release evidence. `docs/CHATGPT_HANDOFF.md` at `3577489` correctly flags H-I4-089/090 as test/evidence truth, not runtime/product semantics.

**Result:** no stale/falsified claim — release-packet prose is consistent.

## Lane 9 — PORT + item-4 factual reconciliation

No stale indexes/status claims require update. Evidence classes remain distinct: developer-local executable provenance (`38379b1`/`3acba04`/`d819d38`/`26aa4e8`/`42be539`/`26a2c40`/`0dbb931`), developer-local bounded review, reviewer-source-review, reviewer-local execution (none claimed), hosted CI (none), live WAN (none), performance (none). Release items 3/4 remain incomplete; `RELEASE_CANDIDATE`/`PRODUCTION_READY`/`FREEZE`/`RELEASED` unchanged.

**READY_LIVE: none** — deterministic local evidence only.
