# Nekomusume ChatGPT Handoff

Checked at: 2026-08-31 20:13 Asia/Shanghai
Repository HEAD reviewed: `23008e904fd7a5ab258349e4679e973c5eb41555`
Previous checked HEAD: `e794f3437acf9300ccb2e128c3656460b82f1a53`

## What changed

The repository-reconciliation gate is now closed. GitHub `main` is a normal merge whose parents preserve both the advanced implementation chain and the reviewer/governance chain:

- `23008e9` — merge authoritative implementation with GitHub governance;
  - implementation parent: `c44340a`;
  - reviewer/governance parent: `735c11d`.

Relative to the previous formal reviewer baseline `e794f34`, the reconciled branch exposes 188 additional commits. The newly reviewable history includes substantial implementation, test, experiment and release-engineering work rather than documentation-only claims. Important recent slices include:

- `91a735c` / `5ee081a` — reproducible Linux package and native x86_64 build evidence;
- `d04fdde` — separate implementation-complete, RC, production, freeze and release governance states;
- `9ee91e8` — explicit CLI lifecycle state plus bounded SIGTERM/SIGINT handling;
- `59ecaca` — install/upgrade/rollback package lifecycle evidence;
- `84cc86f` — architecture support audit;
- `5e2ac79` — bounded explicit version negotiation primitive;
- `743e433` — candidate canonical-vector schema/corpus validator;
- `067bc55` — current/current and future/unsupported compatibility harness;
- `12bb098` — N8 bounded self-owned endpoint matrix;
- `d2e73ee` — authenticated version-admission remediation design;
- `0f799ca` — exact duplicate negotiation hello response replay;
- `5c4cbf1` — bind TCP multistream N1 negotiation transcript into the Noise prologue before Session data admission;
- `c44340a` — record the direct CLI -> wire dependency required by that integration.

The current merge has no GitHub commit-status checks or PR-triggered workflow runs attached through the GitHub API. Repository-local test/gate logs are therefore evidence committed by the coding environment, not an independent GitHub CI attestation.

## Review verdict

**needs repair — correctness/evidence blockers before further RC work**

The previous repository-reconciliation blocker is resolved. The next priority is not a new feature or a broader WAN run. Review found two concrete acceptance defects and one evidence-model gap:

1. **N4 lifecycle readiness is incorrect in the live CLI integration.** `Lifecycle::finalize_readiness()` requires five readiness marks, but the actual TCP/UDP server paths provide only four before finalization. The server then continues serving even though lifecycle state is `FAILED`. The raw N8 logs confirm `lifecycle_state=FAILED readiness=false` before successful exchanges and before the SIGTERM shutdown result.
2. **N7-11 authenticated negotiation admission is only partially closed.** The implementation binds the exact N1 transcript into Noise for TCP multistream, but the process-level malicious-peer tests required by `docs/n7-11-version-authenticated-admission-design.md` are absent. Wire/crypto unit tests do not prove the executable CLI rejects a transcript-binding mismatch or unsupported-only negotiation before Session data admission.
3. **The N2 canonical-vector corpus is structurally validated but not executable evidence.** `scripts/validate-canonical-vectors.py` checks JSON shape and requires hard-coded `oracle=true` fields; it does not invoke current Rust encoders/decoders/state-machine logic. The corpus remains a useful candidate fixture (`freeze=false`) but must not be treated as proven canonical interoperability vectors yet.

These are repairable within the existing architecture. No maintainer value decision or new authorization is required.

## Evidence boundaries

- `docs/status.md` correctly keeps `IMPLEMENTATION_COMPLETE=true` separate from `RELEASE_CANDIDATE=false`, `PRODUCTION_READY=false`, `FREEZE=false`, and `RELEASED=false`.
- N8 endpoint evidence is bounded authenticated runtime evidence, not public-WAN/NAT evidence. `192.168.122.1` is explicitly a self-owned host-address path from the same host, not an independently observed Internet path.
- The four N8 matrix rows do support bounded TCP/UDP authenticated multi-record exchange. That result survives the lifecycle-readiness defect; what does **not** survive is any claim that those runs demonstrated a valid READY lifecycle transition.
- `artifacts/n8-20260831/tcp-loopback-server.log` and `lifecycle-sigterm.log` explicitly record `lifecycle_state=FAILED readiness=false` before `STOPPED`. Preserve these raw artifacts; do not edit them into a pass.
- `artifacts/n8-20260831/lifecycle-result.txt` saying `SIGTERM graceful shutdown PASS` can support bounded signal termination/cleanup only. It does not prove READY was ever reached.
- N5 package evidence supports install A -> upgrade B -> rollback A, external-state retention, package hashes and cleanup on the self-owned x86_64 host. It is package-lifecycle evidence, not public-WAN evidence.
- N3 correctly defers previous/current interoperability until a real prior frozen release exists; current/current and unsupported/future rejection are the valid present claims.
- N1 negotiation is bounded and fail-closed. TCP multistream now cryptographically binds its exact negotiation transcript to Noise, but that does not yet mean all CLI probe/failover/UDP paths negotiate versions.
- The candidate canonical-vector corpus has `freeze=false`. Structural schema validation is not equivalent to executing the vectors against the implementation.
- `docs/standing-vps-lab-authorization.md` remains authoritative for in-scope bounded self-owned client <-> VPS TCP/UDP/WAN work. Ordinary `40080/40081`, bounded capture, diagnostics, benchmark, HY2 comparison and cleanup do not require another per-run approval.
- Release/security/production claims remain blocked even though bounded self-owned WAN execution is authorized.

