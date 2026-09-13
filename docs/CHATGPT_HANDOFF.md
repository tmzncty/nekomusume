# ChatGPT reviewer handoff — R9 source path active; R9-2 multi-record repair required before R9-3

## Reviewed repository truth

- Developer source/test progress resumed after the prior stagnation handoff.
- Current reviewed developer head before this handoff: exact `f849fe9c3f82fb3e560bfe4c79f36afc725e8e23`.
- New R9 sequence:
  - `aa483694b83e554fd802bee1250c32162e20106e` — first cross-process `failover --reliable-udp` seam;
  - `ec0319362753c67b3957753bf919c4cc50bc0f94` — developer-local exact-tree provenance for `aa48369`;
  - `f0178e3d460694508b1da9924c17dfe45ea8a221` — second logical record added to reliable-UDP send path;
  - `f849fe9c3f82fb3e560bfe4c79f36afc725e8e23` — style-only collapse of that nested branch.
- Current exact `f849fe9` GitHub-hosted `stable checks` and `nightly decode fuzz smoke` are green. These are extra cross-evidence only; the last persisted developer-local clean exact-tree gate is for `aa48369`, not the later multi-record tree.
- Open PRs: none.
- No WAN/VPS execution occurred in this sequence.
- Governance remains unchanged: item 3 incomplete; item 4 incomplete; `RELEASE_CANDIDATE=false`; `PRODUCTION_READY=false`; `FREEZE=false`; `RELEASED=false`; current `READY_LIVE: none`.

## Reviewer report

Independent bounded review is persisted at:

`docs/reviews/independent-r9-cross-process-f849fe9-20260914.md`

Verdict: R9 has genuinely started on the correct existing failover process seam, but **R9-2 is not closed**. Do not revert the useful `aa48369` integration. Repair the current multi-record ownership/demux defects, gate the repaired exact tree, then continue immediately into R9-3 onward without waiting for reviewer cadence.

# Current blocking findings for R9 progression

## H-R9-001 — multi-record Session DeliveryAck is swallowed by the R9 receive demux

`recv_udp_delivery_ack` still has a single `expected: &OutboundRecord`. It returns only for that one logical Session acknowledgement. When a `ReliableUdpRuntime` is present, every other authenticated plaintext is treated as a possible Carrier ACK and then silently `continue`s if it is not a valid `RecordType::Ack`.

After `f0178e3`, record 2 is sent through reliable UDP, so its Session `DeliveryAck` may arrive while the helper is waiting for record 1. That acknowledgement is consumed and discarded. After the helper returns, `SessionRuntime::delivery_ack` is called only for record 1. The later settlement loop decodes Carrier ACKs only.

The same branch discards `apply_ack` errors and bypasses the pre-existing malformed-authenticated-message bound while R9 mode is active.

### Required repair

Build one bounded authenticated receive/demux loop for R9 that separates:

1. Carrier packet ACK (`RecordType::Ack`) -> canonical decode -> `apply_ack`; typed rejection/diagnostic on failure, no mutation on rejection;
2. Session `ProcessMessage::DeliveryAck` -> validate against a bounded outstanding logical-record set/map and apply `SessionRuntime::delivery_ack` exactly once for each matching `(stream, offset, len)`;
3. unexpected authenticated control/malformed plaintext -> bounded rejection/diagnostic, not silent success-path consumption.

Carrier packet ACK and Session delivery confirmation must remain separate evidence domains.

## H-R9-002 — record 2 has both a reliable owner and an untracked direct UDP owner

`f0178e3` sends `records[1]` through `ReliableUdpRuntime`, but the unchanged legacy failover code later again takes `records.get(1)`, reseals it with a fresh AEAD sequence and sends it directly as `udp_uncertain_range_sent` outside the reliable runtime.

That creates two concurrent Carrier sends for the same Session range in the nominal no-loss path and makes reliable recovery ownership non-authoritative. It will confound Data-loss/PTO evidence and can manufacture baseline duplicates.

### Required repair

When `--reliable-udp` is active, logical records selected for the R9 reliable subset must not also traverse the legacy untracked UDP-send branch. For R9-2, carry at least two distinct logical records through one `ReliableUdpRuntime`; start any remaining legacy uncertain/failover subset after those reliable-owned records. Historical behavior without `--reliable-udp` must remain unchanged.

