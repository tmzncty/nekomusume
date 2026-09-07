# Nekomusume ChatGPT Handoff

Checked at: 2026-09-08 03:59 Asia/Shanghai
Repository main HEAD reviewed: `74abbd3e8d7b36869c4bbfd7184bd8314f18b979`
Previous reviewer handoff commit: `74abbd3e8d7b36869c4bbfd7184bd8314f18b979`
Current implementation branch: `work/e1a-staged-accounting-20260907` at exact `0b293f74c9a59a37272034268f00120198837dbe`
Previous checked implementation HEAD: `655df00b0b6439339ffccdf9a0962f5bb1c66b82`
Historical partial-E2 branch retained: `work/continue-20260904` at `d271a99a2ab26abbcb146c411ba0fde697395abe`

## What changed

The outward-runtime queue is moving again. Since the previous checked implementation HEAD, the work branch first reconciled reviewer-only main lineage (`7606812`, `74abbd3`) through merge `19e3b75`, then landed one substantive runtime commit:

- `0b293f7` — **real periodic authenticated-TCP synchronized key-update seam**. `periodic-client` / `periodic-server` now accept a bounded `--key-update-after N` experimental schedule, reject `N=0` and `N>=count`, keep the same authenticated Session, and call the existing `SecureSession::update_key_phase()` after the Nth successful request/DeliveryAck boundary. A real loopback/process TCP test crosses the boundary with three authenticated records, observes one client and one server key-update event, and confirms all 3/3 records.

Exact implementation-head CI is green:

- `work/e1a-staged-accounting-20260907` exact `0b293f74c9a59a37272034268f00120198837dbe` — Rust CI run `34156789753`, `success`.

Current `main` exact `74abbd3` is also green (Rust CI run `34150247966`, `success`). The work branch is ahead of `main`; this is expected implementation/reviewer coordination, not a defect.

This is the first new outward runtime capability after the pre-auth closure work. The design stays inside the existing architecture: there is no new wire control message, no new crypto primitive, no Session/Carrier/ACK semantic change, and no new policy number. The schedule is explicitly experimental/out-of-band; both controlled endpoints receive the same boundary. The server rekeys after writing the Nth authenticated DeliveryAck, and the client rekeys only after successfully opening and applying that ack, so the next client application record is phase 1. TCP ordering preserves the old-phase ack before later phase-1 traffic.

No VPS/WAN key-update evidence exists yet at this exact implementation head.

## Review verdict

**ACCEPT_LOCAL_RUNTIME_AND_MOVE_LIVE — accept `0b293f7` as a real local authenticated-socket key-update capability with exact-head green CI. Do not reopen pre-auth checker work. Add one small process-level desynchronization regression (no new framework), then immediately run one bounded self-owned VPS key-update observation under standing authorization and reconcile the release matrix.**

The remaining process-level negative test is an evidence-hardening task, not a new architecture blocker. C2 terminal-source retention remains an isolated release/security policy limitation and does not block runtime/VPS work.

No administrator action is required.

## Review findings and retained boundaries

### KEYUP-RUNTIME-001 — ACCEPTED — positive real-socket local key update

`0b293f7` moves key update beyond fixture-only crypto tests:

- one authenticated periodic TCP Session exists before and after the update;
- records/DeliveryAck complete on both sides of the boundary;
- both controlled peers update exactly at the configured successful exchange boundary in the normal path;
- the implementation reuses `SecureSession::update_key_phase()`; that primitive rekeys both directions, increments the authenticated record-context key phase, resets send nonce/replay state, rejects old-phase ciphertext after commit, and rejects a second update beyond current `MAX_KEY_PHASE=1`;
- no new unauthenticated control frame or dynamic negotiation is introduced.

This is local/process evidence only. It is not a production key-rotation protocol, not dynamic peer-initiated rekey negotiation, and not WAN evidence.

### KEYUP-RUNTIME-002 — MEDIUM evidence gap — schedule mismatch needs one process-level fail-closed regression

The crypto crate already proves an unsynchronized phase update rejects the peer's old-phase ciphertext without changing the peer state. The new periodic runtime, however, has only a positive same-boundary process test. Because the schedule is out-of-band configuration rather than negotiated protocol state, add exactly one bounded process test with intentionally different client/server boundaries and prove:

