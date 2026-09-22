# Independent bounded pre-auth rejection/resource-accounting re-challenge — `d96aabe`

**Anchor:** exact reachable `d96aabeb066e6aaaaa279d2ce153d4ff554040ea`.

**Result:** bounded **NO-FINDING** for the inspected engineering-control surface. This does **not** close D019 source-retention policy, RSEC-001 promotion/adversarial-load suitability, release item 3, release item 4, security approval, RC, production, freeze, or release.

## Scope and invariant challenged

Inspected exact-current owners and applicable contract:

- `crates/neko-cli/src/preauth.rs` — `ListenerAdmission`, staged TCP reservation ownership, input/response charging, queue ownership, expiry and release;
- `crates/neko-crypto/src/lib.rs` — `PreauthBudget` / `ProcessPreauthAdmission` permit, reject, expiry and release semantics;
- exact-current failover-server pre-auth call sites in `crates/neko-cli/src/main.rs`, including pending UDP negotiation/cache ownership and the H-I4-091 malformed classification path;
- `SECURITY.md`, D019 / `docs/adr/m1-g0-preauth-resource-budget.md`, `docs/reviews/resource-abuse-evidence-2026-09-04.md`, and current release packet/status boundaries.

The challenge was: before authentication, input/work/response/queue ownership must be charged before the protected work; malformed/rejected/expired work must fail closed and must not become authentication, path, packet-ACK or Session-delivery evidence; state/permit/queue ownership must converge on release/expiry/error without using a diagnostic as protocol evidence.

## Source challenge

No concrete counterexample was found in this bounded pass:

1. `ListenerAdmission::admit_carrier` expires stale process state before a new admission, reserves process state before creating the per-state budget, and releases the process state if the inner budget cannot be constructed.
2. Staged TCP input uses paired inner/process permits. `read_staged_frame` abandons an active staged reservation on every non-complete return; begin/extend/complete failures terminalize/reject the process owner rather than allowing later parsing to reopen it.
3. `charge_input` charges the per-state budget first, then the process/global owner; if the process charge fails it rolls the just-added inner input charge back and returns failure. `charge_response` has the analogous rollback on process-permit failure.
4. TCP/UDP response helpers settle the one-shot process response permit by `complete_response` only after the bounded send succeeds, and call `abandon_response` on timeout/setup/send/restore failures. No inspected response failure produces delivery/path/ACK evidence.
5. Queue ownership is explicit (`PreauthQueuePermit` inside `QueueReservation`). Ordinary dequeue consumes it; process expiry atomically removes the process-side queue count and the application owner is invalidated before it can be dequeued again.
6. The pending failover UDP owner charges every same-peer datagram before cached-hello comparison or Noise classification. A successful authenticated transition takes the pending owner, dequeues its reservation, and transfers the admission owner to the handshake-progress lifetime instead of silently duplicating it.
7. The exact-current invalid/unsupported first-negotiation path releases the admission before emitting `malformed_or_unadmitted`. Admission/charge failures likewise do not create authentication/session/path evidence. The H-I4-091 diagnostic remains a test/evidence barrier only and is emitted after cleanup on the malformed branch.
8. `ProcessPreauthAdmission::release` removes the live state and subtracts state-owned memory/queue accounting; `reject_state` is terminal/idempotent marking rather than a successful admission transition. Existing deterministic tests retain one-shot permit and expiry/release coverage.

The latest developer changes through `b4af007` are process-test readiness-helper changes; they do not materially alter these pre-auth accounting owners. The exact-current H-I4-091 diagnostic ordering was nevertheless re-read because it is part of the rejection/evidence boundary.

## Existing deterministic evidence reused

This source challenge reuses, without promoting, the already-reachable deterministic/process evidence indexed by `docs/reviews/resource-abuse-evidence-2026-09-04.md` and the exact-tree notes for cached retry/input/expiry/response-permit ownership. In particular, the retained-owner tests already exercise cached first-Noise retries plus nonmatching inputs under one source-owned packet ceiling, terminal failure beyond the ceiling, expiry invalidation, response-permit settlement, and zero live state after release.

No reviewer-local Rust command, load test, WAN run, capacity benchmark or cross-platform execution was performed in this pass. The developer-reported exact-tree gate for the immediately preceding process-helper repair remains `docs/notes/i4-cli-proc-096-provenance-b4af007-20260922.md`; it is not reclassified as pre-auth adversarial-load evidence. No decoder/parser/crypto-framing owner changed in this review, so no fuzz run is claimed or requested.

## Explicit exclusions / still-open gates

- **D019 source-retention/no-reset policy remains a maintainer/security policy gate.** This review does not choose TTL/LRU/history/capacity/security values or reinterpret that conflict.
- **RSEC-001 remains HIGH for release/security/public-listener promotion** because representative adversarial-load suitability and the separate release/security decision remain absent. The no-finding here is only an independent bounded engineering-control challenge.
- This review does not claim public-listener safety, production capacity, security approval, complete item-4 closure, WAN correctness or performance.
- `RELEASE_CANDIDATE=false`, `PRODUCTION_READY=false`, `FREEZE=false`, `RELEASED=false`; `READY_LIVE: none` remains unchanged.

## Queue consequence

The pre-auth malformed/rejection accounting lane may advance as a bounded no-finding support item. The next dependency-ready local lane is cross-platform CLI/process-test semantics, followed by diagnostic/machine-output boundary, algorithmic/resource boundedness, package/build reproducibility, item-4/release-packet reconciliation, targeted materially-changed core spot review, repository-wide 13-surface refill, and conditional live only if a genuinely new real-network question appears.
