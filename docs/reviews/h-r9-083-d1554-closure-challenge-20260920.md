# H-R9-083 closure challenge at `d1554a2`

**Classification:** HIGH — release-gate correctness/evidence truth  
**Reviewed repository HEAD:** `d1554a21258c4fe25e8b2ea0a07768f66133236c`  
**Developer source/test commit challenged:** `d1554a21258c4fe25e8b2ea0a07768f66133236c`  
**Owners:** `scripts/bench/process-resource-sampler.py`, `scripts/bench/process-resource-sampler-test.py`, `scripts/bench/validate-process-resource.py`  
**Scope:** R9-11D terminal process/socket cleanup truth only. No WAN/live/performance conclusion.

## What `d1554a2` correctly repairs

The developer repair materially improves the prior H-R9-083 state:

- every required `/proc/net/{tcp,tcp6,udp,udp6}` table read now fails closed on `OSError` instead of silently continuing;
- local-port parse failure returns `None` rather than being skipped;
- definitive absence is emitted only after the loop over all four tables completes without finding ownership or encountering those handled failures;
- escaped TCP and UDP fixture helpers are now killed from `finally`, so an assertion failure no longer bypasses the cleanup attempt.

Those changes close the specific `files_read > 0` false-absence counterexample from `docs/reviews/h-r9-083-reopen-partial-proc-observation-20260920.md`.

## Why H-R9-083 is not yet closed

### 1. TCP row parsing still has an uncaught malformed-row path

After parsing `fields[1]`, the terminal oracle reads `fields[3]` for TCP outside the guarded parse block:

```python
if name.startswith("tcp"):
    if fields[3] == "0A":
        return True
```

A malformed/truncated TCP row can therefore have a syntactically valid local endpoint in `fields[1]` — including a caller-owned port — but lack `fields[3]`. That raises `IndexError` instead of returning `None`/unknown. The already-accepted H-R9-083 contract is stronger: any parse failure that prevents complete terminal ownership determination must remain unknown, never crash or be promoted to absence.

This is a source-level counterexample, not merely missing test polish.

### 2. The required partial-observation regression is still absent

`d1554a2` changes the implementation and wraps escaped-helper cleanup, but it does not add the focused deterministic regression required by the existing finding. The current tests still exercise normal-host escaped TCP/UDP ownership only; they do not inject a required-table read failure or malformed terminal row and assert `unknown` plus `cleanup.complete == false`.

Without a mutation-sensitive negative, a later regression back to partial-observation success can pass the committed process-resource suite.

### 3. Escaped-owner fixtures still do not consume their readiness markers

Both escaped helper programs write `esc.ready` / `escu.ready` only after the socket is bound, but the parent fixture never waits for or asserts those markers before accepting the sampler result. The direct child exits immediately after spawning the `setsid()` descendant. Scheduler ordering can therefore let the terminal oracle run before the escaped descendant has bound the port.

The prior escaped-descendant closure contract required deterministic readiness/ownership establishment rather than scheduler luck. Fixed sleeps are not an acceptable replacement. The smallest repair is a bounded readiness handshake/oracle that proves the escaped owner is established before the parent path is allowed to complete, while retaining assertion-safe cleanup.

## Required closure

Keep the repair narrow and preserve current semantics:

1. make **all** terminal TCP row fields needed to decide ownership fail closed to `None` on malformed/truncated input; do not let a parse/index failure escape or become absence;
2. add focused deterministic partial-observation tests that exercise the helper directly or through a controlled seam, including at minimum:
   - three readable no-match tables + one required table unavailable => `None`;
   - malformed/truncated TCP row with a caller-owned local port but unavailable state field => `None`, not exception/absence;
   - result construction with unknown terminal ownership cannot emit `owned_sockets_after_exit=0` or `cleanup.complete=true`;
3. make escaped TCP and UDP regressions deterministically prove their owner has reached the bound-socket state before the parent/sampler can certify terminal cleanup; use the existing readiness marker or an equivalent bounded handshake, not arbitrary sleep;
4. keep the `try/finally` helper cleanup and ensure the helper is actually gone/bounded-clean after the fixture;
5. do not redesign Session/Carrier/ACK/wire/crypto, change D019/policy values, or add a capacity/timeout/security number merely to satisfy the test.

After the focused process-resource suite is green, run the normal final pushed exact-tree gate:

- `PYTHONDONTWRITEBYTECODE=1 bash scripts/check.sh`
- `git diff --check`
- clean tree
- persist exact pushed SHA, UTC start/end, exit codes, OS/arch and stable Rust version.

No decoder/parser/crypto framing owner is changed here, so no mechanical fuzz run is required.

## Evidence boundary

No reviewer-local test execution is claimed in this note. The finding is from exact-current source/test inspection. `d1554a2` has not yet supplied the required final exact-tree provenance record in the repository, and hosted CI remains supplemental rather than a wait condition.

Until this closes, do not treat R9-11D2/D3, R9-12 provenance, or the earlier broad `7ff2a0d` R9-11D note as complete. Independent D1 work may proceed only where it does not rely on the terminal cleanup oracle.