# ChatGPT reviewer handoff — live reliable-UDP S1–S9 reviewed; repair health-evidence replay before runtime/WAN

## Reviewed repository truth

- Current developer source tree reviewed: exact `94cfd4732dca6aae3e88fd188125653d9b44e900` after the continuous S1–S9 sequence opened by `b505a60`.
- Current `main` before this handoff refresh additionally contains docs-only exact `01ec9211dbcb319753649be9597c546d5cacac7c`, which records developer-local provenance and a self-described independent no-finding review.
- Source/test delta from `b505a60..94cfd47`: `neko-carrier` now owns a `neko-reliable` dependency and `PathRecovery`; `neko-wire` adds packet/ACK grammar; new carrier integration tests cover auth, loopback exchange, controlled loss/retransmit, health/fallback and observability. No `neko-cli` source changed in this sequence.
- Developer-local exact-tree provenance for `94cfd47` is present in `docs/local-gate-94cfd47-20260913.md`; hosted `stable checks` and `nightly decode fuzz smoke` on exact `94cfd47` are both green and remain cross-evidence only.
- Release/governance remains unchanged: item 3 incomplete; item 4 incomplete; `RELEASE_CANDIDATE=false`; `PRODUCTION_READY=false`; `FREEZE=false`; `RELEASED=false`; D019, `SessionRuntime.events`, and RSEC-001 remain separate policy/security gates.

The S1–S9 work is substantial and is **not** rejected wholesale. However the `01ec921` no-finding review is not accepted as final closure: this reviewer found one correctness/evidence HIGH plus several bounded follow-ups that must close before a `READY_LIVE` row is created.

## Finding H-RUDP-001 — repeated polling replays one cumulative loss sample into multiple health failures

**Severity: HIGH for reliable-UDP health/fallback correctness and any S11 live claim.**

`PathRecovery::health_sample()` currently returns a cumulative snapshot:

- `loss_per_mille = packets_lost * 1000 / packets_sent`;
- current smoothed RTT;
- current `pto_count`.

The S8 test `packet_loss_drives_udp_degrade_then_warm_tcp_fallback_replays_uncertain_once` computes one 625/mille snapshot after one ACK/loss transition, then calls `CarrierHealth::observe(PathId(1), sample)` repeatedly with that **unchanged same snapshot** until the hysteresis counter reaches `Degraded`.

That is not evidence of sustained packet loss. It demonstrates that repeated polling of one historical cumulative observation can be counted as several new bad observations. A runtime loop that polls the same snapshot can therefore create a false degradation/fallback without any new packet/recovery evidence.

### Required repair

Implement the smallest explicit fresh-evidence bridge between `PathRecovery` and `CarrierHealth`.

Acceptable shape: a bounded cursor/checkpoint or equivalent monotonic observation token that makes a health observation consumable **at most once per new recovery evidence state**. Do not change existing health thresholds or invent new security/performance values.

Required tests:

1. one loss transition -> one bad health observation;
2. repeated polls with no new packet/ACK/PTO/recovery evidence -> no additional bad streak advancement;
3. distinct new bad recovery intervals/updates can eventually reach Degraded under the existing hysteresis;
4. a subsequent genuinely clean/new interval follows existing recovery semantics rather than replaying old cumulative loss forever;
5. packet feedback still cannot validate a Path or confirm Session delivery.

Do not enter S11/live execution until this HIGH is closed and independently re-reviewed.

## Finding M-RUDP-002 — two packet-number ownership sources exist

**Severity: MEDIUM correctness/design-integrity.**

`PathRecovery` owns `next_packet_number` / `allocate_packet_number()`, but the actual authenticated S3–S5 path does not use that allocator. Instead it seals through `SecureSession`, extracts the first 8-byte Noise stateless record sequence, and treats that AEAD nonce/sequence as the packet number passed to `PathRecovery::on_sent`.

For the committed S3 candidate semantics, the authenticated record sequence is therefore the real packet-number source. Keeping a second independent allocator in `PathRecovery` creates a divergence footgun and contradictory ownership claim.

### Required repair

Use one source of truth. Under current S3 semantics, prefer the smallest change that treats the authenticated `SecureSession` record sequence as the live packet number and removes/deprecates the unused independent allocator/state from `PathRecovery`, unless exact-current code proves it is required by another tested owner.

Tests must cover monotonicity/exhaustion through the actual chosen source and must preserve nonce non-reuse. Do not reset/reuse a crypto sequence merely because path generation changes.

If solving this would require redesigning the core crypto construction, stop that sub-lane and record the exact conflict rather than inventing a new nonce scheme.

## Finding M-RUDP-003 — one negative ACK test is vacuous

**Severity: MEDIUM evidence integrity; code semantics may already be correct.**

