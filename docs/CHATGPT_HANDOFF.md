# ChatGPT reviewer handoff — `fb09cf5` ownership repair accepted in part; authenticated multi-record demux still blocks R9-3

## Reviewed repository truth

- Latest developer source/test SHA reviewed: exact `fb09cf59e81782a5b2ecf12466bf47f72a9e51d6` (`fix(cli): R9-2 multi-record demux + reserved-record ownership`).
- Exact `f74f1e6d711258bb2cba0853feed4359d799bee6` adds developer-local provenance only; no source/test change after `fb09cf5` in that developer sequence.
- Independent reviewer recheck is persisted at `docs/reviews/independent-r9-repair-recheck-fb09cf5-20260914.md` (reviewer commit `8850484f7b917cedc2c52e2826f9abf0144a4b71`).
- `docs/local-gate-fb09cf5-20260914.md` records a clean detached developer-local exact-tree gate: `PYTHONDONTWRITEBYTECODE=1 bash scripts/check.sh` exit 0, `git diff --check` exit 0, clean initial/final tree, Linux x86_64, rustc 1.98.0.
- Current provenance descendant `f74f1e6` has green GitHub-hosted `stable checks` and `nightly decode fuzz smoke`; hosted checks are extra cross-evidence only.
- Open PRs: none. No WAN/VPS execution occurred in this sequence. `READY_LIVE: none` remains authoritative.
- Governance unchanged: item 3 incomplete; item 4 incomplete; `RELEASE_CANDIDATE=false`; `PRODUCTION_READY=false`; `FREEZE=false`; `RELEASED=false`.

## Reviewer verdict

`fb09cf5` is useful forward progress and must not be reverted, but **R9-2 is not yet closed and R9-3 remains blocked**.

### H-R9-005 — implementation repair accepted; discriminating regression still required

The reviewed exclusive-bound implementation is correct:

- reliable mode starts legacy uncertain ownership at index 2;
- `uncertain_count = uncertain_end.saturating_sub(uncertain_start)`;
- tracking consumes only `[uncertain_start, uncertain_end)`;
- direct uncertain send is additionally guarded by `uncertain_start < uncertain_end`.

So when recovery/migration-back reserves the final record (`uncertain_end == 2` in a three-record reliable run), index 2 is no longer assigned or wire-sent by the pre-promotion legacy path.

The current test delta, however, only extends the ordinary no-recovery reliable-UDP process test. It does not execute reliable UDP together with migration-back/recovery. Keep the implementation; add the requested combined discriminating regression as part of M-R9-004 closure.

### H-R9-001 — OPEN HIGH: serial expected-record receive is still not a bounded demultiplexer

`recv_udp_delivery_ack` still accepts exactly one `expected: &OutboundRecord`. `fb09cf5` invokes it serially for record 0 and then record 1.

With `rt` present, an authenticated plaintext that is not the currently expected Session `DeliveryAck` enters the Carrier-ACK branch; if it is not a canonical Carrier ACK, the function still unconditionally `continue`s. Therefore a legitimate record-1 Session `DeliveryAck` arriving before record 0 can be consumed/discarded by the first helper invocation, causing the later record-1 wait to time out.

The same branch also still:

- bypasses `MAX_POST_HANDSHAKE_MALFORMED` for authenticated non-ACK/control/malformed plaintext while reliable mode is active;
- discards `rt.apply_ack(...)` errors with `let _ = ...`, so rejected/future/stale Carrier feedback is not typed/observable at this boundary.

This is a local mechanically determined correctness/evidence finding. No maintainer policy choice is needed.

### M-R9-004 — OPEN/PARTIAL

Current durable process evidence proves offset 16 is reliable-owned, emits a second `r9_udp_delivery_ack_validated` happy-path event, avoids the old offset-16 legacy uncertain direct send, and reaches authoritative `remaining_in_flight:0` in the no-loss case.

It still must prove:

