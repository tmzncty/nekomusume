# Exact-tree check.sh provenance — f8d6ab4 (2026-09-25)

Developer-local exact-tree gate on the final reachable pushed SHA for
R-MBOX-MTU (oversized-datagram drop boundary in `FaultInjectCarrier`).

- **SHA:** `f8d6ab4c3b515ab69b8dfa252019beea57f0c51c`
  (`test(carrier): R-MBOX-MTU — oversized-datagram drop boundary in
  FaultInjectCarrier`)
- **Command:** `PYTHONDONTWRITEBYTECODE=1 bash scripts/check.sh`
- **Tree state:** clean detached worktree (`/tmp/nmMTU`) at the pushed SHA,
  `git status --short` empty before and after the run; `git diff --check`
  exit 0
- **UTC start:** 2026-09-25T03:12:34Z (log captured in
  `/tmp/neko-f8d6ab4-check.log`)
- **UTC end:** 2026-09-25T03:18:40Z
- **Exit code:** 0
- **OS / arch:** Linux 6.8.0-137-generic, GNU/Linux (x86_64)
- **Rust toolchain:** `rustc 1.98.0 (88d9e12ae 2026-08-18)` stable
  (`cargo 1.98.0 (797e8a9bc 2026-08-05)`)
- **Scope:** full `scripts/check.sh` — `cargo fmt --check`, `cargo check
  --workspace --locked`, `cargo test --workspace --locked` (36 "test result:
  ok" suites; neko-carrier lib 93/93 including the two new deterministic
  MTU regressions: `mtu_boundary_drops_oversized_only`,
  `mtu_drop_is_drop_class_not_reorder_class`), `cargo clippy --workspace
  --all-targets --locked -- -D warnings`, plus all
  governance/status/evidence/preauth/shell/release/reproducibility/smoke
  policy scripts and bounded warm-failover adapter tests. All passed.
- **Semantics implemented (fixture-only):** `max_payload_bytes: Option
  <usize>` on `FaultPolicy` — a send strictly exceeding the bound is dropped
  at the seam (drop-only: no fragmentation, no PTB/ICMP feedback, no
  interface-MTU change, no PLPMTUD policy, no wire-framing change). Oversized
  drops are drop-class: never enter the bounded reorder buffer, never
  release a withheld record, never duplicated. Strict-exceeds boundary
  (exact-boundary passes) asserted deterministically.
