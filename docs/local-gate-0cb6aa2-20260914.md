# Local exact-tree gate — `0cb6aa2` (R8 LOW follow-up repair)

Lane: the independent R8 re-review (`docs/reviews/independent-r8-executable-repair-8c9f584-20260914.md`)
and its verified LOW follow-up (`docs/reviews/independent-r8-low-followup-3795326-20260914.md`).

## What this tree changes

`crates/neko-cli/src/main.rs` and `crates/neko-cli/tests/probe.rs` only. No production
Carrier/Session/crypto/wire semantics changed and no dependency changed.

- **LOW 1 — the H-R8-005 failure path now has a regression.** The declared invariant is that
  `ok=true` is unreachable with unfinished delivery, but it was only reachable ad hoc. The new
  built-binary test forces the settlement bound to expire (`--drop-ack-every 1 --settle-ms 1`) and
  pins the truthful outcome: `ok=false`, `settled=false`, `partial=true`, exit code `1`, with
  `"ok":true` asserted absent. Focused lab tests went from 2 to 3.
- **LOW 2 — the settlement verdict is bound to the authoritative view.** `settled` is now
  `outstanding.is_empty() && remaining_in_flight == 0`, and `partial` derives from that plus the
  existing partial flag. Measured behaviour is unchanged (the lab map and the runtime view agreed
  in every run), so this is not a behaviour fix; it removes reliance on lab-local bookkeeping.

## LOW 3 — retained, with a corrected reason

The R8 LOW follow-up reviewer checked the premise and found my stated reason for declining the
change **factually inverted**. `SessionRuntime` holds `events: Vec<RuntimeEvent>` created with
`Vec::new()` and only ever appended (`self.events.push(...)`); there is no `truncate`/`drain`/
`retain`/`clear`/`split_off` and `RuntimeLimits` has no events-capacity field. The buffer is
therefore **unbounded**, not bounded, so the undercount risk I cited ("a bounded buffer would
silently stop reporting duplicates") does not exist.

The decision to keep the inference nevertheless stands, for the actual reason: because the buffer
is unbounded, reading `DuplicateDedup` from it would couple the duplicate counter to unbounded
retained state whose retention policy is exactly the already-open `SessionRuntime.events` gate. The
current inference is exact — `receive()` has precisely two `Ok` paths (`DuplicateDedup` with no
delivery, and `DataReceived` which queues), and an offset gap hard-errors as `Protocol`, so it
surfaces as a conflict rather than a duplicate — and it needs no unbounded buffer. A durable fix
(a small explicit bounded duplicate counter, or resolving the events policy) belongs to the
`SessionRuntime.events` gate, not to this nit.

## LOW 4 — no action

Every lab frame is offset-derived by construction (`FrameId(frame_offset)`), which is exactly the
invariant the carrier fixture's explicit `FrameId` -> offset map exists to protect. No lab path
builds a non-offset frame.

## Exact-tree gate

Executed on a clean isolated worktree at exact `0cb6aa2ddfbd8b218cfe01d98a2f1e934d96fd95`:

- `cargo test -q -p neko-cli --test probe --locked`: exit `0` (42 passed, including the 3 lab tests)
- `cargo clippy -q --workspace --all-targets --locked -- -D warnings`: exit `0`
- `PYTHONDONTWRITEBYTECODE=1 bash scripts/check.sh`: exit `0`
- `git diff --check`: exit `0`
- clean initial/final tree: `true` / empty
- UTC `2026-09-13T18:22:32Z` -> `2026-09-13T18:25:06Z`
- Linux x86_64; rustc `1.98.0`

No wire decoder, parser or framing grammar changed, so no decode fuzz rerun is claimed or required.

## Provenance correction

The identical repair was first committed as `3795326`, which was measured green in a private
worktree but **never pushed**; the R8 LOW reviewer correctly flagged it as local-only. `0cb6aa2` is
the same change cherry-picked onto the then-current `origin/main` (its code tree is byte-identical:
`git diff --stat 3795326 0cb6aa2 -- crates/ schema/ scripts/` is empty), re-gated on the pushed SHA,
and it is the commit that carries this provenance. `3795326` must not be cited as landed evidence.

## Not claimed

R9, R8 acceptance beyond the bounded local review, Q4-E plaintext side, standalone Q4-B,
`SessionRuntime.events`, D019, RSEC-001, signing/key-custody/SBOM, previous-frozen-release
interoperability, item 3, item 4 and release authority. Flags unchanged:
`RELEASE_CANDIDATE=false`, `PRODUCTION_READY=false`, `FREEZE=false`, `RELEASED=false`,
`READY_LIVE: none`.
