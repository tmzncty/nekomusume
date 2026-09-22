# H-I4-092 exact-tree provenance

**Exact pushed source/test SHA:** `28e1caa6a68b7012ed7126e5e448608843ba7a6f`
(`test(cli): H-I4-092 BarrierProof — barrier-derived token for post snapshot`).

`malformed_classification_barrier` returns `BarrierProof` only on `Ok`; the
post-churn `gated_resource_snapshot` destructures it inside the gate, so a
refactor moving the post snapshot above barrier success cannot compile. The
pre-churn edge keeps the `!churn_started` assert inside the same helper call
as the measurement.

## Gate record

- worktree: clean `git worktree` at exact `28e1caa`
- `PYTHONDONTWRITEBYTECODE=1 bash scripts/check.sh` — **exit 0**
- `git diff --check` — **exit 0**
- `git status --short` — clean
- UTC start 2026-09-22T02:50:44Z → UTC end 2026-09-22T02:56:35Z
- OS/arch: Linux x86_64
- Rust: rustc 1.98.0 (88d9e12ae 2026-08-18), stable

This is developer-reported local provenance, not reviewer-local execution or
hosted CI. No decoder/parser/crypto framing change — no fuzz required. No
release flag, D019, or policy value touched. `READY_LIVE: none`.
