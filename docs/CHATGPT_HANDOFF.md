# ChatGPT reviewer handoff — H-R9-065 terminal-state escape is front HIGH; R9-6 blocked

## Current repository truth

- Latest developer-owned source/test commit: exact `dcbcf55160e918180c3a5e96259bffcefef34e44` (`fix(carrier): H-R9-064 teardown quiesces live Recovery ownership`).
- Current independent reviewer anchor: [`docs/reviews/independent-r9-6-terminal-state-20260919.md`](reviews/independent-r9-6-terminal-state-20260919.md), added by reviewer commit `ef80586abe1ebaff63093ce19b349e3f3652cafa` after reviewing reachable `main` at `62bab736bca59393e6a16d4eaf2b9a59acd71e03`.
- **H-R9-064 narrow sender live-owner defect is accepted as repaired at exact `dcbcf55`:** teardown now reconciles sender Recovery in-flight / Reno bytes / packet->frame / retained plaintext to zero, and empty `pto_probe()` no longer mutates PTO state.
- **H-R9-065 is open HIGH:** the same teardown is not a terminal runtime transition. It leaves receiver `PacketAckTracker` alive, so a pre-teardown pending ACK can still be emitted afterward; it also replaces `PathRecovery` with a fresh owner for the same path/generation, losing the committed packet-number high-water so `on_packet_sent` can recreate live ownership (including reuse of a previously committed packet number) after teardown. The runtime has no terminal/quiesced owner bit or equivalent gate.
- H-R9-065 also exposes an evidence-boundary problem: replacing `PathRecovery` zeros lifetime diagnostics (`packets_sent`, `packets_lost`, RTT/PTO/outcome counters) instead of clearly separating historical facts from current live ownership. Already-emitted diagnostics remain immutable, but a later terminal summary consulting the runtime can no longer distinguish quiesced history from “never happened.”
- **H-R9-062 remains independently closed at exact `ad2296db56230a6ec8665273b15b8aa82536f9fb`.** `--fail-r9-first-send` exercises the production post-return first-send owner/error branch and proves typed socket failure with no positive first-send/settlement evidence.
- **H-R9-063 remains independently closed at exact `2b4abdfe23f1beffc0f72a8d551d4f6b8e08276d`.** First-send abort releases retained plaintext only when the aborted packet was the final Recovery owner; sibling copies preserve the needed bytes.
- Developer-local clean exact-tree provenance for exact `dcbcf55`: `PYTHONDONTWRITEBYTECODE=1 bash scripts/check.sh` exit 0, `git diff --check` exit 0, clean pushed worktree, Linux x86_64, rustc 1.98.0.
- GitHub-hosted Rust CI for exact `dcbcf55`, run `35403232907`, completed successfully: stable checks and nightly pinned decode fuzz smoke both green. Hosted CI is cross-evidence only and does not contain the H-R9-065 terminal discriminator.
- Reviewer-local execution is **not claimed**. The reviewer container could not resolve `github.com`; source inspection, developer-persisted local provenance, and hosted CI remain separate evidence classes.
- **R9-4 / H-R9-060 remain independently closed** at source/test tree `95d9388`; **R9-5 remains closed no-finding** at `b9f0dc5467771f2398d06f2f6a66ebb3a36c1111`.
- Earlier accepted findings remain closed absent contradictory current evidence: Candidate A future/never-sent ACK guard; Candidate B mixed datagram-drop observability; H-R9-050 exact-wire retransmit admission; H-R9-051 retransmit socket rollback; H-R9-052/H-R9-053 committed ACK-valid watermark restoration; H-R9-054 committed-watermark reuse discriminator; H-R9-055 executable pre-deadline PTO owner; H-R9-057 one-shot ACK-delay/reorder; H-R9-058 exact Session ACK witness; H-R9-059 operation-owned bounded witness; H-R9-060 typed R9-4 accepted-empty evidence; H-R9-061 first-send socket transaction; H-R9-062 production-owner fault-injection oracle; H-R9-063 first-send retained-plaintext cleanup.
- `READY_LIVE: none`; release item 3 incomplete; release item 4 incomplete; `RELEASE_CANDIDATE=false`; `PRODUCTION_READY=false`; `FREEZE=false`; `RELEASED=false`.

The external coding agent must synchronize to current `main` and continuously execute every dependency-ready slice below: implementation/review -> focused deterministic tests -> commit -> push -> clean exact-tree local gate/provenance -> next slice. Reviewer cadence is only a check frequency and is never a work-ticket length or reason to idle.

# READY_LOCAL 1 — H-R9-065 / R9-6 terminal-state escape — FRONT HIGH

Repair the concrete terminal-lifecycle defect without inventing new policy values or changing core Session/Carrier/ACK/crypto/wire architecture.

Required contract:

1. Give `ReliableUdpRuntime` one authoritative terminal/quiesced state (or equivalent single-owner mechanism). `teardown()` must not mean “fresh same-generation runtime.”
2. Preserve the accepted H-R9-064 sender cleanup: after teardown, Recovery sent/outstanding packet copies, Reno bytes/per-packet charge, `packet_frames`, and retained plaintext are zero.
3. Reconcile receiver ACK ownership too. A pending pre-teardown `PacketAckTracker` obligation must not survive and `poll_outgoing_ack` must not emit a fresh ACK from the terminated runtime.
4. After teardown, data/feedback mutators must be fail-closed or observationally inert. Challenge at least:
   - `can_send` / `on_packet_sent`;
   - `on_retransmit_sent`;
   - `on_packet_received`;
   - `poll_outgoing_ack`;
   - `apply_ack`;
   - `pto_probe`;
   - `poll_health`.
   No new packet/frame/plaintext ownership, ACK emission, PTO/loss/health transition, Session/Carrier success evidence, or active send eligibility may appear.
5. Do not lose the committed packet-number invariant. A previously committed packet number must not become reusable merely because teardown replaced sender Recovery; a fresh packet number must likewise not reanimate a terminal runtime. `PathRecovery` documents the packet number as the authenticated `SecureSession` record sequence / AEAD-nonce identity; do not weaken that fail-closed boundary.
6. Preserve historical diagnostics separately from current live ownership wherever existing public accessors/terminal diagnostics treat them as history. Do not make `packets_sent`/loss/PTO history silently become “never happened” merely to obtain quiescence.
7. Add one deterministic terminal discriminator on the real runtime owner:
   - before teardown, commit at least one sender packet and create one pending receiver ACK obligation;
   - assert nonzero live ownership and capture historical diagnostic state;
   - teardown;
   - assert current ownership is zero;
   - assert `poll_outgoing_ack` emits nothing;
   - assert both reuse of the old committed PN and a fresh PN cannot recreate ownership on this terminal runtime;
   - assert post-teardown receive/ACK/PTO/health calls create no new transition/evidence;
   - if lifetime diagnostic accessors remain public, assert the historical facts stay truthful.
8. Add a paired non-terminal control proving an ordinary runtime still accepts strictly increasing packet numbers, emits receiver ACKs, settles ACK/loss, and releases owners normally.
9. Re-audit executable terminal/error exits that retain a `ReliableUdpRuntime` after cleanup. They must not consult a terminal runtime for new success/health/recovery evidence. Prefer one runtime-owned terminal transition, not caller-side duplicated cleanup flags.
10. Preserve H-R9-061/H-R9-062/H-R9-063 semantics and the repaired H-R9-064 live-owner zero. `--drop-r9-data` remains deliberate Recovery-owned controlled loss; first-send/retransmit socket abort remains transactional; aborted reservations remain non-ACK-valid.
11. No new TTL/LRU/history-size/capacity/security values, no D019 change, no `SessionRuntime.events` retention decision, no release-authority change.
12. On the final pushed repair SHA, run and persist:

```text
PYTHONDONTWRITEBYTECODE=1 bash scripts/check.sh
git diff --check
```

Record exact reachable pushed SHA, UTC start/end, exit codes, OS/arch, stable Rust version and clean-tree state. Fuzz is not mechanically required unless the repair actually changes wire decoder/parser/crypto framing.

On closure, continue immediately to READY_LOCAL 2 without waiting for reviewer cadence.

# READY_LOCAL 2 — R9-6 remaining ownership / resource boundedness completion

After H-R9-065 closes, complete the rest of R9-6 as a bounded independent challenge rather than assuming the terminal repair covers every ownership seam.

Minimum scope:

- first-send and retransmit packet-copy ownership cannot survive refusal/abort/terminal cleanup incorrectly;
- retransmit exact-wire admission remains truthful;
- committed packet numbers are never reused and aborted reservations never become ACK-valid history;
- retained plaintext/frame ownership remains tied to a real Recovery/retry owner and releases deterministically on ACK/loss/final teardown;
- packet-to-frame maps, Reno bytes-in-flight, Recovery sent/outstanding state and retained plaintext reconcile after positive and negative outcomes;
- receiver-side pending ACK/range ownership has a truthful terminal lifetime and cannot emit post-terminal feedback;
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

H-R9-062/H-R9-063/H-R9-064/H-R9-065 production socket-success/failure, retained-ownership and terminal boundaries must be included. No diagnostic may manufacture evidence in another domain. Add only discriminators that would actually fail if the production owner regressed; do not generate schema/checker filler.

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
- no post-terminal retransmit, delivery confirmation, Carrier ACK emission, health-driven promotion or new Data ownership;
- listener/socket/session/recovery/Reno/ACK-tracker/retained-plaintext state is deterministically cleaned on normal and error exits;
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

**Not READY. `READY_LIVE: none`.** Standing authorization remains valid, but H-R9-065 and the current R9 queue are deterministic local correctness/evidence work and create no unresolved real-network question that loopback/process evidence cannot answer. Do not repeat HY2, warm failover, periodic/soak, package lifecycle, migration-back, endpoint/key migration, IPv6, PLPMTUD or Experimental Track evidence merely because the VPS remains rented.

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
