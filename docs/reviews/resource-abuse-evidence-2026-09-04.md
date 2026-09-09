# Resource and abuse-limit evidence review — 2026-09-04

## Current developer-tested reconciliation — `fe4c85a`; historical review foundation — `6952fa9`

**Engineering evidence state:** `ENGINEERING_CONTROLS_REVIEWED` for the bounded non-policy pre-auth controls and model/process observation through exact `fe4c85a2213cd57f7ca6d1d3a6cd1b7a48bfc4ee`: source tuple projection, charge-before-parse and exact response charging, all nine inventoried responder surfaces/seven admission sites, including source-owned cached first-Noise retries, common pre-work charging of every owned-peer datagram until authenticated progress, and terminal retirement of secure/cache/guard state when that owner expires, sibling-scoped pending-owner inventory, controller/state/attempt-bound one-pending response permits, atomic rollback and terminalization, source/global concurrency ceilings, expiry/release/error cleanup, earliest-bound response deadlines, and aggregate-only secret-safe diagnostics. This is a bounded exact-tree engineering review, not an independent security audit or adversarial-load/capacity approval.

**Policy state:** `SOURCE_RETENTION_POLICY_BLOCKED`. D019 terminal source-accounting retention after the last live state disappears remains unresolved; the current release/reopen behavior is retained as evidence of that conflict, not compliance. No TTL/LRU/history bound, external authority, or D019 weakening is selected here.

**Claim boundary:** RSEC-001 is not fully closed. This bounded exact-tree engineering review is deterministic/process evidence only, not an independent security audit, adversarial-load or production-capacity result, RC decision, or release authorization. `RELEASE_CANDIDATE=false`, `PRODUCTION_READY=false`, `FREEZE=false`, `RELEASED=false` remain required.

**Historical review-foundation tree:** exact `6952fa9928d895fcd3f9e51bfeebbe3719660488`. Current developer-tested facts above and the ChatGPT reviewer handoff are review/navigation input, not the independent two-person security approval required by D019 and release item 4.

**Scope:** reviewer-facing mapping of repository evidence to `SECURITY.md`. This is not an external audit, security approval, production-capacity claim, RC decision, or protocol freeze.

## Findings

### RSEC-001 — Pre-auth admission and bounded model/process recovery observations exist; adversarial-load suitability and promotion remain absent

**Severity:** HIGH for release/security promotion; does not invalidate bounded authenticated research probes.

`crates/neko-crypto/src/lib.rs::PreauthBudget` enforces per-state input/response bytes and packets plus the 3x anti-amplification rule. `ProcessPreauthAdmission` supplies process-owned source/global concurrency, memory, queue, source-lifetime input/response/work, global rate-window, per-packet work, monotonic idle/lifetime and cleanup accounting with atomic deterministic boundary tests.

`crates/neko-cli/src/preauth.rs::ListenerAdmission` now binds that state to the received IP-family/address/port tuple without retaining textual addresses. The ordinary TCP/UDP probe, failover TCP/UDP, endpoint-rebind UDP, periodic TCP and multistream TCP listener paths reserve bounded state on accept/first datagram, charge bounded frame/datagram input before negotiation or Noise parsing, charge exact framed/wire response bytes before send, use controller/state/attempt-bound one-pending permits with the earliest state/socket deadline, and release state only after authenticated data admission. UDP malformed first packets remain silent; authenticated Session data is outside this pre-auth budget.

This is implementation and deterministic/process-test evidence, not independent security review, adversarial load evidence, production-capacity validation, or proof that candidate numeric values are suitable for public deployment. The executable remains a bounded temporary research probe and must not be promoted to a public listener, RC, production-capacity or security-approved service.

Exact `f74b823` adds one bounded deterministic response-accounting observation at the small test limits. Exact `21e42af` adds compact checks at the unchanged default candidate boundaries (8 same-source states, 1,024 global states, 256 queued states and 2,048 response bytes / 4 packets) plus one real-process recovery observation. Exact `976f90b` makes the source-domain evidence executable by pre-binding and retaining eight sockets with asserted-unique source ports for eight five-byte malformed UDP attempts, then completing one authenticated 16-byte exchange in the same long-lived server process with bounded Linux FD/RSS observations where `/proc` exists and asserted listener/identity cleanup. It also retains the original admission through cached first-Noise retries, exact-charges each retry, rejects the first operation beyond the four-packet response ceiling, and releases/cache-clears on authenticated progress. Exact `5be8c6e` moves input/work charging to the common owned-peer receive path before cached-message comparison or AEAD open: the cached retry and first non-matching authenticated Data each charge exactly once, the first input beyond the four-packet fixture ceiling terminalizes before classification, and post-progress authenticated traffic leaves the pre-auth domain only after owner release. Exact `9695621` proves that owner expiry before first Data also retires the associated secure/cache/ResumeGuard capability before receive/classification; a 1,200 ms delayed stale Data obtains no DeliveryAck, while a fresh admitted negotiation in the same bounded process succeeds with exactly one authenticated 16-byte record. These are not adversarial-load capacity/suitability evidence and do not recommend candidate values.

