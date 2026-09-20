# Developer bounded I4-PORT re-challenge — cross-platform CLI/process-test semantics

**Anchor:** exact source/test tree `26aa4e8` + reviewer/dev commits through `e8fbc65`.
**Prior note superseded where falsified:** `dev-i4-port-cross-platform-cli-process-20260921.md` (`11678b2`) — its platform-guard, `/proc` fail-closed, `signal_hook` cross-platform, and temp/runtime cleanup claims remain valid on the current tree; its Session/Carrier delivery-evidence claims are superseded by H-I4-086/087/088 repairs.

**Owners inspected:** `neko-cli` `main.rs` `read_identity`/`load_or_generate`
(`#[cfg(unix)]` `O_NOFOLLOW`/`O_CLOEXEC`/permissions), `fd_count` `/proc/self/fd`
fallback, `signal_hook` SIGTERM/SIGINT; `process-resource-sampler.py` `/proc`/
`setpgid`/`killpg`/`reap_group_children`; `owned-lab-control-plane.sh` `ps`/`ss`/`kill`.

## Per-invariant coverage

| Invariant | Coverage |
|---|---|
| platform guards are explicit, not implicit | `#[cfg(unix)]` wraps `O_NOFOLLOW`/`O_CLOEXEC` open flags and owner-only permission checks — on non-Unix the open still works but the owner-only check is skipped rather than fabricated |
| Linux `/proc` usage is fail-closed | `fd_count` reads `/proc/self/fd` and returns `None` on `read_dir` failure; the sampler reads `/proc/<pid>/stat` + `/proc/net/{tcp,tcp6,udp,udp6}` — any unreadable required table returns `None` (unknown), never promoted to absent/success |
| signals/process groups/setsid semantics | `signal_hook` registers SIGTERM/SIGINT cross-platform; the sampler uses `os.setpgid`/`os.killpg`/`waitpid(-pgid)` — a documented Linux/POSIX scope — with `reap_group_children` WNOHANG drain and TERM→KILL escalation; `setsid()`-escaped descendants are caught by the independent owned-port oracle (H-R9-082/083) |
| temp/runtime cleanup assumptions | sampler result write is `tmp.write_text → chmod 0600 → os.replace`; `owned-lab-control-plane` cleanup uses `ps`/`ss`/`kill` with bounded attempts; `fixture.timeout-group`/`escaped-*`/`partial-obs` prove truthful `complete=false` + nonzero exit |
| deterministic unsupported-platform behavior, no false pass | non-Unix builds compile (no `build.rs`, no `std::os::unix` outside `#[cfg(unix)]`) but the owner-only identity check is skipped — the file is still read and must parse; `/proc`-dependent sampler/olcp code is scoped to Linux bench owners, not the protocol path |
| Session/Carrier delivery evidence kept separate from platform/process evidence | H-I4-086/087/088 repairs do not change the platform-surface claims; the Session `sent`/`aggregate_records` bookkeeping and MemoryCarrier record cap are owner-internal, not platform evidence |

## Reachable regressions named

- `sigterm_after_ready_stops_and_releases_tcp_and_udp_bindings`
- `periodic_server_signal_cleanup_is_bounded`
- `fixture.timeout-group` / `fixture.escaped-descendant` / `fixture.escaped-udp` / `fixture.partial-obs` / `fixture.exit-race`
- `invalid_bind_never_emits_ready`
- `empty_messages_are_bounded_by_record_count` (H-I4-088 record-cap regression)

## Result

No concrete defect found across cross-platform CLI/process-test semantics on
the current tree. The prior note's platform-surface claims stand; the
Session/Carrier delivery-evidence claims superseded by H-I4-086/087/088 are
owner-internal bookkeeping changes, not platform evidence.

**READY_LIVE: none** — deterministic local evidence only.
