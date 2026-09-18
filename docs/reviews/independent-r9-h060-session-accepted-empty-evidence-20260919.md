# Independent R9 review — H-R9-060 current-operation duplicate Session ACK evidence

Review anchor before this note: exact `bcc460a5a0a8cd8b41a16bb24b8119a3f6ec5641` on `main`; latest developer-owned source/test tree is exact `8d2eea9ec66fb4260c93a86e683d77f47faa9923` (the anchor is its docs-only child).

This is bounded independent release-item-4 review support. It is not a release/security approval, protocol freeze, production authorization, WAN result, or performance conclusion.

## Scope re-read

The current repository truth was re-read, including `README.md`, `AGENTS.md`, `ROADMAP.md`, `IMPLEMENTATION_PLAN.md`, `SECURITY.md`, `docs/status.md`, `docs/standing-vps-lab-authorization.md`, `docs/vps-rental-window-priority.md`, `docs/decisions.md`, `docs/carrier-architecture.md`, `docs/specs/nekomusume-session-v0.md`, `docs/release-security-review-packet.md`, `docs/CHATGPT_HANDOFF.md`, the exact-current R9 demux / post-return server and client owners, `crates/neko-cli/tests/probe.rs`, and exact-source hosted CI.

Developer-owned source/test commits reviewed since the previous finding are:

- `5f99a51a91b477b58f0b41d9b63e562ae2d60dea` — removes Session-lifetime `confirmed_ranges` and moves the exact duplicate witness to the bounded current operation;
- `8d2eea9ec66fb4260c93a86e683d77f47faa9923` — adds the zero-length discriminator and rebinds stale negative residual oracles.

No reviewer-local execution is claimed. Repository-persisted developer-local provenance and GitHub-hosted CI are kept as separate evidence classes.

## H-R9-058 / H-R9-059 repair review: no source defect found in the repaired witness shape

The repaired demux now owns a caller-provided `BTreeSet<(stream, offset, len)>`. The tuple is inserted only when an authenticated Session `DeliveryAck` exact-matches a currently admitted `outstanding` record. A later freshly authenticated exact duplicate consults that same current-operation set and is classification-only accepted-empty. An unmatched authenticated logical ACK still consumes the bounded malformed budget and fails closed at the existing bound.

The Session-lifetime historical set introduced by `08cbdaf6` is removed from `SessionRuntime`; the current witness lives in the failover operation and the post-return operation and therefore dies with that operation. This closes the H-R9-059 retained-history defect without inventing TTL/LRU/history-size/capacity policy. Dedicated current tests also reject an interior/subrange tuple and `len == 0`, so restoring the broad `confirmed_watermark >= end` predicate is no longer compatible with those negatives.

Exact developer source/test `8d2eea9` has GitHub-hosted Rust CI run `35372825479`: `stable checks` succeeded, and the nightly decode fuzz-smoke job also succeeded. The current handoff separately records a clean developer-local exact-tree gate for `8d2eea9`; this reviewer does not relabel that as reviewer-local execution.

## H-R9-060 — HIGH — R9-4 cannot yet prove that the real duplicate Session ACK took the repaired accepted-empty path

The R9-4 production scenario now has the right ingredients: the server withholds the first post-return Carrier ACK, the client reaches a real PTO and sends a fresh-PN retransmission, and the server processes the stable logical Data again and freshly seals another Session `DeliveryAck` for the same `(stream, offset, len)` range.

However, the post-return caller invokes `recv_udp_delivery_ack` with a no-op diagnostic callback:

```text
&mut |_| {}
```

Inside the demux, the only observable classification when the second exact Session ACK hits `confirmed_acks` is the callback string `accepted_empty_logical_ack`; the helper then `continue`s and returns no typed outcome for that classification. Therefore the real post-return owner discards the only evidence that distinguishes the intended repaired branch from another non-terminal receive iteration.

