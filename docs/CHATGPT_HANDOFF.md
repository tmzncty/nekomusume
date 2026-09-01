# Nekomusume ChatGPT Handoff

Checked at: 2026-09-01 08:01 Asia/Shanghai
Repository HEAD: `ca2481017ed704c81ebfb97739cb6db2370ca510`
Previous reviewed implementation HEAD: `8726623cf375ab3ef478e6af4993e20bff2383e9`
Previous reviewer handoff commit: `85e0569dd76cdd50f6d2f701a010eaaa29407ab7`

## What changed

Two substantive coding-agent commits landed after the previous review:

- `8973ce5` — **N9 governance/spec/test transition; no production transport change.** It mechanically freezes the reviewed 42-vector / 10-domain canonical corpus, recomputes its content identity, updates the validator/schema/generator and governance checks, marks N9 complete, and records `CANONICAL_CORPUS_V1_FROZEN=true` while correctly keeping `FREEZE=false`, `RELEASE_CANDIDATE=false`, `PRODUCTION_READY=false`, and `RELEASED=false`. The frozen scope still excludes Noise/ciphertext, carrier packetization, failover/resume and the whole protocol.
- `ca248101` — **real generic probe implementation + tests.** Ordinary TCP and UDP `neko server` / `neko client` paths now perform canonical `VersionNegotiator` exchange before Noise, authenticate the exact negotiation binding in the Noise prologue, and gate application echo on `admit_data()`. New process tests cover malformed/unsupported negotiation, duplicate hello, unsupported selection and transcript mismatch before application data. GitHub Actions for this HEAD completed successfully.

The reviewer re-read the current failover/resume runner because it is the next explicit implementation-plan dependency and because the rented VPS makes real-socket evidence time-sensitive. The failover path is **not** yet equivalent to the newly repaired generic probe path:

- `failover_server` / `failover_client` still start Noise directly; no canonical version negotiation is performed on the initial UDP path or the TCP resume path.
- `InitiatorHandshake::with_resume_binding` currently has no way to compose the negotiation prologue binding with the existing resume claim, although the responder can already be created with a generic prologue binding and then call `receive_first_with_resume`.
- The same-process failover runner can retain the initial selected version and require the TCP resume negotiation to select the same version; this does not require adding a fake second production version or changing the frozen N1 bytes.

A second, independent **evidence-integrity finding** is now concrete and must be repaired before new failover VPS evidence is treated as release evidence:

1. failover diagnostic `start` events hard-code `count=8`, `payload_bytes=64`, `udp_port=40081`, and `max_seconds=15` instead of reporting the actual CLI parameters;
2. `failover_server` initializes `duplicates = 0` and never increments it, yet reports `duplicates=0` as if measured;
3. the client reports `udp_blackhole=true` and emits `udp_blackhole_injected`, but the current CLI path does not observe a real UDP health failure or PTO threshold: after the first UDP record/ACK it simply moves to TCP. The historical VPS run is therefore valid evidence of a **controlled client-induced UDP-stop -> TCP resume transition over real sockets**, but it must not be promoted to evidence that the runtime detected actual WAN degradation/blackhole.

These are evidence/runner defects, not proof of a Session deduplication bug. Existing deterministic `FailoverController`/`SessionRuntime` tests still exercise PTO threshold, uncertain resend and logical dedup separately. The next slice should integrate those domains truthfully rather than preserving a constant/misleading diagnostic claim.

## Review verdict

**PASS for N9 freeze and generic TCP/UDP probe negotiation. NEEDS REPAIR for failover/resume negotiation and failover evidence instrumentation before release-WAN evidence collection.**

Do not reopen the canonical corpus and do not redo generic probe negotiation. The next Primary is the failover/resume closure: authenticate negotiation on both carrier handshakes, bind resume to the same selected Session version, and repair the runner so its diagnostics describe what actually happened.

This project is not globally blocked. After the failover runner is green, use the rented VPS immediately for new post-negotiation real-socket evidence under standing authorization. Do not spend the rental window on unrelated local polish.

## Evidence boundaries

