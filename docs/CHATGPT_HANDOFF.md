# ChatGPT reviewer handoff — R8 executable path landed but is NOT accepted; repair before R9

## Reviewed repository truth

- Developer source/test head reviewed: exact `94290a1de8e71e021a31a5fbdb5b54a16db439d1` (`refactor(cli): lab --scenario reliable-udp spelling`).
- Reviewer-only descendant: `de3d2d71bd1b142c460b885ca0324e89ce3597d0` adds `docs/reviews/independent-r8-executable-94290a1-20260914.md`; no source/test semantics changed by that reviewer commit.
- New developer sequence since the previous handoff:
  - `4b7a999` — executable loopback reliable-UDP lab implementation;
  - `5537e93` — lock/dependency tree closure;
  - `2df0e51` — developer-local exact-tree provenance for `5537e93`;
  - `94290a1` — source-only command spelling refactor to `lab --scenario reliable-udp`.
- Exact `5537e93` has developer-local clean exact-tree `scripts/check.sh` + `git diff --check` provenance. That provenance does **not** cover the later source change `94290a1`.
- Exact `94290a1` hosted `nightly decode fuzz smoke` is green, but hosted `stable checks` is **red** in `bash scripts/check.sh` (run `34769803250`).
- Open PRs: none. No WAN/VPS experiment occurred in this sequence.
- Governance remains unchanged: item 3 incomplete; item 4 incomplete; `RELEASE_CANDIDATE=false`; `PRODUCTION_READY=false`; `FREEZE=false`; `RELEASED=false`; `READY_LIVE: none`.

## Reviewer verdict

R8 is genuine mainline progress: reliable UDP moved from the test-only composition into a normal executable CLI surface with real loopback UDP sockets, mature authenticated `SecureSession`, `ReliableUdpRuntime`, and a real `SessionRuntime` above Carrier. However **R8 is not accepted yet**. The bounded independent challenge at exact `94290a1` found a red exact-head gate plus multiple mechanically repairable correctness/evidence defects.

The durable review is:

`docs/reviews/independent-r8-executable-94290a1-20260914.md`

All findings below are ordinary implementation/evidence repairs under committed semantics. They do **not** require maintainer policy or a new architecture decision. External coding agent should repair continuously, gate the final pushed tree, then continue through the remaining R8 scenarios and independent re-review without waiting for reviewer cadence.

## Priority 0 — restore a green exact-head tree

### BLOCKER-GATE-001 — `lab --scenario` refactor breaks legacy no-scenario lab

Current code does:

```rust
if parse(args, "--scenario", None) == "reliable-udp" { ... }
```

but `parse(..., None)` is the repository's **required-value** parser: absence calls `fail("missing --scenario")`. Existing process coverage intentionally invokes `neko lab --json` and expects the legacy canned failover fixture to succeed. Therefore exact `94290a1` deterministically breaks the preserved no-scenario contract and exact-head `scripts/check.sh` is red.

Repair now:

1. make `--scenario` optional without changing the generic required parser semantics elsewhere;
2. preserve `neko lab` / `neko lab --json` legacy behavior;
3. `--scenario reliable-udp` enters R8;
4. unknown/duplicate/missing scenario values fail deterministically before socket/identity side effects;
5. add built-binary process tests for both old and new surfaces.

Do not proceed to R9 while the current source tree is red.

## Priority 1 — R8 correctness/evidence HIGHs

### H-R8-001 — receiver ACKs the wrong packet-number space

`SecureSession::seal` is `sequence:u64be || Noise ciphertext`. `open()` authenticates/replay-checks that outer sequence and returns only decrypted application plaintext with record context removed.

Client correctly uses `sealed[..8]` as recovery packet number. Server currently decrypts first and then computes:

```rust
let pn = u64::from_be_bytes(plain[..8]...);
```

but `plain[..8]` is the encoded `NK` record header, not the authenticated outer sequence. Server ACK ranges therefore do not name the sender's recovery packet numbers. Client then hides the resulting mismatch with `let _ = rt.apply_ack(...)`.

Required repair:

- capture the outer sequence from received record bytes (`buf[..8]`) before `open`;
- do not commit that sequence to ACK/recovery state until `open()` succeeds and the authenticated inner record is valid Data;
- keep crypto sequence/recovery packet number, Carrier `FrameId`, and Session `(stream, byte_offset)` distinct;
- an ACK that fails `apply_ack` must be a typed negative/failure, never silently ignored.

Mandatory focused regression: sent packet `N` -> authenticated receiver ACK includes exactly `N` -> sender applies it and retires that in-flight copy. Tampered records create no ACK state. Future/never-sent ACK still fails closed.

