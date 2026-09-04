# Resource and abuse-limit evidence review — 2026-09-04

**Reviewed tree:** `bb9e26814365e82465a6636ac7d7f74107b4b53f`

**Scope:** reviewer-facing mapping of repository evidence to `SECURITY.md`. This is not an external audit, security approval, production-capacity claim, RC decision, or protocol freeze.

## Findings

### RSEC-001 — Process-global pre-auth admission controller exists but is not integrated

**Severity:** HIGH for release/security promotion; does not invalidate bounded single-session local tests.

`crates/neko-crypto/src/lib.rs::PreauthBudget` enforces one budget object's input/response bytes and packets plus the 3x anti-amplification rule. `ProcessPreauthAdmission` now supplies process-owned, source-keyed/global concurrency, memory, queue, input/response rate-window, work-unit, monotonic lifetime and cleanup accounting with atomic deterministic boundary tests. The executable handshake path remains bounded separately by CLI framing, time, payload and command limits.

The process admission controller is not yet composed into each TCP/UDP listener before allocation, parse, queue and response scheduling. Its source key remains an opaque caller projection rather than a reviewed carrier/source-tuple policy, and real listener instrumentation does not yet expose charge/release evidence. Therefore the repository must not claim that the explicit `SECURITY.md` global pre-auth memory/CPU/rate red line is satisfied in the executable runtime, and no public-listener, RC, production-capacity or security-approval path may be promoted on current evidence.

**Required evidence to close:** integrate the process-owned admission controller at every pre-auth listener boundary, bind a reviewed source-key projection, prove charge-before-allocation/parse/work/send plus expiry/concurrency/overflow/cleanup/no-evidence-on-rejection in process tests, then obtain independent security review. Do not invent replacement numeric limits merely to close this finding.

## Evidence matrix

| Security red line | Exact implementation/test evidence | Evidence class | Boundary / unresolved finding |
|---|---|---|---|
| No custom cryptographic primitive | `crates/neko-crypto/Cargo.toml`; `crates/neko-crypto/src/lib.rs` Noise implementation and dependency review in `docs/research/crypto-dependency-review-2026-08-29.md` | local deterministic + review input | Library use is evidenced; independent cryptographic/security review remains absent. |
| Nonce uniqueness / key phase fails closed | `NonceSequence`, replay-window and synchronized/unsynchronized key-update tests in `crates/neko-crypto/src/lib.rs` | local deterministic | State-model and bounded runtime evidence only; real-session live key update remains implementation-blocked. |
| Unauthenticated control cannot mutate delivery/path/ACK state | handshake/trust/authz negative tests in `crates/neko-crypto/src/lib.rs`; negotiation and malformed setup tests in `crates/neko-cli/tests/probe.rs`; evidence-domain tests in `crates/neko-carrier/tests/integration_gates.rs` | local deterministic/process | No public service/security approval. RSEC-001 still blocks process-wide admission claims. |
| UDP anti-amplification | `PreauthBudget`, `ProcessPreauthAdmission` and `preauth_tests` in `crates/neko-crypto/src/lib.rs` | local deterministic | Per-object 3x and process source/global accounting exist as composable state; listener integration and source-tuple policy remain absent (RSEC-001). |
| Malformed lengths/counts/offsets and allocation bounds | `crates/neko-wire/src/lib.rs`, `crates/neko-wire/tests/canonical_vectors.rs`, `crates/neko-session/src/lib.rs`, `crates/neko-cli/src/framed.rs`, fuzz target/corpus | local deterministic + fuzz CI | Candidate parser evidence, not exhaustive proof or production-safety claim. |
| Unknown version/type/frame behavior | canonical corpus, `crates/neko-wire/tests/n3_compatibility.rs`, CLI negotiation process tests | local deterministic/process | Corpus v1 is frozen; global protocol and release interoperability are not frozen. |
| Duplicate/old packet, replay and ResumeGuard | replay-window and `ResumeGuard` tests in `crates/neko-crypto/src/lib.rs`; resumed-session and process-runner tests in `crates/neko-carrier/tests/` | local deterministic/process | Bounded candidate behavior; no previous frozen release exists. |
| Per-connection memory/queue/offset bounds | `RuntimeLimits`/`SessionRuntime`, `DeliveryLedger`, datagram queue tests in `crates/neko-session/src/lib.rs`; carrier-manager bounds in `crates/neko-carrier/src/lib.rs` | local deterministic | Hard constructors and atomic queue tests exist. Cross-session process ownership is not a production admission controller (RSEC-001). |
| Global memory/CPU/rate bounds | `ProcessPreauthAdmission` in `crates/neko-crypto/src/lib.rs`; hard per-runtime ceilings in `crates/neko-session/src/lib.rs`; CLI command/duration/bytes bounds | local deterministic | Process-owned accounting state is tested but not integrated at listener boundaries. **RSEC-001 HIGH remains open.** |
| Secret-safe logging / no plaintext payload or key disclosure | observability schema/checker; `capabilities --json` secret-free checks; CLI output review | local deterministic/static | Public keys and aggregate byte/hash metadata are intentionally visible. No claim of side-channel elimination. |
| No open proxy/default forwarding | `crates/neko-cli/src/main.rs` declares bounded probe-only behavior; server commands perform fixed authenticated echo/session workloads and expose no forwarding destination | local static/process | Research listeners remain temporary and bounded; no production daemon approval. |
| Cleanup/resource exhaustion paths | Session cancellation/timeout tests, lifecycle signal/rebind process tests, owned-lab control-plane cleanup tests, retained negative artifacts | local deterministic/process + bounded VPS history | Historical artifact cleanup fields remain immutable; later postchecks do not rewrite failures. |

## Verification

- Full `scripts/check.sh` passed on the owned Linux host for exact `bb9e268`.
- Exact-head GitHub Actions is the CI authority for the pushed commit; this review does not substitute an older run.
- No protected identity, credential, private endpoint material or raw private diagnostics were read or added by this review.

## Promotion boundary

`RELEASE_CANDIDATE=false`, `PRODUCTION_READY=false`, `FREEZE=false`, and `RELEASED=false` remain required. RSEC-001 blocks release/security/public-listener promotion, not unrelated bounded local correctness work.