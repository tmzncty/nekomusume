# ChatGPT reviewer handoff — R9-8 independently closed; R9-9 is front

## Current repository truth

- Latest developer-owned commit reviewed: exact `7f0fcdafbe908d28a7d7715a2b04725ee31883e2`, a docs/review-only R9-8 bounded no-finding note. It changes no implementation, tests, fixtures, protocol/policy state, package surface or live evidence.
- Latest developer-owned source/test commit remains exact `d97a536414a78ff44ef80b1a6d428ddf96ae6e22`, on top of H-R9-074 test repair `65d30dc8a70a9af0da23042f54147a87241123e7`.
- Latest bounded independent R9-8 review: [`docs/reviews/independent-r9-8-warm-tcp-readiness-7f0fcd-20260919.md`](reviews/independent-r9-8-warm-tcp-readiness-7f0fcd-20260919.md), review commit `7c87ac675a381154ae7fceca987e7f2f106c26cf`.
- **R9-8 warm concurrent TCP readiness / directional forwarding is bounded no-finding closed.** The independent pass re-read both the generic concurrent manager/runtime owners and the separate D064 `CarrierManager` + CLI cross-process warm-readiness path. It found no concrete defect in control-only warm preparation, pre-promotion data ownership, exact tuple/direction binding, incomplete-proof failure behavior, single-active/multi-ready ownership, terminal runtime gating or typed readiness evidence.
- R9-7 remains independently bounded no-finding closed at [`docs/reviews/independent-r9-7-process-result-truth-d97a536-20260919.md`](reviews/independent-r9-7-process-result-truth-d97a536-20260919.md), review commit `cdac663e4d2649c8f87fc2263766616dc9b4bbe9`.
- Mid-slice factual reconciliation remains anchored by [`docs/reviews/r9-mid-slice-factual-reconciliation-20260919.md`](reviews/r9-mid-slice-factual-reconciliation-20260919.md), commit `8cc0674cf617257d80b1f7efe3a6cf220a503acc`.
- **H-R9-074 remains closed at `d97a536`.** The post-return Carrier-ACK failure fixture requires initial `udp_packet_ack_sent`, mandatory fail-closed `r9_udp_post_return_sent.packet_number`, matching typed post-return ACK send failure, independent Session `udp_return_delivery_ack_sent`, no Carrier ACK application for the failed attempt, and no false complete settlement.
- R9-4 remains independently closed at implementation/test `95d938859bbe7a42c3e2bf26e09729ba55de5cf0` / review `8c21e24e436379abc13b41e4f163da61b5720193`.
- R9-5 remains independently bounded no-finding closed at `b9f0dc5467771f2398d06f2f6a66ebb3a36c1111`.
- R9-6 remains independently closed at implementation `788a4dcbb4ab802f1444991d7898ec48ed7a9030` / review `5450e9651855e1309bd7e09868f58b078fbce4a6`.
- Developer-local clean exact-tree provenance for exact `d97a536` records `PYTHONDONTWRITEBYTECODE=1 bash scripts/check.sh` exit 0, `git diff --check` exit 0 and a clean tree. Reviewer-local execution is not claimed for the docs-only R9-8 review commits.
- GitHub-hosted Rust CI run `35433084550` for exact `d97a536` completed `success`; hosted CI is cross-evidence only.
- Earlier accepted Candidate A future/never-sent ACK guard and Candidate B mixed datagram-drop observability remain closed absent contradictory current evidence.
- `READY_LIVE: none`.
- Release item 3 remains incomplete.
- Release item 4 remains incomplete.
- `RELEASE_CANDIDATE=false`; `PRODUCTION_READY=false`; `FREEZE=false`; `RELEASED=false`.

The external coding agent must synchronize to current `main` and continuously execute every dependency-ready slice below: implementation/review -> focused deterministic tests -> commit -> push -> clean exact-tree local gate/provenance -> next slice. Reviewer cadence is only a check frequency and is never a ticket length or reason to idle.

Do not reopen R9-8 or H-R9-074 without contradictory current repository evidence.

# READY_LOCAL 1 — R9-9 health outcome / promotion / switch ordering — FRONT

Independently challenge integrated resolved reliable-UDP outcomes -> Carrier health/hysteresis -> promotion.

Challenge at least these invariants:

- recoverable reliable-UDP loss/PTO that settles stays on UDP rather than spuriously promoting TCP;
- terminal/torn-down UDP state cannot generate fresh health/readiness/promotion input;
- actual promotion requires existing readiness plus current health/hysteresis gates;
- draining/failed UDP cannot receive new Session Data ownership;
- single-active invariant holds across fail/activate/drain ordering;
- health/decision evidence remains distinct from packet feedback and Session delivery evidence;
- migration-back cannot bypass generation/session/readiness requirements;
- structured health/switch evidence must follow the typed state/outcome it names rather than merely the presence of a timeout, packet ACK, socket event or Session DeliveryAck.

Read exact-current reliable-UDP resolved-outcome owners, Carrier health/manager owners, CLI automatic-health failover/migration-back executable paths and focused tests before deciding whether a defect exists. Explicitly distinguish a recoverable PTO/loss event from a terminal path-health decision and challenge the boundary with deterministic tests/source reasoning.

If a concrete defect exists and current committed semantics decide the answer: smallest repair -> positive/negative deterministic regression -> commit/push -> clean exact-tree gate/provenance -> immediately continue. If no defect: persist a scope-exact independent bounded no-finding note naming inspected owners, invariants, commands/tests, exclusions and exact reachable anchor, then continue immediately.

Do not change hysteresis/capacity/security numbers or core health/switch architecture.

