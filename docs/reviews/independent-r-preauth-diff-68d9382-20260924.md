# Independent pre-auth owner-diff/reuse challenge — `68d9382`

**Source anchor:** exact reachable `68d938279478b91661f786e830e99af83db457f1`. The immediately preceding reviewer-only note commit does not alter the inspected owners.

**Result:** bounded **NO-FINDING / REUSE** for the exact-current pre-auth rejection/resource-accounting engineering-control surface. D019 and RSEC-001 remain separate policy/release-security gates.

## Owner-diff check

The latest dedicated bounded pre-auth challenge is `docs/reviews/independent-preauth-rejection-accounting-d96aabe-20260922.md`, anchored at reachable `d96aabeb066e6aaaaa279d2ce153d4ff554040ea`. It inspected:

- `crates/neko-cli/src/preauth.rs` (`ListenerAdmission`, staged TCP reservation/input/response/queue ownership, expiry/release);
- `crates/neko-crypto/src/lib.rs` (`PreauthBudget`, `ProcessPreauthAdmission`, permit/reject/expiry/release semantics);
- failover-server pre-auth call sites in `crates/neko-cli/src/main.rs`;
- the applicable security/ADR/resource-abuse/release boundaries.

An exact GitHub compare from `d96aabe...` to current source anchor `68d9382...` reports product/test changes only in:

- `crates/neko-carrier/src/lib.rs`;
- `crates/neko-cli/tests/multistream.rs`;
- `crates/neko-cli/tests/probe.rs`;
- `crates/neko-reliable/src/lib.rs`;

plus review/provenance/release documentation. None of the three pre-auth semantic owners above changed in that interval.

The later CLI test-harness process-ownership repairs H-I4-097..115 therefore do not move the production pre-auth accounting owner. The later Recovery/Reno/CarrierState repairs likewise do not change pre-auth admission, budget, permit, queue, rejection, or expiry semantics.

## Reused invariant result

Because the semantic owners are byte-for-byte outside the post-review changed-file set, the prior bounded no-finding remains applicable to the exact-current owner:

- protected pre-auth work remains charged before protected work;
- paired inner/process input and response accounting retains rollback on process-side refusal;
- response permits are completed only after bounded send success and abandoned on setup/send/restore failure;
- queue ownership remains explicit and converges on dequeue/expiry/release;
- malformed/rejected/expired work does not become authentication, path, packet-ACK, or Session-delivery evidence;
- source-owned state is released/expired according to the existing implementation, without silently converting diagnostics into protocol evidence.

No contrary current-owner counterexample was found by the owner-diff check.

## Explicit exclusions / gates

- **D019 source-retention/no-reset policy remains maintainer/security policy.** This review does not select TTL/LRU/history/capacity/security values or change reset semantics.
- **RSEC-001 remains HIGH for public-listener release/security promotion** until its separate adversarial-load/release-security evidence and decision are satisfied.
- This reuse result is not public-listener safety, production-capacity proof, security approval, item-3 closure, item-4 closure, RC/freeze/release authority, WAN evidence, or performance evidence.

## Evidence boundary

This slice used exact GitHub owner history/compare plus the reachable prior independent review. No reviewer-local Rust/full-gate command, hosted CI, fuzz, WAN, cross-platform execution, adversarial-load benchmark, or new developer-local provenance is claimed. No decoder/parser/crypto-framing owner moved, so fuzz is not requested.

`RELEASE_CANDIDATE=false`, `PRODUCTION_READY=false`, `FREEZE=false`, `RELEASED=false`; `READY_LIVE: none` remains unchanged.

## Queue consequence

The pre-auth diff lane closes as current-owner **REUSE**. Continue repository-wide 13-surface current-owner inventory and only create new bounded challenge lanes for an actually moved/unreviewed implemented owner. Preserve H-I4-119, D019 and RSEC-001 as separate gates rather than turning them into coding-agent stop conditions for unrelated READY_LOCAL work.
