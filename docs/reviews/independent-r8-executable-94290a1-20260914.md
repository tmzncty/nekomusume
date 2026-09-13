# Independent bounded R8 executable review — exact `94290a1`

**Reviewed exact tree:** `94290a1de8e71e021a31a5fbdb5b54a16db439d1` (`refactor(cli): lab --scenario reliable-udp spelling`).

**Scope:** the newly executable `neko lab --scenario reliable-udp` path, its exact dependency/provenance commits (`4b7a999`, `5537e93`, `2df0e51`, `94290a1`), the public `ReliableUdpRuntime` contract, `SecureSession` record-sequence format, existing CLI process-test expectations, and current hosted checks. This is a bounded correctness/evidence review, not a security audit, cryptanalysis, WAN result, release decision, or production approval.

## Verdict

**R8 is not accepted yet.** The executable direction is correct and is real progress beyond the old test-only composition, but current exact `94290a1` has one red exact-head stable gate plus multiple mechanically repairable correctness/evidence defects. Do not proceed to R9/WAN until these are closed and one exact pushed tree has a clean local gate and bounded independent re-review.

## BLOCKER-GATE-001 — current exact HEAD is red

GitHub-hosted run `34769803250` on exact `94290a1` has:

- `nightly decode fuzz smoke`: success;
- `stable checks`: **failure** in `bash scripts/check.sh`.

The source-only `94290a1` refactor changed `lab()` from an optional legacy selector to:

```rust
if parse(args, "--scenario", None) == "reliable-udp" { ... }
```

but `parse(..., None)` is a required-value parser: absence calls `fail("missing --scenario")`. Existing process coverage intentionally invokes `neko lab --json` and expects the legacy deterministic failover demo to remain successful. Therefore the current source breaks the preserved no-scenario compatibility contract and explains a deterministic full-gate failure without needing a speculative CI attribution.

**Required repair:** parse `--scenario` optionally without making it mandatory; preserve existing no-scenario `lab` behavior; reject unknown scenario values explicitly; add actual binary process tests for no-scenario compatibility, the `reliable-udp` scenario, invalid/duplicate/missing scenario values, and bounded `--rounds`/`--drop-every` validation. Then rerun a clean exact-tree gate on the final pushed source SHA.

## HIGH H-R8-001 — receiver ACKs the wrong packet-number space

`SecureSession::seal` format is explicitly:

```text
sequence:u64be || Noise ciphertext
```

and `SecureSession::open` authenticates/replay-checks that outer sequence and returns only the decrypted application plaintext (record context stripped).

The client correctly derives its recovery packet number from `sealed[..8]`. The server currently does the opposite after decrypting:

```rust
let plain = server_sess.open(&buf[..n])?;
let rec = neko_wire::decode(&plain)?;
let pn = u64::from_be_bytes(plain[..8]...);
rt.on_packet_received(pn, true);
```

`plain[..8]` is the beginning of the encoded `NK` record header, not the authenticated SecureSession sequence. Server ACK ranges therefore do not name the sender's recovery packet numbers. The client then suppresses the resulting error with `let _ = rt.apply_ack(...)`.

The persisted sample is consistent with this broken accounting: 16 initial records produce 69 ACK sends, 58 retransmits, only 4 first deliveries and 65 duplicate observations. That sample is useful negative evidence; it is **not** successful R8 recovery evidence.

**Required repair:** capture the outer sequence from the received authenticated record bytes (`buf[..8]`) before `open`, but only commit it to `on_packet_received` after `open` succeeds and the authenticated Data record is structurally valid. Keep packet number distinct from Session byte offset and Carrier `FrameId`. `apply_ack` failure must not be discarded; an authenticated ACK that cannot be applied must fail/record a typed negative rather than silently continue.

Mandatory regressions should prove that a sent packet `N` yields an ACK range containing exactly `N`, that the ACK retires the sender's in-flight copy, that a forged/tampered record cannot create ACK state, and that future/never-sent ACK remains fail-closed.

## HIGH H-R8-002 — controlled suppression bypasses the cwnd admission contract

The runtime public contract states callers must consult `can_send(bytes)` **before sealing/sending** and must honor pacing between admissions. The lab currently does:

```rust
if !suppress && !rt.can_send(1200) { continue; }
...
rt.on_packet_sent(...);
if suppress { /* do not socket-send */ }
```

Thus every packet selected for controlled loss bypasses congestion-window admission even though it is still recorded as an in-flight send. Controlled suppression is supposed to model a packet admitted/sent by the transport and lost after that decision, not create traffic the sender would have been forbidden to admit.

**Required repair:** apply the same cwnd admission to every initial packet, including packets later suppressed by the fault seam; do not consume Session byte space or recovery/plaintext ownership on a refusal. Honor the runtime pacing contract with a bounded deterministic/real deadline rather than ignoring it. Retransmission must also respect the admitted send contract and report any refusal explicitly.

## HIGH H-R8-003 — PTO is fired unconditionally, not by a PTO deadline

