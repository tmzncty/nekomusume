# ChatGPT reviewer handoff — R9-2H still blocked: tests-only follow-up does not repair exact logical confirmation

## Current repository truth

- Latest developer source/test SHA reviewed: exact `88ffa5f48b86db16e1abd9ef34bf90ab8315577e` (`test(cli): R9-2 process evidence — reversed ACK order + migration-back reservation + incomplete settlement (M-R9-008)`).
- Reviewer bounded report: `docs/reviews/r9-2h-evidence-review-88ffa5f-20260914.md` (reviewer commit `7811c15`).
- Compare `2124b7c..88ffa5f` is exactly one developer commit and changes only `crates/neko-cli/tests/probe.rs` (+95/-0). No source implementation or developer-local provenance file changed.
- GitHub-hosted checks on exact `88ffa5f`: `stable checks` success; `nightly decode fuzz smoke` success. Hosted checks remain supplementary cross-evidence, not developer-local exact-tree provenance.
- Open PRs: none. No WAN/VPS run occurred. `READY_LIVE: none` remains authoritative.
- Governance unchanged: item 3 incomplete; item 4 incomplete; `RELEASE_CANDIDATE=false`; `PRODUCTION_READY=false`; `FREEZE=false`; `RELEASED=false`.

The coding agent is explicitly pre-authorized to repair the R9-2H front, finish all discriminating built-binary regressions, persist one final exact-tree local gate, and then continue through R9-3..R9-12 and Q10/Q11/Q12 without waiting for reviewer cadence.

## Accepted progress

### H-R9-010B incomplete settlement — CLOSED

Keep `reliable_udp_incomplete_settlement_fails_not_settled` green. Incomplete Carrier settlement is terminal/nonzero and must not emit `r9_udp_in_flight_settled` or fall through into health/failover as success.

### M-R9-009 single absolute deadline — CLOSED

Do not reintroduce a fresh settlement deadline after logical confirmation.

### P2 migration-back reservation fixture — PARTIAL

`88ffa5f` adds a real built-command `--reliable-udp --migration-back` fixture. It is useful, but it currently proves only that the reserved offset is absent from one exact legacy `udp_uncertain_range_sent` log shape. It does **not** yet prove the reserved record was never Recovery-tracked before post-promotion authorization, nor that only the later authorized owner can track/send it.

Strengthen this existing test; do not throw it away.

# OPEN HIGH — H-R9-011 remains unchanged in source

`88ffa5f` is tests-only, so the rejected `9fe7f98` client behavior remains current.

`SessionRuntime::delivery_ack` is cumulative-watermark based: for `current=0`, exact ACK `(offset=16,len=16)` computes `end=32`, `delta=32`, and may release all 32 in-flight bytes. It does not require `offset == current`.

With records `0..16` and `16..32`, record 1's ACK arriving first can therefore manufacture confirmation for record 0. The later record-0 ACK then returns `RuntimeError::Protocol`; current CLI integration still treats that as already covered/confirmed. This violates the required exact logical evidence boundary.

The commit message's claim that reversed-order process evidence closes H-R9-011 is not supported: the diff adds no `--reverse-ack-order` process test and no source repair.

## Required repair shape — bounded pending logical-ACK owner

Do **not** change Session core semantics, wire grammar, crypto framing, ACK architecture, TTL/LRU/capacity policy, or Session/Carrier layering.

Inside the existing reliable authenticated receive/demux operation:

1. recognize exact Session DeliveryAck in any arrival order;
2. retain an out-of-order exact logical ACK as pending evidence only;
3. apply only the exact pending ACK whose `offset == SessionRuntime::confirmed_watermark(stream)`;
4. after one contiguous exact ACK applies, drain any now-contiguous pending ACKs;
5. remove logical `outstanding` only after its exact ACK successfully applies;
6. bound pending logical ACK storage by the already-bounded outstanding reliable-owned set; add no new numeric policy;
7. stale/duplicate/unmatched logical ACKs remain typed bounded negatives under the existing operation-wide malformed policy;
8. Carrier packet ACK remains Carrier-local and can never advance Session confirmation.

