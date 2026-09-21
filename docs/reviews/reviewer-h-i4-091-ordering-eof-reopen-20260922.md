# H-I4-091 continuation — closure claim is still mechanically falsifiable

Severity: **HIGH (evidence-oracle correctness / bounded test-process cleanup)**

Reviewed repository anchor: `5ff70a8cefd68a87072933132b42c66701f6586c`.

Developer-owned commits reviewed since the prior reviewer handoff:

- `2a36892a958028ad3ce9c8ddd2e79e7c6cf68dec` — implementation/test: factors `malformed_classification_barrier`, adds the missing-classification negative regression, preserves post-`preauth.release` diagnostics, and moves the positive server duration high enough for the bounded barrier/client sequence.
- `a5e8e7665041027234fd5d6ea8f125ddefe51bbc` — docs-manifest anchor correction only.
- `5ff70a8cefd68a87072933132b42c66701f6586c` — handoff/provenance closure claim.

The previously identified runtime/test-thread source defects are repaired: malformed classification is emitted after the corresponding admission release on the invalid-negotiation path, and the main test thread no longer blocks directly in `read_line`. The developer-local exact-tree gate recorded for `a5e8e76` is accepted as developer-reported provenance only.

H-I4-091 nevertheless remains **OPEN / HIGH** because the current closure claim does not satisfy its own mutation-sensitive ordering requirement, and one failure path still does not deterministically reap the child.

## Finding A — the claimed post-snapshot ordering oracle does not guard the snapshot

Current positive-test order is:

1. send `ATTEMPTS` malformed datagrams;
2. call `malformed_classification_barrier(...)` and wait for all classifications;
3. take the Linux `/proc` post-resource snapshot;
4. run the authenticated failover client;
5. join the reader and only then count `barrier_lines.matches("malformed_or_unadmitted")` and assert `== ATTEMPTS`.

The source comment says that moving the post snapshot before the barrier "deterministically fails this count assert". That is false. A concrete mutation can move only the Linux `after = process_resource_snapshot(pid)` block above the `malformed_classification_barrier(...)` call while leaving the barrier and later `barrier_lines` count unchanged. The barrier still eventually sees all `ATTEMPTS` events and the count assert still passes. Resource growth caused while those malformed inputs are processed can therefore again happen **after** the sampled `after` value without this supposed oracle detecting the mutation.

This is the same false-negative class H-I4-091 was opened to prevent, so source order being correct today is not enough to close the evidence-oracle contract.

### Required minimal repair

Add a test-local mutation-sensitive barrier state/token whose truth is established only by successful completion of `malformed_classification_barrier`, and assert that state immediately at the post-resource snapshot site before sampling. A simple local `barrier_complete=false -> true` transition is sufficient if the snapshot block checks it; a typed test-only token is also acceptable. The important property is mechanical: moving the snapshot site above barrier success must fail deterministically (or fail to compile), independent of later log counting.

Do not change the existing Linux-only `/proc` boundary or FD/RSS margins.

## Finding B — stdout EOF returns without deterministic child reap

Inside `malformed_classification_barrier`, timeout/disconnect paths kill and `wait()` the child, but the `Ok(None)` branch (reader observed stdout EOF before enough classifications) returns `Err(...)` immediately without `kill`, `try_wait`/`wait`, or joining the reader handle. `Child` drop is not a reap contract. An early-exit/early-stdout-close missing-classification case can therefore leave process cleanup outside the helper's claimed bounded ownership semantics.

The current negative regression sends zero malformed datagrams to a server whose duration exceeds the barrier deadline, so it specifically exercises the timeout path and does not cover this early-EOF branch.

### Required minimal repair

On every unsuccessful barrier exit, restore deterministic ownership before returning: terminate if still needed, reap the child, and join/reconcile the reader handle without creating an unbounded wait. Add a focused early-EOF/insufficient-classification regression if needed to make this branch mutation-sensitive. Reuse the existing helper/seam; do not add a protocol/runtime capability.

## Preserved facts and exclusions

- H-I4-090 remains closed: the affirmative baseline is before `churn_started = true`, and that flag flips before the first malformed send.
- The current executable source order `preauth.release(admission)` -> `malformed_or_unadmitted` on the invalid-negotiation path is accepted.
- This review does **not** claim a production/runtime resource leak; the finding is about the release-evidence oracle and bounded test-process ownership.
- `malformed_or_unadmitted` remains diagnostic-only evidence; do not promote it to authentication, Session delivery, Carrier readiness, packet ACK, wire, or release semantics.
- No wire decoder/parser/crypto framing owner changed; fuzz is not required for this repair.
- No new capacity, TTL, LRU/history, security, signing, publication, or release policy value is requested.
- No reviewer-local Rust/full-gate/fuzz/WAN/performance execution is claimed here.

## Closure and continuation order

1. Repair the snapshot ordering guard and EOF/failure-path reap semantics with focused deterministic regressions.
2. Run focused tests, then on the final pushed exact tree run `PYTHONDONTWRITEBYTECODE=1 bash scripts/check.sh`, `git diff --check`, and verify a clean tree; persist exact reachable developer-local provenance.
3. Continue immediately with I4-PORT-RES causal re-challenge, pre-auth rejection/resource accounting, cross-platform CLI/process semantics, diagnostic/machine-output boundary, boundedness reconciliation, item-4/release-packet reconciliation, repository-wide 13-surface refill, and conditional live only if a genuinely new unresolved network question becomes READY.

`READY_LIVE` remains `none`; release items 3 and 4 remain incomplete; `RELEASE_CANDIDATE=false`, `PRODUCTION_READY=false`, `FREEZE=false`, and `RELEASED=false` remain unchanged.