- the next cross-boundary authenticated record/ack fails closed;
- no later `periodic_summary` success is fabricated;
- no DeliveryAck/success-equivalent evidence is emitted for the rejected post-boundary record;
- process shutdown/cleanup remains bounded.

Do not add a new wire negotiation mechanism merely to satisfy this test. Do not build another generic checker. This is a narrow regression and should not delay the subsequent VPS run beyond the same coding invocation.

Existing crypto-level stale-phase and second-update tests remain sufficient for those primitive invariants; do not duplicate every crypto fixture at CLI level.

### KEYUP-RUNTIME-003 — diagnostic boundary

The current client/server key-update events are parseable and secret-safe enough for the bounded live question. If the existing evidence collector benefits from a summary field such as `key_updates=1` / final phase, adding it is acceptable, but **summary cosmetics are not a VPS blocker** if the collector can retain the explicit `periodic_*_key_update seq=... key_phase=1` events plus the normal application summary.

### RSEC-001E2-GUARD — CLOSED at `655df00`

Retain the accepted responder-ordering guard; do not reopen it without a concrete regression.

### RSEC-001E1A — CLOSED at `164731d` / current lineage

Retain staged one-logical-record TCP accounting across ordinary, periodic, multistream and failover responder handshakes.

### RSEC-001C1 — CLOSED at `f7e2cf1`

Carrier-aware pre-auth source projection remains accepted and bounded.

### RSEC-001C2 — POLICY LIMITATION at `f066af5`

`docs/adr/m1-g0-preauth-source-retention-amendment-request.md` remains unresolved. Do not invent TTL/LRU/history/epoch/eviction values. For current bounded research execution, preserve bounded in-process cleanup and explicitly keep literal D019 terminal-source no-reset as an unimplemented release/security limitation rather than inventing a retention mechanism. This disposition does not claim D019 compliance and does not block independent runtime/WAN work.

## Evidence boundaries

- `IMPLEMENTATION_COMPLETE=true` remains a bounded research-baseline flag only.
- `CANONICAL_CORPUS_V1_FROZEN=true` remains corpus-specific only.
- `RELEASE_CANDIDATE=false`, `PRODUCTION_READY=false`, `FREEZE=false`, `RELEASED=false` remain required.
- Exact `0b293f7` is implementation + local real-socket/process evidence with exact-head green CI; it is not WAN, reliability-rate, performance or production evidence.
- The fixed `--key-update-after N` schedule is an experiment harness contract, not a new negotiated wire protocol and not a production rekey policy.
- A positive self-owned VPS run can prove only that two controlled peers synchronously rekey an already authenticated bounded Session at the configured boundary while application/DeliveryAck traffic continues. It cannot prove arbitrary peer-initiated rekey, dynamic resynchronization, long-term rotation policy, public reachability, reliability rate or superiority.
- Historical WAN/HY2/failover/periodic positive and negative evidence remains immutable at exact commit boundaries.
- Standing VPS authorization explicitly covers bounded self-owned key-update experiments.
- Protected identity material, SSH private keys, credentials, private endpoint material and raw private diagnostics remain unread/untracked/uncommitted. Use local secret configuration only through the intended SSH/client mechanism; never print or copy private key contents.

## Rolling Work Queue

Coordination cadence remains reviewer `:00`, coding agent expected wake/resume `:20`; neither is a work-duration limit. If implementation/test/CI/experiment work is already running, do not interrupt it. Finish a coherent package -> gates -> commit -> push -> immediately consume the next dependency-satisfied package. One commit, one hour, one reviewer interval, no-diff audit result or ordinary CI pending state is not a stop condition.

### Q0 — Reconcile reviewer main lineage

**Status:** `READY_LOCAL`; coordination only.

Fetch current `origin/main` and normally merge this handoff into `work/e1a-staged-accounting-20260907`. Preserve `0b293f7` and all implementation history; no force-push. `docs/CHATGPT_HANDOFF.md` remains reviewer-owned/read-only to the coding agent.

If already reconciled, skip.

**Continue immediately to A:** yes.

### A — One bounded runtime desynchronization regression

