# H-R9-083 closure challenge at `b0bb93a`

**Classification:** HIGH — release-gate correctness/evidence truth  
**Reviewed repository HEAD:** `b0bb93aebeaf6e47cb9ddf3880931b484bc1a8d1`  
**Developer source/test commit challenged:** `b0bb93aebeaf6e47cb9ddf3880931b484bc1a8d1`  
**Owners:** `scripts/bench/process-resource-sampler.py`, `scripts/bench/process-resource-sampler-test.py`, `scripts/bench/validate-process-resource.py`  
**Scope:** H-R9-083 terminal escaped-owner cleanup evidence only. No WAN/live/performance conclusion.

## What `b0bb93a` improves

The escaped TCP and UDP fixture parents now remain in the sampler-created process group while they wait for the escaped descendant's bind/readiness marker. In the ordinary successful path this establishes the intended ordering: the original group cannot become empty until the escaped owner has bound and signalled readiness. The patch also attempts bounded `/proc/<pid>` disappearance checks after `SIGKILL`.

The underlying source repair from `d1554a2` remains correct for the previously identified four-table terminal ownership issue: TCP/TCP6/UDP/UDP6 are all required for definitive absence, and incomplete observation remains `unknown`.

## Why H-R9-083 is still OPEN

### 1. The focused regression is currently deterministically broken by an unbound `Path`

`process-resource-sampler-test.py` imports the module `pathlib`, not `Path` into the test module namespace. The new TCP and UDP cleanup loops call:

```python
Path(f"/proc/{pid}").exists()
```

instead of `pathlib.Path(...)` (or importing `Path`). Once the escaped PID file exists, the `finally` block sends `SIGKILL` and then raises `NameError: name 'Path' is not defined`. The exception is not covered by the `(OSError, ValueError)` handler.

This test is executed directly by `scripts/check.sh`, so exact `b0bb93a` cannot be accepted as a clean exact-tree closure/provenance anchor until the focused suite actually runs green.

**Smallest repair:** use the already imported `pathlib.Path` consistently (or explicitly import `Path`) in both bounded disappearance loops; keep the assertion inside assertion-safe cleanup.

### 2. The parent wait still has a timeout fall-through that does not prove readiness was consumed

Both generated parent helpers do:

```python
deadline = time.monotonic() + 10
while not os.path.exists(ready) and time.monotonic() < deadline:
    time.sleep(0.02)
sys.exit(0)
```

If the deadline expires, the original-group parent exits successfully even though it did not consume readiness. The outer test can still pass in an interleaving where the escaped child binds just after that deadline but before the sampler's terminal socket observation. That means a passing test does not strictly prove the claimed invariant "original-group path cannot finish before readiness is consumed".

**Smallest repair:** make the parent fail closed on readiness timeout (nonzero exit or equivalent deterministic failure) rather than silently exiting 0. Do not replace the handshake with a larger sleep.

### 3. The accepted mutation-sensitive result-construction negative is still missing

The exact-current helper regression still directly proves `owned_port_sockets_present(...) is None`, while `main()` currently maps `group_empty=True + socket_state=None` to `owned_sockets_after_exit=None` and `cleanup.complete=false`. The mapping is correct, but the focused suite still lacks the previously required mutation-sensitive result-construction oracle. A future `None -> zero/complete` regression could therefore survive the direct helper tests.

Keep this narrow: factor the two cleanup-classification expressions into a pure helper, or expose an equivalent controlled seam, and prove `group_empty=True + socket_state=None` cannot produce `owned_sockets_after_exit=0` or `complete=true`. No new schema/framework is needed.

## Required closure

1. fix the `Path`/`pathlib.Path` test failure in both escaped TCP/UDP cleanup paths;
2. make the parent readiness timeout fail closed so a passing regression proves readiness was consumed before original-group exit;
3. retain bounded disappearance verification for both escaped owners;
4. add the already-requested mutation-sensitive result-construction negative for `group_empty=true + socket_state=unknown`;
5. retain four-table missing/malformed/clean terminal-oracle regressions;
6. run the focused process-resource suite, then on the final pushed source/test SHA run `PYTHONDONTWRITEBYTECODE=1 bash scripts/check.sh`, `git diff --check`, confirm a clean tree, and persist exact SHA / UTC start-end / exit codes / OS-arch / stable Rust provenance.

No decoder/parser/crypto framing change is involved, so do not mechanically run fuzz. Do not redesign Session/Carrier/ACK/wire/crypto, change D019, or invent timeout/capacity/security policy values.

## Evidence boundary

This is an independent source/test challenge. Reviewer-local execution is not claimed. GitHub combined status and commit-workflow lookup exposed no hosted run for exact `b0bb93a`; absence of hosted evidence is not treated as failure. The source counterexample above is sufficient to reject `b0bb93a` as the final H-R9-083 closure anchor.

After this HIGH closes, continue immediately through the existing R9-11D1 -> D2 -> D3 -> D4 -> R9-12 -> final independent R9 -> Q10/Q11/Q12 -> repository-wide item-4 refill queue without waiting for reviewer cadence.