## Half-Day Work Package

### Primary — Repair N4 lifecycle readiness and supersede the contradictory evidence

**Goal**

Make the live CLI server lifecycle truthful: STARTING must become READY only after explicit, meaningful prerequisites are satisfied; a failed readiness transition must never continue into serving application traffic; bounded signal shutdown must start from a genuinely READY server when that is what the evidence claims.

**Why now**

This is a live correctness and evidence-integrity defect in a release-gate area. The repository currently contains successful exchange logs whose server lifecycle simultaneously says `FAILED`. Continuing RC work would compound that contradiction.

**Preconditions**

- Preserve current raw N8 artifacts unchanged.
- Read `crates/neko-cli/src/lifecycle.rs`, the TCP/UDP server paths in `crates/neko-cli/src/main.rs`, existing CLI process tests and N8 audit/evidence files.
- Do not change protocol wire semantics, identity format, trust policy or standing authorization.

**Likely files / crates**

- `crates/neko-cli/src/lifecycle.rs`;
- `crates/neko-cli/src/main.rs`;
- CLI process/integration tests;
- a new small lifecycle-repair evidence note/artifact set;
- if necessary, a narrow erratum/superseding note linked from `docs/n8-wan-loopback-audit-20260831.md` without rewriting old raw logs.

**Minimum behavior**

1. Replace the anonymous readiness counter with named, idempotent prerequisites or an equivalent explicit representation. Duplicate marks must not compensate for a missing prerequisite.
2. The prerequisites should correspond to real server facts, for example: configuration accepted; identity/trust/state initialized; socket successfully bound; runtime initialized; accept/receive loop ready. Use the implementation's actual architecture rather than mechanically copying these names.
3. READY may be emitted only after all required prerequisites are true and before the first application exchange is accepted.
4. If readiness finalization fails, the process must fail/exit; it must not continue accepting TCP or UDP traffic in `FAILED` state.
5. SIGTERM/SIGINT after READY must enter the existing bounded shutdown path, stop accepting new work, release the listener/socket, and reach STOPPED within the current bounded contract. Do not invent seamless active-session drain if it is not implemented.
6. Startup/bind/configuration failure must never emit READY.

**Tests / fixtures**

- lifecycle unit test: each named prerequisite is required; duplicate setting is idempotent; one missing prerequisite cannot become READY;
- TCP process test waits for an observable READY line before starting the client exchange and verifies the exchange succeeds;
- UDP equivalent where supported;
- startup failure/invalid bind test verifies no READY line;
- SIGTERM-after-READY process test verifies bounded STOPPED and listener release/rebindability;
- cleanup assertion for experimental processes/listeners.

**Evidence repair**

- Keep the existing N8 raw logs exactly as historical evidence.
- Add a new repair evidence bundle/note showing a genuine READY -> DRAINING/STOPPED (or READY -> STOPPED if the bounded contract intentionally elides observable DRAINING) path.
- Explicitly state that the old N8 exchange matrix remains valid as exchange evidence but its readiness subclaim is superseded because the old logs recorded FAILED.

**Validation**

Run at minimum:

```text
cargo fmt --all -- --check
cargo check --workspace --locked
cargo test --workspace --all-targets --locked --no-fail-fast
cargo clippy --workspace --all-targets --locked -- -D warnings
bash scripts/check.sh
git diff --check
```

No wire/parser change is expected in this slice, so fuzz is not mandatory unless the repair touches such code.

**Completion definition**

A real TCP/UDP server cannot serve in FAILED readiness state; process tests observe READY before traffic; bounded shutdown/cleanup is demonstrated from a genuinely READY process; old contradictory evidence is preserved and correctly superseded rather than rewritten.

**Do not expand into**

- systemd/service-manager integration;
- persistent daemonization;
- new protocol features;
- longer soak runs;
- public-WAN/failover/benchmark work;
- RC freeze/release declaration.

### Follow-up 1 — Close N7-11 with executable malicious-peer tests

**Dependency:** Primary complete and full gate green.

Use the already-committed remediation contract in `docs/n7-11-version-authenticated-admission-design.md`; do not redesign it unless implementation facts force a documented decision change.

Add the missing process-level acceptance tests in the TCP multistream path:

1. A raw/malicious TCP peer completes a syntactically valid N1 exchange but constructs Noise with a one-byte-different negotiation binding (or otherwise proves exact transcript mismatch). The executable must fail before a successful `SecureSession`/`ProcessMessage`/multistream success diagnostic exists.
2. An unsupported-only negotiation must fail closed before Noise/data admission. Do not add an operator CLI version-selection feature merely to make the test convenient; a test peer/fixture is sufficient.
3. Preserve the existing positive multistream path and allowlist rejection tests.

