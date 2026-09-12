# ChatGPT reviewer handoff — escalate persistent multistream execution stall

## Reviewed state

- Previous reviewer handoff: exact `894799d57cba9999fab47267733cd365381b3ebb` (`docs(handoff): break multistream READY_LOCAL stagnation`).
- Previous reviewed developer-owned head remains exact `e96b9c1be84dbafb78010dca30cb7e3d5f5ada1a` (`docs: index independent item-4 checkpoint`).
- Before this refresh, default `main` still pointed to reviewer exact `894799d57cba9999fab47267733cd365381b3ebb`: **no developer-owned commit landed for nearly 24 hours after the already-explicit test-only repair was queued**.
- Exact `894799d` GitHub-hosted cross-evidence is green for both `stable checks` and `nightly decode fuzz smoke`. Hosted CI is additional evidence only; it is not a wait condition.
- The exact-current source still contains the same defect in `crates/neko-cli/tests/multistream.rs`: `executable_rejects_unsupported_only_negotiation_before_noise_or_data` calls `socket.read(&mut byte).unwrap()` and accepts only `Ok(0)`.
- Open PR #3 also records the same Windows `ConnectionReset` mismatch, but that PR is a one-commit fork branch based on old exact `01c876c`, is currently non-mergeable, and does **not** close this exact-current seam. Do not merge/cherry-pick that broad pre-auth PR merely to repair this test.
- Repository release truth remains unchanged: item 3 incomplete, item 4 incomplete, `RELEASE_CANDIDATE=false`, `PRODUCTION_READY=false`, `FREEZE=false`, `RELEASED=false`, D019 policy-blocked, and `READY_LIVE: none`.

## Reviewer verdict — persistent execution stall, not repository blocker

`AGENTS.md` 3.2 explicitly says a dependency-ready `READY_LOCAL` must not remain at the queue head while repository integrity, CI, authorization, and architecture are all clear. The previous handoff already supplied the exact owner, semantics, replacement shape, negative-test contract, gate and continuation. Since the code is still unchanged, the remaining problem is **execution of the queued coding slice**, not missing reviewer design work.

Do not spend another cycle on broad review notes, proposal-only output, watcher/polling, GitHub-hosted-CI waiting, or new documentation before landing this repair. The coding agent should execute the patch below as its first action.

## MUST_EXECUTE_LOCAL 1 — land the test-only EOF/RST repair now

Owner: `crates/neko-cli/tests/multistream.rs`

Test: `executable_rejects_unsupported_only_negotiation_before_noise_or_data`

Current behavior to replace:

```rust
assert_eq!(
    socket.read(&mut byte).unwrap(),
    0,
    "server entered Noise/data admission"
);
```

Required equivalent behavior:

```rust
match socket.read(&mut byte) {
    Ok(0) => {}
    Err(error) if error.kind() == std::io::ErrorKind::ConnectionReset => {}
    Ok(n) => panic!("server emitted {n} byte(s) after unsupported negotiation"),
    Err(error) => panic!("unexpected terminal-close error: {error}"),
}
```

Protected invariant: a syntactically valid hello offering only an unsupported version must terminate before any Noise/Session data is emitted. TCP may expose that fail-closed termination as orderly EOF or reset. The observable failure is **bytes emitted after rejection**, not whether the kernel chose FIN or RST.

Hard bounds:

- accept only `Ok(0)` or `ErrorKind::ConnectionReset` as terminal rejection;
- any `Ok(n > 0)` remains a hard test failure;
- any other I/O error remains a hard failure unless an already-existing contract independently proves equivalence;
- keep `assert_uniform_handshake_rejection`, including exact `neko: handshake rejected\n`, no success JSON and no record output;
- do not modify runtime code to force FIN;
- do not weaken unsupported-version negotiation rejection;
- do not create a general socket-close abstraction/framework for this seam;
- no fuzz is required because this is test-only and does not change wire/parser/crypto/framing behavior.

### Focused validation

Run a bounded timing-variation repetition, then the whole target:

```bash
for i in $(seq 1 20); do
  cargo test -p neko-cli --test multistream \
    executable_rejects_unsupported_only_negotiation_before_noise_or_data -- --exact
done
cargo test -p neko-cli --test multistream
```

If Cargo's exact filter syntax needs a mechanical adjustment, change only the invocation, not the semantic contract.

Commit the coherent test repair, push it, and then validate the **exact committed developer SHA** from a clean checkout/worktree:

```bash
PYTHONDONTWRITEBYTECODE=1 bash scripts/check.sh
git diff --check
```

Record concise developer-local provenance: exact SHA, commands, UTC start/end, exit codes, host OS/arch, Rust stable version, and clean-tree state. Do not record credentials, local/private addresses, endpoint topology, or unnecessary absolute paths.

