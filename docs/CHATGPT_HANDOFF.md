# Nekomusume ChatGPT Handoff

Checked at: 2026-09-01 21:59 Asia/Shanghai
Repository HEAD: `1eed79c01c82f723beb496100c5d3ebf74a970e4`
Previous checked implementation HEAD: `9d890510c5b694b71f33e13aa68937bcf4f97814`
Previous reviewer handoff commit: `ac4b30eb12872c4dbd91eb86ad24d9bf698e04f7`

## What changed

One substantive coding-agent commit landed after the previous reviewer handoff:

- `1eed79c` — **runtime implementation + manager repair + codec/tests + ADR/status/plan update; no new VPS evidence**. It replaces the prior client-local synthetic D064 readiness observations with three encrypted `ReadinessRequest` / `ReadinessResponse` exchanges on the pre-established TCP fallback, binds Session/path generation/delivery epoch/challenge ID, derives the peer `admitted` bit from bounded server runtime state, resets manager readiness on wrong candidate dimensions, separates a common experiment timing origin from the UDP failure-observation start, adds exact-length runtime-codec tests, and updates D064/status text. The existing loopback warm failover process test now necessarily traverses real peer request/response I/O before it can succeed.

This closes the central provenance defect from the previous review: the official warm client no longer promotes from three locally asserted `true` booleans. It also materially improves timing provenance by recording readiness request/response times from an origin created before warm preparation.

However, the current exact HEAD is **not yet ready for a D064 warm VPS evidence claim**. Review of the actual server path found two concrete fail-closed/boundedness defects and one verification gap introduced/exposed by the new peer-controlled runtime codec. These should be repaired first; then the rented-VPS window should be spent immediately on current-head warm/cold evidence rather than more unrelated local polish.

No GitHub commit-status/CI checks are attached to `1eed79c` through the available status API. Repository-local checks, if run by the coding environment, remain local evidence rather than independent CI attestation.

## Review verdict

**NEEDS REPAIR — accept the real authenticated readiness-exchange design, but close server admission/deadline semantics and the new parser verification gate before current-head VPS warm claims.**

The project is not globally blocked. The bounded release-evidence matrix remains the active phase. The fastest truthful sequence is:

```text
server-side D064 fail-closed + timing repair
    -> negative/process/parser verification + full local gates
    -> exact-head owned-VPS warm/cold recovery sample
    -> current-head periodic/resource evidence
    -> HY2 equal-application paired sample
```

The current VPS rental window is time-limited, so once the local correctness gate below is green, do not spend another cycle on unrelated documentation or speculative features.

## Review findings

### R-101 PASS — peer proof now exists before official warm promotion

The official warm client sends three bounded encrypted readiness requests and only advances manager readiness after decrypting a response with the exact Session/path-generation/delivery-epoch/challenge tuple and `admitted=true`. The peer computes admission from live bounded runtime state instead of accepting a caller-supplied boolean. This resolves the previous review's synthetic-observation defect for the normal executable path.

The manager also now clears accumulated readiness when a stale/wrong candidate dimension is supplied, matching the documented consecutive-readiness reset policy more closely.

### R-102 HIGH — the server does not fail closed after an unadmitted readiness response

`failover_server` computes `admitted` for each readiness request and returns that bit to the client, but after the three-response loop it unconditionally emits:

```text
carrier_event name=tcp_resource_admitted ...
```

and then enters the TCP application-data receive loop. There is no server-side `all_readiness_admitted`/candidate-admitted gate before application data.

The official client aborts on `admitted=false`, so the happy path is not affected. But an authenticated custom peer can send an exact challenge sequence containing a wrong Session/path/generation/epoch tuple, receive `admitted=false`, continue sending the remaining readiness messages, and still reach the server's application-data loop. That makes resource admission advisory on the responder and makes the unconditional `tcp_resource_admitted` event potentially false.

This is a D064 fail-closed correctness/evidence defect. The responder must retain a bounded aggregate admission result and refuse/close before application data if any required readiness observation was unadmitted or invalid. `tcp_resource_admitted` must be emitted only after the complete accepted sequence.

### R-103 HIGH — the documented one-second readiness deadline is currently client-side only

