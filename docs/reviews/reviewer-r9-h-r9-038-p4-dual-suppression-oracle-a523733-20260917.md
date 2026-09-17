# H-R9-038 — P4 dual-suppression evidence/oracle review

**Severity:** HIGH (evidence/oracle correctness; no new runtime-state contradiction proven)

**Developer source/test anchor reviewed:** `a5237330b16ca199dc2c16bebcf728859c8a0491`

## Scope

Bounded independent review of the two new P4 built-binary negatives and their smallest post-return Carrier-ACK suppression seam:

- `crates/neko-cli/src/main.rs` post-return server owner around `post_ack_pack`, `--suppress-r9-dack`, and `--suppress-r9-post-return-pack`;
- `crates/neko-cli/src/main.rs` client post-return dual-settlement owner;
- `crates/neko-cli/tests/probe.rs::reliable_udp_post_return_carrier_ack_withheld_fails`;
- `crates/neko-cli/tests/probe.rs::reliable_udp_post_return_session_ack_withheld_fails`;
- current `docs/CHATGPT_HANDOFF.md` READY_LOCAL 1/2 acceptance contract;
- provisional Session/Carrier evidence separation in `docs/specs/nekomusume-session-v0.md` / carrier architecture.

Hosted Rust CI run `35183369645` for exact `a523733` completed successfully. That is hosted cross-evidence only; this review did not execute a developer-local exact-tree gate and does not relabel hosted CI as local provenance.

## Finding

The source seam itself is directionally consistent with the existing architecture: `--suppress-r9-post-return-pack` suppresses only the ordinary post-return Carrier ACK after the Session DeliveryAck, while `--suppress-r9-dack` suppresses only the Session DeliveryAck and leaves the Carrier ACK path intact. No new Session/Carrier/ACK architecture change is required by this finding.

The two new process tests are not acceptance-grade and can pass without proving the P4 invariant that neither evidence domain substitutes for the other.

### P4-A — Session ACK present, Carrier ACK withheld

Current test weaknesses:

1. `finish_server(server)` returns `(status, log)` but the test binds the status as `_srv_status` and never requires server success.
2. Client Session evidence is only `contains("r9_udp_return_delivery_ack")`; it is not exactly-once and is not bound to `stream=1, offset=48, len=16`.
3. Server evidence is only broad presence/absence of `udp_return_delivery_ack_sent` / `udp_return_packet_ack_sent`; cardinality and exact post-return identity are not checked.
4. The test does not prove the terminal residual state: Session complete while Recovery remains nonzero. Current client runtime has enough state (`post_outstanding`, `rt.in_flight()`) but emits no classification-only residual-domain diagnostic before the bounded post-return failure.
5. It does not explicitly exclude downstream success continuation (`r9_udp_post_return_settled`, accounting/summary/client-ok continuation) from a false settlement premise beyond the single settled-event absence.

A process that prints one Session event and later fails for an unrelated reason can satisfy the present oracle.

### P4-B — Carrier ACK present, Session ACK withheld

Current test weaknesses:

1. Server status is again discarded rather than required to succeed.
2. Client Carrier evidence is only `contains("r9_udp_return_packet_ack")`; the test does not require exactly one positive event with `applied=true, retired=true`.
3. The test does not require exactly one client post-return send, exactly one server Carrier ACK, exactly one client retirement, or three-way equality of their `packet_number` values even though exact-current diagnostics already expose those identities.
4. It does not require zero `r9_udp_return_packet_ack_rejected` / zero `r9_udp_return_packet_ack_accepted_empty`; a wrong classification could coexist with the broad positive substring and still pass.
5. It does not prove the terminal residual state: Recovery settled (`remaining_in_flight=0`) while the exact Session range remains outstanding. There is no classification-only residual-domain diagnostic before bounded failure.
6. Server Session suppression is asserted only as broad absence of `udp_return_delivery_ack_sent`; exact positive Carrier cardinality and exact post-return binding are not checked.

## Smallest repair contract

Do not redesign Session/Carrier/ACK/wire/crypto semantics.

1. Add one classification-only post-return terminal diagnostic at the existing bounded dual-settlement failure boundary, emitted only after reading the actual remaining state. It should report at least `session_outstanding=<count>` and `remaining_in_flight=<count>`. It must mutate no state and must never be treated as success.
2. Strengthen P4-A to require:
   - server success;
   - exactly one server post-return Session DeliveryAck, zero server post-return Carrier ACK;
   - exactly one client `r9_udp_return_delivery_ack` with `stream=1, offset=48, len=16`;
   - zero positive post-return Carrier retirement / zero rejected / zero accepted-empty substitute;
   - terminal diagnostic with `session_outstanding=0` and `remaining_in_flight>0`;
   - typed/nonzero client termination on the existing bounded post-return terminal path;
   - zero `r9_udp_post_return_settled` and zero downstream success continuation.
3. Strengthen P4-B to require:
   - server success;
   - exactly one client `r9_udp_post_return_sent`, exactly one server `udp_return_packet_ack_sent`, exactly one positive client `r9_udp_return_packet_ack` with `applied=true, retired=true`;
   - exact three-way packet-number equality;
   - zero server `udp_return_delivery_ack_sent` for post-return offset 48 and zero client `r9_udp_return_delivery_ack` for that range;
   - zero rejected/accepted-empty substitute;
   - terminal diagnostic with `remaining_in_flight=0` and `session_outstanding=1`;
   - typed/nonzero client termination on the existing bounded post-return terminal path;
   - zero `r9_udp_post_return_settled` and zero downstream success continuation.
4. Run focused process tests first, then on the final pushed developer SHA run the normal clean exact-tree developer-local gate (`PYTHONDONTWRITEBYTECODE=1 bash scripts/check.sh`, `git diff --check`, clean tree) and record provenance separately. No decoder/parser/crypto-framing change is expected, so fuzz is not required merely for this diagnostic/test repair.

If the stronger oracle reveals a runtime contradiction, repair only the smallest current-semantics defect and keep the two evidence domains separate.

## Exclusions

This review did not re-open D019, capacity/TTL/history policy, signing/SBOM/publication policy, previous-release compatibility, core Session/Carrier/ACK/crypto/wire architecture, RC/freeze/release authority, or any live/VPS question. `READY_LIVE` remains `none` unless a later concrete unresolved real-network question is produced.
