# Nekomusume ChatGPT Handoff

Checked at: 2026-09-01 10:07 Asia/Shanghai
Repository HEAD: `f680702a45a1bbbf0672a0b8d46b3756308c99b1`
Previous reviewed implementation HEAD: `23e300723ddd948652c7dae4e0409b414ea8a587`
Previous reviewer handoff commit: `f7214d4312ffc4f45d306a8ab0c152f9fd322e06`

## What changed

One substantive coding-agent commit landed after the previous reviewer handoff:

- `f680702` — **implementation + tests; closes the core authenticated-delivery-control defect identified at `23e3007`.** The failover runner now seals UDP `DeliveryAck` messages with the established unreliable secure session, opens and semantically validates the exact Session/stream/offset/length on the client before confirmation, seals resumed TCP `DeliveryAck` messages with the fresh TCP Noise transport and validates them before confirmation, removes the redundant ignored plaintext post-Noise `Resume` message, uses `SessionRuntime`-allocated outbound records and delivery acknowledgements on the client, replaces the hard-coded UDP byte diagnostic with actual ciphertext/application fields, and adds bounded UDP negotiation/Noise retry caches so same-peer retransmissions do not get misclassified as a new protocol stage.

The new tests cover exact DeliveryAck semantics, plaintext non-authentication, replay/tamper rejection through real `SecureSession` operations, positive process-level encrypted UDP/TCP acknowledgement events, and first-selection loss with a duplicate client hello while unrelated-peer traffic is present. The push-triggered GitHub **Rust CI run #75 completed successfully** for exact HEAD `f680702`.

No replacement VPS run has been committed after this fix yet. Therefore the old `23e3007` real-socket observation remains useful only for the facts explicitly preserved in the previous handoff: negotiated/authenticated UDP admission, negotiated/authenticated TCP resume, ordered server receive, and cleanup. Its old ACK wording is not retroactively upgraded by `f680702`.

## Review verdict

**SAFE TO CONTINUE WITH REQUIRED EVIDENCE/DOCUMENTATION CLOSURE — the core authenticated DeliveryAck repair is accepted; immediately consume the rented VPS window after the remaining local closure.**

The previously identified security/evidence blocker is closed at code level sufficiently to proceed to replacement bounded real-socket evidence. Do not reopen N9; the canonical corpus v1 freeze remains a separate closed fact. Do not mark the bounded release evidence matrix complete yet.

One retry-evidence seam remains worth closing before broader degraded-path claims: `f680702` implements a cached Noise first-response replay path (`handshake_cache`) but the new deterministic process test explicitly exercises first **negotiation-selection** loss, not first **Noise response** loss. That does not block the controlled post-fix VPS rerun, but it must be tested or explicitly bounded before claiming the UDP handshake path is robust under response loss.

## Evidence boundaries

- `CANONICAL_CORPUS_V1_FROZEN=true` remains correct; global `FREEZE=false`, `RELEASE_CANDIDATE=false`, `PRODUCTION_READY=false`, and `RELEASED=false` remain correct.
- `f680702` changes the executable failover/control path and tests; it does not create natural WAN degradation detection or a production failover service.
- GitHub Rust CI is green at `f680702`. This is independent repository CI evidence for the committed code, not a VPS/WAN result.
- The client now treats Session DeliveryAck success as cryptographic + semantic validation rather than “some bytes arrived”. UDP packet/path ACK and Session DeliveryAck remain distinct concepts.
- The controlled runner still performs an explicit application fault/UDP-stop transition. It is not evidence that the current runtime automatically detects a natural UDP blackhole/PTO threshold and invokes failover.
- `docs/status.md` is stale: its CLI row still says failover/resume negotiation is absent even though `12e918a` and current HEAD implement canonical negotiation + authenticated binding for that path.
- `docs/m3-wan-failover-gate.md` is stale: it still says public WAN use is prohibited pending review and describes a “next executable seam” that has already been substantially implemented. Standing authorization permits bounded self-owned TCP/UDP execution; public/general/production claims remain blocked.
- Standing authorization remains active. A replacement run is justified because the code/instrumentation materially changed after `23e3007`; this is not an unchanged rerun.
- The VPS is a time-limited one-month asset. Once the short closure below is green, VPS-only evidence outranks unrelated local polish.

## Work Package — close corrected evidence contract -> harvest VPS evidence -> unlock automatic failover/resource/comparison work

