# Nekomusume ChatGPT Handoff

Checked at: 2026-09-08 12:57 Asia/Shanghai
Repository main HEAD reviewed before this handoff: `643216794ffb53577213a6d4ba05457ea3f84768`
Previous reviewer handoff commit: `643216794ffb53577213a6d4ba05457ea3f84768`
Previous checked implementation HEAD: `bd43f5b466e5f4eb39ec516582dd1a8f49a89f61`
Current execution branch: `work/e1a-staged-accounting-20260907` at exact `f024458926ec66d1d07ccf9b07534ae3d71bdf60`
Current runtime implementation under review: exact `5d6582c0a674306cf47002d4cd26d6e0c6065c22`
Exact implementation/evidence-head Rust CI: run `34188050956` — `success`
Historical partial-E2 branch retained: `work/continue-20260904` at `d271a99a2ab26abbcb146c411ba0fde697395abe`

## What changed

The coding agent consumed the previous migration-back review and closed the live-evidence gaps in one coherent runtime/test repair, then ran exactly one bounded self-owned VPS observation and retained a narrow evidence note. This is healthy `Agent implement -> reviewer challenge -> Agent repair -> live evidence` iteration and is a genuine outward result.

Accepted exact `5d6582c` repairs:

- `--migration-back` is now a distinct terminal experiment mode rather than being silently aliased to the legacy restore seam;
- the client uses exact-peer `recv_from` for recovery and rejects a response from any source other than the configured peer;
- recovery is a one-shot fresh authenticated challenge for this bounded experiment, so a successful observation can truthfully carry zero observed retry/loss/PTO;
- active TCP health is measured from a real resumed authenticated Data -> exact `DeliveryAck` round trip rather than connect/Noise setup time;
- one measured UDP recovery `HealthSample` is reused for both manager observation and the migration candidate;
- the server requires exact Session/path/generation/epoch/challenge semantics and a complete post-return UDP Data/DeliveryAck milestone before a requested migration-back run can report success;
- the final post-return application record remains unassigned until the manager has actually authorized UDP return;
- final accounting includes that UDP-confirmed record correctly;
- the migration-back tamper negative now launches a real server, proves TCP fallback/resumed DeliveryAck first, reaches recovery, then fails closed with no migration/post-return success;
- exact `5d6582c` passed the full repository gate and exact execution/evidence head `f024458` has green Rust CI.

The retained bounded VPS observation at `docs/notes/migration-back-vps-5d6582c-20260908.md` is internally consistent with those semantics: one IPv4 Session, three 16-byte records, scripted application-level UDP reply cessation, authenticated warm TCP fallback, measured TCP health, one exact-peer generation-1 UDP recovery challenge, hold-gated return, and only then one new UDP application record with authenticated `DeliveryAck`. Client and server exited successfully and cleanup found no retained experimental listener/process.

## Review verdict

**MIGRATION_BACK_BOUNDED_LIVE_ACCEPTED — accept exact `5d6582c` runtime plus exact `f024458` evidence/status reconciliation for the bounded question. No HIGH/BLOCKER remains in this closure. Integrate this accepted lineage to `main`, then immediately open the next missing runtime capability: authenticated UDP source-endpoint rebinding with fresh path-generation validation.**

This acceptance is deliberately narrow. The live observation used a scripted application-reply cessation and a bounded test-only 10 ms TCP `DeliveryAck` delay to cross the already-existing score margin. It therefore does **not** establish natural path recovery, a reliability rate, real-path comparative performance, public reachability, or production readiness. Those exclusions are already stated in the evidence note and must remain.

No administrator action is required for the migration-back closure or the next local endpoint-rebinding implementation. Standing authorization covers a later bounded self-owned TCP/UDP endpoint-migration observation if the local/runtime gates are green.

## Reviewer findings / navigation decisions

### MIGBACK-015 — ACCEPTED — the prior four live-evidence defects are closed

The reviewed `5d6582c` code now binds recovery to the exact peer and exact authenticated tuple, uses measured comparable health inputs, makes requested migration-back terminal rather than fall-through, and contains a real process-level tamper negative. Do not reopen MIGBACK-010 through MIGBACK-014 without a new concrete regression.

