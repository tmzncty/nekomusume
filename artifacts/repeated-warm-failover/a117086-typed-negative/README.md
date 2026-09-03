# Exact `a117086` corrected structured six-cycle outcome

## Verdict

This directory preserves the minimal sanitized result of the **sole authorized live outer invocation** at exact commit `a117086fa69553a36021137900b6052050624a8b`. The launcher exited 1 after 2,303 ms. The schema-valid batch is a typed negative: 0/6 completed cycles; cycle 1 is `invalid_cycle_evidence` because the collector returned nonzero without a valid stdout row. **No retry occurred.** This is not a WAN failover pass, a completed runtime row, or public/general reachability evidence.

The separate retained `dry.json` was synthetic preflight only and is not live evidence. Preflight and live execution are intentionally distinguished; the preflight six rows are not archived as results.

## Failure boundary and evidence limits

The retained artifact establishes only the immediate orchestration/evidence-collection boundary: nonzero collector with no valid row. The collector diagnostic was not retained in the batch artifact, so no deeper cause or runtime conclusion is assigned. Full/prefix/typed-negative facts are limited to the retained artifact and logs: there is no full cycle, valid prefix, or typed runtime-negative row—only the batch-level typed `invalid_cycle_evidence` negative.

Because no cycle row exists, per-cycle semantic/accounting/timing/exit fields are **not collected**, rather than zero or successful. Remote server resources are **not_collected_remote**; no local transport observation is substituted. Required endpoint provenance was part of the corrected structured plan/contract, but no cycle row exists in which to assert it. No endpoint, host, identity, credential, command, or topology material is retained here.

Separate post-run observations (not fields carried by the zero-row artifact) verified zero experiment processes/listeners/temporary paths and removal of deployment, identities, private plan, cleanup launcher, and worktree. Cleanup does not convert the absent row into runtime evidence. Because the artifact has zero cycle rows, endpoint provenance, resource scope, and cleanup fields cannot be positively validated from an artifact row; remote resources remain `not_collected_remote`. Binary identity was checked before execution (same release bytes on both endpoint descriptors); binary SHA-256 `60b4272269363021a38897d422de4f1953d6a778651e613fdd255627e987e15f`, size 1,142,256 bytes.

## Integrity

- Result: `result.json`
- Result SHA-256: `71ab1cab9828f72f1b1e044bbbef5d39178ac0af9ea1a022916e872e3b5c63b6`
- Schema: `nekomusume.repeated-warm-failover.v1`
- Exact live outer invocation count: 1; retry count: 0
- Privacy scan: no endpoint, host, identity, credential, SSH plan, or topology material
- Prior historical negative `c156868` is preserved elsewhere and is not rewritten by this artifact
