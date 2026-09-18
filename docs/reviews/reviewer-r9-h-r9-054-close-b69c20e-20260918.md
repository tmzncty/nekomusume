# Independent bounded review — H-R9-054 send-side packet-number reuse closure

Date: 2026-09-18

Exact developer source/test anchor: `b69c20e050a5ce777608ffa8a2260fb6a6cf42b2`

Reviewed handoff anchor: `6853d369001049cc76eb846c86d3a3e76b5c7786`

Outcome: **NO BLOCKER/HIGH found in this bounded closure review. H-R9-054 may remain closed.** This does not close R9-3; H-R9-042 remains dependency-ready local work.

## Claim challenged

After a genuinely sent packet number `N` has become the committed send high-water and later packet number `M > N` is reserved then aborted before successful send, the committed high-water must remain `N`. A late/duplicate ACK for genuinely sent-and-retired `N` remains legal, an ACK for aborted `M` fails closed, and the send owner must reject future packet-number reuse at or below the committed watermark without adding Recovery/Reno/frame ownership. A genuinely fresh packet number above the watermark remains admissible.

## Exact-current owners inspected

- `crates/neko-reliable/src/lib.rs`: `Recovery::on_sent`, `Recovery::abandon_sent`, `Recovery::on_ack`, committed `largest_sent` / reservation watermark behavior.
- `crates/neko-carrier/src/lib.rs`: `ReliableUdpRuntime` / `PathRecovery` send owner and the H-R9-054 path-recovery discriminator added at `4761827b7edd4ad149c884f47f1232738ca5377e`.
- `crates/neko-cli/src/main.rs`: executable `admit_retransmit` owner and the H-R9-054 CLI discriminator added at `d995afddc819964a5f994eba960071e840334337`, formatting-only follow-up `b69c20e050a5ce777608ffa8a2260fb6a6cf42b2`.
- Current `docs/CHATGPT_HANDOFF.md`, status/release evidence boundaries, carrier architecture and Session/Carrier separation.

## Challenge result

The current Recovery owner checks packet-number monotonicity before mutating send ownership. Aborting the top reservation restores the prior committed watermark rather than deriving history from the current in-flight map, while ACK validation uses that committed high-water instead of requiring current `sent` membership. That preserves legal late/duplicate ACK classification for genuinely sent-and-retired identities while keeping aborted pre-send identities outside ACK-valid history.

The Carrier-level discriminator creates the required history shape and then explicitly refuses reuse at the committed watermark (`3`) and below it (`2`) with unchanged `in_flight()`, followed by a fresh packet (`5`) admitted as the paired positive control. The executable CLI/runtime discriminator independently attempts reuse of committed `N=0` and current watermark `1`; both are refused with Recovery in-flight and Reno bytes unchanged, then fresh packet `101` is admitted. These two tests now discriminate a regression that weakens the real send owner as well as the lower seam.

No current evidence contradicts H-R9-054 closure. I did not find a reachable current caller that creates overlapping unresolved packet-number reservations requiring a different nested-reservation policy; that case is excluded rather than silently generalized.

## Evidence separation

I did **not** execute reviewer-local tests or a reviewer-local clean exact-tree gate in this automation runtime; no such pass is claimed here. The repository records developer-local clean exact-tree provenance for `b69c20e` separately.

GitHub-hosted Rust CI is only cross-evidence: run `35331703441` on exact `b69c20e050a5ce777608ffa8a2260fb6a6cf42b2` completed successfully, and run `35332312955` on docs-only descendant `6853d369001049cc76eb846c86d3a3e76b5c7786` also completed successfully. No live/WAN evidence and no performance conclusion are introduced by this review.

No decoder/parser/crypto-framing code changed in the reviewed developer slice, so the decode fuzz requirement is not mechanically applicable to H-R9-054.

## Exclusions / next dependency-ready work

This bounded review does not re-review the entire reliable-UDP state machine, does not choose any D019/capacity/TTL/LRU/security value, does not alter Session/Carrier/ACK/crypto/wire architecture, and does not make RC/freeze/release/production decisions.

R9-3 remains open specifically on H-R9-042: add a deterministic exact-time negative at `deadline_us - 1` showing no PTO/retransmit/ownership mutation, paired with an at/after-deadline positive. Current source already contains both caller-side `now_us >= deadline_us` gating and Recovery-side early-return semantics; the remaining work is an acceptance-grade discriminator unless a focused regression disproves those semantics.
