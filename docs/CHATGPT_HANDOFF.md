# ChatGPT reviewer handoff — H-R9-082 FRONT HIGH; R9-11D process/socket lifecycle remains open

## Current repository truth

- Exact developer source/test tree remains `730f993c687683f00f82859cbb7587d9fd6f5b84`; later commits through this handoff are review/navigation docs only.
- Developer bounded R9-11D review `7ff2a0d750dff69d0477018977b6567e62e0c251` is **not accepted as R9-11D closure** pending independent review. Source review found **H-R9-082 OPEN HIGH** at reviewer anchor `45d3d4a87190db7587380b7751013066c7bc6606`: the process-resource sampler samples/signals/verifies only the original sampler-created process group, then derives `owned_sockets_after_exit=0` and `cleanup.complete=true` solely from that group becoming empty. A descendant that leaves the original group (for example with `setsid()`) can retain an owned listener while the current oracle certifies complete cleanup. **H-R9-082 is now CLOSED** at exact `0831076eb96e4fd2fbbb71bed7190a21d41fca0f`: `cleanup.complete`/`owned_sockets_after_exit` now require BOTH the original process group empty AND no caller-owned port still bound as a listener by any surviving descendant (`owned_port_listeners_present()` checks `/proc/net/tcp{,6}`). A `setsid`-escaped descendant holding the owned listener is no longer certified as cleaned up — `owned_sockets_after_exit=None` and `complete=false`. Developer-local clean exact-tree provenance for exact `0831076`: `PYTHONDONTWRITEBYTECODE=1 bash scripts/check.sh` exit 0, `git diff --check` exit 0, clean worktree, 2026-09-20T08:58:37Z → 2026-09-20T09:03:51Z, Linux x86_64, rustc 1.98.0.
- **H-R9-081 is CLOSED** at `730f993`: the strengthened regression creates nonzero resolved pre-quiesce loss, then proves the first clean post-quiesce resolved interval reports `loss_per_mille == 0`, is fresh exactly once, and leaves lifetime loss diagnostics intact. Developer-local clean exact-tree provenance for exact `730f993`: `PYTHONDONTWRITEBYTECODE=1 bash scripts/check.sh` exit 0, `git diff --check` exit 0, clean worktree, 2026-09-20T05:50:25Z -> 2026-09-20T05:55:22Z, Linux x86_64, rustc 1.98.0. No reviewer-local execution is claimed. Connected GitHub status/workflow endpoints expose no hosted run for this SHA; absence is not treated as failure.
- **R9-11B is CLOSED as bounded independent no-finding support** at reviewer anchor `3b3a9182a27cc61df22250d370b87a0b50e9d253`, reviewing exact `730f993`. It covers Recovery/PathRecovery/ReliableUdpRuntime teardown ownership, receiver ACK obligations, retransmit plaintext, Reno charge, post-teardown mutators, future/never-sent ACK fail-closed behavior, and H-R9-081's quiescent health boundary.
- **R9-11C is CLOSED as bounded independent no-finding support** at reviewer anchor `defb2dedd208da437a2035e5fe0417a160b713f2`, after challenging developer review `a2a1e6cf6b2890a836cc9d6271061897730585cb` against exact owner source. Retained replay identity/bytes survive UDP teardown; `FailoverController::tcp_resend()` is driven from retained `(DataId, bytes)` state; `ConcurrentCarrierManager` marks failed/draining-owner ranges uncertain before replay, replays stable `LogicalRangeId + bytes`, gates stale generations/readiness, and does not let migration-back bypass validation/health/hold. This closure does not replace executable caller/Session-proof checks from R9-10 or the final independent R9 review.
- **H-R9-080 remains CLOSED** at exact `052b6eab40f8e4257d4480866b21f64ce3e5230c`. **R9-11A remains CLOSED** as bounded independent no-finding support at exact `a83b89850b75d6ff405c004b8186856987f5cd9e`; `SessionRuntime.events` remains `POLICY_BLOCKED_RESOURCE_BOUND` and no retention cap/TTL/LRU/history value has been invented.
- Candidate A (`Recovery::on_ack` future/never-sent largest) remains closed by the pre-mutation guard/regressions. Candidate B (`record_datagrams` mixed queue/terminal projection) remains closed by the current mixed-delta/reason regressions.
- Sampler observation-handshake determinism HIGH remains closed at exact `e021b27cffe4a87de185757a8d47b8685f462bb6`; H-R9-082 is a different terminal ownership/evidence seam.
- `READY_LIVE: none`. Do not repeat HY2, warm failover, periodic/soak, package lifecycle, migration-back, endpoint/key migration, IPv6, PLPMTUD or Experimental Track without a new code/instrumentation/hypothesis/path condition creating a concrete unresolved self-owned real-network question.
- Release item 3 and item 4 remain incomplete. `RELEASE_CANDIDATE=false`, `PRODUCTION_READY=false`, `FREEZE=false`, `RELEASED=false` remain unchanged.

