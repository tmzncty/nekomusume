# Local exact-tree gate — `8bd87e3` (R9-2E bounded authenticated demux owner)

Lane: reviewer handoff `5d2d188` (`docs(handoff): keep R9-3 blocked on authenticated demux closure`),
repairing **H-R9-001** and the demux half of **M-R9-004**.

## What this tree changes

`crates/neko-cli/src/main.rs` only. No wire grammar, Session/Carrier layering, crypto
architecture or policy value changed, and no dependency changed.

The serial single-`expected` receive shape for reliable mode is replaced by one bounded demux
owner (`recv_udp_delivery_ack`) over the outstanding reliable-owned logical records. Every
successfully authenticated plaintext is classified exactly once:

1. **Session DeliveryAck** matching any outstanding record is consumed as that record's
   confirmation, order-independently, and retired from the outstanding set. The caller applies
   `SessionRuntime::delivery_ack` for exactly the record that was acknowledged, so a later
   record's confirmation arriving before an earlier one is no longer swallowed.
2. **Canonical Carrier packet ACK** is applied to recovery and reported as a typed
   `applied`/`rejected` outcome. A rejection never mutated recovery and is no longer discarded
   (`let _ =` removed here and in the settlement loop).
3. **Any other authenticated plaintext** consumes the finite malformed/ignored budget in reliable
   mode too, with a typed reason. Stale/duplicate/unexpected logical ACKs are typed bounded
   negatives rather than matches, so a peer cannot make the owner spin unboundedly.

The post-return (migration-back) path goes through the same single owner and accepts only a real
Session DeliveryAck for its own record.

## Discriminating regressions

Both are unit tests over the real receive owner with real loopback UDP sockets and an
authenticated `SecureSession` pair:

- `cli_regression_tests::reliable_demux_matches_a_later_record_ack_while_an_earlier_one_is_outstanding`
  places only record 1's Session DeliveryAck on the wire while records 0 and 1 are both
  outstanding, and requires record 1 to be matched and retired while record 0 stays outstanding.
  Verified discriminating: with the matching predicate temporarily restricted to the serial
  expectation (offset 0), the test **fails** with `left: 0 / right: 16`; unmodified it passes.
- `cli_regression_tests::reliable_demux_bounds_an_authenticated_unexpected_logical_ack` proves an
  authenticated but unmatched logical ACK is a typed bounded negative (`unexpected_logical_ack`)
  and never retires a record.

## Exact-tree gate

Executed on a clean isolated worktree at exact
`dffdda2c91e11b7526dbaf2e2e58182ee99beae3`, whose
`crates/neko-cli/src/main.rs` blob (`09814d590b48d1bca71bd5cdf9832f86aa166edc`) is byte-identical
to the pushed tree `8bd87e388745ae175f8b188f13936f1c1fe86ea4`:

- `cargo test -q -p neko-cli --locked`: exit `0` (bin 42, multistream 6, probe 43)
- `cargo clippy -q --workspace --all-targets --locked -- -D warnings`: exit `0`
- `PYTHONDONTWRITEBYTECODE=1 bash scripts/check.sh`: exit `0`
- `git diff --check`: exit `0`
- clean initial/final tree: `true` / empty
- UTC `2026-09-14T02:23:31Z` -> `2026-09-14T02:26:22Z`
- Linux x86_64; rustc `1.98.0`

## Concurrent-editor note

A concurrent editor had left an uncommitted, partial version of this repair in the shared
worktree (its diff is preserved at
`/tmp/neko-concurrent-r9-2e-partial-20260914T020649Z.patch`). Its order-independent matching idea
was adopted, but that draft was **not** correct as left: it returned only the datagram length, so
the caller still applied `delivery_ack` for record 0 regardless of which record had actually been
acknowledged, and it left the Carrier-ACK branch still unconditionally `continue`-ing while
bypassing the malformed budget. This commit completes and corrects the repair, and the shared
worktree was left untouched.

## Provenance

The gate was measured on `dffdda2` (code-only tree). `8bd87e3` is the same change rebased onto
the then-current `origin/main` after a docs-only advance; its `crates/neko-cli/src/main.rs` blob
is byte-identical to the measured tree, so the measurement applies unchanged. A docs-only
descendant records the reconciliation.

## Not claimed

R9-3 and later R9 slices; the migration-back reserved-record process regression from R9-2F; R8
acceptance beyond the bounded local review; Q4-E plaintext side; standalone Q4-B;
`SessionRuntime.events`; D019; RSEC-001; signing/key-custody/SBOM; previous-frozen-release
interoperability; item 3; item 4; release authority. Flags unchanged:
`RELEASE_CANDIDATE=false`, `PRODUCTION_READY=false`, `FREEZE=false`, `RELEASED=false`,
`READY_LIVE: none`.
