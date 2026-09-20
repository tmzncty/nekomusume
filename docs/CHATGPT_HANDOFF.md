# ChatGPT reviewer handoff — H-R9-083 FRONT HIGH; R9-11D remains open

## Current repository truth

- Current reachable developer source/test anchor is `696dd02c7021cb5cf1fb35f757bd1cd88c0a7c00`, building on source repair `d1554a21258c4fe25e8b2ea0a07768f66133236c`. `696dd02` adds a controlled `net_dir` seam plus direct partial-observation regressions; it is now the exact-current executable test/source tree for H-R9-083 review.
- **H-R9-083 remains OPEN HIGH** after independent exact-current challenge in `docs/reviews/h-r9-083-696dd02-closure-challenge-20260920.md` (`814d230df3ac5fc5f3d04d695ac6e0922dc5147c`). The earlier missing-table and malformed-TCP-row counterexamples are repaired and now directly tested. The remaining blocker is narrower: escaped TCP/UDP ownership establishment is still not a deterministic pre-terminal happens-before handshake, because the fixture waits for/asserts the readiness marker only *after* the synchronous sampler invocation returns. The `finally` blocks also only send `SIGKILL` and do not verify bounded disappearance; the direct `None` helper regression does not yet mutation-test result construction (`unknown` must never become cleanup zero/complete).
- H-R9-082 remains closed for the original process-group-empty / escaped-owner false cleanup inference. `d1554a2` preserves four-table TCP/TCP6/UDP/UDP6 present/absent/unknown terminal ownership semantics and assertion-safe `finally` cleanup attempts.
- Developer-reported clean exact-tree provenance exists for exact `40cbc6a` (`PYTHONDONTWRITEBYTECODE=1 bash scripts/check.sh` exit 0, `git diff --check` exit 0, clean tree, 2026-09-20T11:15:10Z → 2026-09-20T11:20:30Z, Linux x86_64, rustc 1.98.0). No final exact-tree provenance for `696dd02` is accepted yet. Reviewer-local execution is not claimed. GitHub combined-status/workflow lookup exposed no hosted run for exact `696dd02`; absence of hosted evidence is not failure.
- Developer bounded R9-11D note `7ff2a0d750dff69d0477018977b6567e62e0c251` remains useful support but is **not** accepted as R9-11D closure while H-R9-083 and dedicated D1-D4 independent lanes remain open.
- H-R9-081 remains CLOSED at `730f993c687683f00f82859cbb7587d9fd6f5b84`; R9-11B remains CLOSED at `3b3a9182a27cc61df22250d370b87a0b50e9d253`; R9-11C remains CLOSED at `defb2dedd208da437a2035e5fe0417a160b713f2`; H-R9-080 and R9-11A remain CLOSED at their existing reachable anchors. Candidate A (future/never-sent ACK) and Candidate B (mixed queue/terminal observability projection) remain closed by current code/tests.
- Sampler observation/release determinism HIGH for the ordinary owned child remains closed at `e021b27cffe4a87de185757a8d47b8685f462bb6`; H-R9-083 is specifically about terminal escaped-owner cleanup evidence.
- `READY_LIVE: none`. Do not repeat HY2, warm failover, periodic/soak, package lifecycle, migration-back, endpoint/key migration, IPv6, PLPMTUD or Experimental Track without a new code/instrumentation/hypothesis/path condition creating a concrete unresolved self-owned real-network question.
- Release items 3 and 4 remain incomplete. `RELEASE_CANDIDATE=false`, `PRODUCTION_READY=false`, `FREEZE=false`, `RELEASED=false` remain unchanged.

The external coding agent must synchronize to current `main` and continuously execute dependency-ready work: implementation/review -> focused deterministic tests -> commit -> push -> clean exact-tree local gate/provenance -> next slice. Reviewer cadence is only a check frequency. Do not wait for the next reviewer after closing a slice.

## READY_LOCAL 1 — FRONT HIGH: H-R9-083 escaped-owner terminal cleanup truth

Read `docs/reviews/h-r9-083-d1554-closure-challenge-20260920.md` and `docs/reviews/h-r9-083-696dd02-closure-challenge-20260920.md`, then exact-current sampler/validator/tests before editing.

Required closure:

1. preserve the corrected four-table terminal ownership semantics: TCP/TCP6 LISTEN plus UDP/UDP6 bound sockets for caller-supplied ports; unreadable/malformed required observation remains `unknown`, never absence;
2. replace the post-hoc escaped TCP/UDP readiness assertion with a **true pre-terminal happens-before handshake**: the original-group path must not be able to finish before the escaped descendant has bound and its readiness has been consumed. A larger fixed sleep is not an ownership oracle;
3. retain assertion-safe cleanup and additionally verify each escaped helper is actually gone with a bounded disappearance/reap oracle; signal delivery alone is not cleanup proof;
4. add one focused mutation-sensitive result-construction negative: `group_empty=true` + terminal socket state `unknown` must not yield `owned_sockets_after_exit=0` or `cleanup.complete=true`;
5. keep the new missing-table / truncated-TCP-row / all-four-clean direct regressions green;
6. run focused process-resource tests, then on the final pushed source/test SHA run `PYTHONDONTWRITEBYTECODE=1 bash scripts/check.sh`, `git diff --check`, confirm clean tree, and persist exact SHA / UTC start-end / exit codes / OS-arch / stable Rust provenance.

No decoder/framing change: do not mechanically run fuzz. Do not redesign Session/Carrier/ACK/wire/crypto, change D019, or invent timeout/capacity/security policy values. After closure, continue immediately to READY_LOCAL 2.

