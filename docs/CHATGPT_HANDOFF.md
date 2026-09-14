# ChatGPT reviewer handoff — R9-2E partially accepted; single-owner demux closure still required

## Reviewed repository truth

- Latest developer source/test SHA: exact `8bd87e388745ae175f8b188f13936f1c1fe86ea4` (`fix(cli): R9-2E bounded authenticated receive/demux owner`).
- Exact `42f0c9f14321994f17e6d91795743d62f6acada1` is a docs-only descendant recording local provenance for the same `crates/neko-cli/src/main.rs` blob.
- This reviewer pass adds `docs/reviews/independent-r9-demux-recheck-8bd87e3-20260914.md` at docs-only exact `1a43e8aa56b72e598c227452f222af4e596d1487`.
- Developer-local provenance records `cargo test -p neko-cli`, workspace clippy, `PYTHONDONTWRITEBYTECODE=1 bash scripts/check.sh`, `git diff --check`, and clean-tree success for a code-identical measured tree; current hosted `stable checks` and `nightly decode fuzz smoke` are green and remain separate cross-evidence.
- Open PRs: none. No WAN/VPS execution occurred in this sequence. `READY_LIVE: none` remains authoritative.
- Governance unchanged: item 3 incomplete; item 4 incomplete; `RELEASE_CANDIDATE=false`; `PRODUCTION_READY=false`; `FREEZE=false`; `RELEASED=false`.

## Reviewer verdict on `8bd87e3`

### Original order-sensitive logical-ACK defect — PARTIALLY ACCEPTED

The new helper now accepts a Session `DeliveryAck` for any currently outstanding reliable-owned logical record and returns the actual matched `OutboundRecord`; the caller applies `SessionRuntime::delivery_ack` to that record. A later logical ACK is therefore no longer silently consumed merely because an earlier record is still outstanding.

The focused loopback/authenticated unit regression is discriminating for this matching bug. Carrier `apply_ack` rejection is also no longer discarded through `let _ = ...` at this first receive boundary.

However R9-2 is **not closed**. The implementation still does not satisfy the single bounded authenticated demux-owner contract required before R9-3.

## H-R9-006 — OPEN HIGH: malformed budget resets across valid/rejected Carrier ACKs

`recv_udp_delivery_ack` owns `let mut malformed = 0usize` locally, then returns on every matched Session DeliveryAck **and every canonical Carrier ACK**, including `apply_ack` rejection. The caller invokes the helper again while logical confirmations remain.

Therefore `MAX_POST_HANDSHAKE_MALFORMED` is not a finite budget over the reliable receive operation. An authenticated peer can interleave, for example, two malformed/unexpected plaintexts with one canonical Carrier ACK and repeatedly reset the counter. Rejected Carrier ACKs also trigger the reset.

The absolute deadline bounds elapsed time but does not make the advertised malformed-count ceiling truthful.

### Required repair

Make invalid-input accounting persistent across the entire reliable receive/settlement owner. Prefer one stateful owner loop; if classification remains factored into a helper, pass one persistent mutable budget/counter state through every call.

Add a discriminating regression: authenticated malformed #1, malformed #2, canonical Carrier ACK, malformed #3 must still trip the third-malformed bound rather than reset after the ACK.

## H-R9-007 — OPEN HIGH: settlement is still a second untyped receive owner

Once `outstanding` is empty, `failover_client` leaves the demux helper and starts a separate settlement loop. That loop reparses authenticated datagrams itself.

Current consequences:

- authenticated non-ACK plaintext in settlement is silently ignored instead of consuming the same finite malformed/ignored budget and typed classification path;
- successful Carrier ACK applications in settlement do not increment the earlier `packet_ack_applied` counter;
- `r9_udp_packet_ack_outcomes` is emitted **before** settlement, so its totals may omit the ACKs that actually drain recovery;
- the claimed invariant “every successfully authenticated plaintext is classified exactly once by one bounded owner” is therefore still false over the complete operation.

