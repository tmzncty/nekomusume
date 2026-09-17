# ChatGPT reviewer handoff — H-R9-037 independently closed; R9-3 blocked on dual P4 + provenance

## Current repository truth

- Latest developer-owned source/test commit reviewed: exact `d5d5b23dd007c4293e9df7e0eb9aa2bee828bc74` (`test(cli): H-R9-037 reverse-order true three-way packet bind — client send == server ACK == client retire`).
- Independent bounded no-finding review: [`docs/reviews/reviewer-r9-h-r9-037-close-d5d5b23-20260917.md`](reviews/reviewer-r9-h-r9-037-close-d5d5b23-20260917.md), reachable through reviewer commit `f2529087d4898d5d177adc0e41fe986aa93fe0c0`.
- H-R9-037 is closed at exact `d5d5b23`: the reverse-order fixture requires both processes to succeed, exactly one client post-return send, exactly one server Carrier ACK, exactly one positive client Carrier retirement, exact three-way packet-number equality, exactly one Session transition `stream=1 offset=48 len=16`, zero rejected/accepted-empty classification, exactly one `r9_udp_post_return_settled` containing `remaining_in_flight=0`, Carrier-before-Session order, and settlement after both transitions.
- GitHub-hosted Rust CI run `35179469348` for exact `d5d5b23` is SUCCESS: `stable checks` and `nightly decode fuzz smoke` both completed successfully. This is hosted cross-evidence only.
- Developer-local clean exact-tree provenance for `d5d5b23` remains repository-recorded separately: `PYTHONDONTWRITEBYTECODE=1 bash scripts/check.sh` exit 0, `git diff --check` exit 0, clean worktree at pushed SHA, 2026-09-17T03:49:17Z → 2026-09-17T03:53:01Z, Linux x86_64, rustc 1.98.0 (88d9e12ae 2026-08-18). The reviewer did not relabel this as reviewer-executed evidence.
- P2 C1-C4 remain closed. H-R9-036/H-R9-035/H-R9-034/H-R9-033/H-R9-032 and earlier repairs remain closed. Candidate A future/never-sent ACK guard and candidate B mixed queue/generic-drop observability repair remain closed.
- Open PRs at review time: none.
- `READY_LIVE: none`; release item 3 incomplete; release item 4 incomplete; `RELEASE_CANDIDATE=false`; `PRODUCTION_READY=false`; `FREEZE=false`; `RELEASED=false`.

The external coding agent must synchronize to current `main` and continuously execute every dependency-ready slice below: implementation/review -> focused deterministic tests -> commit -> push -> next slice. Reviewer cadence is only a check frequency and is never a work-ticket length or reason to idle.

## Accepted progress that must remain closed

Do not revert these exact-current repairs without contradictory evidence:

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

# READY_LOCAL 1 — P4 Session ACK present / post-return Carrier ACK withheld

Create one deterministic built-binary negative that reaches the existing post-return reliable owner, sends the exact Session DeliveryAck, but withholds only the **post-return** Carrier packet ACK.

Exact-current implementation facts:

- historical `--suppress-r9-ack` is wired to the initial reliable-UDP packet-ACK path and is too broad/wrong-phase for this question;
- the post-return server owner already consumes the pending ACK obligation once into retained `post_ack_pack`, sends Session DeliveryAck, then on the ordinary path sends `post_ack_pack` afterward;
- therefore the smallest seam is a post-return-only Carrier-ACK suppression flag at the ordinary post-return send branch. Do not fold that flag into stale/future/reverse `seam_active`, because those seams intentionally change ACK ordering/classification. Consuming `post_ack_pack` and deliberately not sending it is sufficient for this fault injection; do not redesign ACK generation or Recovery.

Prefer one common classification-only terminal diagnostic immediately before the existing post-return dual-settlement timeout if needed to make both P4 negatives discriminating. A suitable existing-semantics shape is to report the exact two remaining domains, for example Session outstanding count plus Recovery `remaining_in_flight`; this diagnostic must not mutate Session/Recovery, add a policy value, or become a success signal.