- Canonical corpus v1 is frozen exactly at the reviewed 42-vector / 10-domain corpus; global protocol/release `FREEZE` remains false.
- Generic TCP/UDP probe negotiation at `ca248101` is implementation + process-test evidence with green GitHub CI; it is not failover/resume evidence and not a security audit.
- `docs/status.md` correctly says the generic probe is negotiated/authenticated but failover/resume negotiation is still missing.
- `IMPLEMENTATION_PLAN.md` correctly leaves “Negotiation path completion” unchecked because failover/resume remains.
- The existing `docs/vps-failover-success-2026-08-30.md` remains useful historical evidence for its exact old candidate, but its `udp_blackhole_injected` transition is application-controlled. It does **not** satisfy the release-matrix row “observed UDP degradation -> automatic TCP fallback”.
- Current failover Session/Resume semantics remain bounded: same logical Session, fresh Noise transport, ResumeGuard, uncertain resend/dedup and Session-level delivery are separate from packet/path feedback.
- Standing self-owned VPS authorization is active: ordinary bounded TCP/UDP listener, failover/migration, bounded capture, soak, cleanup and comparison work within its limits need no new per-run permission.
- The rented VPS is a time-limited asset. Once this Primary is green, post-negotiation VPS evidence outranks documentation polish and speculative feature work.

## Work Package — truthful negotiated failover -> immediate VPS evidence

This package is intentionally thick. Execute A -> B -> C -> D in dependency order. Do not stop after adding one constructor or one test. If a real correctness/security defect appears, use the fallback.

### Primary A — Close authenticated negotiation + evidence integrity for failover/resume

**Goal**

Make the executable failover runner truthfully prove this sequence:

```text
initial UDP path negotiates version
-> exact UDP negotiation transcript is Noise-authenticated
-> UDP application data is admitted
-> a bounded, explicitly classified failure/stop condition is observed
-> TCP resume path negotiates version
-> selected resume version equals the Session's initial negotiated version
-> exact TCP negotiation transcript + existing ResumeBinding are authenticated by fresh Noise
-> ResumeGuard accepts the carrier attachment
-> uncertain Session ranges are resent/deduplicated
-> application data remains exactly-once where that existing Session claim is made
```

Do not change the frozen canonical N1 bytes.

**Likely files**

- `crates/neko-crypto/src/lib.rs`;
- `crates/neko-cli/src/main.rs`;
- `crates/neko-cli/tests/probe.rs` or a dedicated failover process test file if that keeps scope clearer;
- `crates/neko-carrier/tests/resumed_session.rs` and existing failover tests as required;
- `docs/m3-wan-failover-gate.md` / `docs/spec/m3-tcp-failover.md` for evidence-boundary corrections;
- `IMPLEMENTATION_PLAN.md` and `docs/status.md` only when the implementation/gate genuinely changes.

**A1 — Compose negotiation binding with resume Noise**

1. Add the smallest non-breaking crypto API needed for an initiator to combine the existing `ResumeBinding` payload with a non-empty generic prologue binding. A `with_resume_binding_and_prologue_binding(...)`-style constructor is acceptable.
2. Preserve legacy empty-binding behavior and existing `with_resume_binding` callers. Do not change old Noise bytes accidentally.
3. The responder should use the existing generic prologue-binding constructor with `receive_first_with_resume` unless a smaller refactor is clearly safer; do not introduce wire-policy dependency from `neko-crypto` to `neko-wire`.
4. Unit-test equal binding success, one-bit negotiation-binding mismatch rejection before `SecureSession`, existing resume-claim validation, and oversized binding fail-closed behavior.

**A2 — Negotiate both failover carrier handshakes**

1. Initial UDP failover handshake must use the same canonical N1 hello/response and authenticated binding as the generic UDP probe before Noise application admission.
2. Store the selected version as part of the **runner's current logical Session state** for this same-process candidate.
3. TCP resume must perform a fresh canonical N1 exchange before fresh Noise.
4. Before resumed application data, require the TCP selected version to equal the initial Session selected version. With only one current production version, test this guard with a small state/helper unit test or raw-peer failure cases; do not add a fake second supported production version merely for coverage.
5. Bind the exact TCP negotiation transcript into the same fresh Noise handshake that carries/authenticates the existing ResumeBinding. The resume claim and negotiation must not be two unrelated unauthenticated facts.
6. Only after negotiation, Noise authentication, same-version guard, and ResumeGuard may resumed `ProcessMessage::Data` reach `SessionRuntime`.
7. Preserve Session-level confirmed/uncertain/dedup semantics; negotiation success is not delivery proof.
8. Preserve pre-auth/resource limits and UDP amplification bounds. Duplicate/late negotiation attempts must remain bounded/fail-closed.

**A3 — Repair failover evidence instrumentation before any new VPS claim**

