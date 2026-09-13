# ChatGPT reviewer handoff — Q7 provenance accepted; repair R7 Session byte-offset ownership before Q4 expansion

## Reviewed repository truth

- Exact current `main`: `23b1cf60cd5f4cf870a4c9f66ef25e618201310f` (`docs: record exact-tree gates for Q7 cleanup and H-RUDP-001D regression`).
- Since the previous reviewer handoff, production source remains the accepted `e0729d96f59988e7b918d21cfb19579c319b0733` carrier-layer repair plus test-only `901991d3c9abfde25ef76927b33ee7bc69f4ee23` and comment/provenance-only descendants. There is no new VPS/WAN evidence and no open PR.
- `docs/local-gate-901991d-20260913.md` is accepted as **developer-persisted local exact-tree provenance**: clean detached `252a59d` and `901991d` each report `scripts/check.sh` exit 0, `git diff --check` exit 0, clean initial/final tree, Linux x86_64 and rustc 1.98.0. This is not reviewer-executed local CI.
- GitHub-hosted checks on exact `23b1cf6` are green: `stable checks` and `nightly decode fuzz smoke`. They remain additional hosted cross-evidence only.
- Governance is unchanged: item 3 incomplete; item 4 incomplete; `RELEASE_CANDIDATE=false`; `PRODUCTION_READY=false`; `FREEZE=false`; `RELEASED=false`; `READY_LIVE: none`; D019, `SessionRuntime.events`, RSEC-001, signing/SBOM and final release authority remain separate gates.

## Accepted closure from the previous lane

### H-RUDP-011 — CLOSED

The carrier runtime now has one bounded plaintext owner. `on_packet_sent` reserves retransmit plaintext before Recovery/Reno state, preserves typed `RetransmitError`, rolls back only a reservation created by that failed call, refuses retransmit without retained plaintext, and tears down packet->frame mapping together with the plaintext owner.

### H-RUDP-001D — CLOSED

Manager-facing loss evidence uses resolved-outcome deltas (`acked + lost` denominator), not send-time deltas. The new `901991d` regression is non-vacuous: a PTO-only sample cannot erase the denominator of a later ACK that newly declares old packets lost.

### H-RUDP-012 — CLOSED

Production `neko-carrier` no longer owns Session logical delivery/dedup state. The integration fixture composes `neko_session::SessionRuntime`; `neko-session` remains dev-only for the carrier crate. The previously quarantined concurrent-editor patch that re-added carrier-side `DeliveryLedger`/`dedup_suppressed` remains **not evidence and must not be reapplied**.

### Q7 independent bounded re-review — ACCEPTED

The Q7 review and D1/D2/D3 documentation cleanup are accepted. Do not gratuitously reopen the already-closed ownership/layering/health questions unless a new source change directly invalidates them.

## New HIGH — H-RUDP-014: coherent R7 fixture uses message-index offsets where Session requires byte offsets

This is a correctness/evidence defect in `crates/neko-carrier/tests/reliable_udp_runtime.rs`, not a new architecture choice.

### Exact mismatch

The fixture's normal sender path currently does:

```text
offset = next_offset
next_offset += 1
send_data_at(offset, payload)
```

and advances `next_offset` **before** the cwnd/send-admission result is known.

But the actual Session runtime contract is byte-oriented:

- `SessionRuntime::queue_send` returns the current `next_send` and then advances it by `data.len()`;
- `SessionRuntime::receive` accepts new data only when `record.offset == next_receive`, then advances `next_receive` by `record.data.len()`;
- an offset below `next_receive` is accepted only as an exact duplicate already retained at that exact `(stream, offset)`.

Therefore the current supposed clean Round 1 is not a truthful Session flow. For example, payload `d0` has length 2 and is sent at offset 0, so the receiver advances to byte offset 2. The next payload `d1` is currently sent at offset 1; that is below `next_receive` but is not an exact duplicate for `(stream=1, offset=1)`, so `SessionRuntime::receive` rejects it with `Protocol`. The fixture records that rejection in `session_conflicts`, but the main coherent-loop test does not assert the Session counters, so carrier-level assertions can stay green while the Session layer is already failing.

