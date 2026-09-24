# ChatGPT reviewer handoff — H-I4-119 semantics gate; READY_LOCAL queue continues

## Current repository truth

- Synchronize to current `main` before work. Exact code/tests/reachable commits/current specs outrank this handoff, chat memory and stale checkbox state.
- **H-I4-119 is no longer an ordinary READY_LOCAL repair. It is a MAINTAINER / CORE ACK-PTO SEMANTICS GATE.** Exact-current `Recovery::on_ack` resets `pto_count` whenever an ACK newly retires at least one sent packet. The earlier finding proposed changing that to reset only when a newly retired packet was `ack_eliciting=true`, but current repository docs do not uniquely specify that stronger rule, and RFC 9002 Appendix A.7 resets `pto_count` after any nonempty `newly_acked_packets` set (subject to its address-validation exception) while using `IncludesAckEliciting(...)` for RTT sampling instead. Reconciliation note: `docs/reviews/reviewer-h-i4-119-pto-reset-semantics-gate-20260924.md`, commit `49166d0b9d7faa67a89f472d57dee28e5554b633`. Historical finding `docs/reviews/reviewer-h-i4-119-non-ack-eliciting-ack-pto-reset-20260924.md` remains preserved.
- **Do not auto-patch H-I4-119.** Maintainer/spec must explicitly decide whether this project intends (a) any newly acknowledged sent packet to reset `pto_count`, or (b) only newly acknowledged ack-eliciting packets to reset it, and how that choice relates to this project's simplified PTO-count-based health/persistent-congestion evidence. Do not invent new thresholds/capacity values.
- Important boundary remains: current `ReliableUdpRuntime::{on_packet_sent,on_retransmit_sent}` constructs only `ack_eliciting=true` `SentPacket`s. The disputed seam is the reusable `Recovery` / `PathRecovery` model, not a demonstrated canonical runtime ACK-only send failure.
- **H-I4-118 CLOSED** at developer source/test `71b62c642a6ea736fd162653ede9b29868e08eee` with reachable developer-local exact-tree provenance indexed by the closing handoff `270265f13c2644f3151dcb21d02ccd1dbbb38902`. H-I4-116/117 are also closed with reachable provenance. H-I4-097..115 remain closed unless their exact owners materially move. Candidate A/B remain closed unless their owners move.
- **R-CS-1 CLOSED** at `259fb58ec19a47fe4e11900dbe6b338bb09f9467`; **R-CS-2 CLOSED** at `44a77cb7f4abc124606ba77e56edc74cacccf395`; **R-CM-DIFF CLOSED** at `44fd3d69edd9802a73721a10596e0f19ac04f404`. Recovery/CarrierState reconciliation is reachable at `fe2243a89e974991877972f43b9f13bd4b261ebf`.
- **R-PKG/BLD-DIFF CLOSED** at `12374ee99f5bf5bdd5096e8d5e24a88d9046d188`; **R-OBS/ADAPTER-DIFF CLOSED** at `e228ec61a25a54e77de9b5ba38a4fda4393c1191`; **R-FS/SESSION-DIFF CLOSED** at `33b6a3c907ba0d37913def5cc98fd1402900bfec`; **R-CLI/BND-DIFF CLOSED** at `a34f38f0a6cda7086195a45562307035a476ad69`. These are bounded source/owner-diff no-findings, not current exact-tree execution records. Grouped release/item-4 reconciliation is reachable at `d4e2e42b7a16e6946771c44bc436be9017c6993a`.
- Release items **3 and 4 remain incomplete**. `RELEASE_CANDIDATE=false`, `PRODUCTION_READY=false`, `FREEZE=false`, `RELEASED=false` remain unchanged.
- **`READY_LIVE: none`.** Standing self-owned VPS authorization remains valid, but do not repeat historical live evidence absent a materially new code/instrumentation/hypothesis/path-condition question.

## MAINTAINER / SPEC GATE — H-I4-119

Do not let this gate become an agent-idle reason. It blocks only the disputed `pto_count` reset semantic.

The historical challenge sequence is still useful: unresolved ack-eliciting packet + later non-ack-eliciting packet + PTO(s) + ACK only the non-ack-eliciting packet. What is **not** decided is whether resetting the shared `pto_count` in that sequence is wrong for Nekomusume.

Before any implementation change, maintainer/spec must choose/document the intended rule:

1. **Any newly acknowledged packet resets `pto_count`:** retain current reset condition; H-I4-119 closes as no code defect after adding/adjusting semantic documentation/tests if useful.
2. **Only newly acknowledged ack-eliciting packets reset `pto_count`:** explicitly adopt this project-specific rule; then the earlier narrow implementation idea becomes dependency-ready.

Do not silently infer the choice from Reno byte charging or receiver ACK-obligation behavior; those are distinct meanings already attached to `ack_eliciting`.

## Dependency-ready rolling queue — continue immediately

The coding/review agent must continue these lanes without waiting for H-I4-119 or for the next reviewer cadence:

1. **R-REC-REFILL-RESIDUAL.** Rechallenge exact-current Recovery seams not requiring the disputed PTO-reset choice: duplicate/stale ACK ownership after H-I4-118, ACK-range atomicity/high-water, positive/zero-loss Reno accounting after H-I4-116, persistent-congestion effect/wiring after H-I4-117 (excluding the disputed reset trigger), frame-copy/retransmit lifetime, `abandon_sent` rollback/watermark ownership, deterministic fault simulation, health freshness bridge. Concrete defect -> smallest repair/tests/provenance; no defect -> precise bounded no-finding note. Do not rebuild Recovery architecture.
2. **R-PREAUTH-DIFF.** Re-read current pre-auth rejection/accounting owners against the last independent pre-auth review. If semantic owners are unchanged, record bounded reuse; if moved, challenge the moved owner. D019 source-retention/no-reset is a maintainer/security policy gate and must not be decided here.
3. **R-CORE-INV.** Produce a repository-wide current-owner inventory across all thirteen standing surfaces: exact owner path(s), latest semantic-owner commit, latest independent review/repair anchor, and whether moved/unreviewed since that anchor. This is navigation/review support, not a new schema framework project.
4. **R-CORE-MOVED-1.** First dependency-ready implemented core owner revealed by inventory as materially moved or lacking dedicated bounded review. Read exact source/tests/spec; challenge a concrete invariant; repair only if semantics already decide the answer.
5. **R-CORE-MOVED-2.** Second independent moved/unreviewed owner under the same contract. A bounded no-finding is valid item-4 support.
6. **R-CORE-MOVED-3.** Third independent owner when available so the queue stays deep; prefer correctness/security/resource/lifecycle seams over docs polish.
7. **R-RELEASE-PACKET-DIFF.** After the next 3–4 coherent review/repair slices, reconcile `docs/release-security-review-packet.md`, `docs/status.md`, and item-4 facts. Never imply one SHA executed all historical tests; preserve evidence classes.
8. **R-SECURITY-BOUNDARY-DIFF.** Cross-check exact-current `SECURITY.md`, applicable ADR/spec claims, and implementation evidence only where semantic owners/claims moved. Do not invent security values or convert research evidence into approval.
9. **R-CLI/PROC-RESIDUAL only if owner moved.** Reuse the current process/boundedness reviews when unchanged; if changed, challenge exit-code/JSON/human-output and process/socket/thread cleanup semantics.
10. **R-OBS/SESSION-RESIDUAL only if owner moved.** Candidate B remains closed while its projection owner is unchanged. If observability or SessionRuntime owners moved, challenge the moved seam rather than repeating old review wholesale.
11. **R-PKG/BLD-RESIDUAL only if owner moved.** Reuse current package/build review while manifests/scripts/native hooks/unsafe inheritance remain unchanged; otherwise challenge exact moved owner.
12. **CONDITIONAL LIVE only on a changed real-network question** inside standing authorization. Current classification remains `READY_LIVE: none`; do not repeat HY2, warm failover, periodic/soak, package lifecycle, migration-back, endpoint/key migration, IPv6, PLPMTUD or Experimental Track without a new concrete unresolved path condition.
13. **FINAL broad factual reconciliation only after inventory + moved-owner challenges materially close.** Item 4 remains unchecked until remaining independent-review/policy/external boundaries are truthfully resolved; no automatic RC/freeze/release/production transition.

## Standing 13-surface inventory requirement

Every meaningful reviewer refill must verify whether each surface still has a dedicated, reachable, independent bounded review or a materially moved owner:

1. `neko-reliable` UDP recovery — ACK range/future/stale ACK/loss/retransmit/RTT/PTO/persistent congestion/Reno/fault simulation;
2. `neko-carrier` `CarrierState` — generation/validation/hysteresis/single-active/drain/fail/activate;
3. Concurrent Carrier Manager / health / migration-back;
4. FairScheduler / multi-stream / Session + stream flow-control accounting;
5. Memory/UDP/TCP carrier adapter close/error/resource semantics;
6. `SessionRuntime` lifecycle/resource/window/DeliveryAck accounting;
7. `neko-observe` projection/event/counter/high-water correctness;
8. package/reproducibility/operator scripts;
9. dependency/build manifests/lock/features/build/native hooks/unsafe inheritance;
10. cross-platform CLI/process-test semantics;
11. CLI exit-code / JSON / human-output contract;
12. algorithmic resource boundedness without capacity-pressure benchmark or invented policy values;
13. release packet factual consistency/evidence boundary.

A narrow no-finding never means repository-wide queue exhaustion. While item 4 is open, an implemented core surface that has moved or has never received dedicated bounded challenge is READY review-support work.

## Current REUSE map

- **Recovery:** R-REC-1/2/3 remain scoped no-findings; R-REC-4a/4b positive-loss/fault-simulation closed; H-I4-116/117/118 closed through `71b62c6`; H-I4-119 is now a maintainer/spec semantic gate, not a READY repair.
- **CarrierState:** current R-CS-1/R-CS-2 no-finding notes; do not infer D064 runtime migration policy from old M0 CarrierState.
- **Concurrent Carrier Manager / health / migration-back:** current R-CM-DIFF plus prior dedicated anchors unless semantic owners move.
- **FairScheduler / multistream / SessionRuntime:** current `R-FS/SESSION-DIFF` + prior I4-FS1/FS2/lifecycle chain; retained-history capacity remains policy-blocked.
- **Carrier adapters + observability:** current `R-OBS/ADAPTER-DIFF`; Candidate B closed while projection owner unchanged.
- **Package/reproducibility/operator + dependency/build:** current `R-PKG/BLD-DIFF`; signing/SBOM/key-custody/publication remain external/policy.
- **CLI output/process + algorithmic boundedness:** current `R-CLI/BND-DIFF`; Linux evidence is not other-OS proof; no invented capacity values.
- **Pre-auth:** reuse `independent-preauth-rejection-accounting-d96aabe-20260922.md` if owners unchanged; D019 is separate.
- **Release/evidence:** recovery/CarrierState reconciliation `fe2243a8` + four-slice reuse reconciliation `d4e2e42`; packet remains evidence index, not approval.

## Review -> repair / provenance contract

For every bounded slice: read exact-current owner source/tests + applicable spec/ADR/status claim; state the invariant; try to falsify it with source reasoning and focused deterministic tests. If a concrete defect exists and committed semantics already decide the answer, make the smallest repair + positive/negative regression + commit/push, then run the final pushed developer source SHA through:

```text
PYTHONDONTWRITEBYTECODE=1 bash scripts/check.sh
git diff --check
```

and verify a clean tree. Persist a reachable exact-tree provenance record with SHA, UTC start/end, exit codes, OS/arch and stable Rust version. Hosted CI is separate cross-evidence and never a wait condition. Wire decoder/parser/crypto-framing changes additionally use the pinned fuzz toolchain and required decode build/run; do not run fuzz mechanically for unrelated code.

If no defect is found, record a scope-precise independent bounded no-finding note naming owners inspected, commands/tests if actually run, exclusions and exact reachable anchor. Do not change code merely to manufacture review churn.

## Evidence discipline / stop conditions

- Developer-reported local CI, persisted developer-local provenance, reviewer source/control-flow review, reviewer-local generic OS checks, hosted CI, live WAN evidence and performance conclusions are distinct classes.
- Accepted exact-tree provenance must anchor a GitHub-resolvable pushed SHA; never publish local-only/unreachable SHA as shared evidence.
- Never decide D019; TTL/LRU/history/capacity/security values; signing/key-custody/SBOM/publication; previous frozen release policy; core Session/Carrier/ACK/crypto/wire architecture; destructive/canonical migration; RC/freeze/release/production authority.
- A new correctness/security/evidence BLOCKER/HIGH immediately becomes FRONT **only when current committed semantics determine a repair**. If the finding requires a core semantic/policy choice, classify it as a maintainer/spec gate and continue independent READY work.
- Queue exhaustion is legal only after the broad 13-surface inventory shows no unreviewed moved core owner, no concrete defect, no READY review-support lane, no READY live question, and all remaining work is genuinely policy/external/environment/release-authority gated.
