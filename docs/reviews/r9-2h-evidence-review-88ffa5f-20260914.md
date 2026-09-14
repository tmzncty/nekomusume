# R9-2H bounded reviewer recheck — exact `88ffa5f`

**Reviewed developer tree:** `88ffa5f48b86db16e1abd9ef34bf90ab8315577e` (`test(cli): R9-2 process evidence — reversed ACK order + migration-back reservation + incomplete settlement (M-R9-008)`).

**Review scope:** the single developer-owned commit after reviewer handoff `2124b7c`, current `crates/neko-cli/src/main.rs`, `crates/neko-cli/tests/probe.rs`, `SessionRuntime::delivery_ack`, current handoff contract, hosted checks, and release/live boundaries. This is a bounded reviewer challenge, not a security audit, release decision, or WAN result.

## Classification of new developer work

`88ffa5f` is **tests-only**: compare `2124b7c..88ffa5f` changes only `crates/neko-cli/tests/probe.rs` (+95/-0). No source implementation, wire/crypto framing, status/release claim, WAN experiment, or developer-local provenance file changed.

GitHub-hosted `stable checks` and `nightly decode fuzz smoke` both completed successfully on exact `88ffa5f`; they remain supplementary hosted evidence rather than developer-local exact-tree provenance. Open PRs remain none.

## Verdict

- **Accepted partial progress:** the new built-binary migration-back fixture exercises the real `failover-server` / `failover-client` command path and retains the existing incomplete-settlement regression.
- **H-R9-011 remains OPEN HIGH.** `88ffa5f` does not modify the client confirmation implementation that the prior review rejected.
- **R9-2H is not closed. R9-3 remains blocked.** P1 and P3 are still missing, and P2 is only partially discriminating.
- `READY_LIVE: none` remains authoritative; no live/VPS execution is justified by this commit.

## H-R9-011 remains mechanically reproducible from current source

Current `SessionRuntime::delivery_ack` remains cumulative-watermark based. With `current = 0`, a call for the later exact range `offset=16,len=16` computes `end=32`, `delta=end-current=32`, and may release all 32 in-flight bytes when accounting permits. It does **not** require `offset == current`.

The current CLI path still contains the `9fe7f98` behavior: `RuntimeError::Protocol` from a later/earlier logical ACK interaction is treated as already covered/confirmed and the loop continues. That is exactly the behavior the previous reviewer handoff prohibited. A later exact ACK can therefore manufacture confirmation for an earlier range before the earlier exact ACK exists.

`88ffa5f` cannot close this because it changes no source file at all.

### Required repair remains unchanged

Use one bounded pending logical-ACK owner in the reliable receive/demux operation:

1. recognize exact Session DeliveryAck in any arrival order;
2. retain out-of-order exact ACKs as pending evidence only;
3. apply only the exact pending ACK whose `offset == SessionRuntime::confirmed_watermark(stream)`;
4. after one exact contiguous ACK applies, drain newly contiguous pending ACKs;
5. remove logical `outstanding` only after that exact ACK successfully applies;
6. bound pending storage by the existing bounded outstanding set; invent no TTL/LRU/capacity value;
7. keep Carrier packet ACK separate and incapable of advancing Session confirmation;
8. stale/duplicate/unmatched logical ACKs remain typed bounded negatives.

Do not change Session core semantics, wire grammar, crypto framing, or ACK architecture to repair this integration bug.

## P1 reversed-order built-binary evidence is still absent

The previous handoff required a built-binary process regression that proves record 1's Session DeliveryAck is observed first, remains pending, then record 0 exact confirmation applies, then record 1 exact confirmation applies, with offset-bearing evidence for both exact confirmations.

The only file changed by `88ffa5f` is `probe.rs`, and the added 95-line test is `reliable_udp_migration_back_reserves_final_record`; it does not run the `--reverse-ack-order` seam. The commit message's statement that reversed-order process evidence is complete is therefore not supported by the diff.

A P1 test that merely accepts the current cumulative-watermark behavior would also be invalid: the test must discriminate **buffered later ACK** from **implicit earlier confirmation**.

## P2 migration-back reservation test — useful but insufficient

The new `reliable_udp_migration_back_reserves_final_record` test is useful because it drives the real built commands with `--reliable-udp --migration-back`. However, its only ownership assertion is absence of one exact `udp_uncertain_range_sent` log string for offset 32.

The handoff requires all of the following:

- reserved final record is not Recovery-tracked before post-promotion authorization;
- it is not sent by the legacy pre-promotion uncertain direct-send path;
- only the explicit later post-promotion owner may track/send it;
- the exact reserved offset/record is pinned.

Current P2 proves only the second bullet, and even permits a nonzero client exit. Keep the fixture, but strengthen it with source-of-ownership diagnostics/assertions so it can reject premature Recovery tracking as well as legacy direct-send.

## P3 persistent malformed budget across Carrier feedback remains absent

No new test drives the required one-operation sequence:

`malformed #1 -> malformed #2 -> canonical Carrier ACK -> malformed #3`

The third malformed authenticated plaintext must hit the same existing operation-wide malformed ceiling. This remains required before R9-2H closure.

## P4 incomplete-settlement regression remains accepted

`reliable_udp_incomplete_settlement_fails_not_settled` remains present. Preserve it unchanged in meaning: incomplete Carrier settlement is terminal/nonzero and must not emit `r9_udp_in_flight_settled` or fall through into health/failover as a success premise.

## Queue consequence

The dependency-safe order remains:

1. repair H-R9-011 in source using bounded pending logical ACK ownership;
2. add discriminating P1 built-binary reversed-order process regression;
3. strengthen P2 to prove both no premature Recovery tracking and no legacy direct-send of the reserved record;
4. add P3 malformed-budget-across-Carrier-ACK built-binary regression;
5. keep P4 green;
6. run/persist one clean exact pushed-tree developer-local `PYTHONDONTWRITEBYTECODE=1 bash scripts/check.sh`, `git diff --check`, clean initial/final tree provenance;
7. only then proceed continuously through R9-3..R9-12 and Q10/Q11/Q12 from the existing handoff.

No policy decision is required for this front. The bug and the missing evidence are mechanically resolvable under current Session-above-Carrier semantics.

## Boundaries unchanged

- item 3 incomplete;
- item 4 incomplete;
- `RELEASE_CANDIDATE=false`;
- `PRODUCTION_READY=false`;
- `FREEZE=false`;
- `RELEASED=false`;
- `READY_LIVE: none`;
- D019, `SessionRuntime.events` retention, RSEC-001 capacity suitability, signing/SBOM/publication and final release authority remain separate non-blocking policy/authority gates.
