# ChatGPT reviewer handoff — break multistream READY_LOCAL stagnation

## Reviewed state

- Previous reviewer handoff: exact `e357ba7936ccb089b6615480d53bcf244ef6e3af` (`docs(handoff): close item-4 checkpoint and fix multistream flake`).
- Previous reviewed developer-owned head: exact `e96b9c1be84dbafb78010dca30cb7e3d5f5ada1a` (`docs: index independent item-4 checkpoint`).
- Current developer-owned head remains exact `e96b9c1be84dbafb78010dca30cb7e3d5f5ada1a`: **no new developer commit has landed since the previous reviewer handoff**.
- Before this handoff refresh, default `main` still pointed to reviewer exact `e357ba7936ccb089b6615480d53bcf244ef6e3af`.
- Exact `e357ba7` GitHub-hosted cross-evidence is green for both `stable checks` and `nightly decode fuzz smoke`. Hosted CI is additional evidence only; it is not a wait condition and does not remove the concrete test reliability defect below.
- Repository release truth is unchanged: release item 3 remains incomplete, item 4 remains incomplete, `RELEASE_CANDIDATE=false`, `PRODUCTION_READY=false`, `FREEZE=false`, `RELEASED=false`, D019 remains policy-blocked, and current `READY_LIVE: none` remains authoritative.

## Reviewer verdict — implementation stagnation, not a blocker

The same explicit `READY_LOCAL` slice has remained at the head of the queue without a developer commit while there is no repository-integrity, CI, authorization, architecture, or environment blocker. Under `AGENTS.md` section 3.2 this is **implementation stagnation**; it must not turn into watcher/polling behavior.

The prior handoff was already semantically correct. This refresh therefore does **not** invent a new task or broaden scope. It makes the exact owner, replacement expression, negative-test contract, validation sequence, and post-fix continuation unambiguous so the coding agent can execute immediately.

## READY_LOCAL 1 — exact test-only repair, execute now

Severity: **LOW/MEDIUM test reliability / evidence determinism**. Runtime behavior is not the defect.

### Owner and exact seam

File: `crates/neko-cli/tests/multistream.rs`

Test: `executable_rejects_unsupported_only_negotiation_before_noise_or_data`

Current exact-current assertion performs:

```rust
assert_eq!(
    socket.read(&mut byte).unwrap(),
    0,
    "server entered Noise/data admission"
);
```

The semantic invariant is: after a syntactically valid negotiation hello whose only offered version is unsupported, the server must terminate before emitting Noise/Session data. A rejecting TCP peer may expose that terminal close as either orderly EOF (`Ok(0)`) or `ConnectionReset`; the historical hosted failure demonstrated the latter.

### Pre-authorized replacement contract

Replace only that close-observation assertion (or use a tiny test-local helper if that is clearer) with the equivalent of:

```rust
match socket.read(&mut byte) {
    Ok(0) => {}
    Err(error) if error.kind() == std::io::ErrorKind::ConnectionReset => {}
    Ok(n) => panic!("server emitted {n} byte(s) after unsupported negotiation"),
    Err(error) => panic!("unexpected terminal-close error: {error}"),
}
```

Required invariants:

- `Ok(0)` is accepted as terminal rejection;
- `ConnectionReset` is accepted as the same terminal rejection observation;
- any `Ok(n > 0)` is a hard failure because bytes were emitted after unsupported negotiation;
- any other I/O error is a hard failure unless an already-existing repository contract independently proves it equivalent;
- keep the existing `assert_uniform_handshake_rejection` check, including exact `neko: handshake rejected\n` stderr and no success/record output;
- do **not** modify runtime code to force FIN instead of RST;
- do **not** weaken unsupported-version fail-closed behavior;
- do **not** create a generic socket-test framework for this seam.

This is a test-only change. No wire/parser/crypto/framing semantics change is intended, so no fuzz run is required for this slice unless implementation scope unexpectedly expands into those areas.

### Required focused validation

After editing, run a bounded repetition of exactly this test to exercise close timing variability, then the whole multistream process-test target. A suitable bounded shape is:

```bash
for i in $(seq 1 20); do
  cargo test -p neko-cli --test multistream \
    executable_rejects_unsupported_only_negotiation_before_noise_or_data -- --exact
done
cargo test -p neko-cli --test multistream
```

If the exact Cargo test filter needs a syntactic adjustment, adjust only the invocation, not the behavioral contract.

Then commit the coherent test repair and validate the **exact committed developer SHA** from a clean checkout/worktree:

```bash
PYTHONDONTWRITEBYTECODE=1 bash scripts/check.sh
git diff --check
```

Also verify the exact-tree checkout is clean before/after the gate. Persist concise developer-local provenance containing exact SHA, commands, UTC start/end, exit codes, OS/arch, Rust stable version, and clean-tree state. Do not include local addresses, credentials, endpoint topology, or unnecessary absolute paths.

If the local gate is red, fix the actual failure and rerun the relevant gate. Do not use absence/presence of GitHub Actions as a substitute for the local exact-tree result.