The external coding agent must synchronize to current `main` and continuously execute dependency-ready work: implementation/review -> focused deterministic tests -> commit -> push -> clean exact-tree local gate/provenance -> next slice. Reviewer cadence is only a check frequency. Do not wait for the next reviewer after closing a slice.

## READY_LOCAL 1 — FRONT HIGH: H-R9-082 escaped descendant process/socket cleanup

Read `docs/reviews/h-r9-082-escaped-descendant-cleanup-20260920.md` and exact-current sampler/validator/tests before editing.

Required closure:

1. add a bounded deterministic regression where a sampler-owned helper creates a descendant that demonstrably leaves the original process group (for example `setsid()`), binds a caller-supplied loopback listener, and signals readiness only after the listener exists;
2. let the original helper/group terminate while that escaped descendant/listener still exists; current buggy semantics must not be allowed to certify `process_group_empty=true` + `owned_sockets_after_exit=0` + `complete=true` merely because the original group disappeared;
3. the test must prove the escaped PID/listener existed at the challenged seam and bounded-clean every helper even on assertion failure; use readiness/release or another deterministic handshake, not an arbitrary sleep;
4. make sampler cleanup ownership truth cover sampler-owned descendants beyond the original process-group id and/or add an independent terminal socket ownership/rebind oracle; do not infer socket count zero solely from original-group emptiness;
5. keep truthful unknown/failure rather than promotion when cleanup cannot be proved.

Use the smallest settled-semantic repair. Do not redesign Session/Carrier/ACK/wire/crypto, change D019, invent timeout/capacity/security values, or open live work. Run focused process-resource tests, then the final pushed developer SHA clean exact-tree gate/provenance. No decoder/framing change means no mechanical fuzz run. After this closes, continue immediately to READY_LOCAL 2 without waiting for reviewer cadence.

## READY_LOCAL 2 — R9-11D1 process result-truth / terminal classification independent re-challenge

Developer note `7ff2a0d` is useful research/support but is not the dedicated independent closure. Re-read the exact-current automatic-health failover/migration-back executable owners and process fixtures. Challenge success, explicit failure, timeout, shutdown and post-return terminal classifications as one bounded slice.

Required invariants:

1. terminal failure/timeout/stop cannot later be rewritten as READY/success merely because another cleanup/read/child step returns zero;
2. structured result JSON, human output and process exit status describe the same terminal outcome;
3. missing/unknown cleanup observation stays failure/unknown rather than being inferred from a successful application result;
4. post-return work cannot manufacture new delivery/switch/readiness success evidence after the authoritative result is terminal;
5. negative fixtures prove the intended failure phase was actually entered rather than passing vacuously before the challenged seam.

Concrete defect -> smallest repair + regression + focused tests. No defect -> scope-precise bounded no-finding note and continue immediately to R9-11D2.

## READY_LOCAL 3 — R9-11D2 TCP/UDP listener and socket ownership

Re-challenge after H-R9-082 repair:

- every listener/socket no longer owned after success, explicit failure, timeout or shutdown is deterministically closed;
- cleanup is proved by owner/socket evidence or successful same-address/port rebind where that is the committed oracle, never inferred from child exit 0 or original-process-group emptiness alone;
- escaped/reparented sampler-owned descendants cannot retain a listener while cleanup reports zero;
- partial setup failures cannot strand a listener while returning a successful/clean result;
- repeated close/shutdown remains idempotent and cannot create false positive lifecycle events;
- do not broaden into live WAN unless this review creates a genuinely real-network-only unresolved question.