Verify that external error output remains suitably uniform and does not turn negotiation/authentication internals into a useful oracle beyond the existing research boundary.

Run targeted `neko-wire`, `neko-crypto`, and `neko-cli` tests plus the full repository gate. If negotiation/parser code changes, rerun fuzz smoke.

Only after these executable negative paths pass may the N7-11 design document be marked closed. Do not fan version negotiation into UDP/probe/failover in this slice; the design itself explicitly reserves those for later anti-replay/amplification/resume work.

### Follow-up 2 — Make canonical vectors executable rather than self-attested

**Dependency:** Follow-up 1 complete, or Primary complete if this is used as the independent fallback while N7 process testing is temporarily blocked.

Turn `fixtures/canonical-vectors.v1.json` from a structurally valid candidate corpus into machine-executed evidence:

- add a Rust integration test, small adapter, or equivalent executable harness that loads the fixture and actually invokes current implementation logic for supported operations;
- prove byte equality, decode/expected equality and round-trip behavior where those operations have real canonical bytes;
- for expected failures/state-only concepts with no meaningful encoder or canonical byte representation, define truthful oracle semantics instead of asserting `encode_equals_bytes=true` by convention;
- keep `scripts/validate-canonical-vectors.py` as a structural/schema gate if useful, but do not let hard-coded booleans substitute for execution;
- if some current entries are conceptual contract fixtures rather than wire vectors, classify/move them rather than fabricating byte interoperability;
- keep `freeze=false`; N9 alone may freeze a reviewed corpus.

Run the executable vector test, structural validator and full `scripts/check.sh` gate.

### Follow-up 3 — Reconcile stale navigation/release-boundary documentation

**Dependency:** correctness/security evidence blockers above closed, unless performed as a documentation-only fallback that does not change implementation claims.

Several repository-facing documents now lag the reconciled code:

- `README.md` still says `Research bootstrap / pre-Milestone 0` despite `docs/status.md` recording the bounded research implementation complete;
- `docs/specs/nekomusume-session-v0.md` says current Rust models do not implement sockets/runtime/cryptography/live failover, which is no longer a true repository-wide statement. Reframe this as the **scope of that provisional normative document**, not absence of code;
- `docs/spec/m5-release-readiness-gate.md` still lists self-owned VPS/non-loopback execution itself as blocked. Update it to distinguish standing **execution authorization** from still-missing **release evidence/security/production approval**;
- `docs/era4-protocol-release-v1.md` still says no wire negotiation implementation exists. Update the boundary precisely: the N1 primitive exists and TCP multistream has authenticated admission; global probe/failover/UDP negotiation is not thereby proven;
- `IMPLEMENTATION_PLAN.md` currently duplicates the roadmap (including its `# 猫娘 Roadmap` title) even though `AGENTS.md` treats it as the executable-plan source. Restore a coherent executable current N/RC plan or explicitly redefine the execution source so agents do not select stale roadmap checkboxes as work.

Do not convert candidate/blocked items to PASS merely to make documents look consistent. Preserve `RELEASE_CANDIDATE=false`, `PRODUCTION_READY=false`, `FREEZE=false`, and `RELEASED=false` until their independent gates are actually satisfied.

### Fallback

If the lifecycle repair is unexpectedly blocked by a real implementation dependency, do **not** jump ahead to RC freeze or broaden WAN experiments. Keep the lifecycle defect explicit and choose one independent bounded repair that does not rely on false readiness, preferably:

1. executable canonical-vector evidence; or
2. the documentation/status drift repair above.

A standing-authorized WAN experiment is not an authorization blocker, but it is also not the right fallback while a release-area correctness defect is open.

## Completion gates

- Live server reaches READY only from explicit complete prerequisites.
- FAILED readiness cannot continue serving traffic.
- Process-level TCP/UDP readiness and signal-shutdown tests pass with cleanup/rebind evidence.
- Historical N8 logs remain immutable; new evidence explicitly supersedes only the invalid readiness subclaim.
- N7-11 malicious transcript/unsupported negotiation tests reject before Session data admission.
- Canonical vectors have executable implementation-backed oracles; structural booleans alone are not treated as proof.
- Full repository verification passes after each behavior-changing slice.
- Release/production/freeze flags remain truthful.

## Do not expand into

- new experimental carriers, 0-RTT, FEC enablement or heterogeneous striping without an observed problem and a fresh gate;
- third-party targets or scanning;
- production firewall/route/DNS/proxy/tunnel/qdisc changes;
- long-duration/high-volume/high-concurrency experiments outside standing authorization;
- calling self-owned host-address evidence public WAN/NAT evidence;
- calling candidate canonical vectors frozen interoperability evidence;
- declaring RC/production/security approval from local tests or E3 observations.

## Questions requiring maintainer decision

none.
