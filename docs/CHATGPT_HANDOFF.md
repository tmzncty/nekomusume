# Nekomusume ChatGPT Handoff

Checked at: 2026-09-08 12:00 Asia/Shanghai
Repository main HEAD reviewed: `bc6e31c3a8ed9bbc9cce83874f46ec8485762e5e`
Previous reviewer handoff commit: `bc6e31c3a8ed9bbc9cce83874f46ec8485762e5e`
Previous checked implementation HEAD: `0130bce740a331260bec6606e159006e8fc3f847`
Current execution branch: `work/e1a-staged-accounting-20260907` at exact `bd43f5b466e5f4eb39ec516582dd1a8f49a89f61`
New developer repair: `bd43f5b` — `fix: make migration-back recovery evidence truthful`
Exact execution-head Rust CI: run `34185413504` — `success`
Exact reviewer-head Rust CI for `bc6e31c`: run `34182410892` — `success`
Historical partial-E2 branch retained: `work/continue-20260904` at `d271a99a2ab26abbcb146c411ba0fde697395abe`

## What changed

The coding agent consumed the previous review promptly, merged current reviewer `main`, and landed a coherent migration-back repair. This is healthy implementation/review iteration, not a stall.

Accepted repairs at exact `bd43f5b`:

- the server now requires `SessionId(7001)` together with path 1 / generation 1 / delivery epoch 1 before answering the recovery control request;
- `tcp_resumed` is emitted before the recovery owner is enabled;
- the final post-return application record is no longer placed in the pre-failure uncertain set;
- final accounting now includes the post-return UDP-confirmed record and no longer counts that record as replayed/uncertain;
- the fabricated UDP `100 us` and TCP `500 us` samples were removed;
- recovery RTT is derived from a live monotonic challenge interval;
- a tamper injection hook and a nominal negative test were added;
- exact-head CI is green.

These are meaningful fixes. Do **not** revert them or restart migration-back design from scratch.

However, the current exact head is still **not VPS-ready**. Review of the actual socket/runtime code found three live-evidence correctness gaps that can produce a false or incomparable migration-back conclusion despite green CI. They are ordinary local implementation fixes; no administrator decision, new wire message, crypto change, or new numeric policy is required.

## Review verdict

**CONTINUE_WITH_REQUIRED_LIVE-EVIDENCE_FIXES — preserve `bd43f5b`, repair exact-peer validation, comparable truthful health sampling, and terminal experiment closure; replace the false-positive negative test. Then run local real sockets, exact-head CI, and go directly to one bounded self-owned VPS migration-back observation.**

No administrator action is required. Standing authorization already covers the later bounded migration/recovery VPS run.

## Reviewer findings

### MIGBACK-010 — HIGH — client recovery proof does not verify the UDP source endpoint

The recovery client sends with `send_to(target)` but receives the challenge response with `u.recv(...)`, which discards source-address information. AEAD authentication prevents an unauthenticated third party from forging the response, but the reviewed recovery contract is stricter: migration validation must bind **exact peer + exact Session/path/generation/epoch + fresh challenge**. A valid authenticated datagram arriving from a different source endpoint must not become recovery-path validation.

**Required repair:** use `recv_from`, require `source == target` before accepting/opening the response for recovery success, and ignore/reject non-target datagrams without producing `validated`, health, migration, or post-return application evidence. Preserve the server's existing `source == udp_peer` check.

A small loopback negative or helper-level test for wrong-source recovery is appropriate if easy; do not build a new harness solely for this.

### MIGBACK-011 — HIGH — health is measured now, but the two samples are still not semantically comparable or truthful under retries

The hard-coded RTT constants are gone, but the current replacement still biases the migration gate:

- active TCP health uses `tcp_authenticated - tcp_connect_started`, i.e. connect + negotiation + Noise setup duration, not a live-path application/control RTT comparable to the UDP recovery challenge;
- that TCP sample sets `pto=1` even though this value is not derived from an observed PTO in the sampled exchange;
- UDP recovery repeatedly sends the **same sealed unreliable record** after 200 ms receive timeouts, while the accepted `HealthSample` still reports `loss_per_mille=0` and `pto=0`;
- the accepted UDP RTT is computed once for `manager.observe(...)` and then recomputed again for `MigrationCandidate`, so the manager sample and candidate can differ slightly.

