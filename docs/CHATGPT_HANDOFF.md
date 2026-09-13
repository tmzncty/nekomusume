# ChatGPT reviewer handoff — Q4/R7 accepted; execute R8/R9 reliable-UDP path continuously

## Reviewed repository truth

- Exact current `main` at this review: `0959c6b513d05a8a9de4dc89241b4aec2f3d0a03` (`docs: record exact-tree gate for the honest Q4-F counter`).
- Latest source/test change: exact `360eea2d1966a9d2360aa3e6b3aeab2505f18ff7`, test-only. Production carrier source remains the already accepted `e0729d96f59988e7b918d21cfb19579c319b0733` repair line; no new product-semantic change landed after that line in this reviewed sequence.
- `360eea2` closes the sole LOW test-quality nit from the independent Q4/R7 review: the coherent Q4-F scenario now counts `non_degraded_samples` honestly and asserts exactly `1`; with only the historical send-time-denominator defect restored the same scenario reports `2` and fails. This makes the negative control explicit rather than relying only on the `0..2` loop bound.
- Developer-persisted exact-tree provenance for `360eea2` is recorded in `docs/local-gate-1a3535b-20260913.md`: clean detached tree, `scripts/check.sh` exit 0, `git diff --check` exit 0, Linux x86_64, rustc 1.98.0, clean initial/final tree. This is developer-reported/persisted local CI, not reviewer-executed local CI.
- GitHub-hosted checks on exact `0959c6b` are green: `stable checks` and `nightly decode fuzz smoke`. These are additional hosted cross-evidence only.
- Open pull requests: none. No VPS/WAN experiment occurred in this sequence.
- Governance remains unchanged: item 3 incomplete; item 4 incomplete; `RELEASE_CANDIDATE=false`; `PRODUCTION_READY=false`; `FREEZE=false`; `RELEASED=false`; `READY_LIVE: none` until the new executable/process path creates and passes a truthful live gate. D019, `SessionRuntime.events`, RSEC-001, signing/key-custody/SBOM, previous-frozen-release interop and final release authority remain separate policy/external gates.

## Reviewer verdict

### `360eea2` Q4-F test repair — ACCEPT

The previous independent review correctly identified the old `bad_intervals >= 1` assertion as inert and semantically inverted. The replacement assertion is now aligned with actual `CarrierHealth` semantics: one bad sample below `degrade_after=2` leaves the stored state unchanged, so the useful externally visible discriminator is the number of non-degraded samples before the second consecutive bad sample crosses the degradation threshold. The current scenario therefore expects exactly one non-degraded interval; restoring the old send-time denominator erases the first resolved-loss interval and yields two non-degraded intervals/no degradation.

No production behavior, wire, crypto, Session or Carrier semantic changed. Q4 remains CLOSED and R7 remains ACCEPTED for the bounded local scope. Do not reopen H-RUDP-011, H-RUDP-001D, H-RUDP-012, H-RUDP-014 or the Q4 suite absent a new source change or contradictory failure.

### Current architecture/status check

Repository truth still says:

- Session is above Carrier; UDP packet ACK/RTT/loss/PTO are Carrier-local and must never become Session delivery evidence.
- UDP is the primary Carrier and TCP is the first fallback; no concurrent UDP+TCP striping.
- `docs/status.md` still describes `reliable-udp` as a deterministic bounded recovery model with **no live service / Session-delivery promotion**.
- `IMPLEMENTATION_PLAN.md` item 3 remains open specifically because the accepted `25e0daa` WAN fallback is controlled application reply cessation, not packet-recovery/PTO-driven degradation.
- Standing authorization already allows bounded self-owned TCP/UDP reliable-datagram/failover/recovery experiments. Permission is not the blocker; an executable truthful path is.
- VPS-rental policy says local work that directly unlocks new VPS-only evidence outranks unrelated local hardening.

Therefore **R8 is the earliest genuine READY_LOCAL mainline task**. Queue exhaustion no longer applies while this executable integration is absent.

## R8 — bounded executable local reliable-UDP lab path — READY_LOCAL NOW

