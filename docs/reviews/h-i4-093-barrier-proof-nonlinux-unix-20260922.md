# H-I4-093 — BarrierProof binding is not warning-clean on non-Linux Unix

**Severity:** HIGH (release/item-4 test portability/evidence correctness)

**Reviewed exact tree:** `0b15ef3e3769267dcf53cbcb4c50756f6aaa5d3c`

**Developer commits reviewed:**

- `28e1caa6a68b7012ed7126e5e448608843ba7a6f` — H-I4-092 `BarrierProof` source/test repair.
- `0b15ef3e3769267dcf53cbcb4c50756f6aaa5d3c` — handoff/provenance reconciliation.

This is an exact-current static/source review. No reviewer-local Rust gate, non-Linux Unix execution, WAN run, fuzz run, or performance claim is made here. The persisted `28e1caa` gate remains developer-reported Linux x86_64 provenance.

## Accepted from H-I4-092

The Linux causal proof shape is now materially stronger and H-I4-092's original post-barrier proof seam is closed:

- `malformed_classification_barrier(...)` returns `BarrierProof` only on its success path;
- the current Linux post-churn `gated_resource_snapshot(...)` consumes that returned value inside the same helper call that acquires the `/proc` measurement;
- the pre-churn edge still checks `!churn_started` inside the same helper call as the baseline measurement;
- current physical ordering is `READY -> affirmative baseline -> churn starts -> all bounded malformed classifications/cleanup -> affirmative post snapshot`.

The existing bounded timeout/EOF cleanup and post-`preauth.release(...)` diagnostic ordering remain accepted because their owners did not materially change in these commits.

## Concrete defect

`28e1caa` changed the barrier call site to:

```rust
let (child, reader_handle, barrier_proof) =
    malformed_classification_barrier(...)?;
```

The enclosing test is `#[cfg(unix)]`, so this binding exists on Linux, macOS and BSD-like Unix targets. However, `barrier_proof` is read only inside a later `#[cfg(target_os = "linux")]` `/proc` block.

Therefore on `unix && !target_os="linux"` the test still compiles and runs its portable socket/lifecycle path, but the local `barrier_proof` binding is never used. That produces the ordinary Rust `unused_variables` warning. The repository standard gate includes:

```text
cargo clippy --workspace --all-targets --locked -- -D warnings
```

so a non-Linux Unix gate turns that warning into an error. This regresses the earlier H-I4-091 requirement that Linux-only resource-oracle tokens must not make the non-Linux Unix process-test surface warning-dirty.

The Linux exact-tree provenance for `28e1caa` does not contradict this finding: Linux consumes `barrier_proof`, so that platform cannot expose the warning.

This is not a runtime transport defect and does not invalidate the Linux `/proc` resource observation itself. It is a release/item-4 cross-platform test/evidence defect.

## Closure contract

Apply the smallest test-only cfg-safe repair:

1. Preserve the current barrier-derived `BarrierProof` requirement on Linux. Do not replace it with a freely constructible constant/no-op gate and do not weaken H-I4-092's causal ordering.
2. On non-Linux Unix, explicitly consume/sink the returned proof or otherwise structure the destructuring so the binding is warning-clean while the portable socket/lifecycle test continues to run.
3. Preserve H-I4-090/H-I4-091 behavior: baseline before first malformed send, bounded off-thread classification wait, timeout/disconnect/EOF child cleanup, post-release diagnostic ordering, existing FD/RSS margins and `ATTEMPTS`.
4. Do not change Session/Carrier/ACK/wire/crypto semantics or any capacity/TTL/LRU/history/security number.
5. No fuzz is required unless decoder/parser/crypto framing owners change.
6. Run focused CLI process tests, then on the final pushed source/test SHA run `PYTHONDONTWRITEBYTECODE=1 bash scripts/check.sh`, `git diff --check`, and verify a clean tree. Persist exact reachable SHA, UTC start/end, exit codes, OS/arch, Rust stable version and clean-tree state.
7. If a real non-Linux Unix toolchain/runner is available, an all-target clippy/check there is useful cross-evidence; if it is not available, do not fabricate such execution. Keep the Linux local gate and static cfg reasoning classified separately.

## Queue consequence

Front this HIGH before expanding the current review surface. After closure, continue immediately with the retained rolling queue: I4-PORT-RES causal re-challenge; pre-auth malformed/rejection resource accounting; cross-platform CLI/process semantics; diagnostic/machine-output boundary; boundedness; package/build reproducibility spot re-challenge; item-4/release-packet reconciliation; repository-wide 13-surface refill; conditional live only if a materially new real-network question becomes READY.

`READY_LIVE: none`; release items 3/4 remain incomplete; `RELEASE_CANDIDATE=false`, `PRODUCTION_READY=false`, `FREEZE=false`, `RELEASED=false` remain unchanged.
