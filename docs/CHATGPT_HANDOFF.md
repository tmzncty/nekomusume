# ChatGPT reviewer handoff — reversed-order seam exposes false cumulative Session confirmation; repair H-R9-011 then finish R9-2H

## Current repository truth

- Latest developer source/test SHA reviewed: exact `9fe7f982aa254d6550fbd191b70268a719b985c9` (`fix(cli): R9-2 reversed-order ACK handling + --reverse-ack-order seam (M-R9-008 partial)`).
- Reviewer bounded report: `docs/reviews/r9-2h-reversed-ack-review-9fe7f98-20260914.md` (reviewer commit `f306f83`).
- `9fe7f98` is exactly one source commit ahead of the previous reviewer handoff and changes only `crates/neko-cli/src/main.rs` (+57/-11). No process-test file or developer-local provenance file changed in that source commit.
- GitHub-hosted checks on exact `9fe7f98`: `stable checks` success; `nightly decode fuzz smoke` success. Hosted checks are supplementary cross-evidence only.
- Open PRs: none. No WAN/VPS run occurred in this sequence. `READY_LIVE: none` remains authoritative.
- Governance unchanged: item 3 incomplete; item 4 incomplete; `RELEASE_CANDIDATE=false`; `PRODUCTION_READY=false`; `FREEZE=false`; `RELEASED=false`.

The coding agent is explicitly pre-authorized to repair the R9-2H front, add all remaining discriminating process tests, run/persist the final exact-tree local gate, then continue through R9-3..R9-12 and Q10/Q11/Q12 without waiting for another reviewer cycle.

## Accepted progress

### H-R9-010B incomplete settlement — remains CLOSED

Do not reopen absent contradictory source/test evidence. Incomplete Carrier settlement remains a typed terminal negative and cannot fall through into health/failover as a success premise.

### M-R9-009 single absolute deadline — remains CLOSED

Settlement continues to reuse `application_deadline`; no fresh post-confirmation time budget was reintroduced.

### Reversed-order server seam — useful partial progress

`9fe7f98` adds a test-only server seam that can defer record 0's Session DeliveryAck and emit record 1's ACK first. This is the correct kind of fault injection for R9-2H P1 and does not require a wire/crypto architecture change.

However, the client-side confirmation treatment is not correct and P1 is **not** closed.

# OPEN HIGH — H-R9-011: later exact DeliveryAck manufactures earlier confirmation

`SessionRuntime::delivery_ack` is watermark-based: it computes `end = offset + len`, then releases `end - current_confirmed` bytes if that delta fits the in-flight accounting. It does not require `offset == current_confirmed`.

With two 16-byte reliable-owned records:

```text
record 0 = offset 0..16
record 1 = offset 16..32
confirmed watermark = 0
send_inflight = 32
```

record 1's exact ACK arriving first causes `delivery_ack(16,16)` to advance the watermark directly to 32 and release all 32 bytes. That implicitly confirms record 0 before record 0's exact ACK exists. When record 0's ACK arrives later, `end=16 < current=32` returns `RuntimeError::Protocol`.

The new `9fe7f98` caller then treats that `Protocol` as “already covered” and continues. `recv_udp_delivery_ack` has already removed the matching logical record from `outstanding` before the caller applies Session confirmation, so the outstanding set can become empty without two exact confirmations.

This violates the existing R9-2H P1 contract: reversed order must not manufacture confirmation for the earlier logical range. It is mechanically repairable without changing Session core semantics, so R9-3 stays blocked on this local HIGH rather than escalating to a maintainer architecture decision.

## Required repair shape — bounded pending logical-ACK owner

Do **not** redefine `SessionRuntime::delivery_ack`, wire grammar, ACK architecture, crypto framing, or policy values for this repair.

Use the current reliable receive/demux owner plus a bounded pending logical-ACK set:

1. Authentication/demux may recognize an exact Session DeliveryAck in any arrival order.
2. An out-of-order exact ACK is retained as pending evidence; it does **not** yet advance Session confirmation.
3. Only a pending ACK whose exact `offset == SessionRuntime::confirmed_watermark(stream)` may be applied to `delivery_ack`.
4. After one exact contiguous ACK applies, drain any now-contiguous pending ACK(s) in order.
5. Remove an `outstanding` logical record only after its exact ACK has successfully applied to SessionRuntime. Do not use `RuntimeError::Protocol` as a generic success/covered signal.
6. Pending ACK storage is bounded by the already-bounded reliable-owned `outstanding` set. Do not invent a new capacity/TTL/LRU value.
7. Duplicate/stale/unmatched logical ACKs remain typed bounded negatives under the existing operation-wide malformed policy.
8. Carrier packet ACK remains Carrier-local and never advances Session confirmation.