A second defect follows from the same ownership shape: `send_data` increments `next_offset` before `send_data_frame` checks `rt.can_send(...)`. A cwnd refusal therefore consumes a logical offset for data that was never admitted/sent, creating an artificial future gap.

This invalidates the **coherent R7 Session-level evidence** until repaired. It does not invalidate the already-accepted carrier-local recovery/ACK/ownership unit results.

## Q1 — repair H-RUDP-014 first

### Required invariant

Session logical identity is `(SessionId, stream, byte_offset)` and is owned above Carrier. Packet number / AEAD nonce and Carrier `FrameId` remain separate identities. A refused Carrier send must not consume Session byte space.

### Preferred implementation shape

Prefer reusing sender-side `SessionRuntime::queue_send` / `pop_send` in the integration fixture to obtain the authoritative `OutboundRecord { stream, offset, data }`, **if this can be done without falsely treating packet ACK as Session DeliveryAck or otherwise changing current Session/Carrier architecture**.

A smaller fixture-only byte-offset allocator is acceptable if it is mechanically pinned to the same contract, but it must:

- advance by `payload.len()` rather than one per message;
- use checked arithmetic;
- commit the new offset only after the send has passed the relevant admission boundary;
- leave the next logical offset unchanged on cwnd refusal or another pre-send refusal.

Do not move this state back into `ReliableUdpRuntime` / `neko-carrier` production code.

### Mandatory focused regressions

1. Variable-length clean flow: payload lengths `2, 3, 1` use offsets `0, 2, 5`, all are accepted by `SessionRuntime`, with no conflict/protocol rejection.
2. Cwnd refusal / other pre-send refusal does **not** consume or skip the next Session byte offset.
3. The existing clean no-loss coherent round strongly asserts Session truth, not only Carrier truth: four first deliveries, zero Session conflicts, zero unexpected duplicates.
4. Replacement/retransmit keeps the same byte offset while using a fresh packet number/AEAD nonce and the same stable Carrier frame identity.
5. Replacement-first then late-original remains exactly one Session delivery plus one duplicate suppression.
6. Same `(stream, byte_offset)` with different bytes still fails closed and the error is retained, never swallowed.
7. Packet ACK still does not call or imply Session `DeliveryAck`; Session-delivery evidence and Carrier recovery remain separate.

Run at least:

```bash
cargo test -p neko-carrier --test reliable_udp_runtime
cargo test -p neko-carrier --all-targets
cargo clippy -p neko-carrier --all-targets --locked -- -D warnings
```

Then on the final pushed source/test SHA in a clean detached checkout/worktree:

```bash
PYTHONDONTWRITEBYTECODE=1 bash scripts/check.sh
git diff --check
```

Record exact SHA, UTC start/end, exit codes, OS/arch, Rust stable and clean initial/final tree. No decoder/framing change is implied by this repair, so do not mechanically rerun fuzz unless source changes actually touch those surfaces.

## Rolling queue after H-RUDP-014 closes

Do not wait for reviewer cadence between these dependency-ready slices. Preserve completed scenarios rather than rewriting the fixture from scratch.