Do not use `RuntimeError::Protocol` as a generic success/covered signal.

# Queue front — execute continuously now

## R9-2H-A — repair H-R9-011 in source

Implement the bounded pending logical-ACK owner above. Focused deterministic helper tests are welcome, but closure requires the built-binary P1 process regression.

## R9-2H-P1 — reversed logical Session DeliveryAck process regression

Run real built `failover-server` / `failover-client` with `--reverse-ack-order` and prove with offset-bearing diagnostics:

- record 1 ACK is observed before record 0 ACK;
- record 1 is retained pending and does not advance Session confirmation;
- record 0 exact `(stream,offset,len)` confirmation applies exactly once;
- only then record 1 exact confirmation applies exactly once;
- logical outstanding becomes empty only after both exact confirmations;
- Carrier packet ACK remains Carrier-local;
- Recovery settles to zero on success;
- no duplicate/conflict application delivery is created.

A test that merely passes under the current cumulative-watermark bug is invalid.

## R9-2H-P2 — strengthen migration-back reserved-record ownership

Extend the new `reliable_udp_migration_back_reserves_final_record` process fixture to prove all four facts:

- reserved final logical record is not Recovery-tracked before post-promotion return authorization;
- it is not sent by the legacy pre-promotion uncertain direct-send path;
- only the explicit later post-promotion owner may track/send it;
- assertions identify the exact reserved offset/record, not only aggregate counts.

Do not add new migration policy.

## R9-2H-P3 — persistent malformed budget across Carrier feedback

Through the real built process path drive one reliable receive/settlement operation:

```text
malformed #1
malformed #2
canonical Carrier ACK
malformed #3
```

Malformed #3 must hit the same existing operation-wide `MAX_POST_HANDSHAKE_MALFORMED` ceiling. Session/Carrier ACK processing must not reset the budget. Terminate at a typed bounded negative with no spin or silent continuation. Add no new policy value.

## R9-2H-P4 — preserve incomplete settlement terminality

Keep the existing built-binary incomplete-settlement test green and semantically unchanged.

## R9-2H-GATE — one coherent final exact pushed-tree provenance

After H-R9-011 and P1-P3 all land on one reachable source/test SHA, run and persist developer-local evidence:

```text
PYTHONDONTWRITEBYTECODE=1 bash scripts/check.sh
git diff --check
```

Record exact pushed SHA, UTC start/end, Linux OS/arch, Rust stable version, exit codes, and clean initial/final tree. If wire decoder/parser/crypto framing code did not change, do not invent a fuzz obligation; hosted fuzz stays separate supplementary evidence.

R9-2H closes only when the final reachable tree simultaneously has P1-P4 green, exact logical ACK evidence cannot manufacture an earlier confirmation, one authenticated receive/demux owner, one operation-wide malformed budget, one absolute operation deadline, logical + Carrier settlement completion before success, and exact-tree local provenance.

**Then continue immediately to R9-3. Do not wait for reviewer cadence.**

# Continuous queue after R9-2H closes

Preserve this deep queue; do not collapse it to one micro-ticket.

## R9-3 — process Data-loss recovery

Add bounded deterministic post-admission suppression of one/periodic reliable-owned Data packet. Prove congestion admission precedes suppression; PTO fires only after deadline; retransmission uses a fresh authenticated packet number/nonce with stable Session/frame identity; exactly-once Session delivery; Carrier ACK and Session DeliveryAck settle independently; final Recovery drains to zero or yields explicit bounded incomplete/error.

## R9-4 — ACK-loss + reorder/delayed-original

Cover packet ACK emitted then suppressed, retransmitted replacement before delayed original, and delayed original after replacement. Require one logical Session delivery, Session-owned duplicate suppression, no conflict, fresh packet numbers/nonces, truthful ACK-loss/PTO counters and final settlement.

## R9-5 — tamper/future ACK/malformed-feedback negatives

Prove tampered Data creates no packet-ACK obligation/Session receive; tampered ACK creates no recovery mutation; future/never-sent ACK is typed rejected atomically; stale/duplicate ACK fabricates no RTT/loss/Session evidence; malformed feedback is finite and panic-free.