## READY_LOCAL 2 — R9-11D1 process result-truth / terminal classification

Independently re-challenge exact-current automatic-health failover/migration-back executable owners and process fixtures. Verify success, explicit failure, timeout, shutdown and post-return classifications:

- terminal failure/timeout/stop cannot later be rewritten as READY/success because cleanup/read/child work returns zero;
- structured result JSON, human output and exit status describe the same authoritative terminal outcome;
- missing/unknown cleanup stays failure/unknown;
- post-return work cannot manufacture new delivery/switch/readiness success evidence;
- negative fixtures prove the intended failure phase was actually reached.

Concrete defect -> smallest repair/regression/gate. No defect -> scope-precise bounded no-finding note, then continue immediately to D2.

## READY_LOCAL 3 — R9-11D2 TCP/UDP listener and socket ownership

After H-R9-083:

- success/failure/timeout/shutdown deterministically release no-longer-owned TCP/UDP sockets;
- cleanup is proved by owner/socket evidence or committed same-address/port rebind oracle, never inferred from child exit or original-PGID emptiness;
- escaped/reparented descendants cannot retain TCP or UDP ownership while cleanup reports zero;
- partial setup failure cannot strand sockets while returning success/clean;
- repeated close/shutdown is idempotent and creates no false-positive lifecycle evidence.

Repair findings immediately; otherwise record bounded no-finding and continue to D3.

## READY_LOCAL 4 — R9-11D3 child/process-group/temp-runtime cleanup

Challenge process-owned resources independently of socket ownership:

- bounded reap/kill path on success, failure, timeout and interruption, including descendants that leave the original group;
- readiness/observation handshakes remain deterministic;
- temporary runtime paths are removed or truthfully reported unknown/failed; cleanup failure is never silently promoted;
- cleanup reporting exposes no secrets/private topology/unnecessary absolute paths;
- no new timeout/capacity/security policy values merely to satisfy tests.

Concrete defect -> repair/regression/gate; no defect -> bounded no-finding and continue to D4.

## READY_LOCAL 5 — R9-11D4 shutdown / post-return false-success negatives

Adversarially cross-check executable failover/migration-back and probe/process fixtures:

- shutdown during setup/traffic/recovery leaves no later READY/success/delivery event;
- timeout followed by late child/socket completion cannot overwrite terminal outcome;
- cleanup/rebind evidence belongs to the same exact run/process owner, not a stale predecessor;
- application/session success proof plus truthful cleanup/result state is required; cleanup-only success is insufficient;
- failure/unknown remains failure/unknown even if a later independent postcheck finds zero residue.

Do not rewrite immutable historical artifacts. Then do one concise R9-11A-D factual reconciliation.

## READY_LOCAL 6 — R9-12 final exact-tree provenance

After coherent R9-11 repair/review stabilizes, persist developer-local provenance for the final pushed developer source/test SHA: `PYTHONDONTWRITEBYTECODE=1 bash scripts/check.sh`; `git diff --check`; clean tree; exact pushed SHA; UTC start/end; exit codes; OS/arch; stable Rust. Run pinned decode fuzz only if the coherent group actually touches wire decoder/parser/crypto framing. Hosted CI remains supplemental.

## READY_LOCAL 7 — dedicated final independent R9 bounded review

On the final R9 source/test tree, independently re-challenge Recovery, receiver ACK ownership, Session logical proof, Carrier readiness/promotion, uncertain retention, TCP replay/dedup, migration-back, process result truth and terminal cleanup. No-finding bounded review is valid item-4 support. Concrete defect converts immediately to repair.

## READY_LOCAL 8 — Q10 factual reconciliation

Reconcile release packet/status claims against exact reachable R9 review/provenance anchors. Update only factual indexes/boundaries that changed. Do not mark item 4 complete or promote local evidence into WAN/security/release claims.

## READY_LOCAL 9 — Q11 release-evidence boundary reconciliation

Recheck item 3/WAN rows, historical negatives and `READY_LIVE`. Current authoritative classification remains `READY_LIVE: none`; only a new code/instrumentation/hypothesis/path condition creating a concrete unresolved self-owned real-network question may open a live lane.

## READY_LOCAL 10 — Q12 governance/release-state reconciliation

Confirm D019 and other policy/value gates remain separate and RC/production/freeze/release authority has not been inferred from engineering completion. Do not select TTL/LRU/history/capacity/security numbers, signing/key-custody/SBOM/publication policy, destructive migration, frozen-release policy or release authority.

## READY_LOCAL 11 — repository-wide item-4 refill

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

No-finding independent review is valid item-4 support. Do not count one narrow no-finding as repository-wide exhaustion.

## READY_LOCAL 12 — conditional live question only

Only if new code/instrumentation/hypothesis/path condition creates a specific unresolved self-owned real-network question, classify it against `docs/standing-vps-lab-authorization.md` and `docs/vps-rental-window-priority.md`. Ordinary bounded self-owned TCP/UDP work inside standing authorization needs no per-run approval. Do not manufacture live work merely because the VPS is rented.

## Stop / escalation conditions

Continue implementation/review -> tests -> commit -> push -> next dependency-ready slice without waiting for reviewer cadence. Stop/escalate only for an unresolved BLOCKER/HIGH that cannot be auto-decided, core Session/Carrier/ACK/crypto/wire architecture change, D019/policy-value choice, destructive/canonical migration, out-of-standing-authorization operation, third-party/production/new-credential permission, maintainer-selected adversarial-load/benchmark conditions, or entry into a genuinely new release stage.
