# ChatGPT reviewer handoff — carrier HIGHs closed; Q7 accepted; concurrent-edit anomaly recorded

## Reviewed repository truth

- Exact current HEAD: `901991d3c9abfde25ef76927b33ee7bc69f4ee23` (`test(carrier): prove a PTO-only health sample cannot erase a later resolved loss`).
- Reviewed source tree: `e0729d96f59988e7b918d21cfb19579c319b0733` (`fix(carrier): restore Session-above-Carrier layering (H-RUDP-012)`), on top of `a576ce0` (`fix(carrier): resolved-outcome loss delta + single bounded plaintext owner (H-RUDP-001D/011)`) and `252a59d` (`docs(carrier): drop orphaned Session-dedup doc and stale loss formula`).
- The only source change in `e0729d9..901991d` is `crates/neko-carrier/src/lib.rs` (+58/-11): `252a59d` comment-only, `901991d` test-only. No production behavior change in that interval.
- Independent verification at these trees: `cargo test -p neko-carrier --all-targets` -> lib **69 passed** at `e0729d9` and **70 passed** at `901991d` (including the new regression), all integration targets green; `cargo clippy -p neko-carrier --all-targets --locked -- -D warnings` clean; production `crates/neko-carrier/src/` has **zero** `neko_session` references; `neko-session` remains a **dev-dependency only**; `grep -rn dedup_suppressed crates/` is empty.
- No new VPS/WAN evidence is accepted in this sequence. `READY_LIVE` remains none.
- Governance unchanged: item 3 incomplete; item 4 incomplete; `RELEASE_CANDIDATE=false`; `PRODUCTION_READY=false`; `FREEZE=false`; `RELEASED=false`; D019, `SessionRuntime.events`, RSEC-001, signing/SBOM and final release authority remain separate gates.

## Reviewer verdict

### H-RUDP-011 — CLOSED (verified)

Transactional owner is real: `on_packet_sent` orders cwnd check -> reserve bounded plaintext -> `recovery.on_sent` -> only then `packet_frames.insert`, so every refusal leaves no recovery packet, no Reno charge and no packet->frame entry. The typed `PathRecoveryError::Retransmit(RetransmitError)` preserves capacity/oversize/conflict instead of collapsing them. A failed attempt rolls back only a copy newly reserved by that call, so a pre-existing identical frame survives. `on_retransmit_sent` fails before recording a packet when the stable frame has no retained plaintext ownership, and `teardown()` drops the packet->frame map and the plaintext owner together. `RetransmitBuffer` is hard-bounded (64 frames / 8192 bytes) with oversize check before capacity check, saturating arithmetic and a full reset on `clear()`.

### H-RUDP-001D — CLOSED (verified)

`fresh_health_sample()` now consumes the **resolved-outcome interval delta** (`resolved_packets` / `resolved_lost`, updated only from `RecoveryResult` acked+lost) with a `checked_div` zero fallback; `outcome_epoch` advances only on a resolved outcome (non-empty ack/lost/retransmit, or a PTO that schedules probes), never on a bare `on_sent`. So a PTO-only sample cannot consume the denominator and a later ACK that newly declares old packets lost cannot be reported as a clean zero-loss sample. Regression `pto_only_health_sample_cannot_erase_a_later_resolved_loss` (`901991d`) exercises reviewer-mandatory cases 1 and 4 and is **non-vacuous**: it asserts at least one newly resolved loss, then asserts the exact ratio `newly_lost*1000/(1+newly_lost)`, derived from the public `packets_lost()` delta rather than a hard-coded reorder policy.

### H-RUDP-012 — CLOSED (verified)

Production `neko-carrier` no longer owns Session delivery state: `deliver_logical`, the unbounded `delivered` map, `dedup_suppressed` and `PathRecoveryError::DeliveryConflict` are gone; the crate keeps carrier-local production deps (`neko-reliable`/`neko-wire`) and `neko-session` stays dev-only. The R7 fixture composes a bounded `neko_session::SessionRuntime` with an explicitly opened stream, counts Session-layer exact-duplicate suppression, and retains conflicting-duplicate errors instead of swallowing them with `let _`.

### Q7 bounded re-review — ACCEPTED; D1/D2/D3 repaired