`ReliableUdpRuntime::pto_probe()` is documented as **"Fire a PTO"**. The current CLI calls it after every round, regardless of RTT/PTO deadline or whether the outstanding packet has actually waited long enough. This converts a caller-triggered control action into apparent PTO/retransmission evidence and is a major reason the sample explodes to 58 retransmissions.

A deterministic fault harness may advance a bounded logical clock, but it still must demonstrate that the current recovery deadline was reached before firing PTO. A real-time lab may use a bounded timer. It must not call `pto_probe()` merely because one loop iteration ended.

**Required repair:** make PTO firing deadline-driven from current recovery timing state; distinguish `pto_due`, `pto_fired`, and ordinary polling. Add a negative test proving a pre-deadline poll creates no probe/retransmit/health evidence.

## HIGH H-R8-004 — JSON/counters discard errors and can fabricate successful evidence

Current paths discard several results:

- server `on_packet_received` result;
- server ACK socket-send result while still incrementing `acks_sent`;
- client `apply_ack` result;
- retransmit `on_retransmit_sent` result;
- retransmit socket-send result while still incrementing `retransmits`.

Session `receive` errors are also folded into `duplicates`, so a conflicting logical record is indistinguishable from an exact idempotent duplicate.

The emitted JSON always says `"ok":true` and currently omits several fields the R8 handoff required (conflicts, application bytes, ACKs actually applied, newly lost packets, PTO fires, retained in-flight state/final settlement, cleanup outcome). `packets_sent` includes deliberately suppressed datagrams and excludes retransmission wire sends, so its name is not a truthful socket-send count.

**Required repair:** do not increment an observed-success counter unless that operation actually succeeded. Preserve typed/structured negative outcomes. Separate at minimum initial attempts/admitted sends/socket sends/suppressed faults, ACK emitted/socket-sent/applied/rejected, PTO fires, retransmission attempts/sends, first Session deliveries/exact duplicates/conflicts, application bytes, packet newly-lost/remaining-in-flight, manager-only switch state, and cleanup. `ok=true` must require the declared scenario invariants; otherwise emit `ok=false`/nonzero exit or an explicitly partial outcome.

## HIGH H-R8-005 — no bounded settlement phase; `ok=true` can be emitted with unfinished delivery

The client terminates after the fixed offer loop, sets `server_done`, joins the receiver thread and prints success. ACKs generated for the final sends/retransmits need not be drained, recovery can remain in-flight, and application delivery may be incomplete. The retained sample itself reports 4 first deliveries from 16 offered logical records while still claiming `ok:true`.

**Required repair:** after bounded offers, drive a bounded settlement/quiescence phase until all scenario-required logical bytes are delivered/duplicates classified and recovery is drained, or until a finite deadline produces an explicit failure/partial result. Do not stop the server before final ACK/recovery settlement. A clean no-loss case must converge without PTO/retransmit; a controlled-loss case must converge exactly once at Session level or truthfully fail.

## MEDIUM M-R8-006 — manager-only TCP state is labeled as actual fallback

The R8 lab opens only UDP sockets. `ready_standby()`/`WarmFallback` operate on CarrierManager state; there is no authenticated TCP standby socket/application transfer in this command. Therefore `fallback=true` / `active_tcp=true` is at most a **manager decision/model observation**, not a real TCP transport switch or same-Session TCP delivery.

Keep this distinction explicit in JSON/human output (`manager_switch_*` or equivalent) and documentation. The first actual TCP carrier/application promotion remains R9-D and must not be pre-claimed by R8.

## R8 coverage gaps still open after the repairs

The current source adds no new `crates/neko-cli/tests/probe.rs` process cases for the scenario. After the HIGH repairs, continue the already authorized R8 queue rather than stopping at one green invocation:

1. exact clean no-loss scenario: ACKs retire in-flight state, zero PTO/retransmit/duplicate/conflict;
2. one controlled Data loss with deadline-driven recovery and exactly one Session delivery;
3. one controlled ACK loss with replacement + exact duplicate suppression;
4. replacement-before-delayed-original ordering;
5. authenticated tamper/malformed/future ACK negatives with zero state promotion;
6. cwnd/plaintext-owner refusal atomicity and pacing observability;
7. bounded settlement/cleanup and truthful JSON/human/exit behavior;
8. real built-binary process tests for valid/invalid arguments and legacy no-scenario compatibility.

Then persist developer-local exact-tree `scripts/check.sh` + `git diff --check` provenance and request one bounded independent R8 re-review. Only a clean R8 moves to R9 connected/external process sockets.

## Governance/live boundary

No WAN/live experiment was reviewed here. `READY_LIVE: none` remains controlling. Item 3 and item 4 remain incomplete; release/RC/freeze/production flags remain unchanged. D019, `SessionRuntime.events`, RSEC-001, signing/key-custody/SBOM and final release authority remain separate non-blocking policy gates for this R8 repair work.
