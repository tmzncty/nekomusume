# ChatGPT reviewer handoff — H-R9-064 closed at dcbcf55; R9-6 blocked on reviewer closure

## Current repository truth

- Latest developer-owned source/test commit: exact `dcbcf55160e918180c3a5e96259bffcefef34e44` (`fix(carrier): H-R9-064 teardown quiesces live Recovery ownership`), on top of `2b4abdfe23f1beffc0f72a8d551d4f6b8e08276d`.
- Current independent reviewer anchor: [`docs/reviews/independent-r9-6-teardown-ownership-20260919.md`](reviews/independent-r9-6-teardown-ownership-20260919.md), added by reviewer commit `8109dff340ae06133326df004ec63e7b3574bae0` after reviewing reachable `main` at `b017700582398eb78942ca8ee205c6d50e439341`.
- **H-R9-062 remains independently closed at exact `ad2296db56230a6ec8665273b15b8aa82536f9fb`.** `--fail-r9-first-send` injects a socket error after production post-return reliable-UDP admission into the same executable owner/error branch used by real `UdpSocket::send_to`; the process regression proves typed failure and absence of positive first-send/settlement evidence.
- **H-R9-063 is now independently closed at exact `2b4abdf`.** `ReliableUdpRuntime::abandon_sent` releases retained plaintext only when the aborted packet copy was the last Recovery owner of that frame. The sibling negative proves an overlapping surviving packet copy keeps plaintext; a single-copy abort releases the orphan.
- **H-R9-064 closed:** `ReliableUdpRuntime::teardown` now replaces `PathRecovery` with a fresh owner for this path/generation instead of only clearing `packet_frames` and `RetransmitBuffer`. After teardown, Recovery in-flight, Reno bytes/charge, sent-map and retained plaintext are all zero; post-teardown `pto_probe` is observationally inert and cannot manufacture new PTO/health evidence. The regression asserts zero ownership and zero post-teardown PTO mutation.
- Developer-local clean exact-tree provenance for exact `dcbcf55`: `PYTHONDONTWRITEBYTECODE=1 bash scripts/check.sh` exit 0, `git diff --check` exit 0, clean worktree at pushed SHA, 2026-09-18T22:51:07Z → 2026-09-19T06:55:00+08:00, Linux x86_64, rustc 1.98.0.
- GitHub-hosted Rust CI for exact `2b4abdf`, run `35398511814`, completed successfully. This is cross-evidence only; it does not contradict H-R9-064 because the current teardown oracle does not assert Recovery/Reno/PTO quiescence.
- Reviewer-local execution is **not claimed**. Exact pushed source/test inspection, developer-persisted local provenance and hosted CI remain separate evidence classes.
- **R9-4 / H-R9-060 remain independently closed** at source/test tree `95d9388`; **R9-5 remains closed no-finding** at `b9f0dc5467771f2398d06f2f6a66ebb3a36c1111`.
- Earlier accepted findings remain closed absent contradictory current evidence: Candidate A future/never-sent ACK guard; Candidate B mixed datagram-drop observability; H-R9-050 exact-wire retransmit admission; H-R9-051 retransmit socket rollback; H-R9-052/H-R9-053 committed ACK-valid watermark restoration; H-R9-054 committed-watermark reuse discriminator; H-R9-055 executable pre-deadline PTO owner; H-R9-057 one-shot ACK-delay/reorder; H-R9-058 exact Session ACK witness; H-R9-059 operation-owned bounded witness; H-R9-060 typed R9-4 accepted-empty evidence; H-R9-061 first-send socket transaction; H-R9-062 production-owner fault-injection oracle; H-R9-063 first-send retained-plaintext cleanup.
- `READY_LIVE: none`; release item 3 incomplete; release item 4 incomplete; `RELEASE_CANDIDATE=false`; `PRODUCTION_READY=false`; `FREEZE=false`; `RELEASED=false`.

The external coding agent must synchronize to current `main` and continuously execute every dependency-ready slice below: implementation/review -> focused deterministic tests -> commit -> push -> clean exact-tree local gate/provenance -> next slice. Reviewer cadence is only a check frequency and is never a work-ticket length or reason to idle.

# READY_LOCAL 1 — H-R9-064 / R9-6 terminal teardown ownership — FRONT HIGH

Repair the concrete terminal-lifecycle defect without inventing new policy values or changing core Session/Carrier/ACK/crypto/wire architecture.

Required contract:

