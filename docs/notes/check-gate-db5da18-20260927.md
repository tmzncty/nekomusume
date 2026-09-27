# Exact-tree check.sh provenance — db5da18 (2026-09-27 local / 2026-09-27 UTC)

Developer-local exact-tree gate on the final reachable pushed SHA for the
neko-observe event-JSON-envelope boundary-pin slice.

- **SHA:** `db5da18565c250db0c7096190aaa10e556443ac5`
  (`test(observe): pin the observability event JSON envelope verbatim`)
- **Command:** `PYTHONDONTWRITEBYTECODE=1 bash scripts/check.sh`
- **Tree state:** clean detached worktree (`/tmp/nmOG2`) created from the
  literal pushed SHA `db5da18` (worktree head asserted equal to it);
  `git diff --check` exit 0; `git status --short` empty before and after
- **UTC start:** 2026-09-27T01:41:04Z (log `/tmp/nm-ob-gate2.log`, 1204 lines)
- **UTC end:** 2026-09-27T01:47:17Z
- **Exit code:** 0 (host load average ~8.5 from unrelated jobs)
- **OS / arch:** Linux 6.8.0-137-generic, GNU/Linux (x86_64)
- **Rust toolchain:** `rustc 1.98.0 (88d9e12ae 2026-08-18)`

## Gate attempt history (disclosed)

**Attempt 1** (same SHA, same command, worktree `/tmp/nmOG`, 01:36:44Z-01:40:57Z)
returned **EXIT=101**: one failure in `neko-cli`'s `probe.rs` suite -
`sigterm_after_ready_stops_and_releases_tcp_and_udp_bindings` panicked with
`server not ready: child stdout ended before ready`. The other 73 tests in that
suite passed in 235.53s. Host load average was ~7.9.

This is the **known-fragile probe.rs harness** documented in the standing
briefing (the full `cargo test -p neko-cli` takes ~235s and fails intermittently
under load; at most one clean re-run is permitted). The same suite passed in the
pre-commit workspace run at load ~11.0, and the failing test is in a suite this
change does not touch (a test-only addition to `neko-observe`).

**Attempt 2** (the same literal SHA, `scripts/check.sh` re-run cleanly from a
fresh worktree) returned **EXIT=0**, 1204 log lines, recorded above. Per the
standing rule only one clean re-run was taken.

## Pre-commit sequence (strictly serial)

`cargo fmt --all --check` 0, `clippy --workspace --all-targets -D warnings` 0,
then `cargo test --workspace`: 38 suites, 686 passed, 0 failed
(2026-09-27T01:32:16Z-01:36:21Z, load average ~11.0). Commit, push, remote
verification and the gate were separate tool invocations; `origin/main` was
fetch-verified equal to HEAD before the gate. The gate was issued **alone**,
asserting the literal SHA.

## Slice scope

Test-only, **additions only: +78 lines** (0 deletions) in
`crates/neko-observe/src/lib.rs` - two tests in `tests`. No semantics change.

20 hand-written mutants over `Event::to_json_line` and `correlation_json`; **20
killed, 0 survivors, 0 HANGs, 0 BADANCHOR**. There is **no residual to report for
this slice**. The pre-existing tests left 10 survivors; the two new tests close all
10. Baseline measured green in a separate worktree (`/tmp/nmOb`).

## The gap

The outer envelope of `to_json_line` was entirely unwitnessed. Every existing
assertion on a JSON line used `contains` on an INNER fragment
(`"datagram.dropped"`, `"error_code":"queue_full"`, `"stream_id":"stream:1"`), and
the string `"schema"` never appeared in any test. All of the following survived
the probe before this slice:

- the `"schema"` key name and its `nekomusume.observability-event.v1` value
- the `"schema_version"` key name and its value (`1`)
- the `"sequence"`, `"observed_at_ms"` and `"correlation"` key names
- the ORDER of the envelope fields
- the correspondence between the format placeholders and the ARGUMENTS passed to
  them (reordering the arguments survived)
- a missing separator between two fields

This is the published Era-4 v1 event contract - consumers parse these key names,
so a rename is a breaking wire change nothing would have caught.

Pinned two ways: a hand-built `Event` asserted verbatim with different values in
the neighbouring numeric fields (41 / 7) so a swap is visible rather than masked,
and with one `None` correlation field alongside two present siblings; plus a real
event produced end-to-end through `Producer::record_health`, asserted verbatim,
which also pins that an absent correlation key is OMITTED rather than emitted
empty.

`correlation_json` itself was already covered - an existing test asserts its
output directly with two fields, and mutants dropping/renaming/reordering the
optional keys or dropping the closing brace are all killed - so this slice covers
the envelope only, and that is stated rather than implied.

## Process note

A scratch probe used to print the exact output lines was initially removed by
matching stale text and the match failed. Rather than patching around it, the file
was restored wholesale from a pre-scratch byte copy and the tests re-applied, then
verified with md5 equality, a byte-for-byte `diff`, and a grep confirming no
scratch residue remained. The snapshot/format/parse steps and the probe were
separate serialized tool calls.

## Evidence boundary

Developer-local execution only; not reviewer-local, hosted CI, WAN, or
performance evidence. No production source changed. No H-I4-119, D019,
release-flag or policy-value change. `READY_LIVE: none`.