Repair concrete defects immediately; otherwise record bounded no-finding and continue to R9-11D3.

## READY_LOCAL 4 — R9-11D3 child/process-group/temp-runtime cleanup

Challenge process-owned resources independently of socket ownership:

- spawned helper/child/process ownership has a bounded reap/kill path on success, failure, timeout and interruption, including sampler-owned descendants that leave the original process group;
- the sampler/readiness fixture handshake remains deterministic and cannot release the child before the observation used by the oracle;
- temporary runtime paths created by the process fixture are removed or truthfully reported unknown/failed; cleanup failure is never silently promoted;
- cleanup reporting must not expose secrets, private topology or unnecessary absolute paths;
- do not introduce new timeout/capacity/security policy numbers merely to make cleanup tests pass.

Concrete defect -> repair/regression/gate; no defect -> bounded no-finding and continue immediately to R9-11D4.

## READY_LOCAL 5 — R9-11D4 shutdown / post-return false-success negatives

Finish R9-11D with an adversarial local lifecycle cross-check across executable failover/migration-back and probe/process fixtures:

- shutdown while setup/traffic/recovery is in progress leaves no later READY/success/delivery event;
- timeout followed by late child/socket completion cannot overwrite the terminal outcome;
- cleanup and rebind evidence are attributed to the same exact run/process owner rather than a stale predecessor;
- success requires the committed application/session proof plus truthful cleanup/result state; cleanup-only success is insufficient;
- failure/unknown remains failure/unknown even if a later independent postcheck finds zero residue.

Do not rewrite immutable historical artifacts to make cleanup fields match later observations. After this slice, perform one concise R9-11A-D factual reconciliation rather than rewriting large release docs per micro-slice.

## READY_LOCAL 6 — R9-12 final exact-tree provenance

After coherent R9-11 repair/review stabilizes, persist developer-local provenance for the final pushed developer source/test SHA: `PYTHONDONTWRITEBYTECODE=1 bash scripts/check.sh`; `git diff --check`; clean tree; exact pushed SHA; UTC start/end; exit codes; OS/arch; stable Rust. Run pinned decode fuzz only if the coherent group actually touches wire decoder/parser/crypto framing. Hosted CI remains supplemental.

## READY_LOCAL 7 — dedicated final independent R9 bounded review

On the final R9 source/test tree, independently re-challenge Recovery, receiver ACK ownership, Session logical proof, Carrier readiness/promotion, uncertain retention, TCP replay/dedup, migration-back, process result truth and terminal cleanup. No-finding bounded review is valid item-4 support. Concrete defect converts immediately to repair; do not claim R9 closure first.

## READY_LOCAL 8 — Q10 factual reconciliation

Reconcile release packet/status claims against exact reachable R9 review/provenance anchors. Update only factual indexes/boundaries that changed. Do not mark item 4 complete or promote local evidence into WAN/security/release claims.

## READY_LOCAL 9 — Q11 release-evidence boundary reconciliation

Recheck item 3/WAN rows, historical negatives and `READY_LIVE`. Current authoritative classification remains `READY_LIVE: none`; only a new code/instrumentation/hypothesis/path condition creating a concrete unresolved self-owned real-network question may open a live lane.

## READY_LOCAL 10 — Q12 governance/release-state reconciliation

Confirm D019 and other policy/value gates remain separate and RC/production/freeze/release authority has not been inferred from engineering completion. Do not select TTL/LRU/history/capacity/security numbers, signing/key-custody/SBOM/publication policy, destructive migration, frozen-release policy or release authority.

## READY_LOCAL 11 — repository-wide item-4 refill

Item 4 remains incomplete. Repeat the broad core-surface inventory rather than declaring queue exhaustion from R9 alone. Prefer implemented core surfaces lacking a dedicated reachable independent bounded challenge on the current/relevant tree, or whose semantics changed since the prior challenge:

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
