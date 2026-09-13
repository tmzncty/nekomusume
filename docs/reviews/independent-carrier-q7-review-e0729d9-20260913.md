# Independent Q7 bounded R7 re-review — carrier layering/ownership repair at exact `e0729d9`

**Reviewed source/test revision:** `e0729d96f59988e7b918d21cfb19579c319b0733` (`fix(carrier): restore Session-above-Carrier layering (H-RUDP-012)`).
**Index/docs head reviewed:** `dcad89ade2ccdde449a8ff4527e2ea1afcd6395a` (docs-only provenance note for the same source tree).
**Review type:** independent bounded re-review (Q7), read-only. **Date:** 2026-09-13 (UTC `2026-09-13T15:08:15Z`). Host `tmzn@192.168.122.1`, workdir `/media/tmzn/DATA5/nekomusume-work`, rustc 1.98.0.
**`crates/neko-carrier/src/lib.rs` sha256:** `9671e80d928c5c336a6562879c1d9c6924729c4f7a65cc2886ca84a0068a00d3`.
**Not** an audit, cryptanalysis, adversarial-load assessment, independent maintainer approval, or release authorization. No source file was edited.

## Verdict

**No correctness, layering, resource-bound, or evidence-separation defect found.** The H-RUDP-012 layering repair and the H-RUDP-011 transactional-owner remainder are substantively correct at this exact tree.

**Three documentation-only defects** were found (stale doc-comments left behind by the `deliver_logical` removal and one stale formula). They do **not** affect behaviour, tests, clippy or any gate. Reported here before any edit, per the review contract; source remains unedited.

## Q7 checklist — validated points

| # | Q7 area | Result | Evidence |
|---|---|---|---|
| 1 | Transactional owner | OK | `on_packet_sent` (`lib.rs:4550-4589`): cwnd check → reserve plaintext → `recovery.on_sent` → only then `packet_frames.insert`. Any refusal leaves no recovery packet, no Reno charge, no packet→frame entry. Owner error preserved as `PathRecoveryError::Retransmit(RetransmitError)` (`:4096-4098`, `:4567-4569`). |
| 2 | Rollback isolation | OK | `already_retained` captured before `track` (`:4566`); rollback releases only a copy this call newly reserved (`:4580-4583`), so a pre-existing identical frame survives. Regression `recovery_rejection_preserves_an_already_retained_identical_frame`. |
| 3 | Retained-state bounds | OK | `RetransmitBuffer` (`:4769-4858`) bounded by `max_frames`/`max_bytes` (constructed `64, 8192` at `:4577-4578`); `track` checks `bytes.len() > max_bytes` → `FrameTooLarge`, then frame/byte capacity → `Capacity`, before insert; `saturating_add`/`saturating_sub` throughout; `clear()` resets both count and byte total. |
| 4 | Ownership-required retransmit | OK | `on_retransmit_sent` (`:4705-4727`) fails **before** recording a packet when `retransmit.get(frame).is_none()` (`:4715-4717`). Regression `retransmit_requires_retained_ownership_and_teardown_is_deterministic`. |
| 5 | Deterministic teardown | OK | `teardown()` (`:4732-4735`) drops `packet_frames` and the plaintext owner together. |
| 6 | Resolved-health math (H-RUDP-001D) | OK | `outcome_epoch` advances only on resolved outcomes — non-empty ack/lost/retransmit (`:4190-4195`) or probe-scheduling PTO (`:4215-4217`); a bare `on_sent` does not (`:4144-4145`). `fresh_health_sample` (`:4277-4299`) returns `None` when unchanged, and uses the resolved interval delta `resolved_packets`/`resolved_lost` (`:4182-4187`) with `checked_div` → `0` for a zero denominator. No send-time denominator on the manager path. |
| 7 | Session/Carrier layering | OK | Zero `neko_session` references in `crates/neko-carrier/src/`; `neko-session` is a **dev-dependency only** in `crates/neko-carrier/Cargo.toml`; production deps remain `neko-reliable`/`neko-wire`. |
| 8 | Duplicate/conflict propagation | OK | Fixture composes a real bounded `neko_session::SessionRuntime` with an explicitly opened stream and counts Session-layer exact-duplicate suppression; conflicting bytes are retained in `last_session_error` instead of `let _`. New test `session_layer_suppresses_exact_duplicates_and_fails_closed_on_conflict` (first delivery accepted; exact duplicate counted once, not redelivered; conflicting bytes fail closed and never delivered). |
| 9 | Nonce/FrameId separation | OK | Packet number is the authenticated `SecureSession` sequence/AEAD nonce (`:4037`, `:4131`); `FrameId` is the stable retransmit identity. `packet_frames` maps packet→frames (`:4587`, `:4725`) and `apply_ack` looks up real frames, explicitly never reconstructing `FrameId(packet_no)` (`:4626`, `:4628-4631`). |
| 10 | cwnd/pacing honesty | OK | `can_send` is check-only (`:4221-4224`); `pacing_interval_us` is a deterministic function of smoothed RTT (`:4226-4230`). Refused admission records nothing (`:4558-4561`). |
| 11 | ACK vs Session evidence separation | OK | `PathRecovery` deliberately carries no `SessionRuntime`/`DeliveryLedger`/`confirm_received` (`:4032-4043`); packet ACK retires Carrier packet/frame only; Session delivery evidence lives in the composed fixture's Session layer. No `(stream, offset)` map remains in Carrier (`grep` empty). |

