# ChatGPT reviewer handoff — `67a5717` improves P1 but R9-2H evidence gate remains open; continue P1 -> P4 -> deep R9 queue

## Current repository truth

- Latest developer source/test SHA reviewed: exact `67a57173a5c904f138cf0d920d5a71d426d50bc5` (`test(cli): P1 reversed-ACK exact structured evidence`).
- Independent bounded recheck: `docs/reviews/r9-2h-p1-evidence-recheck-67a5717-20260915.md` (reviewer commit `052c492`).
- Hosted GitHub checks on exact `67a5717`: `stable checks` success; `nightly decode fuzz smoke` success. Hosted checks are supplementary only; final R9-2H developer-local exact-tree provenance is still absent.
- Open PRs: none. No new WAN/VPS experiment. `READY_LIVE: none` remains authoritative.
- Governance unchanged: item 3 incomplete; item 4 incomplete; `RELEASE_CANDIDATE=false`; `PRODUCTION_READY=false`; `FREEZE=false`; `RELEASED=false`.

The coding agent is explicitly pre-authorized to finish the remaining P1 evidence assertions, then P2-P4 + exact-tree provenance, and immediately continue through R9-3..R9-12 and Q10/Q11/Q12 without waiting for reviewer cadence. Reviewer cadence is not a work-ticket boundary.

## Accepted implementation/source progress

Do not reopen these source findings unless new process evidence contradicts them:

- H-R9-011: out-of-order later Session DeliveryAck is buffered and cannot cumulatively confirm earlier unconfirmed bytes.
- H-R9-012/H-R9-013/H-R9-014: applied Session ACK evidence follows actual mutation order; direct and buffered applied evidence carries exact `stream + offset`, buffered application marks `buffered=true`; logical `outstanding + pending_acks` must retire before Carrier-only settlement.
- one operation-wide malformed counter and one absolute application deadline; Carrier packet ACK remains Carrier-local; incomplete Recovery settlement is terminal and cannot feed downstream health/failover as success.
- `940b17f` + `34ae142`: the reserved final record after successful migration-back is owned by the existing reliable-UDP runtime; server creates Carrier ACK obligation only after authenticated/opened/decoded/correct-Session accepted Data; client completion requires both exact Session confirmation and Recovery `in_flight()==0`; Session DeliveryAck and Carrier ACK remain separate evidence domains and either order is allowed.

# R9-2H evidence front — READY_LOCAL, execute continuously

## P1 — exact reversed-order built-process evidence — PARTIAL at `67a5717`

`67a5717` now requires exactly one buffered event (`offset=16`, `watermark=0`), forbids the covered shortcut, preserves exactly two applied confirmation identities, and ties final Recovery settlement to the structured `r9_udp_in_flight_settled` event with `remaining_in_flight=0`.

Finish the same built-binary test in one small test-only slice:

1. require `out.status.success()` for the client process;
2. tie the order proof to the exact structured event lines already collected — buffered offset 16 first, then applied offset 0, then applied buffered offset 16 — not generic `find("offset")`/`rfind("buffered")` substrings from unrelated diagnostics;
3. prove the existing pre-settlement boundary (`outstanding + pending_acks` empty before Carrier-only settlement) using the existing structured evidence/invariant boundary; do not add a new framework.

Do not change protocol semantics for P1.

## P2 — exact reserved post-migration ownership + dual settlement

Strengthen `reliable_udp_migration_back_reserves_final_record` on real built binaries. For exact reserved fixture record `stream=1, offset=32`, require successful completion and exact evidence that:

- no pre-promotion Recovery ownership / legacy uncertain owner exists for offset 32;
- `udp_migrated_back` precedes exactly one post-return reliable send;
- server accepts the authenticated post-return Data and produces one independent Session DeliveryAck plus one independent Carrier packet ACK obligation;
- client applies the exact Session confirmation and Carrier ACK separately;
- post-return Recovery reaches `in_flight=0` before success;
- both Carrier-ACK-first and Session-ACK-first ordering are accepted.

Evidence-only structured events are allowed if needed for unambiguous assertions; do not invent new protocol semantics.

## P3 — persistent malformed budget across valid Carrier feedback

Drive one reliable receive/settlement operation with authenticated sequence:

```text
malformed #1
malformed #2
canonical Carrier ACK
malformed #3
```

Malformed #3 must hit the existing `MAX_POST_HANDSHAKE_MALFORMED` ceiling. Valid Carrier feedback must not reset the operation-wide malformed counter. Termination must be typed, finite and non-spinning. A bounded test-only server seam is allowed; do not change the numeric limit.

## P4 — preserve incomplete settlement terminality, including post-return

Keep the existing incomplete-settlement negative: remaining in-flight != 0 is terminal/nonzero, emits incomplete, never emits `r9_udp_in_flight_settled`, and cannot feed downstream health/failover from a false premise.

Extend the same truth boundary to the post-migration reserved-record owner by suppressing/missing one of the two independent evidence domains; require nonzero terminal outcome, no post-return settled/success claim, and no downstream continuation based on incomplete ownership.