D064 now documents a one-second readiness response deadline and bounded control resources. The client sets `tcp.set_read_timeout(Some(READINESS_DEADLINE))`, but the accepted server `TcpStream` does not set a corresponding read deadline before negotiation/handshake/readiness reads. Once `tcp.accept()` returns, `read_frame` can therefore block beyond the failover server's outer `duration` loop if an authenticated or partially authenticated peer stops sending.

At minimum, the server's readiness phase must have a bounded read deadline consistent with the documented D064 contract. Prefer also bounding the accepted failover connection's negotiation/handshake/data reads by the remaining experiment duration so the CLI's advertised finite duration cannot be defeated by a stalled accepted stream. Keep this scoped to the bounded research runner; do not invent an async service architecture.

### R-104 HIGH — `failure_observation_elapsed_us` changed meaning and is now mislabeled

The common experiment origin was the right repair, but the health diagnostic currently reports both:

```text
failure_observation_started_us
failure_observation_elapsed_us
```

relative to `experiment_origin`. `failure_observation_started_us` should be an absolute-relative timestamp from the common origin. `failure_observation_elapsed_us` should remain the elapsed duration of that observation window (for example `now - observation_started`), not another absolute-relative timestamp.

As written, later observation windows can report a large cumulative "elapsed" value that includes all earlier warm preparation and prior windows. That can corrupt recovery/evidence analysis. Repair the field semantics or rename it if a different quantity is intended; do not silently change the established diagnostic meaning.

### R-105 MEDIUM — the new peer-controlled `ProcessMessage` kinds are not covered by the repository fuzz target

`ReadinessRequest` and `ReadinessResponse` extend the runtime/failover `ProcessMessage::decode` surface with network-controlled bytes. Unit tests cover exact roundtrip, every truncation, trailing bytes and the boolean byte, which is good. But the repository's current `fuzz/fuzz_targets/decode.rs` exercises `neko-wire` record decoding and negotiation only; it does not fuzz `neko_session::ProcessMessage::decode`.

`AGENTS.md` requires fuzz smoke when external parser/decode behavior changes. Extend the existing fuzz target or add the smallest dedicated target so arbitrary bytes exercise `ProcessMessage::decode` under panic-free/bounded roundtrip properties, seed it with the new readiness messages, and run the relevant bounded fuzz smoke. This is a verification gate, not evidence of a known parser exploit.

### R-106 MEDIUM — happy-path process coverage exists, but negative readiness I/O is not yet demonstrated end-to-end

The existing warm loopback process test now traverses real request/response I/O because the executable path requires it, so the happy path is meaningful. Manager tests cover several stale/generation/reset cases and codec tests cover byte-shape failures.

What is still missing is process-level proof that peer failures cannot produce warm/admitted/data state: wrong tuple, `admitted=false`, malformed/tampered readiness ciphertext, wrong/replayed challenge, timeout/stall, and fewer than three valid responses should all fail closed at the correct layer. Add focused process tests around the smallest seam needed; do not build a giant adversarial framework.

### R-107 NOTE — N9 remains closed and does not need reopening for this runtime codec

`CANONICAL_CORPUS_V1_FROZEN=true` is explicitly corpus-scoped and excludes failover/resume/runtime process messages. The new readiness codec is documented in the D064 runtime ADR and has its own tests. Do not mutate the frozen N9 corpus merely because runtime control messages were added. If the runtime codec contract needs deterministic vectors, keep them in the failover/runtime evidence surface rather than silently broadening the frozen corpus scope.

### R-108 NOTE — VPS backlog remains high-value and READY immediately after repair

Older self-owned evidence already proves real IPv4 TCP/UDP current/current behavior, controlled cold resume, bounded lifecycle samples, resource sampling and periodic-session behavior at older exact commits. It does **not** prove the new D064 warm path at `1eed79c` or its repaired successor.

IPv6 remains environment-blocked because the owned server path lacked a global IPv6 address/default route; do not mechanically rerun that unchanged failure. HY2 v2.9.3 remains pinned and the application-forwarding comparison seam is documented, but no valid equal-application paired result has landed yet.

## Evidence boundaries