Exact `0f8d06a` records a bounded independent Q7 re-review of `e0729d9` (no correctness/layering/resource-bound/evidence-separation defect) plus three documentation-only defects. `252a59d` repairs all three exactly: the orphaned `deliver_logical` doc attached to `on_packet_received`, the truncated duplicate fragment before `on_packet_sent`, and the stale send-time loss formula doc. Comment-only; no behavior, API or wire/crypto/Session semantics change.

## ANOMALY — unexplained concurrent edit of the shared worktree (recorded; NOT evidence)

While the developer lane worked in the shared worktree, `crates/neko-carrier/src/lib.rs` was modified to **re-add** `delivered: neko_session::DeliveryLedger` and `pub dedup_suppressed` to `ReliableUdpRuntime` — the exact H-RUDP-012 layering violation that `e0729d9` removed — mixed with a draft of the H-RUDP-001D regression test. The developer lane quarantined that diff to `/tmp/neko-unexpected-rework-20260913T151409Z.patch` (sha256 `da2c9a36121dfc655d869412bd5f90ae8d7adc908537971b53606f29f7a934ec`), restored the committed state, and re-applied only the legitimate test.

**Provenance verdict from the review session: this rework did NOT originate from the review session.** Evidence:

1. Every commit the review session authored on this repository is documentation-only with **zero** files under `crates/`, `scripts/`, `schema/` or `fuzz/` (`a24370b`, `5d85e09`, `60c1f25`, `0f8d06a`).
2. The only non-read operation the review session performed on any source file was `touch crates/neko-carrier/src/lib.rs`, used once to force a clippy rebuild. That updates mtime only and cannot add content; it is disclosed because it can make the file look "recently modified".
3. The patch content is the opposite of the review session's output: the Q7 review recommended **removing** carrier-side logical dedup, and the patch embeds a draft of the developer lane's own test (`pto_only_health_sample_cannot_erase_a_later_resolved_loss`), which the later committed `901991d` improves (derives the ratio from `packets_lost()` instead of hard-coding `750`).

**Disposition:** treat as an unexplained concurrent editor, not as evidence for any claim. Do not re-apply it as-is — it re-introduces the H-RUDP-012 violation and, as written, would also require promoting `neko-session` from `[dev-dependencies]` to `[dependencies]`, which the accepted architecture forbids. The committed tree at `901991d` was verified clean: no `delivered` / `dedup_suppressed` / `deliver_logical` / `neko_session::` in production source; the only remaining `DeliveryLedger` mentions are negative prose (`lib.rs:4042`, `lib.rs:4884`).

## Still open (not blocked by the above)

- **Q4 remainder:** the full ten-scenario coherent R7 fixture is **NOT** done; only the Session composition and the duplicate/conflict flow were added. The remaining scenarios (no loss; one recoverable loss; ACK loss/PTO replacement; replacement-first then late original; plaintext bound refusal; PTO-sample-before-later-loss; clean recovery after old loss; sustained distinct bad resolved outcomes -> warm fallback; invalid standby -> no false switch) are still required before R7 acceptance.
- **Q5:** the supersede classification of `docs/local-gate-eeb49d7-20260913.md` is recorded without rewriting the old file.
- **Q8/R8** executable CLI/lab integration and **Q9/R9** process/socket acceptance; then **Q10** observability/release-boundary integration and **Q11** status/release reconciliation.
- **Q12** changed-hypothesis self-owned VPS evidence only if Q11 produces a genuine `READY_LIVE` row within standing authorization.
- **Separate policy/security gates:** `SessionRuntime.events` retained-state policy; D019 source-retention/no-reset; RSEC-001 representative adversarial-load/capacity suitability; signing/key custody/SBOM/publication trust; previous-frozen-release interoperability; RC/freeze/release/production authority.
- `READY_LIVE: none`.

## Accepted pieces that should not be gratuitously rewritten

Unless a repair reveals a direct dependency bug, preserve: future/unsent ACK rejection; canonical ACK grammar and pending-ACK/no-ACK-of-ACK; fresh packet number/AEAD nonce separate from stable FrameId; per-packet charged-byte Reno accounting; the cwnd gate/pacing interval API; explicit fallback success/failure with a real switch event; authenticated packet observation before ACK tracking; and packet-feedback / Session-delivery evidence separation.

## Stop conditions

Stop expansion only for an unresolved new BLOCKER/HIGH, a required core Session/Carrier/ACK/crypto/wire architecture choice not already decided by repository invariants, destructive/canonical migration, production/third-party/new-credential need, work outside standing authorization, or actual tool/runtime exhaustion. No release flag is advanced by this handoff.
