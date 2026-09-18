# Independent R9 review — R9-4 ACK-loss / delayed-original reorder closure

Review anchor: exact developer-owned source/test tree `95d938859bbe7a42c3e2bf26e09729ba55de5cf0`; review performed against its reachable docs-only child `d4e19fde13ee6e64d277dfdc1fd7d940491b9068` on `main`.

This is bounded release-item-4 review support. It is not a release/security approval, protocol freeze, production authorization, WAN result, or performance conclusion.

## Scope and repository truth re-read

This pass re-read the current default-branch state and the required repository sources: `README.md`, `AGENTS.md`, `ROADMAP.md`, `IMPLEMENTATION_PLAN.md`, `SECURITY.md`, `docs/status.md`, `docs/standing-vps-lab-authorization.md`, `docs/vps-rental-window-priority.md`, `docs/decisions.md`, `docs/carrier-architecture.md`, `docs/specs/nekomusume-session-v0.md`, `docs/release-security-review-packet.md`, `docs/CHATGPT_HANDOFF.md`, plus the exact-current R9 client/server owners, `recv_udp_delivery_ack`, `SessionRuntime::receive`, reliable-UDP Recovery ownership, and `crates/neko-cli/tests/probe.rs`.

Developer-owned changes reviewed since H-R9-060 was opened:

- `7ea9a8442563ef7d0dd7f6c70acdba53a003fdff` — implementation + process-test evidence: the real post-return owner no longer discards Session-demux classification; it emits the helper's typed `accepted_empty_logical_ack` / negative classification through the normal diagnostic projection, and the real R9-4 process regression requires the accepted-empty classification while forbidding `unexpected_logical_ack`.
- `95d938859bbe7a42c3e2bf26e09729ba55de5cf0` — process-test evidence: every positive client Carrier retirement packet number must be present in the server's actually emitted Carrier-ACK packet-number set.
- `d4e19fde13ee6e64d277dfdc1fd7d940491b9068` — docs-only handoff; no runtime semantics.

No reviewer-local execution is claimed. Evidence classes remain separate below.

## Challenged R9-4 invariant

The bounded scenario must prove that losing exactly the first legitimate post-return Carrier ACK does not duplicate Session delivery or corrupt Recovery ownership:

1. original post-return Data is authenticated and accepted;
2. its first Session DeliveryAck is allowed through and populates the current-operation exact ACK witness;
3. exactly the first Carrier ACK is withheld, causing a real PTO and fresh-PN retransmission;
4. the server receives the stable logical retransmission, `SessionRuntime::receive` takes its exact-duplicate idempotent path, and the server freshly seals another same-range Session DeliveryAck;
5. the client classifies that fresh same-range Session ACK as accepted-empty with no second positive Session transition and without using Carrier feedback as logical delivery evidence;
6. delayed-original and fresh Carrier feedback retire only real packet identities, and the operation settles with zero Recovery in-flight ownership;
7. no policy/capacity/wire/crypto architecture is changed merely to satisfy the fixture.

## Source / oracle challenge and result

### One-shot suppression and real retransmission

The current server seam has both `delayed_post_ack` and a separate `delay_reorder_done` latch. The first post-return Carrier ACK is retained; the next post packet flips the latch, emits the delayed original first, then the fresh copy ACK; later packets ACK normally. The earlier alternating-suppression defect is therefore not present in the current owner.

On the client, PTO is driven by the executable `due_pto_probe` owner, retransmission is re-encoded/re-sealed onto a fresh secure-record packet number, exact wire bytes are used for congestion admission, and Recovery ownership is committed before the socket send only through the already-reviewed retransmit transaction path. A socket-send error rolls that retransmit reservation back and cannot emit the positive retransmit diagnostic.

### Session duplicate classification is exercised through the real owner

The post-return operation starts with one admitted `post_outstanding` record and an empty operation-owned `post_confirmed_acks` set. `recv_udp_delivery_ack` inserts `(stream, offset, len)` only when an authenticated Session DeliveryAck exactly matches and removes that admitted outstanding record. A later authenticated same-tuple ACK can emit `accepted_empty_logical_ack`; any other unmatched authenticated logical ACK emits `unexpected_logical_ack` and consumes the finite malformed budget.