## Findings (documentation-only; no behaviour or gate impact)

**[LOW] D1 — orphaned `deliver_logical` doc now documents `on_packet_received`.** `crates/neko-carrier/src/lib.rs:4591-4596` is the doc block of the removed `deliver_logical`, left in place and concatenated onto `on_packet_received` (`:4599`). It re-asserts a Carrier-layer logical delivery/dedup path ("this logical-identity dedup is the delivery path") and names the removed `dedup_suppressed` counter — the exact layering claim H-RUDP-012 removed. It is the only remaining occurrence of `dedup_suppressed` in the tree (`grep -rn dedup_suppressed crates/` → `lib.rs:4594` only). Minimal repair: delete lines `4591-4596` (keep `:4597-4598`, the real `on_packet_received` doc).

**[LOW] D2 — second orphan fragment truncated mid-sentence.** `crates/neko-carrier/src/lib.rs:4541-4543` is another leftover fragment of the `deliver_logical` doc, ending mid-clause ("…a later duplicate carrying the SAME bytes is suppressed and"), prepended to `on_packet_sent`'s real doc (`:4544-4549`). Minimal repair: delete lines `4541-4543`.

**[LOW] D3 — stale send-time loss formula in the raw-counter doc.** `crates/neko-carrier/src/lib.rs:4258-4259` states "Loss-per-mille is `packets_lost*1000/packets_sent`" above `packets_sent()`/`packets_lost()` (`:4260-4265`). That is the send-time cumulative formula H-RUDP-001D identified as the defect; the manager health path uses the resolved interval delta (`:4289-4293`). The stated formula appears nowhere in code (`grep -rn "packets_lost \* 1000" crates/` → none). Minimal repair: state that these are raw lifetime counters for diagnostics and that manager health uses `fresh_health_sample()` deltas.

## Reproduced evidence at `e0729d9` / `dcad89a`

- `cargo test -p neko-carrier --all-targets` → carrier lib **69 passed, 0 failed**, plus all integration targets green (2+1+2+5+3+1+5+2+5+2+1+2+3+2).
- `cargo clippy -p neko-carrier --all-targets --locked -- -D warnings` → clean (neko-carrier compiled, no warnings).
- Policy checks: `check-markdown-links.sh` (15 links), `check-release-boundaries.sh`, `check-era4-closure.py` (0 open-ready / 16 already-sufficient / 0 dependency-blocked), `check-status-evidence.sh`, `check-plan-sync.sh` → all pass.
- No decoder/framing change in this commit, so no fuzz rerun was required or performed; the parent does not claim one either.

## Boundary / not claimed

This review is bounded to the Q7 areas above on exact `e0729d9`. It does **not** accept R7, does not cover the full ten-scenario coherent fixture (Q4 remains partial — only Session composition and the conflict flow were added), does not verify Q5 beyond noting the recorded supersede classification, and does not cover R8/R9. No release flag is changed: `RELEASE_CANDIDATE=false`, `PRODUCTION_READY=false`, `FREEZE=false`, `RELEASED=false`; `READY_LIVE: none`.
