# Era 3 closure: bounded M3/M4 evidence

Status: factual repository closure at parent `7d259af1829a45ace7bffe551834e49ced37193a`; not a protocol freeze, security approval, production release, or authorization for a new network experiment.

The machine-readable companion is [`era3-capabilities.v1.json`](era3-capabilities.v1.json). It is intentionally conservative: `supported` means only that the named, bounded evidence exists. It does not generalize beyond the evidence class and limits recorded for that entry.

## Provenance and preservation

The authoritative live repository `/media/tmzn/DATA5/nekomusume-research/repo` was inspected without reset on 2026-08-30. Its HEAD was exactly `7d259af1829a45ace7bffe551834e49ced37193a` on `candidate/g0-governance-status-repair6`, with the pre-existing untracked `neko-server.identity`. That identity was neither read nor copied into this worktree. Commit `7d259af` was locally reachable and inspected; it adds the bounded carrier-health observation CLI. This closure was prepared in a separate worktree from that exact parent.

Historical positive and failed evidence remains in place. In particular, the successful bounded VPS record does not erase the earlier failed candidate/configuration record, and this closure does not rerun or reinterpret old validation.

## Live VPS verified

- `m3-surviving-session-udp-tcp-failover`: the retained authorized run records established UDP state, injected UDP blackhole, fresh authenticated TCP resume for stable Session 7001, eight ordered 64-byte records, zero reported duplicates, successful exits, and cleanup. This is exact-candidate/route/bounds evidence only.
- `m3-vps-ordered-delivery`: ordered delivery and the observed duplicate count are properties of that same bounded run, not a long-session, NAT, IPv6, security, or performance claim.

## Deterministic or local evidence

- `m3-session-delivery-and-migration`: socket-free tests cover ordered delivery, uncertain ranges, confirmation on a changed path, bounded close, migration guards, and fail-closed exhaustion.
- `m4-authenticated-multistream-flow-control`: bounded process/loopback tests cover encrypted multistream framing, per-stream/session flow-window exhaustion, acknowledgement and fair scheduling behavior.
- `m4-carrier-manager-migration-back`: deterministic tests cover validation, generation, health, score margin, hold and single-switch guards. They do not perform WAN migration-back.
- `m4-carrier-health-observation`: parent `7d259af` adds bounded argument parsing, sample limits, and machine-readable local health evidence output. It does not itself measure a remote path.

## Unsupported, blocked, or inconclusive

The following entries are explicitly `supported: false` in the manifest and are regression-checked by `scripts/check-era3-capabilities.sh`:

- `zero-rtt`: explicitly disabled; no early data implementation.
- `concurrent-heterogeneous-multipath`: explicitly disabled; no UDP+TCP striping or aggregation.
- `production-readiness`: blocked; no production or security approval.
- `nat-endpoint-change`: inconclusive; no retained validation.
- `ipv6-surviving-session-failover`: inconclusive; the retained surviving-session run was not IPv6 evidence.
- `long-soak`: inconclusive; the soak document is a plan and does not authorize or prove a run.
- `performance-superiority`: inconclusive; no fair HY2 superiority result is claimed.

These negative entries are capabilities the documentation must not silently promote. A future change may alter them only by changing the manifest, evidence, closure note and tests together under a new review boundary.
