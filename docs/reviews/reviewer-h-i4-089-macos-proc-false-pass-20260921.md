# Reviewer finding H-I4-089 — `cfg(unix)` does not make Linux `/proc` evidence portable

**Severity:** HIGH (evidence / cross-platform process-test truth)

**Repository anchor inspected:** `cd82e7ead83b8efbef05b6bf3a4314dcb7498e0c`.

**Exact executable/source-test tree:** `42be53917ee58359ad86532a6aab0136f75047b9`.

**Owners inspected:** `crates/neko-cli/tests/probe.rs`, `docs/reviews/dev-i4-port-post-42be539-20260921.md`, `docs/reviews/reviewer-item4-current-closure-314ae06-20260921.md`, and `docs/CHATGPT_HANDOFF.md`.

This is a bounded reviewer source/evidence challenge. It is not reviewer-local test execution, hosted CI, live WAN evidence, a release decision, or a portability claim.

## Claim challenged

The post-`42be539` I4-PORT closure says the POSIX/process-resource fixtures are correctly scoped by `#[cfg(unix)]`, and explicitly states that Windows/macOS verification was not performed because `#[cfg(unix)]` removes these fixtures there. The repository-wide queue-exhaustion handoff then treats the cross-platform CLI/process-test surface as closed.

## Counterexample

`process_resource_snapshot(pid)` is Linux-specific:

```rust
let fd_count = fs::read_dir(format!("/proc/{pid}/fd")).ok()?.count();
let status = fs::read_to_string(format!("/proc/{pid}/status")).ok()?;
```

Its only challenged caller is the malformed-UDP/resource-growth test. The test is gated with `#[cfg(unix)]`, not `#[cfg(target_os = "linux")]`.

`cfg(unix)` is true on macOS and other Unix targets. Therefore the fixture is **not** compiled out on macOS. On a normal macOS host without Linux procfs, both snapshots become `None`. The resource assertion is guarded by:

```rust
if let (Some((before_fd, before_rss)), Some((after_fd, after_rss))) = (before, after) {
    ...
}
```

so unavailable `/proc` evidence is silently skipped and the test can continue as a success. That is a false-pass for the resource-observation subclaim, not a truthful non-Linux skip.

The narrower `signal_term()` helper is genuinely POSIX-shaped and `kill -TERM` is not the problem here. The concrete defect is promoting a Linux `/proc` measurement seam under the broader `unix` cfg and then treating absence as an optional success path.

## Why this reopens the queue

This directly falsifies the exact post-`42be539` I4-PORT closure statement that `#[cfg(unix)]` removes the affected fixture on macOS/non-Linux targets. It also falsifies the repository-wide conclusion that no current cross-platform process-test evidence gap remains. The runtime/product semantics are not changed by this finding; the defect is test/evidence truth.

## Minimal closure contract

1. Do not treat Linux `/proc` resource observation as generic `unix` evidence.
2. On Linux, the resource snapshot used by this regression must be affirmative; a missing/failed snapshot must not silently turn the resource-growth subclaim into PASS.
3. On non-Linux Unix targets, either:
   - keep the portable socket/lifecycle part of the test and explicitly compile/execute the `/proc` resource assertion only on Linux, or
   - gate the whole fixture to Linux if the committed test contract is intentionally Linux-only.
   Do not claim macOS coverage from a Linux-only observation seam.
4. Add a source-level or deterministic regression that makes the platform boundary mutation-sensitive (at minimum proving the Linux-only measurement cannot silently degrade to `None -> pass`).
5. Re-run the affected focused test(s), then on the final pushed source/test SHA run:
   - `PYTHONDONTWRITEBYTECODE=1 bash scripts/check.sh`
   - `git diff --check`
   - clean-tree verification
   and record exact SHA, UTC start/end, exit codes, OS/arch, Rust stable version, and clean-tree state.
6. Re-run the bounded I4-PORT independent source challenge against the repaired tree before restoring repository-wide item-4 queue exhaustion.

No wire decoder/parser/crypto framing owner is implicated; do not run fuzz mechanically.

## Exclusions

- No requirement to add macOS as a release target.
- No change to first-RC platform policy.
- No new capacity/TTL/security-policy value.
- No D019 change.
- No live/VPS run.
- No core Session/Carrier/ACK/crypto/wire architecture change.

**READY_LIVE: none.**
