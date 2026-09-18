# ChatGPT reviewer handoff — H-R9-062 closed at ad2296d; R9-6 blocked on reviewer closure

## Current repository truth

- Latest developer-owned source/test commit: exact `ad2296db56230a6ec8665273b15b8aa82536f9fb` (`test(cli): H-R9-062 production-owner first-send socket-failure discriminator`), on top of `f8670a5a0ec95c4159e046fa12c4eec2177c63ff`.
- Current independent reviewer finding anchor: [`docs/reviews/independent-r9-6-first-send-production-oracle-20260919.md`](reviews/independent-r9-6-first-send-production-oracle-20260919.md), added by reviewer commit `9be0fb58fe2d54fd6f2abda7ca9aad073beb4900` from exact reviewed HEAD `b88c803a76b35bb30f163f231745437df616d892`.
- **H-R9-061 source repair remains the implementation baseline.** Exact `f8670a5` makes all three executable reliable-UDP first-send call sites check the real `UdpSocket::send_to` outcome: positive sent diagnostics occur only on `Ok`; `Err` calls `ReliableUdpRuntime::abandon_sent`. `abandon_sent` removes packet-to-frame state and routes through `PathRecovery::abandon_sent`, reversing Recovery/Reno/charged/`packets_sent` ownership and restoring the committed PN watermark. The deliberate `--drop-r9-data` seam remains intentionally Recovery-owned loss.
- **H-R9-062 closed:** `--fail-r9-first-send` injects a socket `Err` on the production post-return first-send owner — the same code path a real `Err` takes. `reliable_udp_first_send_socket_failure_rolls_back` proves `r9_udp_post_return_send_failed` emits, `r9_udp_post_return_sent` never escapes, `r9_udp_post_return_settled` absent (no fabricated Session transition), and the run terminates through residual/timeout.
- Developer-local clean exact-tree provenance for exact `ad2296d`: `PYTHONDONTWRITEBYTECODE=1 bash scripts/check.sh` exit 0, `git diff --check` exit 0, clean worktree at pushed SHA, 2026-09-18T20:50:24Z → 2026-09-19T04:55:00Z+8, Linux x86_64, rustc 1.98.0.
- GitHub-hosted Rust CI for exact `f8670a5`, run `35388578455`, is green: `stable checks` passed `bash scripts/check.sh`; nightly decode fuzz smoke also passed. Hosted CI is cross-evidence only and does not inject the missing production first-send socket failure.
- Reviewer-local execution is **not claimed**. The reviewer environment could not resolve `github.com` for a local clone/check. Exact pushed source/test inspection, developer-persisted local provenance, and hosted CI are separate evidence classes.
- **R9-4 / H-R9-060 remain independently closed** at exact source/test tree `95d9388`. The one-shot Carrier-ACK suppression, real PTO/fresh-PN retransmission, operation-owned exact Session duplicate witness, typed accepted-empty projection, Carrier packet-identity binding and zero-in-flight terminal settlement remain without contradictory current evidence.
- **R9-5 remains closed no-finding** at `b9f0dc5467771f2398d06f2f6a66ebb3a36c1111`: current future/never-sent, stale/duplicate, tampered, multi-range and aborted-reservation Carrier feedback continuations had reachable deterministic seams and remained atomic under the bounded audit.
- Earlier accepted findings remain closed absent contradictory current evidence: Candidate A future/never-sent ACK guard; Candidate B mixed datagram-drop observability; H-R9-050 exact-wire retransmit admission; H-R9-051 retransmit socket rollback; H-R9-052/H-R9-053 committed ACK-valid watermark restoration; H-R9-054 committed-watermark reuse discriminator; H-R9-055 executable pre-deadline PTO owner; H-R9-057 one-shot ACK-delay/reorder; H-R9-058 exact Session ACK witness; H-R9-059 operation-owned bounded witness; H-R9-060 typed R9-4 accepted-empty evidence.
- `READY_LIVE: none`; release item 3 incomplete; release item 4 incomplete; `RELEASE_CANDIDATE=false`; `PRODUCTION_READY=false`; `FREEZE=false`; `RELEASED=false`.

The external coding agent must synchronize to current `main` and continuously execute every dependency-ready slice below: implementation/review -> focused deterministic tests -> commit -> push -> clean exact-tree local gate/provenance -> next slice. Reviewer cadence is only a check frequency and is never a work-ticket length or reason to idle.

# READY_LOCAL 1 — H-R9-062 / R9-6A production first-send owner discriminator — FRONT HIGH

Do **not** redesign the rollback primitive that `f8670a5` already repaired unless a new concrete defect is found. Close the evidence/oracle gap against the executable production owner.

Required contract:

1. Preserve exact-wire congestion admission, `SecureSession` sequence/nonce as the single PN source, the current complete `abandon_sent` rollback, and existing retransmit behavior.
2. Expose the **smallest production-owned socket-outcome seam** used by all three executable reliable-UDP first-send call sites. A tiny helper/trait/callback around the actual send outcome plus commit/abort diagnostic ownership is acceptable. Do not create a second transport architecture.
3. The deterministic regression must execute the same production owner that real CLI code calls. A copied condition, or another helper-only test that manually invokes `on_packet_sent` + `abandon_sent`, is not sufficient.
4. Inject a send failure **after successful admission** and prove:
   - no positive `*_sent` diagnostic escapes;
   - the matching typed `*_send_failed` classification is emitted where the current call site owns it;
   - packet-copy Recovery/Reno/charged/packet-frame ownership is gone;
   - the aborted PN is not ACK-valid historical sent state;
   - no RTT/PTO/loss/Session/Carrier-success transition is fabricated from the nonexistent socket send.