## H-R9-003 — server records packet ACK obligation before valid R9 Data classification

The server currently calls `server_rt.on_packet_received(pn, true)` immediately after authenticated `open_unreliable`, before `ProcessMessage::decode`, before proving `Data`, and before checking the expected Session identity.

The outgoing ACK is polled later inside the valid-Data branch, but the ACK tracker can already contain an earlier authenticated non-Data/control/malformed packet. A subsequent Data packet may therefore emit a range acknowledging traffic that never entered the R9 reliable-Data path, and this grows into an ACK-of-ACK/control-feedback hazard as the bidirectional path expands.

### Required repair

Commit packet receipt to the R9 receiver ACK tracker only after authentication and structurally valid `ProcessMessage::Data` classification with the expected Session identity. Session duplicate/conflict handling stays in `SessionRuntime`. Do not make arbitrary authenticated controls/ACKs ack-eliciting R9 Data packets without an explicit later architecture/wire decision.

## M-R9-004 — current process regression does not prove the new multi-record claim

The existing R9 process regression still checks only process success, presence of packet/Session ACK events, and `remaining_in_flight=0`. `f0178e3` changed source only and added no assertions for two unique logical records, two Session confirmations, zero clean-baseline duplicates/conflicts, or two packet owners being retired.

### Required regression

On the repaired no-loss R9-2 path, prove at minimum:

- at least two distinct Session byte offsets were reliable-owned and wire-sent;
- both logical records were first-delivered exactly once by Session;
- both corresponding Session `DeliveryAck`s were independently validated/applied;
- Carrier ACKs retire both packet owners; final authoritative `in_flight()==0`;
- zero PTO and zero retransmission in this no-loss baseline;
- zero Session conflict, and preferably zero duplicate in the clean baseline;
- cleanup/listener/identity behavior remains valid.

# Immediate continuous queue

External coding agent should execute continuously from current `main`:

## R9-2A — repair authenticated receive demux

Primary owner: `crates/neko-cli/src/main.rs`.

Implement the bounded multi-record Session-ACK + Carrier-ACK demux above. Reuse current `SessionRuntime`, `ReliableUdpRuntime`, `ProcessMessage`, `AckPayload`, deadlines and diagnostics. Do not add a second protocol stack or logging framework.

## R9-2B — restore one authoritative UDP recovery owner per logical range

Prevent reliable-owned records from also taking the legacy direct `udp_uncertain_range_sent` path. Keep non-R9 historical behavior unchanged.

## R9-2C — tighten server packet-ACK admission boundary

Move `on_packet_received` to the valid authenticated expected-Session Data path. Add focused negative tests showing authenticated non-Data/control/malformed input cannot enter outgoing R9 packet ACK ranges.

## R9-2D — make the process test genuinely multi-record and discriminating

Add assertions listed in M-R9-004. The test must fail if either logical Session ACK is swallowed, if record 2 is sent through both owners, or if a non-Data authenticated packet is spuriously acknowledged by R9 recovery.

## R9-2E — repaired exact-tree gate

After the repair sequence reaches one coherent pushed source/test SHA, run in a clean checkout/worktree:

```bash
PYTHONDONTWRITEBYTECODE=1 bash scripts/check.sh
git diff --check
```

Record exact SHA, UTC start/end, OS/arch, rustc, exit codes, clean initial/final tree. Hosted CI remains separate cross-evidence.

No new wire/parser grammar is required by these repairs. If implementation nevertheless changes external decode/framing grammar, run the pinned decode fuzz commands before closure.

# Pre-authorized continuation after repaired R9-2

Do not stop after the repaired no-loss baseline. Continue immediately:

## R9-3 — process Data-loss recovery

Add bounded deterministic post-admission suppression of one/periodic Data packet. Prove congestion admission precedes suppression; PTO fires only after deadline; retransmission uses a fresh authenticated packet number/nonce with stable frame/Session identity; exactly-once Session delivery; final recovery drains to zero or returns an explicit bounded partial/failure outcome.

## R9-4 — ACK-loss and reorder/delayed-original

Separate cases: packet ACK emitted then suppressed; replacement arrives before delayed original; late original after replacement. Require one logical Session delivery, Session-owned duplicate suppression, no conflict, fresh packet numbers and deadline-driven PTO.

## R9-5 — tamper / future ACK / malformed feedback negatives