1. Replace hard-coded diagnostic start fields with the actual parsed count, payload size, UDP/TCP ports and duration. Add regression tests using non-default legal parameters so hard-coded defaults cannot return unnoticed.
2. Stop reporting `duplicates=0` unless it is genuinely measured. Either instrument the logical duplicate outcome from the Session/failover path, or remove/rename that field to an honest observable. A constant zero is not evidence.
3. Stop using `udp_blackhole=true` / `udp_blackhole_injected` as if it proves detected network failure when the client merely elects to stop using UDP. Preserve historical evidence unchanged, but make new runner terminology explicit, e.g. `controlled_udp_stop` / `application_fault_injection`, unless the runtime actually observed bounded missing ACK/PTO evidence.
4. Prefer upgrading the runner to use existing `FailoverController` semantics: send/track bounded UDP work, observe missing delivery/ACK timeouts, require the configured consecutive PTO/failure threshold, then switch. For a deterministic real-socket test without host qdisc/firewall changes, a lab-only bounded server behavior that intentionally withholds UDP ACKs after N records is acceptable if clearly named as **controlled endpoint fault injection** and impossible to confuse with natural WAN loss.
5. If such a fault-injection seam is added, keep it explicit, bounded, off by default, available only to the experimental failover command, and covered by process tests. It must not become a production/network mutation mechanism.
6. Emit enough structured evidence to distinguish:
   - negotiated/authenticated UDP path;
   - last successful UDP logical acknowledgement;
   - controlled drop/timeout observations;
   - threshold crossing / failover reason;
   - negotiated/authenticated TCP resume;
   - uncertain/replayed/duplicate/confirmed/lost bytes or records where the implementation can actually measure them;
   - cleanup result.

**Required tests**

At minimum cover:

- negotiated current/current UDP -> TCP resume success;
- generic current Session version retained across the TCP resume guard;
- unsupported/malformed resume-path negotiation rejection before Noise/data;
- TCP negotiation transcript mismatch causes Noise failure before resume admission;
- resume claim mismatch and negotiation mismatch independently fail closed;
- duplicate/late negotiation is bounded and cannot reset an established Session profile;
- controlled UDP ACK withholding triggers the actual bounded failure threshold if A3.4 is implemented;
- one missing ACK/PTO alone does not equal hard failure when current controller contract requires more;
- uncertain resend/dedup still produces exactly-once logical application bytes in the existing bounded claim;
- non-default legal count/bytes/ports/duration appear correctly in diagnostics;
- no constant duplicate/blackhole success field survives without an actual measurement.

**Validation**

Run targeted crypto/CLI/carrier/session tests, then:

```text
cargo fmt --all -- --check
cargo check --workspace --locked
cargo test --workspace --all-targets --locked --no-fail-fast
cargo clippy --workspace --all-targets --locked -- -D warnings
bash scripts/check.sh
git diff --check
```

Run fuzz smoke if public parser/wire decode behavior changes. A pure composition/runner change does not need a fabricated fuzz claim unless the repository gate requires it.

**Completion definition**

Failover/resume no longer bypasses canonical negotiation; same-Session resume cannot silently change negotiated version; exact negotiation is authenticated with the same fresh Noise transport that authenticates the resume; diagnostics are parameter-true and measurement-true; all local gates pass. Only then mark negotiation-path completion checked.

### Follow-up B — Immediate rented-VPS post-negotiation lab batch

**Dependency:** A green, pushed, binary identity recorded. No new maintainer approval is required for this bounded self-owned work.

Use one cleanup-safe bounded lab session where compatible measurements do not confound each other. Keep total activity inside standing authorization and record separate experiment IDs/scenario labels.

**B1 — Negotiated generic real-socket sanity**

- TCP current/current negotiated/authenticated echo on self-owned client <-> VPS;
- UDP current/current negotiated/authenticated echo;
- use actual non-secret parameters, binary/git identity, timestamps, result and cleanup metadata;
- if practical, one bounded unsupported/malformed negotiation negative row on the self-owned endpoint, proving no application echo.

These rows verify the new `ca248101` behavior on real sockets; they do not need to wait for natural network degradation.

**B2 — Negotiated failover/resume over real sockets**

Run the newly repaired failover path with the controlled endpoint fault seam if A implements one. The classification must be explicit:

```text
real TCP/UDP sockets + controlled endpoint UDP ACK/drop fault
!= naturally observed WAN degradation
```

Require event ordering/evidence for negotiation -> auth -> UDP data/ACK -> bounded missing-ACK/PTO threshold -> TCP negotiation -> fresh authenticated resume -> replay/dedup -> completion.

If A does not produce a truthful automatic threshold-driven runner, do **not** rerun the old unconditional switch and call it release failover evidence. Run only B1 and mark B2 blocked by `evidence instrumentation gap`.

**B3 — Repeated lifecycle sample**