If a local gate fails, fix the actual failure and rerun. Do not substitute GitHub-hosted CI for this exact-tree local gate.

**When green, immediately continue to REVIEW SUPPORT 2; do not wait for reviewer cadence.**

## REVIEW SUPPORT 2 — one narrow contradiction sweep only

After the repair is committed and exact-tree green, perform one short exact-current sweep only for a concrete adjacent contradiction in:

1. process-test/operator terminal-close determinism;
2. evidence distinctions among developer-local exact-tree CI, GitHub-hosted CI, independent-review support, live WAN evidence and performance claims;
3. current item-4 evidence with a named owner/API/call path and an answer already fixed by current spec/ADR/runtime semantics.

A new candidate is `ACCEPT` / `ACCEPT_WITH_BOUNDS` only if all are true:

- exact owner file/API/call path is named;
- a current repository behavior or claim is actually false or ambiguous;
- existing semantics already determine the answer without inventing policy;
- a bounded positive/negative regression can prove the repair.

If such a defect exists: smallest repair -> bounded regression -> commit/push -> exact-tree local gate -> concise provenance -> continue. If no such defect exists, go directly to CHECKPOINT 3.

Reject as filler: another broad ledger sweep, generic checker/parser/socket framework, schema normalization without a real producer/consumer contradiction, new benchmark infrastructure without a current question, repeated docs-only reclassification, unchanged WAN negative retry, or speculative Experimental Track work.

## CHECKPOINT 3 — coding queue exhaustion is a valid stop condition

If MUST_EXECUTE_LOCAL 1 is green and the narrow sweep finds no new concrete defect, record the implementation queue as genuinely exhausted. Do not manufacture 6–10 fake slices and do not enter 5-minute/30-minute/hourly polling loops.

Classify remaining work instead:

- **item 4 / independent review depth:** bounded support notes do not equal final independent security/release judgment;
- **D019 policy/value decision:** source-retention/no-reset policy remains maintainer-blocked; do not invent TTL/LRU/history-size policy;
- **RSEC-001 suitability:** representative adversarial-load/capacity-suitability evidence remains unestablished and pressure/capacity conditions needing maintainer judgment are not implicitly authorized;
- **item 3 / environment evidence:** natural-loss evidence remains incomplete; IPv6 remains environment-blocked; retained HY2/periodic/repeated-failover negatives stay frozen absent a materially new hypothesis;
- **release/production authority:** RC, production readiness, freeze, release and production authorization remain separate explicit decisions.

A genuinely exhausted coding queue is a real stop condition for implementation expansion. Resume only when repository truth produces a new dependency-ready implementation/evidence question or a maintainer decision unlocks a blocked gate.

## Live/release boundary

- `IMPLEMENTATION_COMPLETE=true` only in the repository's bounded research/governance sense;
- item 3 incomplete;
- item 4 incomplete;
- `RELEASE_CANDIDATE=false`;
- `PRODUCTION_READY=false`;
- `FREEZE=false`;
- `RELEASED=false`;
- D019 remains `SOURCE_RETENTION_POLICY_BLOCKED`;
- RSEC-001 still lacks representative adversarial-load/capacity-suitability evidence and final independent release/security judgment;
- `READY_LIVE: none` remains authoritative.

Do not rerun unchanged HY2, repeated warm failover, periodic/soak, package lifecycle, migration-back, endpoint/source migration, key update, IPv6, PLPMTUD or Experimental Track work merely because the VPS remains rented. Standing authorization remains valid for a future materially changed dependency-ready self-owned TCP/UDP question; this handoff creates no such live question.

## Rolling queue

1. **MUST_EXECUTE_LOCAL:** exact EOF/RST test repair in `crates/neko-cli/tests/multistream.rs`.
2. **FOCUSED VALIDATION:** bounded repeated single-test run + whole multistream target.
3. **EXACT-TREE CLOSURE:** commit/push; clean exact developer SHA; `scripts/check.sh` + `git diff --check`; persist sanitized provenance.
4. **REVIEW SUPPORT:** one narrow exact-current contradiction sweep with strict acceptance criteria.
5. **CONDITIONAL READY_LOCAL:** any concrete existing-semantics defect found by step 4 is pre-authorized for smallest-fix closure.
6. **CHECKPOINT:** otherwise record coding-queue exhaustion and classify item-3/item-4/D019/RSEC-001/environment/release-authority obstacles without filler.

Only stop earlier for a genuine BLOCKER/HIGH correctness/security/evidence finding, core Session/Carrier/ACK/crypto/wire architecture choice, destructive/canonical-meaning migration, action outside standing authorization, production/third-party action, new credential/server permission, benchmark/adversarial-load conditions needing maintainer value judgment, repository breakage, or actual runtime/tool-budget exhaustion.
