# ChatGPT reviewer handoff — R9-2H partial repair landed red; R9-3 remains blocked

## Current repository truth

- Latest developer source/test SHA reviewed: exact `1786f2f217b60923d97f8722ccdf48708c908750` (`fix(cli): R9-2 single-owner demux malformed budget + settlement`).
- Reviewer finding report: `docs/reviews/r9-2h-review-1786f2f-20260914.md` (reviewer commit `d91691d`).
- `1786f2f` changed only `crates/neko-cli/src/main.rs`; no process-test file or developer-local provenance was added.
- GitHub-hosted `nightly decode fuzz smoke` on `1786f2f` is green, but hosted `stable checks` is **red**: `bash scripts/check.sh` fails compiling the `neko-cli` test target with four `E0061` callsite errors after the helper signature gained `malformed: &mut usize`.
- No WAN/VPS run occurred. `READY_LIVE: none` remains authoritative.
- Governance unchanged: item 3 incomplete; item 4 incomplete; `RELEASE_CANDIDATE=false`; `PRODUCTION_READY=false`; `FREEZE=false`; `RELEASED=false`.

R9-3 is **not READY**. The coding agent is pre-authorized to repair R9-2H immediately and must continue through the deep queue once the closure gate is genuinely green; do not wait for another reviewer cycle.

## Accepted partial progress in `1786f2f`

Two useful implementation directions landed:

1. `recv_udp_delivery_ack` now takes caller-owned `&mut usize`, so a caller can preserve the malformed/ignored count across Session and Carrier demux returns.
2. The old ad-hoc settlement decoder (`recv_udp_until` + independent `open_unreliable` + ACK parser) was removed. Carrier settlement now reuses the same authenticated classification helper and records applied/rejected ACK outcomes; final ACK counters moved after the drain attempt.

These are partial repairs for H-R9-006/H-R9-007, not closure.

# Queue front — execute continuously now

## BLOCKER R9-2H-GATE-002 — restore a green exact source/test tree

Hosted `stable checks` on exact `1786f2f` reports four stale direct test callsites of `recv_udp_delivery_ack` (hosted compiler lines 4962, 5009, 5059, 5121): the new tenth `&mut usize` argument is missing.

Repair every callsite coherently; do not patch only enough to compile one test. Focused CLI tests must compile/run before the full gate.

After the final source/test repair is pushed, run and persist developer-local exact-tree provenance for:

- focused `neko-cli` tests relevant to the receive/demux owner;
- `PYTHONDONTWRITEBYTECODE=1 bash scripts/check.sh`;
- `git diff --check`;
- initial/final clean tree;
- exact pushed SHA, UTC start/end, OS/arch, Rust stable version, exit codes.

Hosted checks remain cross-evidence only.

## HIGH H-R9-010 — incomplete Carrier settlement still masquerades as settled

Current source still performs:

1. a `while !outstanding.is_empty()` logical-confirmation phase;
2. then a separate caller settlement phase while `rt.in_flight() > 0`.

In the settlement phase, any classifier timeout / malformed-budget exhaustion / receive error reaches `Err(_) => break`. Execution then emits both `r9_udp_packet_ack_outcomes` and **`r9_udp_in_flight_settled` regardless of whether `rt.in_flight()` is still nonzero**, then continues into health/failover logic.

Required invariant:

- successful completion exists only when logical outstanding is empty **and** authoritative recovery `in_flight()==0`;
- a timeout/error/malformed-bound before both are true is an explicit bounded partial/failure result;
- never emit a successful-looking `r9_udp_in_flight_settled` marker with remaining in-flight state;
- incomplete settlement must stop/fail this operation before downstream R9 behavior consumes a false settlement premise.

Minimal acceptable implementation shape: one stateful receive-operation result/owner that retains outstanding logical keys, persistent malformed count, Carrier applied/rejected counts and logical confirmation count, and returns a typed complete/partial/error result.

## MEDIUM M-R9-009 — use one absolute receive-operation deadline

`1786f2f` still creates a fresh `settle_deadline = Instant::now() + secs.min(10)` after the original `application_deadline` logical phase.

