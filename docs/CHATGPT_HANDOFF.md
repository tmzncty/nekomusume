# ChatGPT reviewer handoff — H-R9-066 closed at 788a4dc; R9-6 blocked on reviewer closure

## Current repository truth

- Latest developer-owned source/test commit: exact `788a4dcbb4ab802f1444991d7898ec48ed7a9030` (`fix(carrier): H-R9-066 terminal teardown preserves history + gates control plane`), on top of `b5e7bc90599adca19c1f40e7e7be87b0728c8980`.
- Current independent reviewer anchor: [`docs/reviews/independent-r9-6-terminal-history-control-20260919.md`](reviews/independent-r9-6-terminal-history-control-20260919.md), added by exact `602e6a55cd14f1ac66dbf6329c9e47c31389473b`.
- **H-R9-064 remains closed at exact `dcbcf55`:** terminal cleanup reconciles sender Recovery in-flight / Reno bytes / packet->frame / retained plaintext to zero and post-cleanup PTO is inert.
- **H-R9-065 narrow packet/recovery guard is accepted at exact `b5e7bc9`:** `torn_down` blocks the reviewed data/feedback methods (`can_send`, send/retransmit admission, receive ACK obligation, outgoing ACK, recovery ACK, PTO/health mutation).
- **H-R9-066 closed:** `Recovery::quiesce`/`PathRecovery::quiesce` clears only LIVE ownership (sent/outstanding/acked/reservations/reno bytes/charge) while lifetime diagnostics (`packets_sent`/`lost`, `rtt`, `pto_count`, resolved counters, cwnd history) stay truthful — terminal summary is not "never happened". `ready_standby`/`activate_udp` are no-ops and `manager_mut` returns `None` after teardown — no new readiness/activation/switch control state escapes. `teardown_preserves_lifetime_history_and_gates_control_plane` proves `packets_sent`/`lost` preserved after teardown and control mutators inert.
- Developer-local clean exact-tree provenance for exact `788a4dc`: `PYTHONDONTWRITEBYTECODE=1 bash scripts/check.sh` exit 0, `git diff --check` exit 0, clean worktree at pushed SHA, 2026-09-19T00:55:55Z → 2026-09-19T01:00:41Z, Linux x86_64, rustc 1.98.0.
- GitHub-hosted Rust CI run `35407172293` for exact `b5e7bc9` completed successfully: stable checks and nightly pinned decode fuzz smoke both green. Hosted CI is cross-evidence only and does not contain the H-R9-066 discriminator.
- Reviewer-local execution is not claimed in this pass; the finding is source/test/spec review plus persisted developer/hosted evidence separation.
- **R9-4 / H-R9-060 remain independently closed** at source/test tree `95d9388`; **R9-5 remains closed no-finding** at `b9f0dc5467771f2398d06f2f6a66ebb3a36c1111`.
- Earlier accepted findings remain closed absent contradictory current evidence: Candidate A future/never-sent ACK guard; Candidate B mixed datagram-drop observability; H-R9-050 exact-wire retransmit admission; H-R9-051 retransmit socket rollback; H-R9-052/H-R9-053 committed ACK-valid watermark restoration; H-R9-054 committed-watermark reuse discriminator; H-R9-055 executable pre-deadline PTO owner; H-R9-057 one-shot ACK-delay/reorder; H-R9-058 exact Session ACK witness; H-R9-059 operation-owned bounded witness; H-R9-060 typed R9-4 accepted-empty evidence; H-R9-061 first-send socket transaction; H-R9-062 production-owner fault-injection oracle; H-R9-063 first-send retained-plaintext cleanup; H-R9-064 live-owner teardown cleanup.
- `READY_LIVE: none`; release item 3 incomplete; release item 4 incomplete; `RELEASE_CANDIDATE=false`; `PRODUCTION_READY=false`; `FREEZE=false`; `RELEASED=false`.

The external coding agent must synchronize to current `main` and continuously execute every dependency-ready slice below: implementation/review -> focused deterministic tests -> commit -> push -> clean exact-tree local gate/provenance -> next slice. Reviewer cadence is only a check frequency and is never a work-ticket length or reason to idle.

# READY_LOCAL 1 — H-R9-066 / R9-6 terminal history + control-plane closure — FRONT HIGH

Repair the exact-current terminal-lifecycle defects without changing Session/Carrier/ACK/crypto/wire architecture or inventing policy values.

Required contract:

