# Exact `c6ab8fd` cross-host repeated warm-failover attempt

## Verdict

This directory preserves the minimal non-secret result of the **one and only live invocation** at exact commit `c6ab8fdee384b900ee3b2420dde179452bd5187c`. Invocation count is exactly 1; retry count is 0. The launcher returned a schema-valid typed negative: 0/6 completed cycles, with cycle 1 `invalid_cycle_evidence`; the collector returned exit 2 and no valid stdout row.

The pre-invocation binary identity matched the staged endpoint descriptors: SHA-256 `60b4272269363021a38897d422de4f1953d6a778651e613fdd255627e987e15f`, size 1,142,256 bytes. The final external post-run cleanup observation reported zero experiment processes, listeners, and temporary paths. That external zero cleanup is separate from the zero-row artifact and does not create a cycle result.

## Immediate seam and limits

The immediate seam is **orchestration/evidence collection**: the cycle adapter returned nonzero without a valid stdout row. Retained private diagnostic evidence is the literal non-secret summary `live failover collector: missing JSON event: start` (SHA-256 `6f1f1c44a571fb2f9638887e736cd70de6466fc918e54547dec9fd843ecbe686`, 51 bytes); its private file remains outside Git. The collector code raises this diagnostic when the required `start` event is absent (`run-live-warm-failover-cycle.py`, `one_event(..., required=True)`), and the outer runner records the typed negative when the child exits nonzero without a valid row. The retained evidence does **not** prove whether the missing event arose from remote server event absence, output framing, or early exit; that deeper cause is indeterminate and is not asserted.

Because there is no cycle row, this artifact cannot positively prove endpoint provenance, resources, per-cycle accounting, timing, exit fields, or cleanup fields. No runtime failover, WAN/public reachability, or production claim follows. The separate synthetic preflight is not live evidence and is not archived here. Prior artifacts remain preserved and this handoff was not modified.

## Integrity

- Result: `result.json`
- Result SHA-256: recorded in `sha256sums.txt`
- Schema: `nekomusume.repeated-warm-failover.v1`
- Private diagnostic SHA-256: `6f1f1c44a571fb2f9638887e736cd70de6466fc918e54547dec9fd843ecbe686`
- CI before invocation: exact-head repository CI was green (per retained handoff/evidence)
- Privacy: no endpoint, host, identity, credential, command, or private diagnostic content is committed
