# ChatGPT reviewer handoff — HIGH H-R9-037 open at 855c679; R9-3 blocked on exact reverse-order proof + dual P4 + provenance

## Current repository truth

- Latest developer-owned source/test commit reviewed: exact `855c679a4085fded7382d081299ad1f6b3d915e9` (`test(cli): H-R9-036 reverse-order exact oracle — three-way packet bind + cardinality + Session exact`).
- Independent reviewer checkpoint: [`docs/reviews/reviewer-r9-h-r9-037-reverse-order-server-bind-855c679-20260917.md`](reviews/reviewer-r9-h-r9-037-reverse-order-server-bind-855c679-20260917.md), reachable through reviewer commit `d5541b5b420852351fa0bee3bc2a4d77354061e9`.
- The prior handoff commit `e670619a04d08c9b8b77297064ea299daeaf29bd` incorrectly closed H-R9-036 by claiming three-way client-send/server-ACK/client-retirement packet-number equality and settled-zero proof. Exact `855c679` does **not** inspect the server `udp_return_packet_ack_sent` event in the reverse-order test, does not require exactly one client post-return send, compares only client send == client retirement, and does not assert the settlement line contains `remaining_in_flight=0`. This is reopened as **H-R9-037 HIGH — evidence/oracle correctness**.
- This finding does **not** yet establish a new runtime state-machine defect. The exact-current runtime already emits the missing server packet-number diagnostic, and the positive P2 fixture already contains the required exact cardinality / three-way binding pattern. Treat the repair as test-only first.
- P2 C1-C4 remain closed; H-R9-035/H-R9-034/H-R9-033/H-R9-032 and earlier repairs remain closed. Candidate A future/never-sent ACK guard and candidate B mixed queue/generic-drop observability repair remain closed.
- Developer-local clean exact-tree provenance recorded for `855c679`: `PYTHONDONTWRITEBYTECODE=1 bash scripts/check.sh` exit 0, `git diff --check` exit 0, clean worktree at pushed SHA, 2026-09-17T02:51:26Z → 2026-09-17T02:55:15Z, Linux x86_64, rustc 1.98.0 (88d9e12ae 2026-08-18). This remains provenance for the exact weak-oracle tree; it does not close H-R9-037.
- GitHub-hosted Rust CI run `35175799014` for exact `855c679` completed SUCCESS: `stable checks` and `nightly decode fuzz smoke` both green. Hosted CI remains cross-evidence, not developer-local provenance and not a substitute for missing assertions.
- Open PRs at review time: none.
- `READY_LIVE: none`; release item 3 incomplete; release item 4 incomplete; `RELEASE_CANDIDATE=false`; `PRODUCTION_READY=false`; `FREEZE=false`; `RELEASED=false`.

The external coding agent must synchronize to current `main` and continuously execute every dependency-ready slice below: implementation/review -> focused deterministic tests -> commit -> push -> next slice. Reviewer cadence is only a check frequency and is never a work-ticket length or reason to idle.

## Accepted progress that must remain closed

Do not revert these exact-current repairs without contradictory evidence:

- P2 C1-C4 exact closure at `142f0a1` + `8d0ccd5` + `1f8bbb0` + `be03dc3`: TCP replay identity bound on both sides; migration milestones exact cardinality/order; server dual-domain stream/offset/len; positive-path three-way packet-number equality; exactly-one settled cardinality and post-transition order.
- `424583d` source repair: ordinary no-seam post-return path no longer sends a duplicate Carrier ACK; ordinary ordering remains Session DeliveryAck then Carrier ACK, while `--reverse-post-return-ack` sends Carrier ACK before Session DeliveryAck. H-R9-037 reopens only the **reverse-order process oracle**, not this source-side seam.
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
- candidate A future/never-sent ACK engine guard and candidate B mixed queue/generic-drop observability repair.

# READY_LOCAL 1 — HIGH: H-R9-037 reverse ACK-order exact server binding / settled-zero oracle

Strengthen `crates/neko-cli/tests/probe.rs::reliable_udp_post_return_reversed_ack_order_settles` at the exact-current runtime before continuing to the P4 negatives.

The exact test must require all of the following:

1. client and server both succeed;
2. exactly one client `r9_udp_post_return_sent`;
3. exactly one server `udp_return_packet_ack_sent`;
4. exactly one positive client `r9_udp_return_packet_ack` with `applied=true` and `retired=true`;
5. exact equality of the client-send / server-ACK / client-retirement `packet_number` values;
6. exactly one client `r9_udp_return_delivery_ack` with `stream=1`, `offset=48`, `len=16`;
7. zero post-return rejected and accepted-empty Carrier classifications;
8. exactly one `r9_udp_post_return_settled`, and that exact line contains `remaining_in_flight=0`;
9. the positive Carrier retirement precedes the Session transition, and settlement is strictly after both actual transitions.

Use the existing `packet_number(...)` helper and the exact cardinality / server-log binding pattern already present in the positive P2 and stale/future post-return fixtures. Prefer a test-only repair. Only change runtime code if the strengthened oracle exposes a real contradiction. Do not change Session/Carrier/ACK/wire/crypto architecture, deadline values, capacity values, D019 policy, or release authority.

After the focused regression passes, run the ordinary developer-local exact-tree gate on the final pushed source/test SHA:

```text
PYTHONDONTWRITEBYTECODE=1 bash scripts/check.sh
git diff --check
```

Record exact reachable pushed SHA, UTC start/end, OS/arch, Rust stable version, both exit codes and clean-tree state. No decoder/parser/crypto framing change is expected; do not mechanically run fuzz for a test-only repair.

# READY_LOCAL 2 — P4 Session ACK present / post-return Carrier ACK withheld

Create/strengthen one deterministic built-binary negative that reaches the existing post-return reliable owner, sends the exact Session DeliveryAck, but withholds only the **post-return** Carrier packet ACK.

Do not use a broad suppression that prevents the run from reaching migration-back. Historical `--suppress-r9-ack` is wired to the initial reliable-UDP packet-ACK path; either add a post-return-only Carrier-ACK suppression seam or make an exact-current refactor whose scope is demonstrably equivalent.

Require:

- exact positive Session transition `stream=1 offset=48 len=16`;
- no positive post-return Carrier retirement;
- Recovery remains nonzero/unsettled until the bounded terminal path;
- client exits typed/nonzero within the existing bounded post-return deadline;
- no `r9_udp_post_return_settled` marker;
- no downstream health/failover/migration success continuation caused by a false settlement premise.

If current process evidence cannot distinguish which domain remains outstanding, a minimal classification-only terminal diagnostic is allowed; it must not alter Session/Recovery state or invent a policy value.

# READY_LOCAL 3 — P4 Carrier ACK present / Session ACK withheld

The existing post-return `--suppress-r9-dack` seam already has the correct runtime shape: it withholds only the Session DeliveryAck while the real Carrier ACK is sent. Strengthen the existing `reliable_udp_post_return_incomplete_is_terminal` oracle before changing runtime semantics.

Require:

- exactly one positive Carrier retirement `applied=true, retired=true` for the exact post-return packet;
- exact three-way packet-number equality between client send, server Carrier ACK and client retirement;
- no client `r9_udp_return_delivery_ack` transition for offset 48;
- logical Session confirmation remains outstanding until the bounded terminal failure;
- client exits typed/nonzero;
- no `r9_udp_post_return_settled` marker and no downstream success continuation.

Only repair runtime code if the strengthened oracle exposes a real contradiction. READY_LOCAL 2+3 together prove neither evidence domain substitutes for the other.

# READY_LOCAL 4 — final R9-2 developer-local exact-tree provenance

After READY_LOCAL 1-3 are on one final pushed R9-2 source/test SHA, use a safe clean checkout/worktree and run at least:

```text
PYTHONDONTWRITEBYTECODE=1 bash scripts/check.sh
git diff --check
```

Persist exact reachable pushed SHA, UTC start/end, OS/arch, Rust stable version, both exit codes, clean initial/final tree, and clearly label the evidence developer-local. Run `cargo fuzz build decode` plus `cargo fuzz run decode -- -max_total_time=30 -max_len=8192` through `scripts/fuzz-toolchain.sh` only if this R9-2 closure actually changes decoder/parser/crypto framing. Hosted Actions remain separate cross-evidence. Continue directly to R9-3 after this gate; do not wait for the next reviewer.

# READY_LOCAL 5 — R9-3 Data-loss recovery

Suppress one reliable-owned Data only after congestion admission so the fault seam cannot bypass ownership accounting. Challenge the full current recovery contract:

- PTO fires only after its deadline;
- retransmission uses a fresh packet number / crypto nonce but stable frame/logical identity;
- receiver Session delivery remains exactly once;
- Session DeliveryAck and Carrier packet ACK remain separate evidence domains;
- positive completion reaches Recovery zero, otherwise failure is typed and bounded.