Commit + push when green. **Immediately continue to REVIEW SUPPORT 2; do not wait for reviewer cadence.**

## REVIEW SUPPORT 2 — one narrow contradiction sweep after the test fix

The broad item-4 checkpoint is already complete. Do not repeat a generic repository audit and do not manufacture checker/harness/documentation work.

Perform one short exact-current sweep only for a concrete contradiction adjacent to:

1. process-test/operator close determinism;
2. release-facing evidence distinctions among developer-local exact-tree CI, GitHub-hosted CI, independent-review support, live WAN evidence, and performance claims;
3. a current item-4 evidence defect with a specific owner/API/call path and an answer already supplied by current semantics.

A candidate is `ACCEPT` / `ACCEPT_WITH_BOUNDS` only when all of these are true:

- exact owner file/API/call path is named;
- a current repository claim or behavior is actually false/ambiguous;
- the existing specification/ADR/runtime semantics supply the answer without policy invention;
- a bounded positive/negative regression can prove the repair.

If such a defect exists: smallest repair -> bounded regression -> commit/push -> exact-tree local gate -> concise provenance -> continue.

Explicitly reject as filler: another broad ledger sweep, generic checker/parser/socket framework, schema normalization without a real producer/consumer contradiction, new benchmark infrastructure without a current question, repeated docs-only reclassification, or re-running an unchanged WAN negative.

If no concrete repairable defect exists, proceed directly to CHECKPOINT 3.

## CHECKPOINT 3 — honest coding-queue exhaustion

If READY_LOCAL 1 is green and the narrow contradiction sweep finds no new concrete defect, the coding queue is **genuinely exhausted**. Do not enter a 5-minute/30-minute/hourly watcher loop and do not invent 6–10 fake slices to satisfy a nominal queue size.

Classify what remains instead:

- **independent review depth:** item 4 still requires genuine independent security/release judgment beyond bounded support notes;
- **D019 policy/value decision:** source-retention/no-reset policy remains maintainer-blocked; do not invent TTL/LRU/history policy;
- **RSEC-001 suitability:** representative adversarial-load/capacity-suitability evidence remains unestablished and pressure/capacity conditions requiring maintainer judgment are not implicitly authorized;
- **release item 3 / environment evidence:** item 3 and natural-loss evidence remain incomplete; IPv6 remains environment-blocked; current HY2/periodic/repeated-failover lines remain frozen at retained negatives unless a materially new hypothesis appears;
- **release/production authority:** RC, production readiness, freeze, release, and production authorization remain separate explicit decisions.

A genuinely exhausted coding queue is a real stop condition for implementation expansion, not a reason to poll GitHub repeatedly. New work should resume only when repository truth produces a new dependency-ready implementation/evidence question or a maintainer decision unlocks one of the blocked gates.

## Exact-current release/live boundary

Repository truth remains:

- `IMPLEMENTATION_COMPLETE=true` only in the repository's bounded research/governance sense;
- release item 3 remains incomplete;
- release item 4 remains incomplete despite bounded independent-review support;
- `RELEASE_CANDIDATE=false`;
- `PRODUCTION_READY=false`;
- `FREEZE=false`;
- `RELEASED=false`;
- D019 remains `SOURCE_RETENTION_POLICY_BLOCKED`;
- RSEC-001 still lacks representative adversarial-load/capacity-suitability evidence and final independent release/security judgment;
- `READY_LIVE: none` remains authoritative.

Do not rerun unchanged HY2, repeated warm failover, periodic/soak, package lifecycle, migration-back, endpoint/source migration, key update, IPv6, PLPMTUD, or Experimental Track work merely because the VPS remains rented. Standing authorization remains valid for a future materially changed dependency-ready self-owned TCP/UDP question; this handoff creates no such live question.

## Rolling queue

1. **READY_LOCAL:** exact test-only EOF/RST repair in `crates/neko-cli/tests/multistream.rs`.
2. **FOCUSED VALIDATION:** bounded repeated single-test run + whole multistream test target.
3. **EXACT-TREE CLOSURE:** commit/push; clean exact developer SHA; `scripts/check.sh` + `git diff --check` + clean-tree verification; persist sanitized provenance.
4. **REVIEW SUPPORT:** one narrow exact-current contradiction sweep with strict acceptance criteria.
5. **CONDITIONAL READY_LOCAL:** any concrete existing-semantics defect found by step 4 is pre-authorized for smallest-fix closure.
6. **CHECKPOINT:** if no such defect remains, record coding-queue exhaustion and classify item-3/item-4/D019/RSEC-001/environment/release-authority obstacles without filler.

Only stop earlier for a genuine BLOCKER/HIGH correctness/security/evidence finding, core Session/Carrier/ACK/crypto/wire architecture choice, destructive/canonical-meaning migration, action outside standing authorization, production/third-party action, new credential/server permission, benchmark/adversarial-load conditions needing maintainer value judgment, repository breakage, or actual runtime/tool-budget exhaustion.
