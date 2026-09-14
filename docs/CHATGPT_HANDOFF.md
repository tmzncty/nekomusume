# ChatGPT reviewer handoff — `3d7206a` advances P1 but leaves acceptance gaps; finish R9-2H evidence gate, then continue deep R9 queue

## Current repository truth

- Latest developer source/test SHA reviewed: exact `3d7206a73bf0638b1bdc0cb39bb527ca57b037df` (`test(cli): strengthen reversed-ACK applied-identity assertions (M-R9-008 P1)`).
- Independent bounded recheck: `docs/reviews/r9-2h-p1-evidence-recheck-3d7206a-20260915.md` (reviewer commit `df6e336`).
- Hosted GitHub checks on exact `3d7206a`: `stable checks` success; `nightly decode fuzz smoke` success. Hosted checks are supplementary only; no developer-local clean exact-tree provenance for the final P1-P4 tree is accepted yet.
- Open PRs: none. No new WAN/VPS experiment. `READY_LIVE: none` remains authoritative.
- Governance unchanged: item 3 incomplete; item 4 incomplete; `RELEASE_CANDIDATE=false`; `PRODUCTION_READY=false`; `FREEZE=false`; `RELEASED=false`.

The coding agent is explicitly pre-authorized to finish the remaining P1 assertions, then P2-P4 + exact-tree provenance, and immediately continue through R9-3..R9-12 and Q10/Q11/Q12 without waiting for reviewer cadence. Reviewer cadence is not a work-ticket boundary.

## Accepted implementation progress through `34ae142`

Prior R9-2H closures remain accepted:

- H-R9-011: out-of-order later Session DeliveryAck is buffered and cannot cumulatively confirm earlier unconfirmed bytes.
- H-R9-012/H-R9-013/H-R9-014: applied Session ACK evidence follows actual mutation order and carries exact `stream + offset`; buffered applied evidence carries `buffered=true`; logical `outstanding + pending_acks` must retire before Carrier-only settlement.
- one operation-wide malformed counter; one absolute application deadline; Carrier packet ACK remains Carrier-local; incomplete Recovery settlement is terminal and cannot feed downstream health/failover as success.
- `940b17f`: after successful migration-back the reserved final UDP record reuses the existing client `ReliableUdpRuntime`; `can_send` precedes ownership commit; `on_packet_sent` registers the exact secure-record sequence / bytes / stable `FrameId(record.offset)` / retained plaintext before socket send.
- `34ae142`: server post-return Data creates a Carrier ACK obligation only after authenticated/opened/decoded/correct-Session/accepted Session receive; client post-return completion requires both exact Session confirmation and `ReliableUdpRuntime.in_flight()==0`; Session DeliveryAck and Carrier ACK remain separate evidence domains and either order is allowed.

Do not reopen those source findings unless new process evidence contradicts them.

# R9-2H evidence front — READY_LOCAL, execute continuously

The implementation is ahead of its acceptance evidence. Finish these coherent evidence slices on real built binaries; do not stop after any one test.

## R9-2H-P1 — exact reversed-order built-process evidence — PARTIAL at `3d7206a`

`3d7206a` improves the test by requiring exactly two applied `r9_udp_delivery_ack_validated` lines and pinning the mutation order/identity as:

1. `stream=1, offset=0`, not `buffered=true`;
2. `stream=1, offset=16, buffered=true`.

This closes the earlier ambiguous applied-identity assertion, but P1 is **not yet complete**. Finish the same built-binary regression without changing protocol semantics:

1. require successful client exit;
2. require exactly one `r9_udp_delivery_ack_buffered` with `stream=1`, `offset=16`, `watermark=0`;
3. keep exactly two applied `r9_udp_delivery_ack_validated` events with the identities above;
4. require no `r9_udp_delivery_ack_covered` shortcut for either record;
5. prove logical `outstanding + pending_acks` is empty before Carrier settlement begins, using the existing evidence boundary rather than a new framework;
6. require the final relevant Recovery settlement event to carry `remaining_in_flight=0`, not merely any unrelated substring;
7. keep the order proof tied to those exact structured event lines, not generic `offset` occurrences elsewhere in diagnostics.

