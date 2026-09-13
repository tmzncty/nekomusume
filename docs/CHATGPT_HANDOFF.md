# ChatGPT reviewer handoff — R8 ACCEPTED; continue directly into R9 cross-process reliable-UDP + real warm-TCP promotion

## Reviewed repository truth

- Current source/test closure reviewed: exact `8c9f5848988c1e3062c1492f7d87818e3c31ce4a` (`fix(cli): repair R8 executable reliable-UDP lab correctness and evidence`).
- Developer-local exact-tree provenance is retained in `docs/local-gate-8c9f584-20260914.md`: clean isolated exact tree, `PYTHONDONTWRITEBYTECODE=1 bash scripts/check.sh` exit 0, `git diff --check` exit 0, clean initial/final tree, Linux x86_64, rustc 1.98.0, focused lab tests green.
- Independent bounded re-review is reachable in `docs/reviews/independent-r8-executable-repair-8c9f584-20260914.md` at reviewer commit `108d7c096638fa793b2f7e522bedea2d72bc7ae1`. It independently verified all prior R8 BLOCKER/HIGH/MEDIUM findings as closed and found no new BLOCKER/HIGH.
- The current default branch before this handoff refresh is exact `108d7c096638fa793b2f7e522bedea2d72bc7ae1`; commits after `8c9f584` through `108d7c0` are provenance/review documentation only.
- No WAN/VPS run is claimed by the R8 closure. Open PR state must be re-read before work; repository truth wins over this note if a developer commit lands concurrently.
- Governance is unchanged: item 3 incomplete; item 4 incomplete; `RELEASE_CANDIDATE=false`; `PRODUCTION_READY=false`; `FREEZE=false`; `RELEASED=false`; current pre-R9 `READY_LIVE: none`.

## Reviewer verdict — R8 ACCEPT

The prior R8 gate is closed. The executable `neko lab --scenario reliable-udp` path now provides a bounded, truthful loopback composition of real UDP sockets + authenticated `SecureSession` + `ReliableUdpRuntime` + `SessionRuntime` above Carrier.

Verified properties include:

- optional `--scenario` parsing preserves legacy `neko lab` / `lab --json` behavior and fails malformed/unknown/duplicate scenario input before socket side effects;
- authenticated outer AEAD sequence is the Carrier recovery packet number; ACK applies to that exact packet-number space only after authentication and valid Data decode;
- cwnd admission precedes controlled suppression; refused sends consume no Session byte offset/recovery ownership;
- PTO is deadline-driven on a bounded logical clock rather than fired on every poll;
- success counters increment only after the corresponding operation succeeds and negative outcomes are retained separately;
- finite settlement prevents `ok=true` with unfinished recovery; forced incomplete settlement returns `partial=true`, `ok=false`, exit 1;
- clean, controlled Data-loss and controlled ACK-loss scenarios converge with one Session delivery per logical record and `remaining_in_flight=0`;
- manager warm-fallback output is explicitly scoped as manager-decision-only: R8 opens no TCP socket and makes no transport-migration claim.

The independent re-review's four LOW observations are **non-blocking and must not delay R9**. They may be folded into nearby tests opportunistically, especially one built-binary regression for the forced partial-settlement failure path, but do not create a new acceptance round unless a concrete regression appears.

**LOW follow-up status (independent verification, 2026-09-14):** LOW 1 and LOW 2 are already repaired and verified at exact `3795326605198dba81b54a781a85d2703ce7a546` — the forced partial-settlement failure path is now pinned by a built-binary test (`ok=false`/`settled=false`/`partial=true`/exit `1`, and `"ok":true` asserted absent), and the settlement verdict is bound to the runtime's authoritative `in_flight()` view as well as the lab's own map, with all four measured configurations unchanged. See `docs/reviews/independent-r8-low-followup-3795326-20260914.md`. **Provenance: `3795326` was local-only at review time (not on `origin/main`, no remote branch carrying it) and must be pushed before it is cited as landed-on-main evidence.** LOW 3 was deliberately not changed and that decision is correct, though its stated reason was inverted: the `SessionRuntime.events` buffer is in fact **unbounded** (a plain `Vec` that is only ever appended — no truncation, no limit field), so the feared "bounded buffer silently undercounts" risk does not exist; but reading it would couple duplicate accounting to unbounded retained state whose policy is the already-open `SessionRuntime.events` gate. The current inference is exact and should stay; the durable fix belongs to that gate. LOW 4 needs no action (every lab frame is offset-derived).

