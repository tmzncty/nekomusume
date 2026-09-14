# ChatGPT reviewer handoff — H-R9-010B closed at `54da45f`; finish R9-2H process evidence, then continue R9

## Current repository truth

- Latest developer source/test SHA reviewed: exact `54da45fcb5e818c3137ded0f2460565f72c4b0af` (`fix(cli): R9-2 incomplete settlement is terminal, not fallthrough (H-R9-010)`).
- Reviewer bounded recheck: `docs/reviews/r9-2h-terminal-settlement-recheck-54da45f-20260914.md` (reviewer commit `0db56ae`).
- GitHub-hosted checks on exact `54da45f`: `stable checks` success; `nightly decode fuzz smoke` success. Hosted checks are supplementary cross-evidence only.
- No developer-local exact-tree provenance has yet landed for exact `54da45f`; the required local `scripts/check.sh` / `git diff --check` / clean-tree / UTC / OS-arch / Rust-stable evidence remains outstanding.
- Open PRs: none. No WAN/VPS run occurred in this sequence. `READY_LIVE: none` remains authoritative.
- Governance unchanged: item 3 incomplete; item 4 incomplete; `RELEASE_CANDIDATE=false`; `PRODUCTION_READY=false`; `FREEZE=false`; `RELEASED=false`.

The coding agent is pre-authorized to finish the remaining R9-2H evidence/test gate immediately and, once green, continue through R9-3..R9-12 and Q10/Q11/Q12 without waiting for another reviewer cycle.

## Accepted progress

### H-R9-010B incomplete-settlement control flow — CLOSED

`54da45f` closes the control-flow half that remained after `6ca11a0`:

- when Carrier settlement ends with `ReliableUdpRuntime::in_flight() != 0`, the client emits `r9_udp_settlement_incomplete`;
- it does not emit `r9_udp_in_flight_settled`;
- it immediately calls `fail("r9 reliable-UDP settlement incomplete")`;
- downstream `CarrierHealthEvidence`, `FailoverController`, `CarrierManager`, uncertain-range handling and fallback work therefore cannot consume incomplete settlement as a successful premise.

The new built-binary regression `reliable_udp_incomplete_settlement_fails_not_settled` runs real `failover-server` / `failover-client` binaries. Test-only `--suppress-r9-ack` withholds Carrier packet ACK while Session DeliveryAck can still arrive, forcing remaining in-flight. The client must exit nonzero and expose the typed incomplete boundary.

Do not reopen H-R9-010B absent contradictory new source/test evidence.

### M-R9-009 single absolute deadline — remains CLOSED

Settlement still reuses `application_deadline`. No fresh post-confirmation deadline was reintroduced.

### H-R9-006 / H-R9-007 single-owner demux plumbing — accepted pending process closure

The current path retains one operation-wide malformed counter across helper returns and uses the same authenticated classifier for logical Session ACK and Carrier packet ACK settlement. The remaining work is discriminating built-binary evidence, not another decoder/framework rewrite.

# Queue front — execute continuously now

## R9-2H-FINAL — three remaining built-binary regressions + exact-tree provenance

The previous R9-2H gate required four process-level discriminating regressions. Exact `54da45f` adds the incomplete-settlement case; the other three still need to be implemented through the real built `failover` command path.

### P1 — reversed logical Session DeliveryAck order

Create a test-only server seam that, for the two reliable-owned logical records, deliberately emits Session DeliveryAck for record 1 before record 0.

Require all of:

- both exact `(stream, offset, len)` logical ranges confirm exactly once;
- order reversal does not lose/swallow the later record or manufacture confirmation for the earlier one;
- Carrier packet ACK remains Carrier-local and never becomes Session confirmation;
- Recovery settles to zero on the successful run;
- no duplicate/conflict application delivery is created.

Do not implement a second receive owner to make the test pass; exercise the current bounded authenticated demux.

### P2 — `--reliable-udp + migration-back` reserved-record ownership

Use the real built command path and pin the existing ownership invariant:

- the reserved final logical record must not be `Recovery`-tracked before post-promotion return authorization;
- it must not be sent by the legacy pre-promotion UDP uncertain direct-send path;
- only the explicit later post-promotion owner may track/send it;
- the test should discriminate the exact reserved offset/record, not just count total events.

This is ownership evidence, not a new migration policy.

### P3 — persistent malformed budget across Carrier feedback

Through the real built process path, force this authenticated sequence during one reliable receive/settlement operation:

```text
malformed #1
malformed #2
canonical Carrier ACK
malformed #3
```

Require malformed #3 to hit the same operation-wide `MAX_POST_HANDSHAKE_MALFORMED` ceiling. A Session DeliveryAck or applied/rejected Carrier ACK must not reset the budget. The run must fail/terminate at the bounded negative and must not spin or silently continue.

Do not introduce a new malformed-policy value.

### P4 — incomplete settlement — already present, retain it

Keep `reliable_udp_incomplete_settlement_fails_not_settled` green. Do not weaken it while adding P1-P3.

## R9-2H exact closure gate

After P1-P3 land, on the final pushed source/test SHA run and persist developer-local exact-tree evidence:

```text
PYTHONDONTWRITEBYTECODE=1 bash scripts/check.sh
git diff --check
```

Record exact pushed SHA, UTC start/end, Linux OS/arch, Rust stable version, exit codes, and clean initial/final tree. If any wire decoder/parser/crypto framing code was changed, also use the repository-pinned fuzz toolchain exactly as required; do not run it mechanically for unrelated CLI-only tests.

R9-2H closes only when the final reachable tree simultaneously has:

- all four built-binary regressions green;
- one authenticated receive/demux ownership model;
- one operation-wide malformed budget;
- one absolute receive-operation deadline;
- success only when logical outstanding work and Carrier in-flight settlement are both complete;
- incomplete settlement terminal/gated;
- full local gate + `git diff --check` + clean-tree provenance.

**Once this gate is green, proceed immediately to R9-3. Do not wait for reviewer cadence just to rename the next slice.**

# Continuous queue after R9-2H closes

Preserve this deep queue; do not collapse to one micro-ticket.

## R9-3 — process Data-loss recovery

Add bounded deterministic post-admission suppression of one/periodic reliable-owned Data packet. Prove congestion admission precedes suppression; PTO fires only after deadline; retransmission uses a fresh authenticated packet number/nonce with stable Session/frame identity; exactly-once Session delivery; Carrier ACK and Session DeliveryAck settle independently; final Recovery drains to zero or yields explicit bounded incomplete/error.

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

**Not READY yet — implementation/evidence dependency.** The rented-VPS policy prioritizes correctness and the local work that unlocks a truthful new live question. R9-2H-FINAL and R9-3..R9-12 are that unlock chain. Once Q11 creates a specific changed `READY_LIVE` row, standing authorization already permits one bounded self-owned TCP/UDP run; do not ask again for generic WAN permission.

# Non-blocking policy/authority gates

Keep separate from this mechanically determined R9 chain:

- `SessionRuntime.events` retention (`POLICY_BLOCKED_RESOURCE_BOUND`);
- D019 source-retention/no-reset policy;
- RSEC-001 capacity/adversarial-load suitability;
- signing/key custody/SBOM/publication trust;
- previous frozen-release interoperability;
- final RC/freeze/release/production authority.

Do not invent policy values while working R9.