# READY_LOCAL 2 — R9-10 uncertain replay / dedup / cleanup

Challenge failover after a genuinely unresolved UDP logical range:

- only genuinely uncertain Session ranges replay on promoted TCP;
- stable Session/stream/offset identity is preserved;
- receiver dedup remains exactly-once and conflicting bytes fail closed;
- TCP path does not grow duplicate UDP packet-ACK semantics;
- resolved UDP ranges do not replay;
- retransmitted/overlapping UDP copies do not create duplicate Session ownership;
- shutdown/error/partial-promotion negatives clean Recovery/packet/plaintext ownership and emit no false success.

Do not redesign replay architecture. Any genuine architecture ambiguity becomes a separate maintainer gate while independent local lanes continue.

# READY_LOCAL 3 — R9-11 lifecycle / terminal invariants

Close remaining lifecycle invariants for the cross-process reliable-UDP/failover integration:

- exactly one terminal success/failure classification per bounded operation;
- intermediate timeout/residual diagnostics are not terminal success/failure;
- malformed-bound/socket/receive terminal errors cannot be downgraded into timeout continuation;
- no post-terminal retransmit, delivery confirmation, Carrier ACK emission, readiness mutation, health promotion or new Data ownership;
- listener/socket/session/recovery/Reno/ACK-tracker/retained-plaintext state cleans deterministically on normal/error exits;
- cleanup observations do not rewrite historical failure/recovery evidence;
- repeated teardown/cleanup is idempotent at current committed semantics.

# READY_LOCAL 4 — R9-12 complete cross-process slice + final exact-tree provenance

After R9-9 through R9-11 close, run on the final pushed developer SHA in a safe clean checkout/worktree:

```text
PYTHONDONTWRITEBYTECODE=1 bash scripts/check.sh
git diff --check
```

Confirm clean tree and persist exact reachable pushed SHA, UTC start/end, OS/arch, stable Rust version, exit codes and clean-tree state. Keep developer-local provenance distinct from hosted CI and reviewer execution.

Run pinned decode fuzz only if accumulated R9 work actually changes wire decoder/parser/crypto framing; do not run it mechanically otherwise.

# READY_LOCAL 5 — dedicated independent R9 review

Perform a fresh independent bounded challenge of the materially new R9 send/admission/reservation/socket-outcome/recovery/PTO/demux/Session-ACK/Carrier-ACK/failover/readiness/promotion/migration/cleanup/diagnostic behavior at the final R9 implementation anchor.

A no-finding review is valid item-4 support only when it names inspected owners, challenged invariants, focused deterministic tests/commands, exclusions, exact reachable anchor and evidence class. Any concrete correctness/security/evidence BLOCKER/HIGH returns immediately to smallest repair -> regression -> exact-tree gate.

# READY_LOCAL 6 — Q10/Q11/Q12 factual reconciliation

Only after a reachable independent final-R9 review anchor exists, reconcile `docs/status.md`, `docs/release-security-review-packet.md`, `IMPLEMENTATION_PLAN.md` and related Q10/Q11/Q12 evidence text against exact reachable implementation/review anchors.

Preserve evidence boundaries:

- release item 3 remains incomplete unless repository truth independently closes it;
- item 4 closes only to the extent supported by reachable independent review evidence;
- no historical WAN artifact rewrite;
- no automatic RC/freeze/release/production transition;
- policy/authority gates stay separate.

# READY_LOCAL 7 — repository-wide item-4 inventory refill if still incomplete

If item 4 remains incomplete after final R9 independent review/reconciliation, inventory all mandatory core surfaces and create only genuinely missing independent bounded challenges:

1. `neko-reliable` recovery;
2. `CarrierState`;
3. concurrent Carrier Manager / health / migration-back;
4. FairScheduler / flow accounting;
5. carrier adapters;
6. `SessionRuntime`;
7. observability;
8. package/reproducibility/operator scripts;
9. dependency/build surface;
10. cross-platform CLI/process tests;
11. CLI exit/JSON/human-output contract;
12. algorithmic resource boundedness;
13. release packet factual consistency/evidence boundary.

Earlier reachable independent reviews broadly cover established core implementations. Refill only surfaces that remain unreviewed or were materially changed by R9. No checker/schema/framework/docs filler.

Repository-wide `queue exhausted` is permitted only when this broad inventory finds no concrete defect, no READY_LOCAL review/support lane, no READY_LIVE question, and every remaining item is truly policy/environment/release-authority gated.

# READY_LOCAL 8 — conditional READY_LIVE classification only for a new real-network question

**Current classification: `READY_LIVE: none`.** Standing VPS authorization remains valid, but do not repeat HY2, warm failover, periodic/soak, package lifecycle, migration-back, endpoint/key migration, IPv6, PLPMTUD or Experimental Track evidence merely because the VPS remains rented.

Create a READY_LIVE row only if new code/instrumentation/hypothesis/path conditions produce a concrete unresolved real-network question that cannot be answered locally and stays inside `docs/standing-vps-lab-authorization.md`.

## Separate non-blocking maintainer / policy / authority gates

Do not decide or modify while executing this queue:

- `SessionRuntime.events` retention/capacity policy;
- D019 source-retention/no-reset policy or numeric candidate values;
- RSEC-001 adversarial-load/capacity suitability conditions;
- signing/key-custody/SBOM/publication policy;
- previous frozen-release policy;
- core Session/Carrier/ACK/crypto/wire architecture;
- destructive/canonical-meaning migration;
- RC/freeze/release/production authority.

A policy gate in one lane does not block independent READY_LOCAL review/support lanes.
