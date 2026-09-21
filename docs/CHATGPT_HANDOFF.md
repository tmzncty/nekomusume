# ChatGPT reviewer handoff — H-I4-089 fronts reopened I4-PORT queue

## Current repository truth

- Synchronize to current `main`. Reviewer finding **H-I4-089** is reachable at `78050ab4c4a66fed81b4e767f973d9a1921617a7` (`docs/reviews/reviewer-h-i4-089-macos-proc-false-pass-20260921.md`). It supersedes the immediately previous `f0013f7` / `cd82e7e` **queue-exhausted conclusion only where that conclusion depended on I4-PORT being closed**; it does not invalidate unrelated closed surfaces.
- The current executable/source-test tree is still exact `42be53917ee58359ad86532a6aab0136f75047b9`. Commits after it through `78050ab` are review/evidence/prose only; no decoder/parser/crypto-framing owner changed in this interval.
- **H-I4-089 is CLOSED** at `26a2c40be8a524024f58c7b771aec8b924ae6524`: `process_resource_snapshot` and its resource-growth assertions are now `#[cfg(target_os = "linux")]` — on Linux the snapshot must be affirmative (panic on `None`), on other Unix targets the portable socket/lifecycle part of `udp_listener_rejects_bounded_malformed_churn` still runs without fabricating proc evidence. Exact-tree provenance: `docs/notes/h-i4-089-provenance-26a2c40-20260921.md` — `check.sh` exit 0, `git diff --check` exit 0, clean worktree, 2026-09-21T06:56:21Z → 07:02:25Z, Linux x86_64, rustc 1.98.0.
- The prior developer I4-PORT note at `5e9b4b22bbfe8d336026fee238cd5ae22c282f6e` contains the now-falsified statement that `#[cfg(unix)]` removes these fixtures on macOS. The prior reviewer closure at `f0013f72f3f15f8e35df53205e7740000afad2ab` likewise cannot support repository-wide PORT closure until H-I4-089 is repaired and re-challenged.
- The narrower `signal_term()` seam is not itself the finding: external `kill -TERM` is a POSIX-shaped test seam. Do not broaden H-I4-089 into a rewrite of all process tests. The concrete defect is Linux-only `/proc` measurement + optional-`None` success semantics under the broader Unix cfg.
- **H-I4-086 remains CLOSED** at `ca629d702cf832c12bf74bdfc5f9d07ebb77ab17` + `3acba049fee5f9297cc8a4c3867250635c255717`: DeliveryAck cannot confirm queued-but-undrained bytes; sent/drained bookkeeping is terminal-owned and released.
- **H-I4-087 remains CLOSED** at source repair `14e2f520a0fc9d41bcfcb1bd480758008a1dd4ea` with developer closure/provenance support through `d819d38559d60b6bd99df8a2453321809c046116`: `max_queue_records` is aggregate over send+recv ownership. Preserve the provenance fact that one startup/socket timing flake preceded a clean rerun; do not promote it to hosted/reviewer CI.
- **H-I4-088 remains CLOSED** at `26aa4e8036d61da7924d9fc5e587081d8a0f448f`: MemoryCarrier empty messages consume a finite queue-record slot derived from the already committed queue bound; dequeue releases it. Do not invent a separate record-cap policy value.
- I4-FS2, Candidate A/B, CarrierState/CarrierManager, unchanged observability, unchanged package/operator owners, H-I4-085 dependency/build truth, CLI-M/CLI-H, and closed R9 process/result seams remain closed unless their exact-current owner changes or a new counterexample falsifies a claim.
- `SessionRuntime.events` retained-history capacity remains a maintainer/security policy gate. Do not choose a history-size/TTL/LRU/capacity value while repairing this PORT finding.
- Release items **3 and 4 remain incomplete**. `RELEASE_CANDIDATE=false`, `PRODUCTION_READY=false`, `FREEZE=false`, `RELEASED=false` remain unchanged.
- **`READY_LIVE: none`.** H-I4-089 is entirely local/test/evidence truth. Do not repeat HY2, warm failover, periodic/soak, package lifecycle, migration-back, endpoint/key migration, IPv6, PLPMTUD or Experimental Track merely because the VPS is still rented.

## FRONT — H-I4-089 repair contract

This HIGH is dependency-ready and automatically decidable from current committed semantics. It does **not** require maintainer policy input.

1. Keep Linux `/proc` resource observation explicitly Linux-scoped. Do not describe it as generic Unix/macOS coverage.
2. On Linux, the resource snapshot used by the malformed-UDP/resource-growth regression must be affirmative. A failed/missing snapshot must fail/skip truthfully according to a deliberate test boundary; it must **not** silently become `None -> pass` for the resource-growth claim.
3. On non-Linux Unix, choose the smallest truthful shape:
   - either keep portable socket/lifecycle behavior under Unix and compile/execute only the `/proc` resource assertion on Linux; or
   - gate the entire affected fixture to Linux if that fixture's committed purpose is intentionally Linux-only.
   Do not add macOS/release-platform claims merely to close this finding.
4. Add a deterministic/mutation-sensitive regression or source-level guard that locks the boundary: the Linux-only measurement path cannot silently degrade from unavailable observation to a successful resource claim.
5. Preserve the existing portable `Child::kill()` / signal/lifecycle coverage where it is genuinely platform-independent or POSIX-wide; do not mechanically cfg-elide unrelated tests.
6. Run the focused affected process/probe tests. Then, on the final pushed source/test SHA in a safe clean checkout/worktree, run:
   - `PYTHONDONTWRITEBYTECODE=1 bash scripts/check.sh`
   - `git diff --check`
   - verify the tree is clean.
   Record exact reachable pushed SHA, commands, UTC start/end, exit codes, OS/arch, Rust stable version and clean-tree state. Do not record secrets, private addresses/topology, credentials or unnecessary absolute paths.
