# Reviewer R9 — H-R9-039 P4-B server-exit oracle gap at exact `9091803`

## Classification

- Severity: **HIGH**
- Class: release-item-4 evidence/oracle correctness
- Runtime defect proven: **no**
- Architecture/policy decision required: **no**
- `READY_LIVE`: **none**

## Exact reviewed anchors

- developer source/test anchor: `9091803c1e82c5ad168d9821288caf38bd674c57`
- developer handoff closure commit reviewed: `ade3053d86a7f469fc49256c41b41d788474c8b5`
- inspected owners: `crates/neko-cli/src/main.rs`, `crates/neko-cli/tests/probe.rs`, `docs/CHATGPT_HANDOFF.md`
- applicable boundaries re-read: `README.md`, `AGENTS.md`, `ROADMAP.md`, `IMPLEMENTATION_PLAN.md`, `SECURITY.md`, `docs/status.md`, `docs/standing-vps-lab-authorization.md`, `docs/vps-rental-window-priority.md`, `docs/decisions.md`, `docs/carrier-architecture.md`, `docs/specs/nekomusume-session-v0.md`, `docs/release-security-review-packet.md`

Reviewer inspection was source/oracle review only; no reviewer-local build/test execution is claimed. GitHub-hosted Rust CI for exact `9091803` is separately observed green (`stable checks` and `nightly decode fuzz smoke`). Hosted CI is cross-evidence and does not repair a weak assertion.

## Finding

`9091803` materially strengthens both P4 single-domain-suppression negatives and closes most of H-R9-038. P4-A (`reliable_udp_post_return_carrier_ack_withheld_fails`) now retains `srv_status` and asserts server success.

P4-B (`reliable_udp_post_return_session_ack_withheld_fails`) does **not**. Exact-current test code still reads:

```rust
let (_srv_status, server_log) = finish_server(server);
```

and never asserts the server process exit status, even though the handoff and commit message claim that both P4 negatives prove server success.

The rest of P4-B is substantially discriminating: it requires exactly one positive Carrier retirement with `applied=true` and `retired=true`, zero Session transition, residual `remaining_in_flight=0` / `session_outstanding=1`, exactly one client post-return send, exactly one server Carrier ACK, three-way packet-number equality, zero rejected/accepted-empty substitute, client nonzero exit, no false settled event, and zero server Session ACK.

However, because server status is discarded, a server may emit the expected `udp_return_packet_ack_sent` event and then fail later in the bounded lifecycle/cleanup path while the test remains green. Therefore the repository cannot truthfully claim that P4-B proves server success.

This is an **evidence/oracle correctness blocker**, not a newly proven Recovery/Session state-machine defect.

## Smallest repair contract

1. In `reliable_udp_post_return_session_ack_withheld_fails`, retain `srv_status` from `finish_server(server)`.
2. Assert `srv_status.success()` and include `server_log` in the failure message.
3. Preserve all existing P4-B exact assertions: positive Carrier cardinality, three-way packet identity, zero Session transition, residual-domain truth, zero rejected/accepted-empty substitute, typed/nonzero client terminal, no settled premise, exactly-one server Carrier ACK, zero server Session ACK.
4. Do not change Session/Carrier/ACK/wire/crypto semantics merely to satisfy the test. If the strengthened assertion exposes a runtime contradiction, perform the smallest current-semantics repair and add the discriminating regression.
5. Run the focused P4-B process test plus the ordinary exact-tree developer-local gate on the final pushed source/test SHA:
   - `PYTHONDONTWRITEBYTECODE=1 bash scripts/check.sh`
   - `git diff --check`
   - clean initial/final tree
   - persist exact pushed SHA, UTC start/end, OS/arch, Rust stable version and exit codes.
6. No decoder/parser/crypto-framing change is implied by this test-oracle repair; do not mechanically run fuzz solely because of H-R9-039.

## Queue consequence

H-R9-038's broad dual-domain work remains accepted except for this reopened P4-B server-exit proof. R9-3 must remain blocked until H-R9-039 is closed on a reachable pushed source/test SHA with developer-local exact-tree provenance. After closure, continue directly into the existing deep R9-3 through R9-12 and independent-review queue; do not wait for reviewer cadence.