### Required repair

Use the same classifier/owner for logical-confirmation and Carrier-settlement phases. Do not keep a second ad-hoc Carrier-ACK decoder loop.

The reliable receive owner completes only when:

1. all required reliable-owned Session DeliveryAck expectations have been applied exactly once; **and**
2. `ReliableUdpRuntime::in_flight()==0`.

Otherwise it ends with explicit bounded timeout/partial failure. Final Carrier ACK applied/rejected totals are emitted only after settlement and include all ACKs processed by the owner.

## M-R9-008 — OPEN MEDIUM: required built-binary/process regressions still incomplete

`8bd87e3` changes only `crates/neko-cli/src/main.rs`. Its new discriminating tests are unit tests over real loopback UDP sockets and an authenticated `SecureSession`, not built-binary/process tests through the real `failover` command.

Before R9-2 closure, retain the previous R9-2F requirement:

1. **Reversed logical DeliveryAck order through the built binary.** Add a test-only server seam that holds record 0's Session DeliveryAck until record 1 is accepted, then sends record 1 ACK before record 0. Prove both exact logical keys are applied once, no logical ACK becomes Carrier feedback, Carrier recovery settles, no conflict, and the baseline needs no PTO/retransmit unless deliberately injected.
2. **`--reliable-udp + migration-back` reservation.** Prove the reserved final logical record is neither reliable-tracked nor legacy wire-sent before post-promotion return authorization; only its later explicit owner may send it.

The ordinary no-loss multi-record process test remains useful but is not a substitute for these two discriminating process cases.

## R9-2H — coherent repair/closure package, pre-authorized now

Do not wait for another reviewer cycle. Implement H-R9-006 + H-R9-007 + M-R9-008 as one coherent package where practical:

1. one reliable receive/demux owner state with outstanding logical confirmations, recovery settlement, one persistent malformed budget, and typed counters;
2. one classification path for every successfully authenticated plaintext: exact Session DeliveryAck -> canonical Carrier ACK -> bounded invalid/unexpected;
3. one final completion predicate (`outstanding.is_empty() && in_flight()==0`) under the existing absolute bound;
4. final typed outcome counters only after completion/partial outcome;
5. focused interleaved-malformed/Carrier-ACK regression;
6. reversed logical-ACK built-binary/process regression;
7. reliable+migration-back reservation process regression;
8. exact pushed-tree local gate + provenance.

No wire tag, new Session ledger, new Carrier delivery semantics, D019 choice, retention policy, capacity/security number, or release decision is required.

# Continuous queue after R9-2H closes

Keep this entire queue available; do not shrink back to one hourly ticket.

## R9-3 — process Data-loss recovery

Add bounded deterministic post-admission suppression of one/periodic reliable-owned Data packet. Prove congestion admission precedes suppression; PTO fires only after deadline; retransmission uses fresh authenticated packet number/nonce with stable frame/Session identity; exactly-once Session delivery; Carrier ACK and Session DeliveryAck settle independently; final recovery drains to zero or yields explicit bounded partial/failure.

## R9-4 — ACK-loss + reorder/delayed-original

Cover packet ACK emitted then suppressed, retransmitted replacement before delayed original, and delayed original after replacement. Require one logical Session delivery, Session-owned duplicate suppression, no conflict, fresh packet numbers/nonces, truthful ACK-loss/PTO counters and final settlement.

## R9-5 — tamper/future ACK/malformed-feedback negatives

Prove unauthenticated/tampered Data creates no packet-ACK obligation and no Session receive; tampered ACK creates no recovery mutation; future/never-sent ACK is typed rejected atomically; stale/duplicate ACK fabricates no RTT/loss/Session evidence; malformed ACK/range/control handling is finite and panic-free.

## R9-6 — pacing/cwnd/plaintext-owner atomicity

