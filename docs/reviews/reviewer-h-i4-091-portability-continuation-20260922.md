# Reviewer continuation — H-I4-091 non-Linux Unix warning cleanliness

Anchor reviewed: `90831a72a4de7194e0c6ed1feeba806a45e5aaf6` (`test(cli): close malformed barrier ordering and EOF cleanup`).

Classification: **HIGH continuation / release-item-4 evidence + portability gate**. This is not a new runtime leak finding and does not reopen H-I4-090.

## What is accepted from `90831a7`

The exact-current implementation closes the two source-shape holes called out by the previous handoff:

- successful `malformed_classification_barrier(...)` completion is followed by a local `barrier_complete` token, and the Linux post-resource snapshot asserts that token before sampling;
- the stdout-EOF `Ok(None)` failure path now drops the receiver, kills/reaps the child, and joins the reader before returning `Err`.

The timeout/disconnect bounded-failure paths and post-`preauth.release(admission)` `malformed_or_unadmitted` ordering remain intact.

## Concrete remaining defect

`barrier_complete` is declared unconditionally in the surrounding `#[cfg(unix)]` integration test, but its only read is inside `#[cfg(target_os = "linux")]`.

On a non-Linux Unix target (for example macOS), the Linux snapshot block is compiled out while `let barrier_complete = true;` remains. That leaves a local variable with no use. The repository gate runs:

```text
cargo clippy --workspace --all-targets -- -D warnings
```

so this source shape is not warning-clean on the very non-Linux Unix surface the handoff requires us to preserve. A Linux-only clean gate cannot prove this portability claim.

This is a concrete compile/lint portability defect introduced by the closure commit, not a reason to change runtime semantics or resource-policy values.

## Smallest closure

1. Keep the post-barrier mutation oracle mechanically coupled to the Linux post-snapshot site while making the surrounding non-Linux Unix build warning-clean. Prefer the smallest cfg placement/shape; do not weaken the ordering oracle.
2. Preserve the current deterministic EOF cleanup and all existing timeout/disconnect cleanup.
3. Add the previously requested focused early-EOF/insufficient-classification regression if it is not already present, so the newly repaired EOF ownership path is mechanically protected. Do not create a general process-test framework.
4. Run focused CLI process tests, then on the final pushed source/test SHA run `PYTHONDONTWRITEBYTECODE=1 bash scripts/check.sh`, `git diff --check`, and verify a clean tree. Persist exact reachable SHA, UTC start/end, exit codes, OS/arch, Rust stable version, and clean-tree state. Do not mislabel Linux-local provenance as macOS/BSD execution.
5. No fuzz is required unless wire decoder/parser/crypto framing owners actually change.

After closure, continue directly into I4-PORT-RES and the existing rolling queue. `READY_LIVE` remains `none`; release/governance flags remain unchanged.

## Reviewer evidence boundary

This note is exact-current source/diff reasoning. The reviewer did **not** run Rust tests, the full gate, cross-platform CI, fuzz, WAN experiments, or performance tests in this review. GitHub-hosted status absence is absence of hosted evidence, not a failure result.
