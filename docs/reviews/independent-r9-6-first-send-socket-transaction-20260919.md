# Independent R9 review — H-R9-061 first-send socket transaction divergence

Review anchor: exact reachable `main` tree `4816e46e4a95634b3abe7392b3ddd6068e7de35a`. Latest developer-owned source/test tree remains `95d938859bbe7a42c3e2bf26e09729ba55de5cf0`; later reachable commits through `4816e46` are review/handoff documentation only.

This is bounded release-item-4 review support. It is not release/security approval, protocol freeze, production authorization, WAN evidence, or a performance conclusion.

## Repository truth re-read

This pass re-read the current default-branch state and the required repository sources: `README.md`, `AGENTS.md`, `ROADMAP.md`, `IMPLEMENTATION_PLAN.md`, `SECURITY.md`, `docs/status.md`, `docs/standing-vps-lab-authorization.md`, `docs/vps-rental-window-priority.md`, `docs/decisions.md`, `docs/carrier-architecture.md`, `docs/specs/nekomusume-session-v0.md`, `docs/release-security-review-packet.md`, `docs/CHATGPT_HANDOFF.md`, plus the exact-current reliable-UDP first-send/retransmit owners, `ReliableUdpRuntime`, Recovery ownership and current process tests.

New commits reviewed since the previous reviewer handoff anchor `16210ad5a8fcede394cab95cbe1458cb9ba07271`, by class:

- `7ea9a8442563ef7d0dd7f6c70acdba53a003fdff` — implementation + process evidence: H-R9-060 post-return Session-demux classification projection.
- `95d938859bbe7a42c3e2bf26e09729ba55de5cf0` — test/evidence: binds positive client Carrier retirement packet identities to server-emitted Carrier ACK packet identities.
- `d4e19fde13ee6e64d277dfdc1fd7d940491b9068`, `8c21e24e436379abc13b41e4f163da61b5720193`, `67028be796d7c0fe25e030c7bc692e9428e41eb8`, `b9f0dc5467771f2398d06f2f6a66ebb3a36c1111`, `4816e46e4a95634b3abe7392b3ddd6068e7de35a` — documentation/review/handoff only; no additional runtime semantics.

R9-4/H-R9-060 remains independently closed at exact source/test tree `95d9388`. R9-5 remains a bounded no-finding review across current Carrier-ACK continuations. This pass advances into the already queued R9-6 first-send/socket-outcome ownership challenge.

## H-R9-061 — HIGH correctness/evidence: first-send Recovery commit precedes socket outcome

The current post-return reliable-UDP first-send owner performs these operations in order:

1. congestion admission on the exact sealed wire length;
2. derives the authenticated secure-record packet number;
3. commits `ReliableUdpRuntime::on_packet_sent(...)`, which reserves retransmit plaintext, records Recovery sent state/Reno charge and packet-to-frame ownership;
4. emits positive `r9_udp_post_return_sent` evidence naming that packet number;
5. only afterwards, unless the deliberate `--drop-r9-data` loss seam is active, calls the real `UdpSocket::send_to` and unwraps its result.

Therefore a real OS/socket send failure occurs *after* Recovery ownership and after a positive `sent` diagnostic have already been committed. The datagram may never have been accepted by the socket, yet repository evidence has already claimed that it was sent. No first-send rollback is invoked on that error path. The process subsequently aborts through `unwrap`, so the in-memory state does not survive process death, but the false positive diagnostic already escaped and the first-send transaction has not respected the same reservation/commit/abort boundary already enforced for retransmissions.

The two earlier reliable-UDP first-send owners have the same Recovery-before-socket ordering. Their positive diagnostics happen after `send_to`, so they do not have the same escaped-positive-evidence defect, but a real send error still has no rollback path before process termination. R9-6 explicitly requires challenging first-send and retransmit transactions across executable owners, so the repair must not create a one-off post-return exception while leaving the other reliable first-send owners semantically divergent.

This is distinct from the deliberate `--drop-r9-data` seam. That seam intentionally commits Recovery ownership and suppresses the wire send to model a packet that is lost after transport ownership; its retained Recovery state is required to drive PTO/retransmission. An ordinary socket error is not controlled network loss and must not silently inherit that meaning.

