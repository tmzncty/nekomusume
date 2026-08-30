# VPS soak plan (not authorized for execution)

## Status

`execution_not_authorized`: this document is a plan only. Real public WAN/VPS
experiments remain frozen. Historical authorization cannot be reused.

## Previous/current interoperability

Classification: `unavailable` / `not-applicable`. The repository has no formal
previous protocol release or frozen v0 wire contract; all crates are `0.1.0`,
and candidate branches are not releases. Reopen only after a versioned release
artifact exists with a frozen compatibility contract, reproducible build
identity, and an explicitly selected previous/current pair.

## Soak authorization boundary

The standing authorization covers bounded work up to 10 minutes. Any 30-minute
or longer soak requires fresh explicit administrator authorization. This plan
does not authorize execution.

## Medium-soak design

Proposed duration tiers are 5 minutes then 10 minutes, with a hard wall-clock
cap of 10 minutes unless a new authorization explicitly changes it. Use a
known released/candidate commit recorded in the result, a dedicated authorized
endpoint, and a bounded workload configuration. Do not use the unrelated
Hysteria service as a workload.

Sample every 1 second while the workload is alive, and capture at start, each
sample, abort, and cleanup:

- process RSS (KiB), with PID/executable identity;
- open FD count and `/proc/<pid>/fd` classification;
- socket count and relevant local endpoints;
- CPU time and application progress/integrity counters;
- exit code, signal, and cleanup status.

Abort immediately on any unexpected process, listener, endpoint, identity or
artifact; RSS exceeding 2x the start sample for two consecutive samples; FD
count exceeding 2x the start count or +64 for two consecutive samples; socket
count exceeding the configured bound; stalled progress for 30 seconds;
application integrity failure, authentication failure, protocol error, or
wall-clock deadline. Thresholds are abort guards, not leak proofs.

## Artifact retention

Retain only a redacted JSON result, hashes of the exact binary/script/config,
sample series, bounded exit/cleanup facts, and safe packet metadata if a
separate capture is authorized. Never retain payloads, keys, identities, raw
transient logs, or unrestricted packet captures. Delete temporary binaries,
client-generated identities, captures and logs after verified cleanup.

## Cleanup and evidence

Use isolated temporary paths and a trap; terminate the workload at the bounded
deadline, wait for descendants, verify no process/listener/socket remains, and
verify only pre-existing files remain. Record cleanup verification and any
pre-existing dirty state separately. A run is invalid if attribution cannot be
made.

The plan remains `execution_not_authorized`; no listener, probe, soak, route,
firewall, proxy, netns or production-network change is implied by this file.
