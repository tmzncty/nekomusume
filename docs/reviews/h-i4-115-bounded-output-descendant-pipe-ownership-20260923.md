# H-I4-115 — bounded-output helper may block after direct child exit when a descendant holds inherited pipes

**Severity:** HIGH — release/item-4 test-harness boundedness / evidence reliability  
**Reviewer anchor:** exact current `main` observed at `9a46fe91185bb8c00aa2140575bf787581b945c3`  
**Scope:** `crates/neko-cli/tests/probe.rs::bounded_wait_with_output` and `crates/neko-cli/tests/multistream.rs::bounded_wait_with_output`; test harness only. This is **not** a production Session/Carrier/wire/crypto defect.

## Current repository truth

H-I4-113 and H-I4-114 source repairs are accepted at `46d624e2272fcd5cf9d86457d018c4f508b8ba10`, with reachable developer-local exact-tree provenance in `docs/notes/h-i4-113-114-provenance-46d624e-20260923.md`. GitHub Actions also has one completed successful Rust CI run for exact `46d624e`; that hosted run is separate cross-evidence, not a substitute for the persisted developer-local gate.

The current handoff still carries an obsolete H-I4-114 FRONT section even though the same file now says H-I4-113/114 are closed. This finding supersedes that stale FRONT; the deeper rolling queue remains valid.

## Concrete defect

Both exact-current bounded-output helpers:

1. take the direct child's piped stdout/stderr;
2. start `read_to_end` reader threads;
3. poll only the direct `Child` with `try_wait()` against a deadline;
4. once the direct child is observed exited, unconditionally call `JoinHandle::join()` on both reader threads.

The helpers' comments treat direct-child exit as proof that the pipes are EOF-closed. That implication is false on Unix: a descendant can inherit the stdout/stderr write descriptors. The direct child can exit and be reaped while the descendant keeps either pipe open, leaving `read_to_end` blocked and making the subsequent `join()` exceed the advertised harness deadline indefinitely (or until the descendant finally closes the descriptor).

This is an independent ownership seam from H-I4-106/107/109. Those findings hardened direct-child exit, kill, reap and pipe-full behavior; they did not prove EOF for inherited descendant writers after the direct child exits.

Exact-current `neko-cli` source does not itself launch external subprocesses, so this finding does **not** claim an existing product runtime descendant leak. The defect is in the generic test-harness boundedness contract: a process/lifecycle regression or wrapper that creates an inherited writer can turn a supposed bounded negative into a stranded gate.

## Independent bounded challenge

Reviewer source inspection of exact-current `probe.rs` and `multistream.rs` found the same unconditional post-exit reader joins in both helpers.

A reviewer-local generic Linux process check (not repository CI and not exact-tree Rust execution) used a direct child equivalent to:

```text
sh -c 'sleep 3 & exit 0'
```

with stdout/stderr piped. The direct shell exited in about 0.001 s, while reading the pipe to EOF completed only about 3.001 s later, after the background descendant closed its inherited writer. This confirms the OS ownership premise; it is not being reported as a repository test result.

## Closure contract

1. Add a deterministic negative regression for the inherited-writer shape: direct child exits promptly while a finite synthetic descendant keeps stdout and/or stderr open past the helper deadline. The regression must prove the harness returns/fails within its declared bound instead of blocking on a reader join. Keep the synthetic descendant finite and cleanup-safe.
2. Repair **both** `probe.rs` and `multistream.rs` helpers (or a genuinely smaller shared shape only if it does not create framework churn). Direct-child exit alone must no longer authorize an unbounded reader `join()`.
3. Preserve concurrent draining while the child is live so pipe-full deadlock protection from H-I4-106/108 remains intact. Preserve the existing direct-child `try_wait`/kill/bounded-reap classification from H-I4-107/109.
4. Reader completion after child exit must itself be causally bounded. A reader-completion timeout must be an explicit harness failure, not success or silent evidence loss. Do not introduce production process-group/session policy merely to repair a test helper.
5. Do not broaden this into conversion of fail-fast keygen/help/config fixtures, do not change Session/Carrier/ACK/crypto/wire semantics, and do not invent new repository-wide timeout/capacity/security values.
6. No decoder/parser/crypto-framing implementation change is implicated; do not run fuzz mechanically.
7. On the final pushed source/test SHA run `PYTHONDONTWRITEBYTECODE=1 bash scripts/check.sh`, `git diff --check`, confirm clean tree, and persist reachable developer-local exact-tree provenance with exact SHA, UTC start/end, exit codes, OS/arch, stable Rust and clean-tree state.
8. Continue immediately to the remaining ownership/cross-platform/boundedness queue after closure; do not infer repository-wide queue exhaustion from this helper repair.

## Evidence boundary

This review is exact-current GitHub source/control-flow inspection plus a generic reviewer-local Linux process-semantics check. The reviewer did **not** execute the repository Rust/full gate, fuzz, WAN, performance, or cross-platform runtime tests in this pass. `READY_LIVE` remains `none`; release items 3/4 and all release/governance flags remain unchanged.