Using bounded test-only limits, prove initial/retransmit sends consult congestion admission; refusal consumes no Session byte space and no committed packet/plaintext/recovery owner; retransmission plaintext owner is bounded/single-source-of-truth; pacing deadlines finite; teardown releases ownership.

## R9-7 — truthful process observability/result contract

Distinguish Data offered/admitted/wire-sent/suppressed; Carrier ACK emitted/wire-sent/suppressed/applied/rejected; Session DeliveryAck emitted/applied/duplicate/rejected; PTO due/fired; retransmit attempted/admitted/wire-sent/refused; resolved acked/lost/in-flight; Session first-delivery/duplicate/conflict/application bytes; malformed-budget use; cleanup/final outcome. Increment counters only after represented actions succeed.

## R9-8 — actual authenticated warm TCP standby

Reuse existing failover TCP connect/accept, canonical negotiation, fresh Noise trust/authz, resume/readiness tuple+generation+delivery-epoch binding and resource admission. No standby application Data before atomic promotion.

## R9-9 — resolved UDP health -> hysteresis -> real TCP promotion

Fresh resolved UDP outcomes drive Carrier health. Recoverable loss stays on UDP; PTO-only observation cannot erase later resolved loss; distinct bad resolved intervals cross committed hysteresis and promote only to an actually ready TCP standby; invalid/unready standby yields `FallbackFailed`; old historical loss is not replayed as new evidence.

## R9-10 — uncertain Session replay across UDP -> TCP

At promotion, replay at least one genuinely uncertain logical Session range over promoted TCP. Draining UDP accepts no new application Data. Receiver deduplicates by Session/stream/byte offset; application bytes exactly once. Do not add TCP packet ACK.

## R9-11 — timeout/shutdown/cleanup matrix

Cover setup/application/PTO/settlement deadlines, controlled stop, failed standby promotion, malformed/tampered input, listener/socket/process cleanup and same-address rebind where supported.

## R9-12 — coherent exact-tree gate + independent bounded review

Independent review challenges Session-above-Carrier layering, authenticated packet identity/ACK, bounded plaintext ownership, single receive-owner classification, deadline-driven PTO/pacing/cwnd, fresh health/hysteresis, real warm TCP readiness/promotion, uncertain Session replay/dedup, result truthfulness and cleanup/no-public-exposure. BLOCKER/HIGH -> smallest repair + regression + re-gate + continue; LOW/NOTE does not halt progression.

# Q10/Q11/Q12 after R9

- **Q10:** integrate only genuinely new R9 evidence into existing observability; no second logging framework.
- **Q11:** reconcile `docs/status.md`, `IMPLEMENTATION_PLAN.md`, `ROADMAP.md` and release packet only for earned status. Only a green independently reviewed cross-process R9 + real TCP promotion may create the changed specific `READY_LIVE` row.
- **Q12:** then execute exactly one minimal changed-hypothesis self-owned client<->VPS run under standing authorization, with temporary unprivileged listeners, exact commit/binary/parameters, authenticated recovery/Session/switch evidence and cleanup. Preserve negative evidence; no unchanged same-class retry.

# VPS opportunity

**Not READY yet.** R9 receive ownership is improved but still split across two loops and the malformed budget is not operation-wide; required process regressions are incomplete. This is an implementation/evidence dependency, not a permission blocker.

Once repaired R9-2H..R9-12 and Q11 establish a new specific live question, standing authorization already covers the bounded self-owned TCP/UDP run. Do not ask again for ordinary WAN authorization.

# Non-blocking policy/authority gates

Keep separate from the mechanically determined R9 chain:

- `SessionRuntime.events` retention (`POLICY_BLOCKED_RESOURCE_BOUND`);
- D019 source-retention/no-reset policy;
- RSEC-001 capacity/adversarial-load suitability;
- signing/key custody/SBOM/publication trust;
- previous frozen-release interoperability;
- final RC/freeze/release/production authority.

Do not invent policy values while working R9.
