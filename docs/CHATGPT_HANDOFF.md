# ChatGPT reviewer handoff — H-R9-039 closed at 021d79d; R9-3 blocked on final provenance

## Current repository truth

- Latest developer-owned source/test commit reviewed: exact `021d79d88dadc9dbc7cf749745b2395c06602b29` (`test(cli): P4-B assert server success (H-R9-039)`), building on `9091803`.
- Prior independent reviewer finding: [`docs/reviews/reviewer-r9-h-r9-039-p4b-server-exit-oracle-9091803-20260917.md`](reviews/reviewer-r9-h-r9-039-p4b-server-exit-oracle-9091803-20260917.md), reachable through reviewer commit `fe3c87330c7e23986f893d98b3ff22d578e7bae5`.
- H-R9-039 is closed at exact `021d79d`: P4-B now retains `srv_status` from `finish_server` and asserts `srv_status.success()` with `server_log` in the failure message, preserving all other P4-B exact assertions.
- H-R9-038 is fully closed at exact `9091803` + `236e962` + `a523733` + `021d79d`: both P4 negatives now prove server success, exact cardinality/identity, residual-domain terminal evidence, zero misclassification, typed nonzero client exit, and no false settled premise.
- P2 C1-C4 and H-R9-037/H-R9-036/H-R9-035/H-R9-034/H-R9-033/H-R9-032 and earlier accepted repairs remain closed. Candidate A and B remain closed.
- Developer-local clean exact-tree provenance for `021d79d`: `PYTHONDONTWRITEBYTECODE=1 bash scripts/check.sh` exit 0, `git diff --check` exit 0, clean worktree at pushed SHA, 2026-09-17T06:54:00Z → 2026-09-17T06:57:45Z, Linux x86_64, rustc 1.98.0 (88d9e12ae 2026-08-18). One flaky failure of `sigterm_after_ready_stops_and_releases_tcp_and_udp_bindings` on the first run; rerun at same SHA passed clean (55/56 then 1/1 on isolated retry).
- Open PRs at review time: none.
- `READY_LIVE: none`; release item 3 incomplete; release item 4 incomplete; `RELEASE_CANDIDATE=false`; `PRODUCTION_READY=false`; `FREEZE=false`; `RELEASED=false`.

The external coding agent must synchronize to current `main` and continuously execute every dependency-ready slice below: implementation/review -> focused deterministic tests -> commit -> push -> next slice. Reviewer cadence is only a check frequency and is never a work-ticket length or reason to idle.

## Accepted progress that must remain closed

Do not revert these exact-current repairs without contradictory evidence:

- H-R9-039 P4-B server-exit assertion at `021d79d`.
- H-R9-038 P4 dual-domain residual diagnostics and exact-oracle strengthening at `236e962` + `9091803` + `021d79d`.
- H-R9-037 reverse-order true three-way packet bind at `d5d5b23`, independently reviewed at `f2529087`.
- H-R9-036 reverse-order exact oracle at `855c679`.
- Post-return ACK arrival-order challenge at `83d10b0` + `424583d` + `855c679` + `d5d5b23`.
- P2 C1-C4 exact closure at `142f0a1` + `8d0ccd5` + `1f8bbb0` + `be03dc3`: TCP replay identity bound on both sides; migration milestones exact cardinality/order; server dual-domain stream/offset/len; positive-path three-way packet-number equality; exactly-one settled cardinality and post-transition order.
- `424583d` ordinary no-seam post-return path sends no duplicate Carrier ACK; ordinary order remains Session DeliveryAck then Carrier ACK, while `--reverse-post-return-ack` sends Carrier ACK before Session DeliveryAck.
- H-R9-034 settlement-continuation accepted-empty classification and late-stale discriminating proof at `b801b65`.
- H-R9-033 initial-loop accepted-empty classification at `7a7c48c`.
- H-R9-032 post-return stale/future packet-number cross-bind and settlement-order proof at `ddafbb1`.
- H-R9-030 ACK-obligation source repair and H-R9-029 post-return three-way classification.
- H-R9-027 stale semantic ACKs may be re-sealed under fresh authenticated envelopes; future ACKs use a non-empty never-sent range.
- H-R9-026 keeps real retirement, accepted-empty and typed rejection distinct.
- H-R9-025/H-R9-024 exact current-packet identity and positive post-return client/server packet binding source repair.
- H-R9-023/H-R9-022/H-R9-021 malformed terminal/order evidence.
- H-R9-020 count=4 ownership accounting (`2 reliable UDP + 1 uncertain TCP + 1 post-return reliable = 4`).
- reversed initial Session ACK buffering/ordered application.