5. Pair it with a success outcome using a fresh legal PN that preserves ownership, emits positive sent evidence and can settle through the existing ACK path.
6. Keep `--drop-r9-data` semantically distinct: deliberate controlled loss intentionally retains Recovery ownership so PTO/retransmission can recover it. Never convert it into socket-abort semantics.
7. Audit all three current reliable-UDP first-send call sites against the shared production owner. A single production helper plus discriminating call-site tests is preferable to duplicated test-only logic.
8. No new TTL/LRU/history-size/capacity/security values, no D019 change, no ACK redesign, no capacity-pressure benchmark, no release-authority change.

After the final pushed repair SHA, run and persist the ordinary exact-tree developer gate:

```text
PYTHONDONTWRITEBYTECODE=1 bash scripts/check.sh
git diff --check
```

Record exact reachable pushed SHA, UTC start/end, exit codes, OS/arch, stable Rust version and clean-tree state. Decoder/parser/crypto-framing fuzz is required only if those surfaces actually change.

On closure, continue immediately to READY_LOCAL 2 without waiting for reviewer cadence.

# READY_LOCAL 2 — R9-6B ownership / resource boundedness completion

After H-R9-062 closes, finish the remainder of R9-6 as an independent bounded challenge rather than assuming the socket-transaction fix closes every ownership question.

Minimum scope:

- first-send and retransmit packet-copy ownership cannot survive refusal/abort/terminal teardown incorrectly;
- retransmit exact-wire admission remains truthful;
- committed packet numbers are never reused; aborted reservations do not become ACK-valid history;
- stable retained plaintext/frame ownership is bounded by existing owners and deterministically released on ACK/loss/terminal cleanup;
- **specifically challenge first-send abort:** after the packet copy is removed from Recovery, any retained stable plaintext must have a real existing retry owner; if no retry owner can schedule it, it must not become orphan retained state and terminal teardown must deterministically clear it. Do not invent TTL/LRU/capacity policy to answer this;
- packet-to-frame maps, Reno bytes-in-flight and Recovery packet state reconcile after every positive/negative outcome;
- historical classification metadata stays bounded/accounted by existing ownership.

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
- residual-domain failure;
- terminal result/settlement.

H-R9-062's production socket-success/failure evidence boundary must be included. No diagnostic may manufacture evidence in another domain. Add only discriminators that would actually fail if the production owner regressed; do not generate schema/checker filler.

# READY_LOCAL 4 — R9 mid-slice factual reconciliation

After the coherent R9-4/R9-5/R9-6/R9-7 group is closed, perform one **small factual reconciliation** of release/item-4 state before continuing. Do not rewrite the full release packet yet.

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
- readiness is not inferred from TCP connect, UDP packet feedback, or Session DeliveryAck;
- readiness resource admission/generation/session/epoch binding remains exact;
- failed/incomplete readiness cannot emit positive promotion evidence or steal active ownership.

No policy-value changes.

# READY_LOCAL 6 — R9-9 health + promotion

Challenge integrated resolved reliable-UDP outcomes -> Carrier health/hysteresis -> promotion:

- recoverable reliable-UDP loss/PTO that successfully settles must stay on UDP rather than spuriously promote TCP;
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
- shutdown/error/partial-promotion negatives clean retained ownership and emit no false success.

Do not change core replay architecture. Any ambiguity requiring such a change is a maintainer/architecture gate while independent local lanes continue.

# READY_LOCAL 8 — R9-11 lifecycle / terminal invariants

Close remaining lifecycle and terminal invariants for the new cross-process reliable-UDP/failover integration:

- exactly one terminal success/failure classification per bounded operation;
- intermediate timeout/residual diagnostics are not terminal success/failure;
- no post-terminal retransmit, delivery confirmation, promotion or new Data ownership;
- listener/socket/session/recovery retained state is deterministically cleaned on normal and error exits;
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

Earlier reachable independent reviews cover the established core implementations broadly; refill only genuinely implemented surfaces that still lack a dedicated reachable challenge or where R9 materially changed the owner. No checker/schema/framework/docs filler.

Repository-wide `queue exhausted` is permitted only when this broad inventory finds no concrete defect, no READY_LOCAL review/support lane, no READY_LIVE question, and every remaining item is truly policy/environment/release-authority gated.

## VPS opportunity

**Not READY. `READY_LIVE: none`.** Standing authorization remains valid, but H-R9-062 and the current R9 queue are deterministic local correctness/evidence work and produce no unresolved real-network question that loopback/process evidence cannot answer. Do not repeat HY2, warm failover, periodic/soak, package lifecycle, migration-back, endpoint/key migration, IPv6, PLPMTUD or Experimental Track evidence merely because the VPS remains rented.

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
