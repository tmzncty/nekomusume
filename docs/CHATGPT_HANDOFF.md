# ChatGPT reviewer handoff — exact P2 order accepted; identity/cardinality closure still READY_LOCAL

## Current repository truth

- Latest developer-owned source/test commit reviewed: exact `1aa95abddfc23d64d377fa65172ef8cf740ff987` (`test(cli): P2 exact causal-order evidence for server + client recovery chain (M-R9-009, M-R9-010)`).
- Reviewer checkpoint: `docs/reviews/reviewer-r9-p2-order-1aa95ab-20260916.md`.
- `1aa95ab` is **test-only** (`crates/neko-cli/tests/probe.rs`); no runtime source changed.
- GitHub-hosted `stable checks` SUCCESS and `nightly decode fuzz smoke` SUCCESS for exact `1aa95ab`. Hosted CI is cross-evidence only; final R9-2 developer-local clean exact-tree provenance is still absent.
- Open PRs: none at this review.
- No new WAN/VPS experiment.
- `READY_LIVE: none`; item 3 incomplete; item 4 incomplete; `RELEASE_CANDIDATE=false`; `PRODUCTION_READY=false`; `FREEZE=false`; `RELEASED=false`.

The external coding agent must synchronize to current `main` and continue every dependency-ready slice below without waiting for reviewer cadence. Reviewer cadence is not a work-ticket length.

## Accepted progress — preserve it

### H-R9-019 exact controlled TCP replay identity — CLOSED

Controlled fallback selects the exact ownership partition beginning at `uncertain_start`; do not reintroduce positional `skip(1)` replay selection.

### M-R9-008 dedicated post-return terminal settlement — ACCEPT_WITH_BOUNDARIES

`r9_udp_post_return_settled` remains the terminal post-return success event and must only be emitted after logical confirmation and Carrier Recovery settlement, with authoritative `remaining_in_flight=0`.

### M-R9-009/010 causal-order assertions — ACCEPT_PARTIAL

Exact `1aa95ab` improves the positive count=4 built-binary fixture by proving observed order for:

- client: `udp_recovery_challenge_sent < udp_recovery_validated < udp_migrated_back`;
- server: `udp_recovery_owner_started < udp_recovery_validated < udp_return_delivery_ack_sent < udp_return_packet_ack_sent`.

Do not revert these order checks. They do **not** yet close exact P2 identity/cardinality below.

### Previously highlighted candidates A/B — CLOSED on current HEAD

- `Recovery::on_ack` rejects a largest ACK above `largest_sent` before RTT/loss/PTO mutation.
- `neko-observe::record_datagrams` keeps `queue_dropped` as a subset of generic `dropped` and preserves mixed terminal vs queue-full reasons.

Do not reopen without a new reproducer.

# READY_LOCAL front — finish exact P2 in the existing count=4 fixture

Do not create a parallel P2 scenario. Do not change protocol/Session/Carrier/ACK architecture. Tighten `reliable_udp_migration_back_reserves_final_record` until one built-binary run proves the complete ownership chain.

## P2-C1 — exact TCP replay identity/count

Require exactly one client `tcp_delivery_ack_validated` and exactly one server `tcp_delivery_ack_sent` for `seq=2` (stream 1 / offset 32 in this fixed fixture). Reject replay evidence for `seq=0`, `seq=1`, and reserved `seq=3`.

Use the existing `seq` diagnostics; add stream/offset fields only if exact identity cannot otherwise be shown. Do not create a new replay protocol or second ownership table.

## P2-C2 — exact client migration -> post-return send

Preserve the new challenge/validation/migration order and additionally require:

```text
udp_recovery_challenge_sent
  < udp_recovery_validated
  < udp_migrated_back
  < r9_udp_post_return_sent(seq=0, offset=48)
```

`r9_udp_post_return_sent` must occur exactly once for offset 48. The reserved record must still never appear as legacy `udp_uncertain_range_sent` offset 48.

## P2-C3 — exact server post-return identity/cardinality

Preserve the new server event order and require:

1. exactly one `udp_recovery_owner_started`;
2. exactly one `udp_recovery_validated`;
3. exactly one `udp_return_delivery_ack_sent` with `seq=3` (reserved offset 48);
4. exactly one `udp_return_packet_ack_sent` for the same bounded post-return owner;
5. strict order `owner_started < validated < delivery_ack_sent < packet_ack_sent`.

The Session-ACK diagnostic already derives `seq` from `post_offset / bytes`; assert it. For the Carrier ACK, use uniqueness + same-branch strict order first; add only the smallest contextual field if required for a mechanically unambiguous association.

## P2-C4 — exact client logical + Carrier settlement

Current `udp_return_delivery_ack_validated` carries no explicit stream/offset. Add only the smallest diagnostic fields needed to prove that the logical confirmation is stream 1 / offset 48; do not change Session semantics.

Then require exactly once each:

- `udp_return_delivery_ack_validated` for stream 1 / offset 48;
- `r9_udp_return_packet_ack` with `applied=true`;
- `r9_udp_post_return_settled` for stream 1 / offset 48 / `remaining_in_flight=0`.

The settled event must occur strictly after **both** acknowledgement-domain events, regardless of which ACK arrived first. No success/settled event may precede completion of both domains.

Once P2-C1..C4 are green, positive P2 causal evidence is closed. Continue immediately.

# R9-2 continuation — execute without waiting

## R9-2I-E — post-return acknowledgement-order seam

The same bounded post-return receive owner must succeed in both authenticated arrival orders:

- Session DeliveryAck -> Carrier packet ACK;
- Carrier packet ACK -> Session DeliveryAck.

Use one bounded test-only ordering seam. Both runs must reach the same exact `r9_udp_post_return_settled` terminal event. No sleep-based ordering, duplicate ACK implementation, second receive owner or fresh per-order deadline.

## R9-2I-A — automatic-health replay exact-identity regression

The automatic-health path historically reduced exact `FailoverController::tcp_resend()` ownership to cardinality and reconstructed a contiguous replay slice. Add a focused regression comparing actual replay `(DataId, payload)` identities against exact controller ownership.

- If current contiguous partitioning is exactly equivalent, retain the regression as bounded no-finding evidence.
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

After exact P2 + both ACK arrival orders + automatic exact-identity regression + P3/P4 all exist on one reachable source/test SHA, persist developer-local clean exact-tree provenance:

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

**Not READY — implementation/evidence + independent-review dependency.** Rental-window priority is acknowledged, but the current candidate does not yet provide a truthful new WAN question.

Unlock sequence:

```text
complete R9-2 -> R9-3..R9-12 -> Q10/Q11 -> specific READY_LIVE row -> Q12
```

If Q11 creates a changed-hypothesis `READY_LIVE` question, standing authorization already covers the bounded self-owned TCP/UDP VPS run. Permission is not the blocker.

# Separate non-blocking policy gates

`SessionRuntime.events` retention; D019 source retention/no-reset; RSEC-001 adversarial-load/capacity suitability; signing/key custody/SBOM/publication trust; previous frozen-release interoperability; final RC/freeze/release/production authority.

Do not invent policy values while working R9, and do not let these independent gates starve dependency-ready R9 implementation/review work.
