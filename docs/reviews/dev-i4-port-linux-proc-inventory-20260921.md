# Developer bounded I4-PORT-LINUX review — `/proc` owner/callsite inventory

**Anchor:** exact source/test tree `0dbb931` + reviewer commits through `3577489`.
**Owners inspected:** `crates/neko-cli/tests/probe.rs` `process_resource_snapshot`
(`/proc/<pid>/{fd,status}`), `signal_term` (POSIX `kill`), all test callers;
`scripts/bench/process-resource-sampler.py` `read_proc`/`owned_port_sockets_present`
(`/proc/<pid>`, `/proc/net`).

## Per-invariant coverage

| Invariant | Coverage |
|---|---|
| no hidden broad `cfg(unix)` on Linux-only `/proc` access | `process_resource_snapshot` is `#[cfg(target_os = "linux")]`, not `cfg(unix)` — `/proc` is Linux-only; macOS `cfg(unix)` builds compile it out. Its callers (`udp_listener`) are `#[cfg(unix)]` for the UDP-churn surface but the snapshot call sites are `#[cfg(linux)]`, so a macOS run skips the `/proc` observation rather than false-passing |
| no optional-observation false-pass | on Linux the snapshot `assert!(matches!((before,after),(Some(_),Some(_))))` — observation must be affirmative, not silently skipped; `Option` is only used to compose the assert, not to pass absent data |
| no partial-table promotion | `owned_port_sockets_present` returns `None` (unknown) when any required `/proc/net` table is unreadable or a row is unparseable — never promoted to absent |
| Linux sampler vs portable CLI vs hosted CI distinguished | `process-resource-sampler.py` is explicitly Linux (`/proc/<pid>`, `/proc/net`, process groups); CLI `signal_term`/churn fixtures are `#[cfg(unix)]` POSIX; `Child::kill()`/`UdpSocket` callers are portable `std::process`/`std::net` |
| no unsupported-platform invocation | `/proc` reads exist only under `#[cfg(target_os = "linux")]` (Rust) or in the Linux-specific sampler script |

## Reachable regressions named

- `udp_listener_rejects_bounded_malformed_churn_then_authenticates_and_cleans_up`
  — `#[cfg(unix)]` test, `#[cfg(linux)]` snapshot, affirmative `Some(_)` assert.
- `sigterm_after_ready`/`periodic_server_signal_cleanup`/`endpoint_rebind_*`
  — `#[cfg(unix)]` POSIX signal fixtures.
- sampler `owned_port_sockets_present` partial-observation `None` regressions.

## Result

No hidden broad `cfg(unix)`, no optional-observation false-pass, no
partial-table promotion, no unsupported-platform `/proc` invocation. Linux
`/proc` ownership is correctly `cfg(linux)`; POSIX signal fixtures are
`cfg(unix)`; portable `std` APIs stay un-gated.

**READY_LIVE: none** — deterministic local evidence only.
