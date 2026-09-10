# Local queued Session context boundary — `531e0bc`

Developer-run bounded review and exact-tree validation of reachable implementation/test commit `531e0bce50ad11f82e51ba4fb2ed9b36729206f3`.

## Adjudicated candidate rule

`Unsent -> InFlight` is a local queue/send-state transition and does not itself claim delivery, authentication under a newer key phase, or rebinding to a newer path. An `Unsent` segment therefore retains its assignment-time `SessionContext` when it becomes `InFlight`, even if a disjoint operation has advanced the ledger-global key/path context.

The focused executable scenario proves:

- segment A is inserted at `(delivery_epoch=1,key_phase=0,path_generation=1)`;
- disjoint segment B advances ledger-global context to `(1,1,2)`;
- `mark_in_flight(A)` preserves A's assignment context, leaves the global context at `(1,1,2)`, and does not advance the watermark;
- only `confirm_received(A, (1,1,2))` applies the explicit newer confirmation evidence to both A and the ledger and advances A's watermark.

This is safe within the current bounded model because `mark_in_flight` has no external evidence/context input, packet/TCP feedback cannot confirm logical delivery, stale duplicate insertion remains globally fail closed, and `confirm_received` performs both segment-local and ledger-global monotonic validation before mutation. Real callers also assign and send immediately under one context; no caller uses this method to claim key/path migration.

The bounded pass over `insert -> mark_in_flight -> mark_uncertain -> confirm_received -> watermark/limits` found no remaining concrete contradiction with the written candidate evidence contract. This closes the current C-row question, not all imaginable Session semantics.

## Verification

Focused checks before commit:

- `cargo test -p neko-session --all-targets`: passed (`41` unit tests plus readiness seed test)
- `cargo test -p neko-carrier --test integration_gates`: passed (`5`)
- `cargo test -p neko-carrier --test m3_long_migration`: passed (`3`)
- `git diff --check`: passed

Exact-tree gate:

- `PYTHONDONTWRITEBYTECODE=1 bash scripts/check.sh`: exit `0`
- `git diff --check`: exit `0`
- gate: `2026-09-10T18:14:00Z` -> `2026-09-10T18:15:56Z`
- host: `Linux 6.8.0-137-generic x86_64`
- Rust: `rustc 1.98.0 (88d9e12ae 2026-08-18)`
- initial/final source tree: clean

The final classification/documentation tree `ae0f41d941f4bf1a4180c349623796fc4d1b9007` was also validated from a clean exact worktree with `PYTHONDONTWRITEBYTECODE=1 bash scripts/check.sh` and `git diff --check`, both exit `0`, at `2026-09-10T18:18:21Z` -> `2026-09-10T18:20:17Z` on the same host/Rust toolchain; initial/final source tree was clean.

This is developer-local bounded candidate Session evidence, not protocol freeze, complete Session validation, interoperability, independent review, RC, release, public-listener, or production authorization.
