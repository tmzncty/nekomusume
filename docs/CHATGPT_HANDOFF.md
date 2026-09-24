# ChatGPT reviewer handoff — middlebox/reachability queue active

## Current repository truth

- Synchronize to current `main` before work. Exact code/tests/reachable commits/current specs outrank this handoff, chat memory and stale checkbox state.
- **Current repository HEAD reviewed:** `cd0817424703c13d25ee900aca19a57d6071e572`.
- **Latest developer-owned source/test anchor reviewed:** `bb686ce0068da411943a33af6664fc373acbfe47`.
- **H-I4-120 is CLOSED.** `9e79e6b...` restored the authorized pre-gate Recovery baseline and `4511e4f...` made the disputed regression semantics-neutral. Reachable developer-local exact-tree provenance remains valid for that tree.
- **H-I4-119 remains MAINTAINER / CORE ACK-PTO SEMANTICS.** Do not choose between any-newly-acked reset and ack-eliciting-only reset. The current baseline is authorized implementation state only, not a final semantic decision.
- **H-I4-121 / H-I4-122 / H-I4-123 are CLOSED on the current source/test chain.** The final process seam is precisely `--drop-post-auth-delivery-acks`: it suppresses Session DeliveryAck only, not version negotiation, Noise handshake, or reliable-UDP Carrier packet ACKs. `bb686ce...` fixes the reliable-UDP replay-cardinality mismatch by deriving the retained DeliveryAck-await width from the same logical ownership partition as the client and adds an exact replay-identity regression.
- Reachable developer-local exact-tree provenance for `bb686ce...` is in `docs/notes/check-gate-bb686ce-20260925.md`: full `scripts/check.sh` exit 0, `git diff --check` exit 0, clean detached tree, exact UTC start/end, Linux x86_64, stable Rust 1.98.0. GitHub combined status for `bb686ce...` currently exposes no status entries; do not call this hosted CI.
- Exact-current reviewer source/control-flow pass found no new BLOCKER/HIGH in the H-I4-121/122/123 closure chain.
- Evidence boundary remains narrow: `FaultInjectCarrier(loss_percent=100)` is deterministic Carrier-level all-send loss at the in-memory seam only. Historical `FaultPolicy::one_way` is alternating-send suppression on one wrapped endpoint, not a directional topology model.
- Release/security packet remains an evidence index, not release authority. Release items **3 and 4 remain incomplete**. `RELEASE_CANDIDATE=false`, `PRODUCTION_READY=false`, `FREEZE=false`, `RELEASED=false`.
- **READY_LIVE: none.** Do not repeat existing HY2, warm failover, periodic/soak, package lifecycle, migration-back, endpoint/key migration, IPv6, PLPMTUD or Experimental Track runs without a new concrete unresolved real-network question.

## FRONT — R-MBOX-REORDER-DELAY

This is dependency-ready local research/test-support work from `docs/research/middlebox-reachability-and-network-behavior.md`.

Exact-current fact:

- `FaultPolicy` already exposes `duplicate`, `reorder`, and `delay_ms`.
- `FaultInjectCarrier::send` currently applies `delay_ms`, loss/blackhole/close/alternating-send behavior, but does **not** consult `duplicate` or `reorder`.
- Therefore duplicate/reorder behavior is not yet implemented evidence. Treat this as an implementation gap, not as an existing capability and not as a production defect.

Closure contract:

1. Re-read exact-current `crates/neko-carrier/src/lib.rs`, the middlebox/reachability research plan, current carrier architecture, current status/release packet and this handoff.
2. Reuse/extend `FaultInjectCarrier`; do not create a parallel impairment framework.
3. Make reorder + delay deterministic and bounded for the scenario. The coding agent may choose the smallest test-support representation allowed by `AGENTS.md` (for example a one-record bounded reorder buffer or another equally small explicit ordering model), but the exact semantics must be encoded in focused regressions rather than only comments.
4. Preserve existing blackhole/loss/close behavior and the current evidence boundary for `one_way`.
5. If `duplicate` is activated in the same coherent slice, assert exact multiplicity and ordering. Do not let duplicate/reorder silently alter Session/Carrier production semantics.
6. Keep configured delay small and bounded in tests. Do not invent a new production timer/readiness/hysteresis/congestion/security/capacity policy merely to implement the fixture.
7. Add positive and negative deterministic tests that can prove the chosen reorder/delay behavior is actually exercised. Avoid flaky wall-clock upper-bound assertions; if elapsed time is used, rely only on a small configured lower-bound condition and the outer repository test timeout.
8. Commit/push the source/test slice, then continue immediately to the next READY slice; reviewer cadence is not a work-ticket boundary.
9. After the final pushed developer source SHA for a coherent group, run:
   `PYTHONDONTWRITEBYTECODE=1 bash scripts/check.sh`
   `git diff --check`
   verify clean tree and persist reachable exact-tree provenance with exact SHA, UTC start/end, exit codes, OS/arch and stable Rust version.
10. No decoder/parser/crypto-framing change is implied; do not run fuzz mechanically unless the actual diff expands into those owners.

## Dependency-ordered rolling queue

1. **R-MBOX-REORDER-DELAY — FRONT.** Reuse `FaultInjectCarrier`; activate real deterministic bounded reorder/delay evidence and optionally the already-declared duplicate control in the same small fixture seam.
2. **R-MBOX-MTU.** Model a local oversized-datagram / packet-size drop boundary without changing interface MTU, PLPMTUD policy or wire framing.
3. **R-MBOX-CLEAN-RECOVERY.** Controlled loss followed by a clean path; prove recovery/validation under existing Carrier semantics and no duplicate Session delivery.
4. **Grouped release/item-4 factual reconciliation.** After the three slices above, refresh release-packet facts and evidence boundaries; do not rewrite policy or release flags.
5. **R-MBOX-REPEATED-FAILOVER.** Bounded repeated failover/recovery transitions; assert single-active ownership, valid generation transitions, no duplicate delivery and event order.
6. **R-MBOX-RESOURCE.** Bounded repeated-transition resource ownership; check convergence / absence of monotonic leaked in-flight/session/path state. Do not turn this into capacity-pressure benchmarking or invent limits.
7. **R-MBOX-TRANSITION-MATRIX.** Reconcile injected impairment classes against current Carrier/Session state/event expectations and exact tested/unresolved boundaries.
8. **Pre-auth/security owner-diff reuse check.** Re-read only if source ownership moved; D019 remains a maintainer/security-policy gate.
9. **Package/reproducibility/build owner-diff reuse check.** Cargo manifests/lock/features/native hooks/scripts and exact package operators.
10. **Observability/adapters owner-diff reuse check.** Event/counter/high-water semantics plus Memory/UDP/TCP close/error/resource semantics.
11. **FairScheduler + SessionRuntime moved-owner challenge.** Multi-stream/session+stream flow-control, lifecycle/resource/window/DeliveryAck accounting.
12. **CLI/process/cross-platform + algorithmic boundedness.** Exit/JSON/human contract, child/socket/thread ownership and bounded algorithms; no pressure benchmark.
13. **Recovery repaired-owner refill.** Re-check post-H-I4-116/117/118 deltas while explicitly excluding the unresolved H-I4-119 policy seam.
14. **Repository-wide thirteen-surface refill.** Queue exhaustion is valid only if every implemented core surface has current bounded review/no-finding or a real external/policy gate.
15. **Conditional live only.** Create READY_LIVE work only if a new source/instrumentation/hypothesis/path condition leaves a concrete unresolved owned-endpoint network question.

## Stop / escalation boundary

Do not ask the maintainer for ordinary local design choices covered by current specs/AGENTS. Escalate only for BLOCKER/HIGH that cannot be automatically repaired, H-I4-119/core ACK-PTO semantics, D019 or other policy/value decisions, core Session/Carrier/ACK/crypto/wire architecture changes, destructive/canonical migration, authorization beyond the standing VPS boundary, production/third-party/new-credential permissions, adversarial-load/benchmark conditions requiring maintainer choice, or entry into a new release stage.