`crates/neko-carrier/tests/reliable_udp_exchange.rs::malformed_ack_record_never_reaches_recovery` ends its adjacent-range check with `...is_err() || true`, so that assertion always passes.

Replace it with a real assertion at the correct boundary:

- if `encode_ack` intentionally rejects adjacent/noncanonical ranges, assert the encoder error directly; and/or
- construct raw malformed bytes and assert `decode_ack` rejects them.

No `|| true`, `unwrap_or_default` masking, or equivalent vacuous predicate. Re-run focused wire/carrier tests and fuzz only if parser bytes/code change.

## Candidate M-RUDP-004 — verify congestion accounting for non-ack-eliciting packets

**Severity: candidate MEDIUM; prove before changing code.**

`PathRecovery::on_sent` charges Reno bytes-in-flight only when `packet.ack_eliciting`, while `Recovery::on_ack` reports `acked_bytes`/`lost_bytes` from all retired `SentPacket` bytes and `PathRecovery::on_ack` feeds those totals into `reno.acked`/`reno.lost`.

Write a focused regression using an allowed `ack_eliciting=false` packet. It must not inflate cwnd or retire congestion bytes that were never charged. If the mismatch reproduces, fix the accounting at the smallest layer; do not change congestion-control constants. If current semantics intentionally prohibit such packets from recovery ownership, encode that invariant explicitly and reject them rather than leaving ambiguous accounting.

## S1–S9 acceptance boundary

Accepted as useful foundations after the repairs above:

- S2 packet/ACK grammar and packet header, subject to the vacuous-test cleanup;
- authenticated ACK application only after successful `SecureSession::open`;
- real UDP loopback socket tests;
- frame-level fresh re-encoding rather than old-ciphertext resend;
- recovery/Carrier observability layer separation;
- existing Session/Carrier/Path evidence separation remains conceptually correct.

Not yet accepted as a complete executable reliable-UDP runtime/fallback path:

- receiver ACK generation is still test-assembled rather than owned by a production/lab runtime component;
- recovery-to-health hysteresis has the fresh-evidence HIGH above;
- the S8 fallback fixture manually calls health observation and manager fail/activate operations rather than exercising a single executable event loop;
- `b505a60..94cfd47` changes no `neko-cli` source, so there is not yet a deployable CLI/owned-lab command that exercises this integrated recovery path across hosts.

Therefore **`READY_LIVE: none` remains authoritative** until the executable seam below is complete and reviewed.

## Continuous READY_LOCAL queue

The external coding agent should continue immediately. Do not wait for the next reviewer after each slice.

### R1 — close H-RUDP-001 fresh-evidence health bridge

Repair and test the HIGH exactly as specified above. Commit/push, then continue.

### R2 — close M-RUDP-002 single packet-number source

Remove/diverge-proof the unused second packet-number allocator under current authenticated-sequence semantics. Preserve nonce non-reuse and packet monotonicity. Commit/push, continue.

### R3 — close M-RUDP-003 vacuous negative test

Make the adjacent/noncanonical ACK negative executable and meaningful. Commit/push, continue.

### R4 — resolve M-RUDP-004 non-ack-eliciting congestion accounting

Regression first, then smallest repair if reproduced. Commit/push, continue.

### R5 — receiver-side authenticated ACK tracker/generator

**Goal:** stop hand-assembling ACK ranges in integration tests and give the UDP recovery runtime a bounded receiver-side ACK state.

Requirements:

- obtain packet number from the authenticated SecureSession record sequence only after successful `open`/context validation;
- maintain canonical bounded ACK ranges with the existing hard limits;
- encode ACK through existing `AckPayload` grammar;
- duplicate/reorder behavior deterministic; malformed/tampered packets never enter ACK state;
- ACK state remains Carrier-local and produces no Session delivery evidence;
- bounded memory and cleanup.

Do not invent a second ACK protocol or new wire version.

### R6 — bounded retransmission frame ownership for executable runtime

**Goal:** a runtime caller must be able to map `FrameId` retransmit output back to bounded authenticated plaintext/frame material for fresh re-sealing without retaining/reusing old ciphertext images.

Prefer reuse of existing Session/Carrier ownership rather than a second delivery ledger. If a minimal carrier-local frame table is required, hard-bound it to existing recovery limits and release it exactly when no outstanding copy remains.

Tests: ACK release, loss -> fresh re-encode, overlapping original/retransmit copies, duplicate ACK, cleanup/cancel/close, capacity refusal. No Session confirmation from packet ACK.

### R7 — one coherent local runtime/event-loop fixture

Build a bounded local runtime fixture that composes, in one actual flow rather than manual test choreography:

- UDP socket send/receive;
- authenticated packet number + receiver ACK generation;
- ACK application, RTT/loss/PTO/retransmit;
- fresh-evidence health bridge;
- existing warm TCP standby readiness;
- CarrierManager degradation/fail/promotion;
- Session uncertain replay/dedup accounting;
- observability.

A single transient loss observation followed only by polling must **not** cause fallback. Multiple genuinely new bad observations may cause fallback according to existing health/hysteresis semantics.

### R8 — expose the runtime through an executable bounded lab path

No WAN row may be created from integration tests alone.

Reuse the existing CLI/failover/owned-lab machinery where practical; avoid adding a permanent new public command if a bounded experimental/lab mode of an existing command is enough. The executable path must use the same reliable-UDP runtime components from R5–R7, not duplicate the tests in CLI code.

Required capabilities for the first experiment:

- exact binary/commit identity;
- bounded authenticated UDP records with automatic receiver packet ACKs;
- deterministic **lab-only sender-side packet suppression** or another non-production injection mechanism to create controlled packet loss without modifying production qdisc/route/firewall;
- warm TCP standby and automatic health-driven fallback;
- structured packet/PTO/retransmit/switch/application counters;
- bounded duration/count/bytes and cleanup.

This controlled injection is evidence of controlled packet loss, not natural Internet loss.

### R9 — process/loopback end-to-end acceptance

Exercise the executable R8 path locally as real processes/sockets. At minimum:

- clean no-loss run with no false fallback;
- one isolated loss that recovers without replaying the same health evidence into fallback;
- sustained distinct controlled losses that cross existing hysteresis and trigger warm TCP fallback;
- uncertain/replayed/duplicate/lost application byte accounting;
- ACK loss and PTO path;
- process/listener cleanup and bounded state.

### R10 — dedicated independent re-review of repaired executable surface

Challenge R1–R9 on exact pushed tree:

- fresh-vs-replayed health evidence;
- single packet-number/nonce source and no reuse;
- ACK generation/canonical bounds;
- packet/frame/congestion exact-once accounting;
- retransmission crypto freshness;
- manager automatic transition, not manual test choreography;
- Session delivery separation;
- resource bounds and secret-safe observability;
- CLI/process truth boundaries.

Concrete finding -> repair before live. No finding -> precise accepted note + exact-tree provenance.

### R11 — create one genuinely new READY_LIVE experiment

Only after R10 acceptance, update the live opportunity classification from `READY_LIVE: none` for exactly one bounded question:

> On self-owned client/VPS endpoints, does the executable authenticated reliable-UDP path recover from controlled sender-side packet suppression and, when distinct bad recovery observations cross the existing health hysteresis, automatically fail over to the already-ready warm TCP path without duplicate/lost logical application delivery?

Use minimum parameters within standing authorization. Do not touch production qdisc/route/firewall. Preserve exact commit/binary identity, actual injection parameters, packet/PTO/retransmit/switch/application counters and cleanup. A negative result is valid.

### R12 — item-3/item-4 reconciliation and refill

After R11, distinguish:

- controlled local suppression vs natural Internet degradation;
- packet recovery success vs Session delivery;
- one bounded result vs reliability/performance rate.

Then refill from the actual new evidence. D019, `SessionRuntime.events`, RSEC-001, signing/SBOM and RC/freeze/release/production decisions remain separate gates.

## Required validation

For every coherent source/test slice:

```text
PYTHONDONTWRITEBYTECODE=1 bash scripts/check.sh
git diff --check
clean initial/final tree
```

Persist exact pushed SHA, commands, UTC times, exits, OS/arch, stable Rust and clean-tree state. Hosted GitHub Actions remains optional cross-evidence.

Wire/parser byte changes require the pinned decode fuzz build/run (`-max_total_time=30 -max_len=8192`). Pure carrier/runtime/test changes do not mechanically require parser fuzz.

## Stop conditions

Stop only the affected lane for:

- unresolved BLOCKER/HIGH;
- required core Session/Carrier/crypto/wire architecture change not already determined by committed semantics;
- destructive migration;
- new security/capacity numeric policy;
- production/third-party/out-of-standing-authorization action;
- new credentials or unavailable environment;
- actual repository/tool failure.

Otherwise continue R1 -> R12 without waiting for reviewer cadence.

## Governance

- item 3 incomplete;
- item 4 incomplete;
- `RELEASE_CANDIDATE=false`;
- `PRODUCTION_READY=false`;
- `FREEZE=false`;
- `RELEASED=false`;
- D019 policy-blocked;
- `SessionRuntime.events` policy-blocked;
- RSEC-001 promotion/suitability open;
- `READY_LIVE: none` until R10 accepts an executable path and R11 is explicitly classified READY.
