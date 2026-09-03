# Nekomusume ChatGPT Handoff

Checked at: 2026-09-03 17:00 Asia/Shanghai
Repository HEAD reviewed: `12941fabb4726c99d98cd7f225e5b236564c7bb6`
Previous reviewed implementation HEAD: `3d545859e06690c528a717015c9b7023d05ea420`
Previous reviewer handoff commit: `4607c752c9ba8051fa7eb54c88b4d31c749ebe89`

## What changed

The latest coding-agent result is the exact-`3d54585` HY2/Nekomusume paid attempt retained by `12941fa`.

The attempt made real progress but did not produce a comparison:

- `nekomusume-1` completed successfully with the 1,200-byte deterministic payload;
- `hy2-1` exited with `client_exit` and zero application bytes;
- the overall result is `BLOCKED_HARNESS` at `hy2-1-failed`;
- there are no complete pairs and therefore no comparative median/P95 or superiority evidence;
- automatic cleanup was conservatively recorded failed only because `remote_process_groups_reaped=false`, while listeners were zero, remote temp removal/local cleanup succeeded, and later serialized postchecks found no experiment residue without rewriting the original artifact.

The retained HY2 artifact does **not** include enough durable failure-cause detail to distinguish the HY2 client exit into a concrete QUIC/TLS/auth/config/path class. It records `client_exit`, exit code, timing/resource fields and the valid sample prefix, but not a sufficiently specific transport/client error reason.

This reveals the current process problem: the repository has spent many review/repair cycles on one HY2 benchmark gate (admission, SSH preflight, evidence deadline, listener readiness, result validation, cleanup, then client exit), while the release-evidence matrix contains other independent READY or directly-unlockable rows. `IMPLEMENTATION_PLAN.md` explicitly says a blocked network row must not block independent work.

## Review verdict

**SAFE_TO_CONTINUE — stop treating HY2 as the release-matrix primary; pivot mainline to other VPS evidence while making HY2 diagnosis self-contained**

The project is not globally blocked. HY2 comparison remains an open matrix row, but it is now a **side diagnostic track**, not the main scheduler gate.

Do not perform another paid HY2 retry until the harness can preserve a useful sanitized client/server failure reason or path classification from one invocation. Do not spend another full review cycle only on cleanup bookkeeping unless a new cleanup defect can actually leave owned resources behind.

The VPS rental window is time-limited. Mainline work must now advance a different release-evidence row or the smallest implementation seam that unlocks one.

## Evidence boundaries

- `IMPLEMENTATION_COMPLETE=true` remains the bounded research baseline.
- `CANONICAL_CORPUS_V1_FROZEN=true` remains corpus-specific only.
- `RELEASE_CANDIDATE=false`, `PRODUCTION_READY=false`, `FREEZE=false`, and `RELEASED=false` remain correct.
- Exact `25e0daa` controlled application-level UDP reply-cessation warm fallback remains one bounded positive cross-host failover result; it is not natural packet-loss/PTO blackhole evidence.
- Exact `25e0daa` periodic direct-path run remains one approximately five-minute positive sample; it is not a general long-lived reliability result.
- Exact `3d54585` HY2 attempt proves only a valid two-sample prefix: Nekomusume success then HY2 `client_exit`. It is not comparative performance evidence.
- The later post-run cleanup observations do not retroactively change the artifact's conservative cleanup failure.
- IPv6 remains environment-blocked unless a genuinely owned end-to-end IPv6 path exists.
- Natural UDP degradation/PTO-blackhole, NAT/source-endpoint change, live migration-back, real-session key update and live PMTUD remain open unless current runtime/CLI surfaces can demonstrate them truthfully.
- HY2 comparison remains open, but one row may be blocked without blocking the rest of the release-evidence matrix.

## Work Package — Release Matrix Diversification + HY2 Observability Side Track

### Primary A — Audit and execute the next non-HY2 VPS-only release-evidence row

**Goal**

Use the rented VPS for a different high-value release-evidence question instead of continuing HY2 harness thrash.

**Required first step: executable-surface audit**

Inspect the current CLI/runtime/tests/events and classify each candidate below as:

```text
READY_LIVE
BLOCKED_IMPLEMENTATION
BLOCKED_ENVIRONMENT
ALREADY_SUFFICIENT_FOR_CURRENT_BOUNDARY
```

Candidates, in priority order:

1. genuine NAT/source-endpoint change;
2. real-session migration-back after carrier recovery;
3. real-session key update on a live authenticated Session;
4. live-path PMTUD observation;
5. repeated real-socket lifecycle/open-exchange-close with a scientifically distinct question from the accepted five-minute periodic row;
6. natural/transport-level UDP degradation -> TCP fallback if a real loss/PTO/blackhole seam exists that is not the already-accepted application-level reply-cessation seam.

Do not choose by label alone. A fixture/state-model/API without a live executable path is `BLOCKED_IMPLEMENTATION`, not VPS evidence.

**If any candidate is READY_LIVE:**

Execute exactly one bounded self-owned VPS experiment under standing authorization, with:

- exact git/binary identity;
- actual parameters;
- experiment ID;
- structured events/logs;
- CPU/RSS/FD/socket metrics where meaningful;
- cleanup verification;
- explicit statement of what the run does and does not prove.

Stay within the standing 10-minute / 256 MiB / 32-session boundary. Preserve negative results.