**Status:** `READY_LOCAL`; narrow evidence-hardening, no architecture work.

**Goal / why now:** prove the out-of-band periodic key-update schedule fails closed when controlled peers are misconfigured, before treating the same-boundary path as READY_LIVE evidence.

**Files/concepts:** existing periodic process test helpers and `crates/neko-cli/tests/probe.rs`; only touch runtime code if the test exposes a real defect.

**Protected invariants:** no silent phase resynchronization; no success/DeliveryAck after rejected cross-boundary ciphertext; bounded timeout/shutdown; existing same-boundary success remains green.

**Behavior/test:** run server/client with different valid `--key-update-after` boundaries on a small count (for example count 3 with boundaries 1 vs 2). Assert non-success at the first incompatible post-boundary exchange and no successful final periodic summary. Reuse existing crypto stale-phase/second-update tests rather than cloning them.

**Gate:** targeted process tests + `scripts/check.sh` + `git diff --check`. No fuzz requirement unless untrusted parser/wire decode changes (it should not).

**Commit/push:** one coherent regression/fix checkpoint if a diff is needed. If an equivalent process regression already exists by execution time, skip.

**Continue immediately to B:** yes.

### B — Classify live key update READY_LIVE and run one bounded VPS observation

**Status:** `PREAUTHORIZED_AFTER_A_GREEN`; standing authorization covers it.

**Goal / why now:** obtain time-sensitive real-network evidence for a runtime capability that now exists locally, maximizing VPS evidence value per rental day.

Use the existing local secret endpoint/SSH configuration without reading or printing private key contents. Verify the owned target with non-secret UID/hostname checks before deployment. Do not modify production firewall/route/DNS/proxy/tunnel/qdisc or existing unrelated services.

Use the smallest periodic TCP profile that gives authenticated application traffic on both sides of exactly one update boundary. Suggested shape: 3-5 records, small payload, update after record 1 or 2, comfortably below standing limits. This is a correctness observation, not a benchmark.

Collect and retain:

- experiment ID;
- exact git commit and binary hash/size;
- exact count/bytes/duration/update boundary;
- authentication events on both endpoints;
- explicit client/server key-update event and phase;
- application records / DeliveryAck before and after update;
- attempted/confirmed/missing/duplicate counts;
- cheap CPU/RSS/FD/socket observations if existing sampler makes them available without complicating the run;
- explicit listener/process/temp cleanup verification.

If positive: record one bounded observation only, no reliability rate. If negative: preserve exact stage/evidence, clean up, and do not unchanged-retry. A retry requires changed code/instrumentation/config/hypothesis/path condition.

**Continue immediately to C after cleanup:** yes.

### C — Reconcile live-key-update evidence and authoritative release classifications

**Status:** `PREAUTHORIZED_AFTER_B`.

Update the exact authoritative files needed to stop stale classification:

- `IMPLEMENTATION_PLAN.md` release-evidence opportunity section: live key update is no longer `BLOCKED_IMPLEMENTATION`; classify according to the actual B result;
- `ROADMAP.md` real-environment notes if the result materially changes the bounded live key-update state;
- `docs/status.md` only if a capability/evidence row genuinely changes;
- add/retain a small evidence note/artifact with exact commit, parameters and cleanup boundary.

Do not rewrite historical artifacts. Do not promote release/production/freeze flags. State what the observation does **not** prove.

If the bounded live-key-update question is answered positively, mark that narrow question sufficient rather than scheduling freshness reruns.

**Continue immediately to D:** yes.

### D — Select next outward runtime seam from remaining `BLOCKED_IMPLEMENTATION`

**Status:** `READY_LOCAL_AFTER_C`; agent proposal authority applies.

Recompute exact-current candidates and compare 1-3 minimal shapes. Default preference is **migration-back/recovery over real sockets**, because manager-state logic already exists and real recovery evidence has high rental-window value. Other candidates may win if exact code makes them materially smaller/higher value:

- endpoint/path migration if the owned environment can create a genuine source/path change without production routing changes;
- live PMTUD integration into an authenticated real path;
- another release row newly made dependency-ready by recent runtime work.

