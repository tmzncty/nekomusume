# Independent R9 review — H-R9-059 retained Session ACK witness

Review anchor before this note: exact `1bbf7e8939ba2c6a73015a6a57b0c63ac6706018` on `main`.

This is a bounded independent reviewer note for release item 4. It is not a release, security approval, production authorization, protocol freeze, WAN result, or performance conclusion.

## Scope read

Current repository truth and the current slice were re-read before classification, including `README.md`, `AGENTS.md`, `ROADMAP.md`, `IMPLEMENTATION_PLAN.md`, `SECURITY.md`, `docs/status.md`, `docs/standing-vps-lab-authorization.md`, `docs/vps-rental-window-priority.md`, `docs/decisions.md`, `docs/carrier-architecture.md`, `docs/specs/nekomusume-session-v0.md`, `docs/release-security-review-packet.md`, `docs/CHATGPT_HANDOFF.md`, exact-current `SessionRuntime`, the R9 Session-ACK demux/tests, the R9 process negatives, and exact-head hosted CI.

Developer-owned commits since the previous reviewer handoff are:

- `08cbdaf63fa24152eb3387cb9eae98a34e9d21c3` — implementation repair for H-R9-057/H-R9-058;
- `1bbf7e8939ba2c6a73015a6a57b0c63ac6706018` — clippy-only binding cleanup in the new CLI regression.

No reviewer-local execution is claimed in this note. Hosted CI is cross-evidence only; it is not relabelled as developer-local or reviewer-local CI.

## H-R9-057 source defect: closed, exact-head gate still red

The one-shot delay/reorder source defect is repaired at `08cbdaf6`:

- `delay_reorder_done` begins false;
- only the first post-return ACK can be withheld;
- the next post-return packet releases the delayed ACK and sets `delay_reorder_done = true`;
- subsequent packets take the ordinary ACK path and cannot automatically re-arm the delay.

Exact-head hosted run `35365100701` confirms that `reliable_udp_ack_loss_delayed_original_reorder_settles` now passes. The previously observed alternating-ACK suppression is therefore not reproduced on the repaired source.

However the same exact-head hosted run remains red because the two older post-return negative tests still assert `residual.len() == 1` after the runtime was intentionally changed to keep bounded PTO/recovery running until the operation deadline:

- `reliable_udp_post_return_carrier_ack_withheld_fails`: Session is complete, Carrier Recovery remains nonzero, repeated PTO/retransmit produces repeated residual diagnostics, and the final result is correctly nonzero/not-settled;
- `reliable_udp_post_return_session_ack_withheld_fails`: Carrier Recovery is zero, Session remains outstanding, more than one residual diagnostic is emitted, and the final result is correctly nonzero/not-settled.

Do not restore premature first-timeout terminalization to satisfy those stale cardinality assertions. Rebind both oracles to final/terminal residual truth and positive-transition cardinalities, then restore a truthful exact-head stable gate.

## H-R9-058: narrowed predicate is directionally correct, but not closed

`recv_udp_delivery_ack` no longer uses the false predicate `confirmed_watermark(stream) >= end`; it now calls `SessionRuntime::is_confirmed_range(stream, offset, len)`. This correctly removes the obvious prefix-watermark equivalence.

But H-R9-058 is not closed for two reasons.

First, the attempted repair did not add the requested discriminator negatives. The new CLI regression proves only the positive case: a fresh authenticated exact duplicate for a previously confirmed `offset=0,len=16` range is accepted-empty. The existing unexpected-ACK test uses a far unrelated `offset=4096,len=16`; it does not prove that an interior/subrange or zero-length ACK below the watermark is rejected. Restoring the old broad watermark predicate can therefore still escape the intended exact-range safety oracle unless dedicated negative cases are added.

Second, the chosen witness creates the new H-R9-059 resource defect below.

## H-R9-059 — HIGH — Session-lifetime confirmed-range history is not bounded/accounted by the operation

`SessionRuntime` now owns:

```text
confirmed_ranges: BTreeMap<StreamId, BTreeSet<(u64, u64)>>
```

Every positive `delivery_ack` inserts one historical `(offset,end)` tuple. The set is cleared only with the whole runtime state; it is not pruned when the current bounded R9 operation settles or when a logical record leaves current ownership.

This is a lifetime historical ACK ledger, not bounded current-operation witness state. It conflicts with the H-R9-058 repair contract, which explicitly prohibited adding unbounded ACK history or inventing TTL/LRU/history-size policy.

