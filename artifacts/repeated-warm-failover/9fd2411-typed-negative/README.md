# Exact `9fd2411` cross-host repeated warm-failover attempt

## Verdict

This directory preserves the minimal non-secret result of the **one and only live invocation** against exact commit `9fd24112763469c27b97ffdbdcccdb149259aae7`. The launcher exited 1 after 9,758 ms. The schema-valid batch is a typed negative: 0/6 completed cycles, with cycle 1 classified as `invalid_cycle_evidence` because the cycle collector returned nonzero without a valid stdout row. It is not a WAN failover pass, a completed runtime row, or public/general reachability evidence.

The exact release binary used by both endpoint descriptors was 1,142,256 bytes with SHA-256 `60b4272269363021a38897d422de4f1953d6a778651e613fdd255627e987e15f`; pre-invocation identity checks matched the staged local and remote binaries. A final post-run residue check reported zero experimental listeners/processes/temporary paths. Those checks bound identity and cleanup, but they do not turn the absent cycle row into runtime-correctness evidence.

## Failure boundary

Classification: **orchestration/evidence collection boundary**.

The retained batch establishes only that the cycle adapter returned nonzero and emitted no valid stdout row. In `scripts/bench/run-repeated-warm-failover.py`, a nonzero child with empty stdout is converted to this exact `invalid_cycle_evidence` detail. In `scripts/bench/run-live-warm-failover-cycle.py`, collection/configuration/process/evidence exceptions are written only to stderr, return 2, and deliberately leave stdout empty. The outer runner captured but did not preserve that stderr in the batch. Therefore the exact inner exception is not recoverable from retained non-secret evidence; assigning environment/path, runtime correctness, or cleanup as the deeper root cause would overclaim. No unchanged retry was made.

The separate private `dry.json` was only a synthetic preflight. Its six generated rows are not live evidence and lack `endpoint_provenance`; it is intentionally not archived here.

## Integrity and boundaries

- Result: `result.json`
- Result SHA-256: `4744a3b407537f0e442668f1c05f05e5218ad74bfc410e098a8fd89a8bca9f59`
- Schema: `nekomusume.repeated-warm-failover.v1`
- Live invocation count: exactly 1
- Retry count: 0
- Exact-head CI before invocation: run `33765521128`, green
- Privacy: endpoint, host, identity, credential, and command material are not present
