# ChatGPT reviewer handoff — H-R9-035 reopens positive P2 C4 exact binding

## Current repository truth

- Latest developer-owned source/test commit reviewed: exact `142f0a1c0d01c6f22278b4fc96cf758396e2511b` (`test(cli): P2 C1-C3 exact — TCP replay seq/stream bind + migration cardinality + server dual-domain`), preceded by exact `8d0ccd58f9905cf69646493d31a7277c373aba81` for P2 C4 tightening. Both are test-only.
- Current independent reviewer checkpoint: [`docs/reviews/reviewer-r9-h-r9-035-142f0a1-20260917.md`](reviews/reviewer-r9-h-r9-035-142f0a1-20260917.md), reachable through reviewer commit `93f5f14f36398c1d9176f11f129cef98912d770d`.
- **H-R9-035 is OPEN (HIGH, evidence/oracle correctness):** the ordinary positive P2 fixture binds client `r9_udp_post_return_sent.packet_number` to server `udp_return_packet_ack_sent.packet_number`, but does not bind the client positive `r9_udp_return_packet_ack.packet_number` into the same equality; it also does not require exactly one `r9_udp_post_return_settled`. Runtime already exposes the needed packet number and current source does not show a contradictory state-machine defect. Reopen only the acceptance proof unless the stronger oracle fails.
- P2 C1-C3 remain accepted at exact `142f0a1`. P2 C4 remains partially accepted at `8d0ccd5`: exact Session `stream=1 offset=48 len=16`, exactly one positive Carrier event with `applied=true, retired=true`, and zero rejected/accepted-empty are present, but the full three-way packet-number equality and exactly-one settled cardinality are not yet proved on the ordinary no-seam path.
- H-R9-034/H-R9-033/H-R9-032/H-R9-030/H-R9-029/H-R9-028 and earlier H-R9 repairs remain closed unless contradictory repository evidence appears.
- Candidate A (`Recovery::on_ack` future/never-sent guard) and candidate B (`record_datagrams` mixed queue/generic drop classification) remain closed on current code.
- Developer-local clean exact-tree provenance currently recorded for exact `142f0a1`: `PYTHONDONTWRITEBYTECODE=1 bash scripts/check.sh` exit 0, `git diff --check` exit 0, clean worktree at pushed SHA, 2026-09-16T23:57:49Z → 2026-09-17T00:00:54Z, Linux x86_64, rustc 1.98.0 (88d9e12ae 2026-08-18). This provenance predates the H-R9-035 oracle repair that is now required, so a final R9-2 exact-tree provenance lane remains open.
- Latest main hosted Rust CI observed for docs head `c8196a9bfc4ca7a1ec2f5865f9f629cce0936847` completed success (run `35164927626`). Hosted CI is cross-evidence only, not developer-local provenance.
- Open PRs at review time: none.
- `READY_LIVE: none`; release item 3 incomplete; release item 4 incomplete; `RELEASE_CANDIDATE=false`; `PRODUCTION_READY=false`; `FREEZE=false`; `RELEASED=false`.

The external coding agent must synchronize to current `main` and continuously execute every dependency-ready slice below: implementation/review -> focused deterministic tests -> commit -> push -> next slice. Reviewer cadence is only a check frequency and is never a work-ticket length or reason to idle.

## Accepted progress that must remain closed

Do not revert these exact-current repairs without contradictory evidence:

- P2 C1-C3 at `142f0a1`: TCP replay identity is bound on client/server to `stream=1 offset=32 seq=2`; migration challenge/validation/migration/post-return-send milestones have exact cardinality/order; server post-return Session ACK is `stream=1 offset=48 len=16`.
- P2 C4 partial tightening at `8d0ccd5`: exact client Session transition plus positive `applied=true, retired=true` Carrier transition and zero rejected/accepted-empty on the ordinary positive path. H-R9-035 only reopens the missing three-way packet-number equality and settled cardinality/order exactness.
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
- candidate A future-ACK engine guard and candidate B mixed queue/generic-drop observability repair.

# READY_LOCAL 1 — H-R9-035 positive P2 C4 exact three-way packet binding

Test/oracle-only by default. In `reliable_udp_migration_back_reserves_final_record`:

1. require exactly one client `r9_udp_post_return_sent`, exactly one server `udp_return_packet_ack_sent`, and exactly one client positive `r9_udp_return_packet_ack`;
2. parse `packet_number` from all three and require exact equality;
3. retain `applied=true` and `retired=true` on the client retirement event;
4. retain zero `r9_udp_return_packet_ack_rejected` and zero `r9_udp_return_packet_ack_accepted_empty` on the ordinary positive path;
5. require exactly one `r9_udp_post_return_settled` with `stream=1 offset=48 remaining_in_flight=0`;
6. retain exactly one Session transition `r9_udp_return_delivery_ack` with `stream=1 offset=48 len=16` and prove settlement is strictly after both that transition and the exact Carrier retirement.

Do not alter Session/Carrier/ACK/wire/crypto semantics merely to satisfy the test. If the strengthened oracle exposes a real contradiction, repair the smallest current-semantic defect, add a discriminating regression, gate it, push, and continue.

**R9-3 remains blocked until READY_LOCAL 1-4 are closed and READY_LOCAL 5 final provenance is recorded.**

# READY_LOCAL 2 — post-return ACK arrival-order challenge

Through the exact same authenticated post-return receive owner, same absolute deadline and same malformed budget, deterministically cover both:

1. Session DeliveryAck -> Carrier packet ACK;
2. Carrier packet ACK -> Session DeliveryAck.

Both must converge to exactly one logical confirmation, exactly one exact cross-bound packet retirement, zero false/rejected/accepted-empty shortcut, exactly one settled-zero result and both processes success. Reorder only already-produced authenticated ACK datagrams with a narrow test seam; do not add another protocol owner or policy value.

# READY_LOCAL 3 — P4 Session ACK present / Carrier ACK withheld

Add/strengthen one deterministic built-binary negative where the exact post-return Session DeliveryAck arrives but the Carrier ACK is withheld. Require:

- exact positive Session transition for `stream=1 offset=48 len=16`;
- Recovery remains unsettled;
- typed bounded nonzero terminal failure;
- no `r9_udp_post_return_settled` marker;
- no downstream health/failover/migration success continuation caused by a false premise.

# READY_LOCAL 4 — P4 Carrier ACK present / Session ACK withheld

Mirror the previous lane for the other evidence domain:

- exact positive Carrier retirement for the cross-bound post-return packet;
- logical Session confirmation remains outstanding;
- typed bounded nonzero terminal failure;
- no settled marker and no downstream success continuation.

These two P4 lanes prove neither evidence domain substitutes for the other.

# READY_LOCAL 5 — final R9-2 developer-local exact-tree provenance

After READY_LOCAL 1-4 are on one final pushed R9-2 source/test SHA, use a safe clean checkout/worktree and run at least:

```text
PYTHONDONTWRITEBYTECODE=1 bash scripts/check.sh
git diff --check
```

Persist exact reachable pushed SHA, UTC start/end, OS/arch, Rust stable version, both exit codes, clean initial/final tree, and clearly label the evidence developer-local. Run `cargo fuzz build decode` plus `cargo fuzz run decode -- -max_total_time=30 -max_len=8192` through `scripts/fuzz-toolchain.sh` only if this R9-2 closure actually changes decoder/parser/crypto framing. Hosted Actions remain separate cross-evidence. Continue directly to R9-3 after this gate; do not wait for the next reviewer.

# READY_LOCAL 6 — R9-3 Data-loss recovery

Suppress one reliable-owned Data only after congestion admission so the fault seam cannot bypass ownership accounting. Challenge the full current recovery contract:

- PTO fires only after its deadline;
- retransmission uses a fresh packet number / crypto nonce but stable frame/logical identity;
- receiver Session delivery remains exactly once;
- Session DeliveryAck and Carrier packet ACK remain separate evidence domains;
- positive completion reaches Recovery zero, otherwise failure is typed and bounded.

Use current M2 semantics; do not invent a new ACK or retransmission architecture.

# READY_LOCAL 7 — R9-4 ACK-loss + delayed original/reorder

