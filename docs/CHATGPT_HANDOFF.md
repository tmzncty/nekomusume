# ChatGPT reviewer handoff — R7 closure rejected; three HIGHs block R8/live promotion

## Reviewed repository truth

- Current default branch: exact `38adf0e252de4d8011339b4fe1dc945fe268d7b4` (`docs(provenance): eeb49d7 repaired R7 runtime gate`).
- Current substantive source tree is exact `eeb49d71572df1746f2924e72d4b515d8fa3327d`; `38adf0e` is provenance documentation only.
- Since the previous reviewer handoff `bd66dc9`, developer source/test work landed:
  - `4f98483`: cwnd admission + pacing interval API;
  - `c661921`: explicit fallback success/failure and real switch event;
  - `1e53ffe`: a new carrier-owned `(stream, offset)` logical-delivery dedup map;
  - `b1959d2`: coherent socket/crypto fixture rewired to `ReliableUdpRuntime`;
  - `eeb49d7`: mutable manager + recovery-engine accessors for the fixture;
  - `38adf0e`: developer-local exact-tree gate/provenance for `eeb49d7`.
- Developer-local exact-tree provenance for `eeb49d7` records `scripts/check.sh`, `git diff --check`, clean initial/final tree, Linux x86_64 and Rust 1.98.0 all green. Current GitHub-hosted stable checks and decode fuzz smoke are also green on `38adf0e`. Hosted checks remain cross-evidence only.
- No new VPS/WAN evidence is accepted in this sequence. `READY_LIVE` remains none.
- Governance remains unchanged: item 3 incomplete; item 4 incomplete; `RELEASE_CANDIDATE=false`; `PRODUCTION_READY=false`; `FREEZE=false`; `RELEASED=false`; D019, `SessionRuntime.events`, RSEC-001, signing/SBOM and final release authority remain separate gates.

## Reviewer verdict on the new work

### M-RUDP-009 — ACCEPT_WITH_DEPENDENCY

`4f98483` correctly adds a real `can_send` gate and deterministic pacing interval to `ReliableUdpRuntime`. Cwnd refusal itself is atomic with respect to the runtime because it returns before recovery mutation.

However, a cwnd-admitted send is still not transactionally safe because H-RUDP-011 remains open below. Do not call R7 accepted until the plaintext/recovery ownership transaction is repaired.

### M-RUDP-010 — ACCEPT

`c661921` closes the previously identified false-success path in `poll_health`: `WarmFallback` now carries the real switch event only after UDP fail + TCP activate succeed, and transition failure returns `FallbackFailed` rather than being swallowed.

This does not validate the health input itself; H-RUDP-001D remains open.

### H-RUDP-008 implementation — REJECTED AS CURRENTLY LAYERED

`1e53ffe` demonstrates the correct *identity idea* — `(stream, offset)` is stable across fresh packet numbers — but implements Session logical-delivery ownership inside `neko-carrier::ReliableUdpRuntime` as an unbounded `BTreeMap<(u32,u64), Vec<u8>>`.

That violates two already-committed repository invariants:

1. **Session is above Carrier.** Logical Session delivery/dedup belongs to `neko-session`, not to a UDP Carrier recovery runtime.
2. **Retained state must be bounded.** The new `delivered` map has no frame/byte/window bound and retains full application payload copies indefinitely for the runtime lifetime.

The repository already has a bounded `SessionRuntime::receive(InboundRecord, ...)` path with stream/offset ownership, exact-duplicate suppression, conflicting-duplicate fail-closed behavior, queue/window/record/total-byte bounds, and Session event accounting. Reuse that existing component in the integration fixture/runtime composition rather than creating a second Session implementation in `neko-carrier`.

The coherent fixture also currently calls `let _ = rt.deliver_logical(...)`, so even the newly added `DeliveryConflict` can be swallowed at the integrated receive path. The claimed "conflict fail closed" property is therefore not established by Q6.

### Q6 coherent fixture — NOT ACCEPTED

`b1959d2` is useful integration progress: it uses real UDP sockets, authenticated packet numbers, ACK emission, retransmission, cwnd admission and automatic fallback in one fixture. But it composes unresolved HIGH defects below and therefore cannot be the R7 acceptance fixture yet.

