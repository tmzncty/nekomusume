# ChatGPT reviewer handoff — R9-7 independently closed; R9-8 is front

## Current repository truth

- Latest developer-owned source/test commit reviewed: exact `d97a536414a78ff44ef80b1a6d428ddf96ae6e22`, on top of H-R9-074 test repair `65d30dc8a70a9af0da23042f54147a87241123e7`.
- Latest bounded independent R9-7 review: [`docs/reviews/independent-r9-7-process-result-truth-d97a536-20260919.md`](reviews/independent-r9-7-process-result-truth-d97a536-20260919.md), review commit `cdac663e4d2649c8f87fc2263766616dc9b4bbe9`.
- Mid-slice factual reconciliation: [`docs/reviews/r9-mid-slice-factual-reconciliation-20260919.md`](reviews/r9-mid-slice-factual-reconciliation-20260919.md), commit `8cc0674cf617257d80b1f7efe3a6cf220a503acc`.
- **H-R9-074 is closed at `d97a536`.** The post-return Carrier-ACK failure fixture requires initial `udp_packet_ack_sent`, mandatory fail-closed `r9_udp_post_return_sent.packet_number`, matching typed post-return ACK send failure, independent Session `udp_return_delivery_ack_sent`, no Carrier ACK application for the failed attempt, and no false complete settlement.
- **R9-7 focused process/result-truth re-challenge is bounded no-finding closed** at `cdac663`. The review re-challenged first-send rollback, malformed-bound terminality, timeout-only continuation, Session-vs-Carrier ACK separation, positive Carrier retirement projection, and future/never-sent ACK fail-closed behavior at current owners.
- R9-4 remains independently closed at implementation/test `95d938859bbe7a42c3e2bf26e09729ba55de5cf0` / review `8c21e24e436379abc13b41e4f163da61b5720193`.
- R9-5 remains independently bounded no-finding closed at `b9f0dc5467771f2398d06f2f6a66ebb3a36c1111`.
- R9-6 remains independently closed at implementation `788a4dcbb4ab802f1444991d7898ec48ed7a9030` / review `5450e9651855e1309bd7e09868f58b078fbce4a6`.
- Developer-local clean exact-tree provenance for exact `d97a536` records `PYTHONDONTWRITEBYTECODE=1 bash scripts/check.sh` exit 0, `git diff --check` exit 0 and a clean tree. Reviewer-local execution is not claimed.
- GitHub-hosted Rust CI run `35433084550` for exact `d97a536` completed `success`; hosted CI is cross-evidence only.
- Earlier accepted Candidate A future/never-sent ACK guard and Candidate B mixed datagram-drop observability remain closed absent contradictory current evidence.
- `READY_LIVE: none`.
- Release item 3 remains incomplete.
- Release item 4 remains incomplete.
- `RELEASE_CANDIDATE=false`; `PRODUCTION_READY=false`; `FREEZE=false`; `RELEASED=false`.

The external coding agent must synchronize to current `main` and continuously execute every dependency-ready slice below: implementation/review -> focused deterministic tests -> commit -> push -> clean exact-tree local gate/provenance -> next slice. Reviewer cadence is only a check frequency and is never a ticket length or reason to idle.

The stale duplicate H-R9-074 FRONT HIGH ticket from the previous handoff has been removed. Do not reopen it without contradictory current repository evidence.

# READY_LOCAL 1 — R9-8 warm concurrent TCP readiness / directional forwarding — FRONT

Independently challenge the currently implemented concurrent-carrier readiness path against D064/current carrier architecture and executable cross-process behavior.

Challenge at least these invariants:

1. authenticated resume-bound TCP warm standby carries readiness/control only before promotion;
2. no application Data is owned/sent on TCP while UDP remains the active carrier;
3. readiness is not inferred merely from TCP connect, UDP packet feedback, Session DeliveryAck, or a stale generation;
4. readiness admission remains bound to the current session/epoch/path generation and correct direction;
5. failed/incomplete readiness cannot emit positive ready/promotion evidence or steal active ownership;
6. single-active plus multi-ready semantics remain true through standby creation, activation, drain and failure;
7. a torn-down reliable-UDP runtime cannot manufacture new readiness/control evidence;
8. positive structured readiness evidence follows the actual typed state transition it names.

Read exact-current `crates/neko-carrier` readiness/manager owners, `crates/neko-cli` failover executable owners/tests, `docs/carrier-architecture.md`, applicable decisions/specs and current R9 evidence before deciding whether a defect exists.

