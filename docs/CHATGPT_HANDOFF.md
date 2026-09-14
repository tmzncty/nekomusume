# ChatGPT reviewer handoff — `6ca11a0` improves settlement evidence but R9-2H still blocks R9-3

## Current repository truth

- Latest developer source/test SHA reviewed: exact `6ca11a0c505d7634c487520b96e37d3a358309dd` (`fix(cli): R9-2 settlement typed outcome + single deadline`).
- Developer provenance descendant: `047bf8e6cdddba207755d98e952740329b02d362` (`docs(provenance): 6ca11a0 R9-2 settlement closure gate`).
- Reviewer recheck: `docs/reviews/r9-2h-settlement-recheck-6ca11a0-20260914.md` (reviewer commit `3193d5c`).
- Developer-local exact-tree provenance for `6ca11a0`: `PYTHONDONTWRITEBYTECODE=1 bash scripts/check.sh` exit 0, `git diff --check` exit 0, clean initial/final tree, UTC start/end, Linux x86_64, rustc 1.98.0.
- GitHub-hosted `stable checks` and `nightly decode fuzz smoke` on provenance descendant `047bf8e` are both green; they remain supplementary cross-evidence, not substitutes for developer-local provenance.
- Open PRs: none. No WAN/VPS run occurred in this sequence. `READY_LIVE: none` remains authoritative.
- Governance unchanged: item 3 incomplete; item 4 incomplete; `RELEASE_CANDIDATE=false`; `PRODUCTION_READY=false`; `FREEZE=false`; `RELEASED=false`.

R9-3 is **not READY yet**. The coding agent is pre-authorized to finish R9-2H immediately and, once the closure gate is genuinely satisfied, continue through R9-3..R9-12 and Q10/Q11/Q12 without waiting for another reviewer cycle.

## Accepted progress

### R9-2H exact tree — GREEN

`6ca11a0` is a clean source/test tree. The prior compile blocker remains closed and the existing persistent malformed-budget plumbing still passes the full gate.

### M-R9-009 one absolute receive-operation deadline — CLOSED

Settlement now reuses the pre-existing `application_deadline`. It no longer creates a fresh post-confirmation time budget. This is the required minimal correction and does not change wire/Session/Carrier semantics.

### H-R9-010 diagnostic truthfulness — PARTIAL ONLY

`6ca11a0` correctly stops emitting `r9_udp_in_flight_settled` when `ReliableUdpRuntime::in_flight()` is nonzero and instead emits `r9_udp_settlement_incomplete`.

That diagnostic correction is accepted, but **the control-flow half of H-R9-010 is still open**: after the incomplete marker, current `failover_client` falls through unconditionally into `CarrierHealthEvidence`, `FailoverController`, `CarrierManager`, uncertain-range handling, warm-standby setup and later failover logic. An incomplete Carrier settlement therefore still becomes an execution premise for downstream R9 work even though the evidence says it is incomplete.

# Queue front — execute continuously now

## HIGH H-R9-010B — incomplete settlement must terminate/gate the R9 operation

Required success predicate remains:

```text
reliable_receive_complete := outstanding.is_empty() && rt.in_flight() == 0
```

When settlement ends because of timeout, receive error, malformed-budget exhaustion, or any other bounded negative while `rt.in_flight() != 0`:

- emit a typed incomplete/error result (`r9_udp_settlement_incomplete` is acceptable);
- do **not** emit `r9_udp_in_flight_settled`;
- do **not** continue as if the reliable receive phase succeeded;
- do **not** initialize downstream health/failover logic from that incomplete state.

### Preferred minimal implementation shape

Do not add a new wire type, policy value, second decoder or parallel Session owner. Keep the existing authenticated classifier and make the operation result explicit:

```text
Complete {
  outstanding == 0,
  in_flight == 0,
  logical_confirmations,
  carrier_ack_applied,
  carrier_ack_rejected,
  malformed_used
}

Incomplete {
  reason,
  remaining_outstanding,
  remaining_in_flight,
  ...typed counters...
}
```

A minimal direct `fail(...)` / early return after the typed incomplete diagnostic is also acceptable if it preserves cleanup and existing CLI contract. The important invariant is control flow, not a particular API shape.

## MEDIUM M-R9-008 — built-binary/process closure regressions still missing

Exact `6ca11a0` changes only `crates/neko-cli/src/main.rs`; it adds no discriminating process regression. Before R9-2H closes, add all four through the real built `failover` command path:

1. **Reversed logical ACK order** — two reliable-owned logical records; test-only server emits Session DeliveryAck for record 1 before record 0. Require both exact logical ranges to confirm once, Carrier ACKs to remain Carrier-local, Recovery to settle to zero, no conflict.
2. **Reliable + migration-back reservation** — reserved final logical record is neither Recovery-tracked nor legacy wire-sent before post-promotion return authorization; only its explicit later owner sends it.
3. **Persistent malformed budget across Carrier feedback** — authenticated malformed #1 -> malformed #2 -> canonical Carrier ACK -> malformed #3 reaches the same operation-wide `MAX_POST_HANDSHAKE_MALFORMED` ceiling. Applied/rejected Carrier ACK or Session DeliveryAck does not reset it.
4. **Incomplete settlement control flow** — force reliable Carrier settlement to end with `remaining_in_flight > 0`; require `r9_udp_settlement_incomplete`, forbid `r9_udp_in_flight_settled`, and prove no downstream successful health/failover continuation occurs.

Focused helper/unit tests may supplement these but do not replace built-binary process evidence.

## R9-2H exact closure gate

R9-2H closes only on a reachable pushed source/test SHA where all are simultaneously true:

- focused and built-binary regressions compile/pass;
- full `PYTHONDONTWRITEBYTECODE=1 bash scripts/check.sh` passes;
- `git diff --check` passes and initial/final tree is clean;
- developer-local provenance records exact pushed SHA, UTC times, OS/arch, Rust stable and exit codes;
- one persistent malformed budget spans the whole reliable receive/settlement operation;
- every authenticated plaintext is classified once as Session DeliveryAck / Carrier packet ACK / bounded negative;
- one absolute receive-operation deadline is used;
- success requires `outstanding.is_empty() && in_flight()==0`;
- incomplete settlement terminates/gates the operation and cannot feed successful downstream health/failover work;
- all four built-binary discriminating regressions pass.

If this closes in one coherent repair, immediately continue to R9-3 without waiting for reviewer cadence.

# Continuous queue after R9-2H closes

Preserve the deep queue. Do not collapse it to one micro-ticket after each commit.

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

**Not READY yet — implementation dependency.** The rented-VPS priority says correctness first and then local work that directly unlocks a truthful VPS question. H-R9-010B + the R9 process regressions are exactly that local unlock path. Once R9-2H..R9-12 and Q11 establish a specific changed live question, standing authorization already covers one bounded self-owned TCP/UDP run; do not ask again for generic WAN permission.

# Non-blocking policy/authority gates

Keep separate from the mechanically determined R9 chain:

- `SessionRuntime.events` retention (`POLICY_BLOCKED_RESOURCE_BOUND`);
- D019 source-retention/no-reset policy;
- RSEC-001 capacity/adversarial-load suitability;
- signing/key custody/SBOM/publication trust;
- previous frozen-release interoperability;
- final RC/freeze/release/production authority.

Do not invent policy values while working R9.
