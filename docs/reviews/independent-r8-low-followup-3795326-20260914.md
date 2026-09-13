# Independent bounded verification — R8 LOW follow-up at `3795326`, and the LOW 3 determination

**Reviewed revision:** `3795326605198dba81b54a781a85d2703ce7a546` (`test(cli): pin the R8 partial-settlement failure path and bind ok to authoritative in-flight state`), branch `work/r8-lab-low-20260914`.
**Provenance caveat:** `3795326` is **local-only at review time** — it is not on `origin/main` (`108d7c0`) and no remote branch carries it (`git ls-remote origin` -> none). This note verifies the commit object; it must not be treated as landed-on-main evidence until pushed.
**Reviewed files:** `crates/neko-cli/src/main.rs` (sha256 `27c8d106d5918e02…`), `crates/neko-cli/tests/probe.rs`. Only these two changed.
**Method:** isolated detached worktree at `3795326` (the shared worktree is detached at `f4aca03` with a concurrent editor's uncommitted edits, so it was not used). Read-only elsewhere.

## Verdict

**Both repairs verified. No defect found.** Additionally, the LOW 3 premise was tested and is **factually inverted** — with a resulting recommendation that keeps the developer's decision but for a different reason.

## LOW 1 — H-R8-005 failure path now pinned: VERIFIED

New test `lab_reliable_udp_reports_partial_and_exits_nonzero_when_delivery_is_unfinished` forces `--drop-ack-every 1 --settle-ms 1` and asserts exit `1`, `"ok":false`, `"settled":false`, `"partial":true`, and the **absence** of `"ok":true`. Reproduced: focused `lab_` tests now **3 passed** (was 2), and the direct run gives `ok=false, settled=false, partial=true, remaining_in_flight=12`, exit 1. The previously untested invariant is now covered end-to-end through the built binary.

## LOW 2 — verdict bound to the authoritative view: VERIFIED, no behaviour change

`settled = st.outstanding.is_empty() && remaining_in_flight == 0` and `partial = st.partial || !settled`. The `partial` local now feeds the JSON and the human line. Measured on all four configurations — behaviour is identical to the pre-change tree:

| Run | ok | settled | partial | delivered | duplicates | remaining | exit |
|---|---|---|---|---|---|---|---|
| clean no-loss | true | true | false | 8 | 0 | 0 | 0 |
| Data loss (`--drop-every 4`) | true | true | false | 8 | 0 | 0 | 0 |
| ACK loss (`--drop-ack-every 3`) | true | true | false | 8 | 3 | 0 | 0 |
| forced partial (`--drop-ack-every 1 --settle-ms 1`) | false | false | true | 8 | 4 | 12 | 1 |

`ok=true` with unfinished delivery remains unreachable, and the verdict no longer rests solely on lab-local bookkeeping. `--test probe` full suite and `clippy -p neko-cli --all-targets -D warnings` / `fmt --check` are clean.

## LOW 3 — determination: the developer's stated reason is inverted; decision still stands

**Fact established.** `SessionRuntime.events` is `events: Vec<RuntimeEvent>` (`crates/neko-session/src/lib.rs:1193`), constructed with `Vec::new()` (`:1233`) and only ever appended (`self.events.push(…)`, `:1613`). There is **no truncation, drain, retain, clear, split_off, or reassignment** anywhere in the crate, and `RuntimeLimits` has no events/capacity field. The buffer is therefore **unbounded** — it retains every event for the runtime's lifetime.

**Consequence for the argument.** The developer declined to read `DuplicateDedup` from `events()` because they "could not establish that the buffer is unbounded" and feared a bounded buffer would silently stop reporting duplicates. Reliably, the opposite holds: the buffer is unbounded, so an event-reading form would **not** undercount today. The undercount risk that motivated the refusal does not exist.

**But the decision should still stand — for the real reason.** Because the buffer is unbounded, coupling the lab's duplicate counter to it would make duplicate accounting depend on unbounded retained state whose retention policy is exactly the **`SessionRuntime.events` gate already listed as open** in the handoff. Keeping the inference avoids introducing a new dependency on an unresolved policy surface, and the inference is currently exact: `receive()` has precisely two `Ok` paths — `DuplicateDedup` (no delivery) and `DataReceived` (queues) — and any offset gap hard-errors as `Protocol`, surfacing as a conflict rather than a duplicate. The in-code comment states that dependency.

**Recommendation:** keep the inference. The durable improvement is not event-reading; it is either a small explicit bounded duplicate/suppression counter on `SessionRuntime`, or resolving the `events` retained-state policy — both belonging to the open `SessionRuntime.events` gate, not to this LOW nit. Reading `events()` would trade a non-existent undercount risk for a real unbounded-state coupling.

## LOW 4 — no action required: AGREED

Every lab frame is offset-derived (`FrameId(frame_offset)`); no lab path constructs a non-offset frame. The invariant that the carrier fixture's explicit `FrameId -> offset` map protects is structurally guaranteed here.

## Methodology note shared with the lane

The shell-quoting failure class we both hit (`$var` not word-split under `zsh`; a `$var` inside a single-quoted `bash -c`) is a real evidence hazard: it silently yields *default-configuration* output that looks like "the arguments are ignored". All measurements above used literal arguments or explicit `"$@"`, and the counter values reproduce the lane's gate note exactly.

## Boundary / not claimed

Bounded verification of the two LOW repairs plus one factual determination. **Not** claimed: R9 execution, Q4-E plaintext side, a standalone Q4-B scenario, `SessionRuntime.events` resolution, D019, RSEC-001 suitability, signing/key-custody/SBOM, previous-frozen-release interoperability, item 3, item 4, or release authority. Release flags unchanged: `RELEASE_CANDIDATE=false`, `PRODUCTION_READY=false`, `FREEZE=false`, `RELEASED=false`; `READY_LIVE: none`.
