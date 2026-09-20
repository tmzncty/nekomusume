# ChatGPT reviewer handoff — R9-11 all lanes closed; final independent R9 review front

## Current repository truth

- Current reachable developer source/test anchor is `8cbd9af39a600f4ddd5fbc50208791e8584c6d74` (`fix(bench): H-R9-084 sampler exit status follows terminal cleanup truth`).
- **H-R9-083 source/test contract is accepted at exact `2f92781`**: escaped TCP/UDP parent readiness fails closed, bounded disappearance checks are active, all four required `/proc/net/{tcp,tcp6,udp,udp6}` tables preserve present/absent/unknown semantics, and the test-only `--net-dir` seam proves unknown terminal ownership maps to `owned_sockets_after_exit=null` / `cleanup.complete=false`.
- **H-R9-084 is CLOSED** at exact `8cbd9af39a600f4ddd5fbc50208791e8584c6d74`: sampler wrapper exit is nonzero whenever terminal cleanup is not affirmatively complete while the measured child's exit fact remains truthful in structured output. Positive cleanup-complete and child failure/timeout/interruption controls remain separated.
- Developer-local clean exact-tree provenance for exact `8cbd9af`: `PYTHONDONTWRITEBYTECODE=1 bash scripts/check.sh` exit 0, `git diff --check` exit 0, clean worktree, 2026-09-20T14:57:17Z → 2026-09-20T15:03:00Z, Linux x86_64, rustc 1.98.0. This is developer-reported local provenance, not reviewer-local execution or hosted CI.
- **R9-11D1 is CLOSED / bounded no-finding** at reviewer note `6164fc45f21745f7e5afc2e2b2826ecc999a6f93` (`docs/reviews/reviewer-r9-11d1-result-truth-20260920.md`). Exact-current failover-client/server automatic-health + migration-back result owners and process fixtures were independently challenged for terminal failure/timeout/result-truth/post-return false-success at the D1 boundary. No concrete defect was found. This supports release item 4 only; it is not R9 or release closure.
- **R9-11D2 is CLOSED** at dev bounded no-finding `dev-r9-11d2-socket-ownership-20260920.md` (`72c5e06`): TCP/UDP listener and socket ownership challenged across terminal paths — owned sockets released, cleanup evidenced by independent `/proc/net` oracle keyed to the exact `--owned-port` set, escaped/partial states never certified cleaned.
- **R9-11D3 is CLOSED** at dev bounded no-finding `dev-r9-11d3-child-process-group-temp-runtime-20260920.md` (`5a4082c`): bounded TERM→KILL reap on every terminal path, setsid-escaped TCP/UDP holders not promoted, readiness handshakes deterministic, temporary output atomically replaced or truthfully unknown, no secrets/topology/paths in reporting.
- **R9-11D4 is CLOSED** at dev bounded no-finding `dev-r9-11d4-shutdown-post-return-false-success-20260920.md` (`e51a630`): no post-shutdown READY/success/delivery, late completion cannot overwrite terminal outcome, cleanup/rebind evidence bound to exact run/owner, cleanup-only never promoted to success.
- **R9-11A–D factual reconciliation** persisted at `docs/reviews/r9-11a-d-factual-reconciliation-20260920.md` (`ab5409c`) — all seven lanes closed at reachable anchors; R9-11 as a whole is not marked complete pending READY_LOCAL 4/5.
- **R9-12 final exact-tree developer-local provenance** persisted at `docs/notes/r9-12-final-provenance-8cbd9af-20260920.md`: `8cbd9af` check.sh exit 0, diff --check exit 0, clean worktree, 2026-09-20T15:55:57Z → 16:01:40Z, Linux x86_64, rustc 1.98.0.
- H-R9-081 remains CLOSED at `730f993c687683f00f82859cbb7587d9fd6f5b84`; R9-11B remains CLOSED at `3b3a9182a27cc61df22250d370b87a0b50e9d253`; R9-11C remains CLOSED at `defb2dedd208da437a2035e5fe0417a160b713f2`; H-R9-080 and R9-11A remain CLOSED at their existing reachable anchors. Candidate A (future/never-sent ACK) and Candidate B (mixed queue/terminal observability projection) remain closed by current code/tests.
- Reviewer-local test execution is not claimed in the D1 closure. Absence of GitHub-hosted evidence is not failure.
- `READY_LIVE: none`. Do not repeat HY2, warm failover, periodic/soak, package lifecycle, migration-back, endpoint/key migration, IPv6, PLPMTUD or Experimental Track without a new code/instrumentation/hypothesis/path condition creating a concrete unresolved self-owned real-network question.
- Release items 3 and 4 remain incomplete. `RELEASE_CANDIDATE=false`, `PRODUCTION_READY=false`, `FREEZE=false`, `RELEASED=false` remain unchanged. D019 and all policy/value gates remain untouched.

