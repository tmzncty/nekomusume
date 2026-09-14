# ChatGPT reviewer handoff — R9-2H ordering repair accepted; exact applied identity/evidence remains HIGH

## Current repository truth

- Latest developer source/test SHA reviewed: exact `bf80428e78e99c6fccc523d68b59b0b396b462a3` (`fix(cli): emit ACK validated in mutation order — no reversed evidence (H-R9-012)`).
- Reviewer bounded recheck: `docs/reviews/r9-2h-diagnostic-order-recheck-bf80428-20260914.md` (reviewer commit `794fbf9`).
- `bf80428` changes only `crates/neko-cli/src/main.rs`; it adds no built-binary process regression and no developer-local exact-tree provenance file.
- GitHub-hosted checks on exact `bf80428`: `stable checks` success; `nightly decode fuzz smoke` success. Hosted checks are supplementary cross-evidence only, not developer-local provenance.
- Open PRs: none. No new WAN/VPS experiment occurred. `READY_LIVE: none` remains authoritative.
- Governance remains unchanged: item 3 incomplete; item 4 incomplete; `RELEASE_CANDIDATE=false`; `PRODUCTION_READY=false`; `FREEZE=false`; `RELEASED=false`.

The coding agent is explicitly pre-authorized to finish R9-2H, persist final exact-tree local provenance, and then continue through R9-3..R9-12 and Q10/Q11/Q12 without waiting for reviewer cadence.

## Accepted progress

### H-R9-011 Session over-confirmation — ACCEPTED

The bounded `pending_acks` owner still prevents a later exact Session DeliveryAck from advancing the cumulative Session watermark across an earlier unconfirmed range. Only `record.offset == confirmed_watermark(stream)` may apply immediately; later exact ACKs wait pending and drain only when contiguous. No Session core, wire, crypto or capacity policy changed.

### H-R9-012 mutation/evidence order inversion — source repair ACCEPTED

At exact `bf80428`, an in-order current Session ACK now emits its validated diagnostic immediately after the successful `SessionRuntime::delivery_ack` mutation and **before** any buffered pending ACK is drained. Buffered ACK diagnostics are emitted only after each corresponding buffered Session mutation succeeds. The prior inverted diagnostic order is therefore repaired in source.

Keep the prior closures intact:

- incomplete Carrier settlement is terminal/nonzero and never falls through as success;
- Carrier settlement uses the same absolute application deadline;
- one operation-wide malformed counter spans logical confirmation and Carrier settlement;
- Carrier packet ACK remains Carrier-local recovery feedback and cannot advance Session delivery.

# OPEN HIGH — H-R9-013 exact first applied Session ACK is not attributable

R9-2H-P1 requires **offset-bearing** process evidence proving exact offset 0 applies first and buffered exact offset 16 applies second.

Current `bf80428` still emits the first successful logical confirmation as legacy `udp_delivery_ack_validated` with `ciphertext_bytes` only; it has no `stream` or `offset`. Only later reliable confirmations use `r9_udp_delivery_ack_validated` with an explicit offset. Thus the event order is truthful, but the first successful Session mutation cannot be exactly identified from the client event itself.

Do not close R9-2H by inferring offset 0 from the server seam or from test knowledge. That is weaker than the explicit P1 evidence contract.

## Smallest H-R9-013 repair

Do not change Session/Carrier/ACK/wire/crypto architecture and do not add TTL/LRU/capacity policy.

In reliable mode, emit one explicit `r9_udp_delivery_ack_validated`-class event with exact `stream` + `offset` after **every** successful Session DeliveryAck mutation, including the first. If the legacy `udp_delivery_ack_validated` event is compatibility-sensitive, keep it as an additional compatibility event rather than removing/reinterpreting it. Buffered-drain applications should keep a typed `buffered=true` (or equivalent) attribution.

Before transitioning from logical confirmation into Carrier settlement, explicitly verify that both `outstanding` and `pending_acks` are empty; otherwise fail typed. This is ownership truthfulness and introduces no new numeric policy.

# R9-2H queue front — execute continuously

## R9-2H-A — close H-R9-013 exact applied identity