### `38adf0e` provenance — GATE FACT ACCEPTED; CLOSURE CLAIM REJECTED

The developer-local exact-tree gate itself is valid evidence that `eeb49d7` builds/tests cleanly. The sentence describing `eeb49d7` as "the final tree after closing the reviewer's second-round R7 findings" is not accepted: H-RUDP-011 and H-RUDP-001D from the previous handoff are still present in the exact source, and the new H-RUDP-012 layering/boundedness defect was introduced by `1e53ffe`.

Do not rewrite the historical provenance file. After repair, add a superseding provenance/review note that explicitly states `38adf0e` was green-build evidence but not R7 semantic closure.

## Open findings — stop R8/live expansion until closed

### H-RUDP-011 — duplicate/unbounded plaintext owner + non-transactional send remains open

**Severity: HIGH. Blocks R7 acceptance and R8/READY_LIVE.**

Current `ReliableUdpRuntime::on_packet_sent` still performs:

1. `recovery.on_sent(...)` — mutates Recovery/Reno/sent state;
2. `let _ = retransmit.track(frame, plaintext)` — discards `Capacity`, `FrameTooLarge`, and `Conflict`;
3. copies the same plaintext again into a separate unbounded `frame_plaintext` map.

`pto_probe()` reads from that duplicate map, while `RetransmitBuffer` is supposed to be the hard-bounded authoritative plaintext owner. `on_retransmit_sent()` also records a replacement packet without first proving that the stable frame still has retained plaintext ownership.

Required repair contract:

- delete the duplicate `frame_plaintext` map; `RetransmitBuffer` is the single authoritative plaintext owner and `get()` supplies PTO/retransmit bytes;
- never discard `track()` errors;
- cwnd check happens first;
- reserve/track plaintext before committing the packet to recovery, with exact rollback if later `recovery.on_sent` rejects;
- rollback must not delete an already-existing identical stable frame that predated the failed packet attempt;
- `on_retransmit_sent` must fail before recording a packet unless the stable frame is currently retained;
- no outstanding recovery packet may exist without resealable retained plaintext ownership;
- teardown clears packet->frame state and the one bounded owner deterministically.

Mandatory tests:

1. frame larger than plaintext bound -> atomic refusal; recovery/reno/packet map/retained bytes unchanged;
2. first max-frame/max-byte excess -> same atomic refusal;
3. same FrameId + conflicting bytes -> same atomic refusal;
4. recovery rejection after a new plaintext reservation rolls back only that new reservation;
5. recovery rejection while an identical frame was already retained preserves the old frame;
6. retransmit of an unretained/retired FrameId fails before a packet is recorded;
7. original + overlapping replacement keeps one plaintext copy and releases it only after authoritative final-copy retirement;
8. repeated failures cannot exceed existing `RetransmitBuffer` limits.

Do not change/increase the existing limits to make tests pass.

### H-RUDP-001D — health loss denominator is still send-time, not resolution-time

**Severity: HIGH. Blocks truthful automatic degradation/fallback.**

Current `PathRecovery::fresh_health_sample()` still computes:

- `delta_sent = packets_sent - last_health_sent`
- `delta_lost = packets_lost - last_health_lost`
- `loss = delta_lost / delta_sent`, falling back to zero when `delta_sent == 0`.

The previous handoff gave the counterexample and the exact code is unchanged. A PTO health sample can consume the send denominator while those packets remain unresolved; a later ACK can newly declare old packets lost with no new sends, producing `delta_lost > 0`, `delta_sent == 0`, and therefore a falsely clean zero-loss sample.

Required repair contract:

- keep lifetime sent/lost counters only for diagnostics;
- add manager-facing resolved-outcome accounting: resolved packet count = newly acked + newly lost, and resolved-loss count = newly lost;
- update those counters exactly from each `RecoveryResult`;
- `fresh_health_sample()` consumes deltas of resolved outcomes, not send totals;
- PTO-only fresh evidence can carry PTO state but must not consume/erase the denominator for packets that are not resolved yet;
- preserve same-epoch/bare-send freshness rules and current hysteresis thresholds.