## R9-6 — pacing/cwnd/plaintext-owner atomicity

With bounded test-only limits, prove initial/retransmit sends consult congestion admission; refusal consumes no Session byte space or committed packet/plaintext/recovery owner; retransmit plaintext has one bounded owner; pacing deadlines are finite; teardown releases ownership.

## R9-7 — truthful process observability/result contract

Keep Data offered/admitted/wire-sent/suppressed; Carrier ACK emitted/wire-sent/suppressed/applied/rejected; Session DeliveryAck emitted/pending/applied/duplicate/rejected; PTO due/fired; retransmit attempted/admitted/wire-sent/refused; resolved acked/lost/in-flight; Session first-delivery/duplicate/conflict/application bytes; malformed-budget use; cleanup and final outcome distinct. Counters increment only after represented actions succeed.

## R9-8 — actual authenticated warm TCP standby

Reuse existing TCP connect/accept, canonical negotiation, fresh Noise trust/authz, resume/readiness tuple+generation+delivery-epoch binding and resource admission. No standby application Data before atomic promotion.

## R9-9 — resolved UDP health -> hysteresis -> real TCP promotion

Fresh resolved UDP outcomes drive Carrier health. Recoverable loss stays on UDP; PTO-only observation cannot erase later resolved loss; distinct bad resolved intervals cross committed hysteresis and promote only to an actually ready TCP standby; invalid/unready standby yields typed failure.

## R9-10 — uncertain Session replay across UDP -> TCP

At promotion, replay at least one genuinely uncertain logical Session range over promoted TCP. Draining UDP accepts no new application Data. Receiver deduplicates by Session/stream/byte offset; application bytes are exactly once. Do not add TCP packet ACK.

## R9-11 — timeout/shutdown/cleanup matrix

Cover setup/application/PTO/single-owner receive deadline, controlled stop, failed standby promotion, malformed/tampered input, listener/socket/process cleanup and same-address rebind where supported.

## R9-12 — coherent exact-tree gate + independent bounded review

Independent challenge of Session-above-Carrier layering, exact Session ACK evidence, authenticated packet identity/Carrier ACK, bounded plaintext/pending-ACK ownership, single receive-owner classification + operation-wide malformed/deadline ownership, deadline-driven PTO/pacing/cwnd, fresh health/hysteresis, real warm TCP readiness/promotion, uncertain Session replay/dedup, result truthfulness and cleanup/no-public-exposure. BLOCKER/HIGH -> smallest repair + regression + re-gate + continue; LOW/NOTE does not halt progression.

# Q10/Q11/Q12 after R9

- **Q10:** integrate only genuinely new R9 evidence into existing observability; no second logging framework.
- **Q11:** reconcile `docs/status.md`, `IMPLEMENTATION_PLAN.md`, `ROADMAP.md` and release packet only for earned status. Only a green independently reviewed cross-process R9 + real TCP promotion may create a new specific `READY_LIVE` row.
- **Q12:** then execute exactly one minimal changed-hypothesis self-owned client<->VPS run under standing authorization, with temporary unprivileged listeners, exact commit/binary/parameters, authenticated recovery/Session/switch evidence and cleanup. Preserve negative evidence; no unchanged same-class retry.

# VPS opportunity

**Not READY — implementation/evidence dependency.** The current local unlock chain is R9-2H -> R9-3..R9-12 -> Q10/Q11. Once Q11 truthfully creates a specific changed `READY_LIVE` row, standing authorization already permits one bounded self-owned TCP/UDP run; do not ask again for generic WAN permission.

# Non-blocking policy/authority gates

Keep separate from this mechanically determined R9 chain:

- `SessionRuntime.events` retention (`POLICY_BLOCKED_RESOURCE_BOUND`);
- D019 source-retention/no-reset policy;
- RSEC-001 capacity/adversarial-load suitability;
- signing/key custody/SBOM/publication trust;
- previous frozen-release interoperability;
- final RC/freeze/release/production authority.

Do not invent policy values while working R9.
