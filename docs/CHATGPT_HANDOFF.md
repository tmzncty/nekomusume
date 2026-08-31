# Nekomusume ChatGPT Handoff

Checked at: 2026-09-01 01:02 Asia/Shanghai
Repository HEAD reviewed: `ee555e2874420b7cf92a4c088a617daa94b8b23c`
Previous checked HEAD: `23008e904fd7a5ab258349e4679e973c5eb41555`

## What changed

The previous repair package was substantially completed and pushed as four reviewable commits after the prior handoff:

- `4dcb871` — **implementation + tests + bounded local evidence**: replaces anonymous lifecycle readiness counting with named idempotent prerequisites, makes readiness finalization fail closed, and adds process-level READY / invalid-bind / SIGTERM / listener-rebind evidence. The historical N8 raw logs are preserved and only their readiness subclaim is superseded.
- `53c1497` — **tests**: adds executable malicious-peer coverage for the TCP multistream authenticated-admission path. A one-byte negotiation-binding mismatch fails during Noise before `SecureSession`/Session data admission; unsupported-only negotiation is rejected before Noise/data admission with the existing uniform external error surface.
- `b6d3cd6` — **test harness + fixture repair**: adds a Rust integration test that loads `fixtures/canonical-vectors.v1.json`, exercises current wire/negotiation code for a subset of entries, and changes several fixture oracle flags/classifications so expected-failure/state-only entries are no longer universally self-described as byte-roundtrippable.
- `ee555e2` — **documentation/navigation**: reconciles README/release-boundary text and converts `IMPLEMENTATION_PLAN.md` back into an actual current N/RC execution queue rather than a duplicate roadmap.

The repository-local full validation log committed with the N4 repair shows format/check/workspace tests/clippy and repository gates passing. GitHub still has no independent commit-status checks attached to `ee555e2`; this remains coding-environment evidence rather than an independent CI attestation.

## Review verdict

**needs repair, but the previous correctness blockers are closed. N9 is READY and is not blocked by the later independent security-review gate.**

The previous handoff's N4 and N7-11 blockers are satisfactorily repaired within their stated bounded scope:

1. Current `server()` explicitly marks five named readiness prerequisites and exits on incomplete readiness before entering the accept/receive loop. The previous “serve while FAILED” defect is closed.
2. The TCP multistream path now has process-level negative tests for exact negotiation-transcript mismatch and unsupported-only negotiation before Session data admission. The N7-11 bounded TCP admission claim is supportable.

However, N9 candidate-corpus freeze is **not ready to PASS yet**. This reviewer run is the independent corpus review required by the current `IMPLEMENTATION_PLAN.md`, and it found concrete remaining corpus/oracle defects below. Therefore the coding agent must not keep N9 blocked waiting for an “external reviewer handoff”: this handoff is that independent N9 review. The separate **independent release/security review** remains a later release gate and must not be collapsed into N9.

Also reconcile any stale task label such as `N9 Freeze RC manifest`. The authoritative current plan defines N9 as **candidate-corpus review and freeze decision** and explicitly says N9 must not imply RC, security approval, or release. Full RC/release evidence is a later dependency.

## Evidence boundaries

- `RELEASE_CANDIDATE=false`, `PRODUCTION_READY=false`, `FREEZE=false`, and `RELEASED=false` remain correct.
- N4 repair evidence is bounded loopback/process-lifecycle evidence, not WAN or production service evidence.
- N7-11 coverage is **TCP multistream only**. Ordinary `server/client` probe, UDP, and failover/resume still do not thereby gain version negotiation.
- The candidate vector corpus remains `freeze=false` and must stay so during the repair below.
- The new Rust canonical-vector test is real executable evidence for several entries, but it still does not enforce every `oracle.*=true` claim against the fixture's actual `bytes_hex`.
- State-only/conceptual contract fixtures are useful, but they are not canonical wire bytes and must not look like frozen interoperability bytes.
- Standing self-owned VPS authorization remains active. Missing ordinary per-run approval is not a blocker for later bounded TCP/UDP release-evidence work.
- Independent release/security review is still required before RC, but it is **not** a prerequisite for performing N9 corpus repair/review.

## Work Package

### Primary — Close N9 corpus/oracle defects and return it for freeze review

**Goal**

Make `fixtures/canonical-vectors.v1.json` truthful and fully machine-enforced wherever it claims canonical bytes or executable decode/round-trip behavior. Do not freeze it in the same implementation commit; return the repaired candidate (`freeze=false`) for the next independent reviewer decision.

**Why now**

`IMPLEMENTATION_PLAN.md` correctly names N9 as the first READY release-engineering task. The current harness is a strong improvement over structural validation, but several oracle claims are still partially self-attested and the corpus mixes actual wire vectors with pseudo-byte state fixtures. Freezing now would turn those ambiguities into a compatibility contract.

