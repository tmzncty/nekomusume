# H-R9-083 — terminal owned-port oracle is TCP-only and fail-open on observation errors

**Classification:** HIGH — release-gate correctness/evidence truth  
**Reviewed anchor:** `4694bd4684104aa1515962d8cf723d6a13c8dacf`  
**Owners:** `scripts/bench/process-resource-sampler.py`, `scripts/bench/process-resource-sampler-test.py`, `scripts/bench/validate-process-resource.py`  
**Scope:** R9-11D D2/D3 only. No WAN/live/performance conclusion.

## Invariant challenged

A process-resource run may report `cleanup.owned_sockets_after_exit=0` and `cleanup.complete=true` only when the sampler has affirmative terminal evidence that no caller-owned experimental socket represented by `--owned-port` remains. Unknown/incomplete observation must stay unknown/failure.

## Current-tree counterexample

H-R9-082 correctly stopped deriving socket cleanup solely from original-process-group emptiness, but the replacement helper `owned_port_listeners_present()` only scans `/proc/net/tcp` and `/proc/net/tcp6` and only TCP `LISTEN` state. The same sampler's normal resource accounting explicitly treats `--owned-port` as covering TCP, TCP6, UDP and UDP6: `read_proc()` scans `/proc/net/{tcp,tcp6,udp,udp6}` and accepts an explicitly supplied local UDP port without a TCP-style LISTEN state.

Therefore an escaped/reparented descendant can call `setsid()`, bind a caller-supplied UDP port, outlive the original process group, and still make the terminal helper return `False`. Current result construction then promotes that to `owned_sockets_after_exit=0` and `cleanup.complete=true`. The validator accepts exactly that tuple as valid cleanup. The H-R9-082 regression covers only a TCP listener, so it does not falsify this UDP case.

The helper is also fail-open on observation failure: any `OSError`, `ValueError` or `IndexError` while reading/parsing the terminal `/proc/net` tables is swallowed and the function returns `False`, which is indistinguishable from a successful complete scan finding no owned socket. That contradicts the standing evidence rule that missing/unknown cleanup observation must not be promoted to success.

This is a local evidence/correctness defect, not a claim that the product intentionally daemonizes. It blocks acceptance of H-R9-082 as a complete R9-11D socket-ownership closure.

## Required closure

Use the smallest settled-semantic repair:

1. make the terminal owned-port oracle cover the same committed protocol surface as the sampler accounting: TCP/TCP6 listeners and UDP/UDP6 bound sockets for supplied ports;
2. distinguish `present`, `absent`, and `unknown` (or equivalent fail-closed representation); any incomplete `/proc/net` observation must not yield `owned_sockets_after_exit=0` / `cleanup.complete=true`;
3. add a deterministic escaped-descendant UDP regression: descendant demonstrably leaves the original process group, binds the supplied loopback UDP port, signals readiness only after bind, original group ends, and cleanup must remain incomplete while the escaped UDP socket exists;
4. keep the existing TCP escaped-descendant regression green;
5. ensure fixture cleanup is bounded even when an assertion fails (for example `try/finally` around escaped helper cleanup), and prove the challenged socket/PID existed before accepting the negative oracle;
6. do not invent new timeout/capacity/security values or redesign Session/Carrier/ACK/wire/crypto.

## Validation contract

Run the focused process-resource sampler/validator tests, then on the final pushed developer source/test SHA run:

- `PYTHONDONTWRITEBYTECODE=1 bash scripts/check.sh`
- `git diff --check`
- confirm clean tree and persist exact pushed SHA, UTC start/end, exit codes, OS/arch and stable Rust version.

No decoder/parser/crypto framing change is involved, so no mechanical fuzz run is required.

Until this closes, R9-11D2/D3 and R9-12 remain blocked behind this HIGH. Independent R9-11D result-truth work that does not depend on the faulty terminal socket oracle may continue only if it does not expand or rely on the false cleanup claim.