The coherent R7 composition currently lives only in `crates/neko-carrier/tests/reliable_udp_runtime.rs`. `neko-cli` already depends on `neko-carrier`, `neko-session`, `neko-crypto`, and `neko-wire`, and its existing `lab` command is explicitly fixture-maturity. The current `lab()` implementation is only a static/canned failover timeline; it is not evidence that the accepted reliable-UDP runtime is executable outside tests.

### Preferred smallest shape

Prefer extending the existing fixture surface with an explicit scenario such as:

```text
neko lab --scenario reliable-udp [bounded options] [--json]
```

Preserve the existing no-scenario `lab` behavior if compatibility tests depend on it. A separate fixture command is acceptable only if it is materially simpler and keeps the same bounded/non-production boundary. Do **not** turn this into a daemon, public listener, proxy or tunnel.

Implement the real scenario in a normal `neko-cli` module (for example a small `reliable_udp_lab` module) using public production APIs; do not import or copy a test module wholesale. Reuse the accepted semantics from the R7 fixture:

- real loopback UDP sockets through the Carrier abstraction (`UdpLoopbackPair` is already real `std::net::UdpSocket` I/O);
- existing mature crypto/Noise record APIs;
- `ReliableUdpRuntime` for packet number / ACK / RTT / loss / PTO / retransmit / cwnd/pacing / manager health;
- real `SessionRuntime` above Carrier for stream/byte-offset delivery, duplicate suppression and conflict refusal;
- packet number/AEAD nonce, Carrier `FrameId`, and Session `(stream, byte_offset)` remain distinct identities;
- ACK/loss/PTO evidence never calls or implies Session `DeliveryAck`.

A short developer proposal cycle may choose exact helper/API ownership. This is ordinary implementation selection under already committed semantics and is pre-authorized; do not stop for reviewer approval unless it would alter core Session/Carrier/ACK/crypto/wire semantics.

### R8-A executable clean path

Create one real bounded local executable path that completes at least a no-loss exchange using the accepted composition. Required truth:

- bounded count/bytes/time; no input-controlled unbounded loops or retained maps;
- real socket send/recv, authenticated packet, canonical packet ACK, Recovery application, Session receive;
- byte-accurate Session offsets and stable FrameId across a retransmit;
- cwnd admission happens before irreversible send/accounting state;
- cleanup closes sockets/runtime-owned state deterministically;
- `--json` emits only observed counters/events; no canned `"pto"`, `"migrated"`, `"recovered"` event may be printed unless that event actually occurred.

Minimum JSON should distinguish, without secrets/private endpoint material: scenario/version, records offered/delivered/duplicate/conflict, application bytes, UDP packets sent/acked/lost, PTO/retransmit counts, final Carrier/Session outcome, and cleanup result. Null/absent is preferable to a fabricated measurement.

### R8-B deterministic local fault modes

After the clean path works, add bounded local-only controlled suppression sufficient to demonstrate **real runtime behavior**, not a fake result record. Keep the fault seam outside protocol semantics; it may suppress a selected outgoing Data or ACK before socket send.

At minimum exercise:

1. one lost Data packet -> recovery schedules stable frame -> fresh packet/nonce replacement -> exactly one Session delivery;
2. one lost ACK -> PTO/replacement -> late original/duplicate remains one Session delivery;
3. replacement arrives before delayed original -> Session duplicate suppression, not Carrier-owned dedup;
4. tampered/unauthenticated packet/ACK produces no ACK/recovery/Session evidence;
5. cwnd/plaintext-owner refusal remains atomic and consumes no Session byte offset;
6. historical loss is not replayed as new fresh health evidence.

Do not claim natural network loss from these deterministic faults.

### R8-C CLI truth/compatibility

- Update `USAGE`/`capabilities` only to the exact executable surface that now exists; mark it `fixture`/`research`, not production.
- Invalid/oversized options fail before socket/identity side effects where practical.
- Human output and JSON exit semantics must remain deterministic and must not claim Session delivery from Carrier ACK.
- Preserve existing `lab`/other command output unless intentionally and testably extended.
- Add process/CLI tests that invoke the actual built binary for the new scenario, not just direct helper calls.

### R8-D exact-tree gate + independent challenge

On the final pushed R8 source/test SHA, developer must persist clean exact-tree provenance:

```bash
PYTHONDONTWRITEBYTECODE=1 bash scripts/check.sh
git diff --check
```