Make the minimal diagnostic/ownership adjustment above. Preserve H-R9-011 state behavior, H-R9-012 mutation order, the operation-wide malformed budget, the single absolute deadline, and terminal incomplete settlement.

## R9-2H-P1 — real built-binary reversed logical ACK regression

No developer test commit after the prior recheck adds the required real process regression; `bf80428` is source-only. Add a test using real `failover-server` / `failover-client` with the existing `--reverse-ack-order` seam. Assert from real client output:

1. exact offset 16 ACK is observed first;
2. offset 16 is buffered while Session watermark is 0;
3. exact offset 0 applied/validated event is explicit and appears first among successful Session mutations;
4. buffered exact offset 16 applied/validated event appears second;
5. both exact confirmations occur once and logical `outstanding + pending` is empty before success;
6. Carrier packet ACK remains Carrier-local;
7. successful Recovery settlement reaches zero in-flight;
8. no duplicate/conflict application delivery is fabricated.

A test that checks only exit success, event presence, or inferred offset identity is invalid.

## R9-2H-P2 — strengthen migration-back reserved-record ownership

Keep `reliable_udp_migration_back_reserves_final_record`, but make it prove the exact reserved final record:

- is not Recovery-tracked before post-promotion return authorization;
- is not sent by the legacy pre-promotion `udp_uncertain_range_sent` owner;
- is tracked/sent only by the explicit later post-promotion owner;
- is identified by exact offset/record identity, not aggregate counts or one negative log string.

Do not add migration policy.

## R9-2H-P3 — persistent malformed budget across valid Carrier feedback

Through the real built process path drive one reliable receive/settlement operation:

```text
malformed #1
malformed #2
canonical Carrier ACK
malformed #3
```

Malformed #3 must hit the same existing `MAX_POST_HANDSHAKE_MALFORMED` ceiling. The valid Carrier ACK must not reset the operation-wide malformed count. Termination must be typed, finite and non-spinning. Add no numeric policy.

## R9-2H-P4 — preserve incomplete settlement terminality

Keep `reliable_udp_incomplete_settlement_fails_not_settled` green and semantically unchanged: remaining in-flight != 0 is terminal/nonzero, emits incomplete, never emits settled, and never drives downstream health/failover from a false premise.

## R9-2H-GATE — final exact pushed-tree developer-local provenance

After H-R9-013 and P1-P4 land on one reachable source/test SHA, run in a clean worktree and persist:

```text
PYTHONDONTWRITEBYTECODE=1 bash scripts/check.sh
git diff --check
```

Record exact pushed SHA, UTC start/end, Linux OS/arch, Rust stable version, both exit codes, and clean initial/final tree. If wire decoder/parser/crypto framing did not change, do not invent an additional fuzz obligation; hosted fuzz stays supplementary.

R9-2H closes only when one reachable tree proves exact reversed logical confirmation, exact offset-bearing applied evidence, bounded pending ownership, one operation-wide malformed budget, one absolute operation deadline, logical + Carrier settlement before success, migration-back reserved ownership, incomplete-settlement terminality, and local exact-tree provenance.

**Then continue immediately to R9-3. Do not wait for reviewer cadence.**

# Continuous queue after R9-2H

Preserve this deep queue; do not collapse it into one micro-ticket.

## R9-3 — process Data-loss recovery

Add bounded deterministic post-admission suppression of one/periodic reliable-owned Data packet. Prove congestion admission precedes suppression; PTO fires only after deadline; retransmission uses a fresh authenticated packet number/nonce with stable Session/frame identity; application delivery is exactly once; Carrier ACK and Session DeliveryAck settle independently; final Recovery drains to zero or produces an explicit bounded incomplete/error.

## R9-4 — ACK-loss + reorder / delayed original

Cover Carrier packet ACK emitted then suppressed, retransmitted replacement before delayed original, and delayed original after replacement. Require one logical Session delivery, Session-owned duplicate suppression, no conflict, fresh packet numbers/nonces, truthful ACK-loss/PTO counters and final settlement.

## R9-5 — tamper / future ACK / malformed-feedback negatives

Prove tampered Data creates no Carrier ACK obligation or Session receive; tampered ACK creates no recovery mutation; future/never-sent ACK is typed rejected atomically; stale/duplicate ACK fabricates no RTT/loss/Session evidence; malformed feedback is finite and panic-free.

