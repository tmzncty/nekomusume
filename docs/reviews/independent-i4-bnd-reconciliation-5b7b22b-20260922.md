# Developer bounded independent algorithmic/resource boundedness reconciliation

**Anchor:** exact source/test tree `5b7b22b` + commits through `75f4d9c`.
**Owners inspected:** `crates/neko-cli/tests/probe.rs` channel/lifetime surface (`wait_for_ready_marker` `sync_channel(1)`, `malformed_classification_barrier` `sync_channel(64→1)`, `recv_timeout`/`deadline` bounded waits, `bounded_reap_or_kill`, `startup_log`/`collected` String accumulation, `reader_handle` join); `crates/neko-cli/src/main.rs` `SessionRuntime`/`neko-carrier`/`neko-reliable` queue/counter surfaces re-read via prior `dev-i4-bnd`/`independent-i4-bnd-recovery`/`dev-i4-bnd-synthesis` owner-diff.

## Findings + repair

- **Repaired at `5b7b22b`:** `malformed_classification_barrier`'s reader channel was `sync_channel(64)` — an unexplained buffer, not derived from the needed `attempts` count or the repository's established one-item handoff pattern. Changed to `sync_channel(1)` so the reader sends each line only when the barrier is ready to consume it. Seven focused malformed/udp_listener tests pass; clippy clean.
- **No defect in remaining challenged surface:** `wait_for_ready_marker` uses `sync_channel(1)` + `recv_timeout(timeout)`; `malformed_classification_barrier` uses `recv_timeout(remain)` with an explicit deadline and kills+reaps on expiry/EOF/disconnect; `startup_log`/`collected` accumulate bounded stdout lines up to the ready marker or `attempts` events; `reader_handle` join is deferred only until the caller has produced more output (documented causal bound, not unbounded growth).
- **Runtime queue/counter surfaces unchanged:** `SessionRuntime`/`neko-carrier`/`neko-reliable` committed caps (`max_queue_records` aggregate send+recv, MemoryCarrier empty-message record slot, Recovery boundedness) remain covered by dedicated independent reviews — no owner materially changed in this interval. `SessionRuntime.events` retained-history capacity remains a maintainer/security policy gate; no value chosen.

## Commands/tests actually run

- `cargo test -p neko-cli --test probe` — malformed/udp_listener tests pass.
- `cargo clippy -p neko-cli --all-targets -- -D warnings` — clean.
- `PYTHONDONTWRITEBYTECODE=1 bash scripts/check.sh` at `5b7b22b` — exit 0 (`docs/notes/i4-bnd-provenance-5b7b22b-20260922.md`).

## Exclusions

- No capacity-pressure benchmark; no new TTL/LRU/history/capacity/security value.
- No fuzz — no decoder/parser/crypto framing owner changed.

## Result

One concrete bound defect repaired (`sync_channel(64)→(1)`); no remaining
algorithmic/resource boundedness defect in the challenged scope.

**READY_LIVE: none** — deterministic local evidence only.