Mandatory tests:

1. send -> consume PTO health -> later ACK declares old packets lost with no intervening send -> non-zero resolved loss;
2. one old bad resolved outcome followed by clean resolved ACK outcomes does not replay old loss;
3. one ACK with 1 acked + N lost uses exactly `1+N` as resolved denominator and `N` as loss numerator;
4. PTO-only sample does not erase a later resolved-loss sample;
5. same-epoch polls and bare sends emit no new health sample;
6. packet feedback remains incapable of Path validation or Session delivery confirmation.

### H-RUDP-012 — Session delivery/dedup moved into Carrier and is unbounded

**Severity: HIGH architecture + resource-bound defect. Blocks R7 acceptance and R8.**

Current `ReliableUdpRuntime` owns `delivered: BTreeMap<(u32,u64), Vec<u8>>` and `dedup_suppressed`. This is Session/application state inside the Carrier recovery component and it has no hard retained-state bound.

Required repair:

- remove `delivered`, `dedup_suppressed`, `deliver_logical`, and carrier-level `DeliveryConflict` from production `neko-carrier` runtime;
- keep `neko-carrier` production dependencies carrier-local (`neko-reliable`/`neko-wire`); do not add a production dependency on `neko-session` merely to hide the layering violation;
- in the integration fixture / future CLI composition, decode authenticated `ProcessMessage::Data` and pass an `InboundRecord` into an actual bounded `neko_session::SessionRuntime` (or another already-committed Session-layer owner with equivalent bounded semantics);
- open/initialize the Session stream explicitly before receive;
- exact delayed duplicate must be suppressed by Session-layer semantics; conflicting bytes for the same logical identity must propagate an error/fail-closed result and must not be swallowed with `let _`;
- packet ACK/recovery still must not call/stand in for Session delivery acknowledgement.

Tests must prove the composition, including bounded rejection on Session record/queue/window/total limits. No new capacity number is needed: use the existing `RuntimeLimits` and hard ceilings.

## Continuous dependency-ordered queue

Do not wait for the next reviewer between slices. HIGHs stay first; only after all three are closed may R8/live work resume.

### Q1 — H-RUDP-011 single transactional plaintext owner

Implement the exact contract and eight regressions above. Preserve stable FrameId and fresh packet nonce behavior already accepted from `0bd4270`.

### Q2 — H-RUDP-001D resolved-outcome health accounting

Implement resolution-aligned counters and the six regressions above. Preserve M-RUDP-010 switch-result behavior.

### Q3 — H-RUDP-012 restore Session-above-Carrier layering

Remove carrier-owned logical delivery state. Recompose the coherent fixture with bounded `SessionRuntime::receive`; propagate conflicts and resource failures.

### Q4 — coherent R7 fixture rework

Update `crates/neko-carrier/tests/reliable_udp_runtime.rs` so one actual flow proves together:

- real UDP socket send/receive;
- authenticated fresh packet number/nonce;
- bounded canonical ACK and no ACK-of-ACK;
- transactional bounded plaintext ownership;
- stable FrameId across fresh retransmission packet numbers;
- actual cwnd admission + deterministic pacing decision;
- resolved-outcome health evidence;
- Session-layer bounded stream/offset delivery + exact duplicate suppression + conflicting duplicate rejection;
- warm TCP readiness;
- automatic manager fail/promote with truthful `WarmFallback`/`FallbackFailed`;
- packet recovery, health, switch and Session-delivery evidence remain distinct.

Minimum scenarios: no loss; one recoverable loss; ACK loss/PTO replacement; replacement-first then late original; conflicting duplicate; plaintext bound refusal; PTO-sample-before-later-loss; clean recovery after old loss; sustained distinct bad resolved outcomes -> warm fallback; invalid standby -> no false switch.

### Q5 — supersede premature R7 closure claim

After Q1-Q4, add a durable note that classifies `docs/local-gate-eeb49d7-20260913.md` correctly: valid green exact-tree provenance, but its semantic-closure sentence was superseded by the reviewer findings above. Do not rewrite the old file.

