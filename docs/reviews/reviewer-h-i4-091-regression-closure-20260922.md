# H-I4-091 continuation — source barrier repaired, regression/provenance closure still open

Severity: **HIGH (evidence-oracle closure / bounded test execution)**

Reviewed anchor: `53e502ecbc38c349f61653cdd1a46739da0bbdfd`.

## Developer-owned commit reviewed

- `53e502ecbc38c349f61653cdd1a46739da0bbdfd` — implementation/test: moves `malformed_or_unadmitted` to the post-`preauth.release(admission)` side and replaces the direct blocking `read_line` barrier with an off-thread reader plus bounded `recv_timeout`; timeout paths kill/reap the child.

The two source-level defects called out by the previous continuation note are repaired in this exact tree:

1. the classification wait no longer depends on a blocking pipe read in the test thread; and
2. the observable malformed-classification event is emitted after the corresponding pre-auth admission release.

H-I4-091 is **not yet closed**, because the previously required mutation-sensitive regressions and exact-tree provenance are still absent.

## Remaining closure gap A — no bounded-failure negative regression

The current positive malformed-churn test exercises the happy path where all `ATTEMPTS` classifications arrive. It does not deliberately run an insufficient/missing-classification case and prove that the new barrier exits within its bound rather than hanging.

That negative is part of the existing H-I4-091 closure contract. Source inspection can show the intended `recv_timeout`/kill/wait shape, but without a focused failing-sequence regression the bounded-failure behavior is not mechanically protected from a later mutation back to a blocking or non-reaping path.

### Required repair

Add the smallest test-only seam/helper needed to exercise fewer-than-required or missing classification events under a short bounded deadline. The regression must prove deterministic bounded failure and child cleanup. Reuse the existing off-thread-reader pattern; do not add a new runtime/protocol capability or a new security/capacity policy.

## Remaining closure gap B — no ordering oracle for post-cleanup snapshot

`main.rs` now executes `preauth.release(admission)` before emitting `malformed_or_unadmitted`, which is the correct current source order. However the test still has no mutation-sensitive oracle that fails if the Linux post-resource snapshot is moved before the post-cleanup barrier.

The previous closure contract explicitly required this because H-I4-090/091 are evidence-oracle findings: current source order alone does not prevent a future refactor from recreating the same false-negative window while the ordinary positive test remains green.

### Required repair

Keep the existing pre-churn guard. Add a small test-local phase/sentinel or equivalent ordering assertion such that:

- the post snapshot is permitted only after the `ATTEMPTS`th post-release classification has been observed; and
- moving the post snapshot before that barrier deterministically fails the test.

Do not change the existing Linux `/proc/<pid>/fd` + `/proc/<pid>/status` boundary or FD/RSS margins.

## Remaining closure gap C — exact-tree provenance

No accepted developer-local clean exact-tree provenance for `53e502e` is present at review time, and no visible hosted status/workflow run was found for this SHA. Hosted evidence is optional and its absence is not a failure.

After the negative/ordering regressions land, run focused regression(s) and then, on the final pushed source/test SHA:

```text
PYTHONDONTWRITEBYTECODE=1 bash scripts/check.sh
git diff --check
```

Record exact reachable SHA, UTC start/end, exit codes, OS/arch, Rust stable version, and clean-tree state. Do not record secrets, private addresses/topology, credentials, or unnecessary absolute paths.

## Preserved facts / exclusions

- H-I4-090 remains closed: the affirmative Linux baseline is still before `churn_started = true`, which flips before the first malformed `send_to`.
- This note does **not** claim a runtime resource leak.
- `malformed_or_unadmitted` remains diagnostic evidence only; it is not authentication, Session delivery, Carrier readiness, packet ACK, or release evidence.
- No wire decoder/parser/crypto framing owner changed in `53e502e`; fuzz is not required for this repair.
- No new capacity/security/TTL/history policy value is requested.
- No reviewer-local Rust/full-gate/fuzz/WAN/performance execution is claimed here; this is exact-current source/diff review.

## Continuation order

1. H-I4-091 negative bounded-failure regression + post-cleanup ordering oracle;
2. focused test(s) + final pushed exact-tree local gate/provenance;
3. I4-PORT-RES exact-current causal re-challenge;
4. pre-auth malformed/rejection resource-accounting bounded challenge;
5. cross-platform CLI/process semantics;
6. diagnostic/machine-output boundary;
7. algorithmic/resource boundedness reconciliation;
8. item-4/release-packet factual reconciliation;
9. repository-wide 13-surface refill;
10. conditional live only for a genuinely new unresolved real-network question.
