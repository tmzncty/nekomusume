# ChatGPT reviewer handoff — R9-2H implementation stagnation; single-owner closure remains READY_LOCAL

## Current repository truth

- Default `main` before this refresh: exact `2fb3a0f67c44e439ac8ae8fdf54f7bd5d0055395`.
- Latest developer source/test SHA remains exact `8bd87e388745ae175f8b188f13936f1c1fe86ea4` (`fix(cli): R9-2E bounded authenticated receive/demux owner`). No newer developer source/test commit or open PR exists.
- Exact `42f0c9f` is provenance/docs only; exact `1a43e8a` and `2fb3a0f` are reviewer docs only.
- Hosted `stable checks` and `nightly decode fuzz smoke` on `2fb3a0f` are green. They are cross-evidence, not developer-local CI.
- No WAN/VPS run occurred. `READY_LIVE: none` remains authoritative.
- Governance unchanged: item 3 incomplete; item 4 incomplete; `RELEASE_CANDIDATE=false`; `PRODUCTION_READY=false`; `FREEZE=false`; `RELEASED=false`.

This is now **implementation stagnation**, not a missing-permission, CI, architecture, or policy blocker. The coding agent is pre-authorized to modify `crates/neko-cli/src/main.rs` and the corresponding CLI/process tests immediately. Do not wait for another reviewer cycle.

## Current source facts at the code-identical `8bd87e3` tree

The first receive helper still owns `let mut malformed = 0usize` locally and returns after each matched Session DeliveryAck or canonical Carrier ACK. The caller invokes it repeatedly while logical confirmations remain. Therefore a valid/rejected Carrier ACK can reset malformed accounting.

After logical confirmations are empty, `failover_client` still enters a second ad-hoc settlement receive loop. That loop independently calls `open_unreliable`, decodes only Carrier ACKs, silently ignores other authenticated plaintext, and does not contribute to the earlier final ACK counters. The `r9_udp_packet_ack_outcomes` diagnostic is emitted before settlement.

The settlement phase also creates a fresh `Instant::now() + Duration::from_secs(secs.min(10))` deadline rather than remaining under the original `application_deadline`. This is bounded but violates the intended one-owner / one-absolute-operation-bound evidence model.

## H-R9-006 — OPEN HIGH: malformed budget is not operation-wide

`MAX_POST_HANDSHAKE_MALFORMED` must be one persistent budget for the entire reliable authenticated receive/settlement operation.

Required discriminating sequence:

`malformed #1 -> malformed #2 -> canonical Carrier ACK -> malformed #3`

The third malformed input must still exhaust the bound. Applied **and rejected** Carrier ACKs must not reset it. A Session DeliveryAck must not reset it either.

Do not create a new numeric policy. Reuse the existing committed `MAX_POST_HANDSHAKE_MALFORMED` value; only fix its ownership/lifetime.

## H-R9-007 — OPEN HIGH: two receive owners still exist

Replace the helper-plus-settlement split with one stateful authenticated receive owner for reliable mode.

Recommended minimal implementation shape (API spelling may differ):

- one local state object owned by `failover_client`, containing at least:
  - outstanding logical records/keys;
  - persistent malformed count;
  - Carrier ACK applied count;
  - Carrier ACK rejected count;
  - logical confirmations applied count;
- one receive/classification loop that owns the UDP socket receive path after handshake;
- one classification order for every successfully authenticated plaintext:
  1. exact Session `DeliveryAck` for an outstanding key -> apply via `SessionRuntime::delivery_ack`, retire exactly that key;
  2. canonical Carrier packet ACK -> `ReliableUdpRuntime::apply_ack`, record typed applied/rejected outcome;
  3. anything else authenticated -> consume the same persistent malformed/ignored budget;
- unauthenticated/tampered ciphertext remains a bounded negative and creates no Session/Carrier evidence;
- completion only when `outstanding.is_empty() && rt.in_flight() == 0` (or, when reliable runtime is absent, the equivalent logical-only completion);
- no second settlement decoder loop after that owner returns.

Final `r9_udp_packet_ack_outcomes` must be emitted **after** the owner reaches completion or explicit partial failure, so it includes the ACKs that actually drain recovery.

No new wire tag, Session ledger, Carrier delivery semantics, crypto construction, or policy choice is required.

## M-R9-008 — closure tests must be built-binary/process tests

Before R9-2 closure, add all of these through the real `failover` command path:

1. **Reversed logical ACK order:** test-only server seam withholds record-0 Session DeliveryAck until record 1 is accepted, then sends ACK(1) before ACK(0). Require both exact logical ranges to confirm once, Carrier ACKs to stay Carrier-local, recovery to settle, and no conflict.
2. **Reliable + migration-back reservation:** the reserved final logical record must not be reliable-tracked and must not be legacy wire-sent before post-promotion return authorization. Its later explicit owner alone may send it.
3. **Persistent malformed budget across Carrier feedback:** authenticated malformed #1, malformed #2, canonical Carrier ACK, malformed #3 must hit the same operation-wide malformed ceiling.

Focused unit tests may supplement these but do not replace them.

## M-R9-009 — one absolute receive-operation deadline

Do not create a new settlement deadline after logical confirmation. The single receive owner must remain under the existing absolute `application_deadline` (or an equivalent single deadline established once before the owner starts).

A timeout before both completion conditions are true must produce an explicit bounded partial/failure result, including outstanding logical count and remaining recovery in-flight state where already exposed. Do not extend the operation by another `secs.min(10)` window.

## R9-2H acceptance package — execute continuously now

A coherent repair should, in one source/test progression where practical:

1. introduce the stateful single receive owner;
2. remove/reset-proof local malformed ownership from the old helper shape;
3. delete the second ad-hoc settlement decoder loop;
4. keep one original absolute deadline;
5. emit final typed Carrier ACK counters only after complete/partial settlement;
6. add the three discriminating built-binary/process regressions above;
7. keep existing ordinary no-loss/multi-record tests green;
8. run focused CLI tests, then `PYTHONDONTWRITEBYTECODE=1 bash scripts/check.sh`, `git diff --check`, and verify clean tree on the final pushed source/test SHA;
9. record developer-local provenance separately from GitHub-hosted checks;
10. continue directly into R9-3 without waiting for reviewer cadence if all gates are green.

Wire decoder/framing is unchanged by the intended refactor, so decode fuzz is not required solely for this repair. If wire/parser/crypto framing changes unexpectedly, use the pinned fuzz toolchain as required by repository policy.

# Continuous queue after R9-2H closes

Do not collapse this queue after one small repair.

## R9-3 — process Data-loss recovery

Add bounded deterministic post-admission suppression of one/periodic reliable-owned Data packet. Prove congestion admission precedes suppression; PTO fires only after deadline; retransmission uses a fresh authenticated packet number/nonce with stable Session/frame identity; exactly-once Session delivery; Carrier ACK and Session DeliveryAck settle independently; final recovery drains to zero or yields explicit bounded partial/failure.

## R9-4 — ACK-loss + reorder/delayed-original

Cover packet ACK emitted then suppressed, retransmitted replacement before delayed original, and delayed original after replacement. Require one logical Session delivery, Session-owned duplicate suppression, no conflict, fresh packet numbers/nonces, truthful ACK-loss/PTO counters and final settlement.

## R9-5 — tamper/future ACK/malformed-feedback negatives

Prove tampered Data creates no packet-ACK obligation/Session receive; tampered ACK creates no recovery mutation; future/never-sent ACK is typed rejected atomically; stale/duplicate ACK fabricates no RTT/loss/Session evidence; malformed feedback is finite and panic-free.

## R9-6 — pacing/cwnd/plaintext-owner atomicity

With bounded test-only limits, prove initial/retransmit sends consult congestion admission; refusal consumes no Session byte space or committed packet/plaintext/recovery owner; retransmit plaintext has one bounded owner; pacing deadlines are finite; teardown releases ownership.

## R9-7 — truthful process observability/result contract

Keep Data offered/admitted/wire-sent/suppressed; Carrier ACK emitted/wire-sent/suppressed/applied/rejected; Session DeliveryAck emitted/applied/duplicate/rejected; PTO due/fired; retransmit attempted/admitted/wire-sent/refused; resolved acked/lost/in-flight; Session first-delivery/duplicate/conflict/application bytes; malformed-budget use; cleanup and final outcome distinct. Counters increment only after represented actions succeed.

## R9-8 — actual authenticated warm TCP standby

Reuse existing TCP connect/accept, canonical negotiation, fresh Noise trust/authz, resume/readiness tuple+generation+delivery-epoch binding and resource admission. No standby application Data before atomic promotion.

## R9-9 — resolved UDP health -> hysteresis -> real TCP promotion

Fresh resolved UDP outcomes drive Carrier health. Recoverable loss stays on UDP; PTO-only observation cannot erase later resolved loss; distinct bad resolved intervals cross committed hysteresis and promote only to an actually ready TCP standby; invalid/unready standby yields typed failure.

## R9-10 — uncertain Session replay across UDP -> TCP

At promotion, replay at least one genuinely uncertain logical Session range over promoted TCP. Draining UDP accepts no new application Data. Receiver deduplicates by Session/stream/byte offset; application bytes are exactly once. Do not add TCP packet ACK.

## R9-11 — timeout/shutdown/cleanup matrix

Cover setup/application/PTO/single-owner receive deadline, controlled stop, failed standby promotion, malformed/tampered input, listener/socket/process cleanup and same-address rebind where supported.

## R9-12 — coherent exact-tree gate + independent bounded review

Independent challenge of Session-above-Carrier layering, authenticated packet identity/ACK, bounded plaintext ownership, **single receive-owner classification + operation-wide malformed/deadline ownership**, deadline-driven PTO/pacing/cwnd, fresh health/hysteresis, real warm TCP readiness/promotion, uncertain Session replay/dedup, result truthfulness and cleanup/no-public-exposure. BLOCKER/HIGH -> smallest repair + regression + re-gate + continue; LOW/NOTE does not halt progression.

# Q10/Q11/Q12 after R9

- **Q10:** integrate only genuinely new R9 evidence into existing observability; no second logging framework.
- **Q11:** reconcile `docs/status.md`, `IMPLEMENTATION_PLAN.md`, `ROADMAP.md` and release packet only for earned status. Only a green independently reviewed cross-process R9 + real TCP promotion may create a new specific `READY_LIVE` row.
- **Q12:** then execute exactly one minimal changed-hypothesis self-owned client<->VPS run under standing authorization, with temporary unprivileged listeners, exact commit/binary/parameters, authenticated recovery/Session/switch evidence and cleanup. Preserve negative evidence; no unchanged same-class retry.

# VPS opportunity

**Not READY yet.** The current blocker is local implementation/evidence ownership, not authorization. Once R9-2H..R9-12 and Q11 establish a specific new live question, standing authorization already covers the bounded self-owned TCP/UDP run.

# Non-blocking policy/authority gates

Keep separate from this mechanically determined R9 chain:

- `SessionRuntime.events` retention (`POLICY_BLOCKED_RESOURCE_BOUND`);
- D019 source-retention/no-reset policy;
- RSEC-001 capacity/adversarial-load suitability;
- signing/key custody/SBOM/publication trust;
- previous frozen-release interoperability;
- final RC/freeze/release/production authority.

Do not invent policy values while working R9.
