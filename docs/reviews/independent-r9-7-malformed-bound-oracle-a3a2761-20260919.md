# Independent R9-7 review — H-R9-069 malformed-bound terminal oracle

**Review anchor:** exact developer-owned source/test commit `a3a27617819c8ca9d8a061b5f78f1a76db0c066d`.

**Classification:** HIGH — evidence / process-result oracle gap. The H-R9-068 production repair itself remains source-level plausible and fail-closed; this finding is about the regression being able to pass without proving the terminal cause and later-valid-feedback non-resurrection claim that closes H-R9-068.

## Inspected owners

- `crates/neko-cli/src/main.rs`
  - `MAX_POST_HANDSHAKE_MALFORMED`
  - `recv_udp_delivery_ack`
  - post-return `failover_client` dual-settlement loop and its terminal `Err(e)` branch
  - `failover_server` `--flood-r9-ack` seam and the subsequent legitimate Session DeliveryAck / Carrier ACK sends
  - terminal `fail(...)` / final client success output boundary
- `crates/neko-cli/tests/probe.rs`
  - `reliable_udp_post_return_malformed_bound_is_terminal`
  - neighboring R9-4 / H-R9-067 process-result regressions used as oracle-quality controls
- repository governance/spec boundaries in `AGENTS.md`, `SECURITY.md`, `docs/decisions.md`, `docs/carrier-architecture.md`, `docs/specs/nekomusume-session-v0.md`, `IMPLEMENTATION_PLAN.md`, `docs/status.md`, `docs/release-security-review-packet.md`, standing VPS authorization, rental-window priority, and current `docs/CHATGPT_HANDOFF.md`.

## Challenged claim

H-R9-068 closure claims that after the existing malformed budget of fresh authenticated but semantically unadmitted Session DeliveryAcks is exhausted, the executable post-return owner terminates specifically on `UDP delivery acknowledgement malformed bound exceeded`, and later otherwise-valid Session/Carrier feedback cannot resurrect final success.

The source repair now distinguishes an ordinary `UDP delivery acknowledgement timeout` from all other helper errors and calls terminal `fail(e)` for the latter. That source path is consistent with the intended fail-closed semantics.

## Concrete oracle counterexample

The new process regression does **not** prove that terminal cause or the later-valid-feedback challenge:

1. Server `--flood-r9-ack` currently sends `for _ in 0..=3`, i.e. four bad ACKs, while `MAX_POST_HANDSHAKE_MALFORMED == 3`. The test therefore does not pin the existing bound exactly; an accidental off-by-one relaxation to four could remain green.
2. Client assertions require only:
   - at least one `unexpected_logical_ack`;
   - nonzero exit;
   - absence of settled/final-success output.
   They do **not** require three bounded unexpected classifications or the terminal stderr/cause `UDP delivery acknowledgement malformed bound exceeded`.
3. The test discards both `srv_status` and `server_log`. It therefore does not prove that the server actually progressed past the malformed flood and emitted the otherwise-valid post-return Session DeliveryAck and Carrier ACK which are supposed to arrive after the bound is crossed.

A regression in which the client sees one bad ACK and then terminates for an unrelated receive/server failure can satisfy every current assertion. Likewise, if the server fails before sending the legitimate settlement feedback, the test can still pass. In either case the advertised discriminator — malformed-bound terminality that cannot be rescued by later valid feedback — has not been exercised.

This is an item-4 / R9-7 evidence-truth defect rather than a demonstrated new transport-state defect in the current `Err(e)` implementation.

## Required smallest repair

1. Keep `MAX_POST_HANDSHAKE_MALFORMED` unchanged. Do not invent a new numeric policy.
2. Make the flood seam send **exactly the existing bound** (`MAX_POST_HANDSHAKE_MALFORMED`) of independently sealed, authenticated, semantically unadmitted DeliveryAcks.
3. Strengthen the real process regression so it proves the terminal discriminator itself:
   - client observes exactly / at least the existing bound of `unexpected_logical_ack` classifications before termination;
   - client exits nonzero;
   - stderr or another existing typed terminal observation proves `UDP delivery acknowledgement malformed bound exceeded` rather than an unrelated socket/server/timeout failure;
   - no `r9_udp_post_return_settled`, `failover_client_ok`, final client `summary`, or `ordered_records_complete` appears.
4. Prove the server really proceeded to send the otherwise-valid post-flood Session DeliveryAck and Carrier ACK. Prefer existing `udp_return_delivery_ack_sent` / `udp_return_packet_ack_sent` evidence and a successful server terminal result if sufficient; add only the smallest typed seam evidence if ordering cannot otherwise be made deterministic.
5. Preserve the ordinary timeout positives: R9-3 controlled data-drop/PTO and R9-4 one-shot ACK-delay/reorder must still converge. Do not turn timeout continuation into generic terminal behavior.
6. This is a test/seam/oracle repair unless source inspection finds a new current defect. Do not redesign Session/Carrier/ACK/crypto/wire architecture.

## Evidence truth

Developer-local provenance reported in the reachable handoff for exact `a3a2761` records `PYTHONDONTWRITEBYTECODE=1 bash scripts/check.sh` exit 0, `git diff --check` exit 0, clean tree, Linux x86_64, rustc 1.98.0, 2026-09-19T02:51:19Z to 2026-09-19T02:56:13Z.

GitHub-hosted Rust CI run `35416808659` for exact `a3a2761` completed successfully on 2026-09-19: stable `bash scripts/check.sh` green and nightly pinned decode fuzz smoke green. Hosted CI is cross-evidence only; because the current process oracle is under-discriminating, its green result does not close this finding.

Reviewer-local execution is **not claimed**. The reviewer attempted a clean public clone, but the current sandbox could not resolve `github.com`; review here is exact-reachable GitHub source/test inspection plus hosted-CI cross-evidence.

## Exclusions

- no WAN/live claim;
- no new capacity/TTL/LRU/history-size/security numeric policy;
- no D019 change;
- no signing/key-custody/SBOM/publication decision;
- no protocol/wire/crypto architecture change;
- no RC/freeze/release/production authority change.

`READY_LIVE` remains `none` because this finding is deterministic local process/evidence work.