# Independent R9-11B review — H-R9-081 PathRecovery quiesce freshness

**Severity:** HIGH — release-gate correctness / closure-truth

**Exact reviewed anchor:** `6fe8ae83f3e213e0978bed5cf2e723e739857c0f` (source/test tree is unchanged from developer exact `052b6eab40f8e4257d4480866b21f64ce3e5230c`; `6fe8ae8` adds the developer bounded R9-11B no-finding note).

**Owners inspected:** `crates/neko-carrier/src/lib.rs::{PathRecovery::quiesce, PathRecovery::fresh_health_sample, ReliableUdpRuntime::teardown, ReliableUdpRuntime::poll_health}` plus the current R9-11B handoff invariant and the developer no-finding note.

## Finding

The R9-11B no-finding closure is too strong for the exact current public `PathRecovery` seam.

`PathRecovery::fresh_health_sample()` returns `None` only when:

```text
health_epoch == Some(outcome_epoch)
```

but exact-current `PathRecovery::quiesce()` ends with:

```text
health_epoch = None
```

without advancing `outcome_epoch`. Therefore a direct call sequence

```text
resolved outcome -> quiesce() -> fresh_health_sample()
```

necessarily returns `Some(HealthSample)` on the first post-quiesce poll even though no new resolved ACK/loss/retransmit/PTO outcome occurred after quiesce. If a pre-quiesce resolved delta had not yet been consumed, the sample can also consume/re-project that pre-quiesce delta after the quiescent transition.

That contradicts both the method-level freshness contract ("only when a resolved recovery outcome arrived since the last consumption; polling with no new resolved outcome returns None") and the R9-11B handoff invariant that quiesce must not manufacture a fresh health outcome.

The outer `ReliableUdpRuntime` currently limits impact: `teardown()` sets `torn_down`, and `poll_health()` returns `RuntimeEvent::Idle` before calling the recovery health bridge. This finding therefore does **not** claim that the executable outer runtime currently emits a false fallback/health event after teardown. It is a concrete public `PathRecovery` freshness defect and makes the exact `6fe8ae8` R9-11B no-finding closure overbroad.

## Minimal repair contract

Do not redesign recovery, health, Session/Carrier layering, ACK semantics, or any policy value.

1. Add a focused deterministic `PathRecovery` regression that creates at least one real resolved outcome, calls `quiesce()`, and requires the immediate `fresh_health_sample()` to be `None`.
2. Cover both shapes:
   - the pre-quiesce health outcome was already consumed;
   - a pre-quiesce resolved delta remained unconsumed.
   Neither shape may become fresh merely because `quiesce()` happened.
3. Preserve lifetime diagnostics (`packets_sent`, `packets_lost`, RTT/PTO and existing outcome/resolved history) exactly as the current quiesce contract requires.
4. If the public `PathRecovery` object remains reusable after quiesce, a later strictly post-quiesce resolved outcome may become fresh, but its interval delta must start at the quiescent boundary and must not replay pre-quiesce resolved loss. A minimal implementation shape is to mark the current outcome as consumed at quiesce and align the interval-delta baselines with the current resolved counters; use the smallest exact-current-semantic repair.
5. Run focused `neko-carrier` tests, then on the final pushed developer SHA run the clean exact-tree gate:
   - `PYTHONDONTWRITEBYTECODE=1 bash scripts/check.sh`
   - `git diff --check`
   - clean worktree + exact pushed SHA + UTC start/end + exit codes + OS/arch + stable Rust version.

No decode/parser/crypto framing changes are required, so do not mechanically run fuzz.

## Evidence boundary

GitHub-hosted Rust CI run `35484901555` is green for exact `6fe8ae8`. That only proves the current test suite passes; it does not close this missing direct quiesce/freshness regression. No reviewer-local test execution is claimed in this note.

`READY_LIVE: none`. No live/WAN question is created by this finding.
