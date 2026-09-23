# Release/item-4 factual reconciliation — CLI process ownership group at `59c7f30`

## Classification

**NO NEW BLOCKER/HIGH; evidence-index reconciliation only.** Release item 4 remains incomplete and all release/governance flags remain unchanged.

Reachable reconciliation anchor: `59c7f3024494a6263be2f35952ea9061a75db4b9` (`main` at review start).

## Completed group being reconciled

The H-I4-097..115 CLI/process ownership series is now a coherent closed repair group for the exact source owners it challenged. The sequence repaired bounded child terminate/reap error handling, synchronous network-child ownership, server/stdout wait ordering, UDP application deadline ownership, readiness and peer-thread bounds, multistream/probe client ownership, helper cleanup classification, periodic/endpoint client owners, duplicate-negotiation read ownership, and descendant-held pipe completion.

The final source state for the last defect is `1f6d744c9954b9a7a5d693b43a20d137a5c5e75d`, with reachable developer-local exact-tree provenance in `docs/notes/h-i4-115-provenance-1f6d744-20260924.md`. Earlier findings retain their own exact source/provenance anchors; this note does not collapse them into one imaginary all-tests SHA.

Immediately following the repair group:

- `docs/reviews/independent-cli-process-ownership-fcd7a08-20260924.md` records a bounded current-owner **no-finding** sweep of remaining test-owned process/socket/thread completion paths.
- `docs/reviews/independent-cli-cross-platform-process-416fd5e-20260924.md` reconciles the current platform boundary: current package/CI evidence is Linux-scoped; Unix/Linux observations must not be promoted to Windows/macOS execution evidence.
- `docs/reviews/independent-i4-bnd-current-reconciliation-934c878-20260924.md` confirms by owner diff that the core `SessionRuntime`/carrier/recovery boundedness owners have not changed since the dedicated `5b7b22b` review, while preserving the existing `SessionRuntime.events` retained-history cap as a policy/value gate.

## Evidence-class truth

These facts must remain separate in any release/item-4 index:

1. **Developer-local exact-tree execution:** only where a reachable provenance note names the tested source SHA, commands, timestamps, exit codes, OS/arch, Rust version and clean tree.
2. **Reviewer source/control-flow review:** the 2026-09-24 no-finding/reconciliation notes are source/owner-diff review unless they explicitly say otherwise; they are not local Rust execution evidence.
3. **GitHub-hosted CI:** per-SHA workflow/status evidence is independent of local provenance and is not inferred from a green historical SHA.
4. **Cross-platform:** Linux execution and Unix-only regressions do not establish Windows/macOS/BSD behavior.
5. **Live/performance:** none of these process-ownership repairs or reviews are WAN, soak, benchmark-performance or release evidence.

No current packet statement should imply that one commit executed every H-I4-097..115 historical gate. The valid shape is a chain of reachable source/provenance anchors plus bounded independent review notes.

## Current platform/resource boundary

The current package/operator target evidenced in repository notes is `x86_64-unknown-linux-gnu`; hosted stable/fuzz jobs are Ubuntu. Current CLI source also contains Linux/Unix-specific evidence paths (for example `/proc/self/fd` benchmark FD accounting and POSIX signal handling). That is compatible with the present Linux-scoped evidence baseline, but it prevents a generic current-tree cross-platform execution claim.

Likewise, the retained `SessionRuntime.events` history bound remains `POLICY_BLOCKED_RESOURCE_BOUND`. This reconciliation does not choose a TTL/LRU/history-size/capacity/security value and does not convert that policy gate into a coding defect.

## Release-state truth

- `IMPLEMENTATION_COMPLETE=true` remains a bounded research-implementation statement only.
- `RELEASE_CANDIDATE=false`.
- `PRODUCTION_READY=false`.
- `FREEZE=false`.
- `RELEASED=false`.
- Release items 3 and 4 remain incomplete.
- `READY_LIVE: none` remains authoritative unless a new concrete real-network question is produced by changed code/instrumentation/hypothesis/path conditions.

No RC/freeze/release/production authority is exercised by this note.

## Next dependency-ready item-4 lanes

The process-ownership group is no longer a reason for the coding agent to idle. Preserve a deep queue and continue with independent implemented-core challenge/reuse rather than inventing framework/docs churn:

1. pre-auth rejection/resource-accounting current-owner reuse/rechallenge;
2. CLI exit-code / JSON / human-output contract current-owner reuse/rechallenge;
3. package/reproducibility/build/dependency surface current-owner reuse/rechallenge;
4. `neko-reliable` recovery spot challenge against changes since its last dedicated review;
5. `CarrierState` / manager / scheduler / flow-control owner-diff challenge;
6. carrier adapter close/error/resource semantics owner-diff challenge;
7. `SessionRuntime` lifecycle/window/DeliveryAck owner-diff challenge, excluding the policy-selected event-history cap;
8. observability projection/counter/high-water owner-diff challenge;
9. repository-wide 13-surface inventory/refill;
10. conditional live only if a genuinely new unresolved real-network question appears.

A later packet-index edit may link this reconciliation and the 2026-09-24 review notes, but it must not rewrite historical provenance or advance release flags.
