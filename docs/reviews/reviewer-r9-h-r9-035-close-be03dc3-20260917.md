# Independent bounded review — H-R9-035 closure and R9-2 next-lane preflight

Date: 2026-09-17

## Exact reachable scope

- developer source/test anchor: `be03dc31a8806fc6032655f5a1f755a63155f4de`
- prerequisite test anchor: `1f8bbb01fa08ebdd889ac70ccbb12c1185aa5a1e`
- handoff/docs head inspected: `478ed59e4188b5654b6c832619719fe79b46f1f7`
- inspected owners:
  - `crates/neko-cli/tests/probe.rs::reliable_udp_migration_back_reserves_final_record`
  - current post-return server emission owner in `crates/neko-cli/src/main.rs`
  - current post-return client dual-settlement owner in `crates/neko-cli/src/main.rs`
  - `docs/CHATGPT_HANDOFF.md`
  - applicable Session/Carrier evidence-separation contract in `docs/specs/nekomusume-session-v0.md`, `docs/carrier-architecture.md`, and D005/D064 in `docs/decisions.md`

This is an independent bounded source/oracle review. I did not execute a reviewer-local checkout or test command in this pass. GitHub-hosted Rust CI run `35168132685` for exact `be03dc3` completed successfully and is cross-evidence only. The repository handoff separately records developer-local clean exact-tree provenance for exact `be03dc3`; that provenance was not re-executed by this reviewer.

No decoder/parser/crypto-framing source changed in the reviewed developer commits, so this review does not request a new fuzz run merely for H-R9-035.

## H-R9-035 result — CLOSED, no new finding

The strengthened ordinary positive migration-back fixture now proves the missing C4 identities rather than only adjacent facts:

1. exactly one post-return client send is already required;
2. exactly one server `udp_return_packet_ack_sent` is already required;
3. exactly one client positive `r9_udp_return_packet_ack` is required and remains `applied=true, retired=true`;
4. `1f8bbb0` parses that actual Recovery-retirement packet number and binds it to the already cross-bound client-send/server-ACK packet number;
5. the exact Session transition remains one `stream=1, offset=48, len=16` event;
6. rejected and accepted-empty classifications remain forbidden on the ordinary positive path;
7. `be03dc3` requires exactly one `r9_udp_post_return_settled` with `stream=1, offset=48, remaining_in_flight=0`;
8. the fixture still proves settlement occurs after both actual ACK-domain transitions.

The current client owner only emits the positive Carrier event when the Recovery outcome actually retires `post_pn_client`, so the strengthened test is tied to a real state transition rather than a synthetic packet-number echo.

No contradictory correctness/evidence defect was found in this bounded H-R9-035 scope. H-R9-035 should remain closed unless new repository evidence contradicts these exact-current facts.

## READY_LOCAL 2 preflight — pure ACK-order seam is still genuinely open

The current post-return server already has the two canonical outputs needed for the challenge: one exact Session `DeliveryAck` and one canonical Carrier packet ACK produced from the same received post-return Data. The ordinary path sends Session ACK then Carrier ACK.

The existing `--send-stale-ack` / `--send-future-ack` path is **not** a valid substitute for the reverse-order challenge: it intentionally sends the legitimate Carrier ACK before the Session ACK while also injecting accepted-empty or rejected feedback. That violates READY_LOCAL 2's requirement to challenge order without a false/rejected/accepted-empty shortcut.

Use a narrow order-only test seam over the already-produced authenticated datagrams. It must not add another receive owner, deadline, malformed budget, ACK architecture, or policy value. The same current client loop is already structurally order-independent: it checks Session completion and Recovery `in_flight()==0` independently under one absolute deadline and routes Session vs Carrier outcomes through the same `recv_udp_delivery_ack` owner.

Acceptance for both deterministic arms:

- Session DeliveryAck -> Carrier packet ACK;
- Carrier packet ACK -> Session DeliveryAck;
- both processes succeed;
- exactly one Session transition `stream=1 offset=48 len=16`;
- exactly one positive Carrier retirement for the cross-bound post-return packet;
- client send == server Carrier ACK == client retirement packet number;
- zero rejected and zero accepted-empty classifications;
- exactly one settled event with `remaining_in_flight=0`;
- settled is strictly after both actual client-side ACK-domain transitions;
- the test must assert the intended client-observed transition order, not merely server send order.

Test/seam-only is the default implementation shape. If the stronger reverse-order oracle exposes a runtime contradiction, then repair the smallest current-semantic defect and add the discriminating regression.

## READY_LOCAL 3 preflight — Session present / post-return Carrier ACK withheld

There is historical `--suppress-r9-ack` coverage for the **initial** reliable-UDP Carrier-ACK path. Do not use a broad suppression that prevents the run from ever reaching migration-back/post-return ownership. The P4 lane needs suppression scoped to the post-return Carrier ACK only (or an exact-current refactor that is demonstrably equivalent).

Required proof at the client:

- exact positive Session transition `stream=1 offset=48 len=16`;
- no positive post-return Carrier retirement;
- Recovery remains nonzero/unsettled until the bounded terminal path;
- client exits nonzero on the bounded post-return settlement failure;
- no `r9_udp_post_return_settled` and no downstream health/failover/migration success continuation based on a false premise.

If current diagnostics cannot distinguish which evidence domain remains outstanding, a minimal classification-only terminal diagnostic is acceptable; it must not alter Session/Recovery state or introduce a new policy value.

## READY_LOCAL 4 preflight — Carrier ACK present / Session ACK withheld

The existing `--suppress-r9-dack` post-return seam already has the right runtime shape: the server withholds the post-return Session `DeliveryAck` while still sending the real Carrier ACK. The current process test is weaker than the handoff contract, so strengthen the oracle before changing runtime semantics.

Require:

- exactly one positive Carrier retirement for the exact cross-bound post-return packet;
- no `r9_udp_return_delivery_ack` transition for offset 48;
- logical Session confirmation remains outstanding until the bounded terminal failure;
- client exits nonzero;
- no settled marker and no downstream success continuation;
- server-side emitted Carrier packet number remains cross-bound to client send and client retirement.

Only repair runtime code if those stronger assertions expose a real contradiction.

## Exclusions / authority boundary

This review does not decide D019/source-retention, event-history/capacity values, signing/key custody/SBOM/publication, previous frozen-release policy, core Session/Carrier/ACK/crypto/wire architecture, destructive migration, RC/freeze/release/production authority, or adversarial-load capacity suitability.

No live/WAN question was created. `READY_LIVE: none` remains the truthful classification for this slice; existing VPS/HY2/migration-back/soak evidence must not be mechanically repeated.