**Concrete review findings to repair**

1. `negotiation.hello.v0-v2` asserts emitted bytes equal `bytes_hex`, but does not actually feed the fixture bytes through a server-side negotiation operation to prove the claimed decoded semantic versions. If `decode_bytes_equals_expected=true`, the test must exercise the fixture bytes, not only a freshly generated equivalent.
2. `negotiation.no-overlap` and `negotiation.duplicate` build fresh hello bytes from `input` and execute those; they do not require the fixture's `bytes_hex` to be the bytes that produced the expected error. Enforce fixture-byte equality/consumption directly.
3. `frame.oversized` proves that constructing an oversized `Frame::Datagram` fails encoding, but it does not decode the fixture `bytes_hex` even though `decode_bytes_equals_expected=true`. Execute `decode_frames(bytes_hex)` and verify the expected `LengthTooLarge` path.
4. `frame.datagram-max` is semantically misnamed: it contains a one-byte payload (`0c000100`), not the 1024-byte frame payload boundary. Rename it to a truthful minimum/small vector and add a real max-boundary vector, or change the payload to the actual maximum with a deterministic byte oracle.
5. Several `state_only` entries carry non-empty `bytes_hex` despite all byte oracles being false (`ack.range-max`, key-update entries, carrier transition, etc.). A freeze corpus must not make invented/pseudo bytes look canonical. Either move state-only cases to a separate state-contract fixture, allow an explicit `bytes_hex: null`/absent representation, or otherwise make the schema unmistakably non-wire for those rows.
6. `close.empty` is currently classified `state_only`, but `Frame::Close` has a real wire codec (`04 0000`). Promote this row to an actually executed frame vector and enforce encode/decode/round-trip bytes, rather than leaving a real wire surface untested in the canonical corpus.
7. Review the rest of the version-sensitive public wire surface for freeze coverage. At minimum decide explicitly whether the frozen corpus covers or intentionally excludes: outer `RecordType::{Data,Ack,PathChallenge}`, `Frame::{Data,Datagram,DeliveryAck,Close,PathChallenge,PathResponse}`, unknown-ignorable/unknown-critical/reserved frame behavior, negotiation response bytes, malformed/unsupported selected-version response, truncation, max frame count/payload, and canonical varint boundaries. Do not add dozens of rows merely for volume; every omission must be deliberate and documented.

**Likely files**

- `fixtures/canonical-vectors.v1.json`;
- `crates/neko-wire/tests/canonical_vectors.rs`;
- `scripts/validate-canonical-vectors.py` if schema semantics change;
- a small `docs/` corpus-scope note only if needed to state what is and is not frozen.

**Minimum behavior / tests**

- Every row with `oracle.encode_equals_bytes=true` executes the real encoder and compares with fixture bytes.
- Every row with `oracle.decode_bytes_equals_expected=true` consumes the fixture's actual `bytes_hex` through real implementation logic and verifies the expected value/error.
- Every row with `oracle.roundtrip_equals_bytes=true` performs the actual decode/encode (or protocol-equivalent) round trip.
- State-only rows never masquerade as canonical bytes.
- Expected failures remain failures; no “PASS” is manufactured by changing expected semantics to match bugs.
- `freeze` remains `false` in the coding-agent repair commit.

**Validation**

Run targeted `neko-wire` canonical-vector tests, the structural validator, the full repository gate, and fuzz smoke if parser/decoder/schema-adapter code changes in a way covered by the fuzz policy.

**Completion definition**

The candidate corpus is internally truthful, executable oracles are implementation-backed rather than boolean self-attestation, conceptual/state fixtures are clearly separated from byte interoperability, coverage/exclusions are explicit, and the corpus is ready for the next ChatGPT independent freeze review while still `freeze=false`.

**Do not expand into**

- RC declaration or RC manifest freeze;
- changing wire semantics just to make fixture bytes convenient;
- inventing previous-release interoperability;
- new carriers or unrelated performance work.

### Follow-up 1 — Complete authenticated negotiation for ordinary TCP/UDP probe paths

**Dependency:** Primary complete and full gate green. If the next reviewer freezes/accepts the corpus first, consume that handoff before changing frozen bytes; otherwise this slice must preserve the repaired candidate wire semantics.

The current N1 primitive and TCP multistream transcript binding are not global runtime negotiation. Integrate the same bounded negotiation-before-Noise rule into the ordinary `neko server/client` TCP and UDP authenticated probe paths.

Required contract:

1. N1 negotiation completes before the first Noise/session-data message is accepted.
2. The exact accepted hello + selected response are bound into the Noise prologue/authenticated transcript exactly as in the reviewed multistream design; same selected version with a different offer must not authenticate as the same negotiation.
3. Unsupported-only, malformed, duplicate/late, and transcript-mismatch peers fail before Session data admission.
4. TCP framing remains bounded.
5. UDP negotiation response must stay within the existing anti-amplification/resource boundary; duplicate hello may replay the exact accepted response, but changed/late unauthenticated messages must not mutate established state.
6. Preserve current uniform external failure behavior unless a documented operator-only diagnostic is already allowed.
7. Do not add 0-RTT or early application data.

Add positive and malicious-peer process/integration tests for both TCP and UDP. If wire/parser code changes, run fuzz smoke in addition to the full gate.

### Follow-up 2 — Extend negotiation binding through failover/resume without weakening Session/Carrier separation

**Dependency:** Follow-up 1 complete.

Specify and implement the version-binding rule for the failover/resume path. The selected protocol version and exact negotiation context must not be lost when the logical Session moves from UDP to TCP fallback/resume.

Minimum acceptance questions/tests:

- Can a fallback carrier negotiate a different unsupported/downgraded version and still resume the same logical Session? It must fail closed unless the documented compatibility contract explicitly permits the transition.
- Is the negotiation/version context bound to the authenticated resume material/generation so replay from an old carrier generation cannot advance the resumed Session?
- Do uncertain resend/dedup and Session delivery evidence remain independent of carrier packet ACK/readiness evidence?
- Does duplicate/loss recovery remain bounded without turning negotiation messages into application delivery proof?

Keep single-active/multi-ready semantics and the existing no-striping decision. Record any genuinely new architecture choice in `docs/decisions.md` before implementation if the current accepted decisions do not already determine it.

### Follow-up 3 — Start the standing-authorized bounded release-evidence matrix

**Dependency:** Follow-up 2 complete and full local gate green.

Use `docs/standing-vps-lab-authorization.md` directly; do not ask again for ordinary self-owned VPS TCP/UDP permission or count/bytes/duration/port. Start with the smallest reproducible rows that exercise the newly negotiated runtime:

1. independently controlled self-owned VPS TCP authenticated negotiated session;
2. self-owned VPS UDP authenticated negotiated session;
3. bounded UDP degradation -> TCP fallback/resume with exact Session continuity/dedup evidence;
4. bounded short soak within the standing <=10-minute limit if the first three rows pass.

For each row record experiment ID, exact git/binary identity, actual parameters, endpoint-ownership classification, client/server result, structured events, capture metadata if used, and cleanup verification. Negative results remain evidence. Do not call self-owned same-host/host-address observations “public Internet reachability”, NAT evidence, or production readiness.

If IPv6 is unavailable in the actual environment, classify that row `BLOCKED_ENVIRONMENT`; do not invent failure or stop IPv4-independent work. NAT/endpoint-change work should only be attempted when a controllable path actually exists and remains within standing authorization.

### Fallback — Repair authorization/status/navigation drift only if a runtime dependency blocks the main path

There is still one concrete navigation inconsistency: `ROADMAP.md` Milestone 1 says real WAN failover/long-lived/NAT validation is blocked because it “needs new authorization”, while `AGENTS.md` and `docs/standing-vps-lab-authorization.md` explicitly grant ordinary bounded self-owned VPS execution. Correct that wording so the remaining blockers are evidence/environment/release scope, not nonexistent per-run authorization.

Also review `docs/status.md` reachability wording for the same distinction: execution authorization can be present while release/public-reachability claims remain blocked. Do not turn those claims into PASS.

This fallback is documentation/governance repair only; it must not replace the Primary corpus work when the Primary is executable.

## Completion gates

- N9 corpus-review defects above are repaired with `freeze=false` and returned for independent freeze review.
- No `oracle.*=true` remains self-attested without exercising the fixture bytes/real implementation semantics it claims.
- State-only pseudo-bytes are not presented as canonical interoperability bytes.
- Ordinary TCP/UDP probe paths negotiate before Noise and cryptographically bind the exact accepted negotiation transcript before Session data.
- Failover/resume cannot silently downgrade or lose the negotiated-version context across carrier transition.
- Standing-authorized release-evidence rows preserve exact experiment parameters, endpoint ownership, negative results and cleanup.
- Full repository gates remain green after each behavior-changing slice.
- Release/production/freeze flags remain truthful until separate decisions actually pass.

## Do not expand into

- RC/production/security approval before their later gates;
- previous/current interop before a real prior frozen release exists;
- 0-RTT, FEC enablement, concurrent striping, heterogeneous aggregation or exotic carriers without a new observed-problem gate;
- third-party targets or scanning;
- production firewall/route/DNS/proxy/tunnel/qdisc changes;
- >10-minute, >256 MiB, >32-session or other experiments outside standing authorization;
- treating one bounded self-owned VPS result as general public reachability or performance superiority.

## Questions requiring maintainer decision

none.