**Required evidence to close the promotion finding:** representative bounded adversarial-load evidence, suitability review of candidate limits, and a separate release/security decision. The exact-tree engineering review below does not change those requirements or candidate numeric limits.

## Evidence matrix

| Security red line | Exact implementation/test evidence | Evidence class | Boundary / unresolved finding |
|---|---|---|---|
| No custom cryptographic primitive | `crates/neko-crypto/Cargo.toml`; `crates/neko-crypto/src/lib.rs` Noise implementation and dependency review in `docs/research/crypto-dependency-review-2026-08-29.md` | local deterministic + review input | Library use is evidenced; independent cryptographic/security review remains absent. |
| Nonce uniqueness / key phase fails closed | `NonceSequence`, replay-window and synchronized/unsynchronized key-update tests in `crates/neko-crypto/src/lib.rs` | local deterministic | State-model and bounded runtime evidence only; real-session live key update remains implementation-blocked. |
| Unauthenticated control cannot mutate delivery/path/ACK state | handshake/trust/authz negative tests in `crates/neko-crypto/src/lib.rs`; negotiation and malformed setup tests in `crates/neko-cli/tests/probe.rs`; evidence-domain tests in `crates/neko-carrier/tests/integration_gates.rs` | local deterministic/process | No public service/security approval. RSEC-001 still blocks process-wide admission claims. |
| UDP anti-amplification | `PreauthBudget`, `ProcessPreauthAdmission`, `crates/neko-cli/src/preauth.rs`, probe/failover listener integration and tests | local deterministic/process | Per-state 3x plus source/global accounting is integrated; adversarial load evidence remains absent (RSEC-001). |
| Malformed lengths/counts/offsets and allocation bounds | `crates/neko-wire/src/lib.rs`, `crates/neko-wire/tests/canonical_vectors.rs`, `crates/neko-session/src/lib.rs`, `crates/neko-cli/src/framed.rs`, fuzz target/corpus | local deterministic + fuzz CI | Candidate parser evidence, not exhaustive proof or production-safety claim. |
| Unknown version/type/frame behavior | canonical corpus, `crates/neko-wire/tests/n3_compatibility.rs`, CLI negotiation process tests | local deterministic/process | Corpus v1 is frozen; global protocol and release interoperability are not frozen. |
| Duplicate/old packet, replay and ResumeGuard | replay-window and `ResumeGuard` tests in `crates/neko-crypto/src/lib.rs`; resumed-session and process-runner tests in `crates/neko-carrier/tests/` | local deterministic/process | Bounded candidate behavior; no previous frozen release exists. |
| Per-connection memory/queue/offset bounds | `RuntimeLimits`/`SessionRuntime`, `DeliveryLedger`, datagram queue tests in `crates/neko-session/src/lib.rs`; carrier-manager bounds in `crates/neko-carrier/src/lib.rs` | local deterministic | Hard constructors and atomic queue tests exist. Cross-session process ownership is not a production admission controller (RSEC-001). |
| Global memory/CPU/rate bounds | `ProcessPreauthAdmission` in `crates/neko-crypto/src/lib.rs`; `ListenerAdmission` and all executable server call sites in `crates/neko-cli/src/`; hard runtime and CLI bounds | local deterministic/process | Integrated candidate controls exist. **RSEC-001 HIGH remains open for adversarial load evidence and promotion suitability.** |
| Secret-safe logging / no plaintext payload or key disclosure | observability schema/checker; `capabilities --json` secret-free checks; CLI output review | local deterministic/static | Public keys and aggregate byte/hash metadata are intentionally visible. No claim of side-channel elimination. |
| No open proxy/default forwarding | `crates/neko-cli/src/main.rs` declares bounded probe-only behavior; server commands perform fixed authenticated echo/session workloads and expose no forwarding destination | local static/process | Research listeners remain temporary and bounded; no production daemon approval. |
| Cleanup/resource exhaustion paths | Session cancellation/timeout tests, lifecycle signal/rebind process tests, owned-lab control-plane cleanup tests, retained negative artifacts | local deterministic/process + bounded VPS history | Historical artifact cleanup fields remain immutable; later postchecks do not rewrite failures. |

## Verification

- Focused preauth/inventory tests passed on exact `6952fa9`; full `scripts/check.sh` was also exercised, but its periodic process tests exposed an existing startup-scheduling failure boundary (EOF before `periodic_server_ready`) on this run. The exact-tree engineering review does not claim that flaky gate as green.
- A clean exact-tree local `scripts/check.sh` run is the primary reproducible gate for the pushed implementation commit; hosted GitHub Actions, when available, is optional cross-evidence.
- No protected identity, credential, private endpoint material or raw private diagnostics were read or added by this review.

## Promotion boundary

`RELEASE_CANDIDATE=false`, `PRODUCTION_READY=false`, `FREEZE=false`, and `RELEASED=false` remain required. RSEC-001 blocks release/security/public-listener promotion, not unrelated bounded local correctness work.