The retransmit owner is already shaped correctly under H-R9-051: exact-wire admission is followed by a reservation, a real socket `Err` calls `abandon_retransmit`, and only socket success emits `r9_udp_retransmit_sent` and advances the active retransmit packet identity. Do not regress that path while repairing first sends.

## Smallest dependency-ready repair contract

Current committed semantics are sufficient to repair this without changing Session/Carrier/ACK/crypto/wire architecture or inventing a policy value.

1. Preserve exact-wire congestion admission and the secure-record sequence/nonce as the only packet-number source.
2. Give every executable reliable-UDP first-send owner an explicit bounded socket-outcome transaction (a shared helper/API is preferred if it is the smallest shape):
   - reserve the current packet copy/Recovery/Reno/plaintext ownership only after admission;
   - on real socket success, commit positive `sent` evidence and keep the ownership;
   - on real socket error, abort exactly that packet copy before any positive sent evidence can escape; remove its Recovery/Reno and packet-to-frame ownership, and ensure the aborted PN does not become ACK-valid history;
   - preserve the stable retained frame/plaintext only where existing retry semantics require it; do not manufacture a second plaintext/history owner.
3. Keep deliberate controlled-loss injection separate. `--drop-r9-data` must still represent owned-but-lost transport work so PTO can recover it. It may keep the dedicated `r9_udp_post_return_data_dropped` classification, but it must not be confused with a successful OS socket send.
4. A failed socket reservation must not advance any active packet identity, produce Carrier/Session success evidence, alter RTT/PTO from feedback that never existed, or permit the failed PN to be acknowledged as historically sent.
5. Add a deterministic discriminator against the *production first-send owner*, not a copied boolean/helper-only oracle. Inject a socket/send outcome after successful admission and prove the failure path leaves Recovery/Reno/packet-copy ownership uncharged, emits no positive sent event, and leaves the failed PN non-ACK-valid. Pair it with a success control using a fresh legal PN that does emit positive sent evidence and can settle normally.
6. Include all current reliable-UDP first-send call sites in the ownership audit. The already-correct retransmit socket-error regression remains a separate control.
7. Do not introduce TTL/LRU/history-size/capacity/security numeric policy, change D019, redesign ACK architecture, or change the deliberate packet-loss seam into a socket-error seam.

Any source/test repair must finish with developer-local clean exact-tree evidence on the final pushed SHA:

```text
PYTHONDONTWRITEBYTECODE=1 bash scripts/check.sh
git diff --check
```

Record exact pushed SHA, UTC start/end, exit codes, OS/arch, stable Rust version and clean-tree state. No decoder/parser/crypto-framing change is required by this finding, so pinned decode fuzz is not mechanically required unless the implementation actually touches those surfaces.

## Evidence truth

The GitHub-hosted Rust CI for reachable HEAD `4816e46e4a95634b3abe7392b3ddd6068e7de35a`, run `35382804116`, completed successfully. That is hosted cross-evidence only and does not inject a real first-send socket failure after Recovery admission, so it cannot close H-R9-061.

Reviewer-local execution is not claimed. The reviewer environment could not resolve `github.com` for a local clone/check, so this finding is based on exact pushed source/test inspection, repository-persisted developer provenance already recorded for the current source/test tree, and hosted CI kept as distinct evidence classes.

## Queue / live effect

H-R9-061 is a correctness/evidence HIGH and is the front of R9-6. External coding work should repair and gate it immediately, then continue the remaining R9-6 ownership/resource-boundedness audit without waiting for reviewer cadence. R9-7 through R9-12, dedicated independent R9 review, Q10/Q11/Q12 factual reconciliation, and repository-wide item-4 refill remain dependency-ready behind it.

`READY_LIVE: none` remains authoritative. This finding is fully deterministic local correctness/evidence work and creates no unresolved real-network question. Do not repeat HY2, warm-failover, soak, package-lifecycle, migration-back, endpoint/key migration, IPv6, PLPMTUD or Experimental Track evidence merely because the VPS is still rented. D019, retained-state capacity policy, signing/key custody/SBOM/publication policy, core architecture and release/freeze/production authority remain untouched.
