# Independent bounded adjudication — Q4-F coherent-path observability, at exact `8c09425`

**Reviewed revision:** `8c09425208cbee5b8601d05667832a22e0e4c6fd` (`test(carrier): record FrameId to Session offset and never misreport a completed send`), on top of `69e1e57` / `422fafb` / `1a3535b` / `96b43b0`.
**Scope:** the two reported fixture nits, and the Q4-F observability adjudication requested by the developer lane. Read-only; no source edit.
**Date:** 2026-09-13 (UTC `2026-09-13T15:36:12Z`). **Not** an audit or release authorization.

## Part 1 — nit repairs: VERIFIED CORRECT

Both repairs land exactly as recommended, and both choose loud failure over silent fallback:

- **FrameId/offset aliasing removed.** The fixture now records `frame_offsets: BTreeMap<FrameId, u64>` when a frame is first sent (`reliable_udp_runtime.rs:195`) and `pto_retransmit` reads the offset from that map (`:323-326`) instead of assuming `frame.0` is the offset. A missing mapping is `expect("retransmitted frame has a recorded Session byte offset")` — a hard failure, which is the right choice given the failure mode was a *false* Session gap/conflict.
- **No `None` after a completed send.** The post-send offset advance is now `.expect("fixture Session byte offset overflowed u64")` (`:143-147`), so a send that already succeeded can never be read by a caller as "refused, nothing sent".

Reproduced: fixture **9 passed**; `cargo test -p neko-carrier --all-targets` green (lib 70 + all integration targets); `cargo clippy -p neko-carrier --all-targets --locked` warning-free.

## Part 2 — Q4-F adjudication: satisfy it in the coherent path; do NOT add a production accessor

The developer's factual premise is correct but the conclusion is too strong.

**Confirmed fact.** `ReliableUdpRuntime` exposes no raw `HealthSample` / `loss_per_mille`. The public surface is exactly: `new`, `ready_standby`, `activate_udp`, `can_send`, `pacing_interval_us`, `on_packet_sent`, `on_packet_received`, `poll_outgoing_ack`, `apply_ack`, `poll_health` (-> `RuntimeEvent::HealthSample(HealthState)`), `pto_probe`, `on_retransmit_sent`, `teardown`, `manager`, `manager_mut`, `recovery_engine` (-> inner `neko_reliable::Recovery`, not the `PathRecovery` sampler), `in_flight`, `recovery_bytes_in_flight`, `packets_lost`. So there is no fixture-reachable raw sample.

**Why a coherent proof is still possible — the state observable discriminates the defect.** `poll_health` passes the fresh sample through `CarrierHealth::observe`, whose bad test is `sample.pto >= 3 || sample.loss_per_mille >= 500`, and `HealthLimits::default()` has `degrade_after = 2`. A resolved-loss interval with >= 500/mille therefore produces a *state consequence*: two consecutive such intervals reach `Degraded`, which `poll_health` surfaces as `WarmFallback` (ready standby) or `FallbackFailed` (no/!ready standby). Under the removed send-time denominator those same intervals compute `delta_sent == 0 -> 0/mille -> Progress`, so `consecutive_bad` never increments and `Degraded` is unreachable. The state observable therefore **fully discriminates** old vs new behaviour — Q4-F's invariant is testable coherently without new API.

**Evidence the gap is real and not already covered.** The existing `runtime_event_loop_recovers_loss_and_drives_degradation_only_on_new_evidence` reaches `Degraded` via the **`pto_count >= 3`** branch (blackhole PTOs), not via the loss-ratio branch; and its first poll (`:448-452`) asserts only that a `HealthSample` *exists*, never that it is bad. So the coherent path currently never asserts a resolved-loss-driven bad observation.

**Recommended Q4-F test shape (fixture-only, no production change).**
1. Activate UDP with a ready warm TCP standby (so degradation is observable as `WarmFallback`; without a standby it is `FallbackFailed`).
2. Drive one PTO-only interval (`send_data` + `pto_retransmit`, no ACK) and `poll_health` — assert the sample is present and **not** `Degraded`/`Failed` (one PTO keeps `pto_count = 1 < 3`, so this is a loss-based Progress, not a pto-based Failure).
3. With **no intervening send**, deliver an ACK that newly declares resolved loss at >= 500/mille (e.g. send N >= 6 small records, have the receiver read all of them, then ACK only the newest, as `scenario_clean_recovery_after_old_loss_does_not_replay_it` already does). Assert the poll returns a **fresh** sample (not `Idle` -> freshness preserved, i.e. the PTO sample did not consume the loss epoch).
4. Repeat step 3 once more with **no good sample in between** so `consecutive_bad` reaches `degrade_after = 2`, and assert the degradation consequence (`WarmFallback`/`FallbackFailed`).
5. Keep `pto_count < 3` throughout steps 3-4, so the proof is specifically the **loss branch** and cannot be satisfied by the pto-threshold branch.

Practical caveat for whoever implements it: `degrade_after = 2` means a **single** bad resolved interval leaves the state non-degraded; the test must drive two consecutive bad resolved-loss intervals (with no intervening good sample) before asserting degradation. Missing that will look like "the accessor is required" when it is not.

**Why not the optional accessor (option a).** Adding a production accessor purely for test introspection expands the production API surface, which earlier handoffs discouraged. If raw-sample observability is genuinely wanted, **Q10** (observability/release-boundary integration) is the correct home, where it would be a real production need rather than a test hook. Recommendation: **do not add it under Q4-F**.

**Adjudication:** Q4-F is **satisfied by the coherent-path loss-branch scenario above plus the existing `901991d` unit-seam regression**; mark it closed once that scenario exists. No production API change is required or recommended.

## Bounded observation (not a defect; no action required now)

- **[nit] `frame_offsets` is populated only in `send_data_frame` (`:195`).** One other fixture path tracks a frame through the raw runtime API without recording an offset: `reliable_udp_runtime.rs:433` (`client.rt.on_packet_sent(dropped_num, 60_000, 400, FrameId(9001), b"lost")`, deliberately never sent on the wire). If a PTO ever selected that frame, the new `expect` in `pto_retransmit` would panic. It does **not** happen today (the suite is green), and `on_pto` returns the *oldest* outstanding frames (`take(4)` in `FrameId` order), so a high id like `9001` is only selected when fewer than four smaller-id frames remain outstanding — incidental rather than enforced. Because Q4-F/Q4 expansions add PTO cycles and frames, a one-line hardening is worth considering: record the offset for that frame too (a small `note_frame_offset` helper), or route that construction through a helper that records it.
- **[nit] `frame_offsets.insert` silently overwrites** to loud-fail symmetry: if a future test re-sends one `FrameId` under a *different* Session offset, the map would silently keep the last value. Same reasoning the lane applied to the missing-mapping case argues for failing loudly there too. Not reachable in the current suite.

## Not claimed / still open

This adjudication does not accept R7. Still open: **Q4-F** implementation per the shape above, **Q4-E** plaintext side at the unit seam, the Q4 closure package, **Q8/R8**, **Q9/R9**, **Q10-Q12**, `SessionRuntime.events`, D019, RSEC-001, signing/key-custody/SBOM, previous-frozen-release interoperability, item 3, item 4, and release authority. Release flags unchanged: `RELEASE_CANDIDATE=false`, `PRODUCTION_READY=false`, `FREEZE=false`, `RELEASED=false`; `READY_LIVE: none`.
