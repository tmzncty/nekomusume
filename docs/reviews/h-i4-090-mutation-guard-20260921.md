# H-I4-090 follow-up — pre-churn ordering fixed, mutation guard still missing

**Severity:** HIGH (evidence-oracle closure, not a newly observed runtime defect)

**Reviewed anchor:** current `main` at `e83d19259b83180234d34d9bcee3ce48367e24ef`.

**Current source/test owner:** `0dbb93145c4e1cc031ca30258b3c58a5c3155b51` (`crates/neko-cli/tests/probe.rs`).

## What is now correct

The malformed-UDP resource test again captures the Linux `/proc` baseline after server readiness and before the first malformed datagram, and captures the post sample after churn plus the existing settle point. The H-I4-089 Linux-only `/proc` boundary remains intact. Developer-reported clean exact-tree provenance for `0dbb931` is retained separately.

## Remaining closure defect

The previous H-I4-090 closure contract explicitly required a mutation-sensitive guard that would fail if the baseline were later moved back into the post-churn window. The exact `0dbb931` diff only relocates the existing `before = process_resource_snapshot(pid)` statement and adds a comment; the current test contains no independent sequencing oracle that fails when this statement is moved after malformed sends.

That matters because the original defect was itself a false-negative test-oracle ordering bug: the full gate can stay green while measuring two post-churn snapshots. Closing the finding without the requested ordering guard leaves the same class of evidence regression mechanically undetectable.

## Minimal closure contract

1. Keep the current production/test semantics: READY first, affirmative Linux baseline second, then malformed sends, settle, affirmative Linux post sample.
2. Add a small deterministic sequencing guard in the test path so that a mutation moving the baseline after malformed send start fails even when FD/RSS happen to be unchanged. A test-local phase/sentinel tied to the send path is sufficient; do not add a general framework/checker.
3. Preserve existing FD/RSS margins and Linux-only `/proc` policy; introduce no new capacity/security numbers.
4. Run the focused affected test(s), then on the final pushed source/test SHA run `PYTHONDONTWRITEBYTECODE=1 bash scripts/check.sh`, `git diff --check`, and verify a clean tree; persist developer-local provenance with exact SHA/timestamps/exit codes/OS-arch/Rust stable.
5. No decoder/parser/crypto-framing change is involved, so decode fuzz is not required.

After closure, perform one bounded independent re-challenge of the pre/post resource oracle and then reconcile the PORT/item-4 evidence notes before repeating the repository-wide 13-surface refill decision. Do not repeat unrelated unchanged closed lanes merely for activity.

`READY_LIVE: none`; no release/governance flag changes are implied.