### Q6 — final pushed exact-tree local gate

On one coherent source SHA after Q1-Q4:

```text
PYTHONDONTWRITEBYTECODE=1 bash scripts/check.sh
git diff --check
git status --short
```

Persist pushed exact SHA, UTC start/end, exits, OS/arch, stable Rust, clean initial/final tree. If wire/parser/framing code changes, also run pinned decode fuzz; otherwise do not mechanically rerun fuzz solely for carrier/session-only changes.

### Q7 — independent bounded R7 re-review

Challenge exact final source for: transactional owner; retained-state bounds; resolved-health math; Session/Carrier layering; duplicate/conflict propagation; nonce/FrameId separation; cwnd/pacing; truthful fallback result; ACK vs Session evidence separation. Concrete defect -> repair and re-gate; no defect -> precise no-finding note.

### Q8 — R8 executable CLI/lab integration

Only after Q7. Reuse the accepted runtime + existing Session component in current CLI/failover/lab machinery; do not create a second implementation. Add bounded deterministic sender-side packet suppression/loss injection and structured recovery/health/switch/Session counters.

### Q9 — R9 process/socket acceptance

Cover at least no-loss, recoverable loss, ACK loss, PTO replacement, late original, no duplicate logical delivery, conflicting duplicate fail-closed, sustained packet-recovery degradation -> automatic warm TCP fallback, invalid standby -> no false switch, and zero residue cleanup.

### Q10 — observability/release-boundary integration review

Map real recovery ACK/loss/PTO/retransmit, health transition, switch event, Session duplicate/conflict and application delivery to existing bounded observability without promoting packet evidence to logical delivery. Review retained-state bounds again after CLI integration.

### Q11 — status/release reconciliation and READY_LIVE decision

Update `docs/status.md`, `IMPLEMENTATION_PLAN.md`, release packet and handoff only after executable R8/R9 exists. If this creates a genuinely new packet-recovery-driven real-network question, classify the exact `READY_LIVE` row; historical application-reply-cessation evidence does not answer it.

### Q12 — changed-hypothesis self-owned VPS evidence

Only if Q11 produces `READY_LIVE` and the run stays within standing authorization. Minimum bounded evidence should distinguish real packet recovery without fallback from real resolved loss/PTO -> UDP degradation -> same logical Session warm TCP promotion, uncertain replay/dedup, duplicate/lost application bytes, recovery timing, resource observations when available, and cleanup.

### Q13 — item-3/item-4 reconciliation and refill

Preserve all negative evidence. No RC/freeze/release/production promotion follows automatically. Refill from concrete defects or missing evidence exposed by the new live path.

## Accepted pieces that should not be gratuitously rewritten

Unless Q1-Q4 reveal a direct dependency bug, preserve:

- future/unsent ACK rejection;
- canonical ACK grammar and pending-ACK/no-ACK-of-ACK behavior;
- fresh packet number/AEAD nonce separate from stable FrameId;
- per-packet charged-byte Reno accounting;
- M-RUDP-009 cwnd gate/pacing interval API;
- M-RUDP-010 explicit fallback success/failure and real switch event;
- authenticated packet observation before ACK tracking;
- packet feedback / Session delivery evidence separation.

## Separate policy/security gates — still not excuses to idle after HIGH repair

Unchanged:

- `SessionRuntime.events` retained-state policy;
- D019 source-retention/no-reset policy;
- RSEC-001 representative adversarial-load/capacity suitability;
- signing/key custody/SBOM/publication trust;
- previous frozen release / final independent security-release judgment;
- RC/freeze/release/production authority.

Do not invent these decisions during R7 repairs, and do not use them to stop Q1-Q13 once the three current HIGHs are closed.

## Stop conditions

Stop expansion only for an unresolved new BLOCKER/HIGH, a required core Session/Carrier/ACK/crypto/wire architecture choice not already decided by repository invariants, destructive/canonical migration, production/third-party/new-credential need, work outside standing authorization, or actual tool/runtime exhaustion. The three HIGHs above are mechanically resolvable under existing committed architecture and therefore are immediate coding work, not maintainer-policy blockers.