The current process regression `reliable_udp_ack_loss_delayed_original_reorder_settles` checks PTO/retransmit existence, exactly one positive `r9_udp_return_delivery_ack`, no Carrier rejection, at least two server Carrier ACK sends, zero-in-flight settlement, and final success. It does **not** assert any client-side exact-duplicate Session accepted-empty event, does not prove the malformed counter stayed unchanged on that duplicate, and does not prove `unexpected_logical_ack` was absent for the duplicate.

The unit positive `reliable_demux_accepts_exact_duplicate_ack_for_confirmed_range` is also not a substitute for the missing process discriminator: it pre-populates `confirmed_acks` directly instead of first populating the witness through the same real current-operation exact-match path. That unit test proves the lookup predicate, not the cross-process ownership transition that R9-4 claims.

Thus exact `8d2eea9` can stay fully green even if the actual R9-4 retransmission does not demonstrate the repaired H-R9-056/058/059 accepted-empty path. This is an acceptance/evidence HIGH, not a newly proven Session state-machine defect.

## Smallest repair / closure contract

Preserve the current Session/Carrier/ACK architecture and the current bounded caller-owned witness. Do not redesign the ACK format or add historical Session state.

1. Make the real post-return owner retain an observable typed classification for an exact duplicate Session `DeliveryAck`; do not leave the production callback as a sink for this event. A small caller-visible diagnostic/event is sufficient; changing transport semantics is not required.
2. Bind that event to the operation-owned exact tuple and expose enough current-operation state to prove the duplicate charged **zero** malformed budget and caused **zero** second Session transition. Reuse existing counters/state where possible; do not invent a new capacity/security policy.
3. Strengthen the real R9-4 process test so the witness is populated by the first genuine exact Session ACK, then the fresh-PN retransmission causes the server to freshly seal a same-range duplicate ACK, and the client observes at least one exact Session accepted-empty classification for that range.
4. The same test must require exactly one positive `r9_udp_return_delivery_ack` / Session confirmation transition for the range, no `unexpected_logical_ack`, no malformed-budget charge for the exact duplicate, and no Carrier-domain substitution.
5. Preserve the existing negatives: fresh authenticated subrange/interior/wrong tuple/zero-length feedback remains malformed; identical authenticated-envelope replay remains crypto replay rejection.
6. Finish the rest of the existing R9-4 contract in the same coherent slice rather than creating more harness-only tickets: parse original and retransmit packet identities; prove server delayed/sibling Carrier ACKs bind to real transmitted Recovery-owned identities; every positive client retirement is in that real identity set; stale feedback is accepted-empty only where current semantics allow it; and no retransmit occurs after the terminal settled event. Final `in_flight` remains zero and retained frame/plaintext ownership is released.
7. Run the normal exact pushed developer-tree gate (`PYTHONDONTWRITEBYTECODE=1 bash scripts/check.sh`, `git diff --check`, clean tree) and persist truthful provenance. No wire/parser/crypto-framing change is required for this evidence repair, so do not run extra fuzz solely because of this finding; hosted workflow fuzz remains separate cross-evidence if it runs automatically.

The regression must turn red if the post-return duplicate is silently swallowed, charged as malformed, converted into a second Session transition, or if the server seam reverts to alternating ACK suppression.

## Queue effect

H-R9-058 and H-R9-059 source/resource repairs may remain closed. H-R9-060 is now the front blocker for full R9-4 closure. After the coherent R9-4 closure and exact-tree gate, continue immediately through R9-5 adversarial Carrier feedback, R9-6 ownership/resource boundedness, R9-7 evidence truth, R9-8..R9-11 failover/lifecycle challenges, R9-12 final provenance, dedicated independent R9 review, Q10/Q11/Q12 factual reconciliation, then repository-wide item-4 refill if still needed.

`READY_LIVE: none` remains authoritative. This is deterministic local evidence/correctness work and creates no new unresolved real-network hypothesis. No D019 value, retention/capacity policy, core protocol architecture, historical live artifact, or release/freeze/production authority is changed by this finding.
