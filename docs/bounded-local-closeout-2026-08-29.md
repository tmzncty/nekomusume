# Bounded-local candidate closeout — 2026-08-29

## Scope completed

Within DATA5 and local/loopback authorization, the repository now carries
bounded candidate evidence for: wire codec and panic-free fuzzing; Session
ledger/evidence separation; Noise IK authenticated records and pre-auth budgets;
encrypted UDP loopback; deterministic UDP recovery; TCP fallback with uncertain
DataId deduplication; multi-stream/Carrier Manager state; validated migration
back; synchronized key update; PLPMTUD; authenticated unreliable datagrams; and
an isolated netns/netem benchmark harness.

The final boundary audits additionally cover near-maximum PLPMTUD arithmetic,
unreliable-datagram oversize/replay atomicity, FEC block identity limits, and
failover uncertain-state IDs/counters/capacity. FEC remains disabled, 0-RTT
remains disabled, and concurrent/heterogeneous multipath remains disabled.

Repository governance now checks status vocabulary/evidence/coverage, Git-tracked
and non-symlink evidence, Markdown links, release boundaries, roadmap/plan sync,
unique decision IDs, maintained shell syntax, and isolated fuzz worktree hygiene.

## Verification baseline

The closeout ran workspace tests, all-target clippy with warnings denied, the
full repository checker, every mutation regression, isolated decoder fuzz smoke,
and Git diff/worktree cleanliness checks. Exact evidence remains in the commits
and scripts; this document is an index, not security or release approval.

## Remaining blockers — administrator or external evidence required

The following are deliberately not executable under the bounded-local scope:

1. Public/non-loopback listeners and VPS IPv4/IPv6 reachability.
2. Sustained real-WAN UDP/fallback/NAT/endpoint-change experiments.
3. Same-route/same-MTU/same-load Hysteria2 comparison and performance claims.
4. Production tunnel/proxy deployment or replacement of an existing link.
5. Independent security review, protocol freeze, interoperability approval, or
   production release decision.
6. Enabling 0-RTT, default FEC, concurrent UDP+TCP, or heterogeneous aggregation.
7. Privileged Raw IP/ICMP/SCTP/DCCP/GRE/ESP experiments outside a separately
   authorized isolated lab.

No recurring cron or background network task is required. Future work must start
from an explicit authorization for one of these gates or from newly discovered
local correctness evidence.