## H-R9-039 exact finding

`reliable_udp_post_return_session_ack_withheld_fails` at exact `9091803` still contains:

```rust
let (_srv_status, server_log) = finish_server(server);
```

P4-B then proves exactly one positive Carrier retirement, three-way packet-number equality, residual `remaining_in_flight=0` / `session_outstanding=1`, zero Session transition, zero rejected/accepted-empty substitute, client nonzero terminal, no false settlement, exactly one server Carrier ACK and zero server Session ACK — but it never requires the server process itself to exit successfully.

A server can therefore emit the expected ACK diagnostic and fail later in the bounded lifecycle/cleanup path while this test remains green. This falsifies the handoff/commit-message subclaim that P4-B proves server success. No new Recovery/Session state-machine defect is presently proven.

# READY_LOCAL 1 — CLOSED at 021d79d

H-R9-039 P4-B server-success oracle is complete. `srv_status` is retained and asserted with `server_log` in the failure message; all other P4-B exact assertions are preserved.

# READY_LOCAL 2 — R9-3 Data-loss recovery

Suppress one reliable-owned Data only after congestion admission so the fault seam cannot bypass ownership accounting. Challenge the full current recovery contract:

- PTO fires only after its deadline;
- retransmission uses a fresh packet number / crypto nonce but stable frame/logical identity;
- receiver Session delivery remains exactly once;
- Session DeliveryAck and Carrier packet ACK remain separate evidence domains;
- positive completion reaches Recovery zero, otherwise failure is typed and bounded.

Use current M2 semantics; do not invent a new ACK or retransmission architecture.

# READY_LOCAL 3 — R9-4 ACK-loss + delayed original/reorder

Suppress one legitimate Carrier ACK, force a legitimate retransmission, then release the delayed original ACK. Require one logical Session delivery, Session dedup authority, fresh packet numbers/nonces, truthful PTO/loss/retransmit evidence and settled Recovery. A late original may be accepted-empty/stale but must never manufacture a second transition or rejection.

# READY_LOCAL 4 — R9-5 adversarial feedback / every Carrier caller

Challenge future/never-sent/stale/duplicate/tampered feedback across every exact-current `UdpAcknowledgement::Carrier` continuation, including initial receive, settlement and post-return owners. Rejected feedback must be atomic and fail closed without RTT/PTO/loss/cwnd/Session mutation; accepted-empty must not become transition or rejection. H-R9-033/H-R9-034 are closed examples, not reasons to skip the rest of the exact-current call surface.

# READY_LOCAL 5 — R9-6 ownership/resource boundedness

Every first send and retransmit must consult congestion admission before ownership commit; refusal commits no recovery or logical state. Keep one bounded retransmit-plaintext owner and deterministic teardown. Challenge algorithmic boundedness without adding a capacity-pressure benchmark or inventing new capacity/security values.

# READY_LOCAL 6 — R9-7 process/result truth

Independently challenge that Data, Carrier ACK, Session DeliveryAck, PTO/retransmit, Recovery, Session delivery, malformed budget, accepted-empty feedback, rejected feedback and terminal result remain distinct in structured process evidence. Emit/accept a diagnostic only after the exact claimed transition or classification has occurred.

# READY_LOCAL 7 — R9-8 warm TCP readiness

Challenge the existing single-active/multi-ready contract: authenticated resume-bound warm standby may carry readiness/control only and **no application Data before promotion**. Readiness/resource admission remains separate from packet feedback and Session delivery evidence. Use D064/current Carrier Manager semantics; no architecture change.