**If none is READY_LIVE:**

Implement the **smallest local runtime/instrumentation seam** that directly unlocks the highest-priority candidate. This is allowed to be code work, but it must be narrowly tied to one named release-evidence row. Add deterministic tests, run the normal gates, push, and if the row becomes READY_LIVE in the same package, execute one bounded VPS run immediately.

Do not invent a new protocol feature merely to create work.

### Follow-up B — HY2 failure-cause observability repair, no paid retry yet

**Dependency:** Primary A has either produced one new VPS row or a concrete smallest unlock seam.

Repair the HY2 benchmark evidence contract so the next `client_exit` is diagnosable without another reviewer round.

The next blocked artifact must be able to preserve a **sanitized bounded diagnostic** such as:

- HY2 client exit code;
- normalized failure class (`dns`, `connect`, `quic_handshake`, `tls_pin`, `auth`, `local_forward`, `application_echo`, `timeout`, `unknown_client_exit`);
- a short redacted stderr/log digest or hash plus a safe bounded excerpt that contains no endpoint address, password, certificate private material or private topology;
- whether client UDP packets were observed leaving;
- whether they reached the VPS experiment port;
- whether server responses were observed;
- whether responses returned;
- HY2 server/client log stage where available.

The contract must fail closed: if detailed logs cannot be retained safely, classify `unknown_client_exit` rather than dropping the error entirely.

Add deterministic regressions for at least:

- HY2 client exits before payload;
- TLS/auth-style stderr classification;
- timeout/path-style classification;
- secret/address redaction;
- blocked artifact still contains no comparative summary.

Do not change Nekomusume wire/Session/Noise semantics for this task.

### Follow-up C — Reconcile release-matrix status after Primary A

**Dependency:** A complete, positive or negative.

Update `docs/status.md`, `IMPLEMENTATION_PLAN.md`, `ROADMAP.md` and the appropriate compact evidence note/index so the matrix reflects:

- the exact new non-HY2 row result or exact `BLOCKED_IMPLEMENTATION` seam;
- the existing D064/periodic evidence without overclaim;
- the latest HY2 `client_exit` as a retained negative;
- HY2 still open but no longer treated as the sole next executable release task.

Do not erase historical negatives. Do not change governance flags.

### Follow-up D — Only if HY2 observability is now materially better, permit one later changed-hypothesis retry

**Dependency:** B complete, full local gate green, and exact-head CI green.

A new paid HY2 attempt is allowed only if at least one diagnostic variable materially changed and the next failure would produce more information than `client_exit` alone.

If retried:

- exactly one substantive invocation;
- same fair-lifecycle/security contract;
- unique artifact path;
- bounded packet-direction observation if needed;
- complete pairs only may enter comparison statistics;
- on failure, retain the richer diagnostic and then stop HY2 again rather than opening another immediate repair/retry loop.

HY2 must not preempt another READY VPS-only release row merely because it is unfinished.

### Follow-up E — Begin independent release/security review prep in parallel with blocked environment rows

**Dependency:** A/C complete; lower priority than VPS-only evidence but no longer wait for every network row if some are environment-blocked.

Prepare a bounded internal pre-review package for the later independent review:

- resource/abuse limits map;
- compatibility/version policy evidence;
- package rollback/readiness evidence;
- canonical vector/freeze references;
- operator lifecycle/cleanup evidence;
- list of unresolved release-matrix rows with precise status (`positive`, `negative`, `blocked implementation`, `blocked environment`, `open comparison`).

This is preparation, not an independent security review and not RC approval.

## Fallback

If the executable-surface audit shows every non-HY2 network row is truly blocked by missing runtime support:

1. rank the missing seams by smallest implementation cost and highest VPS evidence value;
2. implement only the first direct unlock seam;
3. do not spend the hour polishing HY2 unless the HY2 observability repair is demonstrably smaller and will produce a materially more informative next run;
4. continue local release/security-review preparation rather than idling.

If a candidate requires new credentials, another server, third-party access, production route/firewall/qdisc changes or anything outside standing authorization, mark only that row blocked and continue the next row.

## Completion gates

- HY2 is no longer the sole scheduler gate for release evidence;
- at least one non-HY2 release-evidence row is either executed on VPS or reduced to a concrete `BLOCKED_IMPLEMENTATION` + smallest direct unlock seam;
- no fixture/state-model result is promoted to live evidence;
- HY2 next-failure observability can preserve a safe concrete reason beyond bare `client_exit`, or it remains explicitly blocked for diagnosis without consuming repeated paid attempts;
- status/plan/roadmap reflect the diversified release matrix truthfully;
- standing authorization is used directly for in-scope VPS work;
- governance flags remain unchanged.

## Do not expand into

- repeated unchanged HY2 paid attempts;
- endless cleanup/preflight micro-polish that does not change experimental information value;
- third-party targets, scanning or production network changes;
- enabled FEC, 0-RTT, striping/aggregation or exotic carriers without an observed-problem gate;
- declaring RC/security/production readiness from bounded self-owned evidence;
- treating environment-blocked IPv6 as a reason to stop unrelated work.

## Questions requiring maintainer decision

none.

The slowdown was caused by scheduling concentration on one benchmark row, not by a global project blocker. This handoff intentionally restores the release-evidence matrix as a multi-row queue and prioritizes evidence value per remaining VPS rental day.