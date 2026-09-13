# Independent bounded CLI portability + exit/JSON review — exact `8e11de0`

Bounded independent static review of `crates/neko-cli/src/main.rs` for OS/portability boundaries, exit-code semantics, and human-vs-machine output separation, at reachable exact `8e11de0e5915efa43591069172b97f311aae2c6e`. Not a live WAN run, not signing/operator policy, not an independent security/release approval.

## Challenged invariants and result

### Portability surface (RL11)

- **cfg(unix) boundaries are complete — holds.** Every `std::os::unix`/`libc` use sits inside `#[cfg(unix)]` blocks (`:321-324`, `:337-342`, `:363-367`, `:375-379`) — `O_NOFOLLOW`/`O_CLOEXEC`, `mode(0o600)`, `permissions().mode() & 0o077`, `set_permissions(0o600)`. No bare unix-only call escapes the cfg; on non-unix the safety checks compile out but the fail-closed `open`/`metadata`/`read`/`fail` paths remain.
- **Identity file atomicity/permissions — holds.** `load_or_generate` uses `create_new(true)` (no overwrite), `mode(0o600)`, `AlreadyExists → re-read` for the create race, `set_permissions(0o600)`, `write_all`, then `sync_all` (fsync) before returning — durable, owner-only, race-safe.
- **Read path is fail-closed — holds.** `read_identity` rejects non-regular files, requires owner-only mode on unix, `O_NOFOLLOW` blocks symlink following; every failure path calls `fail` (stderr + exit), never a silent default.
- **Signal handling is a cross-platform abstraction — holds.** `signal_hook::flag::register(SIGINT/SIGTERM)` drives a shared `AtomicBool` shutdown flag; `emit_signal_shutdown` reports `lifecycle_state=STOPPED readiness=false` truthfully. `signal_hook` abstracts the platform signal mapping; no raw `libc` signal call.
- **Listener/socket surfaces use portable APIs — holds.** `TcpListener::bind`/`UdpSocket::bind`/`Shutdown` are std cross-platform; `fail("bind failed")` on error is fail-closed, no silent fallback.

### Exit / JSON / human-vs-machine surface (RL12)

- **Fail-closed exit codes — holds.** `fail(msg)` → `eprintln!("neko: {msg}")` + `process::exit(2)`: errors go to stderr (never mixed into machine stdout) with a non-zero exit. `failover` gate exits `0` only when the artifact reports top-level `"reachable":true`, else `1` (`:3776-3780`).
- **Human vs machine authority separated — holds.** The cat markers `喵~！`/`喵呜呜呜呜…` are emitted **only** on the human (non-`--json`) path (`:3771-3775`); the `--json` path prints the pure machine artifact. Human text never contaminates machine JSON.
- **Success JSON timing — holds.** `--json` output is emitted once, complete, before `process::exit`; the human summary is a separate branch. READY/DRAINING/STOPPED stage strings reflect actual lifecycle transitions (e.g. `lifecycle_state=STOPPED readiness=false`).
- **Secret-safe stderr — holds.** Private-key hex is only written to the owner-only identity file; no secret material reaches `println!`/`eprintln!` beyond public keys and non-secret fields.

## Minor observation (not a defect)

`failover_gate`'s exit decision uses `artifact.contains("\"reachable\":true")` — a string-contains on the serialized fixture JSON. For this fixture's fixed top-level `reachable` field that is reliable, but it would misclassify if the schema ever nested or string-escaped the field. Recorded for awareness only; not a contract defect on the current shape.

## Evidence

- Static review of the CLI portability/output surface on exact `8e11de0`; `scripts/check.sh` keeps the CLI/fixture regressions green.
- No code change; no defect found. No new `READY_LIVE` question; release/governance state unchanged.
