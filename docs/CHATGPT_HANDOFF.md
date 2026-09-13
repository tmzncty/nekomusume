# ChatGPT reviewer handoff — R8 closed; execute R9 continuously on the existing failover process seam

## Reviewed repository truth

- Current default branch before this refresh: exact `0a710d955f3a59d5f8d662a82601e4c97d10c0eb` (`docs: record exact-tree gate for R8 LOW follow-up repair`).
- R8 executable closure remains accepted at exact `8c9f5848988c1e3062c1492f7d87818e3c31ce4a`, with independent bounded re-review at reviewer commit `108d7c096638fa793b2f7e522bedea2d72bc7ae1` and local exact-tree provenance in `docs/local-gate-8c9f584-20260914.md`.
- R8 LOW 1/2 are now **landed, reachable and gated** at exact source/test `0cb6aa2ddfbd8b218cfe01d98a2f1e934d96fd95`, with provenance in `docs/local-gate-0cb6aa2-20260914.md`. The earlier identical `3795326` was local-only and must remain historical/non-shared; use `0cb6aa2` as the landed evidence anchor.
- Exact `0cb6aa2` adds the built-binary forced partial-settlement regression and binds final `settled` to both the lab ownership map and authoritative `ReliableUdpRuntime::in_flight()`. Forced incomplete settlement is pinned to `ok=false`, `settled=false`, `partial=true`, exit `1`.
- Exact `0cb6aa2` clean-tree local gate: focused `neko-cli` process tests green (42 tests), workspace clippy with `-D warnings` green, `PYTHONDONTWRITEBYTECODE=1 bash scripts/check.sh` exit 0, `git diff --check` exit 0, clean initial/final tree, Linux x86_64, rustc 1.98.0.
- Current exact `0a710d9` GitHub-hosted `stable checks` and `nightly decode fuzz smoke` are green; hosted checks remain extra cross-evidence only.
- Open PRs at this review: none.
- No new WAN/VPS experiment occurred in this sequence.
- Governance remains unchanged: item 3 incomplete; item 4 incomplete; `RELEASE_CANDIDATE=false`; `PRODUCTION_READY=false`; `FREEZE=false`; `RELEASED=false`; pre-R9 `READY_LIVE: none`.

## Reviewer verdict on `0cb6aa2` — ACCEPT

No BLOCKER/HIGH/MEDIUM was introduced by the R8 LOW follow-up.

- The forced partial path is now a durable built-binary regression rather than an ad-hoc observation.
- `settled` no longer trusts only lab-local bookkeeping; runtime in-flight state participates in the verdict.
- The change does not alter wire, crypto, Carrier, Session, recovery or dependency semantics.
- R8 remains closed. Do not create another R8 acceptance cycle unless a concrete regression appears.

LOW 3 remains deliberately deferred: `SessionRuntime.events` is an unbounded retained `Vec`, so reading event history for duplicate accounting would couple R8/R9 truth to the already-open retention-policy gap. The current exact inference does not require that unbounded history. LOW 4 remains inapplicable while all R8 frame IDs are offset-derived. Neither LOW may block R9.

# Execution state — R9 is READY_LOCAL now

The previous hour did not land an R9 source/test slice; only the non-blocking R8 LOW follow-up landed. This must **not** turn into idle waiting. `AGENTS.md` explicitly treats a dependency-ready lane with no real blocker as work that should continue, and current repository truth contains no R9 BLOCKER/HIGH.

External coding agent should synchronize to current `main` and execute the R9 queue continuously:

`inspect exact head -> implement coherent slice -> focused tests -> commit/push -> continue next dependency-ready slice -> final clean exact-tree gate at coherent boundary`.

Do not wait for reviewer cadence between R9 sub-slices. Preserve the downstream queue. Reviewer cadence is inspection cadence, not a work quota.

# R9 ownership map — do not start from a blank design

R9 should reuse the **existing failover process path**, not invent a second protocol or a parallel process harness.

## Existing process owners to reuse

### `crates/neko-cli/src/main.rs::failover_server`