## High-throughput execution rule

R8 acceptance is a continuation point, not an idle point. External coding agent should proceed through the R9 queue continuously:

`inspect current exact head -> coherent slice -> focused tests -> commit/push -> clean exact pushed-tree gate as required -> next dependency-ready slice`.

Do not wait for the next reviewer merely because one R9 sub-slice landed. Preserve all still-valid downstream slices. If a source/test commit lands while reviewer documentation is being written, repository source/test truth outranks this file and the next reviewer must reconcile rather than overwrite developer work.

Only a real BLOCKER/HIGH, new core architecture choice, destructive migration, policy/value decision, out-of-authorization action, production/third-party/new-credential need, or repository/tool failure may stop the mainline.

# R9 rolling queue — cross-process reliable UDP to real warm TCP promotion

Maintain the full queue below; do not collapse it to one ticket. It is intentionally deep enough for a fast agent to keep moving.

## R9-1 — choose/reuse the smallest existing process ownership seam

**Goal:** move R8 semantics from one-process loopback fixture into existing separate-process CLI machinery without inventing a second protocol/runtime.

- Re-read existing ordinary UDP, failover/resume, periodic and process-test command paths before adding code.
- Prefer reusing current bind/peer parsing, negotiation, Noise/trust/authz, identity loading, setup/application deadlines, signal/cleanup and JSON conventions.
- Add a carrier-local connected-UDP adapter only if the process path genuinely cannot reuse the existing socket API cleanly.
- CLI owns address/bind/peer selection; Carrier must not become an arbitrary `send_to` proxy.
- Do not change Session/Carrier/ACK/crypto/wire architecture. If a genuinely new wire semantic is required, STOP that sub-slice and continue independent work while escalating it.

**Output:** smallest composition plan in code/tests, preferably immediately implemented rather than a docs-only proposal.

## R9-2 — authenticated cross-process clean reliable-UDP path

Build one bounded server/client path with:

- canonical negotiation before fresh Noise;
- trust/authz before application Data admission;
- UDP Data carried through `ReliableUdpRuntime`;
- Session delivery/dedup through bounded `SessionRuntime`, never Carrier-owned Session maps;
- packet ACK as authenticated Carrier feedback only;
- stable `FrameId` across fresh retransmission packet numbers/nonces;
- finite setup, application, PTO, settlement and overall deadlines;
- bounded count/bytes and deterministic cleanup;
- secret-safe observed JSON.

First acceptance scenario is no-loss separate-process exchange: all offered/admitted records delivered exactly once, ACK retires in-flight, zero conflict, zero PTO/retransmit, zero remaining in-flight, cleanup verified.

## R9-3 — process Data-loss recovery

Add deterministic sender-side post-admission suppression of exactly one or bounded periodic Data packet.

Must prove:

- suppressed packet was cwnd/pacing-admitted but not counted as wire-sent;
- deadline-driven PTO/recovery sends a fresh packet number/nonce for the same stable frame;
- Session byte identity is stable and delivered exactly once;
- recovery drains to zero or returns explicit bounded failure/partial;
- no packet feedback promotes Session confirmed state.

This is controlled fault injection, not natural WAN-loss evidence.

## R9-4 — process ACK-loss and delayed-original/reorder cases

Add separate scenarios for:

1. one authenticated ACK suppressed after emission;
2. replacement arriving before a delayed original;
3. late original after replacement.

Required evidence: due PTO only after deadline; duplicate suppression is Session-owned; conflict remains fail-closed; one logical Session delivery; fresh packet numbers remain Carrier-only.

## R9-5 — process tamper / future ACK / malformed feedback negatives

Prove fail-closed behavior for at least:

- unauthenticated/tampered Data -> no ACK obligation, no recovery receive promotion, no Session receive;
- tampered ACK -> no recovery mutation;
- future/never-sent ACK -> typed rejection and atomic recovery state;
- stale/duplicate ACK does not fabricate RTT/loss/Session evidence;
- malformed packet/ACK bounds do not panic or allocate unboundedly.