plus focused CLI/carrier tests and clippy. If wire decoder/framing grammar changes, run the pinned decode fuzz commands; otherwise do not mechanically fuzz merely because the CLI changed.

Then perform one bounded independent review of the executable R8 path: ownership/bounds, packet-vs-Session evidence, identity separation, observed-vs-fabricated JSON, cleanup, fault-seam truthfulness, and no production/public exposure. Concrete defects are repaired immediately; a clean review proceeds directly to R9.

## R9 — real process/socket acceptance — continue immediately after R8

R8 is same-process local executable evidence. R9 must cross an actual process boundary before any new live/WAN classification.

### R9-A connected UDP adapter if required

Current Carrier production adapter exposed here is the local paired `UdpLoopbackEndpoint`. If a process path needs a normal externally-bound socket adapter, add the **smallest carrier-local connected-UDP adapter** rather than duplicating reliable transport mechanics in CLI. Preferred constraints:

- CLI owns bind/address selection and explicit peer selection;
- Carrier wrapper accepts an already-created/connected `std::net::UdpSocket` plus bounded `UdpLimits`;
- message boundaries preserved; nonblocking/timeout semantics explicit;
- no arbitrary `send_to` proxy surface, no routing/firewall mutation, no path/session evidence promotion;
- close/error behavior follows the existing UDP carrier contract;
- deterministic loopback tests cover oversize, WouldBlock, close and peer/socket error boundaries.

If existing APIs can support the process path cleanly without this adapter, do not add one just for abstraction symmetry.

### R9-B authenticated server/client process path

Prefer reusing/extending the existing bounded `failover`/lab machinery instead of creating a second independent handshake/trust/readiness stack. Exact command spelling is developer-owned; behavior is not:

- canonical version negotiation before fresh Noise;
- trust/authz before application data;
- UDP Data packets go through `ReliableUdpRuntime`, not the old direct request/DeliveryAck-only path;
- receiver Session ownership remains in `SessionRuntime`;
- packet ACK is authenticated Carrier feedback only;
- finite setup/application/PTO/overall deadlines;
- fixed bounded count/bytes and deterministic cleanup;
- diagnostic JSON is secret-safe and uses observed events/counters.

### R9-C process negative/repair matrix

At minimum prove across separate processes:

- clean authenticated reliable-UDP multi-record exchange;
- controlled one-Data suppression recovers with one logical delivery;
- controlled ACK suppression causes PTO/replacement without promoting packet ACK to Session DeliveryAck;
- malformed/tampered/future ACK cannot mutate recovery/Session state;
- shutdown/timeout releases listener/process/temp state;
- no duplicate application delivery when replacement and original both arrive.

### R9-D packet-recovery-driven warm fallback

Only after the process reliable-UDP path is green, connect its **fresh health evidence** to the already accepted warm TCP manager/readiness path. The new question is specifically:

> Can authenticated packet-level loss/PTO/recovery evidence on the UDP Carrier produce truthful degradation and then a warm TCP promotion while preserving the same logical Session/uncertain-byte semantics?

Do not reuse application-level “reply cessation” as if it answered this. Warm TCP still requires the existing authenticated readiness/resource-admission proof and carries no application data before atomic promotion. Failure to switch must be reported as failure; no fabricated success event.

Required controlled process cases:

1. recoverable UDP loss that does **not** cross degradation hysteresis -> remain UDP active;
2. distinct fresh bad resolved outcomes that cross hysteresis -> real `WarmFallback` only if standby is actually valid/ready;
3. invalid/unready standby -> `FallbackFailed`, never switch-success;
4. uncertain Session bytes across promotion -> replay/dedup by Session byte identity, no guessed delivery;
5. packet recovery counters remain separate from Session confirmed watermark.

Persist exact-tree local provenance and one bounded independent R9 review before any WAN run.

## Q10 — observability/evidence integration

Once R9 behavior exists, wire existing bounded observability to the executable/process path rather than inventing another log framework. Required distinctions:

- recovery RTT update;
- newly resolved packet loss;
- PTO/probe scheduling;
- frame retransmission;
- Carrier health transition;
- actual switch success/failure;
- Session duplicate/conflict/confirmed state when the Session layer itself produces that evidence.