This matters because `CarrierManager::migrate_back_to_udp` uses these health fields to authorize the switch. Green CI proves determinism, not that the score is fed comparable live observations.

**Preferred minimal repair:**

1. derive the active TCP sample from a successful resumed authenticated application/control round trip already present in the runtime, preferably first resumed Data send -> exact authenticated `DeliveryAck`; use the observed RTT and do not invent a PTO/loss event that did not occur;
2. make the recovery proof one-shot for this bounded experiment: one fresh sealed challenge, one bounded exact-peer authenticated response deadline. A one-shot success may truthfully carry zero observed retry/loss/PTO for that observation; a timeout is preserved negative evidence. This is simpler and safer than retransmitting one replay-protected ciphertext;
3. if the agent intentionally retains multi-attempt probing instead, each attempt must be fresh/replay-safe and timeouts/loss/PTO must be represented truthfully rather than collapsed to zero;
4. construct one `udp_recovery_sample` value after exact response acceptance and reuse that exact value for both `manager.observe(PathId(1), ...)` and `MigrationCandidate.health`;
5. emit bounded structured diagnostics for the active TCP sample and accepted UDP recovery sample so a future VPS artifact can retain the actual inputs to the migration decision.

No new global score weights, loss thresholds, PTO threshold, or dwell/margin policy may be invented here.

### MIGBACK-012 — HIGH — requested migration-back can still fall through to server success without recovery or post-return delivery

The server's recovery-control block is optional/fall-through. If no valid recovery challenge arrives before its deadline, or if the post-return UDP application record never arrives, the function continues into `failover_server_ok` / summary instead of terminally failing the requested migration-back experiment. In the no-post-return case, the application buffer can contain only the UDP-primary + TCP-resumed records while the structured summary still uses the configured `count` field.

This can turn a failed live migration-back attempt into a misleading server-side success artifact.

The implementation also aliases existing `--restore-udp-replies-after-tcp` and new `--migration-back` into one boolean. That silently changes the semantics of the older bounded restore seam and makes it harder to distinguish “server may answer UDP again” from “this run must prove a complete manager-authorized migration-back”.

**Required repair:** keep an explicit `migration_back` mode separate from the legacy restore seam. `--migration-back` may imply the server is willing to restore UDP control/application replies, but a requested migration-back run must track terminal milestones such as:

- exact recovery request authenticated;
- exact recovery response sent;
- post-return Data accepted after the client-side gate;
- post-return authenticated `DeliveryAck` sent.

If the requested sequence does not reach its required terminal milestone within the bounded deadline, return nonzero / explicit negative diagnostics and **do not emit a success summary claiming the configured record count**. Preserve ordinary historical failover/restore behavior when `--migration-back` is absent.

On the client, `--migration-back` should fail fast unless `--automatic-health-failover` is present and `count >= 3`, because the proof shape requires one UDP-primary record, at least one TCP-resumed record, and one previously-unassigned post-return UDP record.

### MIGBACK-013 — MEDIUM — the new tamper negative is a false-positive test

`migration_back_tamper_fails_closed_before_return` launches `failover-client` by itself and never starts the corresponding server. Therefore the process can fail during initial handshake/setup and still satisfy the test's only assertions (`status != success`, no migration success string). It does not prove the tampered recovery record was ever reached.

**Required repair:** turn this into a real process negative using the same bounded server/client setup as the positive migration test. Prove the run reaches accepted TCP fallback/resume first, then reaches `udp_recovery_challenge_sent`, then fails because the recovery proof is tampered (or wrong-peer/wrong-tuple). Assert:

- prior TCP resumed/DeliveryAck evidence exists;
- recovery challenge was attempted;
- no `migrated_back_to_udp` / equivalent manager success exists;
- no post-return UDP application `DeliveryAck` success exists;
- the client reports/retains TCP as active on recovery failure, or an equivalent explicit fail-closed state diagnostic exists;
- server does not claim a completed migration-back success/count after the failed recovery proof.

