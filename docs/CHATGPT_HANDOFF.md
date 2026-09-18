# ChatGPT reviewer handoff — R9-4 independently closed; R9-5 is front READY_LOCAL

## Current repository truth

- Current reviewer anchor: `docs/reviews/independent-r9-r9-4-closure-95d9388-20260919.md`, added by reviewer commit `8c21e24e436379abc13b41e4f163da61b5720193`.
- Latest developer-owned source/test commit reviewed: exact `95d938859bbe7a42c3e2bf26e09729ba55de5cf0`, on top of implementation/test commit `7ea9a8442563ef7d0dd7f6c70acdba53a003fdff`.
- New developer commits reviewed this pass, by class:
  - `7ea9a84` — **implementation + deterministic process test**: production post-return Session-demux typed classification is emitted instead of discarded; R9-4 observes `accepted_empty_logical_ack` and rejects `unexpected_logical_ack`.
  - `95d9388` — **test/evidence**: every positive client Carrier retirement packet number must be present in the server-emitted Carrier-ACK packet-number set.
  - `d4e19fd` — **docs/handoff only**; no runtime semantics.
- **H-R9-060 is independently closed.** The post-return operation starts with one admitted logical record and an empty operation-owned exact witness. The first genuine exact Session DeliveryAck populates `(stream, offset, len)`; the fresh same-range ACK produced after the real PTO/fresh-PN retransmission is visibly classified `accepted_empty_logical_ack`, produces no second positive Session transition, and `unexpected_logical_ack` is absent. The focused helper regression separately proves this exact accepted-empty branch leaves the malformed counter unchanged; subrange/interior and zero-length authenticated feedback remain fail-closed negatives.
- **R9-4 is independently closed** at exact source/test tree `95d9388`. The one-shot ACK-delay/reorder seam, real PTO + fresh-PN retransmission, exact Session duplicate classification, Carrier identity projection, exactly-one positive Session confirmation, and zero-in-flight terminal settlement were source/oracle challenged with no new BLOCKER/HIGH. See the reviewer anchor above for scope and exclusions.
- H-R9-059 remains closed: no Session-lifetime confirmed-range history; duplicate witness is caller-owned current-operation `BTreeSet<(stream,offset,len)>`.
- H-R9-058 remains closed: exact witness predicate; authenticated subrange/interior and zero-length feedback fail closed.
- H-R9-057 remains closed: `delay_reorder_done` makes `--delay-r9-ack-reorder` exactly one-shot.
- Earlier accepted work remains closed absent contradictory current repository evidence: H-R9-050 exact-wire retransmit admission; H-R9-051 retransmit socket-send rollback; H-R9-052/H-R9-053 committed ACK-valid watermark restoration; H-R9-054 committed-watermark reuse discriminator; H-R9-055 executable pre-deadline PTO owner; H-R9-040 lifecycle-scoped `acked_frames`; H-R9-043..049 repeated-PTO identity/deadline/retirement/source-projection; H-R9-038/H-R9-039 residual/server-exit; H-R9-036/H-R9-037 reverse-order exact oracle; H-R9-032..034 settlement; P2 C1-C4.
- Candidate A (future/never-sent ACK atomic rejection) remains closed. Candidate B (mixed queue/generic datagram-drop observability) remains closed.
- Developer-persisted exact-tree provenance for `95d9388`: `PYTHONDONTWRITEBYTECODE=1 bash scripts/check.sh` exit 0, `git diff --check` exit 0, clean worktree at pushed SHA, UTC 2026-09-18T17:58:47Z → 18:03:19Z, Linux x86_64, rustc 1.98.0.
- GitHub-hosted Rust CI run `35377470589` for exact `95d9388` is green: `stable checks` succeeded; nightly pinned decode fuzz build/run also succeeded. Hosted CI is cross-evidence only.
- Reviewer-local execution is **not** claimed for this pass. Reviewer truth is exact pushed source/test inspection + repository-persisted developer provenance + hosted cross-evidence, kept as distinct evidence classes.
- `READY_LIVE: none`; release item 3 incomplete; release item 4 incomplete; `RELEASE_CANDIDATE=false`; `PRODUCTION_READY=false`; `FREEZE=false`; `RELEASED=false`.