The production post-return caller now forwards this typed classification to observable diagnostics rather than discarding it. The R9-4 process test requires `accepted_empty_logical_ack`, forbids `unexpected_logical_ack`, and requires exactly one positive `r9_udp_return_delivery_ack`. Because this bounded operation has exactly one admitted post-return tuple and the witness begins empty, the observed accepted-empty branch is necessarily the exact tuple populated by the first genuine Session ACK; it is not a pre-filled test-only witness. The helper's focused exact-duplicate regression separately asserts the accepted-empty branch leaves `malformed == 0`, while subrange/interior and zero-length authenticated feedback remain negative/fail-closed.

`SessionRuntime::receive` also independently implements exact-byte duplicate idempotence: an already-received same-offset/same-bytes record emits `DuplicateDedup` and returns success; conflicting bytes fail with `Protocol`. Thus the retransmission can cause another freshly sealed Session DeliveryAck without a second application-delivery mutation.

### Carrier identity and terminal settlement

The server's positive post-return Carrier ACK diagnostics are emitted from packet numbers extracted from authenticated datagrams it actually received on the post-return path. The process test requires at least two such ACK emissions and additionally requires every positive client `r9_udp_return_packet_ack` retirement to name a packet number in that server-emitted ACK set. Client positive retirement projection itself iterates the exact `acked_packets` returned by Recovery rather than a representative packet number.

The same process test requires a real PTO, at least one fresh retransmit, exactly one Session confirmation transition, no Carrier rejection, one terminal `r9_udp_post_return_settled`, and `remaining_in_flight == 0`. The settled event is emitted only after the receive/recovery loop exits on both `post_outstanding.is_empty()` and `rt.in_flight() == 0`; there is no later retransmit owner after that terminal emission in this operation. Prior H-R9-046 coverage remains the dedicated discriminator for post-retirement retransmit ordering and is not replaced by this R9-4 test.

## Finding disposition

**No new BLOCKER/HIGH was found in the bounded R9-4 surface. H-R9-060 and R9-4 are independently closed at exact source/test tree `95d938859bbe7a42c3e2bf26e09729ba55de5cf0`.**

This closure does not imply that every R9 continuation has been independently challenged. In particular, the following remain dependency-ready and intentionally separate:

- R9-5 adversarial Carrier feedback across every continuation;
- R9-6 send/admission/socket-outcome/resource boundedness, including the post-return **first-send** owner whose socket-send ordering is not covered by the retransmit rollback review;
- R9-7 process/result truth across all typed evidence domains;
- R9-8..R9-11 warm readiness, health/promotion, uncertain replay/dedup/cleanup, and terminal lifecycle;
- R9-12 final exact-tree provenance, then dedicated independent R9 review and Q10/Q11/Q12 reconciliation.

## Evidence truth

Developer-persisted local provenance for exact `95d9388` records `PYTHONDONTWRITEBYTECODE=1 bash scripts/check.sh` exit 0, `git diff --check` exit 0, a clean worktree, Linux x86_64 / rustc 1.98.0, and UTC 2026-09-18T17:58:47Z through 18:03:19Z. This remains developer-local evidence, not reviewer execution.

GitHub-hosted Rust CI run `35377470589` for exact `95d9388` completed successfully. Its `stable checks` job passed `bash scripts/check.sh`; its nightly job passed pinned `cargo fuzz build decode` and `cargo fuzz run decode -- -max_total_time=30 -max_len=8192`. Hosted CI is cross-evidence only.

Reviewer-local execution is not claimed in this pass; exact pushed source/test inspection, repository-persisted developer provenance, and hosted cross-evidence are kept distinct.

## Queue / live effect

Advance immediately to R9-5; do not wait for the next reviewer cadence. Preserve the deeper R9-6..R9-12, dedicated independent R9 review, Q10/Q11/Q12 reconciliation, and repository-wide item-4 refill lanes.

`READY_LIVE: none` remains authoritative. This closure creates no new unresolved real-network hypothesis and does not justify repeating HY2, soak, migration-back, endpoint/key migration, IPv6, PLPMTUD, package-lifecycle, or Experimental Track evidence. D019, retained-state capacity policy, signing/key custody/SBOM/publication policy, core Session/Carrier/ACK/crypto/wire architecture, and release/freeze/production authority remain untouched.