### H-R8-002 — controlled suppression bypasses cwnd/pacing admission

The runtime public contract says caller must consult `can_send(bytes)` before sealing/sending and honor `pacing_interval_us` between admissions. Current R8 skips `can_send` when `suppress=true`, yet still calls `on_packet_sent` and charges an in-flight packet.

Controlled suppression represents a packet admitted by the sender and lost after the send decision; it cannot bypass the sender's own congestion gate.

Required repair:

- apply cwnd admission to **every** initial packet, including one later suppressed by the fault seam;
- refused admission consumes no crypto sequence if practical, no Session byte offset, no recovery state, no plaintext ownership;
- honor the pacing deadline contract for initial and retransmission sends using a bounded deterministic/logical or real clock;
- preserve exact measured/declared bytes conservatively; do not make the loss seam a congestion bypass.

### H-R8-003 — PTO is fired every loop instead of when due

`ReliableUdpRuntime::pto_probe()` explicitly means **fire a PTO**. Current lab calls it after every offer round with no PTO deadline check. This manufactures PTO/retransmission pressure instead of observing a deadline-driven recovery event.

Required repair:

- derive a bounded PTO deadline from current recovery timing state;
- pre-deadline poll produces no PTO/probe/retransmit/health evidence;
- only a due PTO calls `pto_probe()`;
- deterministic fault tests may advance a logical clock, but must prove the deadline crossing;
- record PTO count separately from generic polling.

### H-R8-004 — errors are discarded and JSON can fabricate successful counters

Current implementation discards results from:

- receiver `on_packet_received`;
- ACK socket send while still incrementing `acks_sent`;
- sender `apply_ack`;
- `on_retransmit_sent`;
- retransmission socket send while still incrementing `retransmits`.

Session receive conflicts/errors are folded into `duplicates`.

Required repair:

- count success only after actual success;
- preserve typed negative outcomes rather than `let _`;
- distinguish first Session delivery, exact duplicate suppression, conflict, other Session failure;
- distinguish initial offered/admitted/socket-sent/suppressed, ACK emitted/socket-sent/applied/rejected, PTO fired, retransmission admitted/socket-sent, newly lost and still in-flight;
- add application bytes and explicit cleanup/final outcome;
- `ok=true` only when the selected scenario's invariants actually complete; otherwise explicit `ok=false`/nonzero or a clearly partial non-success result.

The retained developer sample (`16` initial, `58` retransmits, `69` ACK sends, `4` delivered, `65` duplicates) is valid negative evidence of the current broken path. Do not cite it as successful R8 recovery.

### H-R8-005 — no bounded settlement phase; success may print with unfinished delivery

The client stops after the offer loop, immediately signals the server thread to stop, joins, and prints `ok=true`. Final ACKs/retransmissions can remain undrained and recovery may remain in-flight. Current sample proves this: only 4 first deliveries from 16 offered records still produced `ok=true`.

Required repair:

- after bounded offers, run a finite settlement/quiescence phase;
- drain authenticated ACKs and due recovery until all required logical bytes are accounted and recovery reaches the declared final condition, or until a finite deadline returns an explicit failure/partial outcome;
- do not stop the server before final ACK/recovery settlement;
- clean no-loss scenario must converge with zero PTO/retransmit/duplicate/conflict;
- controlled-loss scenario must converge to exactly one Session delivery per logical record or truthfully fail.

## Priority 2 — R8 evidence wording / layering

### M-R8-006 — manager-only TCP state is not a real TCP fallback

R8 opens UDP sockets only. `ready_standby()` and `WarmFallback` here are CarrierManager state/model observations; there is no authenticated TCP standby socket/application transfer in the command.

Do not label `fallback=true` / `active_tcp=true` as actual TCP transport migration. Rename/output it as manager switch decision/state or omit the claim until R9-D. The first real TCP carrier promotion remains R9.

## R8 completion matrix — continue after the HIGH repairs

Do not stop after merely restoring the gate. Complete these dependency-safe scenarios on the real built executable:

1. **R8-CLEAN:** no-loss loopback; authenticated Data/ACK; ACK retires in-flight; zero PTO/retransmit/duplicate/conflict; all offered logical bytes delivered once.
2. **R8-DATA-LOSS:** suppress exactly one admitted Data after send accounting; deadline-driven recovery; fresh packet number/nonce; stable `FrameId`; exactly one Session delivery.
3. **R8-ACK-LOSS:** suppress exactly one emitted ACK; due PTO/replacement; late/duplicate data remains one Session delivery.
4. **R8-REORDER:** replacement arrives before delayed original; duplicate suppression is Session-owned.
5. **R8-TAMPER:** tampered/unauthenticated Data or ACK produces no ACK/recovery/Session state promotion.
6. **R8-ADMISSION:** cwnd/plaintext-owner refusal is atomic and consumes no Session byte offset; pacing is observable and bounded.
7. **R8-HEALTH:** historical loss is not replayed as fresh health evidence; recoverable loss below hysteresis does not imply switch.
8. **R8-SETTLE:** bounded final drain/cleanup proves no pending in-flight state for a successful scenario.
9. **R8-CLI:** built-binary tests cover legacy `lab`, `--scenario reliable-udp`, JSON/human fields, argument bounds/unknown/duplicate/missing values, success and explicit failure exit semantics.

After the final source/test SHA:

```bash
PYTHONDONTWRITEBYTECODE=1 bash scripts/check.sh
git diff --check
```

Record clean initial/final tree, exact pushed SHA, UTC interval, OS/arch and rustc. Hosted CI is extra evidence, not a replacement. Decoder fuzz need not be rerun for pure CLI/recovery composition unless wire parser/framing grammar changes.

Then create/refresh one bounded independent R8 review. If no BLOCKER/HIGH remains, continue directly to R9.

## R9 — keep pre-authorized behind R8; do not wait once R8 closes

### R9-A connected UDP carrier seam if actually required

Use the smallest carrier-local adapter for an already-created/connected `UdpSocket` if the cross-process path cannot reuse current APIs cleanly. CLI owns bind/peer selection. Preserve bounded message limits, explicit close/error semantics, no arbitrary proxy/send-to surface, no Session evidence promotion.

### R9-B authenticated cross-process reliable-UDP

Extend/reuse current bounded process machinery:

- canonical negotiation before fresh Noise;
- trust/authz before application data;
- UDP Data goes through `ReliableUdpRuntime`;
- Session receive/dedup stays in `SessionRuntime`;
- authenticated packet ACK remains Carrier feedback only;
- finite setup/application/PTO/overall deadlines;
- bounded count/bytes and deterministic cleanup;
- secret-safe observed JSON.

### R9-C process negative/recovery matrix

Prove separate-process clean exchange, one Data suppression, one ACK suppression, tamper/future ACK refusal, duplicate suppression, timeout/shutdown cleanup, and no Session-delivery promotion from packet ACK.

### R9-D real packet-recovery-health -> warm TCP promotion

Only here connect fresh UDP recovery health to an **actually authenticated/ready TCP standby carrier**. Required cases:

1. recoverable loss below hysteresis -> remain UDP;
2. distinct fresh bad resolved outcomes crossing hysteresis -> real warm TCP switch if standby is truly ready;
3. invalid/unready standby -> `FallbackFailed`, never success;
4. uncertain Session bytes replay/dedup by Session byte identity;
5. packet ACK/recovery counters remain distinct from Session confirmed watermark.

Persist exact-tree local provenance + bounded independent R9 review before WAN classification.

## Q10/Q11/Q12 — preserve behind R9

- **Q10 observability:** reuse existing bounded observability for RTT, newly resolved loss, PTO, retransmit, health transition, actual switch result, and Session-owned duplicate/conflict/confirmed evidence. No plaintext/secrets/private topology.
- **Q11 reconciliation:** update `docs/status.md`, implementation plan and release packet only to exact earned evidence. Keep item 3/4 unchecked unless full criteria genuinely close. Classify `READY_LIVE` only after executable cross-process R9 + independent review + green exact-tree gate.
- **Q12 VPS:** if Q11 creates a genuine READY row, perform one minimal self-owned changed-hypothesis TCP/UDP run under standing authorization. Preserve negative results and cleanup; no same-class blind retry.

## Current policy/non-blocking gates

Still separate and must not stall these repairs:

- `SessionRuntime.events` retention policy (`POLICY_BLOCKED_RESOURCE_BOUND`);
- D019 source-retention/no-reset;
- RSEC-001 representative adversarial-load suitability;
- signing/key custody/SBOM/publication trust;
- previous-frozen-release interoperability;
- final independent security/release/RC/freeze/production authority;
- IPv6 environment and frozen historical HY2/repeated-failover/periodic lines.

Do not invent TTL/LRU/history/capacity/security values in the R8/R9 lane.

## Stop conditions

Pause this mainline only for an unresolved BLOCKER/HIGH that cannot be mechanically repaired under committed semantics, a genuinely new core Session/Carrier/ACK/crypto/wire architecture choice, destructive/canonical migration, production/third-party/new-credential need, action beyond standing authorization, or actual repository/tool failure. Otherwise repair -> test -> commit/push -> exact-tree gate -> next ready slice continuously.
