# Independent bounded challenge — R-REC-4b deterministic fault simulation accounting

**Date:** 2026-09-24  
**Exact developer source anchor:** `4497752b9a5fc7348d322df9c81dd3659d6d5f3b` (`main` at challenge start)  
**Owner:** `crates/neko-reliable/src/lib.rs` — `simulate_fault_profile`, `simulate_delivery`, `VirtualClock`, `SimulationResult`  
**Classification:** independent source/control-flow challenge; no production protocol change

## Challenged invariant

Deterministic fault simulation must produce exact `sent`/`delivered`/`retransmitted`/`rounds` accounting under no-loss, drop-every, finite burst, reorder and blackhole profiles; round bound must be enforced; invalid-limit/overflow must fail closed; blackhole `delivered = 0` must be reported as failure, not silently reinterpreted as success.

## Source reasoning

`simulate_fault_profile` (line 560): `frames == 0 || frames > 65_536` → `InvalidLimit`; `packet_bytes > 65_535` → `InvalidLimit`; `frame_loss > 90` → `InvalidLimit`; `mss == 0 || packet_bytes > mss` → `InvalidLimit`; `frames_per_packet == 0` → `InvalidLimit`. `delivered` accumulates only when `faults[i]` is false (not lost); `blackhole` profile (`i % 8 >= 4`) produces `delivered = 0` — existing test asserts `(delivered, sent) = (0, 8)`, treating zero delivery as the expected failure shape, not success.

`simulate_delivery` (line 620): `total_frames == 0 || > 100_000` → `InvalidLimit`; `rounds > total_frames + 2` → `Capacity` (round bound); `drop_every` first-attempt loss re-pushes the frame — `delivered` still reaches `total_frames` because loss is retransmission, not permanent. Existing test `deterministic_loss_and_reorder_preserve_all_frames` asserts `delivered == 1000`, `rounds <= 2`, `retransmitted == 1000 / drop_every` across `drop_every ∈ {0,100,20,10}` and `reorder ∈ {false,true}`.

`VirtualClock` (line 530): `advance` uses `checked_add` → `Arithmetic` on overflow; monotonic only.

## Result

No concrete defect found. Blackhole `delivered = 0` is already asserted as the expected failure shape in `fault_profiles_cover_burst_reorder_blackhole_and_clock`; `simulate_delivery` accounting is exact and bounded; invalid-limit/overflow paths are fail-closed. R-REC-4b is a no-finding challenge on the exact-current owner.

## Evidence boundary

Developer-source reasoning plus focused deterministic test review, not reviewer-local execution, hosted CI, WAN evidence or performance conclusion. No decoder/parser/crypto framing change; fuzz not required. No release flag, D019, or policy value touched. `READY_LIVE: none`.