If a concrete defect exists and current committed semantics decide the answer: smallest repair -> positive/negative deterministic regression -> commit/push -> clean exact-tree gate/provenance -> immediately continue. If no defect: persist a scope-exact independent bounded no-finding note naming inspected owners, invariants, commands/tests, exclusions and exact reachable anchor, then continue immediately.

No policy values or core architecture changes.

# READY_LOCAL 2 — R9-9 health outcome / promotion / switch ordering

Challenge integrated resolved reliable-UDP outcomes -> Carrier health/hysteresis -> promotion:

- recoverable reliable-UDP loss/PTO that settles stays on UDP rather than spuriously promoting TCP;
- terminal/torn-down UDP state cannot generate fresh health/readiness/promotion input;
- actual promotion requires existing readiness plus current health/hysteresis gates;
- draining/failed UDP cannot receive new Session Data ownership;
- single-active invariant holds across fail/activate/drain ordering;
- health/decision evidence remains distinct from packet feedback and Session delivery evidence;
- migration-back cannot bypass generation/session/readiness requirements.

Do not change hysteresis/capacity/security numbers.

# READY_LOCAL 3 — R9-10 uncertain replay / dedup / cleanup

Challenge failover after a genuinely unresolved UDP logical range:

- only genuinely uncertain Session ranges replay on promoted TCP;
- stable Session/stream/offset identity is preserved;
- receiver dedup remains exactly-once and conflicting bytes fail closed;
- TCP path does not grow duplicate UDP packet-ACK semantics;
- resolved UDP ranges do not replay;
- retransmitted/overlapping UDP copies do not create duplicate Session ownership;
- shutdown/error/partial-promotion negatives clean Recovery/packet/plaintext ownership and emit no false success.

Do not redesign replay architecture. Any genuine architecture ambiguity becomes a separate maintainer gate while independent local lanes continue.

# READY_LOCAL 4 — R9-11 lifecycle / terminal invariants

Close remaining lifecycle invariants for the cross-process reliable-UDP/failover integration:

- exactly one terminal success/failure classification per bounded operation;
- intermediate timeout/residual diagnostics are not terminal success/failure;
- malformed-bound/socket/receive terminal errors cannot be downgraded into timeout continuation;
- no post-terminal retransmit, delivery confirmation, Carrier ACK emission, readiness mutation, health promotion or new Data ownership;
- listener/socket/session/recovery/Reno/ACK-tracker/retained-plaintext state cleans deterministically on normal/error exits;
- cleanup observations do not rewrite historical failure/recovery evidence;
- repeated teardown/cleanup is idempotent at current committed semantics.

# READY_LOCAL 5 — R9-12 complete cross-process slice + final exact-tree provenance

After R9-8 through R9-11 close, run on the final pushed developer SHA in a safe clean checkout/worktree:

```text
PYTHONDONTWRITEBYTECODE=1 bash scripts/check.sh
git diff --check
```

Confirm clean tree and persist exact reachable pushed SHA, UTC start/end, OS/arch, stable Rust version, exit codes and clean-tree state. Keep developer-local provenance distinct from hosted CI and reviewer execution.

Run pinned decode fuzz only if accumulated R9 work actually changes wire decoder/parser/crypto framing; do not run it mechanically otherwise.

# READY_LOCAL 6 — dedicated independent R9 review

Perform a fresh independent bounded challenge of the materially new R9 send/admission/reservation/socket-outcome/recovery/PTO/demux/Session-ACK/Carrier-ACK/failover/readiness/promotion/migration/cleanup/diagnostic behavior at the final R9 implementation anchor.

A no-finding review is valid item-4 support only when it names inspected owners, challenged invariants, focused deterministic tests/commands, exclusions, exact reachable anchor and evidence class. Any concrete correctness/security/evidence BLOCKER/HIGH returns immediately to smallest repair -> regression -> exact-tree gate.

# READY_LOCAL 7 — Q10/Q11/Q12 factual reconciliation

Only after a reachable independent final-R9 review anchor exists, reconcile `docs/status.md`, `docs/release-security-review-packet.md`, `IMPLEMENTATION_PLAN.md` and related Q10/Q11/Q12 evidence text against exact reachable implementation/review anchors.

Preserve evidence boundaries:

- release item 3 remains incomplete unless repository truth independently closes it;
- item 4 closes only to the extent supported by reachable independent review evidence;
- no historical WAN artifact rewrite;
- no automatic RC/freeze/release/production transition;
- policy/authority gates stay separate.

# READY_LOCAL 8 — repository-wide item-4 inventory refill if still incomplete

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

# READY_LOCAL 9 — conditional READY_LIVE classification only for a new real-network question

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