- `RELEASE_CANDIDATE=false`, global `FREEZE=false`, `PRODUCTION_READY=false`, and `RELEASED=false` remain correct.
- `CANONICAL_CORPUS_V1_FROZEN=true` remains a corpus-specific fact only.
- `1eed79c` is a real runtime correctness improvement and carries real local peer readiness I/O, but it has **no current-exact-head VPS warm evidence**.
- The current server can still enter application-data handling after an unadmitted readiness sequence if a custom authenticated peer ignores `admitted=false`; do not call that responder fail-closed yet.
- The current server-side readiness wait is not demonstrably bounded to the documented one-second deadline.
- Current health timing diagnostics contain a mislabeled elapsed field; do not use the current exact-head timing output as reviewed warm/cold measurement evidence.
- No independent GitHub CI attestation is attached to current HEAD.
- Existing older VPS rows remain valid only for their exact commits/scenarios and are not substitutes for current-head warm evidence.
- Standing authorization covers the owned-endpoint TCP/UDP failover, bounded periodic Session, resource sampling, cleanup and HY2 paired work below; no new maintainer permission is required.
- The VPS rental window remains a prioritization constraint: after A-C are green, VPS-only evidence outranks unrelated local polish.

## Work Package — finish D064 fail-closed semantics, then immediately harvest current-head VPS evidence

Execute A -> B -> C -> D -> E in dependency order. This is intentionally a full engineering/evidence batch, not a ten-minute repair ticket. If A/B are completed quickly, continue directly into C and D on the same package.

### Primary A — make responder admission/deadlines and timing diagnostics fail closed

**Goal:** make the new D064 peer readiness exchange truthful on both sides before another warm VPS claim.

Required behavior:

1. Track the complete server readiness sequence. Application-data receive/resume must be reachable only after all three required current-tuple readiness requests were successfully authenticated, correctly ordered and `admitted=true`.
2. If any required request is malformed, unauthenticated, wrong tuple/generation/session/epoch, duplicate/non-consecutive, unadmitted, or times out:
   - do not emit `tcp_resource_admitted`;
   - do not enter the TCP application-data loop for that candidate;
   - terminate/fail the candidate within the bounded research-runner contract;
   - retain a truthful diagnostic/error classification where practical.
3. Emit `tcp_resource_admitted` only after the third accepted peer exchange. Include enough non-secret tuple/provenance fields in structured diagnostics to correlate it with the candidate generation and final challenge without relying on a prose claim.
4. Put a real server-side readiness read deadline on the accepted TCP stream. The D064 phase must not block indefinitely waiting for a readiness frame.
5. Preserve the failover runner's overall finite-duration boundary. If accepted-stream negotiation/handshake/data reads can presently defeat that duration, add the smallest remaining-duration read/write timeout plumbing needed to keep the bounded CLI truthful. Do not turn this into a general daemon/runtime redesign.
6. Fix health diagnostic time semantics:
   - `failure_observation_started_us` = timestamp from common experiment origin;
   - `failure_observation_elapsed_us` = elapsed duration of that observation window;
   - warm connect/auth/resume/readiness, failure decision, promotion and first resumed-data timestamps remain on the common origin;
   - `readiness_satisfied_us` remains the actual third accepted readiness response, not authentication time.
7. Keep cold and warm classification distinct. Do not make cold recovery advertise warm-only readiness completion fields.
8. Do not change the frozen canonical N9 corpus or protocol bytes unrelated to this runtime control path.

### Follow-up B — close negative process tests and the new parser/fuzz gate

**Dependency:** A implemented.

Add focused deterministic tests that exercise the actual peer control path, not only manager setters.

At minimum prove:

- normal warm loopback produces exactly three authenticated peer readiness responses before UDP failure decision and before any TCP application data;
- two accepted responses are insufficient to make the manager/candidate warm;
- wrong Session/path/generation/delivery epoch receives or results in unadmitted/fail-closed behavior and no server `tcp_resource_admitted`/application success;
- duplicate or wrong challenge cannot count toward warm readiness;
- malformed/tampered/unauthenticated readiness ciphertext fails closed;
- readiness timeout/stalled peer respects the server-side deadline and does not hang beyond the bounded runner contract;
- one unadmitted observation followed by later valid-looking requests cannot reach server application-data acceptance;
- manager consecutive-readiness reset remains correct after a failed current-candidate validation;
- UDP remains the sole application-data owner before failure/promotion;
- cold fallback remains functional and separately classified;
- uncertain resend + exact authenticated Session `DeliveryAck` + receiver dedup/conflict behavior still passes after the warm repair;
- parsed timing fields satisfy the real ordering invariant:

```text
tcp connect/auth/resume/readiness_satisfied
    < failure_decided_at
    <= new_active_at
    < first_resumed_data_accepted_at
```

and each `failure_observation_elapsed_us` is a window duration rather than cumulative experiment time.

For the new peer-controlled runtime codec:

1. extend/add bounded fuzz coverage for `neko_session::ProcessMessage::decode`;
2. on successful decode, require re-encode/canonical roundtrip where valid;
3. seed readiness request/response examples if the existing smoke corpus needs them;
4. preserve panic-free, bounded-allocation behavior for arbitrary bytes;
5. run the relevant bounded fuzz smoke after the parser change.

Do not convert this into a new global wire-freeze project.

### Follow-up C — full local gate and exact-head release-evidence rehearsal

**Dependency:** A/B green.

Before VPS deployment, run the complete local gate on one exact commit:

- `cargo fmt --all -- --check`;
- `cargo check --workspace --locked`;
- `cargo test --workspace --all-targets --locked --no-fail-fast`;
- `cargo clippy --workspace --all-targets --locked -- -D warnings`;
- `bash scripts/check.sh`;
- relevant parser/process fuzz smoke;
- `git diff --check`.

Also verify the exact warm process output mechanically enough that the later VPS collector can reject:

- fewer/more than three readiness successes;
- any `admitted=false`;
- `tcp_resource_admitted` before the third valid response;
- application data before promotion;
- non-monotonic or semantically invalid timing fields.

Update `docs/status.md`/D064 wording only if behavior changed materially. Keep the release-evidence matrix open and all RC/security/production/global-freeze flags unchanged.

### Follow-up D — current-exact-head owned-VPS D064 warm/cold + periodic/resource batch

**Dependency:** A-C green. Build/deploy one exact commit and record binary SHA-256. Reuse that exact binary across compatible rows.

This is the highest-value rental-window work once correctness is green.

#### D1 — paired warm/cold recovery sample on real self-owned sockets

Run a bounded sample that is large enough to support the D064 warm/cold measurement contract without becoming a stress test. Prefer an interleaved batch such as **5 warm + 5 cold** recoveries if the complete batch remains comfortably inside the standing wall-clock/traffic/concurrency limits.

For every run record:

- exact git commit / binary hash;
- experiment ID, ports, count/bytes/duration and endpoint-ownership class;
- canonical negotiation and Noise authentication success without secrets;
- warm: all three real peer readiness response IDs/timestamps, admission result and `warm_eligible_at` before failure decision;
- proof UDP remained sole data owner before failure;
- controlled UDP reply-cessation classification and threshold/failure decision timestamp;
- warm vs cold fallback class and promotion gate;
- uncertain range/resend and exact authenticated DeliveryAck result;
- final logical records/bytes, missing/duplicate/conflict counts;
- recovery latency per run and warm/cold median/P95 where sample size supports it;
- process CPU/RSS/FD/socket sample where available;
- cleanup state and retained failures.

Classification must remain: **controlled self-owned application-level UDP reply cessation -> threshold decision -> authenticated TCP recovery**. It is not natural Internet blackhole evidence, public reachability, production failover, or security approval.

Do not repeat unchanged failures merely to chase a prettier latency distribution.

#### D2 — current-head periodic real-socket/resource sample

After D1 cleanup, if the existing periodic runner/resource sampler is still compatible, run one scientifically distinct ~5-minute authenticated periodic Session on the same exact current binary. Keep low traffic and concurrency 1. Record process CPU/RSS/FD/socket observations, records/bytes, failures, missing/duplicate delivery and cleanup. This updates the rented-VPS resilience archive to the new runtime baseline; it is not production long-lived proof.

Do not retry the unchanged IPv6 blocker.

### Follow-up E — first valid HY2 equal-application paired sample

**Dependency:** current exact-head local gate green. May proceed after D1/D2 cleanup; do not run CPU-heavy build/fuzz concurrently with performance sampling.