The external coding agent must synchronize to current `main` and continuously execute dependency-ready work: implementation/review -> focused deterministic tests -> commit -> push -> clean exact-tree local gate/provenance -> next slice. Reviewer cadence is only a check frequency. Do not wait for the next reviewer after closing a slice.

## READY_LOCAL 1 — CLOSED: R9-11D2 TCP/UDP listener and socket ownership

Closed at `72c5e06` — dev bounded no-finding `docs/reviews/dev-r9-11d2-socket-ownership-20260920.md`.

## READY_LOCAL 2 — CLOSED: R9-11D3 child/process-group/temp-runtime cleanup

Closed at `5a4082c` — dev bounded no-finding `docs/reviews/dev-r9-11d3-child-process-group-temp-runtime-20260920.md`.

## READY_LOCAL 3 — CLOSED: R9-11D4 shutdown / post-return false-success negatives

Closed at `e51a630` — dev bounded no-finding `docs/reviews/dev-r9-11d4-shutdown-post-return-false-success-20260920.md`. R9-11A-D factual reconciliation at `ab5409c`.

## READY_LOCAL 4 — CLOSED: R9-12 final exact-tree provenance

`docs/notes/r9-12-final-provenance-8cbd9af-20260920.md` — `8cbd9af` check.sh exit 0, diff --check exit 0, clean worktree, 2026-09-20T15:55:57Z → 16:01:40Z, Linux x86_64, rustc 1.98.0.

## READY_LOCAL 5 — FRONT: dedicated final independent R9 bounded review

On the final R9 source/test tree, independently re-challenge Recovery, receiver ACK ownership, Session logical proof, Carrier readiness/promotion, uncertain retention, TCP replay/dedup, migration-back, process result truth and terminal cleanup. No-finding bounded review is valid item-4 support. Concrete defect converts immediately to repair.

## READY_LOCAL 6 — Q10 factual reconciliation

Reconcile release packet/status claims against exact reachable R9 review/provenance anchors. Update only factual indexes/boundaries that changed. Do not mark item 4 complete or promote local evidence into WAN/security/release claims.

## READY_LOCAL 7 — Q11 release-evidence boundary reconciliation

Recheck item 3/WAN rows, historical negatives and `READY_LIVE`. Current authoritative classification remains `READY_LIVE: none`; only a new code/instrumentation/hypothesis/path condition creating a concrete unresolved self-owned real-network question may open a live lane.

## READY_LOCAL 8 — Q12 governance/release-state reconciliation

Confirm D019 and other policy/value gates remain separate and RC/production/freeze/release authority has not been inferred from engineering completion. Do not select TTL/LRU/history/capacity/security numbers, signing/key-custody/SBOM/publication policy, destructive migration, frozen-release policy or release authority.

## READY_LOCAL 9 — repository-wide item-4 refill

Item 4 remains incomplete. Repeat the broad core-surface inventory rather than declaring queue exhaustion from R9 alone. Prefer implemented surfaces lacking a dedicated reachable independent bounded challenge on the current/relevant tree, or whose semantics changed since the prior challenge:

1. `neko-reliable` ACK ranges / future-unsent / loss-retransmit / RTT-PTO / persistent congestion / Reno / fault simulation;
2. `CarrierState` generation / validation / hysteresis / single-active / drain-fail-activate;
3. concurrent manager / health / migration-back;
4. FairScheduler / multi-stream / Session+stream flow-control accounting;
5. Memory/UDP/TCP carrier close/error/resource semantics;
6. SessionRuntime lifecycle/resource/window/DeliveryAck accounting;
7. observability event/counter/high-water projection;
8. package/reproducibility/operator scripts;
9. Cargo manifests/lock/features/build/native hooks/unsafe inheritance;
10. cross-platform CLI/process-test semantics;
11. CLI exit-code / JSON / human-output contract;
12. algorithmic resource boundedness without capacity-pressure benchmarking or invented policy values;
13. release-packet factual consistency/evidence boundaries.

No-finding independent review is valid item-4 support. Do not count one narrow no-finding as repository-wide exhaustion. Refill into additional coherent dependency-ready lanes as the inventory identifies real uncovered surfaces.

## READY_LOCAL 10 — conditional live question only

Only if new code/instrumentation/hypothesis/path condition creates a specific unresolved self-owned real-network question, classify it against `docs/standing-vps-lab-authorization.md` and `docs/vps-rental-window-priority.md`. Ordinary bounded self-owned TCP/UDP work inside standing authorization needs no per-run approval. Do not manufacture live work merely because the VPS is rented.

## Stop / escalation conditions

Continue implementation/review -> tests -> commit -> push -> next dependency-ready slice without waiting for reviewer cadence. Stop/escalate only for an unresolved BLOCKER/HIGH that cannot be auto-decided, core Session/Carrier/ACK/crypto/wire architecture change, D019/policy-value choice, destructive/canonical migration, out-of-standing-authorization operation, third-party/production/new-credential permission, maintainer-selected adversarial-load/benchmark conditions, or entry into a genuinely new release stage.
