# ChatGPT reviewer handoff — H-R9-032 keeps R9-3 blocked on exact post-return packet binding/order proof

## Current repository truth

- Latest developer-owned source/test commit reviewed: exact `89c436ce6d18912579d67f81de71845110b28236` (`test(cli): H-R9-031 exact post-return oracles — client/server success + cardinality + packet bind`). This commit changes tests only.
- New independent reviewer checkpoint: [`docs/reviews/reviewer-r9-h-r9-032-89c436c-20260917.md`](reviews/reviewer-r9-h-r9-032-89c436c-20260917.md), reachable through reviewer commit `333311079fd7141a4784d5c4c8e3a31d643a04cc`.
- H-R9-031 is partially closed at exact `89c436c`: both post-return fixtures now require client/server success; stale requires exactly one accepted-empty, zero rejection, exactly one true positive Carrier retirement, exact Session `stream=1 offset=48 len=16`, and exactly one settlement carrying `remaining_in_flight=0`; future requires exactly one typed rejection and exactly one positive retirement.
- H-R9-030 source seam remains narrowly closed: post-return ACK obligation is polled once, canonical ACK plaintext retained, stale mode sends real ACK then one re-sealed semantic duplicate, future mode injects never-sent ACK before the real ACK, and ordinary no-seam P2 ordering remains unchanged.
- H-R9-029 three-way client classification remains accepted in source: real retirement, accepted-empty stale/duplicate, and typed rejection are distinct; only actual `acked_packets` containing exact `post_pn_client` may produce positive retirement.
- Hosted Rust cross-evidence on exact `89c436c`: GitHub Actions run `35142674605` completed SUCCESS; both `stable checks` (`bash scripts/check.sh`) and `nightly decode fuzz smoke` succeeded. Hosted CI is cross-evidence only, not developer-local exact-tree provenance.
- Open PRs at review time: none.
- Final developer-local clean exact-tree provenance for the complete R9-2 source/test tree is still absent.
- `READY_LIVE: none`; release item 3 incomplete; item 4 incomplete; `RELEASE_CANDIDATE=false`; `PRODUCTION_READY=false`; `FREEZE=false`; `RELEASED=false`.

The external coding agent must synchronize to current `main` and continuously execute every dependency-ready slice below: implementation/review -> focused deterministic tests -> commit -> push -> next slice. Reviewer cadence is only a check frequency and is never a reason to idle.

## Accepted progress that must remain closed

Do not revert these exact-current repairs without contradictory exact-current evidence:

- H-R9-031 partial process-oracle strengthening listed above.
- H-R9-030 ACK-obligation source repair described above.
- H-R9-029 post-return Carrier feedback preserves three classes and only real `acked_packets` containing exact `post_pn_client` may produce positive retirement.
- H-R9-027 stale semantic ACKs may be re-sealed under fresh authenticated envelopes; future ACKs use a non-empty range whose largest exceeds bounded `largest_sent`.
- H-R9-026 shared-owner/settlement classification keeps real retirement, accepted-empty, and typed Recovery rejection distinct.
- H-R9-025 exact current-packet retirement identity; H-R9-024 positive-P2 cross-process client-send/server-ACK packet-number binding.
- H-R9-023 exact P3 malformed terminal oracle; H-R9-022 applied Carrier ACK position evidence; H-R9-021 malformed -> malformed -> Carrier ACK -> malformed order.
- H-R9-020 count=4 accounting: `2 reliable UDP + 1 uncertain TCP + 1 post-return reliable = 4`.
- reversed initial Session ACK handling buffers later ACKs until the Session watermark reaches them; no cumulative over-promotion.
- candidate A engine guard: future/never-sent largest is rejected before RTT/loss/PTO mutation.
- candidate B mixed queue/generic-drop observability repair remains accepted absent contradictory exact-current evidence.

# READY_LOCAL 1 — H-R9-032 HIGH: finish exact post-return stale/future packet binding and ordering proof

Exact `89c436c` strengthened process success/cardinality, but it did **not** implement the packet-number cross-bind claimed by its commit title and does not prove actual event ordering.

Smallest repair: tests/assertions only unless stronger proof exposes a concrete runtime contradiction.

For both post-return stale and future fixtures:

1. collect exactly one client `r9_udp_post_return_sent` and extract `packet_number`;
2. collect exactly one server `udp_return_packet_ack_sent` and extract `packet_number`;
3. collect exactly one positive client `r9_udp_return_packet_ack` with `retired=true` and extract `packet_number`;
4. require all three packet numbers to be equal; reuse the existing positive P2 parsing/binding pattern rather than inventing a new runtime diagnostic;
5. require exactly one Session transition `r9_udp_return_delivery_ack` with `stream=1 offset=48 len=16`;
6. require exactly one `r9_udp_post_return_settled` with `remaining_in_flight=0`;
7. compare actual client log positions and require settlement strictly after both the exact Session transition and the true positive Carrier retirement.

Classification-specific closure:

- **stale:** exactly one `r9_udp_return_packet_ack_accepted_empty`, zero post-return rejection;
- **future:** exactly one `r9_udp_return_packet_ack_rejected`, zero accepted-empty; the injected future feedback must not manufacture the positive retirement.

No new protocol owner, timeout, malformed budget, policy value or architecture. No decoder/parser/framing semantic change is implied; do not mechanically fuzz only for assertion strengthening.

**R9-3 remains blocked until READY_LOCAL 1 and READY_LOCAL 2 are closed.**

# READY_LOCAL 2 — H-R9-028 HIGH: one-shot discriminating initial stale/future regressions

The initial reliable-UDP injection machinery is separate from the post-return seam and still injects from the ordinary packet-ACK emission path.

## A. initial accepted-empty stale/duplicate

- inject exactly one fresh-envelope semantic duplicate for the bounded operation, not once per ordinary packet-ACK emission;
- require client/server success;
- require exactly one accepted-empty classification, zero rejection, no extra positive Carrier retirement, unchanged Session logical-confirmation cardinality, final Recovery zero.

## B. initial future/never-sent rejection

- exactly one future injection;
- exactly one typed rejection;
- no false positive retirement attributable to it;
- subsequent legitimate Carrier ACK progress and final Recovery zero;
- client/server success and unchanged Session logical completion.

Preserve/reuse the engine-level atomic future-ACK regression. No new policy values.

# READY_LOCAL 3 — positive P2 C1-C4 exact closure

Keep the existing count=4 fixture and strengthen only missing identity/cardinality/order proof.

- **C1 TCP replay identity:** exactly once client and server `stream=1 offset=32` (`seq=2` where emitted); explicitly reject replay evidence for offsets 0, 16, 48.
- **C2 migration ownership chain:** exactly once and strict order `udp_recovery_challenge_sent < udp_recovery_validated < udp_migrated_back < r9_udp_post_return_sent(stream=1,offset=48,packet_number=<pn>)`; offset 48 absent from uncertain/pre-promotion/TCP replay ownership.
- **C3 server dual-domain:** exactly one Session DeliveryAck `stream=1 offset=48 len=16` plus exactly one Carrier ACK for the exact client-owned post-return packet number; do not merge domains.
- **C4 client actual transitions:** exactly one Session transition `stream=1 offset=48 len=16`; exactly one positive Carrier retirement for the cross-bound packet number; zero false/rejected shortcut; exactly one `r9_udp_post_return_settled ... remaining_in_flight=0`; settlement strictly after both actual transitions.

# READY_LOCAL 4 — post-return ACK arrival-order challenge

Through the same authenticated receive owner, same deadline and same malformed budget, deterministically cover both:

- Session DeliveryAck -> Carrier packet ACK;
- Carrier packet ACK -> Session DeliveryAck.

Both must converge to one logical confirmation, one exact packet retirement, no false/rejected shortcut, and Recovery zero. A test seam may reorder already-produced authenticated ACK datagrams only; no new protocol owner or policy value.

# READY_LOCAL 5 — P4 Session ACK present / Carrier ACK withheld

Require positive exact Session transition, typed bounded nonzero terminal failure because Recovery remains unsettled, no settled marker, and no downstream health/failover/migration success continuation.

# READY_LOCAL 6 — P4 Carrier ACK present / Session ACK withheld

Require positive exact Carrier retirement, typed bounded nonzero terminal failure because logical confirmation remains outstanding, no settled marker, and no downstream success continuation.

# READY_LOCAL 7 — R9-2 developer-local exact-tree provenance

On the final pushed R9-2 source/test SHA in a clean safe checkout/worktree run at least:

```text
PYTHONDONTWRITEBYTECODE=1 bash scripts/check.sh
git diff --check
```

Record exact reachable pushed SHA, UTC start/end, Linux OS/arch, Rust stable version, both exit codes, and clean initial/final tree. Run pinned decoder fuzz only if decoder/parser/crypto framing actually changed. Hosted CI remains separate cross-evidence.