Require:

- server process succeeds and proves the post-return owner was reached;
- exactly one server `udp_return_delivery_ack_sent` and zero server `udp_return_packet_ack_sent` for the post-return packet;
- exactly one positive client Session transition `r9_udp_return_delivery_ack` with `stream=1 offset=48 len=16`;
- zero positive client `r9_udp_return_packet_ack` for the post-return packet;
- if the terminal diagnostic is added, it must show Session complete but Recovery still nonzero (`session_outstanding=0`, `remaining_in_flight>0`) immediately before the existing bounded terminal failure;
- client exits typed/nonzero on the existing post-return bounded deadline; use the exact current terminal path rather than a new sleep/policy timeout;
- no `r9_udp_post_return_settled` and no downstream health/failover/migration success continuation caused by a false settlement premise.

Run the focused process test first, then the normal exact-tree local gate for the pushed developer SHA. No decoder/parser/crypto-framing change is expected; do not mechanically run fuzz for a CLI seam/test-only repair.

# READY_LOCAL 2 — P4 Carrier ACK present / Session ACK withheld

The existing post-return `--suppress-r9-dack` seam already has the correct runtime shape: it withholds only the Session DeliveryAck while the real Carrier ACK is sent. Strengthen `reliable_udp_post_return_incomplete_is_terminal` before changing runtime semantics.

Current test is intentionally not yet acceptance-grade: it ignores server status/log, proves only client nonzero + absent settlement + broad error text, and does not prove that the Carrier domain really completed while the Session domain remained outstanding.

Require:

- server process succeeds;
- exactly one client `r9_udp_post_return_sent` and exactly one server `udp_return_packet_ack_sent`;
- exactly one positive client `r9_udp_return_packet_ack` with `applied=true, retired=true`;
- exact three-way packet-number equality between client send, server Carrier ACK and client retirement;
- zero server `udp_return_delivery_ack_sent` for the post-return offset and zero client `r9_udp_return_delivery_ack` transition for offset 48;
- if the common terminal diagnostic from READY_LOCAL 1 is added, it must show Recovery settled but Session still outstanding (`remaining_in_flight=0`, `session_outstanding=1`) immediately before bounded failure;
- client exits typed/nonzero on the existing post-return dual-settlement deadline;
- no `r9_udp_post_return_settled`, no accepted-empty/rejected substitute for the real positive Carrier retirement, and no downstream success continuation.

Only repair runtime semantics if this strengthened oracle exposes a real contradiction. READY_LOCAL 1+2 together must prove neither evidence domain substitutes for the other.

# READY_LOCAL 3 — final R9-2 developer-local exact-tree provenance

After READY_LOCAL 1-2 are on one final pushed R9-2 source/test SHA, use a safe clean checkout/worktree and run at least:

```text
PYTHONDONTWRITEBYTECODE=1 bash scripts/check.sh
git diff --check
```

Persist exact reachable pushed SHA, UTC start/end, OS/arch, Rust stable version, both exit codes, clean initial/final tree, and clearly label the evidence developer-local. Run `cargo fuzz build decode` plus `cargo fuzz run decode -- -max_total_time=30 -max_len=8192` through `scripts/fuzz-toolchain.sh` only if this R9-2 closure actually changes decoder/parser/crypto framing. Hosted Actions remain separate cross-evidence. Continue directly to R9-3 after this gate; do not wait for the next reviewer.

# READY_LOCAL 4 — R9-3 Data-loss recovery

Suppress one reliable-owned Data only after congestion admission so the fault seam cannot bypass ownership accounting. Challenge the full current recovery contract:

- PTO fires only after its deadline;
- retransmission uses a fresh packet number / crypto nonce but stable frame/logical identity;
- receiver Session delivery remains exactly once;
- Session DeliveryAck and Carrier packet ACK remain separate evidence domains;
- positive completion reaches Recovery zero, otherwise failure is typed and bounded.

