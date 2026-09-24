# Exact-tree check.sh provenance — bb686ce (2026-09-25)

Developer-local exact-tree gate on the final reachable pushed SHA after the
H-I4-123 replay-cardinality repair. Completes the H-I4-122/H-I4-123 closure
chain: 2e88bdf (wording rename) → d3c939b (DeliveryAck-only narrowing) →
bb686ce (ownership-partition-derived replay count + combined regression).

- **SHA:** `bb686ce0068da411943a33af6664fc373acbfe47`
  (`fix(carrier): H-I4-123 — derive replay cardinality from ownership
  partition`)
- **Command:** `PYTHONDONTWRITEBYTECODE=1 bash scripts/check.sh`
- **Tree state:** clean detached worktree (`/tmp/nm62dD9LFLS`), branchless
  HEAD equal to `origin/main`; `git status --short` empty before the run;
  `git diff --check` exit 0
- **UTC start:** 2026-09-24T18:45:20Z (log captured in
  `/tmp/neko-bb686ce-check.log`)
- **UTC end:** 2026-09-24T18:51:06Z
- **Exit code:** 0
- **OS / arch:** Linux 6.8.0-137-generic, GNU/Linux (x86_64)
- **Rust toolchain:** `rustc 1.98.0 (88d9e12ae 2026-08-18)` stable
  (`cargo 1.98.0 (797e8a9bc 2026-08-05)`)
- **Scope:** full `scripts/check.sh` — `cargo fmt --check`, `cargo check
  --workspace --locked`, `cargo test --workspace --locked` (36 "test result:
  ok" suites incl. neko-cli probe 74/74 with the new combined regression
  `reliable_udp_delivery_ack_suppression_replays_full_owned_set`),
  `cargo clippy --workspace --all-targets --locked -- -D warnings`, plus all
  governance/status/evidence/preauth/shell/release/reproducibility/smoke
  policy scripts and bounded warm-failover adapter tests. All passed.
- **Note:** H-I4-123 item 2 satisfied — the server's retained-replay width
  is derived from the same committed logical ownership partition as the
  client (`count.min(2)` under `--reliable-udp`, `count.min(1)` otherwise);
  the +1 hard-code is gone. Item 1/4 satisfied by the combined focused
  regression asserting exact replay cardinality/identity (offsets 0/16/32)
  and complete Session delivery without duplicate application bytes.
