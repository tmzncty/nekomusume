# ChatGPT reviewer handoff — H-R9-029 source repaired; focused post-return proof still blocks R9-3

## Current repository truth

- Latest developer-owned source/test commit reviewed: exact `e9472fd56b12c080c1a84553a6a7e4007f8b9f70` (`fix(cli): post-return Carrier ACK three-way outcome (H-R9-029)`).
- New independent reviewer checkpoint: [`docs/reviews/reviewer-r9-h-r9-029-e9472fd-20260917.md`](reviews/reviewer-r9-h-r9-029-e9472fd-20260917.md).
- Prior H-R9-028 checkpoint remains applicable: [`docs/reviews/reviewer-r9-h-r9-028-bbb3e7f-20260916.md`](reviews/reviewer-r9-h-r9-028-bbb3e7f-20260916.md).
- Hosted Rust cross-evidence on exact `e9472fd`: GitHub Actions run `35124075507` completed SUCCESS. Hosted CI is cross-evidence only, not developer-local exact-tree provenance.
- Open PRs at review time: none.
- Final developer-local clean exact-tree provenance for the complete R9-2 source/test tree is still absent.
- `READY_LIVE: none`; release item 3 incomplete; item 4 incomplete; `RELEASE_CANDIDATE=false`; `PRODUCTION_READY=false`; `FREEZE=false`; `RELEASED=false`.

The external coding agent must synchronize to current `main` and continuously execute every dependency-ready slice below: implementation/review -> focused deterministic tests -> commit -> push -> next slice. Reviewer cadence is only a check frequency and is never a reason to idle.

## Accepted progress that must remain closed

Do not revert these exact-current repairs without contradictory exact-current evidence:

- H-R9-029 source shape at exact `e9472fd`: the post-return Carrier consumer now preserves the three recovery result classes. Positive current-packet evidence requires actual `acked_packets` to contain exact `post_pn_client`; typed rejection has its own event; accepted non-retiring feedback has a classification-only event. The dual-domain completion gate remains actual Session outstanding state plus Recovery `in_flight()==0`.
- H-R9-027 stale semantic ACKs are re-sealed under fresh authenticated envelopes; future ACKs use a non-empty range whose largest exceeds bounded `largest_sent`, so intended Recovery guards are reachable.
- H-R9-026 shared-owner + primary/settlement classification keeps real retirement, accepted-empty stale/duplicate, and typed Recovery rejection distinct.
- H-R9-025 post-return current-packet success remains identity-bound to actual `acked_packets` containing the exact post-return packet number.
- H-R9-024 cross-process client-send/server-ACK packet-number binding.
- H-R9-023 exact P3 malformed terminal oracle; H-R9-022 applied Carrier ACK position evidence; H-R9-021 malformed -> malformed -> Carrier ACK -> malformed source order.
- H-R9-020 count=4 final accounting: `2 reliable UDP + 1 uncertain TCP + 1 post-return reliable = 4`.
- reversed initial Session ACK handling buffers later ACKs until the Session watermark reaches them; no cumulative over-promotion.
- candidate A engine guard: future/never-sent largest is rejected before RTT/loss/PTO mutation.
- candidate B mixed queue/generic-drop observability repair remains accepted absent contradictory exact-current evidence.

# READY_LOCAL 1 — H-R9-029 HIGH acceptance closure: focused post-return stale/future process proof

The source-side three-way classification is now present. Do **not** redesign it merely because the old handoff described the pre-`e9472fd` code. The remaining blocker is dedicated built-binary evidence on the migration-back post-return path.

Using the existing authenticated receive owner, existing absolute deadline and existing malformed budget, add deterministic process regressions for:

### A. post-return accepted-empty stale/duplicate

After migration-back and after the exact post-return packet has been registered with Recovery:

1. inject exactly one fresh-envelope semantic stale/duplicate Carrier ACK while the operation remains active;
2. require exactly one `r9_udp_return_packet_ack_accepted_empty` (or exact equivalent) classification;
3. require zero post-return rejection event and no false positive retirement of `post_pn_client` attributable to the stale ACK;
4. later deliver the legitimate Carrier ACK for the exact current packet and exact Session DeliveryAck `stream=1 offset=48 len=16`;
5. require exactly one true current-packet retirement, final Recovery zero, one post-return settled event, and client/server success.

### B. post-return future/never-sent rejection

1. inject exactly one authenticated canonical ACK whose largest exceeds bounded `largest_sent`;
2. require exactly one typed post-return rejection and no false current-packet retirement;
3. later legitimate Carrier progress must still retire exact `post_pn_client`;
4. exact Session confirmation must still occur;
5. final Recovery zero, one post-return settled event, client/server success.

Reuse the existing H-R9-028 injection machinery where practical. Do not add a second receive owner, ACK architecture, deadline, malformed budget, capacity value, protocol field, or Session/Carrier/wire semantic.

**H-R9-029 source repair is accepted directionally but is not closed until these focused process cases are reachable and discriminating.**

# READY_LOCAL 2 — H-R9-028 HIGH: one-shot and discriminating initial stale/future regressions

The initial reliable-UDP stale/future seams reach the intended Recovery branches, but exact-current `probe.rs` still does not prove the required one-shot classifications strongly enough.

## A. accepted-empty stale/duplicate

- move/guard injection ownership so exactly one fresh-envelope semantic duplicate is injected for the bounded operation rather than implicitly once per ordinary packet-ACK emission;
- require client/server success;
- require exactly one accepted-empty classification, zero rejection, no extra positive Carrier retirement, unchanged Session logical-confirmation cardinality, final Recovery zero.

## B. future/never-sent rejection

- exactly one future injection;
- exactly one typed rejection;
- no false positive retirement attributable to it;
- subsequent legitimate Carrier ACK progress and final Recovery zero;
- client/server success and unchanged Session logical completion.

Preserve/reuse the engine-level atomic future-ACK regression. No new policy values.

`e9472fd` does not change decoder/parser/crypto framing semantics, so pinned decode fuzz is not newly required solely for READY_LOCAL 1-2. Focused deterministic tests are required; final local gate remains READY_LOCAL 7.

**R9-3 remains blocked until READY_LOCAL 1 and 2 are both closed.**

# READY_LOCAL 3 — positive P2 C1-C4 exact closure

Keep the existing count=4 fixture and strengthen only missing identity/cardinality/order proof.

- **C1 TCP replay identity:** exactly once client and server `stream=1`, `offset=32` (`seq=2` where emitted); explicitly reject replay evidence for offsets 0, 16, 48.
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