This preserves Session-above-Carrier layering and closes the false evidence promotion in the integration layer.

# Queue front — execute continuously now

## R9-2H-A — repair H-R9-011

Implement the bounded pending logical-ACK owner above. Add focused deterministic tests if useful, but closure requires the built-binary process test below.

## R9-2H-P1 — reversed logical Session DeliveryAck process regression

Run real built `failover-server` / `failover-client` with the reversed-order seam and prove:

- record 1 ACK is observed before record 0 ACK;
- record 1 is buffered rather than used to confirm record 0;
- record 0 exact `(stream,offset,len)` confirmation applies once;
- then record 1 exact confirmation applies once;
- final logical outstanding set becomes empty only after both exact confirmations;
- Carrier packet ACK remains Carrier-local;
- Recovery settles to zero on success;
- no duplicate/conflict application delivery is created.

Prefer offset-bearing client diagnostics for **every** exact logical confirmation so the test discriminates 0 and 16 rather than inferring from a count.

The current `9fe7f98` source changes only `main.rs`; it does not satisfy this built-binary test requirement by itself.

## R9-2H-P2 — `--reliable-udp + migration-back` reserved-record ownership

Use the real built command path and pin:

- reserved final logical record is not `Recovery`-tracked before post-promotion return authorization;
- it is not sent by the legacy pre-promotion UDP uncertain direct-send path;
- only the explicit later post-promotion owner may track/send it;
- assert the exact reserved offset/record, not just aggregate counts.

No new migration policy.

## R9-2H-P3 — persistent malformed budget across Carrier feedback

Through the real built process path force one reliable receive/settlement operation:

```text
malformed #1
malformed #2
canonical Carrier ACK
malformed #3
```

malformed #3 must hit the same operation-wide `MAX_POST_HANDSHAKE_MALFORMED` ceiling. Session/Carrier ACK processing must not reset the budget. The run must terminate at the typed bounded negative, with no spin/silent continuation. Do not introduce a new policy value.

## R9-2H-P4 — retain incomplete-settlement regression

Keep `reliable_udp_incomplete_settlement_fails_not_settled` green. Do not weaken it while repairing P1-P3.

## R9-2H-GATE — one final exact pushed-tree provenance

After H-R9-011 and P1-P3 are all on one final reachable source/test SHA, run and persist developer-local evidence:

```text
PYTHONDONTWRITEBYTECODE=1 bash scripts/check.sh
git diff --check
```

Record exact pushed SHA, UTC start/end, Linux OS/arch, Rust stable version, exit codes, and clean initial/final tree. If no wire decoder/parser/crypto framing code changed, do not mechanically add a fuzz obligation; hosted fuzz remains separate supplementary evidence.

R9-2H closes only when the final reachable tree simultaneously has:

- P1-P4 built-binary regressions green;
- exact logical ACK evidence cannot cumulatively manufacture an earlier confirmation;
- one authenticated receive/demux owner;
- one operation-wide malformed budget;
- one absolute receive-operation deadline;
- success only when logical outstanding work and Carrier in-flight settlement are both complete;
- incomplete settlement terminal/gated;
- full local gate + `git diff --check` + clean-tree provenance.

**Once R9-2H is green, proceed immediately to R9-3. Do not wait for reviewer cadence.**

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

**Not READY yet — implementation/evidence dependency.** R9-2H and R9-3..R9-12 are the local unlock chain. Once Q11 creates a specific changed `READY_LIVE` row, standing authorization already permits one bounded self-owned TCP/UDP run; do not ask again for generic WAN permission.

# Non-blocking policy/authority gates

Keep separate from this mechanically determined R9 chain:

- `SessionRuntime.events` retention (`POLICY_BLOCKED_RESOURCE_BOUND`);
- D019 source-retention/no-reset policy;
- RSEC-001 capacity/adversarial-load suitability;
- signing/key custody/SBOM/publication trust;
- previous frozen-release interoperability;
- final RC/freeze/release/production authority.

Do not invent policy values while working R9.