7. No fuzz is required unless the repair unexpectedly changes decoder/parser/crypto framing. H-I4-089 should not do so.
8. Push the repair/provenance and continue immediately into the review queue below; do not wait for the next hourly reviewer cadence.

## Rolling queue after the HIGH repair

There are currently fewer legitimate independent owners than the earlier 8–15 steady-state target because the previous broad sweep closed unchanged surfaces. Do not manufacture filler merely to hit a number. The following **8 coherent lanes** are the current dependency-ordered queue; lanes 2–7 should collapse/skip themselves if exact-current facts make them redundant, but must not be silently replaced with `queue exhausted` before the broad refill check.

1. **H-I4-089 repair + tests + exact-tree local provenance** — FRONT HIGH, contract above.
2. **I4-PORT-LINUX independent bounded challenge** — inspect exact repaired `process_resource_snapshot` owner/caller and challenge the invariant that missing/failed Linux FD/RSS observation cannot be promoted to successful resource evidence. Use focused deterministic tests/source reasoning; if a concrete defect remains, repair it before continuing.
3. **I4-PORT-UNIX/platform-boundary challenge** — verify Linux-only observation and Unix-wide process/lifecycle semantics are truthfully separated. Confirm no prose/test claims macOS execution merely because `cfg(unix)` is true. No non-Linux execution claim without actual evidence.
4. **I4-PORT current callsite inventory** — bounded source sweep of `/proc`, external `kill`/signals, `setsid`/process-group and related process-resource helpers/callers in current CLI/process tests. The goal is to catch another Linux-only seam hidden behind broader cfg, not to rewrite working cross-platform `std::process` APIs.
5. **I4-PORT closure reconciliation** — write one scope-precise independent no-finding note only after lanes 2–4 are clean, naming inspected owners, focused commands/tests, exclusions, and exact reachable source/test anchor. Do not label developer self-review as reviewer-local evidence.
6. **Release/item-4 factual reconciliation for the changed PORT claim** — update only stale evidence/index/handoff claims that depended on the old PORT closure. Preserve item 3/4 as incomplete and all four governance flags as false. Do not rewrite unrelated historical evidence.
7. **Repository-wide 13-surface owner-diff refill check** — compare exact-current owners against the prior broad inventory. Reopen a surface only for a material owner change, stale/falsified claim, concrete counterexample, or genuinely missing independent bounded challenge. If another real local lane exists, refill from it and continue; only after this broad inventory may repository-wide local technical queue exhaustion be asserted again.
8. **Conditional live** — currently `READY_LIVE: none`. Reopen only if new code/instrumentation/hypothesis/path condition creates a concrete unresolved self-owned TCP/UDP question inside standing authorization. Do not rerun old live rows for freshness.

## Repository-wide refill rules retained

For the broad lane 7 inventory, continue to check all 13 core surfaces rather than making a narrow PORT-only exhaustion claim:

1. `neko-reliable` UDP recovery: ACK ranges, future/unsent ACK, loss/retransmit, RTT/PTO, persistent congestion, Reno, fault simulation;
2. `neko-carrier` `CarrierState`: generation, validation, hysteresis, single-active, drain/fail/activate;
3. Concurrent Carrier Manager / health / migration-back;
4. FairScheduler / multi-stream / Session+stream flow-control accounting;
5. Carrier adapters: Memory/UDP/TCP close/error/resource semantics;
6. `SessionRuntime` lifecycle/resource/window/DeliveryAck accounting;
7. `neko-observe` projection/event/counter/high-water correctness;
8. package/reproducibility/operator scripts;
9. dependency/build: manifests/lock/features/build/native hooks/unsafe inheritance;
10. cross-platform CLI/process-test semantics;
11. CLI exit-code / JSON / human-output contract;
12. algorithmic resource boundedness, excluding capacity-pressure benchmark/new policy values;
13. release-packet factual consistency/evidence boundary.

Candidate A (future/unsent Recovery ACK) and Candidate B (mixed datagram drop reasons) remain closed unless current owner truth changes or a new counterexample appears. A prior no-finding note is not immunity from a real new counterexample, but unchanged code alone is not a reason to replay every review.

## Evidence boundary

- The H-I4-089 reviewer finding at `78050ab` is source/test/review-note inspection. **No reviewer-local tests were executed for that finding.**
- Developer-reported focused tests and exact-tree gates remain developer-local evidence at their own reachable anchors; repository-persisted provenance is distinct again from GitHub-hosted CI.
- An absent hosted run is neither success nor failure.
- No live WAN result or performance conclusion is added by H-I4-089.
- No fuzz claim is made for the finding because decoder/parser/crypto framing is unchanged.
- The previous `f0013f7` broad review remains useful for unchanged surfaces, but its repository-wide exhaustion conclusion is temporarily superseded until the reopened PORT chain and broad refill lane are complete.

## Stop / escalation conditions

H-I4-089 itself is **not** a policy/architecture stop: coding agent should repair and continue without maintainer interaction. Escalate/notify only for an unresolved BLOCKER/HIGH that cannot be automatically decided, a core Session/Carrier/ACK/crypto/wire architecture change, D019 or another true policy/value choice, destructive/canonical migration, out-of-standing-authorization operation, third-party/production/new-credential permission, maintainer-selected adversarial-load/benchmark conditions, or entry into a genuinely new release stage.
