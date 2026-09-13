# Independent bounded R7 challenge and Q4 closure review — exact `9251427` (docs head `403886b`)

**Reviewed revision:** source/test `92514277ebf6874428c365a53435833b3eb49f3e`; docs head `403886b40be19f0526e91c09fa9690309694afef` (both reachable; later commits are docs-only).
**Reviewed artifacts:** `crates/neko-carrier/tests/reliable_udp_runtime.rs` (sha256 `c266da93df2ea83c95495ae3a6827fc039e3be07a1a4abf1a3ec43a23f27d2e4`), `crates/neko-carrier/src/lib.rs` production portion (up to `#[cfg(test)]`, sha256 `dcfa66b60ec34ed6e29619254624f1adcaa8a3aab98277b1d960fa3a3fce95fc`).
**Date:** 2026-09-13 (UTC `2026-09-13T15:49:24Z`). Read-only in the repository; verification used detached scratch worktrees under `/tmp`, removed afterwards.

## Verdict

- **Q4-F non-vacuity fix: VERIFIED in both directions** (independent reproduction, not just self-report).
- **Independent bounded R7 challenge: no concrete defect found** across the six required areas.
- **Recommendation: Q4-F is closed; Q4 may close and R7 may be accepted for the bounded local scope**, with the two disclosed residuals and the standing exclusions below. One LOW test-quality nit remains (non-blocking).

## 1. Q4-F fix — independently reproduced, both directions

`9251427` applies the loop bound (0..2, `:913`) and adds the `bad_intervals >= 1` assertion (`:977`). I reproduced the discrimination myself in a scratch worktree:

| Variant | Q4-F scenario | `901991d` unit control |
|---|---|---|
| `9251427` as committed | **PASS** | PASS |
| only `fresh_health_sample` reverted to the send-time denominator | **FAIL** (`bad_intervals=2`) | **FAIL** (`resolved denominator must be 1 acked + 1 lost / left: 0 / right: 500`) |

The control failing under the same revert proves the revert is faithful (the defect is genuinely restored), so the PASS/FAIL split is a real discrimination and not an artifact of an arbitrary edit. This is the negative control the adjudication asked for.

## 2. Finding (LOW, test quality only — non-blocking)

**[LOW] The newly added `bad_intervals >= 1` assertion is inert and its message is inverted.**
`reliable_udp_runtime.rs:964` increments `bad_intervals` in the branch where the state is **not** `Degraded`/`Failed` — i.e. it counts *non-degraded* samples, not bad ones. Verified consequence: under the restored defect `bad_intervals == 2`, so `bad_intervals >= 1` **passes**, and the test only fails on the subsequent `degraded` assertion (`:981-984`). So the discriminating power of this test comes **solely from the loop bound `0..2`**; the added assertion does not contribute, and its message ("the first resolved-loss interval must be observed as bad") does not match what it computes. The recommended "assert round 0's post-ACK sample is a bad observation" was therefore implemented with inverted semantics and at the wrong place (after the loop rather than after round 0).

*Minimal repair (optional):* increment inside the `Degraded | Failed` branch (making the name honest), or replace the assertion with an explicit first-interval check — e.g. assert round 0's post-ACK poll returns a state that is not `Healthy`/`Unknown` — and keep it before the loop ends. Nothing about the product or the current discrimination depends on this; it is a readability/robustness improvement. It is also latently brittle: with `degrade_after = 1` (or if round 0's interval were ever immediately degrading) the current assertion would fail while the behaviour is correct.

## 3. Independent bounded R7 challenge — six areas

Production code is **byte-identical** to the previously verified `e0729d9` repair (`dcfa66b6…` unchanged across `e0729d9`, `252a59d`, `901991d`, `9251427`, `403886b`); every change since is comment-, test-, or docs-only. I re-challenged each area against the current tree:

| Area | Result | Basis |
|---|---|---|
| Transactional owner | OK | cwnd check -> reserve bounded plaintext -> `recovery.on_sent` -> only then `packet_frames.insert`; typed `Retransmit(RetransmitError)`; rollback only of a copy newly reserved; `on_retransmit_sent` fails before recording without ownership; bounded `RetransmitBuffer` (64 frames / 8192 bytes). |
| Retained-state bounds | OK | Frame/byte capacity and oversize checks before insert with saturating arithmetic; `teardown()` clears map + owner together. |
| Resolved-health math | OK | `outcome_epoch` advances only on resolved outcomes; `fresh_health_sample` uses the resolved interval delta with `checked_div` fallback; diagnostics counters explicitly marked as not the health denominator. |
| Session/Carrier layering | OK | Zero `neko_session` in production `src/`; `[dependencies]` = `neko-reliable` + `neko-wire`; `neko-session` dev-only; `cargo tree -e normal` shows no `neko-session`. |
| Duplicate/conflict propagation | OK | Fixture composes a real bounded `SessionRuntime`; exact duplicates counted, conflicting bytes retained in `last_session_error`, never swallowed. |
| Nonce/FrameId separation | OK | Packet number = authenticated sequence/AEAD nonce; `FrameId` stable; `apply_ack` resolves real frames, never `FrameId(packet_no)`; fixture now records `FrameId -> byte offset` and fails loudly on re-bind. |
| cwnd/pacing | OK | `can_send` check-only; `pacing_interval_us` deterministic from smoothed RTT; refusal records nothing and consumes no Session offset. |
| Truthful fallback result | OK | `WarmFallback` only after real switch; failure returns `FallbackFailed`; invalid-standby scenario asserts no switch **and** that a `FallbackFailed` was actually attempted (non-vacuous). |
| ACK vs Session evidence separation | OK | `PathRecovery` carries no Session owner; fixture asserts Carrier and Session counters independently; ACK does not promote Session delivery. |

**Q4 coverage map** (vendor item -> scenario): A -> `scenario_no_loss_round_trip…`; C -> `scenario_lost_ack_recovers_via_pto_replacement…`; D -> `scenario_replacement_delivered_first_suppresses_the_late_original`; E(cwnd) -> `scenario_cwnd_refusal_consumes_no_session_byte_offset`; E(plaintext) -> `scenario_plaintext_bound_refusal_is_atomic_and_typed` (unit seam); F -> `scenario_pto_only_sample…`; G -> `scenario_clean_recovery_after_old_loss_does_not_replay_it`; H -> `runtime_event_loop_recovers_loss_and_drives_degradation_only_on_new_evidence`; I -> `scenario_invalid_standby_never_produces_a_false_switch`; conflict -> `session_layer_suppresses_exact_duplicates_and_fails_closed_on_conflict`.

## 4. Residuals disclosed for the closure decision

1. **Q4-E plaintext side** is proven at the unit/`ReliableUdpRuntime` seam, not through the socket composition. Acceptable for closure of a locally-scoped fixture suite; it means the coherent path does not itself drive a plaintext-bound refusal.
2. **Q4-B ("one recoverable packet loss")** has no standalone scenario; it is covered jointly by `scenario_replacement_delivered_first_suppresses_the_late_original` (the original is dropped; the retransmission delivers). If the vendor item requires a dedicated data-loss (not ACK-loss) scenario, that is the one gap to consider; it is not a correctness defect.

## 5. Reproduced at `9251427` / `403886b`

- `cargo test -p neko-carrier --all-targets` -> lib **70 passed**; fixture **10 passed**; all 15 targets `ok`.
- `cargo clippy -p neko-carrier --all-targets --locked` -> warning-free.
- Repository worktree clean; scratch verification worktrees removed.
- Concurrent-editor anomaly still absent: production deps carrier-local, no `neko_session` / `delivered` / `deliver_logical` / `dedup_suppressed` in `crates/neko-carrier/src/`, no `Cargo.toml`/`Cargo.lock` drift.

## 6. Exclusions / not claimed

This is a bounded independent review of the local R7 repair and fixture suite. It is **not** an independent maintainer/security approval, cryptanalysis, adversarial-load or capacity-suitability assessment, WAN/VPS evidence, or a release authorization. Still open and unchanged: R8/Q8, R9/Q9, Q10-Q12, `SessionRuntime.events` retained-state policy, D019, RSEC-001 suitability, signing/key-custody/SBOM, previous-frozen-release interoperability, item 3, item 4, and RC/freeze/release/production authority. Release flags: `RELEASE_CANDIDATE=false`, `PRODUCTION_READY=false`, `FREEZE=false`, `RELEASED=false`; `READY_LIVE: none`.
