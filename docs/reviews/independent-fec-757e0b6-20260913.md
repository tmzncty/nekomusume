# Independent bounded XOR FEC candidate review — exact `757e0b6`

Bounded independent review of `FecConfig`/`FecBlock`/`FecError` (`validate`/`encode`/`mark_missing`/`recover_one`/`complete`) in `crates/neko-reliable/src/lib.rs` (`:1194-1295`), against `docs/spec/m2-fec.md`, at reachable exact `757e0b6c056f0bac39c1fb00e24d54cab55f3f72`. FEC remains disabled — this lane does not enable it. Not a WAN claim, not an independent security/release approval.

## Challenged invariants and result

- **Block/symbol/config bounds before allocation/copy — holds.** `encode` runs `config.validate()` (`block_size` 2–32, `symbol_size > 0`, `max_blocks > 0`), rejects `block_id > max_blocks` as `TooManyBlocks`, `symbols.len() != block_size` as `InvalidConfig`, and any `s.len() != symbol_size` as `WrongSymbolSize` — **all before** `vec![0; symbol_size]` allocation or `to_vec` cloning (`:1230-1241` vs `:1243-1251`).
- **Block-id / max-block semantics — holds.** `m2-fec.md` fixes the identity space as `0..=max_blocks` inclusive; `block_id > max_blocks → TooManyBlocks` matches exactly (`fec_block_identity_is_bounded_and_rejection_is_atomic` accepts ids 0 and 3 for `max_blocks=3` and rejects 4).
- **Parity/recovery byte exactness — holds.** Parity is byte-wise XOR over all symbols (`:1243-1247`); `recover_one` reconstructs the single missing symbol as `parity ⊕ ⊕(received others)` (`:1280-1287`). `xor_recovers_single_loss_and_is_reorder_independent` proves byte-exact recovery.
- **Single-loss only; multi-loss fail-closed — holds.** `missing.len() > 1 → Unrecoverable` (`:1276-1277`); no guessed/zero-filled bytes are produced.
- **Duplicate/out-of-range missing — holds.** `mark_missing` rejects `index >= data.len()` as `IndexOutOfRange` and a second mark on the same index as `Duplicate` (`:1257-1261`).
- **Reorder independence — holds.** `received` is a per-index bool bitmap; marking/recovery is order-independent.
- **No FEC→ACK/Session/congestion/enablement evidence — holds.** `FecBlock` is a self-contained recovery candidate producing no Session delivery, ACK, congestion, or enablement signal; `m2-fec.md` keeps it disabled and separate from reliable recovery.

## Evidence

- `cargo test -p neko-reliable` on exact `757e0b6`: `xor_recovers_single_loss_and_is_reorder_independent`, `fec_multi_loss_unrecoverable`, `fec_block_identity_is_bounded_and_rejection_is_atomic`, `fec_mark_missing_bounds` all pass.
- No code change; no defect found. FEC remains a disabled candidate; no new `READY_LIVE` question; release/governance state unchanged.