Do not extend the receive operation after logical confirmation. Establish one absolute deadline once before the owner begins and use it until both logical outstanding and recovery in-flight are complete or the operation explicitly fails partial.

## MEDIUM M-R9-008 — add the required built-binary/process closure regressions

The `1786f2f` diff changed only `src/main.rs`; required process-path closure evidence is still absent.

Before R9-2H closes, add all three through the real built `failover` command path:

1. **Reversed logical ACK order:** test-only server seam accepts two reliable-owned logical records and emits Session DeliveryAck for record 1 before record 0. Require both exact logical ranges to confirm exactly once, Carrier ACKs to remain Carrier-local, recovery to settle and no conflict.
2. **Reliable + migration-back reservation:** reserved final logical record is neither reliable-tracked nor legacy wire-sent before post-promotion return authorization; only its later explicit owner may send it.
3. **Persistent malformed budget across Carrier feedback:** authenticated malformed #1 -> malformed #2 -> canonical Carrier ACK -> malformed #3 must hit the same operation-wide `MAX_POST_HANDSHAKE_MALFORMED` ceiling. Applied or rejected Carrier ACK and Session DeliveryAck must not reset that counter.

Focused helper/unit tests may supplement these but do not replace them.

## R9-2H closure gate

R9-2H closes only when all of the following are simultaneously true on a reachable pushed source/test SHA:

- test target compiles and focused regressions pass;
- one persistent malformed budget spans the full reliable receive/settlement operation;
- every authenticated plaintext is classified exactly once as Session DeliveryAck / Carrier packet ACK / bounded negative;
- completion requires `outstanding.is_empty() && in_flight()==0`;
- partial/timeout/malformed/error cannot be labeled settled or silently continued;
- one absolute application/receive deadline is used, without a fresh post-confirmation extension;
- the three built-binary discriminating regressions pass;
- clean developer-local exact-tree gate + provenance exists.

If the coding agent can finish these in one coherent repair, it should immediately continue into R9-3 without waiting for reviewer cadence.

# Continuous queue after R9-2H closes

Preserve this queue; do not collapse it back to one micro-ticket.

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

Independent challenge of Session-above-Carrier layering, authenticated packet identity/ACK, bounded plaintext ownership, single receive-owner classification + operation-wide malformed/deadline ownership, deadline-driven PTO/pacing/cwnd, fresh health/hysteresis, real warm TCP readiness/promotion, uncertain Session replay/dedup, result truthfulness and cleanup/no-public-exposure. BLOCKER/HIGH -> smallest repair + regression + re-gate + continue; LOW/NOTE does not halt progression.

# Q10/Q11/Q12 after R9

- **Q10:** integrate only genuinely new R9 evidence into existing observability; no second logging framework.
- **Q11:** reconcile `docs/status.md`, `IMPLEMENTATION_PLAN.md`, `ROADMAP.md` and release packet only for earned status. Only a green independently reviewed cross-process R9 + real TCP promotion may create a new specific `READY_LIVE` row.
- **Q12:** then execute exactly one minimal changed-hypothesis self-owned client<->VPS run under standing authorization, with temporary unprivileged listeners, exact commit/binary/parameters, authenticated recovery/Session/switch evidence and cleanup. Preserve negative evidence; no unchanged same-class retry.

# VPS opportunity

**Not READY yet.** Current blockers are local R9-2H correctness/test/evidence ownership, not WAN authorization. Once R9-2H..R9-12 and Q11 establish a specific new live question, standing authorization already covers the bounded self-owned TCP/UDP run.

# Non-blocking policy/authority gates

Keep separate from this mechanically determined R9 chain:

- `SessionRuntime.events` retention (`POLICY_BLOCKED_RESOURCE_BOUND`);
- D019 source-retention/no-reset policy;
- RSEC-001 capacity/adversarial-load suitability;
- signing/key custody/SBOM/publication trust;
- previous frozen-release interoperability;
- final RC/freeze/release/production authority.

Do not invent policy values while working R9.
