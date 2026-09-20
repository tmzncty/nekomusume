# H-R9-083 closure challenge at `696dd02`

**Classification:** HIGH — release-gate correctness/evidence truth  
**Reviewed repository HEAD:** `696dd02c7021cb5cf1fb35f757bd1cd88c0a7c00`  
**Developer source/test commit challenged:** `696dd02c7021cb5cf1fb35f757bd1cd88c0a7c00` (building on source repair `d1554a21258c4fe25e8b2ea0a07768f66133236c`)  
**Owners:** `scripts/bench/process-resource-sampler.py`, `scripts/bench/process-resource-sampler-test.py`, `scripts/bench/validate-process-resource.py`  
**Scope:** R9-11D terminal process/socket cleanup truth only. No WAN/live/performance conclusion.

## What `696dd02` correctly closes

The latest developer slice materially closes two prior H-R9-083 gaps:

- `owned_port_sockets_present()` now exposes a controlled `net_dir` seam without changing normal `/proc/net` behavior;
- direct deterministic tests cover one missing required table => `None`, a truncated TCP row with a caller-owned local port but missing state => `None`, and all four readable/no-match tables => `False`.

Together with `d1554a2`, the terminal oracle now reads all four required TCP/TCP6/UDP/UDP6 tables and fails closed on the previously identified read/parse counterexamples. The earlier partial-observation false-absence source defect is therefore repaired.

## Why H-R9-083 is still OPEN

### 1. The escaped-owner readiness check is after sampler completion, so it is not a happens-before handshake

Both escaped TCP and UDP regressions execute the sampler synchronously:

```python
run(... SAMPLER ...)
for _ in range(200):
    if esc_ready.exists():
        break
```

`run()` does not return until the sampler has already performed terminal ownership observation and written its result. Waiting for `esc.ready` / `escu.ready` afterwards therefore cannot constrain the ordering of the observation being tested.

A scheduler interleaving remains possible in which:

1. the direct helper spawns the `setsid()` descendant and exits;
2. the sampler observes the original process group empty before the escaped descendant binds;
3. the terminal socket oracle sees no supplied-port owner and writes `cleanup.complete=true`;
4. only afterwards the escaped descendant binds and writes the readiness marker;
5. the outer fixture notices the marker after `run()` returns.

The later JSON assertions may catch that interleaving if it happens, but the test does not *force* the ownership-establishment-before-terminal-observation relation. Its pass/fail therefore still depends on scheduler ordering. The accepted closure contract required a deterministic readiness/ownership handshake, not a post-hoc marker assertion.

**Smallest acceptable shape:** keep the original-group helper alive until it has consumed the escaped descendant's readiness signal (or otherwise gate the sampler's terminal observation on that signal), then let the original helper exit. The readiness event must occur after bind and before the sampler can certify terminal cleanup. Use bounded synchronization; do not replace this with a larger fixed sleep.

### 2. Assertion-safe cleanup sends SIGKILL but does not verify bounded disappearance

The current `finally` blocks best-effort `SIGKILL` the escaped PID but do not wait for or verify that `/proc/<pid>` is gone. The accepted H-R9-083 contract explicitly requires the helper to be actually gone/bounded-clean after the fixture, not merely signalled.

Add a bounded reap/disappearance oracle after the kill (or an equivalent deterministic cleanup helper) for both escaped TCP and UDP fixtures. Do not infer disappearance from signal delivery success.

### 3. The direct partial-observation regression does not exercise result construction

The new helper tests prove `owned_port_sockets_present(...) is None`, while exact-current `main()` correctly maps `None` to `owned_sockets_after_exit=None` and `cleanup.complete=false`. That source mapping is currently correct, but the accepted closure contract also requested a mutation-sensitive result-construction oracle so a future `None -> success` regression cannot pass the focused suite.

Keep this narrow: factor the two-line cleanup classification into a pure helper or use an equivalent controlled seam and assert that `group_empty=True + socket_state=None` cannot produce `owned_sockets_after_exit=0` or `complete=true`. Do not invent a new schema or framework.

## Required closure

1. preserve the corrected four-table TCP/TCP6/UDP/UDP6 present/absent/unknown semantics and the new malformed/missing-table direct regressions;
2. make escaped TCP and UDP ownership establishment a true pre-terminal happens-before relation: the original-group path must not be able to finish before escaped bind readiness is consumed;
3. retain assertion-safe cleanup and additionally prove the escaped helper is actually gone with a bounded disappearance/reap check;
4. add one focused result-construction negative: terminal socket state `unknown` cannot yield cleanup zero/complete;
5. run the focused process-resource tests, then on the final pushed source/test SHA run `PYTHONDONTWRITEBYTECODE=1 bash scripts/check.sh`, `git diff --check`, confirm clean tree, and persist exact SHA / UTC start-end / exits / OS-arch / stable Rust provenance.

No decoder/parser/crypto framing change is involved, so do not mechanically run fuzz. Do not redesign Session/Carrier/ACK/wire/crypto, change D019, or invent timeout/capacity/security policy values.

## Evidence boundary

This note is an independent exact-current source/test review. No reviewer-local test execution is claimed. GitHub combined-status and commit-workflow lookup exposed no hosted run for exact `696dd02`; absence of hosted evidence is not treated as failure. The repository-local/developer exact-tree provenance for a final H-R9-083 closure SHA is still required.

Until this closes, do not treat R9-11D2/D3, R9-12 provenance, or the earlier broad `7ff2a0d` lifecycle note as complete. D1 may proceed only where it does not rely on the terminal cleanup oracle; once the HIGH is repaired the coding agent should continue immediately through the existing D1-D4 queue without waiting for reviewer cadence.
