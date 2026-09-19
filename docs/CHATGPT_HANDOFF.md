# ChatGPT reviewer handoff — R9-10B closed at 70e87f0; R9-10C is front

## Current repository truth

- Latest developer-owned source/test commit: exact `70e87f0d6cf2e5ac4cf3d4378880b5fa1d90ec1d` (`fix(cli): R9-10B TCP replay bytes/identity from authoritative retained set`).
- New developer-owned source/test commits since the prior reviewer handoff were reviewed:
  - `f064424f7f06c35fba31fc715da443c866a6a27b` / `52017b5adb26f377857adad2c9a6ed945e539d10` — H-R9-075 SessionRuntime gapped DeliveryAck repair and test restoration;
  - `f1edee86e9bd44c953915b745bd51418302db127` — resumed-session integration fixture applies the first contiguous Session proof before later resumed proofs;
  - `1429df1b565060054e24cfbae4d62bfdda5890da` — gapped-ACK negative now asserts confirmed watermark, per-stream/session send-inflight and event-count atomicity;
  - `70e87f0d6cf2e5ac4cf3d4378880b5fa1d90ec1d` — R9-10B TCP replay now builds each record from the authoritative retained `(DataId, bytes)` set (`FailoverController::tcp_resend`), not a positional slice — a wrong-offset or wrong-bytes replay cannot hide behind a matching count.
- **H-R9-075 remains closed.** `SessionRuntime::delivery_ack` now rejects a forward gap (`offset > confirmed watermark`) before delivery accounting mutation; ordered contiguous ACKs still advance normally.
- **H-R9-076 remains closed.** The resumed-session fixture no longer relies on the forbidden cumulative-gap behavior; successful proof application precedes `confirm`.
- Developer-local clean exact-tree provenance for exact `70e87f0`: `PYTHONDONTWRITEBYTECODE=1 bash scripts/check.sh` exit 0, `git diff --check` exit 0, clean worktree at pushed SHA, 2026-09-19T15:00:05Z → 2026-09-19T15:04:57Z, Linux x86_64, rustc 1.98.0. This remains developer-reported local evidence; reviewer-local execution is not claimed.
- Hosted evidence is separate. Exact `1429df1` Rust CI run `35447262812` had nightly decode fuzz success but stable `scripts/check.sh` failed late in `scripts/bench/process-resource-sampler-test.py` because `fd.peak_count` was 3 rather than the fixture's expected >=9. The unchanged source/test code on docs-only descendant exact `6f4a61c` passed hosted Rust CI run `35447630498` completely. Independent bounded review `43bad9d5` records this as a deterministic-fixture scheduling flaw, not a transport correctness failure and not a reason to invalidate the accepted developer-local gate.
- R9-7 remains independently bounded no-finding closed at `cdac663e4d2649c8f87fc2263766616dc9b4bbe9` over source/test exact `d97a536`.
- R9-8 warm readiness remains independently bounded no-finding closed at `7c87ac675a381154ae7fceca987e7f2f106c26cf`.
- R9-9 health/promotion/switch ordering remains independently bounded no-finding closed at `b4a527a1fff994ee0836893f850c36eadb609eb1`.
- Candidate A (`Recovery::on_ack` future/unsent `largest`) remains closed by the current fail-closed guard before RTT/loss/PTO mutation and dedicated regressions.
- Candidate B (`record_datagrams` mixed queue/terminal drop projection) remains closed by current mixed-delta regressions and separate reason projection.
- `READY_LIVE: none`. Do not repeat HY2, warm failover, periodic/soak, package lifecycle, migration-back, endpoint/key migration, IPv6, PLPMTUD, or Experimental Track without a new code/instrumentation/hypothesis/path condition that creates a concrete unresolved real-network question.
- Release item 3 and item 4 remain incomplete. `RELEASE_CANDIDATE=false`, `PRODUCTION_READY=false`, `FREEZE=false`, `RELEASED=false` remain unchanged.

## READY_LOCAL 1 — FRONT: R9-10B executable TCP replay identity/bytes/dedup/conflict

Challenge the actual promoted-TCP replay owner, not only `ConcurrentCarrierManager` unit models.

Current source seam to challenge:

- the automatic-health path retains uncertain payloads in `FailoverController::uncertain: BTreeMap<DataId, Vec<u8>>`;
- `FailoverController::tcp_resend()` returns the authoritative retained `(DataId, bytes)` set;
- the executable currently uses only `failover.tcp_resend().unwrap().len()` and then sends a positional slice: `records.into_iter().skip(uncertain_start).take(tcp_records)`;
- current construction initially tracks the same contiguous positional slice and no pre-replay `confirm` has been found, so do **not** invent a defect merely because the coupling is weak. Try to create a reachable divergence first.

Review contract:

1. Read exact-current `FailoverController::{track_uncertain,tcp_resend,confirm,receive}` plus the executable UDP->TCP promotion/replay owner and applicable D005/D064/Session v0 boundaries.
2. Prove or falsify that actual TCP replay bytes/identity come from the authoritative retained replay set rather than merely an equal count/position assumption.
3. Challenge at least:
   - stable `DataId` + exact bytes across UDP->TCP;
   - already confirmed ranges absent from replay;
   - exact duplicate idempotent;
   - same `DataId` / different bytes fails closed;
   - Carrier packet ACK evidence cannot suppress required Session replay;
   - TCP does not manufacture a UDP packet-ACK layer;
   - authenticated TCP Session DeliveryAck settles the replay exactly once.