1. Make `ReliableUdpRuntime::teardown` a true quiescent terminal ownership transition for the current runtime/path generation. After teardown, current live state must reconcile to zero: Recovery in-flight packets/outstanding-frame copies, `packet_frames`, Reno bytes-in-flight/per-packet charge, and retained plaintext ownership.
2. Preserve historical facts. Do not erase or rewrite already-emitted loss/failure diagnostics merely to obtain zero ownership; distinguish current live ownership from historical counters/evidence.
3. After terminal teardown, `pto_probe`, recovery ACK application, health polling, or related paths must be fail-closed or observationally inert. They must not create fresh PTO/loss/ACK/Carrier-health/success evidence derived from pre-teardown packets.
4. Strengthen or replace `retransmit_requires_retained_ownership_and_teardown_is_deterministic`. The current oracle is insufficient because `pto_probe().is_empty()` can be true solely because plaintext was cleared. New deterministic assertions must establish:
   - before teardown, overlapping original/retransmit copies produce nonzero `in_flight` and congestion bytes;
   - after teardown, `in_flight == 0`, congestion bytes are zero, packet->frame ownership is gone, retained frames/bytes are zero;
   - a post-teardown PTO/ACK/health call cannot mutate PTO/outcome/health state or manufacture a positive transition.
5. Include a paired normal-lifecycle control: ordinary ACK/loss settlement must still retire exactly the correct packet/frame/plaintext/Reno owners and must not rely on teardown to mask leaks.
6. Re-audit executable terminal/error exits that own `ReliableUdpRuntime`. They must either perform the correct terminal cleanup or drop the owner without later consulting stale recovery state for success/health/diagnostic claims. Prefer one runtime-owned cleanup transition rather than caller-side duplicated field cleanup.
7. Preserve H-R9-061/H-R9-062/H-R9-063 semantics: first-send socket abort remains transactional; `--drop-r9-data` remains deliberate Recovery-owned loss; retransmit socket abort keeps stable plaintext while a legitimate retry owner exists; committed packet numbers are never reused; aborted reservations remain non-ACK-valid.
8. No new TTL/LRU/history-size/capacity/security values, no D019 change, no `SessionRuntime.events` retention decision, no release-authority change.
9. On the final pushed repair SHA, run and persist:

```text
PYTHONDONTWRITEBYTECODE=1 bash scripts/check.sh
git diff --check
```

Record exact reachable pushed SHA, UTC start/end, exit codes, OS/arch, stable Rust version and clean-tree state. This finding does not require fuzz unless the actual repair touches wire decoder/parser/crypto framing.

On closure, continue immediately to READY_LOCAL 2 without waiting for reviewer cadence.

# READY_LOCAL 2 — R9-6 remaining ownership / resource boundedness completion

After H-R9-064 closes, complete the rest of R9-6 as a bounded independent challenge rather than assuming one teardown repair covers every ownership seam.

Minimum scope:

- first-send and retransmit packet-copy ownership cannot survive refusal/abort/terminal cleanup incorrectly;
- retransmit exact-wire admission remains truthful;
- committed packet numbers are never reused and aborted reservations never become ACK-valid history;
- retained plaintext/frame ownership remains tied to a real Recovery/retry owner and releases deterministically on ACK/loss/final teardown;
- packet-to-frame maps, Reno bytes-in-flight, Recovery sent/outstanding state and retained plaintext reconcile after positive and negative outcomes;
- retransmit socket-abort with overlapping sibling copies preserves exactly the stable plaintext still needed, no more and no less;
- repeated bounded first-send aborts do not accumulate orphan ownership;
- historical classification metadata remains bounded/accounted by existing owners.

If a concrete correctness/security/evidence defect appears, repair the smallest current-semantics seam with discriminating positive/negative tests and exact-tree gate. If no defect appears, persist a precise bounded no-finding note and continue immediately.

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
- retained-ownership release vs packet-copy retirement vs terminal teardown;
- residual-domain failure;
- terminal result/settlement.

H-R9-062/H-R9-063/H-R9-064 production socket-success/failure, retained-ownership and teardown boundaries must be included. No diagnostic may manufacture evidence in another domain. Add only discriminators that would actually fail if the production owner regressed; do not generate schema/checker filler.

# READY_LOCAL 4 — R9 mid-slice factual reconciliation

After the coherent R9-4/R9-5/R9-6/R9-7 group is closed, perform one small factual reconciliation of release/item-4 state before continuing. Do not rewrite the full release packet yet.

Check only current facts:

- exact reachable R9 anchors and evidence classes;
- item 3 remains incomplete unless repository truth changed independently;
- item 4 remains incomplete until the later dedicated R9 review/reconciliation;
- `READY_LIVE` remains `none` unless new code creates a concrete unresolved real-network question;
- release/freeze/production flags remain unchanged.

# READY_LOCAL 5 — R9-8 warm TCP readiness

Challenge D064 single-active / multi-ready semantics on the materially new cross-process integration:

- authenticated resume-bound warm standby carries readiness/control only;
- no application Data is owned/sent on TCP before promotion;
- readiness is not inferred from TCP connect, UDP packet feedback or Session DeliveryAck;
- readiness resource admission/generation/session/epoch binding remains exact;
- failed/incomplete readiness cannot emit positive promotion evidence or steal active ownership.

No policy-value changes.

# READY_LOCAL 6 — R9-9 health + promotion

Challenge integrated resolved reliable-UDP outcomes -> Carrier health/hysteresis -> promotion:

- recoverable reliable-UDP loss/PTO that successfully settles must stay on UDP rather than spuriously promote TCP;
- terminated/torn-down UDP state must not generate fresh health samples or promotion input;
- actual promotion requires existing readiness + health/hysteresis gates;
- draining/failed UDP cannot receive new Session Data ownership;
- single-active invariant holds through transition;
- health/decision evidence remains distinct from packet feedback and Session delivery evidence.

# READY_LOCAL 7 — R9-10 uncertain replay + dedup + cleanup

Challenge failover after a genuinely unresolved UDP logical range:

- only genuinely uncertain Session ranges replay on promoted TCP;
- stable Session/stream/offset identity is preserved;
- receiver dedup remains exactly-once and conflicting bytes fail closed;
- TCP path does not grow a duplicate UDP packet-ACK layer;
- resolved UDP ranges do not replay;
- shutdown/error/partial-promotion negatives clean Recovery/packet/plaintext ownership and emit no false success.

Do not change core replay architecture. Any ambiguity requiring such a change is a maintainer/architecture gate while independent local lanes continue.

# READY_LOCAL 8 — R9-11 lifecycle / terminal invariants

Close remaining lifecycle and terminal invariants for the cross-process reliable-UDP/failover integration:

- exactly one terminal success/failure classification per bounded operation;
- intermediate timeout/residual diagnostics are not terminal success/failure;
- no post-terminal retransmit, delivery confirmation, health-driven promotion or new Data ownership;
- listener/socket/session/recovery/Reno/retained-plaintext state is deterministically cleaned on normal and error exits;
- cleanup observations do not rewrite historical failure evidence.

# READY_LOCAL 9 — R9-12 complete cross-process slice + final exact-tree provenance

After R9-6 through R9-11 close, run on the final pushed developer SHA in a safe clean checkout/worktree:

```text
PYTHONDONTWRITEBYTECODE=1 bash scripts/check.sh
git diff --check
```

Confirm clean tree and persist exact reachable pushed SHA, UTC start/end, OS/arch, stable Rust version, exit codes and clean-tree state. Keep developer-local provenance distinct from hosted CI and reviewer execution. Run pinned decode fuzz only if accumulated R9 work actually changes wire decoder/parser/crypto framing.

Do not rewrite historical live evidence.

# READY_LOCAL 10 — dedicated independent R9 review

Perform a fresh independent bounded challenge of the materially new R9 send/admission/reservation/socket-outcome/recovery/PTO/demux/Session-ACK/Carrier-ACK/failover/migration/cleanup/diagnostic behavior at the final R9 implementation anchor.

A no-finding review is valid item-4 support only when it names inspected owners, challenged invariants, focused deterministic tests/commands, exclusions, exact reachable anchor and evidence class. Any concrete correctness/security/evidence BLOCKER/HIGH returns immediately to smallest repair -> regression -> exact-tree gate.

# READY_LOCAL 11 — Q10/Q11/Q12 factual reconciliation

Only after a reachable independent R9 review anchor exists, reconcile `docs/status.md`, `docs/release-security-review-packet.md`, `IMPLEMENTATION_PLAN.md` and related Q10/Q11/Q12 evidence text against exact reachable implementation/review anchors.

Preserve evidence boundaries:

- release item 3 remains incomplete unless repository truth independently closes it;
- item 4 closes only to the extent supported by reachable independent review evidence;
- no historical WAN artifact rewrite;
- no automatic RC/freeze/release/production transition;
- policy/authority gates stay separate.

# READY_LOCAL 12 — repository-wide item-4 inventory refill if reconciliation remains blocked

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

**Not READY. `READY_LIVE: none`.** Standing authorization remains valid, but H-R9-064 and the current R9 queue are deterministic local correctness/evidence work and create no unresolved real-network question that loopback/process evidence cannot answer. Do not repeat HY2, warm failover, periodic/soak, package lifecycle, migration-back, endpoint/key migration, IPv6, PLPMTUD or Experimental Track evidence merely because the VPS remains rented.

Only create a new READY_LIVE row if later code/instrumentation/hypothesis/path conditions produce a concrete unresolved real-network question and the run stays inside `docs/standing-vps-lab-authorization.md`.

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

These gates do not block the independent dependency-ready local work above.
