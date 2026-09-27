# Exact-tree check.sh provenance — dc5e2e8 (2026-09-27 local / 2026-09-27 UTC)

Developer-local exact-tree gate on the final reachable pushed SHA for the
neko-cli multistream JSON artifact boundary-pin slice.

- **SHA:** `dc5e2e84a1170710e3faa9e72dddc3aab5c3bad3`
  (`test(cli): pin the multistream server JSON line and the remaining client fields`)
- **Command:** `PYTHONDONTWRITEBYTECODE=1 bash scripts/check.sh`
- **Tree state:** clean detached worktree (`/tmp/nmMsg`) created from the
  literal pushed SHA `dc5e2e8` (worktree head asserted equal to it);
  `git diff --check` exit 0; `git status --short` empty before and after
- **UTC start:** 2026-09-27T04:42:09Z (log `/tmp/nm-ms-gate.log`, 1216 lines)
- **UTC end:** 2026-09-27T04:48:22Z
- **Exit code:** 0 (host load average ~11.2 from unrelated jobs)
- **OS / arch:** Linux 6.8.0-137-generic, GNU/Linux (x86_64)
- **Rust toolchain:** `rustc 1.98.0 (88d9e12ae 2026-08-18)`

Passed on the first attempt. The load-sensitive `scripts/bench/run-periodic-command-test.py`
step (recorded as a new flake source in the previous slice's note) printed
`periodic command tests: ok` this run; load was ~11 at start.

## Pre-commit sequence (strictly serial)

`cargo fmt --all --check` 0, `clippy --workspace --all-targets -D warnings` 0, then
`cargo test --workspace`: 39 suites, 693 passed, 0 failed
(2026-09-27T04:37:41Z-04:41:45Z, load average ~11.2). Commit, push, remote
verification and the gate were separate tool invocations; `origin/main` was
fetch-verified equal to HEAD before the gate. The gate was issued **alone**,
asserting the literal SHA.

## Slice scope

Test-only, **additions only: +48 lines** (0 deletions) in
`crates/neko-cli/tests/multistream.rs`. `crates/neko-cli/src/multistream.rs` is
**unchanged** by this slice. No semantics change.

18 hand-written mutants over the two JSON artifact lines; **18 killed, 0
survivors, 0 HANGs, 0 BADANCHOR**. There is **no residual to report for this
slice**. The first probe run left 12 survivors, which is what identified the gap.
Baseline measured green in a separate worktree (`/tmp/nmMb`).

## The gap

The existing `bounded_tcp_multistream_loopback_is_ordered_and_json_evidenced` test
captures the SERVER's stdout but only asserts its **exit status** - the server's
JSON line was never examined. **All eight** server-line mutants survived: the `ok`
value, the `role` value, and the `peer`, `streams`, `records_per_stream`,
`bytes_per_record`, `records` and `payload_bytes` key names, anywhere. On the
client line four fields were unasserted: `role`, `records_per_stream`,
`bytes_per_record` and the `events` key name.

The server line's only variable part is `peer` (the client's ephemeral loopback
address), so it is pinned as an exact prefix plus an exact suffix around that one
field - which still fixes every key name, value, order and separator - and the
peer itself is required to be a loopback address with a numeric port. The client
additions pin the role value, the two fields echoing `--records`/`--bytes`, and
that `events` carries the joined event names (required to include a real kind, so
an empty join cannot pass).

## Process note: a mislabelled mutant set, discarded

My first mutant set mislabelled four mutants. The generator searched for each
fragment on the server line FIRST, so mutants named `MS_cli_payload_key` and
friends actually mutated the **server** line - which is why one of them appeared
to satisfy a client assertion. Re-targeting each mutant at its own line showed the
true picture (8 server gaps, 4 client gaps), and the client `records`/
`payload_bytes` mutants were then correctly killed. The mislabelled set is **not**
reported as evidence; only the corrected set is, re-run against the final
formatted test content with the snapshot md5-verified, byte-for-byte diffed, and
grepped for scratch residue.

Cosmetic note: the commit subject/message contains a stray double period (`..`) in
one line. The gate was run on the pushed SHA, so it is left as-is rather than
amended (amending would change the SHA and invalidate the gate record).

## Evidence boundary

Developer-local execution only; not reviewer-local, hosted CI, WAN, or performance
evidence. No production source changed. No H-I4-119, D019, release-flag or
policy-value change. `READY_LIVE: none`.