Suppress one legitimate Carrier ACK, force a legitimate retransmission, then release the delayed original ACK. Require one logical Session delivery, Session dedup authority, fresh packet numbers/nonces, truthful PTO/loss/retransmit evidence and settled Recovery. A late original may be accepted-empty/stale but must never manufacture a second transition or rejection.

# READY_LOCAL 8 — R9-5 adversarial feedback / every Carrier caller

Challenge future/never-sent/stale/duplicate/tampered feedback across every exact-current `UdpAcknowledgement::Carrier` continuation, including initial receive, settlement and post-return owners. Rejected feedback must be atomic and fail closed without RTT/PTO/loss/cwnd/Session mutation; accepted-empty must not become transition or rejection. H-R9-033/H-R9-034 are closed examples, not reasons to skip the rest of the exact-current call surface.

# READY_LOCAL 9 — R9-6 ownership/resource boundedness

Every first send and retransmit must consult congestion admission before ownership commit; refusal commits no recovery or logical state. Keep one bounded retransmit-plaintext owner and deterministic teardown. Challenge algorithmic boundedness without adding a capacity-pressure benchmark or inventing new capacity/security values.

# READY_LOCAL 10 — R9-7 process/result truth

Independently challenge that Data, Carrier ACK, Session DeliveryAck, PTO/retransmit, Recovery, Session delivery, malformed budget, accepted-empty feedback, rejected feedback and terminal result remain distinct in structured process evidence. Emit/accept a diagnostic only after the exact claimed transition or classification has occurred.

# READY_LOCAL 11 — R9-8 warm TCP readiness

Challenge the existing single-active/multi-ready contract: authenticated resume-bound warm standby may carry readiness/control only and **no application Data before promotion**. Readiness/resource admission remains separate from packet feedback and Session delivery evidence. Use D064/current Carrier Manager semantics; no architecture change.

# READY_LOCAL 12 — R9-9/R9-10 health, promotion, uncertain replay and cleanup

Challenge the integrated path rather than isolated helpers:

- resolved UDP outcomes feed existing health/hysteresis;
- recoverable reliable-UDP loss remains on UDP instead of spuriously promoting TCP;
- promotion happens only after existing readiness/health gates;
- only genuine uncertain Session ranges replay on promoted TCP;
- draining/failed UDP receives no new Data ownership;
- receiver Session dedup is exactly-once;
- TCP gets no duplicate UDP packet-ACK layer;
- shutdown/error negatives clean resources and do not emit false success.

# READY_LOCAL 13 — R9-11/R9-12 complete cross-process slice

Close remaining lifecycle/terminal invariants for the materially new cross-process reliable-UDP/failover integration and run its clean exact-tree developer-local gate. Preserve source/test/evidence provenance boundaries; do not rewrite historical live evidence to describe the new local tree.

# READY_LOCAL 14 — dedicated independent R9 review + Q10/Q11/Q12 factual reconciliation

After the implementation/test lanes above are reachable and gated, perform a dedicated independent bounded challenge of materially new R9 send/admission/recovery/demux/Session ACK/Carrier ACK/failover/migration/cleanup/diagnostic behavior.

- A bounded no-finding review is valid release-item-4 support if scope, inspected owners, tests/commands, exclusions and reachable anchor are explicit.
- Any concrete BLOCKER/HIGH returns immediately to smallest repair -> discriminating regression -> exact-tree local gate.
- Only after a reachable independent review anchor should `docs/status.md`, `docs/release-security-review-packet.md` and related Q10/Q11/Q12 evidence text be factually reconciled.
- Do **not** change RC/freeze/release/production authority flags.

## Item-4 / core-surface inventory

Reachable independent bounded review already exists for earlier reliable-UDP engine basics (including the future/never-sent ACK guard), `CarrierState`, concurrent Carrier Manager/health/migration-back, FairScheduler/flow accounting, carrier adapters, `SessionRuntime`, observability (including mixed queue/generic drop classification), package/reproducibility/operator scripts, dependency/build surface, CLI portability/output, algorithmic boundedness/validators, DeliveryLedger/process codec/datagram/crypto API, and wire/parser surfaces.

The **materially new cross-process R9 integration** remains the broad uncovered item-4 surface until READY_LOCAL 14. Therefore repository-wide queue exhaustion is false even if any one narrow seam yields no finding.

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