## R9-6 — pacing / cwnd / plaintext-owner atomicity

With existing bounded test-only controls, prove initial and retransmit sends consult congestion admission; refusal consumes no Session byte space and commits no packet/plaintext/recovery owner; retransmit plaintext has one bounded owner; pacing deadlines are finite; teardown releases ownership.

## R9-7 — truthful process observability/result contract

Keep these domains distinct and ordered by actual success: Data offered/admitted/wire-sent/suppressed; Carrier ACK emitted/wire-sent/suppressed/applied/rejected; Session DeliveryAck observed/buffered/applied/duplicate/rejected; PTO due/fired; retransmit attempted/admitted/wire-sent/refused; resolved acked/lost/in-flight; Session first-delivery/duplicate/conflict/application bytes; malformed-budget use, cleanup and final outcome. Counters/events increment only after the action they claim succeeds.

## R9-8 — actual authenticated warm TCP standby

Reuse existing TCP connect/accept, canonical negotiation, fresh Noise trust/authz, resume/readiness tuple+generation+delivery-epoch binding and resource admission. Warm/standby carries no application Data before atomic promotion.

## R9-9 — resolved UDP health -> hysteresis -> real TCP promotion

Fresh resolved UDP outcomes drive Carrier health. Recoverable loss stays on UDP; PTO-only observation cannot erase later resolved loss; distinct bad resolved intervals cross committed hysteresis and promote only to an actually ready TCP standby. Invalid/unready standby yields typed failure.

## R9-10 — uncertain Session replay across UDP -> TCP

At promotion replay at least one genuinely uncertain logical Session range over promoted TCP. Draining UDP accepts no new application Data. Receiver deduplicates by Session/stream/offset and application bytes remain exactly once. Do not add TCP packet ACK.

## R9-11 — timeout / shutdown / cleanup matrix

Cover setup/application/PTO/single-owner receive deadline, controlled stop, failed standby promotion, malformed/tampered input, listener/socket/process cleanup and same-address rebind where supported.

## R9-12 — coherent exact-tree gate + independent bounded review

Independently challenge Session-above-Carrier layering, exact Session ACK evidence, authenticated packet identity/Carrier ACK, bounded plaintext/pending-ACK ownership, single receive-owner classification + operation-wide malformed/deadline ownership, deadline-driven PTO/pacing/cwnd, fresh health/hysteresis, real warm TCP readiness/promotion, uncertain Session replay/dedup, result truthfulness and cleanup/no-public-exposure. BLOCKER/HIGH -> smallest repair + regression + re-gate + continue; LOW/NOTE does not halt progression.

## Q10 — observability reconciliation

Integrate only genuinely new R9 evidence into existing `neko-observe`/result surfaces. Do not create a second logging framework.

## Q11 — factual status/release reconciliation

Reconcile `docs/status.md`, `IMPLEMENTATION_PLAN.md`, `ROADMAP.md` and the release packet only for evidence actually earned by the final R9 tree. Only a green independently reviewed cross-process R9 plus real TCP promotion may create a new specific `READY_LIVE` row.

## Q12 — one changed-hypothesis self-owned VPS run

Only after Q11 creates a specific `READY_LIVE` question, execute exactly one minimal bounded self-owned client<->VPS run under standing authorization. Record exact commit/binary/parameters, authenticated recovery/Session/switch evidence and cleanup. Preserve negative evidence; no unchanged same-class retry.

# VPS opportunity

**Not READY — implementation/evidence dependency.** Current unlock chain: R9-2H -> R9-3..R9-12 -> Q10/Q11. Once Q11 truthfully creates a specific changed `READY_LIVE` row, standing authorization already covers the bounded self-owned TCP/UDP run; do not ask for generic WAN permission again.

# Non-blocking policy/authority gates

Keep separate from the mechanically determined R9 chain:

- `SessionRuntime.events` retention (`POLICY_BLOCKED_RESOURCE_BOUND`);
- D019 source-retention/no-reset policy;
- RSEC-001 capacity/adversarial-load suitability;
- signing/key custody/SBOM/publication trust;
- previous frozen-release interoperability;
- final RC/freeze/release/production authority.

Do not invent policy values while working R9.