It also bypasses the resource accounting that `SessionRuntime` otherwise exposes. `max_queue_records`, `max_queue_bytes`, send/session windows and `max_total_bytes` account record/payload state, but the B-tree node metadata in `confirmed_ranges` is not charged to those limits. `queue_send` accepts non-empty records down to one byte, and each independently confirmed record can add a distinct retained tuple. Thus retained metadata grows with the number of historical confirmations rather than current outstanding ownership and can materially exceed the intended byte-accounting envelope before the existing cumulative payload ceiling stops further sends. No new numeric cap may be invented by the reviewer or coding agent to patch this.

This is HIGH because `SECURITY.md` requires per-connection/global memory/resource bounds, item-4 explicitly includes algorithmic resource boundedness, and this repair creates retained per-confirmation state outside the current accounting model.

## Smallest allowed repair contract for H-R9-059 + H-R9-058

Preserve the accepted H-R9-056 semantic direction without changing Session/Carrier/ACK architecture:

1. Remove the Session-lifetime `confirmed_ranges` history and the general `SessionRuntime::is_confirmed_range` witness unless it can be made strictly current-operation-owned without a new capacity/retention policy. Do not add TTL, LRU, history-size or other new policy numbers.
2. Put the exact duplicate witness at the current R9 operation owner, where the logical records are already finitely admitted. A minimal shape is a caller-owned collection of exact `(session, stream, offset, len)` records moved/recorded when the first genuine Session ACK exact-matches an admitted outstanding record. Its cardinality must derive from the operation's already-bounded admitted records, not from Session lifetime.
3. A fresh authenticated exact duplicate of one of those already-confirmed current-operation records is classification-only accepted-empty: malformed budget unchanged; no second `SessionRuntime::delivery_ack`; no second window release / `AckReleased` / `Resumed`; no Carrier substitution.
4. Same-session authenticated feedback that is not an exact admitted current-operation record remains fail closed even when its end is below the confirmed watermark. This includes interior/subrange, wrong stream, wrong offset, wrong length, overflow and `len == 0`.
5. Identical authenticated-envelope replay remains crypto replay rejection. The accepted-empty case is a freshly sealed envelope containing the same exact Session ACK semantics.
6. Clear the operation-owned witness at operation/lifecycle completion. It must not become a general Session history store.

Required deterministic regressions must turn red if the broad watermark predicate is restored:

- positive: confirm an admitted exact range, then send a freshly sealed exact duplicate and observe accepted-empty with zero malformed and no second logical transition;
- negative: after the same confirmation, send fresh authenticated interior/subrange feedback whose `end <= confirmed_watermark`; it must enter bounded malformed/unexpected handling, not accepted-empty;
- negative: fresh authenticated `len == 0` must be malformed, never accepted-empty even at watermark zero or above;
- retain wrong stream/offset/length and identical-envelope replay negatives.

Do not use direct arbitrary `SessionRuntime::delivery_ack` calls as proof that a range was an admitted current-operation record unless the test also binds that call to the same real operation ownership path. The witness is about exact admitted record identity, not merely about a method having advanced a watermark.

## Exact-head CI truth

GitHub-hosted Rust CI run `35365100701` for exact `1bbf7e8939ba2c6a73015a6a57b0c63ac6706018` completed **failure**:

- `nightly decode fuzz smoke`: success;
- `stable checks`: failure in `bash scripts/check.sh`;
- R9-4 positive ACK-loss/delayed-original/reorder process test: success;
- two stale post-return negative process tests: failure on residual-diagnostic count (`13 != 1` and `2 != 1` respectively).

All current CLI unit tests, including the new exact-duplicate positive test, passed. That green unit result is not sufficient to close H-R9-058/H-R9-059 because the required discriminator negatives are absent.

No accepted exact-tree developer-local provenance for exact `1bbf7e8` is asserted by this reviewer. A hosted failure cannot be overridden by an ancestor gate.

## Queue effect

H-R9-059 and H-R9-058 stay at the front. H-R9-057's source defect remains closed, but exact-head gate restoration is still dependency-ready work immediately behind the Session-ACK witness repair. After those are green, finish the full R9-4 cross-process closure and continue R9-5 through R9-12, dedicated independent R9 review, Q10/Q11/Q12 factual reconciliation, then repository-wide item-4 refill if still needed.

`READY_LIVE: none` remains authoritative. This finding is deterministic local correctness/resource/evidence work and creates no new unresolved real-network question. No D019, capacity-policy value, core protocol architecture, release/freeze/production authority or historical live evidence is changed by this note.