The new exact applied-event checks are accepted progress; do not remove them.

## R9-2H-P2 — exact reserved post-migration ownership + dual settlement

Strengthen `reliable_udp_migration_back_reserves_final_record`. The current test still permits nonzero client exit and only rejects one legacy `udp_uncertain_range_sent` shape for reserved offset 32, so it does not yet prove the accepted `940b17f`/`34ae142` source behavior.

For exact reserved fixture record `stream=1, offset=32`, require successful built-binary completion and exact structured evidence that:

1. no pre-promotion Recovery ownership exists for offset 32;
2. no legacy pre-promotion `udp_uncertain_range_sent` owns offset 32;
3. `udp_migrated_back` precedes exactly one `r9_udp_post_return_sent` for that exact record;
4. server accepts that authenticated post-return Data and emits exactly one independent Session DeliveryAck and one independent Carrier packet ACK obligation;
5. client applies the exact Session DeliveryAck once;
6. client applies the Carrier ACK through Recovery and post-return `in_flight` reaches zero before success;
7. Carrier ACK first and Session DeliveryAck first are both accepted ordering cases.

A final structured evidence event such as `r9_udp_post_return_settled` carrying exact stream/offset and `remaining_in_flight=0` is acceptable if needed for unambiguous assertions. It is evidence only, not new protocol semantics.

## R9-2H-P3 — persistent malformed budget across valid Carrier feedback

Drive one reliable receive/settlement operation with authenticated sequence:

```text
malformed #1
malformed #2
canonical Carrier ACK
malformed #3
```

Malformed #3 must hit the existing `MAX_POST_HANDSHAKE_MALFORMED` ceiling. The valid Carrier ACK must not reset the operation-wide malformed counter. Termination must be typed, finite and non-spinning. A bounded test-only server seam may emit authenticated malformed plaintext; do not change the numeric limit.

## R9-2H-P4 — preserve incomplete settlement terminality, including post-return

Keep `reliable_udp_incomplete_settlement_fails_not_settled`: remaining in-flight != 0 is nonzero/terminal, emits incomplete, never emits `r9_udp_in_flight_settled`, and never feeds downstream health/failover from a false premise.

Extend the same truth boundary to the post-migration reserved-record owner: suppress/miss one of the two independent evidence domains, require nonzero terminal outcome, no post-return settled/success claim, and no downstream continuation based on incomplete ownership.

## R9-2H-GATE — exact pushed-tree developer-local provenance

After P1-P4 land together on one reachable source/test SHA, use a safe clean worktree and persist:

```text
PYTHONDONTWRITEBYTECODE=1 bash scripts/check.sh
git diff --check
```

Record exact pushed SHA, UTC start/end, Linux OS/arch, Rust stable version, both exit codes, and clean initial/final tree. No decoder/parser/crypto-framing change in `3d7206a`; do not invent a new fuzz obligation. Hosted fuzz remains supplementary cross-evidence.

R9-2H closes only when one reachable tree proves exact reversed logical confirmation, direct+buffered Session applied identity, bounded pending ownership, one operation-wide malformed budget/deadline, pre/post-migration reliable ownership, independent Session + Carrier settlement, exact reserved ownership and incomplete-settlement terminality.

**Then continue immediately to R9-3. Do not wait for reviewer cadence.**

# Continuous queue after R9-2H

Preserve this deep queue. Do not collapse it after one repair or one green test.

## R9-3 — process Data-loss recovery

Add bounded deterministic post-admission suppression of one/periodic reliable-owned Data packet. Prove congestion admission precedes suppression; PTO fires only after deadline; retransmission uses a fresh authenticated packet number/nonce with stable Session/frame identity; application delivery is exactly once; Carrier ACK and Session DeliveryAck settle independently; final Recovery drains to zero or produces explicit bounded incomplete/error.

## R9-4 — ACK-loss + reorder / delayed original

Cover Carrier packet ACK emitted then suppressed, retransmitted replacement before delayed original, and delayed original after replacement. Require one logical Session delivery, Session-owned duplicate suppression, no conflict, fresh packet numbers/nonces, truthful ACK-loss/PTO counters and final settlement.

## R9-5 — tamper / future ACK / malformed-feedback negatives