The external coding agent must synchronize to current `main` and continuously execute every dependency-ready slice below: implementation/review -> focused deterministic tests -> commit -> push -> exact-tree local gate/provenance -> next slice. Reviewer cadence is only a check frequency and is never a work-ticket length or reason to idle.

## Accepted R9-4 closure boundary

Do not reopen R9-4 for cosmetic oracle strengthening unless contradictory current evidence appears. The bounded closure establishes all of the following together:

- the server withholds only the first post-return Carrier ACK, then releases delayed-original before fresh-copy ACK and thereafter ACKs normally;
- the client reaches a real PTO and sends at least one fresh-PN retransmission through the exact-wire admission and socket-outcome transaction path;
- the first exact Session DeliveryAck populates the current-operation witness through the real demux, not by test prefill;
- the retransmission causes a freshly sealed same-range Session ACK and the real post-return caller visibly classifies it accepted-empty;
- exact accepted-empty is classification-only: no malformed charge in focused helper coverage and no second positive `r9_udp_return_delivery_ack` in the process regression;
- authenticated unadmitted/subrange/zero-length Session feedback remains fail-closed; identical authenticated-envelope replay remains a crypto replay rejection, not Session accepted-empty;
- every positive client Carrier retirement comes from Recovery `acked_packets` and is covered by the server's emitted Carrier-ACK identity set;
- terminal settlement occurs exactly once with `remaining_in_flight == 0` after both Session and Carrier domains are resolved.

This does **not** close the still-unreviewed adversarial continuations, first-send socket transaction, broad evidence truth, warm-readiness/promotion/replay/cleanup/lifecycle surfaces below.

# READY_LOCAL 1 — R9-5 adversarial Carrier feedback / every continuation

Challenge future/never-sent/stale/duplicate/tampered and multi-range Carrier feedback across **every current `UdpAcknowledgement::Carrier` continuation**, not only the R9-4 happy/reorder owner. Include initial receive, settlement and post-return owners.

Contract:

1. Read exact-current Carrier ACK decode/demux owner, `ReliableUdpRuntime` / Recovery ACK application, all caller continuations, and relevant prior ACK-review notes.
2. For each continuation distinguish:
   - positive retirement of currently Recovery-owned sent packet(s);
   - sent-and-retired historical identity accepted-empty/stale where current semantics permit;
   - future / never-sent identity rejected atomically;
   - pre-send reservation that was aborted and therefore must **not** become ACK-valid history;
   - malformed/tampered authenticated/wire failures at the correct layer.
3. Rejected feedback must leave RTT, PTO count/deadline, loss/retransmit eligibility, Reno/cwnd/in-flight, retained frame/plaintext ownership and Session state unchanged.
4. Multi-range ACK must project every actual retired packet, not collapse to a representative PN; accepted-empty cannot fabricate positive retirement.
5. Keep Session duplicate classification separate: Session accepted-empty may not substitute for Carrier accepted-empty or Carrier retirement evidence.
6. Use focused deterministic tests against the production owner. If current semantics already decide the answer, repair the smallest caller/runtime seam and add positive + negative regressions. Do not redesign ACK architecture.
7. If no concrete defect is found, persist a bounded independent no-finding note naming owners, tests/commands, exclusions and the exact reachable anchor; then continue immediately to R9-6.

Normal exact-tree gate applies to any source/test repair. Wire/parser/crypto framing fuzz is required only if those surfaces actually change.

# READY_LOCAL 2 — R9-6 ownership / resource boundedness

Challenge first-send and retransmit ownership as transactions rather than assuming H-R9-051 covers both.

Minimum scope:

- post-return **first-send** owner currently performs congestion admission + Recovery ownership before the real socket send; explicitly challenge socket-send failure ordering and whether Recovery/caller ownership / ACK-valid history / positive `r9_udp_post_return_sent` evidence can survive a real send failure;
- retransmit owner must keep exact-wire admission, rollback failed socket reservations, retain stable logical plaintext only as current semantics require, and never reuse a committed packet number;
- every first-send/retransmit path must be congestion-admitted before irreversible ownership commit or use an equivalent bounded reserve/commit/abort transaction;
- retained retransmit plaintext/frame ownership must be bounded by existing owners and deterministically released on ACK/loss/terminal cleanup;
- historical classification metadata must remain bounded/accounted by existing ownership; do not introduce TTL/LRU/history-size/capacity policy;
- no capacity-pressure benchmark and no new numeric policy.

A concrete first-send socket-outcome defect is correctness HIGH and goes to the front as smallest repair + injected failure/success regression + exact-tree gate.

# READY_LOCAL 3 — R9-7 process / result truth

Independently challenge that structured diagnostics correspond to the typed state transition/classification they claim. Keep these domains distinct:

- Data accepted/deduplicated;
- Carrier ACK positive retirement;
- Carrier accepted-empty/stale;
- Carrier rejected feedback;
- Session DeliveryAck positive confirmation;
- Session accepted-empty exact duplicate;
- Session unexpected/malformed feedback;
- PTO fired / retransmit sent;
- Recovery loss/ACK;
- successful vs failed socket send / aborted reservation;
- residual-domain failure;
- terminal result/settlement.

No diagnostic may manufacture evidence in another domain. H-R9-060's Session accepted-empty projection is part of this audit, not a substitute for the whole audit. Add only discriminators that can actually fail when the production owner regresses; do not generate schema/checker filler.

After R9-5 through R9-7 (or any coherent group of 3–4 slices), perform one small factual reconciliation of release/item-4 state before continuing; do not rewrite large docs after each microchange.

# READY_LOCAL 4 — R9-8 warm TCP readiness

Challenge D064 single-active / multi-ready semantics on the materially new cross-process integration:

- authenticated resume-bound warm standby carries readiness/control only;
- no application Data is owned/sent on TCP before promotion;
- readiness is not inferred from TCP connect, UDP packet feedback, or Session DeliveryAck;
- readiness resource admission/generation/session/epoch binding remains exact;
- failed/incomplete readiness cannot emit positive promotion evidence or steal active ownership.

No policy-value changes.

# READY_LOCAL 5 — R9-9 health + promotion

Challenge integrated resolved reliable-UDP outcomes -> Carrier health/hysteresis -> promotion:

- recoverable reliable-UDP loss/PTO that successfully settles must stay on UDP rather than spuriously promote TCP;
- actual promotion requires existing readiness + health/hysteresis gates;
- draining/failed UDP cannot receive new Session Data ownership;
- single-active invariant holds through the transition;
- health/decision evidence is distinct from packet feedback and Session delivery evidence.

# READY_LOCAL 6 — R9-10 uncertain replay + dedup + cleanup

Challenge failover after a genuinely unresolved UDP logical range:

- only genuinely uncertain Session ranges replay on promoted TCP;
- stable Session/stream/offset identity is preserved;
- receiver dedup remains exactly-once and conflicting bytes fail closed;
- TCP path does not grow a duplicate UDP packet-ACK layer;
- resolved UDP ranges do not replay;
- shutdown/error/partial-promotion negatives clean retained ownership and emit no false success.

Do not change core replay architecture; any ambiguity requiring such a change is a maintainer/architecture gate while independent lanes continue.

# READY_LOCAL 7 — R9-11 lifecycle / terminal invariants

Close the remaining lifecycle and terminal invariants for the new cross-process reliable-UDP/failover integration:

- exactly one terminal success/failure classification per bounded operation;
- intermediate timeout/residual diagnostics are not terminal success/failure;
- no post-terminal retransmit, delivery confirmation, promotion, or new Data ownership;
- listener/socket/session/recovery retained state is deterministically cleaned on normal and error exits;
- cleanup observations do not rewrite historical failure evidence.

# READY_LOCAL 8 — R9-12 complete cross-process slice + final exact-tree provenance

After R9-5 through R9-11 close, run on the final pushed developer SHA in a safe clean checkout/worktree:

```text
PYTHONDONTWRITEBYTECODE=1 bash scripts/check.sh
git diff --check
```

Confirm clean tree and persist exact reachable pushed SHA, UTC start/end, OS/arch, stable Rust version, exit codes and clean-tree state. Keep developer-local provenance distinct from hosted CI and reviewer execution. Run pinned decode fuzz only if the accumulated R9 work actually changes wire decoder/parser/crypto framing.

Do not rewrite historical live evidence.

# READY_LOCAL 9 — dedicated independent R9 review

Perform a fresh independent bounded challenge of the materially new R9 send/admission/reservation/socket-outcome/recovery/PTO/demux/Session-ACK/Carrier-ACK/failover/migration/cleanup/diagnostic behavior at the final R9 implementation anchor.

A no-finding review is valid item-4 support only when it names inspected owners, challenged invariants, focused deterministic tests/commands, exclusions, exact reachable anchor and evidence class. Any concrete correctness/security/evidence BLOCKER/HIGH returns immediately to smallest repair -> regression -> exact-tree gate.

# READY_LOCAL 10 — Q10/Q11/Q12 factual reconciliation

Only after a reachable independent R9 review anchor exists, reconcile `docs/status.md`, `docs/release-security-review-packet.md`, `IMPLEMENTATION_PLAN.md` and related Q10/Q11/Q12 evidence text against exact reachable implementation/review anchors.

Preserve evidence boundaries:

- release item 3 remains incomplete unless repository truth independently closes it;
- item 4 closes only to the extent supported by reachable independent review evidence;
- no historical WAN artifact rewrite;
- no automatic RC/freeze/release/production transition;
- policy/authority gates stay separate.

# READY_LOCAL 11 — repository-wide item-4 inventory refill if reconciliation remains blocked

If item 4 remains incomplete after R9 independent review/reconciliation, perform one repository-wide inventory against the mandatory core surfaces:

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
12. algorithmic boundedness;
13. release packet factual consistency/evidence boundary.

Earlier reachable independent reviews cover the established core implementations broadly; refill only genuinely implemented surfaces that still lack a dedicated reachable challenge or where R9 materially changed the owner. No checker/schema/framework/docs filler.

Repository-wide `queue exhausted` is permitted only when this broad inventory finds no concrete defect, no READY_LOCAL review/support lane, no READY_LIVE question, and every remaining item is truly policy/environment/release-authority gated.

## VPS opportunity

**Not READY. `READY_LIVE: none`.** Standing authorization remains valid, but the current R9 queue is deterministic local correctness/evidence work and produces no unresolved real-network question that loopback/process evidence cannot answer. Do not repeat HY2, warm failover, periodic/soak, package lifecycle, migration-back, endpoint/key migration, IPv6, PLPMTUD or Experimental Track evidence merely because the VPS remains rented.

Only create a new READY_LIVE row if later code/instrumentation/hypothesis/path conditions produce a concrete unresolved real-network question and the run stays inside `docs/standing-vps-lab-authorization.md`.

## Separate non-blocking maintainer / policy / authority gates

Do not decide or modify while executing this queue:

- `SessionRuntime.events` retention/capacity policy;
- D019 source-retention/no-reset policy or its numeric candidate values;
- RSEC-001 adversarial-load/capacity suitability conditions;
- signing/key-custody/SBOM/publication policy;
- previous frozen-release policy;
- core Session/Carrier/ACK/crypto/wire architecture;
- destructive/canonical-meaning migration;
- RC/freeze/release/production authority.

These gates do not block the independent dependency-ready local work above.