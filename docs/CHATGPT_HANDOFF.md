# ChatGPT reviewer handoff — R9-2 partial repair accepted; demux + migration-back ownership still block R9-3

## Reviewed repository truth

- Current developer source/test repair SHA: exact `2dcb7316849a0ead04dfd6d4b96cd53480e085d2` (`fix(cli): R9-2A/B/C/D multi-record ownership + ACK demux + Data-admission + discriminating test`).
- Exact `cc6fb959411f8f54ef4ad64bd84df649f874f430` adds developer-local exact-tree provenance only; no source/test change after `2dcb731`.
- Reviewer recheck is persisted at `docs/reviews/independent-r9-repair-recheck-2dcb731-20260914.md` (reviewer commit `b2ce36f2c982cbbf549faa2dca0e73822dad445a`).
- `docs/local-gate-2dcb731-20260914.md` records a clean detached developer-local exact-tree gate: `PYTHONDONTWRITEBYTECODE=1 bash scripts/check.sh` exit 0, `git diff --check` exit 0, clean initial/final tree, Linux x86_64, rustc 1.98.0.
- Current descendant `cc6fb95` GitHub-hosted `stable checks` and `nightly decode fuzz smoke` are green; hosted checks remain extra cross-evidence only.
- No WAN/VPS execution occurred in this sequence. `READY_LIVE: none` remains authoritative.
- Governance remains unchanged: item 3 incomplete; item 4 incomplete; `RELEASE_CANDIDATE=false`; `PRODUCTION_READY=false`; `FREEZE=false`; `RELEASED=false`.

## Reviewer verdict

The R9 source path is real forward progress and the `2dcb731` repair is **partially accepted**. Do not revert it.

Closed:

- **H-R9-002** for the ordinary no-recovery R9-2 path: `records[1]` no longer also traverses the legacy offset-16 direct uncertain UDP owner.
- **H-R9-003**: receiver packet ACK obligation is now committed only after authenticated expected-Session `ProcessMessage::Data` reaches `SessionRuntime::receive` successfully.

Still blocking R9-3:

- **H-R9-001 OPEN** — multi-record Session DeliveryAck demux is still single-record and swallows/ignores the second logical confirmation.
- **H-R9-005 NEW HIGH** — `--reliable-udp` combined with recovery/migration-back can consume the reserved final record early through the legacy uncertain path.
- **M-R9-004 OPEN/PARTIAL** — durable process evidence still does not prove two independent Session confirmations, both Carrier owners retired, zero PTO/retransmit, and clean-baseline Session duplicate/conflict bounds.

Do not advance into R9-3 fault injection until these are repaired and re-gated. These are mechanically determined local correctness/evidence findings; no maintainer policy choice is needed.

# Blocking findings and exact repair contract

## H-R9-001 — authenticated receive demux still swallows Session confirmation for record 2

Current `recv_udp_delivery_ack` still accepts one `expected: &OutboundRecord` and returns only for that one Session `DeliveryAck`.

When `ReliableUdpRuntime` is present, every other authenticated plaintext enters the Carrier-ACK branch. If it is not a canonical Carrier ACK, the function still `continue`s. Therefore:

- record-2 Session `DeliveryAck` arriving before record-1 confirmation is consumed/discarded;
- record-2 Session `DeliveryAck` arriving after the helper returns is ignored by the later settlement loop, which only attempts Carrier-ACK decoding;
- caller-side `SessionRuntime::delivery_ack` is still invoked only for the first `udp_record`;
- `apply_ack` errors remain silently discarded with `let _ = ...`;
- authenticated non-ACK/control/malformed plaintext bypasses `MAX_POST_HANDSHAKE_MALFORMED` whenever R9 runtime is active.

### R9-2A repair

Replace the R9 single-record helper behavior with one bounded authenticated receive/demux loop over a bounded outstanding logical-record set/map.

It must classify each authenticated plaintext exactly once:

1. **Carrier packet ACK** — canonical `RecordType::Ack` -> `decode_ack` -> bounded `AckRanges` -> `apply_ack`; success/rejection must be typed/observable; rejection must not mutate recovery.
2. **Session DeliveryAck** — exact expected Session id + `(stream, offset, len)` match against outstanding reliable-owned logical records -> apply `SessionRuntime::delivery_ack` exactly once; duplicate/stale/unexpected logical ACK is typed/bounded rather than silently accepted.
3. **Unexpected authenticated control / malformed plaintext** — consume the existing bounded malformed/ignored budget and emit typed diagnostic; do not silently continue forever.