## R9-2H exact-tree gate

After P1-P4 land on one reachable source/test SHA, persist developer-local clean exact-tree provenance from a safe worktree:

```text
PYTHONDONTWRITEBYTECODE=1 bash scripts/check.sh
git diff --check
```

Record exact pushed SHA, UTC start/end, Linux OS/arch, Rust stable version, both exit codes, and clean initial/final tree. No decoder/parser/crypto-framing change is required by the current evidence-only front; do not invent a new fuzz obligation.

Then continue immediately to R9-3.

# Continuous queue after R9-2H

Preserve this deep queue; do not collapse it after one green test.

1. **R9-3 process Data-loss recovery:** deterministic post-admission suppression of one/periodic reliable-owned Data packet; cwnd admission before suppression; PTO only after deadline; retransmit with fresh authenticated packet number/nonce and stable Session/frame identity; exactly-once application delivery; independent Carrier ACK and Session DeliveryAck settlement; final Recovery zero or explicit bounded incomplete/error.
2. **R9-4 ACK-loss + reorder/delayed original:** suppress Carrier ACK, retransmit before delayed original, then delayed original; one logical Session delivery, Session-owned duplicate suppression, fresh packet numbers/nonces, truthful loss/PTO counters, final settlement.
3. **R9-5 tamper/future-ACK/malformed negatives:** tampered Data creates no Carrier ACK obligation or Session receive; tampered ACK creates no recovery mutation; future/never-sent ACK rejected atomically; stale/duplicate feedback fabricates no RTT/loss/Session evidence; malformed feedback finite/panic-free.
4. **R9-6 pacing/cwnd/plaintext-owner atomicity:** initial and retransmit sends consult congestion admission; refusal commits no Session byte space or recovery/plaintext owner; one bounded retransmit plaintext owner; finite pacing deadlines; teardown releases ownership.
5. **R9-7 process observability/result truth:** Data, Carrier ACK, Session DeliveryAck, PTO/retransmit, resolved recovery, Session delivery, malformed-budget, cleanup and final-outcome evidence remain distinct; events increment only after the action they claim succeeds.
6. **R9-8 actual authenticated warm TCP standby:** existing TCP connect/accept + canonical negotiation + fresh Noise trust/authz + resume/readiness tuple/generation/delivery-epoch binding + resource admission; warm carries no application Data before atomic promotion.
7. **R9-9 resolved UDP health -> hysteresis -> real TCP promotion:** fresh resolved outcomes drive health; recoverable loss stays UDP; PTO-only observation cannot erase later resolved loss; committed hysteresis promotes only to genuinely ready TCP standby; invalid/unready standby typed failure.
8. **R9-10 uncertain Session replay UDP -> TCP:** replay at least one genuinely uncertain Session range over promoted TCP; draining UDP accepts no new application Data; Session/stream/offset dedup keeps application bytes exactly once; no TCP packet ACK.
9. **R9-11 timeout/shutdown/cleanup matrix:** setup/application/PTO/single-owner receive deadline, controlled stop, failed standby promotion, malformed/tampered input, listener/socket/process cleanup and same-address rebind where supported.
10. **R9-12 coherent exact-tree gate + independent bounded review:** challenge Session-above-Carrier layering, exact Session ACK evidence, authenticated packet identity/Carrier ACK, bounded plaintext/pending ownership, single receive-owner classification/deadline/malformed ownership, PTO/pacing/cwnd, health/hysteresis, warm TCP readiness/promotion, uncertain replay/dedup, result truthfulness and cleanup/no-public-exposure. BLOCKER/HIGH -> smallest repair + regression + re-gate + continue.
11. **Q10 observability reconciliation:** integrate only genuinely new R9 evidence into existing observability/result surfaces; no second logging framework.
12. **Q11 factual status/release reconciliation:** update status/plan/roadmap/release packet only for earned evidence. Only green independently reviewed cross-process R9 plus real TCP promotion may create a specific `READY_LIVE` row.
13. **Q12 one changed-hypothesis self-owned VPS run:** only after Q11 creates a specific `READY_LIVE` question, execute exactly one minimal bounded self-owned client<->VPS run under standing authorization; record exact commit/binary/parameters, recovery/Session/switch evidence and cleanup; preserve negative evidence and do not unchanged-retry.

# VPS opportunity

**Not READY — local evidence dependency.** Unlock chain: finish P1 + P2/P3/P4 + clean exact-tree provenance -> R9-3..R9-12 -> Q10/Q11. Standing authorization already covers the eventual bounded self-owned TCP/UDP run after a specific `READY_LIVE` row exists.

# Non-blocking policy/authority gates

Keep separate from this mechanically determined R9 chain:

- `SessionRuntime.events` retention (`POLICY_BLOCKED_RESOURCE_BOUND`);
- D019 source-retention/no-reset policy;
- RSEC-001 capacity/adversarial-load suitability;
- signing/key custody/SBOM/publication trust;
- previous frozen-release interoperability;
- final RC/freeze/release/production authority.

Do not invent policy values while working R9.
