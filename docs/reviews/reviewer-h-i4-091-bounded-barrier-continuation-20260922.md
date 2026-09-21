# H-I4-091 continuation — malformed-churn processing barrier is not yet bounded/release-complete

Severity: **HIGH (evidence-oracle correctness / bounded test execution)**

Reviewed anchor: `bb3d79625fcac568ea390aa531c8ed7df7b10961` (`main` at review time).

## New developer-owned commits reviewed

- `9b6d90a145b8c13534591336345d2b37b2e13c25` — implementation/test: adds `malformed_or_unadmitted` diagnostic and replaces the fixed sleep with a diagnostic-count barrier.
- `91bf170991e3f5b7f480f25b9f47047792e059d8` — formatting-only follow-up in the test.
- `bb3d79625fcac568ea390aa531c8ed7df7b10961` — checker/inventory maintenance for the changed pre-auth producer anchor.

The direction is correct, but H-I4-091 is **not closed**.

## Finding A — the advertised 5 s barrier can block forever

Exact-current `crates/neko-cli/tests/probe.rs` checks `Instant::now() < deadline` immediately before calling blocking `server.stdout.read_line(&mut line)`. Once inside `read_line`, no timeout is attached to the pipe read. If the next diagnostic never arrives (for example the child stalls, exits without another newline, or fewer UDP datagrams are classified than expected), the test can remain blocked past the 5 s deadline indefinitely. The next deadline check is unreachable until `read_line` returns.

This is already recognized elsewhere in the same test owner: `ready_failover_server` explicitly states that `read_line` is blocking and therefore performs it off-thread, bounding the wait with `recv_timeout`, killing/reaping the child on timeout so EOF releases the reader. The new H-I4-091 loop does not use an equivalent bounded mechanism.

Therefore the current code does not satisfy the handoff closure requirement for a **bounded fail-closed processing barrier**.

### Required repair

Use an actually interruptible/bounded wait. The smallest repository-consistent shape is to reuse the existing off-thread reader + bounded channel-wait pattern (or an equivalent mechanism) while preserving the `ReadyServer` stdout/log ownership needed later. On timeout, terminate/reap the test child so the reader cannot remain stranded. A fixed sleep is not a substitute.

Add a focused negative oracle that exercises a missing/insufficient classification path and proves the barrier exits within its bounded deadline rather than hanging.

## Finding B — the diagnostic becomes observable before the pre-auth admission is released

Exact-current `crates/neko-cli/src/main.rs` emits `malformed_or_unadmitted` and only afterwards calls `preauth.release(admission)`. The test takes the Linux `/proc` post snapshot as soon as the `ATTEMPTS`th diagnostic is observed. Consequently the last diagnostic can cross the stdout pipe before the server has executed the corresponding release, allowing the resource snapshot to race ahead of the final pre-auth cleanup.

For a malformed-churn **resource-growth** oracle, the server-side causal barrier must terminate after the rejection's resource cleanup point, not immediately before it. Move the observable barrier event to the post-release side (or provide an equivalent explicit post-cleanup event). Do not promote this diagnostic into authentication, delivery, Session, or Carrier evidence.

A focused ordering oracle should fail if the post snapshot is moved before the post-cleanup barrier.

## Preserved facts / exclusions

- H-I4-090's pre-churn baseline ordering remains accepted.
- Linux-only `/proc/<pid>/fd` + `/proc/<pid>/status` evidence boundaries and existing FD/RSS margins stay unchanged.
- No new resource/capacity/security policy number is requested.
- No wire/decoder/parser/crypto-framing owner is implicated; fuzz is not required for this repair unless those owners are changed.
- This finding does **not** claim a runtime resource leak. It says the current test cannot yet prove the intended bounded post-churn resource claim.
- No reviewer-local Rust/full-gate/WAN/performance execution is claimed in this note; review is exact-current source/diff reasoning.

## Closure order

1. make the classification wait truly bounded/fail-closed;
2. make the barrier post-cleanup, not pre-release;
3. add focused negative/ordering regressions;
4. run the focused test(s);
5. on the final pushed source/test SHA, run `PYTHONDONTWRITEBYTECODE=1 bash scripts/check.sh`, `git diff --check`, and verify a clean tree; persist exact developer-local provenance;
6. immediately continue with the existing I4-PORT-RES / pre-auth resource-accounting / portability / diagnostic-contract / boundedness / item-4 reconciliation queue.
