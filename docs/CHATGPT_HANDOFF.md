# ChatGPT reviewer handoff — H-R9-028 closed at 206b4b9; R9-3 blocked on P2 C1-C4 closure + ACK order challenge

## Current repository truth

- Latest developer-owned source/test commit reviewed: exact `206b4b936b1401c45bc0a23c730032f91eab8f0d` (`fix(cli): H-R9-028 one-shot initial stale/future ACK injection + discriminating oracles`). Source + tests.
- H-R9-028 is closed at exact `206b4b9`: the initial-path `--send-stale-ack`/`--send-future-ack` seams now inject exactly once per bounded operation (one-shot guards `stale_ack_sent`/`future_ack_sent`); both initial fixtures require client/server success, exactly one accepted-empty or one typed rejection, exactly two applied Carrier ACKs (records 0/1), exactly two `r9_udp_delivery_ack_validated` (offsets 0/16), and `r9_udp_in_flight_settled` `remaining_in_flight=0`.
- H-R9-032 exact packet-number cross-bind and settlement-order proof remains closed at `ddafbb1`.
- H-R9-031 partial process-oracle strengthening remains accepted at `89c436c`.
- H-R9-030 source seam remains narrowly closed.
- H-R9-029 three-way client classification remains accepted in source.
- Hosted Rust cross-evidence on exact `89c436c`: GitHub Actions run `35142674605` completed SUCCESS; hosted CI is cross-evidence only.
- Developer-local clean exact-tree provenance for `206b4b9`: `PYTHONDONTWRITEBYTECODE=1 bash scripts/check.sh` exit 0, `git diff --check` exit 0, clean worktree at pushed SHA, 2026-09-16T21:17:43Z → 2026-09-16T21:20:42Z, Linux x86_64, rustc 1.98.0 (88d9e12ae 2026-08-18).
- Open PRs at review time: none.
- `READY_LIVE: none`; release item 3 incomplete; item 4 incomplete; `RELEASE_CANDIDATE=false`; `PRODUCTION_READY=false`; `FREEZE=false`; `RELEASED=false`.

The external coding agent must synchronize to current `main` and continuously execute every dependency-ready slice below: implementation/review -> focused deterministic tests -> commit -> push -> next slice. Reviewer cadence is only a check frequency and is never a reason to idle.

## Accepted progress that must remain closed

Do not revert these exact-current repairs without contradictory exact-current evidence:

- H-R9-028 one-shot initial stale/future injection + discriminating oracles at `206b4b9`.
- H-R9-032 exact packet-number cross-bind and settlement-order proof at `ddafbb1`.
- H-R9-031 partial process-oracle strengthening at `89c436c`.
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

# READY_LOCAL 1 — CLOSED at ddafbb1

H-R9-032 exact post-return packet-number cross-bind and settlement-order proof is complete.

# READY_LOCAL 2 — CLOSED at 206b4b9

H-R9-028 one-shot initial stale/future injection machinery and discriminating oracles are complete. Both fixtures now satisfy the full acceptance contract.

**R9-3 remains blocked until READY_LOCAL 3+ are closed.**

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