The 10 ms test-only TCP DeliveryAck delay is acceptable **only as part of this explicitly scripted bounded experiment**. It is not network latency evidence and must not be used in a performance claim. The retained note states this correctly.

### NAV-EP-001 — READY_LOCAL — endpoint/source change is now the best outward runtime target

Current planning classifies NAT/source-endpoint change as `BLOCKED_IMPLEMENTATION`: there is no authenticated live rebinding runner. This is a better next target than live PLPMTUD because the carrier architecture already says a Session is not bound to a five-tuple and a Path is a concrete carrier/peer instance, while D064 already includes `address_change` as a carrier-transition reason.

The owned environment can produce a genuine source-endpoint change without modifying production routes, firewall, DNS, proxy, tunnel or qdisc: the client can replace/rebind its UDP socket to a new ephemeral local endpoint while keeping the same authenticated logical Session. This directly unlocks a high-value rental-window VPS question.

**Design authority:** the coding agent may propose/choose the smallest local API/state-machine shape and implement it without waiting. No new wire kind is required for the preferred shape below.

Preferred semantic shape:

1. Start an authenticated UDP Session on source endpoint A, path 1 / generation N, and confirm at least one small application record.
2. Create a new client UDP socket bound to a different ephemeral endpoint B. Do not mutate host routing/firewall/qdisc.
3. From B, send one bounded authenticated unreliable readiness/control record identifying the same Session and a fresh candidate path generation. This is only a candidate signal; it is **not** by itself path validation and must not move application ownership.
4. On receiving an authenticated candidate from a different source, the server keeps A as the current owner and sends its **own fresh authenticated `ReadinessRequest` challenge** to B. Reuse the existing authenticated readiness control envelope; do not add a new wire type solely for this lab seam.
5. B must receive that exact-peer challenge and return the exact authenticated `ReadinessResponse` from B. The server promotes the candidate only after receiving the response from exact B with exact Session/path/generation/epoch/challenge binding.
6. Promotion is atomic: the new endpoint becomes the current UDP path generation; the old endpoint becomes stale/draining for new application ownership. No new application Data is sent to B before validation completes.
7. Send one previously-unassigned application record after promotion and require the exact authenticated Session `DeliveryAck` through B.
8. A later authenticated/replayed/stale-generation record arriving from old endpoint A must not revive the old generation or create `PathValidated`, Session delivery, readiness, or ownership evidence.

This two-way server-generated challenge is important. Merely authenticating a client-originated record from B proves that a holder of Session keys can send from B; it does not prove the server's challenge reached B. Do not silently treat “authenticated packet from a new source” as complete path validation.

**Security/resource invariants:**

- exact source endpoint is part of the candidate-path binding;
- candidate input cannot mutate current application ownership before fresh challenge/response validation;
- no unauthenticated control changes endpoint/path state;
- pre-validation response bytes remain inside existing bounded validation/amplification limits;
- one outstanding candidate/challenge is enough for the bounded seam; do not create unbounded source/challenge history;
- stale generation, wrong Session, wrong delivery epoch, wrong challenge, tamper, replay and old endpoint after promotion fail closed;
- Session delivery, path validation, packet feedback and health remain separate evidence domains;
- no new numeric security policy, TTL/LRU/history limit, wire frame type, crypto primitive or Session ACK semantics.

If the smallest implementation truly requires a new wire type or a new numeric security policy, stop **this lane only**, record the exact reason, and continue to the independent repeated-warm-failover diagnostics fallback. Do not invent policy values.

### NAV-PMTU-001 — NOT READY FOR AUTONOMOUS LIVE IMPLEMENTATION

Do **not** select live PLPMTUD ahead of endpoint rebinding merely because a socket-free `neko-reliable::Plpmtud` model exists. The current PMTUD ADR explicitly says the next implementation gate still must define exact authenticated probe/ACK wire fields, IPv4/IPv6/header accounting, PTB validation, timer/cooldown constants, record-fragmentation semantics and event schemas; it also labels several resource/probe numbers as implementation-gate candidates rather than frozen policy.

Therefore direct live PLPMTUD integration today would cross the project's wire/numeric-policy escalation boundary. The agent may inspect/research it, but must not invent those decisions while a cleaner dependency-ready endpoint-migration lane exists.

### EVID-001 — migration-back claim boundary remains narrow

