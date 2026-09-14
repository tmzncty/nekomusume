# ChatGPT reviewer handoff — R9-2H tree green; completion/settlement contract still blocks R9-3

## Current repository truth

- Latest developer source/test SHA reviewed: exact `4c8906a717505485230a5ea31df768e005c546b9` (`fix(cli): persistent malformed budget across demux calls (H-R9-006)`).
- Developer provenance descendant: `7330aed21d18bf9132d080c16fada37cd07b7b68` (`docs/provenance: 4c8906a R9-2H demux repair gate`).
- Reviewer recheck: `docs/reviews/r9-2h-recheck-4c8906a-20260914.md` (reviewer commit `2a789fa`).
- The previous compile BLOCKER from exact `1786f2f` is closed. `4c8906a` updates all four stale direct helper test callsites and restores a green exact source/test tree.
- Developer-local exact-tree provenance for `4c8906a`: `PYTHONDONTWRITEBYTECODE=1 bash scripts/check.sh` exit 0, `git diff --check` exit 0, clean initial/final tree, UTC start/end, Linux x86_64, rustc 1.98.0.
- GitHub-hosted `stable checks` and `nightly decode fuzz smoke` on exact `4c8906a` are both green; they remain supplementary cross-evidence, not substitutes for developer-local provenance.
- Open PRs: none. No WAN/VPS run occurred in this sequence. `READY_LIVE: none` remains authoritative.
- Governance unchanged: item 3 incomplete; item 4 incomplete; `RELEASE_CANDIDATE=false`; `PRODUCTION_READY=false`; `FREEZE=false`; `RELEASED=false`.

R9-3 is **not READY yet**. The coding agent is pre-authorized to finish R9-2H immediately and, once the closure gate is genuinely satisfied, continue through R9-3..R9-12 and Q10/Q11/Q12 without waiting for another reviewer cycle.

## Accepted progress

### Compile/full-gate blocker — CLOSED

`4c8906a` is the correct minimal repair for the four `E0061` stale test callsites created when `recv_udp_delivery_ack` gained the caller-owned `malformed: &mut usize` parameter. The source/test tree is green locally and in hosted checks.

### H-R9-006 malformed-budget plumbing — ACCEPTED AS PARTIAL CLOSURE

The production reliable receive path creates one `malformed` counter before logical confirmation and passes the same counter through later Carrier-ACK drain calls. A successful/rejected Carrier ACK return therefore does not reset the invalid authenticated-plaintext budget.

This plumbing is correct but the overall operation is still split into two caller loops, so R9-2H remains open on the completion/deadline contract below.

# Queue front — execute continuously now

## HIGH H-R9-010 — incomplete Carrier settlement still masquerades as settled

Current `failover_client` still performs:

1. a logical-confirmation phase while `!outstanding.is_empty()` using `application_deadline`;
2. then a Carrier settlement phase while `rt.in_flight() > 0`.

In the settlement phase, `recv_udp_delivery_ack` timeout / receive error / malformed-bound exhaustion reaches `Err(_) => break`. The caller then still emits:

- `r9_udp_packet_ack_outcomes` with `remaining_in_flight`; and
- **`r9_udp_in_flight_settled` even when `rt.in_flight()` is nonzero**;

then continues into health/failover logic.

Required invariant:

```text
reliable_receive_complete := outstanding.is_empty() && rt.in_flight() == 0
```

Before that predicate is true, timeout/error/malformed exhaustion is an explicit bounded incomplete/error result. It must not emit `r9_udp_in_flight_settled`, and downstream R9 health/failover logic must not consume a successful-settlement premise.

### Preferred minimal implementation shape

Do not add a second decoder, new wire type, or new policy value. Collapse caller ownership into one bounded reliable receive-operation helper/state object that owns for the entire operation:

- outstanding logical records;
- one absolute application/receive deadline;
- the persistent malformed counter;
- logical confirmation count;
- Carrier ACK applied/rejected counters;
- the one authoritative `ReliableUdpRuntime`.

Its completion loop is conceptually:

```text
while !outstanding.is_empty() || rt.in_flight() != 0:
    classify one authenticated plaintext exactly once
    Session DeliveryAck -> apply SessionRuntime::delivery_ack for exact matched range
    Carrier ACK -> apply Recovery feedback only
    bounded negative -> consume the same malformed budget
```

Any error before the predicate becomes false returns typed incomplete/error and terminates this R9 operation. Only the complete return path may emit `r9_udp_in_flight_settled`.

## MEDIUM M-R9-009 — one absolute receive-operation deadline

Current source still creates a new:

```text
settle_deadline = Instant::now() + Duration::from_secs(secs.min(10))
```

after logical confirmation has already consumed time from `application_deadline`.

Remove the fresh post-confirmation extension. Establish one absolute deadline before the reliable receive owner starts and use it until complete or explicit failure. Carrier settlement does not receive a new time budget merely because Session confirmations happened first.

## MEDIUM M-R9-008 — required built-binary/process closure regressions

`4c8906a` changes only four in-module helper-test callsites. Before R9-2H closes, add all three through the real built `failover` command path:

1. **Reversed logical ACK order** — two reliable-owned logical records; test-only server emits Session DeliveryAck for record 1 before record 0. Require both exact logical ranges to confirm once, Carrier ACKs to remain Carrier-local, Recovery to settle to zero, no conflict.
2. **Reliable + migration-back reservation** — reserved final logical record is neither Recovery-tracked nor legacy wire-sent before post-promotion return authorization; only its explicit later owner sends it.
3. **Persistent malformed budget across Carrier feedback** — authenticated malformed #1 -> malformed #2 -> canonical Carrier ACK -> malformed #3 reaches the same operation-wide `MAX_POST_HANDSHAKE_MALFORMED` ceiling. Applied/rejected Carrier ACK or Session DeliveryAck does not reset it.

Focused helper/unit tests may supplement these but do not replace built-binary process evidence.

## R9-2H exact closure gate

R9-2H closes only on a reachable pushed source/test SHA where all are simultaneously true:

- focused regressions compile/pass;
- full `PYTHONDONTWRITEBYTECODE=1 bash scripts/check.sh` passes;
- `git diff --check` passes and initial/final tree is clean;
- developer-local provenance records exact pushed SHA, UTC times, OS/arch, Rust stable and exit codes;
- one persistent malformed budget spans the whole reliable receive/settlement operation;
- every authenticated plaintext is classified once as Session DeliveryAck / Carrier packet ACK / bounded negative;
- success requires `outstanding.is_empty() && in_flight()==0`;
- incomplete settlement cannot emit `r9_udp_in_flight_settled` or continue as success;
- one absolute receive-operation deadline is used;
- all three built-binary discriminating regressions pass.

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

**Not READY yet — implementation/evidence-instrumentation dependency.** The rented-VPS priority explicitly prefers local work that directly unlocks a truthful VPS run, which is exactly this R9 chain. Once R9-2H..R9-12 and Q11 establish a specific changed live question, standing authorization already covers one bounded self-owned TCP/UDP run; do not ask again for generic WAN permission.

# Non-blocking policy/authority gates

Keep separate from the mechanically determined R9 chain:

- `SessionRuntime.events` retention (`POLICY_BLOCKED_RESOURCE_BOUND`);
- D019 source-retention/no-reset policy;
- RSEC-001 capacity/adversarial-load suitability;
- signing/key custody/SBOM/publication trust;
- previous frozen-release interoperability;
- final RC/freeze/release/production authority.

Do not invent policy values while working R9.
