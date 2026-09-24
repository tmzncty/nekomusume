# H-I4-122 — post-auth reply evidence boundary mismatch

**Severity:** HIGH — release-item-4 evidence reliability.  
**Reviewed repository anchor:** reachable `95a2dab577037bc9d4e91e1bc269ff6c5c167801`; developer source/test change under review is reachable `2e88bdffaaf62b7a543bb880950342c81cf7760f`.  
**Review class:** exact-current GitHub source/control-flow review only. No reviewer-local Rust/full-gate, hosted CI, WAN, fuzz, cross-platform, or performance execution is claimed.

## Finding

H-I4-121's contract narrowing is only partially complete.

Exact-current `failover_server` uses `--drop-post-auth-udp-replies` to suppress the authenticated Session `DeliveryAck` branch. In reliable-UDP mode, however, the canonical Carrier packet ACK is sent by `udp.send_to(&sealed_ack, peer)` before and outside that guard. The flag therefore does **not** suppress every authenticated post-auth UDP reply and, specifically, does not suppress reliable-UDP Carrier packet ACKs.

That source contradicts persisted/current wording:

- the source comment says the seam withholds every authenticated application-level UDP reply, explicitly naming both Session DeliveryAck and reliable-UDP Carrier packet ACKs;
- `docs/notes/post-auth-blackhole-rename-2e88bdf-20260925.md` repeats that the process seam withholds every post-authentication application-level UDP reply.

The integration regression `executable_loopback_post_auth_udp_reply_blackhole_drives_tcp_failover` does not pass `--reliable-udp`. It proves the narrower and still useful path condition that negotiation/authentication succeed and Session DeliveryAck replies are withheld from the first application record until health-driven cold TCP fallback completes. It does not prove Carrier packet ACK suppression.

A second wording residue remains: client-side retention comments still describe "hard UDP loss" / "all server UDP replies blackholed from the first datagram", and the integration assertion comment still says "Hard UDP loss -> health evidence drives failover". Those statements are incompatible with the H-I4-121 evidence boundary even though the executable flag/mode name was narrowed once.

## Why this remains HIGH

The exact-tree developer-local gate/provenance at `2e88bdf...` is valid execution provenance, but execution success cannot make an inaccurate path-condition description true. Leaving broader comments/machine label/closure wording in place would allow item-4 evidence to re-acquire the same class of overclaim H-I4-121 was meant to remove.

This finding does **not** establish a production transport correctness defect. It is an evidence/harness truth defect in the local middlebox-support slice.

## Smallest authorized closure

Prefer evidence narrowing; do not expand ACK behavior merely to preserve the current label.

1. Narrow the process seam and current evidence wording to authenticated **Session DeliveryAck** suppression after successful negotiation/authentication. A label such as `post_auth_session_delivery_ack_blackhole` / `--drop-post-auth-delivery-acks` (or equivalently precise wording) is acceptable.
2. Remove the false statement that reliable-UDP Carrier packet ACKs are suppressed unless a separate, justified source/test slice deliberately changes that behavior.
3. Remove residual "all server UDP replies" / "from the first datagram" / generic "hard UDP loss" wording from repaired owner comments and the current closure note. Historical reviewer notes remain historical; supersede rather than silently rewrite them.
4. Preserve the existing behavioral assertions: bounded child ownership, successful negotiation/authentication, `udp_health_failed`, cold fallback, three TCP DeliveryAck validations, 3 records / 48 bytes delivered, and no fabricated duplicate metric.
5. Do not touch H-I4-119/core ACK-PTO semantics, D019, Session/Carrier/wire/crypto architecture, policy values, release flags, or standing authorization.
6. Run the final pushed repaired source SHA through `PYTHONDONTWRITEBYTECODE=1 bash scripts/check.sh`, `git diff --check`, verify clean tree, and persist reachable developer-local exact-tree provenance with required metadata. No decoder/parser/crypto-framing change is implied; no mechanical fuzz run.

## Queue consequence

H-I4-122 is FRONT before R-MBOX-ROOTLESS-LOSS can be treated as evidence-closed. After the narrow wording/label repair and final exact-tree provenance, continue immediately to R-MBOX-REORDER-DELAY and retain the existing deep R-MBOX + 13-surface queue. `READY_LIVE: none` remains unchanged.

GitHub combined status for `2e88bdf...` exposed no status entries and no PR-triggered workflow run was visible in this reviewer pass; that is not hosted CI evidence.
