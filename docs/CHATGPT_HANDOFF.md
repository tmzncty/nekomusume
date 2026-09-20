# ChatGPT reviewer handoff — R9-11D2 front; no open HIGH

## Current repository truth

- Current reachable developer source/test anchor is `8cbd9af39a600f4ddd5fbc50208791e8584c6d74` (`fix(bench): H-R9-084 sampler exit status follows terminal cleanup truth`).
- **H-R9-083 source/test contract is accepted at exact `2f92781`**: escaped TCP/UDP parent readiness fails closed, bounded disappearance checks are active, all four required `/proc/net/{tcp,tcp6,udp,udp6}` tables preserve present/absent/unknown semantics, and the test-only `--net-dir` seam proves unknown terminal ownership maps to `owned_sockets_after_exit=null` / `cleanup.complete=false`.
- **H-R9-084 is CLOSED** at exact `8cbd9af39a600f4ddd5fbc50208791e8584c6d74`: sampler wrapper exit is nonzero whenever terminal cleanup is not affirmatively complete while the measured child's exit fact remains truthful in structured output. Positive cleanup-complete and child failure/timeout/interruption controls remain separated.
- Developer-local clean exact-tree provenance for exact `8cbd9af`: `PYTHONDONTWRITEBYTECODE=1 bash scripts/check.sh` exit 0, `git diff --check` exit 0, clean worktree, 2026-09-20T14:57:17Z → 2026-09-20T15:03:00Z, Linux x86_64, rustc 1.98.0. This is developer-reported local provenance, not reviewer-local execution or hosted CI.
- **R9-11D1 is CLOSED / bounded no-finding** at reviewer note `6164fc45f21745f7e5afc2e2b2826ecc999a6f93` (`docs/reviews/reviewer-r9-11d1-result-truth-20260920.md`). Exact-current failover-client/server automatic-health + migration-back result owners and process fixtures were independently challenged for terminal failure/timeout/result-truth/post-return false-success at the D1 boundary. No concrete defect was found. This supports release item 4 only; it is not R9 or release closure.
- Developer bounded R9-11D note `7ff2a0d750dff69d0477018977b6567e62e0c251` remains useful support but is **not** accepted as total R9-11D closure while dedicated D2-D4 independent lanes remain open.
- H-R9-081 remains CLOSED at `730f993c687683f00f82859cbb7587d9fd6f5b84`; R9-11B remains CLOSED at `3b3a9182a27cc61df22250d370b87a0b50e9d253`; R9-11C remains CLOSED at `defb2dedd208da437a2035e5fe0417a160b713f2`; H-R9-080 and R9-11A remain CLOSED at their existing reachable anchors. Candidate A (future/never-sent ACK) and Candidate B (mixed queue/terminal observability projection) remain closed by current code/tests.
- Reviewer-local test execution is not claimed in the D1 closure. Absence of GitHub-hosted evidence is not failure.
- `READY_LIVE: none`. Do not repeat HY2, warm failover, periodic/soak, package lifecycle, migration-back, endpoint/key migration, IPv6, PLPMTUD or Experimental Track without a new code/instrumentation/hypothesis/path condition creating a concrete unresolved self-owned real-network question.
- Release items 3 and 4 remain incomplete. `RELEASE_CANDIDATE=false`, `PRODUCTION_READY=false`, `FREEZE=false`, `RELEASED=false` remain unchanged. D019 and all policy/value gates remain untouched.

The external coding agent must synchronize to current `main` and continuously execute dependency-ready work: implementation/review -> focused deterministic tests -> commit -> push -> clean exact-tree local gate/provenance -> next slice. Reviewer cadence is only a check frequency. Do not wait for the next reviewer after closing a slice.

## READY_LOCAL 1 — FRONT: R9-11D2 TCP/UDP listener and socket ownership

Independently challenge exact-current executable failover/migration-back, ordinary probe/server, and process-resource fixtures at the socket-ownership boundary. Read exact owners/tests plus the accepted H-R9-082/083/084 repairs before concluding.

Required challenge:

- success/failure/timeout/shutdown deterministically release no-longer-owned TCP and UDP sockets;
- cleanup is proved by owner/socket evidence or a committed same-address/port rebind oracle, never inferred merely from child exit or original-PGID emptiness;
- escaped/reparented descendants cannot retain TCP or UDP ownership while cleanup reports zero;
- partial setup failure cannot strand a bound listener while returning success/clean;
- repeated close/shutdown is idempotent and creates no false-positive lifecycle evidence;
- a positive rebind/absence observation must belong to the same exact run/owned port rather than a stale predecessor.

Concrete defect -> smallest repair + positive/negative regression + focused tests + exact-tree gate/provenance, then continue. No defect -> scope-precise independent bounded no-finding note, then continue immediately to D3. No decoder/framing change: do not mechanically run fuzz. Do not invent new timeout/capacity/security policy values.

## READY_LOCAL 2 — R9-11D3 child/process-group/temp-runtime cleanup

Challenge process-owned resources independently of socket ownership:

- bounded reap/kill path on success, failure, timeout and interruption, including descendants that leave the original group;
- readiness/observation handshakes remain deterministic and cannot succeed because the observer raced ahead;
- temporary runtime paths are removed or truthfully reported unknown/failed; cleanup failure is never silently promoted;
- cleanup reporting exposes no secrets/private topology/unnecessary absolute paths;
- no new timeout/capacity/security policy values merely to satisfy tests.

Concrete defect -> repair/regression/gate; no defect -> bounded no-finding and continue immediately to D4.

## READY_LOCAL 3 — R9-11D4 shutdown / post-return false-success negatives

Adversarially cross-check executable failover/migration-back and probe/process fixtures:

- shutdown during setup/traffic/recovery leaves no later READY/success/delivery event;
- timeout followed by late child/socket completion cannot overwrite terminal outcome;
- cleanup/rebind evidence belongs to the same exact run/process owner, not a stale predecessor;
- application/session success proof plus truthful cleanup/result state is required; cleanup-only success is insufficient;
- failure/unknown remains failure/unknown even if a later independent postcheck finds zero residue.

Do not rewrite immutable historical artifacts. After D4, do one concise R9-11A-D factual reconciliation and continue.

## READY_LOCAL 4 — R9-12 final exact-tree provenance

After coherent R9-11 repair/review stabilizes, persist developer-local provenance for the final pushed developer source/test SHA: `PYTHONDONTWRITEBYTECODE=1 bash scripts/check.sh`; `git diff --check`; clean tree; exact pushed SHA; UTC start/end; exit codes; OS/arch; stable Rust. Run pinned decode fuzz only if the coherent group actually touches wire decoder/parser/crypto framing. Hosted CI remains supplemental.

## READY_LOCAL 5 — dedicated final independent R9 bounded review

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
