# Exact-tree check.sh provenance — 66c9ecb (2026-09-29 UTC/local)

Developer-local exact-tree gate on the final reachable pushed SHA for the
failover / endpoint-rebind `--bytes` bound fix.

- **SHA:** `66c9ecb44ae532dfc83f85234ef73411746668b2`
  (`fix(cli): cap failover / endpoint-rebind --bytes at the sealable Data payload (1170, derived)`)
- **Command:** `PYTHONDONTWRITEBYTECODE=1 bash scripts/check.sh`
- **Tree state:** clean detached worktree (`/tmp/nmFxG`) created from the literal
  pushed SHA `66c9ecb` (worktree head asserted equal to it); `git diff --check` exit
  0; `git status --short` empty before and after
- **UTC start:** 2026-09-29T06:05:51Z (log `/tmp/nm-fx-gate.log`, 1266 lines)
- **UTC end:** 2026-09-29T06:12:16Z
- **Exit code:** 0 (host load average ~8.1 from unrelated jobs)
- **OS / arch:** Linux 6.8.0-137-generic, GNU/Linux (x86_64)
- **Rust toolchain:** `rustc 1.98.0 (88d9e12ae 2026-08-18)`

Local and UTC dates are both 2026-09-29. Passed on the **first** attempt;
`run-periodic-command-test.py` printed `periodic command tests: ok`.

## Pre-commit sequence (strictly serial)

`cargo fmt --all --check` 0, `clippy --workspace --all-targets -D warnings` 0,
`cargo test --workspace`: 42 suites, **726 passed**, 0 failed
(2026-09-29T06:00:37Z-06:04:54Z, load average ~7.6). Commit, push, remote
verification and the gate were separate tool invocations. The gate asserted the
literal SHA.

## The bug

`failover` and `endpoint-rebind` bounded `--bytes` at the literal 1200
(`MAX_BYTES = MAX_UNRELIABLE_DATAGRAM`), measured against the user payload. Both
wrap the payload in a `ProcessMessage::Data` frame, which adds 30 bytes of header,
before calling `seal_unreliable`. That call caps the **encoded frame** at 1200. As a
result, `--bytes` 1171-1200 passed validation, and then:

- the client panicked on a `SessionRejected` unwrap (failover `main.rs:2616`,
  endpoint-rebind `main.rs:4682`; exit 134);
- the server waited out its duration (exit 2, or was killed).

This was found while measuring the PLPMTUD base datagram (D067) and was bisected to
1170 ok / 1171 fail on both commands.

## The fix (option (a), approved by 85461)

- `neko_session::PROCESS_DATA_HEADER_LEN` = 30, defined next to the encoder.
- The CLI bound is `MAX_DATA_FRAME_BYTES = MAX_UNRELIABLE_DATAGRAM -
  PROCESS_DATA_HEADER_LEN` (= 1170). It is derived, not a second literal.
- The bound is used by both failover roles and both endpoint-rebind sides, and the
  messages are generated from it.
- The two client seal sites exit via the typed `fail` path (exit 2) instead of
  panicking. With the new bound this is unreachable from CLI input; it is defensive
  only.
- The frame cap and wire semantics are unchanged. Option (b), widening the frame
  cap, is not done: it would change what "base" means for PLPMTUD (candidate 2).

## Every `1-1200` site checked (85461's request)

| Site | Command | Payload path | Verdict | Evidence |
|---|---|---|---|---|
| `main.rs:1174` | `failover` server | `ProcessMessage::Data` → `seal_unreliable` | **affected → fixed** | real pair: 1170 ok; 1171/1200 now exit 2 at validation |
| `main.rs:2461` | `failover` client | same (seal at `:2616`) | **affected → fixed** | same |
| `main.rs:4237` | `endpoint-rebind-server` | same | **affected → fixed** | real pair: 1170 ok; 1171/1200 previously panicked at `:4682`, now exit 2 |
| `main.rs:4621` | `endpoint-rebind-client` | same (seal at `:4682`) | **affected → fixed** | same |
| `main.rs:467` (`common()`) | `server` / `client` | payload sealed **directly**: `seal_unreliable(&payload)`, no Data frame | **safe**: 1200 is exactly the cap | real pairs at 1170/1171/1200 over UDP **and** TCP: all exit 0/0 |
| `main.rs:5711` | `workload` | socket-free `SessionRuntime::queue_send`; never sealed | **safe** | runs at 1170/1171/1200: all exit 0 |

The server no longer idles. At 1171 and 1200 both sides of failover and
endpoint-rebind now exit 2 within about 1.0 s. Previously the client aborted and
the server ran until its timeout.

## Tests

- New `data_frame_commands_cap_bytes_at_the_sealable_payload` (capabilities). For
  both failover roles, 1171 is refused with the exact message and 1170 passes the
  bytes check. It also includes the guard `MAX_UNRELIABLE_DATAGRAM -
  PROCESS_DATA_HEADER_LEN == 1170`.
- Updated `endpoint_rebind_requires_the_documented_clique_on_both_sides`: the
  message now reads `1-1170`, the over-bound case is 1171, and the legal upper
  edge is 1170. **This test previously asserted that `--bytes 1200` is legal**,
  which is exactly the input that crashed. Pinning a crashing value as legal is
  why the bug survived.
- New neko-session `data_header_constant_matches_the_encoder`: the constant is
  checked against real `encode()` lengths.

## Mutation check (7 mutants, oracle = the two capabilities tests)

All **7 killed** (rc=101): bound +1, bound −1, bound back to 1200, and each of the
four guard sites individually reverted to `MAX_BYTES` (failover server, failover
client, rebind server, rebind client). Attempt history: the first per-side rebind
mutants were BADANCHOR because the surrounding context was identical in server and
client. They were re-anchored on the function header and both were then killed. No
survivors, no equivalents.

## Evidence boundary

Developer-local only. The CLI bound only narrows; no wire, frame cap, H-I4-119,
D019, release flag or READY_LIVE change.