Then continue immediately; do not wait for reviewer cadence.

# READY_LOCAL 8 — R9-3 Data-loss recovery

Suppress one reliable-owned Data only after congestion admission; suppression must not bypass ownership accounting. PTO only after its deadline; retransmit under a fresh packet number/nonce while retaining stable frame/logical identity; exactly-once Session delivery; independent ACK domains; final Recovery zero or typed bounded failure.

# READY_LOCAL 9 — R9-4 ACK-loss + delayed original/reorder

Suppress one Carrier ACK, force legitimate retransmission, then release the delayed original. Require one logical delivery, Session dedup authority, fresh packet numbers/nonces, truthful loss/PTO evidence, and settled Recovery.

# READY_LOCAL 10 — R9-5 adversarial feedback

Future/never-sent/stale/duplicate/tampered feedback must be atomic and fail closed. Rejected feedback cannot mutate RTT/PTO/loss/cwnd/Session state; accepted-empty cannot be mislabeled as transition or rejection. Re-check every current `UdpAcknowledgement::Carrier` caller, including initial, settlement and post-return diagnostics.

# READY_LOCAL 11 — R9-6 ownership/resource boundedness

Every first send/retransmit must consult congestion admission before ownership commit; refusal commits no recovery/logical state. Keep one bounded retransmit plaintext owner and deterministic teardown. Do not invent capacity values.

# READY_LOCAL 12 — R9-7 process/result truth

Data, Carrier ACK, Session DeliveryAck, PTO/retransmit, Recovery, Session delivery, malformed budget, accepted-empty feedback, rejected feedback and terminal result remain distinct. Emit each event only after the exact claimed state transition.

# READY_LOCAL 13 — R9-8 warm TCP readiness

Authenticated resume-bound warm standby carries no application Data before promotion. Readiness/resource admission remains distinct from delivery evidence.

# READY_LOCAL 14 — R9-9/R9-10 health, promotion, uncertain replay and cleanup

Resolved UDP outcomes feed existing health/hysteresis; recoverable loss remains on UDP; promotion only after existing readiness/health gates. Replay only genuine uncertain Session ranges over promoted TCP; draining/failed UDP gets no new Data; Session dedup exact-once; TCP gets no duplicate packet-ACK layer; cover shutdown/cleanup negatives.

# READY_LOCAL 15 — R9-11/R9-12 + Q10/Q11/Q12 coherent closure

Run the clean exact-tree local gate for the complete cross-process reliable-UDP/failover slice, then perform a dedicated independent bounded challenge of materially new R9 send/admission/recovery/demux/Session ACK/Carrier ACK/failover/migration/cleanup/diagnostic surface. A no-finding review is valid item-4 support; any BLOCKER/HIGH returns to smallest repair + regression + exact-tree gate. Only after a reachable independent review anchor may status/release packet be factually reconciled. Do not flip release-authority flags.

## Item-4 / core-surface inventory

Reachable independent bounded review already exists for the earlier reliable-UDP engine basics, CarrierState, Carrier Manager/health/migration-back, FairScheduler/flow accounting, carrier adapters, SessionRuntime, observability, package/reproducibility, dependency/build, CLI portability/output, algorithmic boundedness/validators, DeliveryLedger/process codec/datagram/crypto API, and wire/parser surfaces.

The materially new cross-process R9 integration is not yet independently closed. Queue exhaustion is false.

## VPS opportunity

**Not READY.** Standing authorization remains valid, but authoritative classification remains `READY_LIVE: none`. Current blocker class is local R9 correctness/evidence plus later independent cross-process R9 review. Do not repeat HY2, warm failover, periodic/soak, package lifecycle, migration-back, endpoint/key migration, IPv6, PLPMTUD or Experimental Track evidence merely because the VPS remains rented.

Only create a new `READY_LIVE` row if later code/instrumentation/hypothesis/path conditions create a concrete unresolved real-network question.

## Separate non-blocking policy / authority gates

Do not invent or change while executing the queue above:

- `SessionRuntime.events` retention/capacity policy;
- D019 source-retention/no-reset policy;
- RSEC-001 adversarial-load/capacity suitability conditions;
- signing/key custody/SBOM/publication trust;
- previous frozen-release interoperability policy;
- core Session/Carrier/ACK/crypto/wire architecture;
- destructive/canonical-meaning migration;
- final RC/freeze/release/production authority.

These gates do not justify idling dependency-ready local R9 work.