Accepted facts are: one scripted bounded self-owned IPv4 fallback/recovery/return observation at exact `5d6582c`, exact binary identity recorded, actual bounded parameters recorded, measured TCP and UDP decision inputs recorded, 3/3 logical records / 48 application bytes confirmed, one uncertain/replayed middle record, no duplicate/lost/conflicting record, and cleanup verified.

Not accepted: natural UDP recovery, NAT rebinding, reliability percentage, performance superiority, public reachability, production readiness, protocol freeze or release.

## Accepted repository boundaries

- `RELEASE_CANDIDATE=false`, `PRODUCTION_READY=false`, `FREEZE=false`, `RELEASED=false` remain unchanged.
- `IMPLEMENTATION_COMPLETE=true` remains a bounded research-implementation flag only.
- C2 source-retention remains an explicit release/security policy limitation. Do not invent TTL/LRU/history/epoch values while unrelated post-auth runtime work can continue.
- Existing migration-back evidence is retained at its exact commit boundary. Do not rerun it unchanged.
- Exact `f024458` is green and carries accepted migration-back evidence/status reconciliation; it is not yet merged into reviewer `main` at the time of this handoff.
- IPv6 remains environment-blocked.
- Historical repeated-failover / HY2 negatives remain immutable at their exact commits; no unchanged retry.
- Protected identities, SSH keys, credentials, private endpoints and raw private diagnostics remain unread/untracked/uncommitted.

## Rolling Work Queue

This is a continuous pre-authorized queue. Finish coherent capability/evidence closure -> focused/full gates -> commit -> push -> continue immediately. Reviewer cadence is not a work-ticket length.

### A — Integrate accepted migration-back lineage into `main`

**Status:** `READY_GIT`; immediate coordination cleanup, not a new design gate.

Fetch current reviewer `main`, merge/reconcile it into the execution branch while preserving this reviewer-owned `docs/CHATGPT_HANDOFF.md`, then integrate the accepted `5d6582c` runtime + `f024458` evidence/status lineage to `main` without force-push or history rewriting. Resolve any handoff conflict in favor of the current reviewer file.

**Gate:** resulting integrated exact head must have `scripts/check.sh`, `git diff --check`, and exact-head CI green. If CI is merely pending, independent endpoint-rebinding local design/code may continue on the work branch; do not idle.

**Continue immediately to B:** yes.

### B — Authenticated UDP endpoint-rebinding runtime seam

**Status:** `READY_LOCAL`; proposal authority applies.

**Goal / why now:** turn the remaining NAT/source-endpoint-change matrix row from `BLOCKED_IMPLEMENTATION` into a real authenticated runtime path that can be tested with a genuine source-port change and later observed on the self-owned VPS.

**Primary files/concepts:** `crates/neko-cli/src/main.rs`, existing failover/readiness control helpers, `CarrierManager` / path generation state, process tests. Reuse `ProcessMessage::ReadinessRequest/ReadinessResponse` and existing authenticated unreliable control if structurally suitable. Do not redesign post-auth Data/DeliveryAck.

**Behavior:** implement NAV-EP-001. Prefer a bounded opt-in experimental CLI seam such as endpoint-rebind/source-change rather than silently changing every UDP server path in this first closure.

**Protected invariants:** same logical Session; new exact source is candidate-only until server-generated challenge round trip; generation-scoped validation; single-active application owner; no application Data on candidate before promotion; old source cannot revive after promotion; secret-safe events.

**Minimal structured events:** semantic equivalents of `endpoint_candidate_seen`, `endpoint_challenge_sent`, `endpoint_validated`, `endpoint_promoted`, `endpoint_rebind_failed`, plus generation/path/session-safe identifiers but no raw address in tracked evidence.

**Commit/push:** coherent runtime checkpoint required.

**Continue immediately to C:** yes.

### C — Endpoint-rebinding deterministic + real-loopback negative matrix

**Status:** `PREAUTHORIZED_AFTER_B`.

Positive minimum:

- authenticated application record succeeds on endpoint A;
- client rebinds to a genuinely different local UDP source port B;
- candidate control from B alone does not promote;
- server-generated fresh challenge reaches B;
- exact authenticated response from B validates/promotes fresh generation;
- one previously-unassigned post-promotion Data record via B receives exact Session DeliveryAck;
- accounting/ownership stays single-active.