If B1/B2 are green and time remains, perform a bounded repeated real-socket open/exchange/close sample (TCP and UDP separately or the negotiated failover scenario) with actual success/failure counts and cleanup. This is resilience evidence, not capacity/stress evidence.

Collect process-scoped CPU/RSS/FD/socket observations if current tooling can do so without changing semantics. Do not mix CPU-heavy build/fuzz with performance/resource sampling.

### Follow-up C — Promote only the evidence actually earned

**Dependency:** B complete or explicitly partially blocked.

Update the relevant evidence/status docs so they distinguish at least:

- generic negotiated TCP/UDP real-socket PASS;
- controlled endpoint-fault negotiated failover PASS, if achieved;
- natural/ambient WAN degradation detection: still unproven unless genuinely observed;
- long-lived/NAT/endpoint-change/IPv6 rows: unchanged unless this batch actually measured them;
- cleanup/process/listener state;
- exact candidate commit/binary identity.

Do not rewrite historical artifacts. Add new evidence with supersession/relationship notes where needed.

If the negotiated failover path and full local/CI gates are green, mark `IMPLEMENTATION_PLAN.md` negotiation-path completion done. The next release-evidence matrix remains separate.

### Follow-up D — Rental-window unlock: next VPS-only evidence seam

**Dependency:** A complete; do after B/C or use as fallback if B2 is instrumentation-blocked.

Choose the highest-value READY item that unlocks the next rental-window experiment, in this order:

1. **resource sampler** — reusable process-scoped CPU/RSS/FD/socket sampling with timestamps and experiment identity for Nekomusume/HY2;
2. **equivalent Nekomusume comparison command** — satisfy the existing HY2 workload contract with exact payload file/hash/application byte semantics and JSON `application_bytes`/`fd_count`;
3. **bounded resilience runner** — 5–10 minute steady or idle-with-periodic authenticated session with distinct scenario semantics and no claim of restart persistence;
4. **owned endpoint-change seam** if the existing environment can create a genuine source endpoint change without production route/firewall/qdisc mutation.

Do not start speculative FEC/0-RTT/exotic-carrier work while these rental-window evidence gaps are available.

## VPS opportunity

**READY after Primary A:** negotiated generic TCP/UDP real-socket sanity is already standing-authorized and should be run immediately. Negotiated failover becomes READY only when its runner truthfully detects/classifies the trigger and the resume path is version-bound.

The existing VPS is therefore not a blocker; it is the next evidence target. Missing per-run WAN permission, count/bytes/duration/ports are not valid blockers.

## Completion gates

This batch is complete when dependency-satisfied work has moved as far as possible without inventing claims:

- N9 stays frozen and untouched;
- generic negotiated probe implementation remains green;
- failover/resume performs and authenticates canonical negotiation on both carrier handshakes;
- same logical Session cannot resume under a different selected version in the current runner;
- diagnostic metadata reflects actual parameters;
- duplicate/failure claims are measured or removed, never constants presented as evidence;
- controlled fault injection is clearly separated from natural WAN degradation;
- all changed code passes required local gates and pushed CI is allowed to attest it;
- at least the READY post-negotiation TCP/UDP VPS sanity rows are collected unless a concrete environment/implementation failure prevents them;
- negative results and cleanup state are retained;
- RC/security/production/global freeze flags remain unchanged.

## Fallback

If the combined resume + negotiation binding exposes a real crypto/session incompatibility, preserve a minimal reproducer, keep release state false, and make that correctness issue the only blocker before further failover evidence.

If the current runner cannot observe a real bounded failure threshold without changing production host networking, do not request firewall/qdisc permission and do not fake the result. Add/repair a self-contained endpoint fault-injection seam or classify the release row as `evidence instrumentation gap`; continue READY generic TCP/UDP VPS evidence and resource/comparison instrumentation.

If IPv6/NAT/endpoint-change environment is absent, record that exact environment blocker and continue IPv4/failover/resilience/HY2/resource work. Do not stop the project.

## Do not expand into

- changing frozen canonical N1 corpus bytes;
- RC declaration, security approval, production readiness, release tagging or global protocol freeze;
- treating a controlled endpoint drop as proof of ambient Internet blackhole/degradation behavior;
- previous/current interoperability before a real prior frozen release exists;
- UDP+TCP striping/aggregation;
- third-party targets, scanning, production network mutation or experiments outside standing authorization;
- HY2 superiority claims from one-off or semantically unequal runs;
- speculative experimental carriers while rented-VPS release evidence is READY.

## Questions requiring maintainer decision

none.
