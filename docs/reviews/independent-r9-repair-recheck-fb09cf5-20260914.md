# Independent R9 repair recheck — exact `fb09cf5`

**Reviewed source/test SHA:** `fb09cf59e81782a5b2ecf12466bf47f72a9e51d6` (`fix(cli): R9-2 multi-record demux + reserved-record ownership`).

**Later provenance-only descendant:** `f74f1e6d711258bb2cba0853feed4359d799bee6` adds `docs/local-gate-fb09cf5-20260914.md`; no source/test change after `fb09cf5` at review time.

**Scope:** bounded independent recheck of the R9-2 blockers recorded in `docs/CHATGPT_HANDOFF.md`: multi-record authenticated receive demux, reliable/legacy ownership, migration-back reserved-record ownership, and durable no-loss process evidence. This is not a security audit, WAN result, release decision, or production approval.

## Evidence truth

Developer-local exact-tree provenance for `fb09cf5` records clean detached `PYTHONDONTWRITEBYTECODE=1 bash scripts/check.sh`, `git diff --check`, and clean initial/final tree. Current docs-only descendant `f74f1e6` also has green GitHub-hosted `stable checks` and `nightly decode fuzz smoke`; hosted checks are extra cross-evidence only.

Open PRs at review time: none. No WAN/VPS execution occurred in this sequence. `READY_LIVE: none` remains authoritative.

## Verdict

**R9-2 remains blocked before R9-3.** `fb09cf5` contains useful forward progress and must not be reverted, but it does not close the authenticated multi-record demux requirement.

### H-R9-005 — implementation repair accepted, discriminating evidence still required

The index-bound repair is mechanically correct in the reviewed code:

- `uncertain_start = 2` in reliable mode, otherwise `1`;
- `uncertain_count = uncertain_end.saturating_sub(uncertain_start)`;
- the legacy tracking loop consumes exactly that exclusive range;
- the direct uncertain send is guarded by `uncertain_start < uncertain_end`.

Therefore a three-record recovery/migration-back run with `uncertain_end == 2` no longer assigns or sends record index 2 before the post-promotion return-to-UDP milestone.

However, the new process-test delta only extends the ordinary no-recovery reliable-UDP test. It does **not** exercise `--reliable-udp` together with migration-back/recovery and therefore does not yet provide the requested discriminating regression proving the reserved final logical offset is absent from pre-promotion tracking/wire-send events.

Classification: **correctness implementation repaired; evidence closure still pending under M-R9-004.**

### H-R9-001 — OPEN HIGH: serial per-record helper is still not a demultiplexer

`recv_udp_delivery_ack` still accepts exactly one `expected: &OutboundRecord` and loops until that one Session `DeliveryAck` arrives. With a reliable runtime present, any other authenticated plaintext enters the Carrier-ACK branch; if it is not a canonical Carrier ACK, the function unconditionally `continue`s.

`fb09cf5` calls that same helper once for record 0 and then again for record 1. This proves the happy-path server ordering currently observed, but it does not solve reordering:

- if record-1 Session `DeliveryAck` arrives while the first helper invocation is waiting for record 0, it is authenticated, fails `delivery_ack_matches(expected=record0)`, fails Carrier-ACK decode, and is consumed/discarded by the unconditional `continue`;
- the second helper invocation can then time out even though record 1 was already legitimately confirmed on the wire;
- authenticated non-ACK/control/malformed plaintext still bypasses `MAX_POST_HANDSHAKE_MALFORMED` whenever `rt` is present because the runtime branch continues regardless of whether ACK decoding succeeded;
- `rt.apply_ack(...)` errors remain discarded with `let _ = ...`, so rejected/future/stale Carrier feedback is not typed/observable at this receive boundary.

This is exactly the previously requested demux property and remains a mechanically repairable local correctness/evidence issue. No policy choice is required.

### M-R9-004 — OPEN/PARTIAL

The extended no-loss process test now checks that an offset-16 `r9_udp_delivery_ack_validated` event appears and that the legacy uncertain direct send starts at offset 32. It also retains the earlier `remaining_in_flight:0` assertion.

It still does not durably prove the complete requested contract:

- Session DeliveryAcks for two outstanding reliable-owned records are order-independent rather than merely serial/happy-order;
- both Session confirmations are applied exactly once to `SessionRuntime` under a single bounded receive/demux owner;
- Carrier ACK application rejection is typed rather than silently discarded;
- authenticated unexpected/control/malformed plaintext consumes the bounded rejection budget while reliable mode is active;
- the combined reliable-UDP + migration-back reserved-record rule is exercised;
- clean baseline explicitly pins zero PTO/retransmit and zero Session conflict/duplicate (or a justified duplicate expectation).

## Required next coherent repair

Do not advance to R9-3 yet. Replace the serial expected-record receive shape with one bounded authenticated demux owner over the bounded set of outstanding reliable-owned logical records.

For every successfully authenticated plaintext, classify exactly once:

1. **Carrier packet ACK:** canonical `RecordType::Ack` -> `decode_ack` -> bounded `AckRanges` -> `apply_ack`; record applied/rejected outcome. A rejected ACK must not mutate recovery.
2. **Session DeliveryAck:** decode `ProcessMessage::DeliveryAck`, match exact Session id + `(stream, offset, len)` against the bounded outstanding reliable-owned map/set, apply `SessionRuntime::delivery_ack` once, retire that logical expectation, and type duplicate/stale/unexpected ACKs.
3. **Unexpected authenticated control / malformed plaintext:** consume the existing finite malformed/ignored budget and emit typed diagnostic; never spin silently because reliable mode is active.

The loop may finish when all required logical confirmations have been applied and Carrier in-flight ownership has settled, or return an explicit bounded timeout/partial result. Keep Carrier ACK evidence separate from Session delivery evidence.

Add two discriminating process regressions before re-gating:

- force/reliably arrange reversed Session DeliveryAck order for the two reliable-owned records and prove both are retained/applied exactly once;
- run reliable UDP with migration-back/recovery and prove the reserved final offset is neither tracked nor wire-sent before the post-promotion return-to-UDP authorization milestone.

Then re-run a clean exact pushed-tree `PYTHONDONTWRITEBYTECODE=1 bash scripts/check.sh`, `git diff --check`, clean-tree checks and provenance. No wire/parser grammar change is required by this repair.

## Downstream queue

After this R9-2 closure, continue without waiting through the existing R9-3..R9-12 and Q10/Q11/Q12 queue. No `READY_LIVE` promotion is earned before the repaired R9 process/runtime chain and independent review complete.

Governance unchanged: item 3 incomplete; item 4 incomplete; `RELEASE_CANDIDATE=false`; `PRODUCTION_READY=false`; `FREEZE=false`; `RELEASED=false`.
