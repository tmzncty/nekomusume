# Independent observability stable-v1 re-close — exact `8e11de0`

Bounded independent re-review of the corrected `crates/neko-observe/src/lib.rs` event-buffer path plus the append-only `schema/observability-event.v1.json` adjustment, at reachable exact `8e11de0e5915efa43591069172b97f311aae2c6e`, against `docs/era4-observability-contract.md` and the closed stable-v1 schema. This re-closes the previously open O2/O3 and the follow-up O4/O5/O6 evidence-contract findings. Not a WAN/live claim, not Session delivery evidence, not an independent security/release approval.

## O4 — schema-valid emitted shapes — CLOSED

- `resource.limit_hit` for sequence exhaustion now emits `resource:"event_sequence"` (added append-only to the existing stable-v1 `resource` enum, which the contract explicitly permits) with `limit` as the integer counter `u64::MAX` (`18446744073709551615`) and numeric `observed`. The earlier `"u64_max"` string and non-enum resource value are gone.
- `diagnostic.events_dropped` now emits only the contracted `dropped_total` + `oldest_sequence`; the non-schema `dropped_since_last` field was removed.
- `additionalProperties=false` was not weakened; the only schema edit is the append-only enum value.
- Regressions `sequence_exhaustion_emits_limit_hit_and_never_duplicates` asserts the integer `limit` and `event_sequence` resource, and `ring_overflow_emits_events_dropped_with_floor` asserts no `dropped_since_last`.
- `scripts/check-observability-contract.sh` and `check-observability-contract-test.sh` both pass on the corrected schema.

## O5 — drop diagnostic displacement counted, no recursion — CLOSED

- `push_inner` now counts every retained event it evicts (including the retained event displaced to make room for the diagnostic) into `dropped_total` — the displaced ordinary evidence is genuinely lost, so the count is truthful.
- The diagnostic never counts itself and never re-reports: `flush_drop_report` clears `drops_since_report` before pushing, and the diagnostic append cannot set it again for itself.
- `dropped_total` therefore equals the actual count of ordinary retained evidence no longer retained due to overflow (`switch_datagram_scheduler_and_eviction_are_observed` now asserts `2` for the documented capacity-5 burst: one real eviction plus one diagnostic displacement).

## O6 — post-coalescing retained floor — CLOSED

- `flush_drop_report` now pushes the diagnostic first, then writes its `data` from `self.oldest_sequence()` computed **after** the diagnostic is in the ring, so `oldest_sequence` is the true retained floor including the diagnostic, not the pre-insertion floor. The `ring_overflow_emits_events_dropped_with_floor` regression compares the field against the actual `events().next().sequence`.

## Carried-over correctness (re-verified on this exact tree)

- Strictly-increasing `sequence`; `u64::MAX` is consumed once by the single `resource.limit_hit` marker and never reused.
- Bounded emission: per-kind emission is capped at `capacity`, so an arbitrarily large `u64` counter delta cannot drive unbounded work (`datagram_huge_delta_emits_bounded_events`).
- Mixed queue-full/terminal datagram attribution (`datagram_mixed_drop_window_classifies_queue_and_terminal_separately`) and inconsistent-counter clamping (`datagram_inconsistent_queue_delta_is_bounded_conservatively`) remain correct.
- No second diagnostic queue or generalized logging subsystem was introduced; the single ring remains the only retained store.

## Evidence

- `cargo test -p neko-observe` on exact `8e11de0`: 7 lib tests, all passed.
- Exact-tree gate on `8e11de0`: `PYTHONDONTWRITEBYTECODE=1 bash scripts/check.sh` exit 0, `git diff --check` exit 0, clean tree (UTC start `2026-09-13T00:06:52Z`, end `2026-09-13T00:08:48Z`, Linux 6.8.0-137-generic x86_64, rustc 1.98.0).
- `scripts/check-observability-contract.sh` + `-test.sh`: pass.

O2/O3/O4/O5/O6 are closed on this reachable anchor. No new `READY_LIVE` question; release/governance state unchanged.
