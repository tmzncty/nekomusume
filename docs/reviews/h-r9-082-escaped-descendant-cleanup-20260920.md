# H-R9-082 — escaped descendant can falsify process/socket cleanup evidence

**Classification:** HIGH — release-gate correctness/evidence truth  
**Reviewed anchor:** `7ff2a0d750dff69d0477018977b6567e62e0c251`  
**Owners:** `scripts/bench/process-resource-sampler.py`, `scripts/bench/process-resource-sampler-test.py`, `scripts/bench/validate-process-resource.py`, developer R9-11D review at `docs/reviews/dev-r9-11d-process-lifecycle-20260920.md`  
**Scope:** R9-11D D2/D3/D4 only. No WAN/live/performance conclusion.

## Invariant challenged

A required process-resource run may report complete cleanup only after all benchmark-owned descendant/process/socket ownership represented by the run is actually terminal. Disappearance of the sampler-created **original process group** is not sufficient proof if a descendant can leave that group while retaining benchmark-owned resources.

## Source-level counterexample

The sampler creates one process group for the direct child and all normal descendants, and both sampling and terminal cleanup enumerate only processes whose `/proc/<pid>/stat` process-group id equals that original `pgid`. `stop_and_verify_group(pgid)` sends signals only with `killpg(pgid, ...)` and returns success as soon as `process_group_members(pgid)` is empty.

After that, the result derives

- `owned_sockets_after_exit = 0 if group_empty else None`, and
- `cleanup.complete = group_empty`.

The validator accepts exactly that tuple as complete cleanup. Therefore the socket-after-exit value is not an independent socket-ownership observation; it is inferred from original-process-group emptiness.

A descendant can call `setsid()` (or otherwise enter another process group), keep an owned TCP/UDP listener open, and outlive the original group. The original group can then become empty, making the current sampler report `process_group_empty=true`, `owned_sockets_after_exit=0`, and `complete=true` even though the escaped descendant/socket still exists. The sampler attempts Linux child-subreaper setup, but its cleanup enumeration is still keyed only to the original process group, so reparenting alone does not make such an escaped descendant part of `process_group_members(original_pgid)`.

Current `normal-group` and `timeout-group` regressions only exercise descendants that remain in the original process group. They do not create a `setsid()`/new-pgrp descendant. Consequently developer review `7ff2a0d` overclaims that “nothing escapes reaping” and that `owned_sockets_after_exit==0` proves descendant listener release.

This is a deterministic local evidence defect, not a claim that the normal product process intentionally daemonizes. R9-11D specifically challenges process/socket ownership and false-success cleanup evidence, so the sampler must fail closed for an escaped owned descendant rather than certify cleanup from a narrower group oracle.

## Required closure regression

Add a bounded deterministic fixture with an explicit readiness handshake:

1. sampler launches a helper in its normal created process group;
2. helper launches a descendant that calls `setsid()` (or otherwise demonstrably leaves the original process group), binds a caller-supplied loopback TCP listener, and signals ready only after the listener exists;
3. the original helper/group exits;
4. while the escaped descendant/listener still exists, the sampler must **not** return a cleanup-complete result;
5. the regression must prove the listener/escaped PID really existed at the challenged seam and must bounded-clean any helper it creates even when the assertion fails.

Do not use an arbitrary fixed sleep as the ownership oracle. Use readiness/release or another deterministic bounded handshake.

## Smallest acceptable repair shape

Make cleanup ownership truth cover sampler-owned descendants beyond the original process-group id and/or add an independent terminal socket-ownership/rebind proof appropriate to the committed local oracle. In particular, do not set `owned_sockets_after_exit=0` solely because the original process group is empty when escaped sampler-owned descendants remain possible.

Keep the change local to process/resource evidence semantics. Do not redesign Session/Carrier/ACK/wire/crypto, invent timeout/capacity/security policy values, alter D019, or open a live/WAN lane merely for this local defect.

## Validation contract

After the smallest repair and positive/negative regression:

- run the focused process-resource sampler/validator tests;
- on the final pushed developer source/test SHA run `PYTHONDONTWRITEBYTECODE=1 bash scripts/check.sh`;
- run `git diff --check` and confirm a clean tree;
- persist exact reachable SHA, UTC start/end, exit codes, OS/arch, stable Rust and clean-tree state as developer-local provenance.

No decode/parser/crypto framing change is involved, so fuzz is not mechanically required.

Until this closes, the R9-11D no-finding at `7ff2a0d` is superseded for D2/D3/D4 closure and R9-12 must not advance past this HIGH.