Carrier packet ACK and Session delivery confirmation remain separate evidence domains.

## H-R9-005 — migration-back reserved final record is consumed early in reliable mode

Pre-existing recovery/migration-back semantics deliberately reserve the final logical record until post-promotion return-to-UDP authorization by setting `uncertain_end = records.len() - 1`.

`2dcb731` moved reliable-mode legacy ownership to index 2 but left the iterator count derived from the old start index and unconditionally chooses `uncertain_idx = 2` for the direct uncertain send.

For `--reliable-udp` + recovery/migration-back, especially a three-record run:

- `uncertain_end == 2` means index 2 is supposed to be reserved/unassigned;
- current code can nevertheless track and directly send index 2 before migration-back authorization.

That violates the existing owner transition contract and would poison later R9-8/R9-10 evidence.

### R9-2B repair

Use explicit exclusive index bounds:

- reliable-owned end = `min(2, records.len())` while `--reliable-udp` is active;
- legacy `uncertain_start` = reliable-owned end (legacy mode remains start 1);
- preserve current exclusive `uncertain_end`, including final-record reservation when recovery is enabled;
- track/send legacy uncertain data only when `uncertain_start < uncertain_end`;
- iterator count = `uncertain_end.saturating_sub(uncertain_start)`.

Add a combined `--reliable-udp` + migration-back/recovery regression proving the final reserved logical offset is neither tracked nor wire-sent before the authorized post-promotion return-to-UDP milestone.

## M-R9-004 — multi-record proof remains incomplete

Current test proves offset 16 is reliable-owned and not also emitted as the legacy offset-16 uncertain send. It does not prove the complete R9-2 contract.

### R9-2C evidence completion

The repaired no-loss process path must durably prove:

- two distinct reliable-owned logical offsets were admitted and wire-sent;
- both logical records were first-delivered exactly once by Session;
- two corresponding Session `DeliveryAck`s were independently validated and applied to `SessionRuntime`;
- Carrier packet ACKs retire both packet owners; authoritative `in_flight()==0`;
- zero PTO and zero retransmission in this clean baseline;
- zero Session conflict and zero (or explicitly justified zero-expected) duplicate in clean baseline;
- an injected authenticated non-Data/control/malformed plaintext does not enter server packet-ACK ranges and consumes the bounded rejection budget;
- listener/process/identity cleanup remains valid.

The test must fail if record-2 Session ACK is swallowed or if the final migration-back reserved record is consumed early.

## R9-2D — repaired exact-tree gate

After one coherent pushed source/test SHA closes H-R9-001/H-R9-005/M-R9-004, run in a clean checkout/worktree:

```bash
PYTHONDONTWRITEBYTECODE=1 bash scripts/check.sh
git diff --check
```

Record exact pushed SHA, UTC start/end, OS/arch, rustc, exit codes, clean initial/final tree. Hosted CI remains separate cross-evidence.

No new wire/parser grammar is required. If external decode/framing grammar is changed anyway, run the pinned decode fuzz commands before closure.

# Pre-authorized continuous queue after repaired R9-2

Do not stop after the repaired no-loss baseline. Continue immediately through all dependency-safe slices below.

## R9-3 — process Data-loss recovery

Add bounded deterministic **post-admission** suppression of one/periodic reliable-owned Data packet. Prove congestion admission precedes suppression; PTO fires only after deadline; retransmission uses a fresh authenticated packet number/nonce with stable frame/Session identity; exactly-once Session delivery; both Carrier ACK and Session DeliveryAck settle independently; final recovery drains to zero or yields an explicit bounded partial/failure outcome.

## R9-4 — ACK-loss and reorder/delayed-original

Separate bounded cases:

- packet ACK emitted then suppressed;
- retransmitted replacement arrives before delayed original;
- delayed original arrives after replacement.

Require one logical Session delivery, Session-owned duplicate suppression, no conflict, fresh packet numbers/nonces, deadline-driven PTO, truthful ACK-loss counters and final settlement.

## R9-5 — tamper / future ACK / malformed feedback negatives

Prove:

- unauthenticated/tampered Data -> no packet-ACK obligation and no Session receive;
- tampered ACK -> no recovery mutation;
- future/never-sent ACK -> typed rejection + atomic state;
- stale/duplicate ACK -> no fabricated RTT/loss/Session evidence;
- malformed ACK/range/control bounds are finite and panic-free.