1. Preserve H-R9-064/H-R9-065 live-owner zero and terminal packet/recovery guards.
2. Separate **current live Recovery ownership** from **lifetime historical diagnostics**. After teardown, `in_flight`, Reno bytes/charge, packet->frame, retained plaintext and pending ACK ownership remain zero, but facts that occurred before teardown must not be rewritten to a fresh-zero history.
3. Existing public lifetime diagnostics (`packets_sent`, `packets_lost`, and any terminal/result projection of RTT/PTO/resolved recovery history) must remain truthful after teardown. Do not make “historically sent/lost/PTO-observed, now terminal” indistinguishable from “never happened.”
4. Preserving history must not re-open live Recovery: no packet-number admission, ACK validity, retransmit, PTO, health or cwnd state may become live merely to keep counters.
5. Make the runtime-owned Carrier-manager control surface terminal-aware. After teardown, `ready_standby`, activation/promotion setup, and mutable-manager access reachable through `ReliableUdpRuntime` must be fail-closed or observationally inert. A torn-down runtime must not acquire new readiness/warm/active/switch state or positive Carrier control evidence.
6. Prefer one runtime-owned terminal gate. Do not scatter caller-side cleanup flags. If raw `manager_mut` cannot preserve the invariant safely, narrow/replace that exposure with the smallest typed/gated API required by current callers; do not change D064 states, thresholds or meanings.
7. Deterministic terminal discriminator on the real owner:
   - before teardown, commit at least one sender packet, create a pending receiver ACK obligation, and create nonzero lifetime diagnostic state (include PTO/loss history if the current accessor contract exposes it);
   - capture lifetime diagnostics and manager state;
   - teardown;
   - assert current transport ownership is zero and packet/recovery methods remain terminal;
   - assert historical diagnostic facts remain truthful and equal to the pre-teardown history rather than zeroed;
   - attempt post-terminal readiness/activation/control mutation through every supported public runtime surface and assert no new manager state/evidence appears;
   - repeated teardown must be idempotent and must not rewrite history again.
8. Paired non-terminal control: ordinary live runtime still admits strictly increasing packets, emits ACKs, performs readiness/activation through the supported control surface, settles recovery, and releases ownership normally.
9. Re-audit executable terminal/error exits for final diagnostic/result reads. Terminal summaries may read historical facts but must not manufacture fresh terminal runtime evidence.
10. Preserve committed packet-number non-reuse, H-R9-061/H-R9-062 socket rollback, H-R9-063 retained-plaintext cleanup and deliberate `--drop-r9-data` Recovery-owned loss semantics.
11. No D019 change, no `SessionRuntime.events` retention/capacity decision, no new TTL/LRU/history-size/capacity/security value, no signing/key-custody/SBOM/publication decision, no release-authority change.
12. On the final pushed repair SHA run and persist:

```text
PYTHONDONTWRITEBYTECODE=1 bash scripts/check.sh
git diff --check
```

Record exact reachable pushed SHA, UTC start/end, exit codes, OS/arch, stable Rust version and clean-tree state. Fuzz is not mechanically required unless the repair actually changes wire decoder/parser/crypto framing.

On closure, continue immediately to READY_LOCAL 2 without waiting for reviewer cadence.

# READY_LOCAL 2 — R9-6 remaining ownership / resource boundedness completion

After H-R9-066 closes, finish the bounded independent R9-6 challenge rather than assuming terminal repair covers every seam.

Minimum scope:

- first-send and retransmit packet-copy ownership cannot survive refusal/abort/terminal cleanup incorrectly;
- retransmit exact-wire admission remains truthful;
- committed packet numbers are never reused and aborted reservations never become ACK-valid history;
- retained plaintext/frame ownership remains tied to a real Recovery/retry owner and releases deterministically on ACK/loss/final teardown;
- packet-to-frame maps, Reno bytes-in-flight, Recovery sent/outstanding state and retained plaintext reconcile after positive and negative outcomes;
- receiver-side pending ACK/range ownership has a truthful terminal lifetime and cannot emit post-terminal feedback;
- retransmit socket-abort with overlapping sibling copies preserves exactly the stable plaintext still needed, no more and no less;
- repeated bounded first-send aborts do not accumulate orphan ownership;
- historical classification/diagnostic metadata remains bounded/accounted by existing owners and is not silently reset by cleanup.

If a concrete correctness/security/evidence defect appears, repair the smallest current-semantics seam with discriminating positive/negative tests and exact-tree gate. If no defect appears, persist a scope-exact bounded no-finding note and continue immediately.

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
- historical terminal diagnostic snapshot vs current live-owner zero;
- readiness/warm/active/promotion control evidence;
- residual-domain failure;
- terminal result/settlement.

H-R9-062/H-R9-063/H-R9-064/H-R9-065/H-R9-066 production socket-success/failure, retained-ownership, terminal-history and control-plane boundaries must be included. No diagnostic may manufacture evidence in another domain. Add only discriminators that would actually fail if the production owner regressed; do not generate schema/checker filler.

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
- failed/incomplete readiness cannot emit positive promotion evidence or steal active ownership;
- a torn-down UDP runtime cannot manufacture fresh readiness/control evidence.

No policy-value changes.

# READY_LOCAL 6 — R9-9 health + promotion

Challenge integrated resolved reliable-UDP outcomes -> Carrier health/hysteresis -> promotion:

- recoverable reliable-UDP loss/PTO that successfully settles must stay on UDP rather than spuriously promote TCP;
- terminated/torn-down UDP state must not generate fresh health/readiness/promotion input;
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
- no post-terminal retransmit, delivery confirmation, Carrier ACK emission, readiness mutation, health-driven promotion or new Data ownership;
- listener/socket/session/recovery/Reno/ACK-tracker/retained-plaintext state is deterministically cleaned on normal and error exits;
- cleanup observations do not rewrite historical failure/recovery evidence.

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

**Not READY. `READY_LIVE: none`.** Standing authorization remains valid, but H-R9-066 and the current R9 queue are deterministic local correctness/evidence work and create no unresolved real-network question that loopback/process evidence cannot answer. Do not repeat HY2, warm failover, periodic/soak, package lifecycle, migration-back, endpoint/key migration, IPv6, PLPMTUD or Experimental Track evidence merely because the VPS remains rented.

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
