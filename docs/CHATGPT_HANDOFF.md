# ChatGPT reviewer handoff — H-R9-067 closed at b5e3bcb; R9-7 blocked on reviewer closure

## Current repository truth

- Latest developer-owned source/test commit: exact `b5e3bcbc63edab6ecf049573ce08a54fe31d6686` (`test(cli): H-R9-067 terminal-result oracle — nonzero exit + no success evidence`), on top of `788a4dcbb4ab802f1444991d7898ec48ed7a9030`.
- Current independent reviewer anchor: [`docs/reviews/independent-r9-7-process-truth-788a4dc-20260919.md`](reviews/independent-r9-7-process-truth-788a4dc-20260919.md), added by exact `76d3e9e61c293d768e30925aa04aee7b0bebf96a`.
- **R9-6 remaining ownership/resource-boundedness review is CLOSED, bounded no-finding** at source/test anchor `788a4dc`, persisted by [`docs/reviews/independent-r9-6-ownership-boundedness-788a4dc-20260919.md`](reviews/independent-r9-6-ownership-boundedness-788a4dc-20260919.md), review commit `5450e9651855e1309bd7e09868f58b078fbce4a6`.
- **H-R9-067 closed:** `reliable_udp_first_send_socket_failure_rolls_back` now requires a nonzero client exit (`!status.success()`) and forbids `failover_client_ok`/`summary`/`ordered_records_complete` — a regression that emits residual evidence but then reports success turns red. Production `--fail-r9-first-send` path unchanged.
- Developer-local clean exact-tree provenance for exact `b5e3bcb`: `PYTHONDONTWRITEBYTECODE=1 bash scripts/check.sh` exit 0, `git diff --check` exit 0, clean worktree at pushed SHA, 2026-09-19T01:49:33Z → 2026-09-19T01:54:19Z, Linux x86_64, rustc 1.98.0.
- GitHub-hosted Rust CI for exact `788a4dc` is run `35410930506`, completed `success`. Hosted CI is cross-evidence only and cannot repair the H-R9-067 oracle.
- Reviewer-local execution is not claimed in this pass.
- Earlier accepted findings remain closed absent contradictory current evidence: Candidate A future/never-sent ACK guard; Candidate B mixed datagram-drop observability; H-R9-050 exact-wire retransmit admission; H-R9-051 retransmit socket rollback; H-R9-052/H-R9-053 committed watermark restoration; H-R9-054 committed-watermark reuse discriminator; H-R9-055 executable PTO deadline owner; H-R9-057 one-shot ACK-delay/reorder; H-R9-058 exact Session ACK witness; H-R9-059 operation-owned bounded witness; H-R9-060 typed duplicate-Session-ACK evidence; H-R9-061 first-send socket transaction; H-R9-062 production-owner injection; H-R9-063 retained-plaintext first-send abort cleanup; H-R9-064 live-owner teardown; H-R9-065 terminal packet/recovery guard; H-R9-066 history/control-plane closure.
- R9-4 / H-R9-060 remain independently closed; R9-5 remains bounded no-finding closed; R9-6 is now independently closed.
- `READY_LIVE: none`; release item 3 remains incomplete; release item 4 remains incomplete; `RELEASE_CANDIDATE=false`; `PRODUCTION_READY=false`; `FREEZE=false`; `RELEASED=false`.

The external coding agent must synchronize to current `main` and continuously execute every dependency-ready slice below: implementation/review -> focused deterministic tests -> commit -> push -> clean exact-tree local gate/provenance -> next slice. Reviewer cadence is only a check frequency and is never a work-ticket length or reason to idle.

# READY_LOCAL 1 — H-R9-067 / R9-7 first-send terminal-result oracle — FRONT HIGH

Close the concrete process-oracle gap without changing transport semantics.

Required contract:

1. Keep the real executable `--fail-r9-first-send` production-owner injection after Recovery/Reno/plaintext admission; do not replace it with a helper-only test.
2. In `reliable_udp_first_send_socket_failure_rolls_back`, explicitly assert the client exits nonzero. Pin code 2 only if the existing CLI exit contract already intentionally requires it; otherwise `!status.success()` is sufficient and avoids inventing policy.
3. Explicitly assert the failed run cannot emit final success/result evidence: no `failover_client_ok`, no final client `summary`, no `ordered_records_complete`, no `r9_udp_post_return_settled`.
4. Retain the existing ownership/evidence assertions: one typed `r9_udp_post_return_send_failed` for the failed production first send, no positive `r9_udp_post_return_sent`, no PTO/retransmit for the aborted copy, and zero Recovery in-flight in residual evidence when present.
5. Keep a production positive control using the same post-return owner: ordinary socket-success -> Session DeliveryAck + Carrier retirement -> exactly one settled event -> final process success. Existing positive migration-back/post-return tests may satisfy this if the closure note names the exact discriminator.
6. Do not weaken `--drop-r9-data`: that seam intentionally commits Recovery ownership and may legitimately drive PTO/retransmission to success.
7. This should normally be a tests-only/process-oracle repair. Do not add a generic result checker/schema/framework or redesign Session/Carrier/ACK/crypto/wire behavior merely to close the finding.
8. On the final pushed repair SHA run and persist:

```text
PYTHONDONTWRITEBYTECODE=1 bash scripts/check.sh
git diff --check
```

Record exact reachable pushed SHA, UTC start/end, exit codes, OS/arch, stable Rust version and clean-tree state. Fuzz is not mechanically required unless wire decoder/parser/crypto framing actually changes.