## R9-6 — pacing/cwnd/plaintext-owner atomicity

Use bounded test-only limits. Initial and retransmit sends consult congestion admission; refusal consumes no Session byte space and no committed packet/plaintext/recovery owner; retransmission plaintext ownership remains bounded and single-source-of-truth; pacing deadlines are finite; teardown releases ownership.

## R9-7 — truthful process observability/result contract

Machine-readable evidence must distinguish:

- Data offered/admitted/wire-sent/suppressed;
- Carrier ACK emitted/wire-sent/suppressed/applied/rejected;
- Session DeliveryAck emitted/applied/duplicate/rejected;
- PTO due/fired;
- retransmit attempted/admitted/wire-sent/refused;
- resolved acked/lost and remaining in-flight;
- Session first-delivery/duplicate/conflict/application bytes;
- cleanup and final outcome.

Counters increment only after the action they represent succeeds. No payload/key/private-topology logging.

## R9-8 — actual authenticated warm TCP standby

Only after R9-2..7 are green: reuse existing failover TCP connect/accept, canonical negotiation, fresh Noise trust/authz, resume/readiness tuple+generation+delivery-epoch binding and resource admission. No application Data on standby before atomic promotion.

## R9-9 — resolved UDP health -> hysteresis -> real TCP promotion

Fresh resolved UDP outcomes drive Carrier health. Prove recoverable loss remains on UDP; PTO-only observation cannot erase later resolved loss; distinct bad resolved intervals cross committed hysteresis and promote only to the actually ready TCP standby; invalid/unready standby yields `FallbackFailed`; historical old loss is not replayed as new evidence.

## R9-10 — uncertain Session replay across UDP -> TCP

At promotion, replay at least one genuinely uncertain logical Session range over promoted TCP. Draining UDP accepts no new application Data. Receiver deduplicates by Session/stream/byte offset; application bytes are delivered exactly once. Do not add TCP packet ACK.

## R9-11 — timeout/shutdown/cleanup matrix

Cover setup, application, PTO/settlement deadlines, controlled stop, failed standby promotion, malformed/tampered input, listener/socket/process cleanup and same-address rebind where existing harness supports it.

## R9-12 — coherent exact-tree gate + independent bounded review

Independent review must challenge Session-above-Carrier layering, authenticated packet identity/ACK, bounded plaintext ownership, deadline-driven PTO/pacing/cwnd, fresh health/hysteresis, real warm TCP readiness/promotion, uncertain Session replay/dedup, result truthfulness and cleanup/no public exposure.

BLOCKER/HIGH -> smallest repair + regression + re-gate + continue. LOW/NOTE does not halt progression.

# Q10/Q11/Q12 after R9

- **Q10:** integrate only genuinely new R9 evidence into existing observability; no second logging framework.
- **Q11:** reconcile `docs/status.md`, `IMPLEMENTATION_PLAN.md`, `ROADMAP.md` and release packet only for earned status. If cross-process R9 + real TCP promotion is green and independently reviewed, create a new specific `READY_LIVE` row for this changed implementation/hypothesis.
- **Q12:** execute exactly one minimal changed-hypothesis self-owned client<->VPS run under standing authorization, with temporary unprivileged listeners, exact commit/binary/parameters, authenticated recovery/Session/switch evidence and cleanup. Preserve negative result; no unchanged same-class retry.

# VPS opportunity

**Not READY yet.** The cross-process R9 implementation is materially new, but H-R9-001/H-R9-005 mean current multi-record/recovery ownership evidence is not trustworthy enough for WAN use. This is an implementation correctness dependency, not a permission blocker.

Once repaired R9-2..R9-12 and Q11 create the changed scientific question, standing authorization already covers the bounded self-owned TCP/UDP experiment. Do not ask again for ordinary WAN authorization.

# Non-blocking policy/authority gates

These remain separate and must not stall the mechanically determined local R9 chain:

- `SessionRuntime.events` retention (`POLICY_BLOCKED_RESOURCE_BOUND`);
- D019 source-retention/no-reset policy;
- RSEC-001 capacity/adversarial-load suitability;
- signing/key custody/SBOM/publication trust;
- previous frozen-release interoperability;
- final RC/freeze/release/production authority.

Do not invent policy values while working R9.