Prove unauthenticated/tampered Data creates no packet-ACK obligation and no Session receive; tampered ACK creates no recovery mutation; future/never-sent ACK is typed rejection + atomic state; stale/duplicate ACK does not fabricate RTT/loss/Session evidence; malformed bounds are finite/panic-free.

## R9-6 — pacing/cwnd/plaintext-owner atomicity

Use bounded test-only limits. Initial and retransmit sends consult congestion admission; refusal consumes no Session byte space and no committed packet/plaintext/recovery owner; pacing deadlines are finite; teardown releases ownership.

## R9-7 — truthful process observability/result contract

Machine-readable evidence should distinguish offered/admitted/wire-sent/suppressed Data, ACK emitted/wire-sent/suppressed/applied/rejected, PTO due/fired, retransmit attempted/admitted/wire-sent/refused, resolved acked/lost + remaining in-flight, Session first/duplicate/conflict/application bytes/confirmation, cleanup and final outcome. Counters increment only after represented actions succeed. No payload/key/private-topology logging.

## R9-8 — actual authenticated warm TCP standby

Only after R9-2..7 are green: reuse existing failover TCP connect/accept, canonical negotiation, fresh Noise trust/authz, resume/readiness tuple+generation+delivery-epoch binding and resource admission. No application Data on standby before atomic promotion.

## R9-9 — resolved UDP health -> hysteresis -> real TCP promotion

Fresh resolved UDP outcomes drive Carrier health. Prove recoverable loss stays UDP; PTO-only sample cannot erase later loss; distinct bad resolved intervals cross committed hysteresis and promote only to the actually ready TCP standby; invalid/unready standby yields `FallbackFailed`; historical old loss is not replayed as new evidence.

## R9-10 — uncertain Session replay across UDP -> TCP

At promotion, replay at least one uncertain logical Session range over promoted TCP. Draining UDP accepts no new application Data. Receiver deduplicates by Session/stream/byte-offset and application bytes are delivered exactly once. Do not add TCP packet ACK.

## R9-11 — timeout/shutdown/cleanup matrix

Cover setup, application, PTO/settlement deadlines, controlled stop, failed standby promotion, malformed/tampered input, listener/socket/process cleanup and same-address rebind where existing harness supports it.

## R9-12 — coherent exact-tree gate + independent bounded review

Independent review must challenge Session-above-Carrier layering, authenticated packet identity/ACK, bounded plaintext ownership, deadline-driven PTO/pacing/cwnd, fresh health/hysteresis, real warm TCP readiness/promotion, uncertain Session replay/dedup, result truthfulness and cleanup/no public exposure.

BLOCKER/HIGH -> smallest repair + regression + re-gate + continue. LOW/NOTE does not halt progression.

# Q10/Q11/Q12 after R9

- Q10: integrate only genuinely new R9 evidence into existing observability; no second logging framework.
- Q11: reconcile `docs/status.md`, `IMPLEMENTATION_PLAN.md`, `ROADMAP.md` and release packet only for earned status. If cross-process R9 + real TCP promotion is green and independently reviewed, create a **new specific READY_LIVE row** for the changed implementation/hypothesis.
- Q12: exactly one minimal changed-hypothesis self-owned client<->VPS run under standing authorization, with temporary unprivileged listeners, exact commit/binary/parameters, authenticated recovery/Session/switch evidence and cleanup. Preserve negative result; no unchanged same-class retry.

# VPS opportunity

**Not READY yet.** The new R9 source path is a materially changed implementation, but current H-R9-001..003 mean a WAN run would not produce trustworthy multi-record/recovery evidence. This is an **implementation correctness dependency**, not a permission blocker.

Once repaired R9-2..R9-12 and Q11 make the changed question READY, standing authorization already permits the bounded self-owned TCP/UDP experiment. Do not ask again for ordinary WAN authorization.

# Non-blocking policy/authority gates

These remain separate and do not stall the local R9 repair/progression chain:

- `SessionRuntime.events` retention (`POLICY_BLOCKED_RESOURCE_BOUND`);
- D019 source-retention/no-reset policy;
- RSEC-001 capacity/adversarial-load suitability;
- signing/key custody/SBOM/publication trust;
- previous frozen-release interoperability;
- final RC/freeze/release/production authority.

Do not invent policy values while working R9.