For the chosen candidate, produce a short proposal (in implementation notes/commit context, not reviewer handoff) and immediately implement the smallest runtime seam inside existing Session/Carrier/ACK/crypto/wire architecture. If one candidate genuinely requires a core architecture or new policy decision, select another independent candidate instead of stopping the project.

**Continue immediately to E:** yes.

### E — Local real-socket proof for the next runtime seam

**Status:** `PREAUTHORIZED_AFTER_D`.

Add the smallest loopback/process or netns proof that exercises the chosen runtime behavior with actual sockets and verifies application/evidence semantics, failure boundaries and cleanup. Do not overbuild a generic harness.

Gate with targeted tests + full repository gate; fuzz only for changed untrusted parsing/wire decode. Commit/push and check exact-head CI.

**Continue immediately to F after exact-head green:** yes.

### F — One materially distinct bounded VPS experiment

**Status:** `PREAUTHORIZED_AFTER_E_GREEN` if the chosen question is truthfully READY_LIVE.

Execute one minimal self-owned VPS run under standing authorization. Preserve negative results, exact parameters and cleanup. Do not split a >10-minute scenario into nominally separate runs to evade limits. Do not mix performance comparison with unrelated CPU-heavy build/fuzz work.

**Continue immediately to G:** yes.

### G — Reconcile second runtime/VPS result

**Status:** `PREAUTHORIZED_AFTER_F`.

Update authoritative matrix/status/evidence only at the actual semantic boundary. If the run exposes a runtime defect, repair it before any changed-hypothesis rerun. If it answers the bounded question, classify it sufficient rather than rerunning for freshness.

**Continue immediately to H:** yes.

### H — Independent D019/security debt while C2 remains limited

**Status:** `READY_LOCAL_FALLBACK`; must not displace READY runtime/VPS work.

Only cover genuinely missing deterministic boundaries independent of terminal-source retention: concurrency/global windows, memory/queue/response/anti-amplification, idle/lifetime/100 ms response deadline, cancellation/double cleanup and no-success-evidence barriers. Reuse existing tests; add only distinct missing boundaries. Do not reopen staged responder/checker infrastructure.

Full literal D019 compliance remains unclaimable while C2 is unresolved.

### I — Exact-tree release/security navigation refresh after visible runtime progress

**Status:** `READY_AFTER_C_OR_G`.

After at least one new runtime/VPS bounded question is reconciled, update stale exact-head references and release/security navigation (`docs/reviews/resource-abuse-evidence-2026-09-04.md`, `docs/release-security-review-packet.md`, `docs/status.md`, ROADMAP/IMPLEMENTATION_PLAN as applicable). This is a consolidation pass, not another multi-hour audit lane.

Do not automatically promote RC/production/freeze/release.

## Completion gates

The current outward key-update closure is complete when:

- same-boundary periodic authenticated TCP key update remains locally green;
- one process-level mismatched-boundary configuration fails closed without post-rejection success evidence;
- exact implementation-head CI is green;
- one bounded self-owned VPS key-update observation is retained with exact commit/parameters and cleanup, positive or negative;
- authoritative release-opportunity classification no longer calls live key update `BLOCKED_IMPLEMENTATION` after the implementation exists;
- evidence language distinguishes fixed experimental schedule from dynamic/production rekey protocol;
- governance flags remain unchanged.

The broader queue remains active through D-I unless a real stop condition occurs.

## Do not expand into

- new wire rekey-control frames solely for the fixed-boundary experiment;
- production/public listener deployment;
- new source-retention TTL/LRU/history/epoch/eviction policy without reviewed policy authority;
- FEC/0-RTT/striping/heterogeneous multipath/exotic carriers without an observed-problem gate;
- unchanged repeats of historical failed WAN/HY2/repeated-failover lines;
- reading/printing/copying/committing protected identity or SSH private-key contents;
- production route/firewall/DNS/proxy/tunnel/qdisc changes;
- third-party targets or scanning;
- release/RC/freeze/production promotion.

## Questions requiring maintainer decision

None for the current runtime/VPS queue.

C2 terminal-source retention remains a future release/security policy choice. Until a new policy is approved, the conservative bounded-cleanup + explicit non-compliance limitation is the active disposition; it does not block the independent outward runtime queue.