# Independent bounded R8 executable-path re-review — exact `8c9f584` (docs head `722732e`)

**Reviewed revision:** source/test `8c9f584`; docs head `722732e2d2c4398efcfc5e55f6112d0c2c46ce86`. `8c9f584` and `722732e` differ only by the gate note (`docs/local-gate-8c9f584-20260914.md`), so the reviewed source is identical at both.
**Reviewed files:** `crates/neko-cli/src/main.rs` (sha256 `1b3d7c22ae0411a3…`), `crates/neko-cli/tests/probe.rs`. Only these two non-docs files changed in `f4aca03..722732e`.
**Method:** read-only review of the repository plus an **isolated detached worktree** at `722732e` (the shared worktree was detached at `f4aca03` with a concurrent editor's uncommitted `main.rs`, so it was not used as a build/evidence source). **Date:** 2026-09-14.

## Verdict

**All six findings (BLOCKER-GATE-001, H-R8-001..005, M-R8-006) are addressed and independently verified. No correctness, evidence-fabrication, or exposure defect found.** Four bounded observations (all LOW, non-blocking) below; one recommends a single missing test for the failure path.

## Verification of each finding

**BLOCKER-GATE-001 — VERIFIED.** `parse_scenario` is a dedicated optional parser (`main.rs`): absent key -> `None` (legacy fixture preserved); duplicate -> `fail("duplicate --scenario")`; value-less or flag-valued -> `fail("missing --scenario value")`; unknown -> `fail("unknown lab scenario: …")`. The range checks in `lab_reliable_udp` all precede the first `UdpSocket::bind`, and `parse_scenario` runs in `lab()` before `lab_reliable_udp`, so no negative case can create a socket or identity. Independently reproduced **10 negative cases** (the 8 in `lab_scenario_arguments_fail_closed_before_sockets` plus `--drop-every 17` and `--scenario --json`): every one exits **2** with an empty stdout and a typed stderr message. Plain `lab` and `lab --json` still emit the legacy socket-free `"demo":"failover"` fixture.

**H-R8-001 — VERIFIED.** The receiver reads `seq` from `sbuf[..8]` (the authenticated outer AEAD sequence) and commits it via `rt.on_packet_received(seq, true)` only after `open()` succeeded, `decode` succeeded, and `record_type == Data`. Measured on the clean no-loss path: `acks.applied=8`, `acks.rejected=0`, `remaining_in_flight=0`, `delivered=8` — the previous 69-ACK/58-retransmit behaviour is gone.

**H-R8-002 — VERIFIED.** Admission precedes suppression: `initial_admitted` counts every admitted initial packet (8 in `--drop-every 4`, of which `wire_sent=6` / `suppressed=2`), and the Session byte offset advances only after `on_packet_sent` returned `Ok`, so a refusal consumes no Session byte space. Retransmission consults `rt.can_send(...)` and counts `retransmit_refused` (0 in all runs). A cwnd refusal sets `cwnd_refusals` and `partial=true`.

**H-R8-003 — VERIFIED.** PTO fires only when `now_us >= oldest.sent_at_us + rtt.pto_us(granularity, max_ack_delay, pto_count)` on the deterministic logical clock. Clean no-loss gives `pto_due=0, pto_fired=0`; loss cases fire only past the deadline (`pto_due == pto_fired` in every run I made). Backoff is self-limiting because `on_pto` raises `pto_count`, which raises the next deadline.

**H-R8-004 — VERIFIED.** Every success counter increments after the corresponding operation succeeds (`initial_wire_sent` only on `send` Ok; `acks_emitted` after `seal` Ok; `acks_wire_sent` only on send Ok; `acks_applied` only on `apply_ack` Ok; `delivered` only on `pop_receive` `Some`), and typed negatives are retained (`unauthenticated`, `recv_rejected`, `acks.rejected`, `acks.failed`, `retransmit_refused`). The JSON separates initial offered/admitted/wire_sent/suppressed, ACK emitted/wire_sent/suppressed/failed/applied/rejected, PTO due/fired, retransmit attempts/wire_sent/refused, delivered/duplicates/conflicts, application_bytes, newly_lost, remaining_in_flight, manager switch and cleanup. Conflicts are no longer folded into duplicates. `ok` requires `settled && !partial && conflicts==0 && acks_rejected==0 && recv_rejected==0 && delivered==rounds && cleanup_ok`, and a non-`ok` run exits **1** (reproduced). The gate note's three measured configurations reproduce **exactly**, including `--drop-every 0 --drop-ack-every 3` -> `delivered=8, duplicates=3, acks.suppressed=3, pto_fired=3, remaining_in_flight=0`.

**H-R8-005 — VERIFIED (and reachable).** The bounded settlement loop drains until `outstanding` is empty or `--settle-ms` elapses, then reports `partial=true` and `ok=false`. I forced it: `--drop-ack-every 1 --settle-ms 1` -> `settled=false, partial=true, remaining_in_flight=12, cleanup=verified` and **exit 1**. So `ok=true` with unfinished delivery is not reachable.

**M-R8-006 — VERIFIED.** Both output modes carry `manager_switch_scope` ("manager decision only; this command opens no TCP socket") / `manager-decision-only-no-tcp-socket`. The command opens no TCP socket anywhere.

## Coverage items

| Item | Result |
|---|---|
| Owner / bounds | OK — `RetransmitBuffer` bounds enforced by the runtime; the lab's `outstanding` map grows only via deadline-gated PTO retransmits (backoff) and is capped by `--settle-ms`/`--rounds`, so it is bounded. |
| Packet-vs-Session evidence separation | OK — `acks_*`/`newly_lost`/`remaining_in_flight` are Carrier recovery; `delivered`/`duplicates`/`conflicts` are Session; ACK never promotes Session delivery. |
| Identity separation | OK — packet number = authenticated AEAD sequence; `FrameId` stable; Session identity is `(stream, byte offset)`. |
| Observed vs fabricated JSON | OK — see H-R8-004; no counter is incremented before its operation succeeds. |
| Cleanup | OK — client socket dropped, server socket owned by the joined server thread, then both addresses are rebound to prove release; `cleanup=verified`. |
| Fault-seam truthfulness | OK — suppression is counted as `suppressed`, never as `wire_sent`; the suppressed packet is still admitted, matching the intended semantics. |
| No production/public exposure | OK — both sockets bind `127.0.0.1:0`; no TCP socket, no daemon, no listener; help text and the JSON scope field both state fixture / not production. |

Also reproduced: full `--test probe` **41 passed**; `cargo clippy -p neko-cli --all-targets -- -D warnings` clean; `cargo fmt --all --check` clean; focused `lab_` **2 passed**.

## Bounded observations (LOW, non-blocking)

1. **[LOW] The H-R8-005 failure path has no test.** No test in `probe.rs` asserts `partial`/`ok=false`/exit 1 (grep for `partial` -> none); I could only reach it by an ad-hoc run. Since H-R8-005's whole value is the guarantee "`ok=true` cannot be emitted with unfinished delivery", one bounded test should pin it (e.g. `--drop-ack-every 1 --settle-ms 1` -> assert `partial=true`, `ok=false`, exit 1). A regression here would currently be silent.
2. **[LOW] `ok` is gated on the lab's own `outstanding` map (`settled`) rather than the runtime's authoritative `remaining_in_flight`.** These agreed in every run I made (including the forced-partial run: `settled=false` with `remaining_in_flight=12`), and I confirmed the two views are kept in sync (`neko_reliable::Recovery::on_ack` removes both acked and lost packets from `sent`, and the lab removes the same numbers), so I found **no reachable divergence**. Using `remaining_in_flight == 0` (already reported) would tie the verdict to the authoritative view.
3. **[LOW] `duplicates` is inferred as `receive() == Ok && pop_receive() == None`.** I verified `SessionRuntime::receive` has exactly two `Ok` paths — `DuplicateDedup` (no delivery) and `DataReceived` (queued) — so the inference is currently **exact**, and the measured `duplicates=3` in ACK-loss runs are genuine. It is nonetheless coupled to "Session has no reorder-buffering path"; reading the `DuplicateDedup` event (as the carrier fixture does) would be self-documenting and change-proof.
4. **[LOW] Retransmit re-encodes with `offset: frame.0`.** Correct here because every lab frame is offset-derived (`FrameId(frame_offset)`), unlike the carrier fixture where a non-offset frame existed and required an explicit `FrameId -> offset` map. Not a defect; noted only for consistency with the earlier H-RUDP-014-class caution if the lab ever gains a non-offset frame.

**Methodology caution (for the lane, not a finding):** my first measurement pass used an unquoted `$args` in the remote `zsh`, which does **not** word-split by default; all runs silently printed the *default* configuration, which would have looked like "arguments are ignored". Passing literal arguments reproduced the gate note's numbers exactly. Worth quoting explicitly (or using arrays) in any future shell-driven evidence.

## Boundary / not claimed

Bounded independent re-review of the R8 executable path only. **Not** claimed: R9, Q4-E plaintext-side, a standalone Q4-B scenario, `SessionRuntime.events`, D019, RSEC-001 suitability, signing/key-custody/SBOM, previous-frozen-release interoperability, item 3, item 4, or release authority. Release flags unchanged: `RELEASE_CANDIDATE=false`, `PRODUCTION_READY=false`, `FREEZE=false`, `RELEASED=false`; `READY_LIVE: none`. The concurrent-editor edits in the shared worktree were not adopted and were not used as evidence.
