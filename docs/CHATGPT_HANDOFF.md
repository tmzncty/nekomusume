# ChatGPT reviewer handoff — R9-11D continues; no open HIGH at sampler boundary

## Current repository truth

- Current reachable developer source/test anchor is `8cbd9af39a600f4ddd5fbc50208791e8584c6d74` (`fix(bench): H-R9-084 sampler exit status follows terminal cleanup truth`).
- **H-R9-083 source/test contract is accepted at exact `2f92781`**: the escaped TCP/UDP parent fails closed on readiness timeout, `pathlib.Path` bounded disappearance checks are active, the four required `/proc/net/{tcp,tcp6,udp,udp6}` tables preserve present/absent/unknown semantics, and the test-only `--net-dir` seam proves unknown terminal ownership maps to `owned_sockets_after_exit=null` / `cleanup.complete=false`.
- Developer-local clean exact-tree provenance for exact `8cbd9af`: `PYTHONDONTWRITEBYTECODE=1 bash scripts/check.sh` exit 0, `git diff --check` exit 0, clean worktree, 2026-09-20T14:57:17Z → 2026-09-20T15:03:00Z, Linux x86_64, rustc 1.98.0.
- **H-R9-084 is CLOSED** at exact `8cbd9af39a600f4ddd5fbc50208791e8584c6d74`: the sampler wrapper now returns nonzero (1) whenever terminal cleanup is not affirmatively complete — original group nonempty, owned socket present, or socket observation unknown — while `result["exit"]` preserves the measured child's exit fact (interrupted-signal and child-failure returns still dominate). `fixture.partial-obs` asserts the sampler subprocess itself is nonzero with `owned_sockets_after_exit=null` / `cleanup.complete=false`; positive controls keep exit 0 + `complete=true` + validator-valid output.
- Reviewer-local execution is not claimed. GitHub combined-status/workflow lookup exposed no hosted run for exact `2f92781`; absence of hosted evidence is not failure.
- Developer bounded R9-11D note `7ff2a0d750dff69d0477018977b6567e62e0c251` remains useful support but is **not** accepted as R9-11D closure while H-R9-084 and dedicated D1-D4 independent lanes remain open.
- H-R9-081 remains CLOSED at `730f993c687683f00f82859cbb7587d9fd6f5b84`; R9-11B remains CLOSED at `3b3a9182a27cc61df22250d370b87a0b50e9d253`; R9-11C remains CLOSED at `defb2dedd208da437a2035e5fe0417a160b713f2`; H-R9-080 and R9-11A remain CLOSED at their existing reachable anchors. Candidate A (future/never-sent ACK) and Candidate B (mixed queue/terminal observability projection) remain closed by current code/tests.
- `READY_LIVE: none`. Do not repeat HY2, warm failover, periodic/soak, package lifecycle, migration-back, endpoint/key migration, IPv6, PLPMTUD or Experimental Track without a new code/instrumentation/hypothesis/path condition creating a concrete unresolved self-owned real-network question.
- Release items 3 and 4 remain incomplete. `RELEASE_CANDIDATE=false`, `PRODUCTION_READY=false`, `FREEZE=false`, `RELEASED=false` remain unchanged.

The external coding agent must synchronize to current `main` and continuously execute dependency-ready work: implementation/review -> focused deterministic tests -> commit -> push -> clean exact-tree local gate/provenance -> next slice. Reviewer cadence is only a check frequency. Do not wait for the next reviewer after closing a slice.

## READY_LOCAL 1 — FRONT HIGH: H-R9-084 process-resource sampler terminal result truth

Read `docs/reviews/h-r9-084-process-resource-terminal-result-truth-20260920.md`, exact-current sampler/validator/schema/tests, and the R9-11D1 contract before editing.

Required closure:

1. preserve the measured child's terminal facts in JSON; a child exit 0 remains recorded as child exit 0;
2. make the sampler **wrapper process** return nonzero whenever terminal cleanup is not affirmatively complete (original process group not empty, owned-port socket present, or terminal socket observation unknown). Use an existing script-level failure convention if present; do not invent protocol/wire/security policy;
3. turn the current `fixture.partial-obs` witness into a mutation-sensitive negative: child exit remains 0 in JSON, cleanup remains unknown/incomplete, but the sampler subprocess is nonzero;
4. retain/add a positive child-exit-0 + affirmative cleanup-complete control that returns sampler exit 0 and passes `validate-process-resource.py`;
5. preserve child failure/timeout/interruption truth: cleanup work must never rewrite an already failing child into success, and cleanup failure must never manufacture command success;
6. keep all H-R9-083 escaped TCP/UDP, readiness, four-table present/absent/unknown and bounded disappearance regressions green;
7. run focused process-resource tests, then on the final pushed source/test SHA run `PYTHONDONTWRITEBYTECODE=1 bash scripts/check.sh`, `git diff --check`, confirm clean tree, and persist exact SHA / UTC start-end / exit codes / OS-arch / stable Rust provenance.

No decoder/framing change: do not mechanically run fuzz. Do not redesign Session/Carrier/ACK/wire/crypto, change D019, invent cleanup timeout/capacity/security values, or build a new evidence framework. After closure, continue immediately to READY_LOCAL 2.

## READY_LOCAL 2 — R9-11D1 remainder: process result-truth / terminal classification

After H-R9-084, independently re-challenge exact-current automatic-health failover/migration-back executable owners and process fixtures beyond the sampler-specific repair. Verify success, explicit failure, timeout, shutdown and post-return classifications:

- terminal failure/timeout/stop cannot later be rewritten as READY/success because cleanup/read/child work returns zero;
- structured result JSON, human output and exit status describe the same authoritative terminal outcome;
- missing/unknown cleanup stays failure/unknown;
- post-return work cannot manufacture new delivery/switch/readiness success evidence;
- negative fixtures prove the intended failure phase was actually reached.

Concrete defect -> smallest repair/regression/gate. No defect -> scope-precise bounded no-finding note, then continue immediately to D2.

## READY_LOCAL 3 — R9-11D2 TCP/UDP listener and socket ownership

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