One good runtime negative is enough; do not duplicate all manager unit negatives.

### MIGBACK-014 — MEDIUM evidence instrumentation — measured decision inputs are not retained yet

The future VPS evidence contract requires the actual recovery-health input and migration boundary. Current stdout has `udp_recovery_challenge_sent`, plain `udp_recovered`, and `migrated_back_to_udp`, but does not retain a structured client-side accepted recovery RTT/health sample or the active TCP comparison sample used by the score gate.

Add minimal secret-safe structured events, for example semantics equivalent to:

- `tcp_active_health_observed { path=2, generation=1, rtt_us, loss_per_mille, pto }`;
- `udp_recovery_validated { path=1, generation=1, challenge_id, rtt_us, loss_per_mille, pto }`;
- `udp_migration_hold { active=tcp, ... }` on the first hold rejection;
- `udp_migrated_back { from=tcp/path2, to=udp/path1, generation=1 }`;
- `udp_recovery_failed { active=tcp, reason=<bounded category> }` for the negative path.

Names are flexible. Do not log addresses, keys, payload, or other protected material merely for this evidence.

## Accepted boundaries

- Reusing authenticated `ReadinessRequest/Response` as the bounded recovery-control carrier remains accepted; do not add a new wire kind for this lab seam.
- The one-shot migration-back experiment has one outstanding logical recovery challenge; existing unreliable-record replay protection plus this one-shot state is sufficient for the current bounded freshness shape. Do not turn this into an unbounded challenge history.
- Recovery validation, health score, packet feedback, and Session `DeliveryAck` remain separate evidence domains.
- Generation 0 UDP evidence must never be revived after TCP generation 1 becomes active; recovered UDP is path 1 / generation 1 with fresh evidence.
- Single-active remains mandatory. The post-return record is unassigned before migration and may be sent only after manager authorization; no new TCP application data after successful return.
- C2 source-retention policy remains an explicit release/security limitation and does not block this runtime lane. Do not invent TTL/LRU/history/epoch values.
- `RELEASE_CANDIDATE=false`, `PRODUCTION_READY=false`, `FREEZE=false`, `RELEASED=false` remain unchanged.
- Exact `bd43f5b` is local runtime/CI evidence only. It is not WAN migration-back evidence.
- Protected identities, SSH keys, credentials, private endpoints, and raw private diagnostics remain unread/untracked/uncommitted.

## Rolling Work Queue

This remains a continuous pre-authorized queue. Finish coherent closure -> focused/full gates -> commit -> push -> continue immediately. Reviewer cadence is not a work ticket length.

### A — Finish truthful migration-back runtime closure

**Status:** `READY_LOCAL`; highest priority.

Repair MIGBACK-010 through MIGBACK-014, preferably in one coherent implementation/test closure rather than five tiny checker commits.

**Primary files:** `crates/neko-cli/src/main.rs`, `crates/neko-cli/tests/probe.rs`; touch `neko-carrier` only if a genuinely minimal typed observation helper is needed.

**Protected invariants:** exact peer/tuple/challenge; measured comparable health; no fabricated score inputs; single-active; final record unassigned before return; failure cannot emit success summary; legacy failover semantics preserved without `--migration-back`.

**Gate:** focused process/carrier tests, real loopback TCP/UDP sockets, `./scripts/check.sh`, `git diff --check`. No fuzz rerun unless parser/wire decoding changes.

**Commit/push:** required.

**Continue immediately to B:** yes.

### B — Exact local positive + recovery-specific negative

**Status:** `PREAUTHORIZED_AFTER_A`.

Positive minimum shape: 3 small records; first confirmed on UDP; middle assigned/uncertain then replayed+confirmed on TCP after scripted application-reply cessation; measured resumed TCP health; one exact-peer authenticated generation-1 UDP recovery challenge; measured UDP recovery health; hold rejection while TCP stays active; manager-authorized migration; final previously-unassigned record sent on UDP; exact authenticated final DeliveryAck; consistent accounting and cleanup.

