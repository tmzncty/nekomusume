# Independent bounded verification — Q4-F coherent-path test is non-discriminating at exact `b5cc771`

**Reviewed revision:** `b5cc771e3b075c9426ae2007303f4e1104daffdb` (`test(carrier): prove the PTO-sample invariant inside the coherent R7 path (Q4-F)`).
**Scope:** the new Q4-F coherent-path scenario and the two fixture-hardening repairs. Read-only in the repository; verification used a detached scratch worktree under `/tmp`. No repository source was edited.
**Date:** 2026-09-13 (UTC `2026-09-13T15:41:50Z`). **Not** an audit or release authorization.

## Verdict

- **Fixture hardening (both nit repairs): CORRECT and verified.**
- **Q4-F coherent-path test: implemented as adjudicated, but currently NON-DISCRIMINATING — it passes with the H-RUDP-001D defect restored.** One bounded assertion defect; exact reproducer and minimal fix below. Reported before editing.

## Finding — Q4-F test does not actually detect H-RUDP-001D

**[MEDIUM, test-strength only — no product or evidence claim is invalidated]** — `crates/neko-carrier/tests/reliable_udp_runtime.rs:897-974`.

The test claims to prove that a PTO-only sample cannot consume the loss epoch. It does not: with the defect reintroduced, it still passes. The assertion is only `degraded` (`:971-974`); the `bad_intervals` counter it maintains is never asserted.

**Exact reproducer** (faithful minimal revert of only the health math in `crates/neko-carrier/src/lib.rs::fresh_health_sample`, in a scratch worktree at `b5cc771`):

```
delta_sent = resolved_packets - last_health_sent   ->  packets_sent - last_health_sent
delta_lost = resolved_lost - last_health_lost      ->  packets_lost - last_health_lost
last_health_sent = resolved_packets                ->  packets_sent
last_health_lost = resolved_lost                   ->  packets_lost
```

Verification matrix (all runs on the same scratch tree, `--test reliable_udp_runtime`):

| Test variant | Old (defective) send-time denominator | Fixed (resolved-outcome) denominator |
|---|---|---|
| loop `0..6` — **as committed** | **PASS** (vacuous) | PASS |
| loop `0..2` — isolation experiment | **FAIL** (`bad_intervals=2`, not `Degraded`) | **PASS** |

**Control proving the revert is faithful:** the same revert makes the `901991d` unit-seam test fail exactly as H-RUDP-001D describes —
`pto_only_health_sample_cannot_erase_a_later_resolved_loss` → `assertion left == right failed: resolved denominator must be 1 acked + 1 lost / left: 0 / right: 500`.

**Root cause.** Under the defect, the post-PTO loss interval is erased to `0/mille` → `Progress`, so `consecutive_bad` does not increment on that interval. But the loop keeps running: each later round sends six fresh records, which refill the *send-time* denominator, so `consecutive_bad` still accumulates on rounds 1-2 and `Degraded` is reached inside the 6-round budget. The test therefore tolerates the very interval the defect corrupts. The `Idle` guard (`:952-955`) only catches the *epoch-not-advanced* variant, never the *epoch-advanced-but-0/mille* variant that is the actual defect. With the fix the interval is `5 lost / 6 resolved = 833/mille` → `Failure`, so degradation occurs one round earlier — which is exactly the signal the assertion ignores.

**Minimal fix (one line, verified both ways).** Bound the loop to the two rounds the invariant needs:

```
for round in 0..2u64 {      // was 0..6u64
```

Verified: **FAIL** under the defect (`bad_intervals=2`), **PASS** with the fix. Two equivalent alternatives if a longer loop is preferred for realism: (a) assert the round index at which `Degraded` is first observed is exactly `1` (the second consecutive bad interval) rather than accepting any round within the budget; or (b) assert that round 0's post-ACK sample is a bad observation (e.g. `bad_intervals >= 1` after round 0, or that the returned state is not `Healthy`), since under the defect that interval reports `Progress`.

Recommend the developer's own unused `bad_intervals` counter be asserted regardless — it was already being computed, which suggests a stronger assertion was intended.

## Fixture hardening — verified correct

- `note_frame_offset` (`:155-165`) records `FrameId -> Session byte offset` and fails loudly on re-binding one frame to a different offset, closing the silent-overwrite nit. Used by the send path (`:208`) so every frame built through `send_data_frame` is recorded.
- The raw-API dropped packet in the existing event-loop scenario now records the Session identity its frame would carry (`:447-452`, `dropped_frame = FrameId(9001)` + `note_frame_offset(dropped_frame, client.next_offset)`), so the `expect` added in `pto_retransmit` can no longer panic on that frame. This closes the second nit exactly as recommended.

## Reproduced state at `b5cc771`

- `cargo test -p neko-carrier --test reliable_udp_runtime` → **10 passed**, 0 failed (includes the new scenario, which passes — consistent with the finding).
- `cargo test -p neko-carrier --all-targets` → all **15** targets `ok`.
- `cargo clippy -p neko-carrier --all-targets --locked` → warning-free.
- Production state unchanged and clean: `[dependencies]` = `neko-reliable` + `neko-wire`; no `neko_session` / `delivered` / `deliver_logical` / `dedup_suppressed` in `crates/neko-carrier/src/`; no `Cargo.toml`/`Cargo.lock` drift. The reported concurrent-editor rework remains absent from history.
- Repository worktree clean; the scratch verification worktree was removed.

## Boundary / not claimed

This is a bounded verification of one test and two fixture repairs. It does **not** accept R7 or close Q4. The underlying H-RUDP-001D **production** fix remains correct and independently verified (`901991d` and the earlier Q7 review); only the new coherent-path *test's* discriminating power is at issue. Still open: the Q4-F assertion fix above, Q4-E plaintext side at the unit seam, the Q4 closure package, Q8/R8, Q9/R9, Q10-Q12, `SessionRuntime.events`, D019, RSEC-001, signing/key-custody/SBOM, previous-frozen-release interoperability, item 3, item 4, and release authority. Release flags unchanged: `RELEASE_CANDIDATE=false`, `PRODUCTION_READY=false`, `FREEZE=false`, `RELEASED=false`; `READY_LIVE: none`.