Already owns, in one bounded process path:

- UDP listener + TCP listener;
- `--udp-bind` / `--tcp-bind` and bounded operator port range;
- pre-auth `ListenerAdmission` for UDP/TCP;
- canonical version negotiation;
- Noise/trust/authz;
- `SessionRuntime` with stable stream/byte-offset delivery semantics;
- UDP authenticated Data and Session `DeliveryAck` handling;
- actual TCP accept/auth/readiness/resource-admission path;
- warm/resume semantics and actual TCP application Data after promotion;
- bounded application/experiment deadlines;
- diagnostics and cleanup behavior;
- existing controlled application-level UDP reply-cessation seam.

### `crates/neko-cli/src/main.rs::failover_client`

Already owns:

- UDP peer selection/socket;
- canonical negotiation + Noise binding;
- `SessionRuntime` outbound records;
- authenticated UDP application Data + Session `DeliveryAck` verification;
- automatic-health-failover / warm/cold modes;
- TCP connect, resume/readiness and post-promotion application flow;
- bounded duration/count/bytes and diagnostic output.

### Plain `server` / `client`

These are useful references for the smallest ordinary UDP negotiation/Noise/socket lifecycle, but R9 should **not** fork another independent reliable-UDP protocol path from them if the failover path can host the integration. The R9 goal includes real TCP standby/promotion; `failover_server`/`failover_client` already own that composition.

### `ReliableUdpRuntime`

Keep packet number / ACK / RTT / loss / PTO / retransmission / Reno / pacing and plaintext-retransmit ownership Carrier-local. Do not move Session `(stream, offset)` dedup or logical confirmation into Carrier.

## R9-1 concrete first implementation slice

**Do this now; no docs-only proposal is required.**

1. Add the smallest explicit R9 test/CLI switch on the existing failover command path so the historical application-level failover behavior stays unchanged unless selected. Prefer a bounded experimental flag/mode rather than a new top-level protocol command.
2. Instantiate one `ReliableUdpRuntime` per active UDP recovery owner on client and the corresponding bounded receive/ACK state on server as required by the already-committed R7/R8 semantics.
3. Replace only the selected R9-mode post-auth UDP application transport with the R8-tested reliable-UDP Data/ACK composition. Negotiation, Noise, trust/authz, bind/peer ownership and SessionRuntime stay where they already are.
4. Keep Session delivery above Carrier: server passes authenticated decoded logical Data into existing `SessionRuntime::receive`; packet ACK is emitted only as Carrier feedback and must never call/stand in for Session confirmation.
5. Reuse current `ProcessMessage::Data` / Session byte-offset identity rather than inventing a second application identity.
6. Give setup/application/PTO/settlement work finite existing or test-only bounds. Do not introduce new production security/capacity constants.
7. First focused test is **separate-process no-loss R9**. It must prove exact once application delivery, ACK retirement to zero in-flight, zero retransmit/PTO/conflict, and cleanup/rebind. This is the first source/test commit; do not wait to implement Data-loss before committing if the no-loss slice is coherent and green.
8. After that commit/push, immediately continue R9-3 onward below. Do not stop merely because R9-2 has landed.

If integrating `ReliableUdpRuntime` into `failover_*` exposes a missing API but the semantic answer is already fixed by R7/R8, use the smallest typed API/helper and continue. Stop only if the required change genuinely alters core Session/Carrier/ACK/crypto/wire semantics.

# R9 rolling queue — preserve all slices

## R9-2 — authenticated cross-process clean reliable-UDP path

Acceptance:

- canonical negotiation before fresh Noise;
- trust/authz before Data admission;
- UDP Data through `ReliableUdpRuntime`;
- logical receive/dedup through bounded `SessionRuntime`;
- authenticated packet ACK is Carrier feedback only;
- stable `FrameId` across fresh retransmission packet numbers/nonces;
- finite setup/application/PTO/settlement/overall deadlines;
- bounded count/bytes, cleanup and secret-safe JSON;
- no-loss process test: all offered/admitted records delivered once, ACK retires all in-flight, zero conflict/PTO/retransmit, cleanup verified.