Negative minimum shape: reach TCP resumed state, then tamper or otherwise invalidate the recovery proof; no migration/post-return DeliveryAck/success summary; explicit TCP-active failure state.

**Exact-head Rust CI must be green before C.**

**Continue immediately to C:** yes.

### C — One bounded self-owned VPS migration-back observation

**Status:** `PREAUTHORIZED_AFTER_B_GREEN`; do not ask for another WAN authorization.

Use the smallest workload that crosses the complete sequence, preferably 3 small records, one scripted application-level UDP reply cessation, one recovery challenge and one post-return UDP record. Remain within standing authorization: self-owned endpoints, temporary unprivileged listeners, <=10 minutes, <=256 MiB, <=32 sessions, bounded capture only if needed, explicit cleanup, no production route/firewall/DNS/proxy/tunnel/qdisc changes.

Retain exact commit/binary identity, actual parameters, start/end/duration, scripted fault/recovery class, measured TCP and UDP health inputs, exact peer/path/generation transitions without recording protected address material in Git, hold/migration events, attempted/confirmed/uncertain/replayed/duplicate/lost/conflicting counts, client/server exit, cheap resource observations when already available, and cleanup.

**Claim boundary:** one bounded scripted self-owned fallback/recovery/migration-back observation only. It does not establish natural path recovery, reliability rate, public reachability, superiority, or production readiness.

Preserve first meaningful negative; no unchanged retry.

**Continue immediately to D:** yes.

### D — Reconcile planning/status truth after live result

**Status:** `PREAUTHORIZED_AFTER_C`.

Update only facts actually answered by C in `ROADMAP.md`, `IMPLEMENTATION_PLAN.md`, `docs/status.md`, and evidence references. Current planning still incorrectly calls migration-back `BLOCKED_IMPLEMENTATION`; change that only after the corresponding exact local/live evidence exists.

In the same reconciliation, repair the stale repeated-failover text that still says the next seam is to enter the Python runner. Later exact `9fd2411` / `a117086` already reached the structured outer runner; the remaining useful blocker is bounded sanitized inner-collector failure categorization. Do not rerun those historical negatives unchanged.

**Continue immediately to E:** yes.

### E — Integrate reviewed migration-back lineage to main

**Status:** `PREAUTHORIZED_AFTER_D`.

Once A-D are internally consistent and exact-green, merge/reconcile the implementation/evidence lineage with reviewer `main`. Preserve history and retained negatives; no force push. `docs/CHATGPT_HANDOFF.md` remains reviewer-owned/read-only for the coding agent.

**Continue immediately to F:** yes.

### F — Next VPS-unlock runtime seam

**Status:** `READY_LOCAL_AFTER_E`; proposal authority applies.

Prefer, in order:

1. authenticated endpoint/source migration if the owned environment can produce a genuine endpoint change without production-route mutation;
2. integrate existing authenticated PLPMTUD state into a live probe/ACK path without trusting unauthenticated ICMP;
3. another actual `BLOCKED_IMPLEMENTATION` release row with a short path to self-owned VPS evidence.

Do not select FEC, 0-RTT, striping, heterogeneous aggregation, ICMP/raw custom carriers, or other experimental-track work merely to fill the queue.

**Continue immediately to G:** yes.

### G — Local proof + one materially distinct VPS observation for F

**Status:** `PREAUTHORIZED_AFTER_F_GREEN`.

Real local sockets -> exact-head green -> one minimal materially distinct self-owned VPS observation if the seam is truthfully `READY_LIVE`; retain negative evidence and cleanup; no unchanged retry.

### H — Repeated warm-failover diagnostic fallback

**Status:** `READY_LOCAL_FALLBACK`; only while unrelated CI is pending or if the runtime lane is genuinely blocked.

Do not rerun exact `9fd2411` / `a117086` unchanged. The useful local change is bounded sanitized inner-collector failure categorization propagated into outer structured evidence, followed by synthetic/dry verification. Only changed instrumentation may justify one later materially changed live attempt. Keep this behind missing runtime capabilities so harness work does not become the project again.