Reuse the pinned HY2 v2.9.3 artifact and the existing forwarding research seam. Do not read/reuse production Hysteria secrets and do not stop/reconfigure the existing Hysteria service.

The repository's existing `scripts/bench/compare-hy2.sh` is intentionally loopback-only. **Do not weaken that guard.** If no self-owned-VPS comparison orchestrator exists yet, implement the smallest separate fail-closed orchestrator/adapter that:

- accepts only the explicitly configured owned lab endpoint contract;
- creates experiment-only high ports/config/certificate/auth material;
- starts a temporary HY2 server and client TCP-forwarding path plus temporary loopback echo target;
- invokes an equivalent Nekomusume authenticated echo command;
- uses the same deterministic payload file/length/SHA-256 and same application question: send exact bytes -> receive exact echo;
- records same VPS/client, close time window, route/MTU metadata, authenticated-encrypted security class, single-stream/load shape, finite timeout and run count;
- emits raw samples plus median/P95/failures, CPU user/system, max RSS, FD count and application bytes; `wire_bytes=null` unless capture provenance is trustworthy;
- traps cleanup and verifies temporary listeners/processes/files are gone.

Prefer a small interleaved/nearby paired sample (for example 5 Nekomusume + 5 HY2 runs) rather than two widely separated blocks if orchestration permits. Preserve slower/failed Nekomusume results exactly. Make **no superiority claim** from this first bounded sample.

If a fair pair is genuinely blocked by an implementation/environment gap, record the exact blocker and use remaining VPS time for another already-defined current-head evidence row, not speculative features.

After D/E, reconcile release evidence/status/navigation only to what actually ran. `IMPLEMENTATION_PLAN.md` item 3 remains open until the remaining genuine matrix rows are either evidenced or explicitly reviewed as environment-inapplicable.

## Completion gates

This package is complete only when all of the following are true:

- server application data cannot proceed after any failed/unadmitted readiness sequence;
- `tcp_resource_admitted` is emitted only after the complete accepted readiness gate;
- server-side readiness/accepted-stream waits respect finite bounds;
- `failure_observation_elapsed_us` again means elapsed window duration;
- the official warm path requires three real authenticated exact-tuple peer responses and keeps UDP as sole data owner before promotion;
- negative process tests prove malformed/tampered/wrong/unadmitted/stalled readiness cannot create warm/admitted/data state;
- new `ProcessMessage` decode kinds have bounded fuzz/property coverage and the relevant fuzz smoke passes;
- full local workspace gates pass on the exact VPS candidate commit;
- current-head real owned-VPS warm evidence exists only if D1 proves pre-failure peer readiness and correct timing;
- cold results remain separately classified;
- current-head periodic/resource evidence is preserved if D2 runs;
- HY2 comparison is marked complete only if an equal-application paired sample really runs;
- IPv6 remains environment-blocked unless the actual owned path changes;
- negative and superseded evidence are retained;
- `RELEASE_CANDIDATE=false`, global `FREEZE=false`, `PRODUCTION_READY=false`, and `RELEASED=false` remain unchanged.

## Fallback

If A/B exposes a deeper failover/runtime correctness problem:

1. keep release/VPS warm claims blocked for the affected path;
2. preserve a minimal deterministic reproducer and exact failing state;
3. repair correctness before D064 warm evidence;
4. continue any **independent** VPS evidence row only if it does not rely on the broken path and answers a genuinely new exact-head question;
5. otherwise spend the cycle on the HY2 owned-VPS orchestrator/adapter or other local instrumentation that directly unlocks the next valid VPS experiment.

If the environment blocks fair HY2 comparison, do not modify production Hysteria, firewall, route or tunnel configuration to force it.

## Do not expand into

- changing the frozen N9 corpus for runtime readiness messages;
- concurrent TCP+UDP application-data striping/aggregation;
- enabled FEC, 0-RTT or exotic carriers without an observed-problem gate;
- third-party targets, scanning or production network changes;
- repeated IPv6 probes without a real path change;
- >10-minute single experiments or high-volume/high-concurrency stress outside standing authorization;
- performance superiority claims from first bounded paired samples;
- RC/security/production approval before the independent review gate.

## Questions requiring maintainer decision

none.