Never infer Session delivery from packet ACK or switch success. Do not log plaintext payload, secret key material, unnecessary endpoint identity, or private topology. Existing `SessionRuntime.events` retention remains a separate policy-blocked release issue; do not solve it by inventing a new retention number inside this lane.

## Q11 — status/release reconciliation and READY_LIVE decision

After R8/R9 + independent review:

- update `docs/status.md` reliable-UDP/live-UDP/CLI rows only to the exact new bounded evidence;
- update the release packet and implementation plan without rewriting historical negatives;
- retain item 3 and item 4 unchecked unless their own full criteria are genuinely met;
- classify a concrete `READY_LIVE` row **only if** the new process path is executable, independently bounded-reviewed, has green exact-tree gate, and asks a materially new real-network question.

A likely truthful first live question, if R9 passes, is:

> bounded self-owned IPv4 path with controlled packet-level Data/ACK suppression through the real reliable-UDP engine, measuring recovery/PTO/fresh-health behavior and, if the threshold is deliberately crossed, same-Session warm TCP fallback.

This would be a changed implementation/instrumentation/hypothesis relative to historical `25e0daa`, so it can create a new live lane. It still would **not** be “natural Internet loss” evidence unless loss actually arises naturally and is observed as such.

## Q12 — VPS opportunity once Q11 yields READY_LIVE

Standing authorization already covers this class of ordinary bounded self-owned TCP/UDP experiment. Use the smallest profile that answers the question, e.g. a few small records and a run well below normal standing limits; do not modify production qdisc/route/firewall.

Evidence must bind experiment id, exact binary/commit identity, parameters/fault mode, timestamps, client/server structured results, recovery/PTO/switch events, application records/bytes, duplicates/conflicts/missing delivery, and cleanup. Preserve a negative result exactly. Do not rerun the same failure class without a materially changed hypothesis/instrumentation/config/path condition.

## Deep rolling queue — do not stop after one slice

Execute in dependency order without waiting for reviewer cadence while the next item remains READY:

1. R8-A real local executable clean reliable-UDP lab scenario.
2. R8-B controlled Data-loss / ACK-loss / replacement-order scenarios.
3. R8-C CLI JSON/human/argument/process truth tests.
4. R8-D clean exact-tree gate + provenance.
5. Independent bounded R8 challenge; smallest repair if needed.
6. R9-A minimal connected-socket carrier seam if actually required.
7. R9-B authenticated cross-process reliable-UDP server/client path.
8. R9-C cross-process clean/loss/ACK-loss/tamper/cleanup matrix.
9. R9-D fresh-recovery-health -> warm TCP fallback integration.
10. Exact-tree gate + bounded independent R9 challenge.
11. Q10 bounded structured observability on the real executable/process path.
12. Q11 status/release/item-3 factual reconciliation and concrete `READY_LIVE` classification if earned.
13. Q12 one minimal self-owned VPS changed-hypothesis run if READY_LIVE.
14. Reconcile the retained VPS result, then refill the queue from the newly observed defect/evidence gap rather than returning to generic audit churn.

If the agent can complete several of these in 10–30 minute slices with clean gates, keep going. Reviewer cadence is not a work quota.

## Current non-blocking/policy gates

These remain real but do not block R8/R9 engineering:

- `SessionRuntime.events` retained-state policy (`POLICY_BLOCKED_RESOURCE_BOUND`);
- D019 terminal source-retention/no-reset semantics;
- RSEC-001 representative adversarial-load/capacity suitability;
- signing/key custody/SBOM/publication trust;
- previous-frozen-release interoperability;
- final independent security/release judgment and RC/freeze/release/production authority;
- IPv6 environment absence and frozen historical HY2/repeated-failover/periodic lines.

Do not invent TTL/LRU/history/capacity/security values while working around them.

## Stop conditions

Stop only for a newly discovered unresolved BLOCKER/HIGH that cannot be mechanically repaired under committed semantics, a genuine new core Session/Carrier/ACK/crypto/wire architecture choice, destructive/canonical migration, production/third-party/new-credential need, action outside standing authorization, or actual runtime/tool exhaustion.

A normal implementation choice such as module shape, helper ownership, local fixture command spelling, or connected-socket wrapper API is not by itself a maintainer stop condition; use the repository proposal loop, choose the smallest fail-closed shape, implement, test, push, and continue.
