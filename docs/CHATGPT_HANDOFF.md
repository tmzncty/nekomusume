# ChatGPT reviewer handoff — R9 final accounting HIGH at exact `ff295a7`; P2 exact evidence still READY_LOCAL

## Current repository truth

- Latest developer-owned source/test commit reviewed: exact `ff295a711bf526b1691625fc37f79aaf6ec83841` (`test(cli): P2 exact cardinality — TCP replay offset 32 + settled after both ACKs (M-R9-008 C1-C4)`).
- Reviewer checkpoint: [`docs/reviews/reviewer-r9-p2-accounting-ff295a7-20260916.md`](reviews/reviewer-r9-p2-accounting-ff295a7-20260916.md).
- `ff295a7` is **test-only** (`crates/neko-cli/tests/probe.rs`); no runtime source changed in that commit.
- GitHub-hosted `stable checks` SUCCESS and `nightly decode fuzz smoke` SUCCESS for exact `ff295a7`. Hosted CI is cross-evidence only; final R9-2 developer-local clean exact-tree provenance is still absent.
- Open PRs: none at this review.
- No new WAN/VPS experiment.
- `READY_LIVE: none`; item 3 incomplete; item 4 incomplete; `RELEASE_CANDIDATE=false`; `PRODUCTION_READY=false`; `FREEZE=false`; `RELEASED=false`.

The external coding agent must synchronize to current `main` and continue every dependency-ready slice below without waiting for reviewer cadence. Reviewer cadence is not a work-ticket length.

# STOP FRONT — H-R9-020 final failover accounting contradiction

**Severity: HIGH for correctness/evidence truth.** This is mechanically repairable under current semantics and requires no maintainer policy decision.

The current R9 reliable-UDP + migration-back runtime already computes the authoritative pre-TCP ownership partition:

```text
uncertain_end = records.len() - 1          // recovery_enabled: final record reserved
uncertain_start = 2                        // reliable UDP owns records 0 and 1
uncertain_count = uncertain_end - uncertain_start
```

For the positive count=4 / 16-byte P2 fixture the actual ownership is:

```text
offset 0   reliable UDP confirmed
offset 16  reliable UDP confirmed
offset 32  uncertain -> TCP replay -> confirmed
offset 48  reserved -> migration-back -> reliable UDP -> dual-domain confirmed
```

Therefore the truthful completed result is:

```text
udp_confirmed_records = 2
udp_confirmed_bytes   = 32
uncertain_records     = 1
uncertain_bytes       = 16
replayed_records      = 1
replayed_bytes        = 16
confirmed_records     = 4
confirmed_bytes       = 64
duplicate/lost/conflicting = 0 in this fixture
```

But current `failover_accounting` recomputes `uncertain_records = records.len() - 2` and final confirmed totals as only initial reliable records + TCP replay. The successful P2 therefore reports two uncertain records / 32 uncertain bytes and only three confirmed records / 48 confirmed bytes, even though offset 48 was already post-return confirmed and Recovery settled.

## Required smallest repair

Do not redesign Session/Carrier/ACK/wire semantics and do not invent policy values.

1. Reuse the already-authoritative `uncertain_count` for `uncertain_records` and `uncertain_bytes`.
2. Track post-return confirmation only after the existing dual-domain terminal condition actually succeeds (`post_outstanding` empty and Recovery `in_flight()==0`).
3. Compute final confirmed totals from actual completed ownership transitions, including the post-return record only after that terminal condition.
4. Tighten the existing count=4 built-binary P2 fixture to require the exact accounting values above.
5. Preserve existing settled-after-both-ACK-domain and `remaining_in_flight=0` evidence.

Smallest repair -> focused regression -> commit/push -> continue immediately.

# Accepted progress — preserve it

## H-R9-019 exact controlled TCP replay identity — CLOSED

Controlled fallback selects the exact ownership partition beginning at `uncertain_start`; do not reintroduce positional `skip(1)` replay selection.

## M-R9-008 dedicated post-return terminal settlement — ACCEPT_WITH_BOUNDARIES

`r9_udp_post_return_settled` is the terminal post-return success event and must only be emitted after logical confirmation and Carrier Recovery settlement, with authoritative `remaining_in_flight=0`.

## M-R9-009/010 causal order — ACCEPT_PARTIAL

The positive count=4 built-binary fixture proves:

- client: `udp_recovery_challenge_sent < udp_recovery_validated < udp_migrated_back`;
- server: `udp_recovery_owner_started < udp_recovery_validated < udp_return_delivery_ack_sent < udp_return_packet_ack_sent`.

Preserve these checks.

## Exact `ff295a7` additions — ACCEPT_PARTIAL

Preserve:

- exactly one client `tcp_delivery_ack_validated` event in the positive P2 fixture;
- `r9_udp_post_return_settled` observed strictly after both the Session DeliveryAck diagnostic and Carrier packet-ACK diagnostic.

These additions are useful but do **not** yet satisfy all C1-C4 identity/cardinality requirements below.

## Previously highlighted candidates A/B — CLOSED on current HEAD

- `Recovery::on_ack` rejects a largest ACK above `largest_sent` before RTT/loss/PTO mutation.
- `neko-observe::record_datagrams` keeps `queue_dropped` as a subset of generic `dropped` and preserves mixed terminal vs queue-full reasons.

Do not reopen without a new reproducer.

# READY_LOCAL after H-R9-020 — finish exact P2 in the existing count=4 fixture

Do not create a parallel P2 scenario. Do not change protocol/Session/Carrier/ACK architecture. Tighten `reliable_udp_migration_back_reserves_final_record` in place.

## P2-C1 — exact TCP replay identity/count

Current `ff295a7` counts one client `tcp_delivery_ack_validated` but does not prove its exact identity.

Require:

- exactly one client `tcp_delivery_ack_validated` with `seq=2` (stream 1 / offset 32);
- exactly one server `tcp_delivery_ack_sent` for `seq=2`;
- no TCP replay/ACK evidence for reliable-owned seq 0/1 or reserved seq 3.

Use existing seq diagnostics. Add stream/offset fields only if exact identity cannot otherwise be shown.

## P2-C2 — exact client migration -> post-return send

Require strict order:

```text
udp_recovery_challenge_sent
  < udp_recovery_validated
  < udp_migrated_back
  < r9_udp_post_return_sent(offset=48)
```

Require exactly one post-return send for offset 48. The reserved record must never appear as legacy `udp_uncertain_range_sent` offset 48.

## P2-C3 — exact server post-return identity/cardinality

Require exactly once each:

1. `udp_recovery_owner_started`;
2. `udp_recovery_validated`;
3. `udp_return_delivery_ack_sent` with `seq=3` / reserved offset 48;
4. `udp_return_packet_ack_sent` for the same bounded post-return owner.

Preserve strict order `owner_started < validated < delivery_ack_sent < packet_ack_sent`.

## P2-C4 — exact client logical + Carrier settlement

Runtime diagnostic `udp_return_delivery_ack_validated` currently has no stream/offset identity; its generic diagnostic seq is not a mechanically sufficient logical-range proof. Add only the smallest fields required to prove stream 1 / offset 48; do not change Session semantics.

Then require exactly once each:

- `udp_return_delivery_ack_validated` for stream 1 / offset 48;
- `r9_udp_return_packet_ack` with `applied=true`;
- `r9_udp_post_return_settled` for stream 1 / offset 48 / `remaining_in_flight=0`.

`r9_udp_post_return_settled` must be after **both** acknowledgement-domain events, regardless of arrival order. No success event may precede both-domain completion.

# R9-2 continuation — execute without waiting

After H-R9-020 and exact P2 C1-C4, continue all dependency-ready slices below immediately.

## R9-2I-E — post-return acknowledgement-order seam

The same bounded post-return receive owner must succeed in both authenticated arrival orders:

- Session DeliveryAck -> Carrier packet ACK;
- Carrier packet ACK -> Session DeliveryAck.

Use one bounded test-only ordering seam. Both runs must reach the same exact `r9_udp_post_return_settled` terminal event. No sleep-based ordering, duplicate ACK implementation, second receive owner or fresh per-order deadline.

## R9-2I-A — automatic-health replay exact-identity regression

Current automatic-health path still obtains exact `FailoverController::tcp_resend()` ownership, reduces it to `.len()`, then reconstructs a contiguous replay slice from the local record list. Challenge this explicitly.

Add a focused regression comparing the actual replay `(DataId, payload)` identities against the exact controller-owned resend set.

- If current contiguous partitioning is exactly equivalent for all admitted shapes under current semantics, retain the regression as bounded no-finding evidence.
- If a mismatch is reproduced, consume exact controller ownership rather than cardinality.

Do not redesign failover architecture.

## P3 — persistent malformed budget across valid Carrier feedback

Within one reliable receive/settlement operation drive exactly:

```text
malformed #1 -> malformed #2 -> canonical Carrier ACK -> malformed #3
```

Malformed #3 must hit the existing `MAX_POST_HANDSHAKE_MALFORMED`; the valid Carrier ACK must not reset the operation-wide malformed counter. Termination must be typed, finite and non-spinning. Do not change the numeric limit or create a second settlement budget.

## P4 — incomplete settlement remains terminal

Suppress one acknowledgement domain at a time for the post-migration reserved owner and require:

- nonzero/typed terminal result;
- no `r9_udp_in_flight_settled`, `r9_udp_post_return_settled`, or equivalent success while Recovery remains in flight or exact logical confirmation remains outstanding;
- no downstream health/failover continuation from an unproven settled state;
- no fresh deadline after the operation-wide deadline expires.

## R9-2 closure gate

After H-R9-020 + exact P2 + both ACK arrival orders + automatic exact-identity regression + P3/P4 all exist on one reachable source/test SHA, persist developer-local clean exact-tree provenance:

```text
PYTHONDONTWRITEBYTECODE=1 bash scripts/check.sh
git diff --check
```

Record exact pushed SHA, UTC start/end, Linux OS/arch, Rust stable version, command exit codes and clean initial/final tree. Run pinned decoder fuzz commands only if decoder/parser/crypto framing changed. Hosted CI remains additional cross-evidence.

Then continue immediately without waiting for reviewer cadence.

# Continuous queue — preserve depth

1. **R9-3 Data-loss recovery:** suppress reliable-owned Data post-admission; cwnd admission before suppression; PTO after deadline; fresh packet number/nonce retransmit with stable Session/frame identity; exactly-once Session delivery; independent ACK domains; final Recovery zero or typed bounded failure.
2. **R9-4 ACK-loss + delayed original/reorder:** suppress Carrier ACK, retransmit, release delayed original; one logical delivery; Session dedup; fresh packet numbers/nonces; truthful loss/PTO; settled recovery.
3. **R9-5 tamper/future-ACK/malformed negatives:** tamper mutates neither recovery nor Session; future/never-sent ACK atomic rejection; stale/duplicate feedback fabricates no evidence; malformed finite/panic-free.
4. **R9-6 pacing/cwnd/plaintext ownership:** every send/retransmit consults congestion admission; refusal commits no logical/recovery ownership; one bounded retransmit plaintext owner; teardown releases state.
5. **R9-7 process/result truth:** Data, Carrier ACK, Session ACK, PTO/retransmit, recovery, Session delivery, malformed budget and final outcome remain distinct and are emitted only after the claimed transition.
6. **R9-8 authenticated warm TCP standby:** negotiation + Noise trust/authz + resume/readiness binding + resource admission; no application Data before promotion.
7. **R9-9 resolved UDP health -> hysteresis -> real TCP promotion:** resolved packet outcomes drive health; recoverable loss stays UDP; PTO-only samples cannot erase later loss; only ready TCP can promote.
8. **R9-10 uncertain Session replay UDP -> TCP:** replay exact genuine uncertain Session ranges over promoted TCP; draining UDP gets no new Data; exact-once Session dedup; no TCP packet ACK.
9. **R9-11 timeout/shutdown/cleanup matrix.**
10. **R9-12 coherent exact-tree gate + dedicated independent bounded review of the new cross-process reliable-UDP integration surface.** Any BLOCKER/HIGH returns to smallest repair + regression + re-gate.
11. **Q10 observability reconciliation** using existing surfaces only.
12. **Q11 factual status/release reconciliation.** Only independently reviewed cross-process R9 + real TCP promotion may create a specific `READY_LIVE` row.
13. **Q12 one changed-hypothesis self-owned VPS run** only after Q11 creates that row; preserve negative evidence and cleanup; no unchanged retry.

# Core-surface review inventory

The earlier item-4 sweep already challenged the pre-R9 reliable engine, CarrierState/CarrierManager, scheduler/flow accounting, adapters, SessionRuntime, observability, package/reproducibility, dependency/build, CLI portability/output, algorithmic boundedness/validators, wire/parser and PLPMTUD/FEC/disabled-gate candidates.

The new cross-process R9 integration surface is not covered by those historical notes and still requires its own dedicated bounded independent review at R9-12. Item 4 therefore remains incomplete.

# VPS opportunity

**Not READY — correctness/evidence + independent-review dependency.** Rental-window priority is acknowledged, but a machine-readable accounting contradiction exists on the exact candidate path and P2/R9-2 evidence is not yet closed. Do not run a known-invalid candidate merely to consume rental time.

Unlock sequence:

```text
H-R9-020 -> complete R9-2 -> R9-3..R9-12 -> Q10/Q11 -> specific READY_LIVE row -> Q12
```

If Q11 creates a changed-hypothesis `READY_LIVE` question, standing authorization already covers the bounded self-owned TCP/UDP VPS run. Permission is not the blocker.

# Separate non-blocking policy gates

`SessionRuntime.events` retention; D019 source retention/no-reset; RSEC-001 adversarial-load/capacity suitability; signing/key custody/SBOM/publication trust; previous frozen-release interoperability; final RC/freeze/release/production authority.

Do not invent policy values while working R9, and do not let these independent gates starve dependency-ready R9 implementation/review work.