Prove tampered Data creates no Carrier ACK obligation or Session receive; tampered ACK creates no recovery mutation; future/never-sent ACK is typed rejected atomically; stale/duplicate ACK fabricates no RTT/loss/Session evidence; malformed feedback is finite and panic-free.

## R9-6 — pacing / cwnd / plaintext-owner atomicity

Prove initial and retransmit sends consult congestion admission; refusal consumes no Session byte space and commits no packet/plaintext/recovery owner; retransmit plaintext has one bounded owner; pacing deadlines are finite; teardown releases ownership.

## R9-7 — truthful process observability/result contract

Keep Data, Carrier ACK, Session DeliveryAck, PTO/retransmit, resolved recovery, Session delivery, malformed-budget, cleanup and final-outcome evidence domains distinct. Counters/events increment only after the action they claim succeeds.

## R9-8 — actual authenticated warm TCP standby

Reuse existing TCP connect/accept, canonical negotiation, fresh Noise trust/authz, resume/readiness tuple+generation+delivery-epoch binding and resource admission. Warm/standby carries no application Data before atomic promotion.

## R9-9 — resolved UDP health -> hysteresis -> real TCP promotion

Fresh resolved UDP outcomes drive Carrier health. Recoverable loss stays on UDP; PTO-only observation cannot erase later resolved loss; distinct bad resolved intervals cross committed hysteresis and promote only to an actually ready TCP standby. Invalid/unready standby yields typed failure.

## R9-10 — uncertain Session replay across UDP -> TCP

At promotion replay at least one genuinely uncertain logical Session range over promoted TCP. Draining UDP accepts no new application Data. Receiver deduplicates by Session/stream/offset and application bytes remain exactly once. Do not add TCP packet ACK.

## R9-11 — timeout / shutdown / cleanup matrix

Cover setup/application/PTO/single-owner receive deadline, controlled stop, failed standby promotion, malformed/tampered input, listener/socket/process cleanup and same-address rebind where supported.

## R9-12 — coherent exact-tree gate + independent bounded review

Independently challenge Session-above-Carrier layering, exact Session ACK evidence, authenticated packet identity/Carrier ACK, bounded plaintext/pending-ACK ownership, single receive-owner classification + operation-wide malformed/deadline ownership, PTO/pacing/cwnd, fresh health/hysteresis, real warm TCP readiness/promotion, uncertain Session replay/dedup, result truthfulness and cleanup/no-public-exposure. BLOCKER/HIGH -> smallest repair + regression + re-gate + continue; LOW/NOTE does not halt progression.

## Q10 — observability reconciliation

Integrate only genuinely new R9 evidence into existing `neko-observe`/result surfaces. Do not create a second logging framework.

## Q11 — factual status/release reconciliation

Reconcile `docs/status.md`, `IMPLEMENTATION_PLAN.md`, `ROADMAP.md` and the release packet only for evidence actually earned by the final R9 tree. Only a green independently reviewed cross-process R9 plus real TCP promotion may create a new specific `READY_LIVE` row.

## Q12 — one changed-hypothesis self-owned VPS run

Only after Q11 creates a specific `READY_LIVE` question, execute exactly one minimal bounded self-owned client<->VPS run under standing authorization. Record exact commit/binary/parameters, authenticated recovery/Session/switch evidence and cleanup. Preserve negative evidence; no unchanged same-class retry.

# VPS opportunity

**Not READY — local evidence dependency.** Unlock chain: remaining P1 assertions + P2/P3/P4 + clean exact-tree provenance -> R9-3..R9-12 -> Q10/Q11. Standing authorization already covers the eventual bounded self-owned TCP/UDP run once a specific `READY_LIVE` row exists; do not ask for generic WAN permission.

# Non-blocking policy/authority gates

Keep separate from this mechanically determined R9 chain:

- `SessionRuntime.events` retention (`POLICY_BLOCKED_RESOURCE_BOUND`);
- D019 source-retention/no-reset policy;
- RSEC-001 capacity/adversarial-load suitability;
- signing/key custody/SBOM/publication trust;
- previous frozen-release interoperability;
- final RC/freeze/release/production authority.

Do not invent policy values while working R9.