After closure, continue immediately to READY_LOCAL 2 without waiting for reviewer cadence.

# READY_LOCAL 2 — R9-7 remaining process / result truth

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
- retained-ownership release vs packet-copy retirement vs terminal teardown;
- historical terminal diagnostic snapshot vs current live-owner zero;
- readiness/warm/active/promotion control evidence;
- residual/intermediate timeout classification;
- terminal process success/failure and final settlement/result.

H-R9-062 through H-R9-067 production socket-success/failure, retained ownership, terminal history/control and process-result boundaries must be included. Test or source mutations should turn the appropriate oracle red; do not generate schema/checker filler. If a concrete defect appears, smallest repair + discriminating regression + exact-tree gate. If no further defect appears, persist a scope-exact bounded no-finding note and continue immediately.

# READY_LOCAL 3 — R9 mid-slice factual reconciliation

After R9-4/R9-5/R9-6/R9-7 are coherently closed, perform one small factual reconciliation rather than rewriting the full release packet.

Check only current facts:

- exact reachable R9 implementation/review anchors and evidence classes;
- release item 3 remains incomplete unless repository truth independently changes;
- item 4 remains incomplete until the later dedicated R9 review/reconciliation;
- `READY_LIVE` remains `none` unless new code/instrumentation creates a specific unresolved real-network question;
- RC/production/freeze/released flags remain unchanged.

# READY_LOCAL 4 — R9-8 warm TCP readiness

Challenge D064 single-active / multi-ready semantics on the materially new cross-process integration:

- authenticated resume-bound warm standby carries readiness/control only;
- no application Data is owned/sent on TCP before promotion;
- readiness is not inferred from TCP connect, UDP packet feedback or Session DeliveryAck;
- readiness resource admission/generation/session/epoch binding remains exact;
- failed/incomplete readiness cannot emit positive promotion evidence or steal active ownership;
- a torn-down UDP runtime cannot manufacture fresh readiness/control evidence.

No policy-value changes.

# READY_LOCAL 5 — R9-9 health + promotion

Challenge integrated resolved reliable-UDP outcomes -> Carrier health/hysteresis -> promotion:

- recoverable reliable-UDP loss/PTO that successfully settles must stay on UDP rather than spuriously promote TCP;
- terminated/torn-down UDP state must not generate fresh health/readiness/promotion input;
- actual promotion requires existing readiness + health/hysteresis gates;
- draining/failed UDP cannot receive new Session Data ownership;
- single-active invariant holds through transition;
- health/decision evidence remains distinct from packet feedback and Session delivery evidence.

# READY_LOCAL 6 — R9-10 uncertain replay + dedup + cleanup

Challenge failover after a genuinely unresolved UDP logical range:

- only genuinely uncertain Session ranges replay on promoted TCP;
- stable Session/stream/offset identity is preserved;
- receiver dedup remains exactly-once and conflicting bytes fail closed;
- TCP path does not grow a duplicate UDP packet-ACK layer;
- resolved UDP ranges do not replay;
- shutdown/error/partial-promotion negatives clean Recovery/packet/plaintext ownership and emit no false success.

Do not change core replay architecture. Any ambiguity requiring such a change is a maintainer/architecture gate while independent local lanes continue.

# READY_LOCAL 7 — R9-11 lifecycle / terminal invariants

Close remaining lifecycle and terminal invariants for the cross-process reliable-UDP/failover integration:

- exactly one terminal success/failure classification per bounded operation;
- intermediate timeout/residual diagnostics are not terminal success/failure;
- no post-terminal retransmit, delivery confirmation, Carrier ACK emission, readiness mutation, health-driven promotion or new Data ownership;
- listener/socket/session/recovery/Reno/ACK-tracker/retained-plaintext state is deterministically cleaned on normal and error exits;
- cleanup observations do not rewrite historical failure/recovery evidence.

# READY_LOCAL 8 — R9-12 complete cross-process slice + final exact-tree provenance

After R9-7 through R9-11 close, run on the final pushed developer SHA in a safe clean checkout/worktree:

```text
PYTHONDONTWRITEBYTECODE=1 bash scripts/check.sh
git diff --check
```

Confirm clean tree and persist exact reachable pushed SHA, UTC start/end, OS/arch, stable Rust version, exit codes and clean-tree state. Keep developer-local provenance distinct from hosted CI and reviewer execution. Run pinned decode fuzz only if accumulated R9 work actually changes wire decoder/parser/crypto framing. Do not rewrite historical live evidence.

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

Earlier reachable independent reviews cover established core implementations broadly; refill only genuinely implemented surfaces that still lack a dedicated reachable challenge or where R9 materially changed the owner. No checker/schema/framework/docs filler.

Repository-wide `queue exhausted` is permitted only when this broad inventory finds no concrete defect, no READY_LOCAL review/support lane, no READY_LIVE question, and every remaining item is truly policy/environment/release-authority gated.

## VPS opportunity

**Not READY. `READY_LIVE: none`.** Standing authorization remains valid, but H-R9-067 and the current R9 queue are deterministic local correctness/evidence work and create no unresolved real-network question that loopback/process evidence cannot answer. Do not repeat HY2, warm failover, periodic/soak, package lifecycle, migration-back, endpoint/key migration, IPv6, PLPMTUD or Experimental Track evidence merely because the VPS remains rented.

Only create a new READY_LIVE row if later code/instrumentation/hypothesis/path conditions produce a concrete unresolved real-network question and the run remains inside `docs/standing-vps-lab-authorization.md`.

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