# READY_LOCAL 8 — R9-9 health + promotion

Challenge the integrated path rather than isolated helpers:

- resolved UDP outcomes feed existing health/hysteresis;
- recoverable reliable-UDP loss remains on UDP instead of spuriously promoting TCP;
- promotion happens only after existing readiness/health gates;
- draining/failed UDP receives no new Data ownership.

# READY_LOCAL 9 — R9-10 uncertain replay + dedup + cleanup

Challenge only genuine uncertain Session ranges replaying on promoted TCP; receiver Session dedup must remain exactly-once; TCP gets no duplicate UDP packet-ACK layer; shutdown/error negatives must clean resources and emit no false success.

# READY_LOCAL 10 — R9-11 lifecycle/terminal invariants

Close remaining lifecycle and terminal invariants for the materially new cross-process reliable-UDP/failover integration. Keep terminal classification distinct from intermediate diagnostics and preserve deterministic cleanup.

# READY_LOCAL 11 — R9-12 complete cross-process slice + exact-tree gate

Finish the remaining R9 cross-process implementation/test slice and run its clean exact-tree developer-local gate. Preserve source/test/evidence provenance boundaries; do not rewrite historical live evidence to describe the new local tree.

# READY_LOCAL 12 — dedicated independent R9 review

After the implementation/test lanes above are reachable and gated, perform a dedicated independent bounded challenge of materially new R9 send/admission/recovery/demux/Session ACK/Carrier ACK/failover/migration/cleanup/diagnostic behavior.

A bounded no-finding review is valid release-item-4 support if scope, inspected owners, tests/commands, exclusions and reachable anchor are explicit. Any concrete BLOCKER/HIGH returns immediately to smallest repair -> discriminating regression -> exact-tree local gate.

# READY_LOCAL 13 — Q10/Q11/Q12 factual reconciliation

Only after a reachable independent R9 review anchor exists, factually reconcile `docs/status.md`, `docs/release-security-review-packet.md` and related Q10/Q11/Q12 evidence text with exact reachable implementation/review anchors. Preserve historical evidence boundaries and do **not** change RC/freeze/release/production authority flags.

## Item-4 / core-surface inventory

Reachable independent bounded review already exists for earlier reliable-UDP engine basics (including the future/never-sent ACK guard), `CarrierState`, concurrent Carrier Manager/health/migration-back, FairScheduler/flow accounting, carrier adapters, `SessionRuntime`, observability (including mixed queue/generic drop classification), package/reproducibility/operator scripts, dependency/build surface, CLI portability/output, algorithmic boundedness/validators, DeliveryLedger/process codec/datagram/crypto API, and wire/parser surfaces.

The **materially new cross-process R9 integration** remains the broad uncovered item-4 surface until READY_LOCAL 12. Therefore repository-wide queue exhaustion is false even if any one narrow seam yields no finding.

## VPS opportunity

**Not READY.** Standing authorization remains valid, but authoritative classification is still `READY_LIVE: none`. Current work is local R9 correctness/evidence plus its later independent cross-process review. Do not repeat HY2, warm failover, periodic/soak, package lifecycle, migration-back, endpoint/key migration, IPv6, PLPMTUD or Experimental Track evidence merely because the VPS remains rented.

Only create a new `READY_LIVE` row if later code/instrumentation/hypothesis/path conditions produce a concrete unresolved real-network question that local/loopback evidence cannot answer.

## Separate non-blocking policy / authority gates

Do not invent or change while executing the queue above:

- `SessionRuntime.events` retention/capacity policy;
- D019 source-retention/no-reset policy;
- RSEC-001 adversarial-load/capacity suitability conditions;
- signing/key custody/SBOM/publication trust;
- previous frozen-release interoperability policy;
- core Session/Carrier/ACK/crypto/wire architecture;
- destructive/canonical-meaning migration;
- final RC/freeze/release/production authority.

These gates do not justify idling dependency-ready local R9 work.