Use current M2 semantics; do not invent a new ACK or retransmission architecture.

# READY_LOCAL 5 — R9-4 ACK-loss + delayed original/reorder

Suppress one legitimate Carrier ACK, force a legitimate retransmission, then release the delayed original ACK. Require one logical Session delivery, Session dedup authority, fresh packet numbers/nonces, truthful PTO/loss/retransmit evidence and settled Recovery. A late original may be accepted-empty/stale but must never manufacture a second transition or rejection.

# READY_LOCAL 6 — R9-5 adversarial feedback / every Carrier caller

Challenge future/never-sent/stale/duplicate/tampered feedback across every exact-current `UdpAcknowledgement::Carrier` continuation, including initial receive, settlement and post-return owners. Rejected feedback must be atomic and fail closed without RTT/PTO/loss/cwnd/Session mutation; accepted-empty must not become transition or rejection. H-R9-033/H-R9-034 are closed examples, not reasons to skip the rest of the exact-current call surface.

# READY_LOCAL 7 — R9-6 ownership/resource boundedness

Every first send and retransmit must consult congestion admission before ownership commit; refusal commits no recovery or logical state. Keep one bounded retransmit-plaintext owner and deterministic teardown. Challenge algorithmic boundedness without adding a capacity-pressure benchmark or inventing new capacity/security values.

# READY_LOCAL 8 — R9-7 process/result truth

Independently challenge that Data, Carrier ACK, Session DeliveryAck, PTO/retransmit, Recovery, Session delivery, malformed budget, accepted-empty feedback, rejected feedback and terminal result remain distinct in structured process evidence. Emit/accept a diagnostic only after the exact claimed transition or classification has occurred.

# READY_LOCAL 9 — R9-8 warm TCP readiness

Challenge the existing single-active/multi-ready contract: authenticated resume-bound warm standby may carry readiness/control only and **no application Data before promotion**. Readiness/resource admission remains separate from packet feedback and Session delivery evidence. Use D064/current Carrier Manager semantics; no architecture change.

# READY_LOCAL 10 — R9-9/R9-10 health, promotion, uncertain replay and cleanup

Challenge the integrated path rather than isolated helpers:

- resolved UDP outcomes feed existing health/hysteresis;
- recoverable reliable-UDP loss remains on UDP instead of spuriously promoting TCP;
- promotion happens only after existing readiness/health gates;
- only genuine uncertain Session ranges replay on promoted TCP;
- draining/failed UDP receives no new Data ownership;
- receiver Session dedup is exactly-once;
- TCP gets no duplicate UDP packet-ACK layer;
- shutdown/error negatives clean resources and do not emit false success.

# READY_LOCAL 11 — R9-11/R9-12 complete cross-process slice

Close remaining lifecycle/terminal invariants for the materially new cross-process reliable-UDP/failover integration and run its clean exact-tree developer-local gate. Preserve source/test/evidence provenance boundaries; do not rewrite historical live evidence to describe the new local tree.

# READY_LOCAL 12 — dedicated independent R9 review

After the implementation/test lanes above are reachable and gated, perform a dedicated independent bounded challenge of materially new R9 send/admission/recovery/demux/Session ACK/Carrier ACK/failover/migration/cleanup/diagnostic behavior.

A bounded no-finding review is valid release-item-4 support if scope, inspected owners, tests/commands, exclusions and reachable anchor are explicit. Any concrete BLOCKER/HIGH returns immediately to smallest repair -> discriminating regression -> exact-tree local gate.

# READY_LOCAL 13 — Q10/Q11/Q12 factual reconciliation

Only after a reachable independent R9 review anchor exists, factually reconcile `docs/status.md`, `docs/release-security-review-packet.md` and the related Q10/Q11/Q12 evidence text with the exact reachable implementation/review anchors. Preserve historical evidence boundaries and do **not** change RC/freeze/release/production authority flags.

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