4. Prefer a focused deterministic executable/process regression that mutates the retained-set premise without inventing a new architecture. If no reachable state can make retained identity/bytes diverge from the positional source, record a scope-precise no-finding review explaining why the equivalence is currently enforced.
5. If a concrete reachable mismatch exists and current committed semantics already determine the answer, make the smallest repair so the executable consumes the authoritative retained identities/bytes, add positive/negative regressions, run the final pushed exact-tree gate, record provenance, then continue immediately.

Do not redesign DataId, Session ACK encoding, D064, D019, replay-retention capacities, or wire/crypto architecture in this slice.

## READY_LOCAL 2 — process-resource sampler positive-fixture determinism

Independent review anchor: `43bad9d5cfd91333b2f6df513d58aa29cfac1c55`, `docs/reviews/independent-process-resource-sampler-flake-6f4a61c-20260919.md`.

Hosted run `35447262812` proved the current `known_child.py` positive FD/listener fixture can miss its 0.25-second resource-owning interval and fail with `peak_count=3`; unchanged code passed the next docs-only hosted run. Repair the **test synchronization contract**, not transport semantics and not capacity policy: use a bounded readiness/release handshake (or equivalent deterministic synchronization) so the positive oracle knows the resources exist before requiring the sampler to observe them. Preserve truthful null exit-race behavior and process-group cleanup. Run normal exact-tree local gate. This lane is real package/evidence-reliability work but does not preempt R9-10B correctness review.

## READY_LOCAL 3 — R9-10C replay cleanup / partial-promotion negatives

After R9-10B is closed, challenge replay cleanup and failure atomicity around partial TCP promotion/replay: failed authentication/readiness/promotion must not consume uncertain ownership; a failed/partial replay must not fabricate Session confirmation; successful confirmation removes exactly the proved identity; terminal cleanup must release live replay ownership while retaining only already-approved history semantics. No capacity values may be invented.

## READY_LOCAL 4 — R9-11 lifecycle / terminal resource closure

Bounded independent challenge of the full failover/migration lifecycle after R9-10: socket/Session/Recovery/CarrierManager/replay ownership on success, failure, timeout, shutdown and post-return paths. Reuse existing deterministic process fixtures; do not add soak/capacity pressure. Terminal state must fail closed on further mutators while approved diagnostic history may remain.

## READY_LOCAL 5 — R9-12 final exact-tree provenance

After the coherent R9 repair/review group is stable, run and persist developer-local provenance for the final pushed developer SHA:

- `PYTHONDONTWRITEBYTECODE=1 bash scripts/check.sh`
- `git diff --check`
- clean tree
- exact reachable SHA, UTC start/end, exit codes, OS/arch and stable Rust version

Run pinned decoder fuzz only if the coherent group actually touched wire decoder/parser/crypto framing; otherwise do not mechanically repeat it. Hosted CI remains supplemental cross-evidence.

## READY_LOCAL 6 — dedicated final independent R9 bounded review

On the final R9 exact tree, independently re-challenge the combined invariants across Recovery, Session logical proof, Carrier readiness/promotion, uncertain retention, TCP replay/dedup, migration-back and terminal cleanup. A no-finding bounded review is valid item-4 support. If a concrete defect appears, immediately convert to the smallest repair lane and do not claim R9 closure first.

## READY_LOCAL 7 — Q10 factual reconciliation

Reconcile release packet/status claims against exact reachable R9 review/provenance anchors. Update only factual indexes/boundaries that are actually changed; do not mark release item 4 complete and do not promote historical/local evidence into WAN/security/release claims.

## READY_LOCAL 8 — Q11 release-evidence boundary reconciliation

Recheck release item 3 / WAN rows, historical negative classifications, and `READY_LIVE`. Current authoritative classification remains `READY_LIVE: none`; only a newly created concrete unresolved real-network question may open a live lane.

## READY_LOCAL 9 — Q12 governance/release-state reconciliation

Confirm D019 and other policy/value gates remain separated, and that RC/production/freeze/release authority has not been inferred from engineering completion. No policy numbers, signing/key-custody/SBOM/publication policy, destructive migration, or release authority change may be made here.

## READY_LOCAL 10 — repository-wide item-4 refill

Item 4 remains incomplete, so repeat the broad core-surface inventory rather than declaring queue exhaustion from a narrow seam. Prefer any implemented core surface that lacks a dedicated reachable independent bounded challenge, including reliable recovery, CarrierState, concurrent manager/health/migration-back, scheduler/flow control, adapters, SessionRuntime, observability, package/operator scripts, dependency/build/native hooks/unsafe inheritance, cross-platform CLI/process semantics, CLI JSON/human/exit contracts, algorithmic boundedness, and release-packet factual consistency. No-finding independent review is valid support; concrete defects become repair lanes.

## READY_LOCAL 11 — conditional live question only

Only if new code/instrumentation/hypothesis/path condition creates a specific unresolved self-owned real-network question, classify it against `docs/standing-vps-lab-authorization.md` and `docs/vps-rental-window-priority.md`. Ordinary bounded self-owned TCP/UDP work inside standing authorization does not need per-run approval. Do not manufacture live work merely because the VPS is rented.

## Stop / escalation conditions

Continue implementation/review -> tests -> commit -> push -> next dependency-ready slice without waiting for the reviewer cadence. Stop and escalate only for an unresolved BLOCKER/HIGH that cannot be auto-decided, core Session/Carrier/ACK/crypto/wire architecture change, D019 or other policy/value choice, destructive/canonical migration, out-of-standing-authorization operation, third-party/production/new-credential permission, maintainer-selected adversarial-load/benchmark conditions, or entry into a genuinely new release stage.