Execute A -> B -> C -> D -> E in order while dependencies remain green. Do not stop after the documentation patch if the VPS run is READY. If a new correctness/security defect appears, stop the affected evidence branch and use the fallback.

### Primary A — Close the post-`f680702` evidence contract and retry regression gap

**Goal**

Make the corrected implementation/evidence boundary explicit, finish the deterministic retry regression coverage already implied by the code, and remove status/navigation drift before collecting replacement release evidence.

**Required work**

1. Add an explicit supersession/correction note to the existing `docs/research/followup-b-20260901.md` and `docs/followup-d-vps-evidence-20260901.md` (or the repository's established supersession mechanism). Preserve old evidence rather than rewriting history. State precisely:
   - canonical negotiation/authenticated admission and server ordered receive observations remain valid;
   - the old `udp_ack_observed` / plaintext TCP ACK wording did not establish authenticated Session DeliveryAck;
   - `f680702` is the implementation repair, and only post-fix evidence may support authenticated DeliveryAck claims.
2. Update `docs/status.md` CLI/failover wording so it reflects current facts: generic probes **and failover/resume** perform canonical version negotiation before fresh Noise and bind the transcript before data/resume admission. Keep public/release/production status blocked.
3. Update `docs/m3-wan-failover-gate.md` to current reality. Remove obsolete “all WAN execution prohibited pending review” and “future runner” wording; reference standing authorization for bounded self-owned execution. Preserve the distinction between controlled endpoint-stop evidence and automatic threshold-driven degradation detection.
4. Add a deterministic process test for **first UDP Noise response loss** using the `handshake_cache` seam, or, if the current responder API cannot safely simulate that without a larger redesign, add an explicit tested/documented limitation that prevents this cache from being cited as response-loss evidence. Prefer a tiny test injection flag analogous to `--drop-first-udp-selection`; do not build a general packet-loss framework here.
5. Ensure duplicate/late negotiation hello and duplicate Noise first-message replay never reset selected version, ResumeGuard binding, Session id, path generation, or delivery state. Existing code appears designed this way; add an assertion where needed rather than relying on comments.
6. Keep `IMPLEMENTATION_PLAN.md` negotiation-path item checked; keep the bounded release evidence matrix unchecked.

**Validation**

Run targeted CLI/process tests for both selection-loss and Noise-response-loss seams, then the normal repository gate:

```text
cargo fmt --all -- --check
cargo check --workspace --locked
cargo test --workspace --all-targets --locked --no-fail-fast
cargo clippy --workspace --all-targets --locked -- -D warnings
bash scripts/check.sh
git diff --check
```

Run fuzz smoke only if parser/wire-decoder behavior changes. Do not manufacture a fuzz claim for a runner/test/doc-only patch.

**Completion definition**

The repository no longer overstates old ACK evidence, status/gate docs match the current negotiated failover runner, the bounded retry behavior implemented in `f680702` is covered truthfully, and all local gates are green.

### Follow-up B — Immediate replacement rented-VPS evidence batch

**Dependency:** A green and pushed; exact implementation/binary identity known.

This is the highest-value next work because it is difficult to reconstruct after the one-month VPS rental ends. It is fully within existing standing authorization when kept within the documented bounds.

Batch compatible scenarios under one cleanup-safe lab setup, but keep each scenario/result separately identifiable.

#### B1 — Post-fix standalone generic TCP and UDP negotiated real-socket sanity

Run separate bounded current/current TCP and UDP generic probe/application exchanges between the self-owned client and self-owned VPS using current HEAD. Record:

- exact git commit and binary/package identity;
- address-family label without committing unnecessary addresses;
- actual count/payload/duration/ports;
- selected canonical version;
- authentication/application success/failure;
- one bounded unsupported/malformed negotiation negative row if the existing command makes this practical without artificial complexity;
- final listener/process/temp-file cleanup state.

This is behavior/reachability evidence, not a performance claim.

#### B2 — Replacement controlled failover/resume run with authenticated DeliveryAck evidence

Repeat the controlled endpoint-stop scenario because `f680702` materially changed the ACK/control semantics. Require structured event/evidence ordering equivalent to:

```text
UDP canonical negotiation
-> UDP Noise authentication
-> first UDP logical record
-> encrypted + exact-semantic udp_delivery_ack_validated
-> next logical range sent and left unconfirmed at controlled UDP stop
-> TCP canonical negotiation
-> authenticated ResumeBinding / ResumeGuard
-> resend of unconfirmed logical range
-> receiver dedup/exactly-once application delivery when UDP copy arrived
-> encrypted + exact-semantic tcp_delivery_ack_validated for resumed records
-> ordered application completion
-> cleanup
```

Record actual application bytes and ciphertext lengths separately. Do not restore `udp_ack_observed`-style wording.

Classify this exactly as:

```text
real self-owned TCP/UDP sockets + controlled application endpoint stop
!= natural WAN degradation/PTO detection
!= automatic FailoverController threshold evidence
```

#### B3 — Small repeated real-socket lifecycle sample

If B1/B2 are green, run a bounded repeated open/exchange/close sample (for example 8-16 small cycles; keep total duration/traffic/concurrency comfortably below standing limits). Record:

- successful/failed cycles;
- application bytes/records;
- any duplicate/missing delivery observation supported by real state;
- final listeners/processes;
- cleanup result.

This is leak/resilience evidence, not capacity/stress evidence.

#### B4 — IPv6 opportunistic row only if the owned environment really has it

If both client and VPS have a currently usable self-owned IPv6 path and the generic/failover commands accept the address representation truthfully, repeat only the smallest B1 sanity row over IPv6. If the environment is absent or the CLI needs a local address-format fix, record the exact blocker and continue; do not let IPv6 stop B1-B3 IPv4/self-owned work.

**Evidence rule**

Commit small redacted summaries with experiment ids, timestamps, parameters, identities, results and cleanup. Do not commit private identity material, keys, unnecessary addresses, raw payloads, or large pcaps/logs.

### Follow-up C — Reusable process-resource sampler + one real use

**Dependency:** A green; can proceed if a B row is temporarily environment-blocked.

The VPS rental policy explicitly prioritizes CPU/RSS/FD/socket evidence, and the current repository still lacks a reusable process-scoped sampler suitable for both Nekomusume and HY2.

Build a bounded sampler under `scripts/bench/` (or reuse/extend an existing exact-purpose tool if one already exists after fresh inspection) with a small schema, validator/test, and documentation.

Minimum output per sampled role/process:

- experiment id, implementation, role;
- git/binary identity supplied by the caller;
- start/end/elapsed;
- exit status;
- CPU user/system time where available;
- max RSS or sampled RSS, with units and source clearly identified;
- FD count/peak or sampled count with method identified;
- owned experimental listener/socket count/peak without dumping unrelated connection details;
- application bytes supplied by workload metadata;
- cleanup state.

Requirements:

- finite sampling interval/count/duration;
- no root requirement if standard `/proc`, `ps`, `time`, `ss` or equivalent read-only interfaces suffice;
- never inspect unrelated process payloads or secrets;
- distinguish “not available” from zero;
- schema/validator rejects malformed/missing required metadata;
- deterministic local fixture/test for parsing/aggregation.

After local tests, use it once on a short Nekomusume VPS B1/B2-style run and commit only the small summary. Do not run CPU-heavy build/fuzz concurrently with the measured sample.

### Follow-up D — Unlock **automatic** UDP-degradation -> TCP failover evidence

**Dependency:** B2 green; do not conflate with the already-proven controlled-stop runner.

The bounded release evidence matrix specifically still lacks threshold-driven degradation/failover evidence. Inspect the existing carrier health / `CarrierState` / PTO or health-observation seams and design the smallest truthful integration into the real-socket runner.

**Goal**

Create a bounded experiment path in which the UDP carrier becomes observably unhealthy through the project's actual health/failure contract and the runtime transitions to TCP without the client simply executing an unconditional scripted stop.

Acceptable bounded fault injection on self-owned endpoints should stay application-level unless an already-authorized existing harness can do more without production qdisc/firewall changes. For example, a test server mode may stop replying to UDP after a deterministic point while remaining available on TCP, so the client must accumulate the real configured health/PTO/failure evidence before failover.

Requirements:

- use the existing health/failover state machine rather than inventing a second boolean timer;
- record the exact threshold/events that turn UDP Active -> Degraded/Failed (or the actual repository states);
- preserve Session delivery uncertainty/dedup semantics from the corrected runner;
- bound all retries/timeouts and pre-auth resources;
- add deterministic local/process tests before VPS execution;
- never call an explicit unconditional `controlled_udp_stop` and then claim automatic failover;
- do not modify VPS production route/firewall/qdisc.

If the existing carrier model is not yet connected enough for a bounded implementation slice, produce a precise implementation dependency map and implement the smallest missing seam rather than declaring the release evidence blocked generically.

Once locally green, a bounded self-owned VPS run is READY under standing authorization and should be prioritized in the next lab batch.

### Follow-up E — HY2 paired-comparison command seam, without weakening the existing fail-closed harness

**Dependency:** C sampler available or equivalent CPU/RSS/FD collection already exists; B basic network sanity green. D may continue independently if it is the more direct release blocker.

The repository already pins HY2 v2.9.3 and has `docs/bench/hy2-comparison-workload.md`, but `scripts/bench/compare-hy2.sh` is intentionally loopback-only and currently expects exact per-implementation commands. Do **not** simply remove its loopback guard.

Build the smallest separate self-owned-VPS comparison seam that can supply equal application semantics to the existing comparison contract while preserving standing-authorization safety. Prefer either:

- separate Nekomusume/HY2 workload wrapper commands that each perform one finite equal-payload exchange against fresh temporary high-port experimental listeners, then feed their JSON into a comparison driver; or
- a separate explicitly self-owned-VPS mode/harness with fail-closed ownership/authorization metadata, leaving the existing loopback-only mode intact.

For HY2:

- reuse the already pinned v2.9.3 binary/hash;
- use fresh temporary config/certificate/auth under an experimental temp path;
- do not read/reuse production HY2 secrets;
- do not restart/reconfigure the existing service;
- bind only the intended self-owned experimental endpoint/high port;
- trap cleanup and verify the temporary listener/process disappears.

For both implementations fix the same payload bytes/hash, server/client pair, route/time window, MTU metadata, security class and load. Produce raw paired samples, median/P95/failure count, CPU user/system, RSS, FD and application bytes. `wire_bytes` remains nullable unless trustworthy bounded capture metadata exists.

A first successful paired run is **comparison evidence only**. It is not a superiority claim; preserve slower/failing Nekomusume results exactly.

## Completion gates for this batch

- `f680702` authenticated ACK/control repair remains green under full local gate and GitHub CI or later equivalent CI.
- Old pre-fix ACK evidence is explicitly superseded without deleting history.
- `docs/status.md` and `docs/m3-wan-failover-gate.md` reflect current negotiation/standing-authorization boundaries.
- Selection-loss and Noise-response-loss retry claims are each either deterministically tested or explicitly limited; no cached-response code is promoted beyond its tested behavior.
- At least one post-fix standalone TCP and UDP self-owned VPS sanity row exists unless the environment itself is genuinely unavailable.
- A post-fix controlled failover run proves encrypted + semantic DeliveryAck validation on both carriers and preserves the controlled-stop/natural-degradation distinction.
- Repeated lifecycle evidence has bounded cycle/failure/cleanup metadata if B3 runs.
- Resource sampler output is process-scoped, bounded, schema-validated and actually exercised once when environment permits.
- Automatic degradation/failover remains a separate unchecked release-evidence row until the actual health threshold drives the transition.
- HY2 comparison remains semantically fair and isolated from the existing production service; no one-off superiority claim.
- `RELEASE_CANDIDATE=false`, global `FREEZE=false`, `PRODUCTION_READY=false`, `RELEASED=false` remain unchanged.

## Fallback

If A or B exposes a new production/session/crypto correctness defect:

1. preserve the minimal reproducer and negative evidence;
2. stop additional release/performance claims on the affected path;
3. make the correctness repair the next Primary;
4. run required parser/fuzz gates only if the changed surface requires them;
5. resume B only after the repaired exact commit is green.

If the VPS is temporarily unreachable or one address family is unavailable, do **not** idle and do not rerun unchanged failures. Continue C (resource tooling), D (automatic-failover seam) and E (paired-comparison wrappers) locally, then return to the VPS as soon as a truthful run is possible.

If the existing production HY2 service prevents a safely isolated temporary comparison listener without configuration changes outside standing authorization, stop only E's execution step and preserve the harness work; do not alter/restart the production service.

## Do not expand into

- reopening N9 or changing frozen canonical corpus bytes without a new versioned corpus process;
- calling the controlled endpoint stop natural WAN/PTO degradation;
- public/general reachability claims from self-owned endpoint runs;
- long-duration/high-volume/high-concurrency testing outside standing authorization;
- production route/firewall/DNS/proxy/tunnel/qdisc changes;
- third-party targets/scanning;
- 0-RTT, enabled FEC, striping/aggregation or exotic carriers without an observed-problem gate;
- performance superiority claims from a single or semantically unequal HY2 comparison.

## Questions requiring maintainer decision

none.