- order-independent demux of two outstanding reliable-owned Session DeliveryAcks;
- exactly-once application of both logical confirmations to `SessionRuntime`;
- typed Carrier ACK applied/rejected outcomes rather than silent discard;
- bounded handling of authenticated unexpected/control/malformed plaintext in reliable mode;
- reliable-UDP + migration-back reserved final record is not tracked/wire-sent before post-promotion authorization;
- clean baseline explicitly has zero PTO/retransmit and zero Session conflict/duplicate (or an explicitly justified expected duplicate count).

# Blocking repair contract — R9-2E/F

## R9-2E — one bounded authenticated receive/demux owner

Replace the serial single-`expected` receive shape for reliable mode with one bounded demux loop over the bounded set/map of outstanding reliable-owned logical records.

Every successfully authenticated plaintext is classified exactly once:

1. **Carrier packet ACK** — canonical `RecordType::Ack` -> `decode_ack` -> bounded `AckRanges` -> `apply_ack`; emit applied/rejected typed outcome. Rejection must not mutate recovery.
2. **Session DeliveryAck** — decode exact Session id + `(stream, offset, len)`, match against outstanding reliable-owned logical records, apply `SessionRuntime::delivery_ack` exactly once, retire that logical expectation; duplicate/stale/unexpected logical ACK is typed/bounded.
3. **Unexpected authenticated control / malformed plaintext** — consume the existing finite malformed/ignored budget and emit typed diagnostic. Reliable mode must not silently spin forever.

Carrier packet ACK and Session delivery confirmation remain separate evidence domains. Do not change wire grammar, Session/Carrier layering, crypto architecture or policy values.

The receive owner may complete when all required logical confirmations have been applied and Carrier in-flight state has settled, or return an explicit bounded timeout/partial failure.

## R9-2F — discriminating process regressions

Add at least these two bounded process tests:

1. **Reversed logical-ACK order:** arrange record-1 Session DeliveryAck before record-0 confirmation and prove both are retained/classified/applied exactly once; old serial behavior must fail this regression.
2. **Reliable + migration-back reservation:** run the existing recovery/migration-back seam with `--reliable-udp` and prove the final reserved logical offset is neither tracked nor wire-sent before the authorized post-promotion return-to-UDP milestone.

Also pin the clean no-loss baseline: two reliable logical first-deliveries, two independent Session DeliveryAck applications, Carrier owners retired to `in_flight()==0`, zero PTO/retransmit, bounded malformed budget, zero conflict and cleanup.

## R9-2G — repaired exact-tree gate

After one coherent pushed source/test SHA closes H-R9-001 and M-R9-004:

```bash
PYTHONDONTWRITEBYTECODE=1 bash scripts/check.sh
git diff --check
```

Record exact pushed SHA, UTC start/end, OS/arch, rustc, exit codes and clean initial/final tree. Hosted CI remains separate cross-evidence. No decode fuzz is required unless external decoder/framing grammar is changed.

# Pre-authorized continuous queue after R9-2 closes

Do **not** stop after the demux repair. Continue directly through all dependency-safe work below.

## R9-3 — process Data-loss recovery

Add bounded deterministic post-admission suppression of one/periodic reliable-owned Data packet. Prove congestion admission precedes suppression; PTO fires only after deadline; retransmission uses fresh authenticated packet number/nonce with stable frame/Session identity; exactly-once Session delivery; Carrier ACK and Session DeliveryAck settle independently; final recovery drains to zero or yields explicit bounded partial/failure.

## R9-4 — ACK-loss + reorder/delayed-original

Cover packet ACK emitted then suppressed, retransmitted replacement before delayed original, and delayed original after replacement. Require one logical Session delivery, Session-owned duplicate suppression, no conflict, fresh packet numbers/nonces, truthful ACK-loss/PTO counters and final settlement.

## R9-5 — tamper/future ACK/malformed-feedback negatives

Prove unauthenticated/tampered Data creates no packet-ACK obligation and no Session receive; tampered ACK creates no recovery mutation; future/never-sent ACK is typed rejected atomically; stale/duplicate ACK fabricates no RTT/loss/Session evidence; malformed ACK/range/control handling is finite and panic-free.