Negative minimum: use one meaningful runtime negative rather than duplicating all manager unit tests. Preferred: reach authenticated candidate/challenge on B, then tamper or return wrong challenge/generation/source; assert no promotion, no post-promotion DeliveryAck, no success summary and old active ownership remains valid. Also assert one stale old-A datagram after successful promotion cannot create new delivery/path-validation evidence.

Run focused process tests + relevant carrier tests + `./scripts/check.sh` + `git diff --check`. Run fuzz smoke only if parser/wire decode changes; this design should not require such a change.

**Exact-head CI must be green before D.**

**Continue immediately to D:** yes.

### D — One bounded self-owned VPS endpoint/source-change observation

**Status:** `PREAUTHORIZED_AFTER_C_GREEN`; standing authorization already covers ordinary bounded TCP/UDP migration/recovery work.

Use the smallest genuine source-endpoint change the owned client/VPS setup can produce without production network mutation: client UDP socket A -> new ephemeral UDP socket B on the same controlled client, with the server observing a different source endpoint. It is acceptable if this demonstrates source-port/endpoint rebinding rather than a full public-IP change; describe exactly what changed.

Minimum workload: one pre-rebind confirmed record, one candidate/challenge/response path-validation sequence, one post-promotion confirmed record. Keep one Session, tiny payloads, short command bound. No production route/firewall/DNS/proxy/tunnel/qdisc changes.

Retain exact commit/binary, actual parameters/times, endpoint-change class (`source_port_changed` or stronger if genuinely observed), old/new endpoint inequality as a boolean/hash-safe relation rather than raw private address material, path/generation transitions, challenge outcome, post-promotion DeliveryAck, client/server exit, cleanup and cheap resource observations when already available.

**Claim boundary:** one bounded authenticated self-owned endpoint-rebinding observation only; not general NAT traversal, roaming reliability, public reachability or production readiness.

Preserve the first meaningful negative; no unchanged retry.

**Continue immediately to E:** yes.

### E — Reconcile release matrix/status and integrate endpoint lineage

**Status:** `PREAUTHORIZED_AFTER_D`.

Update only facts actually established by D in `ROADMAP.md`, `IMPLEMENTATION_PLAN.md`, `docs/status.md`, and a compact evidence note/artifact. If D proves only source-port rebinding, write exactly that; do not relabel it as arbitrary NAT/public-IP migration.

Then integrate exact-green implementation/evidence lineage into `main`, preserving reviewer handoff ownership and history.

**Continue immediately to F:** yes.

### F — Repeated warm-failover diagnostic repair while next architecture gate is reviewed

**Status:** `READY_LOCAL_FALLBACK_AFTER_E`; bounded harness work is allowed here because migration-back and endpoint-rebinding have produced outward runtime results first.

Do not rerun exact `9fd2411` / `a117086`. Add bounded sanitized inner-collector failure categorization so a nonzero collector result can be distinguished at least among startup/setup, negotiation/auth, readiness, application/runtime and evidence-serialization/cleanup categories without retaining secrets. Propagate that category to the existing outer typed result.

Synthetic/local verification first. One later live retry is justified only if instrumentation materially changed and exact-head CI is green.

**Continue immediately to G:** yes.

### G — One materially changed repeated-failover live attempt if F resolves the diagnostic blind spot

**Status:** `PREAUTHORIZED_AFTER_F_GREEN`.

Exactly one changed-hypothesis self-owned bounded attempt; retain prefix/negative evidence, exact binary/parameters, failure category and cleanup. Do not require six successful cycles to preserve a useful prefix. Do not mechanically repeat if the same classified failure recurs without a new hypothesis.

### H — Next protocol-gate selection, not speculative implementation

**Status:** `REVIEWER_CHECKPOINT_AFTER_E/G`.

At this point re-evaluate the remaining matrix. Live PLPMTUD still requires its explicit wire/policy implementation gate unless a newer ADR has resolved it. HY2 remains a comparison lane, not the sole release gate. Prefer a real release-matrix blocker over FEC/0-RTT/striping/multipath/exotic-carrier work.

If all remaining runtime-capability rows genuinely require new wire or numeric security policy, that is a real maintainer/reviewer architecture checkpoint and may be escalated then; do not fabricate policy merely to keep commits moving.