## R9-3 — process Data-loss recovery

Add bounded deterministic post-admission suppression of one or periodic Data packet.

Prove suppressed packet was congestion-admitted but not wire-sent; PTO fires only after deadline; retransmission uses a fresh packet number/nonce and stable frame identity; Session byte identity is unchanged and delivered once; final recovery drains to zero or reports explicit partial/failure. Packet feedback must not promote Session delivery state.

## R9-4 — ACK-loss + reorder/delayed-original

Separate cases:

1. authenticated ACK emitted then suppressed;
2. replacement arrives before delayed original;
3. late original follows replacement.

Require one logical Session delivery, Session-owned duplicate suppression, no conflict, fresh Carrier packet numbers and deadline-driven PTO.

## R9-5 — tamper / future ACK / malformed feedback negatives

Prove:

- unauthenticated/tampered Data -> no ACK obligation, no recovery receive promotion, no Session receive;
- tampered ACK -> no recovery mutation;
- future/never-sent ACK -> typed rejection + atomic state;
- stale/duplicate ACK does not fabricate RTT/loss/Session evidence;
- malformed packet/ACK bounds cannot panic or allocate unboundedly.

If decoder/parser/framing grammar changes, run the pinned decode fuzz commands required by policy. Otherwise do not mechanically add fuzz churn.

## R9-6 — pacing/cwnd/plaintext-owner atomicity

Using test-only bounds, prove initial and retransmit sends consult congestion admission; refusal consumes no Session byte space and no committed packet/recovery/plaintext ownership; plaintext owner capacity/conflict/oversize remains typed; pacing deadline is finite/observable; teardown releases all ownership.

Do not invent production capacity/security values.

## R9-7 — truthful process observability/result contract

Machine-readable result must distinguish at least:

- initial offered / admitted / wire-sent / suppressed;
- ACK emitted / wire-sent / suppressed / applied / rejected / failed;
- PTO due / fired;
- retransmit attempted / admitted / wire-sent / refused;
- newly resolved acked/lost + remaining in-flight;
- Session first delivery / duplicate / conflict / application bytes / confirmation boundary when applicable;
- cleanup + final outcome.

No payload/key/private-topology logging. Counters increment after the represented action succeeds.

## R9-8 — actual authenticated warm TCP standby

Only after R9-2..7 are green, reuse the existing failover TCP path. Before declaring warm/ready require actual connect/accept, canonical negotiation, fresh Noise trust/authz, resume/readiness tuple+generation+delivery-epoch binding and resource admission. No new application Data on standby before atomic promotion.

R8 manager-only readiness is not transport readiness evidence.

## R9-9 — recovery health -> hysteresis -> real TCP promotion

Feed fresh resolved UDP outcomes into Carrier health/manager and prove:

1. recoverable loss below hysteresis stays UDP;
2. PTO-only sample cannot erase/invent later resolved loss;
3. distinct fresh bad resolved intervals cross committed hysteresis and promote to the actually ready TCP standby;
4. invalid/unready standby -> `FallbackFailed` only;
5. historical old loss is not replayed as new bad evidence.

Switch evidence must identify actual active transport ownership.

## R9-10 — uncertain Session bytes across UDP -> TCP

At promotion, drive at least one logical range with uncertain UDP delivery. Draining UDP accepts no new application Data. Replay that existing Session range over promoted TCP using current Session semantics. Receiver deduplicates by `(session, stream, byte offset)` and delivers application bytes exactly once. Carrier packet ACK remains distinct from Session delivery acknowledgement; do not add TCP packet ACK.

## R9-11 — shutdown / timeout / cleanup matrix

Bounded tests for setup timeout, application/PTO/settlement timeout, controlled stop, failed standby promotion, malformed/tampered input, listener/socket/process cleanup and same-address rebind where current commands support it. Do not expand into service-manager work.

## R9-12 — coherent exact-tree gate + independent bounded review

After R9-2..11 reach a coherent boundary, run on the **final pushed developer SHA** in a clean checkout/worktree:

```bash
PYTHONDONTWRITEBYTECODE=1 bash scripts/check.sh
git diff --check
```

Record exact SHA, UTC start/end, OS/arch, rustc, exit codes, clean initial/final tree. Hosted CI is extra cross-evidence only.

Independent R9 review must challenge:

- Session-above-Carrier layering;
- authenticated packet-number/ACK identity;
- bounded plaintext/recovery ownership;
- deadline-driven PTO/pacing/cwnd;
- fresh resolved health + hysteresis;
- real warm-TCP readiness/promotion;
- uncertain Session replay/dedup;
- result/counter truthfulness;
- cleanup/no public exposure.

BLOCKER/HIGH -> smallest repair + regression + re-gate + continue. LOW/NOTE does not halt progression.

# Q10/Q11/Q12 — pre-authorized continuation after R9

## Q10 — observability reconciliation

Integrate any genuinely new R9 evidence into existing observability only where useful. Do not build a second logging framework. Preserve packet-recovery, Carrier health/switch and Session delivery as separate evidence domains.

## Q11 — status/release evidence + READY_LIVE decision

Update `docs/status.md`, `IMPLEMENTATION_PLAN.md`, `ROADMAP.md` where status actually changes, and index only earned evidence into `docs/release-security-review-packet.md`.

If R9 cross-process reliable UDP + real TCP promotion is green and independently reviewed, create a **new specific READY_LIVE row** for that changed implementation/hypothesis. Do not reopen frozen historical HY2/periodic/repeated-failover lines.

Likely first live question:

> Does the exact reviewed R9 binary preserve authenticated reliable-UDP recovery accounting and Session exactly-once delivery across a real self-owned client<->VPS path under bounded program-controlled post-admission Data/ACK suppression, and when committed health hysteresis is crossed, does it promote only to an actually authenticated/ready TCP standby with truthful cleanup?

This is controlled cross-host integration evidence, not natural-loss prevalence, production capacity, public reachability or performance superiority.

## Q12 — one minimal changed-hypothesis self-owned VPS run

Only after Q11 marks the exact R9 question `READY_LIVE`, run one bounded self-owned client<->VPS experiment under standing authorization:

- temporary unprivileged TCP/UDP listeners;
- smallest count/bytes/duration sufficient for the question;
- exact binary/commit and actual parameters;
- authenticated Data/ACK/recovery/Session/switch evidence;
- bounded process/socket/resource observations when already available;
- cleanup verification;
- preserve negative result exactly; no unchanged same-class retry.

No production route/firewall/DNS/proxy/tunnel/qdisc modification and no third-party targets.

# VPS opportunity classification

**Current:** not yet `READY_LIVE` because R9 cross-process implementation and independent R9 review have not landed. This is an **implementation dependency**, not a WAN-permission blocker.

Standing authorization already covers the future bounded self-owned TCP/UDP experiment. Once R9/Q11 makes the changed question READY, do not ask again for ordinary WAN authorization.

# Non-blocking policy/authority gates

These do not stall R9/Q10 local correctness work:

- `SessionRuntime.events` retention (`POLICY_BLOCKED_RESOURCE_BOUND`);
- D019 source-retention/no-reset policy;
- RSEC-001 representative adversarial-load suitability;
- signing/key custody/SBOM/publication trust;
- previous-frozen-release interoperability;
- final security/release/RC/freeze/production authority;
- IPv6 environment and frozen historical HY2/repeated-failover/periodic lines.

Do not invent TTL/LRU/history/capacity/security policy values in this lane.

# Stop conditions

Pause the mainline only for:

- unresolved BLOCKER/HIGH that cannot be mechanically repaired under current committed semantics;
- genuinely new core Session/Carrier/ACK/crypto/wire architecture choice;
- destructive/canonical-meaning migration;
- production/third-party/new-credential requirement;
- action outside standing authorization;
- maintainer-only policy/value decision actually on the critical path;
- repository/tool failure.

Otherwise: implement -> test -> commit/push -> next READY slice continuously.