Use current M2 semantics; do not invent a new ACK or retransmission architecture.

# READY_LOCAL 6 — R9-4 ACK-loss + delayed original/reorder

Suppress one legitimate Carrier ACK, force a legitimate retransmission, then release the delayed original ACK. Require one logical Session delivery, Session dedup authority, fresh packet numbers/nonces, truthful PTO/loss/retransmit evidence and settled Recovery. A late original may be accepted-empty/stale but must never manufacture a second transition or rejection.

# READY_LOCAL 7 — R9-5 adversarial feedback / every Carrier caller

Challenge future/never-sent/stale/duplicate/tampered feedback across every exact-current `UdpAcknowledgement::Carrier` continuation, including initial receive, settlement and post-return owners. Rejected feedback must be atomic and fail closed without RTT/PTO/loss/cwnd/Session mutation; accepted-empty must not become transition or rejection. H-R9-033/H-R9-034 are closed examples, not reasons to skip the rest of the exact-current call surface.

# READY_LOCAL 8 — R9-6 ownership/resource boundedness

Every first send and retransmit must consult congestion admission before ownership commit; refusal commits no recovery or logical state. Keep one bounded retransmit-plaintext owner and deterministic teardown. Challenge algorithmic boundedness without adding a capacity-pressure benchmark or inventing new capacity/security values.

# READY_LOCAL 9 — R9-7 process/result truth

Independently challenge that Data, Carrier ACK, Session DeliveryAck, PTO/retransmit, Recovery, Session delivery, malformed budget, accepted-empty feedback, rejected feedback and terminal result remain distinct in structured process evidence. Emit/accept a diagnostic only after the exact claimed transition or classification has occurred.

# READY_LOCAL 10 — R9-8 warm TCP readiness

Challenge the existing single-active/multi-ready contract: authenticated resume-bound warm standby may carry readiness/control only and **no application Data before promotion**. Readiness/resource admission remains separate from packet feedback and Session delivery evidence. Use D064/current Carrier Manager semantics; no architecture change.

# READY_LOCAL 11 — R9-9/R9-10 health, promotion, uncertain replay and cleanup

Challenge the integrated path rather than isolated helpers:

- resolved UDP outcomes feed existing health/hysteresis;
- recoverable reliable-UDP loss remains on UDP instead of spuriously promoting TCP;
- promotion happens only after existing readiness/health gates;
- only genuine uncertain Session ranges replay on promoted TCP;
- draining/failed UDP receives no new Data ownership;
- receiver Session dedup is exactly-once;
- TCP gets no duplicate UDP packet-ACK layer;
- shutdown/error negatives clean resources and do not emit false success.

# READY_LOCAL 12 — R9-11/R9-12 complete cross-process slice

Close remaining lifecycle/terminal invariants for the materially new cross-process reliable-UDP/failover integration and run its clean exact-tree developer-local gate. Preserve source/test/evidence provenance boundaries; do not rewrite historical live evidence to describe the new local tree.

# READY_LOCAL 13 — dedicated independent R9 review + Q10/Q11/Q12 factual reconciliation

After the implementation/test lanes above are reachable and gated, perform a dedicated independent bounded challenge of materially new R9 send/admission/recovery/demux/Session ACK/Carrier ACK/failover/migration/cleanup/diagnostic behavior.

- A bounded no-finding review is valid release-item-4 support if scope, inspected owners, tests/commands, exclusions and reachable anchor are explicit.
- Any concrete BLOCKER/HIGH returns immediately to smallest repair -> discriminating regression -> exact-tree local gate.
- Only after a reachable independent review anchor should `docs/status.md`, `docs/release-security-review-packet.md` and related Q10/Q11/Q12 evidence text be factually reconciled.
- Do **not** change RC/freeze/release/production authority flags.

## Item-4 / core-surface inventory

Reachable independent bounded review already exists for earlier reliable-UDP engine basics (including the future/never-sent ACK guard), `CarrierState`, concurrent Carrier Manager/health/migration-back, FairScheduler/flow accounting, carrier adapters, `SessionRuntime`, observability (including mixed queue/generic drop classification), package/reproducibility/operator scripts, dependency/build surface, CLI portability/output, algorithmic boundedness/validators, DeliveryLedger/process codec/datagram/crypto API, and wire/parser surfaces.

The **materially new cross-process R9 integration** remains the broad uncovered item-4 surface until READY_LOCAL 13. Therefore repository-wide queue exhaustion is false even if any one narrow seam yields no finding.

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