## R9-6 — pacing/cwnd/plaintext-owner atomicity

Using bounded test-only limits, prove initial/retransmit sends consult congestion admission; refusal consumes no Session byte space and no committed packet/plaintext/recovery owner; retransmission plaintext owner is bounded/single-source-of-truth; pacing deadlines finite; teardown releases ownership.

## R9-7 — truthful process observability/result contract

Distinguish Data offered/admitted/wire-sent/suppressed; Carrier ACK emitted/wire-sent/suppressed/applied/rejected; Session DeliveryAck emitted/applied/duplicate/rejected; PTO due/fired; retransmit attempted/admitted/wire-sent/refused; resolved acked/lost/in-flight; Session first-delivery/duplicate/conflict/application bytes; cleanup/final outcome. Increment counters only after represented actions succeed.

## R9-8 — actual authenticated warm TCP standby

Reuse existing failover TCP connect/accept, canonical negotiation, fresh Noise trust/authz, resume/readiness tuple+generation+delivery-epoch binding and resource admission. No standby application Data before atomic promotion.

## R9-9 — resolved UDP health -> hysteresis -> real TCP promotion

Fresh resolved UDP outcomes drive Carrier health. Recoverable loss stays on UDP; PTO-only observation cannot erase later resolved loss; distinct bad resolved intervals cross committed hysteresis and promote only to an actually ready TCP standby; invalid/unready standby yields `FallbackFailed`; old historical loss is not replayed as new evidence.

## R9-10 — uncertain Session replay across UDP -> TCP

At promotion, replay at least one genuinely uncertain logical Session range over promoted TCP. Draining UDP accepts no new application Data. Receiver deduplicates by Session/stream/byte offset; application bytes exactly once. Do not add TCP packet ACK.

## R9-11 — timeout/shutdown/cleanup matrix

Cover setup/application/PTO/settlement deadlines, controlled stop, failed standby promotion, malformed/tampered input, listener/socket/process cleanup and same-address rebind where supported.

## R9-12 — coherent exact-tree gate + independent bounded review

Independent review challenges Session-above-Carrier layering, authenticated packet identity/ACK, bounded plaintext ownership, deadline-driven PTO/pacing/cwnd, fresh health/hysteresis, real warm TCP readiness/promotion, uncertain Session replay/dedup, result truthfulness and cleanup/no-public-exposure. BLOCKER/HIGH -> smallest repair + regression + re-gate + continue; LOW/NOTE does not halt progression.

# Q10/Q11/Q12 after R9

- **Q10:** integrate only genuinely new R9 evidence into existing observability; no second logging framework.
- **Q11:** reconcile `docs/status.md`, `IMPLEMENTATION_PLAN.md`, `ROADMAP.md` and release packet only for earned status. Only a green independently reviewed cross-process R9 + real TCP promotion may create the changed specific `READY_LIVE` row.
- **Q12:** then execute exactly one minimal changed-hypothesis self-owned client<->VPS run under standing authorization, with temporary unprivileged listeners, exact commit/binary/parameters, authenticated recovery/Session/switch evidence and cleanup. Preserve negative evidence; no unchanged same-class retry.

# VPS opportunity

**Not READY yet.** The implementation is materially new, but H-R9-001 means current multi-record receive evidence remains order-sensitive and not trustworthy enough for WAN promotion. This is an implementation dependency, not a permission blocker.

Once repaired R9-2..R9-12 and Q11 establish a new specific live question, standing authorization already covers the bounded self-owned TCP/UDP run. Do not ask again for ordinary WAN authorization.

# Non-blocking policy/authority gates

Keep these separate from the mechanically determined R9 chain:

- `SessionRuntime.events` retention (`POLICY_BLOCKED_RESOURCE_BOUND`);
- D019 source-retention/no-reset policy;
- RSEC-001 capacity/adversarial-load suitability;
- signing/key custody/SBOM/publication trust;
- previous frozen-release interoperability;
- final RC/freeze/release/production authority.

Do not invent policy values while working R9.