If this changes decoder/parser/framing grammar, run the pinned decode fuzz smoke required by repository policy; otherwise do not mechanically rerun fuzz.

## R9-6 — process pacing/cwnd/plaintext-owner atomicity

Exercise the real process composition under deliberately small existing test-only bounds without inventing production policy values:

- initial and retransmit sends consult congestion admission;
- refusal consumes no Session byte space, packet number if avoidable, recovery sent-state or plaintext ownership;
- bounded retransmit owner capacity/conflict/oversize errors remain typed;
- pacing deadline is observable and finite;
- teardown releases packet/frame/plaintext ownership deterministically.

Use fixture/test knobs, not new security/capacity policy constants.

## R9-7 — truthful process observability and result contract

Reuse existing bounded observability/event vocabulary where practical. Machine output must distinguish at minimum:

- initial offered/admitted/wire-sent/suppressed;
- ACK emitted/wire-sent/suppressed/applied/rejected/failed;
- PTO due/fired;
- retransmit attempts/admitted/wire-sent/refused;
- newly resolved acked/lost and remaining in-flight;
- Session first delivery/duplicate/conflict/application bytes/confirmed boundary when applicable;
- cleanup and final outcome.

No plaintext payload, PSK/private key, private topology, or fabricated success counter. Human output may stay compact; JSON/exit code is authoritative.

## R9-8 — real authenticated warm TCP standby in the same process scenario

Only after R9-2..7 are green, connect the already committed warm-TCP/failover semantics to an **actual TCP socket path** in this R9 command/test flow.

Required before calling standby `ready/warm`:

- TCP connect/accept exists;
- canonical negotiation completed;
- fresh Noise authentication/trust/authz completed;
- resume/readiness tuple/generation/delivery-epoch binding passed;
- resource admission passed;
- no new application Data sent on standby before atomic promotion.

Do not reuse R8 manager-only readiness as evidence of real TCP transport readiness.

## R9-9 — recovery-health -> manager hysteresis -> real TCP promotion

Feed **fresh resolved UDP recovery outcomes** into Carrier health/manager and prove separate cases:

1. clean/recoverable loss below hysteresis -> remain UDP;
2. PTO-only sample cannot erase or invent later resolved loss;
3. distinct fresh bad resolved intervals crossing committed hysteresis -> switch to the actually ready TCP standby;
4. invalid/unready standby -> `FallbackFailed`, never success;
5. old historical loss is not replayed as fresh bad evidence.

The switch event must identify real active transport ownership, not merely a model bit.

## R9-10 — uncertain Session bytes across actual UDP -> TCP promotion

Drive at least one bounded case with Data whose UDP delivery is uncertain at promotion.

- draining UDP takes no new application Data;
- uncertain logical ranges are replayed over promoted TCP according to existing Session semantics;
- receiver deduplicates by `(session, stream, byte offset)`;
- application bytes are delivered exactly once;
- packet ACK/recovery counters remain distinct from Session confirmation watermark;
- no TCP packet-level ACK reimplementation is introduced.

## R9-11 — shutdown, timeout and cleanup matrix

Prove bounded cleanup for success and failure paths:

- setup timeout;
- application/PTO/settlement timeout;
- client/server controlled stop;
- failed standby promotion;
- malformed/tampered input;
- post-exit listener/socket/process cleanup and same-address rebind where already supported.

Do not turn this into a service-manager project. The goal is truthful bounded research-process ownership.

## R9-12 — exact-tree gate + provenance + independent bounded R9 review

After the coherent R9 source/test sequence reaches the above boundary:

```bash
PYTHONDONTWRITEBYTECODE=1 bash scripts/check.sh
git diff --check
```

on the **final pushed exact developer SHA** in a clean checkout/worktree. Record exact SHA, UTC interval, OS/arch, rustc, commands/exit codes, clean initial/final tree. Hosted CI is extra cross-evidence only.

Then perform one bounded independent R9 review focused on:

- Session-above-Carrier layering;
- authenticated packet-number/ACK identity;
- bounded retained plaintext/recovery state;
- deadline-driven PTO/pacing/cwnd;
- fresh health evidence and manager hysteresis;
- real warm-TCP readiness/promotion ownership;
- Session uncertain replay/dedup;
- result/counter truthfulness;
- cleanup/no public exposure.

Concrete BLOCKER/HIGH -> smallest repair + regression + re-gate + continue. LOW/NOTE does not halt milestone progression.

# Q10/Q11/Q12 — pre-authorized continuation after R9 review

## Q10 — observability reconciliation

If R9 adds new runtime evidence fields/events, integrate them into existing bounded observability without duplicate logging frameworks. Preserve evidence domains: packet recovery, Carrier health/switch, and Session delivery are distinct.

## Q11 — status/release-evidence reconciliation and READY_LIVE classification

Update `docs/status.md`, `IMPLEMENTATION_PLAN.md`, `ROADMAP.md` where required, and `docs/release-security-review-packet.md` only to exact earned evidence.

If R9 cross-process + real TCP promotion is green and independently reviewed, create a **new, specific READY_LIVE row** for the changed hypothesis. Do not reopen frozen historical HY2/periodic/repeated-failover lines merely because new code exists.

A likely first live question, if dependencies are truly satisfied, is:

> Does the exact reviewed R9 binary preserve authenticated reliable-UDP recovery accounting and Session exactly-once delivery across a real self-owned client<->VPS path under bounded program-controlled post-admission Data/ACK suppression, and when committed health hysteresis is crossed, does it promote only to an actually authenticated/ready TCP standby with truthful cleanup?

This is cross-host real-socket integration evidence under controlled loss injection; it is **not** natural-loss prevalence, production capacity, public reachability, or performance superiority.

## Q12 — one minimal changed-hypothesis self-owned VPS run

Only if Q11 marks that exact question `READY_LIVE`, run one bounded experiment under `docs/standing-vps-lab-authorization.md`:

- administrator-owned client <-> administrator-owned VPS only;
- temporary unprivileged TCP/UDP listeners;
- smallest count/bytes/duration that answers the R9 question, well below standing maxima;
- exact binary/commit identity and actual parameters;
- authenticated Data/ACK/recovery/Session/switch evidence;
- bounded resource/socket observations where already available;
- cleanup verification;
- preserve a negative result exactly; no unchanged same-class retry.

Do not modify production route/firewall/DNS/proxy/tunnel/qdisc and do not target third parties.

# Non-blocking policy / authority gates

These remain separate and must not stall R9/Q10 local correctness work:

- `SessionRuntime.events` retention policy (`POLICY_BLOCKED_RESOURCE_BOUND`);
- D019 source-retention/no-reset policy;
- RSEC-001 representative adversarial-load suitability;
- signing/key custody/SBOM/publication trust;
- previous-frozen-release interoperability;
- final independent security/release/RC/freeze/production authority;
- IPv6 environment and frozen historical HY2/repeated-failover/periodic lines.

Do not invent TTL/LRU/history/capacity/security policy values in this lane.

# Deferred LOW observations from R8

Do not stop R9 for these, but fold them into nearby work when cheap:

1. add one built-binary regression pinning forced partial settlement (`--drop-ack-every 1 --settle-ms 1` -> `partial=true`, `ok=false`, exit 1);
2. consider deriving final R8 `ok` directly from authoritative `remaining_in_flight == 0` as well as the local outstanding map;
3. if Session receive semantics gain buffering, stop inferring duplicate from `Ok + no pop` and inspect explicit `DuplicateDedup` evidence;
4. if R8/R9 permits non-offset-derived `FrameId`, maintain an explicit `FrameId -> Session offset` mapping rather than assuming `frame.0 == offset`.

# Stop conditions

Pause the mainline only for:

- unresolved BLOCKER/HIGH that cannot be mechanically repaired under current committed semantics;
- genuinely new core Session/Carrier/ACK/crypto/wire architecture choice;
- destructive/canonical-meaning migration;
- production/third-party/new-credential requirement;
- action beyond standing authorization;
- maintainer-only policy/value decision that is actually on the critical path;
- repository/tool failure.

Otherwise: implement -> test -> commit/push -> next READY slice continuously. Reviewer cadence is inspection cadence, not a work quota.