1. **H-RUDP-014 repair + mandatory regressions** above.
2. **Q4-A clean/no-loss byte-accurate Session path:** strong Session + Carrier assertions, including in-flight drain and no retransmit.
3. **Q4-B one recoverable packet loss:** actual packet->frame mapping, loss scheduling and successful replacement without Session duplication.
4. **Q4-C ACK loss / PTO replacement:** lost ACK must not become Session delivery evidence; retransmit uses fresh nonce/packet and stable frame/logical byte identity.
5. **Q4-D replacement-first then delayed original:** application delivery remains exactly once; duplicate is suppressed by Session, not Carrier.
6. **Q4-E bounded plaintext/cwnd refusal:** typed plaintext-capacity/oversize/conflict refusal remains atomic; cwnd refusal consumes neither packet state nor Session byte offset.
7. **Q4-F PTO-only health sample before later resolved loss:** preserve the `901991d` invariant inside the coherent path. **Adjudicated 2026-09-13 (see `docs/reviews/independent-carrier-q4f-adjudication-8c09425-20260913.md`):** do NOT add a production `HealthSample`/`loss_per_mille` accessor for this. Assert it coherently through the existing `RuntimeEvent::HealthSample(HealthState)` observable via the loss branch: one PTO-only interval (assert not `Degraded`/`Failed`), then two consecutive resolved-loss intervals at >= 500/mille with no intervening send and `pto_count < 3` (assert freshness, then the `WarmFallback`/`FallbackFailed` consequence). `HealthLimits::default()` has `degrade_after = 2`, so a single bad interval will not degrade — that is expected, not a missing accessor. If raw-sample observability is genuinely wanted, it belongs to Q10 as a production need, not to Q4-F as a test hook.
8. **Q4-G clean resolved outcome after historical loss:** old cumulative loss must not replay as a new bad sample.
9. **Q4-H sustained distinct bad resolved outcomes:** only genuinely fresh bad evidence can cross hysteresis and produce a real `WarmFallback` switch event.
10. **Q4-I invalid/unready TCP standby:** degradation must never fabricate a successful switch; return `FallbackFailed`/non-success truthfully.
11. **Q4 closure package:** exact-tree local gate/provenance, then one independent bounded R7 challenge covering byte-offset ownership, packet/session evidence separation, retransmit ownership, health freshness, cwnd/pacing and fallback truthfulness. Repair any concrete defect immediately.
12. **R8 CLI/local-lab integration:** expose the accepted reliable-UDP runtime through a bounded executable CLI/lab path; do not add a production daemon or public listener.
13. **R9 process/socket acceptance:** real process boundary, authenticated UDP packet recovery, bounded timeouts/cleanup, then packet-recovery-driven degradation -> warm TCP fallback using the same logical Session.
14. **Q10 observability/release-boundary integration:** recovery RTT/loss/PTO/retransmit and switch events are structured and bounded; no packet ACK promotion to logical delivery.
15. **Q11 status/release reconciliation:** update `docs/status.md`, implementation/release packet and item-3 classification only for evidence actually produced. If and only if the new executable/process path creates a materially new real-network question, classify a concrete `READY_LIVE` row.
16. **Q12 VPS opportunity:** once Q11 truthfully yields `READY_LIVE`, run one minimal changed-hypothesis self-owned client<->VPS experiment within standing authorization for real packet recovery/degradation -> TCP fallback, with binary identity, parameters, structured evidence and cleanup. Preserve a negative result if that is what happens.

The queue is intentionally deeper than one reviewer interval. If the external agent finishes several slices quickly with clean gates, continue forward; do not collapse the queue back to one small ticket.

## Evidence / release boundaries

- Current `docs/status.md` still describes reliable UDP as a deterministic candidate and makes no live-service claim. Do not promote it before R8/R9 acceptance and reconciliation.
- Item 3 remains unchecked; the historical `25e0daa` warm fallback is application-level reply cessation, not natural packet-loss/PTO evidence.
- `READY_LIVE: none` remains authoritative until Q11 explicitly changes it on new accepted implementation/instrumentation evidence.
- Existing HY2, repeated-failover, periodic/soak, package lifecycle, migration-back, endpoint/key-migration, IPv6 and PLPMTUD lines remain frozen/answered/blocked as previously classified; do not rerun them merely because the VPS is available.
- Separate policy/security gates remain separate: `SessionRuntime.events` retention, D019, RSEC-001 capacity suitability, signing/key custody/SBOM/publication trust, previous-frozen-release interop, final security/release judgment and RC/freeze/release/production authority.

## Stop conditions

Stop expansion only for a new unresolved BLOCKER/HIGH that cannot be mechanically repaired under current committed semantics, a genuinely new core Session/Carrier/ACK/crypto/wire architecture choice, destructive/canonical migration, production/third-party/new-credential need, work outside standing authorization, or actual runtime/tool exhaustion. H-RUDP-014 is mechanically determined and does **not** require maintainer input: repair it and continue.
