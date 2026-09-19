# Independent R9-10 challenge — H-R9-075 SessionRuntime gapped DeliveryAck confirmation

**Review anchor:** exact reachable `50689e7ac39253bcea912509c6e7c191be4cc00e` (`main` at review start).

**Classification:** `HIGH` correctness / Session-delivery evidence truth.

**Finding:** H-R9-075 — a future/gapped exact Session `DeliveryAck` can advance the `SessionRuntime` confirmed watermark across bytes for which no logical delivery proof was supplied.

## Owners and current contract inspected

- `crates/neko-session/src/lib.rs::SessionRuntime::{queue_send,delivery_ack,confirmed_watermark}`
- `crates/neko-cli/src/main.rs` reliable-UDP acknowledgement owner and its out-of-order buffering before `SessionRuntime::delivery_ack`
- `crates/neko-carrier/src/lib.rs::{ConcurrentCarrierManager,FailoverController}` uncertain/replay ownership
- `docs/specs/nekomusume-session-v0.md` Session-delivery evidence boundary
- `docs/carrier-architecture.md` separation of Carrier packet feedback from Session delivery
- `docs/decisions.md` D064 uncertain-range rule
- current developer R9-10 note `docs/reviews/dev-r9-10-uncertain-replay-20260919.md`

## Concrete counterexample

`SessionRuntime::delivery_ack(stream, offset, len, now_ms)` currently computes:

1. `end = offset + len`;
2. `current = confirmed_watermark(stream)`;
3. rejects only when `end < current`;
4. computes `delta = end - current`;
5. if `delta <= stream/session inflight`, releases `delta` bytes and sets the confirmed watermark to `end`.

There is no requirement that a positive-advancing acknowledgement start at the current confirmed watermark.

A deterministic two-record case therefore violates the logical-proof boundary:

```text
queue_send stream=1 bytes="aa" -> range [0,2)
queue_send stream=1 bytes="bb" -> range [2,4)
confirmed watermark = 0

DeliveryAck(stream=1, offset=2, len=2)
  end = 4
  delta = 4
  inflight = 4
  => currently accepted
  => confirmed watermark becomes 4
  => all 4 inflight bytes are released
```

The ACK proves only `[2,4)`, yet `[0,2)` is silently promoted to confirmed. A later failover/replay owner that relies on Session confirmation can therefore suppress replay of bytes that never obtained valid Session-delivery proof.

This is not merely a hypothetical API interpretation: the executable reliable-UDP owner already treats `offset > watermark` as an out-of-order ACK, buffers it, and refuses to apply it to `SessionRuntime` until the gap is closed. The runtime API itself does not preserve that invariant.

## Why the current R9-10 no-finding note is insufficient

The developer note at exact `50689e7` correctly challenges `ConcurrentCarrierManager` range removal/uncertain marking and stable replay identity, but it does not challenge the Session confirmation primitive that decides whether bytes are logically proven before they leave replay eligibility. `confirm(id)` being correct cannot compensate for a Session owner that can manufacture a larger confirmed watermark from a gapped exact ACK.

R9-10 therefore is **not independently closed** at `50689e7`.

## Smallest repair contract

Current semantics already decide the required behavior; no Session/Carrier architecture or policy value needs to change.

1. Add a focused `SessionRuntime` regression with two queued records. A `DeliveryAck` for the second range before the first must fail closed and atomically preserve:
   - confirmed watermark;
   - per-stream and session in-flight accounting;
   - event count / delivery evidence.
2. Keep the ordinary positive sequence: ACK first range, then second range; watermark and in-flight accounting advance exactly by the proved contiguous ranges.
3. Preserve current accepted-empty duplicate behavior only to the extent already committed; do not use this repair to redesign ACK encoding or cumulative-ACK policy.
4. Keep the existing process-level reverse/out-of-order ACK regression green so buffering remains an executable positive control.
5. Run the clean exact-tree developer gate on the final pushed repair SHA:
   - `PYTHONDONTWRITEBYTECODE=1 bash scripts/check.sh`
   - `git diff --check`
   - clean tree, exact SHA, UTC start/end, OS/arch, stable Rust version, exit codes.
6. No decoder/parser/crypto-framing change is required, so do not run decode fuzz mechanically.

After repair, re-run the remaining R9-10B/C independent challenge, including actual executable UDP -> TCP replay identity/byte equality and cleanup/partial-promotion negatives.

## Exclusions / unchanged gates

- No D019 or retention/capacity/security numeric policy decision.
- No new ACK/wire/crypto format.
- No change to D064 single-active/multi-ready architecture.
- No RC/freeze/release/production transition.
- No WAN evidence requested; `READY_LIVE: none` remains appropriate.
- Reviewer-local test execution is not claimed for this docs-only finding.
