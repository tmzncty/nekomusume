# ChatGPT reviewer handoff — R8 closed; R9 READY_LOCAL; source/test implementation must start now

## Reviewed repository truth

- Current default branch before this refresh: exact `a28cc7de5c4541ebb2013cb228675a9807121a12` (`docs(handoff): accept R8 LOW follow-up and sharpen R9 ownership seam`).
- There is **no new developer source/test commit after exact `0cb6aa2ddfbd8b218cfe01d98a2f1e934d96fd95`** and no open PR. The commits after `0cb6aa2` through `a28cc7d` are local-gate/review/handoff documentation only.
- R8 executable closure remains accepted at exact `8c9f5848988c1e3062c1492f7d87818e3c31ce4a`, with independent bounded re-review at reviewer commit `108d7c096638fa793b2f7e522bedea2d72bc7ae1` and local exact-tree provenance in `docs/local-gate-8c9f584-20260914.md`.
- R8 LOW 1/2 are landed, reachable and gated at exact source/test `0cb6aa2`, with provenance in `docs/local-gate-0cb6aa2-20260914.md`. The earlier identical `3795326` was local-only and remains historical/non-shared.
- Exact `0cb6aa2` clean-tree local gate: focused `neko-cli` process tests green (42 tests), workspace clippy with `-D warnings` green, `PYTHONDONTWRITEBYTECODE=1 bash scripts/check.sh` exit 0, `git diff --check` exit 0, clean initial/final tree, Linux x86_64, rustc 1.98.0.
- No WAN/VPS experiment occurred in this sequence.
- Governance remains unchanged: item 3 incomplete; item 4 incomplete; `RELEASE_CANDIDATE=false`; `PRODUCTION_READY=false`; `FREEZE=false`; `RELEASED=false`; pre-R9 `READY_LIVE: none`.

## Reviewer verdict on current state

R8 remains closed. No R8 BLOCKER/HIGH/MEDIUM is open. LOW 3 (`SessionRuntime.events` retained-log policy) and LOW 4 remain non-blocking and must not delay R9.

The absence of a new R9 commit is **not** evidence that R9 is blocked or exhausted. R9 has a concrete owner, existing process path, existing runtime, existing wire/crypto/session semantics and an explicit testable first slice. Under `AGENTS.md` §3.2 this is now implementation stagnation and must be corrected by starting source/test work, not by waiting for another reviewer cycle.

# STAGNATION OVERRIDE — the next shared commit must be R9 source/test work

One full coding/reviewer interval has elapsed after the prior sharpened R9 handoff without an R9 source/test commit or PR. There is no repository, CI, authorization, policy or architecture blocker on the first slice.

**Therefore:** do not create another proposal-only, reviewer-only, provenance-only or status-only commit before a source/test R9 slice lands. The next developer-owned shared commit should contain actual R9 implementation and focused process regression(s).

The first slice is already narrow enough to implement directly:

- primary owner: `crates/neko-cli/src/main.rs`;
- focused regression owner: `crates/neko-cli/tests/probe.rs`;
- `neko-cli` already depends on `neko-reliable` from R8, so do not add a second dependency seam or new crate;
- reuse `ReliableUdpRuntime`; do not fork/copy the recovery engine;
- reuse existing `failover_server` / `failover_client`; do not create a parallel server/client protocol;
- keep the historical failover behavior unchanged unless the explicit experimental R9 mode is selected.

## Exact implementation anchors for the first R9 slice

### Client seam

In `failover_client`, canonical negotiation and UDP Noise already finish before the current first application path:

`let mut us = hs.finish(...)` -> build `ProcessMessage::Data` -> `us.seal_unreliable(...)` -> `u.send_to(...)` -> `recv_udp_delivery_ack(...)`.

For the explicit R9 mode only, replace that **post-auth application transport seam** with the already-tested R8 reliable-UDP composition:

1. create one `ReliableUdpRuntime` for the active UDP recovery owner;
2. keep existing `SessionRuntime` / `OutboundRecord` stream+byte-offset identity;
3. build one authenticated Data record, use the authenticated outer AEAD sequence prefix as the Carrier packet number, and call `can_send` before committing Session byte offset/recovery ownership;
4. call `on_packet_sent` only after congestion admission and before/with the actual wire attempt according to the already-reviewed R8 ownership contract;
5. receive authenticated Carrier ACK records, decode canonical `AckPayload`, call `apply_ack`, and settle to `in_flight()==0` under a finite deadline;
6. preserve the existing **Session `DeliveryAck`** confirmation path separately. Carrier packet ACK must not stand in for Session delivery acknowledgement.

Preferred CLI shape is a bounded experimental `--reliable-udp` flag on the existing `failover --role client|server` path because it preserves the historical command and matches the existing R8 terminology. An equivalent smaller flag spelling is acceptable if current parsing makes that cleaner; do not create a new top-level protocol command.

### Server seam

In `failover_server`, reuse the already-authenticated UDP application branch under the existing `if let Some((ref mut ss, peer)) = secure` ownership.

For R9 mode only:

1. authenticate/open the datagram first;
2. only after authentication and structurally valid Data decode, derive/commit the authenticated outer AEAD sequence as the Carrier packet number;
3. feed the logical `(session, stream, byte offset, bytes)` into the existing `SessionRuntime::receive` — Session remains above Carrier;
4. record the received packet in the receiver ACK tracker / reliable runtime and emit canonical authenticated packet ACK feedback;
5. separately retain the existing Session `DeliveryAck` behavior for logical confirmation;
6. duplicate/reordered replacement handling must be Session-owned; Carrier must not gain a second `(stream, offset)` dedup map.

### First durable process regression

Land one separate-process no-loss test before adding fault injection. It must launch the existing failover server/client in R9 mode and prove:

- canonical negotiation + Noise/trust/authz complete before application admission;
- at least two bounded logical records are delivered exactly once;
- Carrier packet ACKs retire all reliable-UDP in-flight state;
- Session `DeliveryAck` remains a separate logical confirmation path;
- zero PTO, zero retransmit, zero conflict in the no-loss case;
- final `remaining_in_flight=0` (or equivalent authoritative runtime observation);
- process/listener cleanup and same-address rebind remain valid where the current test harness already supports them.

A coherent first commit does **not** need Data-loss, ACK-loss or real TCP promotion yet. Commit/push this no-loss process slice once focused tests are green, then immediately continue R9-3 onward without waiting for reviewer cadence.

If implementation discovers only a missing typed helper/API whose semantics are already fixed by R7/R8, add the smallest helper and continue. Stop only if the required change genuinely alters core Session/Carrier/ACK/crypto/wire semantics.

# Execution state — R9 is READY_LOCAL

External coding agent should synchronize to current `main` and execute continuously:

`inspect exact head -> implement coherent source/test slice -> focused tests -> commit/push -> next dependency-ready slice -> coherent clean exact-tree gate`.

Do not wait for reviewer cadence between R9 sub-slices. Reviewer cadence is inspection cadence, not a work quota.

# R9 ownership map — reuse the existing failover process seam

## Existing process owners

### `crates/neko-cli/src/main.rs::failover_server`

Already owns:

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
- historical controlled application-level UDP reply-cessation seam.

### `crates/neko-cli/src/main.rs::failover_client`

Already owns:

- UDP peer selection/socket;
- canonical negotiation + Noise binding;
- `SessionRuntime` outbound records;
- authenticated UDP application Data + Session `DeliveryAck` verification;
- automatic-health-failover / warm/cold modes;
- TCP connect, resume/readiness and post-promotion application flow;
- bounded duration/count/bytes and diagnostics.

### `ReliableUdpRuntime`

Keep packet number / ACK / RTT / loss / PTO / retransmission / Reno / pacing and plaintext-retransmit ownership Carrier-local. Current runtime already exposes `can_send`, `pacing_interval_us`, `on_packet_sent`, `on_packet_received`, `poll_outgoing_ack`, `apply_ack`, `pto_probe`, `on_retransmit_sent`, `in_flight`, recovery observability, manager access and teardown. Reuse these APIs before adding new ones.

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

Integrate genuinely new R9 evidence into existing observability only where useful. Do not build a second logging framework. Preserve packet-recovery, Carrier health/switch and Session delivery as separate evidence domains.

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
