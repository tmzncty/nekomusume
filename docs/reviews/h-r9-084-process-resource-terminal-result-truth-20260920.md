# H-R9-084 — process-resource sampler can exit success after cleanup truth fails

**Severity:** HIGH — release-item-4 evidence/result-truth boundary

**Independent reviewer anchor:** exact reachable developer source/test commit `2f92781f6a1b76118ecc2c5ee3d64daf4a2a6985`.

## Scope inspected

- `scripts/bench/process-resource-sampler.py`
- `scripts/bench/process-resource-sampler-test.py`
- `scripts/bench/validate-process-resource.py`
- `scripts/bench/process-resource.schema.json`
- current `docs/CHATGPT_HANDOFF.md` R9-11D1 contract

No reviewer-local execution is claimed in this note. The finding is a source-level deterministic counterexample on the current reachable tree.

## H-R9-083 status at this anchor

The implementation/test changes in exact `2f92781` satisfy the outstanding H-R9-083 source/test contract: escaped TCP and UDP parents fail closed on readiness timeout, the cleanup tests use `pathlib.Path`, bounded escaped-helper disappearance assertions remain active, four-table present/absent/unknown semantics remain fail closed, and the new `--net-dir` seam exercises result construction under an unknown terminal socket observation.

A clean exact-tree developer-local gate/provenance record for `2f92781` is not yet present/reviewed; therefore this note does not promote that commit into final R9 provenance or release evidence.

## Finding

The sampler's own process exit status is still derived only from the measured child/interruption outcome, not from terminal cleanup truth.

At the end of `main()`, current code computes:

- `sockets_still_owned = owned_port_sockets_present(...)`;
- `owned_sockets_after_exit = 0` only when the original process group is empty and socket absence is affirmative;
- `cleanup_complete = group_empty and sockets_still_owned is False`;
- the JSON correctly records `owned_sockets_after_exit = null` and `cleanup.complete = false` when observation is unknown or ownership remains possible.

But after writing that result the wrapper returns the child's `exit_code` (or interruption code) without considering `cleanup_complete`.

Exact `2f92781` now contains a deterministic witness: the new `fixture.partial-obs` invocation runs `/bin/true` with a test `--net-dir` missing one required protocol table. The test asserts JSON `owned_sockets_after_exit is None` and `cleanup.complete is False`, yet invokes the sampler with `run(..., check=True)`. That test can pass only because the sampler itself returns zero despite its own terminal cleanup result being unknown/incomplete.

This also conflicts with the repository's validator/schema boundary: `validate-process-resource.py` accepts only `process_reaped=true`, `process_group_empty=true`, `owned_sockets_after_exit=0`, `complete=true`, and `process-resource.schema.json` fixes the same cleanup fields to those successful values. Therefore the current executable can return success while emitting a result that its own authoritative validation contract rejects.

## Why HIGH

R9-11D1 explicitly requires structured result truth and process exit status to agree, and requires missing/unknown cleanup to remain failure/unknown. A caller that checks the sampler subprocess result before or instead of running the separate validator can currently accept a terminally unproven cleanup as command success. This is an evidence false-positive seam, not a transport-architecture issue.

## Required bounded closure

1. Preserve the measured child's terminal facts inside `result["exit"]`; do not rewrite a child exit 0 into a fictitious child failure.
2. Make the sampler wrapper itself fail nonzero whenever terminal cleanup is not affirmatively complete (`group_empty` false, owned socket present, or terminal socket observation unknown). Use an existing script-level failure convention if one exists; do not invent a protocol/wire/security policy value.
3. Convert the exact current partial-observation witness into a mutation-sensitive regression: `/bin/true` child exit remains recorded as 0, JSON retains `owned_sockets_after_exit=null` / `complete=false`, **and the sampler subprocess itself is nonzero**.
4. Add/retain a positive control where child exit 0 plus affirmative complete cleanup yields sampler exit 0 and validator-valid output.
5. Retain child-failure and timeout semantics: cleanup success must not overwrite an already-failing measured child; terminal cleanup failure must not manufacture success.
6. Keep H-R9-083 escaped TCP/UDP, four-table present/absent/unknown, readiness and bounded-disappearance regressions green.
7. After the repair, run the focused process-resource suite and then the normal clean exact-tree developer-local gate on the final pushed source/test SHA: `PYTHONDONTWRITEBYTECODE=1 bash scripts/check.sh`, `git diff --check`, clean tree, exact pushed SHA, UTC start/end, exit codes, OS/arch and stable Rust provenance.

No decoder/framing change is involved; do not mechanically run fuzz. Do not redesign Session/Carrier/ACK/wire/crypto, change D019, alter cleanup capacity/timeout policy, or turn this into a new result schema/framework project.
