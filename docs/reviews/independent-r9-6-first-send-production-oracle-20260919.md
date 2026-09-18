# Independent R9 review — H-R9-062 first-send production-owner discriminator missing

Review anchor: exact reachable `main` tree `b88c803a76b35bb30f163f231745437df616d892`. Latest developer-owned source/test tree is exact `f8670a5a0ec95c4159e046fa12c4eec2177c63ff`; `b88c803` is the following handoff-only commit.

This is bounded release-item-4 review support. It is not release/security approval, protocol freeze, production authorization, WAN evidence, or a performance conclusion.

## Repository truth re-read

This pass re-read the current default-branch state and the required repository sources: `README.md`, `AGENTS.md`, `ROADMAP.md`, `IMPLEMENTATION_PLAN.md`, `SECURITY.md`, `docs/status.md`, `docs/standing-vps-lab-authorization.md`, `docs/vps-rental-window-priority.md`, `docs/decisions.md`, `docs/carrier-architecture.md`, `docs/specs/nekomusume-session-v0.md`, `docs/release-security-review-packet.md`, `docs/CHATGPT_HANDOFF.md`, plus the exact-current reliable-UDP first-send owners, `ReliableUdpRuntime`, `PathRecovery`, `neko-reliable::Recovery`, and current process-test surface.

New reachable commits reviewed since the previous reviewer handoff anchor `8c7f096189f5b918eb95d38d2c796af56a6c43c8`, by class:

- `f8670a5a0ec95c4159e046fa12c4eec2177c63ff` — implementation + unit-test repair for H-R9-061 across `crates/neko-carrier/src/lib.rs` and `crates/neko-cli/src/main.rs`.
- `b88c803a76b35bb30f163f231745437df616d892` — handoff documentation only.

The source repair itself materially improves the transaction boundary. Each of the three executable reliable-UDP first-send call sites now checks the real `UdpSocket::send_to` result; only `Ok` emits the corresponding positive sent diagnostic, while `Err` calls `ReliableUdpRuntime::abandon_sent`. `ReliableUdpRuntime::abandon_sent` removes the packet-to-frame map and routes through `PathRecovery::abandon_sent`, which removes Recovery state, Reno charge, charged bookkeeping, `packets_sent`, and restores the committed packet-number watermark through `Recovery::abandon_sent`. The deliberate `--drop-r9-data` seam remains distinct: it intentionally preserves Recovery ownership to model loss after transport ownership.

## H-R9-062 — HIGH evidence/oracle correctness: closure lacks a discriminator against the production first-send owner

The prior H-R9-061 review contract explicitly required a deterministic discriminator against the **production first-send owner**, with an injected send/socket failure after successful admission. The test had to prove that the production path emits no positive sent evidence on failure, rolls back packet-copy Recovery/Reno/packet-frame ownership, leaves the failed PN non-ACK-valid, and has a paired success control. The purpose was to ensure a later call-site regression — for example, forgetting `abandon_sent` or moving positive evidence before the socket outcome in one executable owner — cannot remain green merely because lower-level rollback primitives still work.

Exact `f8670a5` does not satisfy that acceptance oracle. Its only new regression is `path_recovery_tests::first_send_abandon_rolls_back_full_ownership`, which directly invokes `ReliableUdpRuntime::on_packet_sent`, manually calls `abandon_sent`, checks Recovery/Reno/ACK-validity state, then manually performs a fresh `on_packet_sent` success control. It never executes any of the three CLI first-send owners, never forces `UdpSocket::send_to` (or an equivalent production-owned send seam) to return `Err` after admission, and never observes the positive/failure diagnostic boundary.

The commit changed only `crates/neko-carrier/src/lib.rs` and `crates/neko-cli/src/main.rs`; no process-test owner or injectable production-send seam was added. Therefore a concrete future regression in the executable caller can pass the new unit test unchanged. This is an acceptance-oracle defect, not evidence that the new rollback primitive itself is currently wrong.

H-R9-061's source repair shape should therefore remain as the current implementation baseline, but its closure claim is reopened as H-R9-062 until a production-owner discriminator exists. R9-6 remains blocked on this evidence/correctness HIGH.

## Smallest dependency-ready repair contract

Current committed semantics are sufficient; no core Session/Carrier/ACK/crypto/wire redesign or new policy value is needed.

1. Preserve exact-wire congestion admission, secure-record sequence/nonce as the sole PN source, and the current `abandon_sent` rollback semantics.
2. Expose the **smallest production-owned send outcome seam** used by all three reliable-UDP first-send call sites (for example a tiny helper/trait/callback around socket outcome plus commit/abort evidence), so deterministic tests can inject `Err` after admission without relying on wall-clock/socket races. Do not create a second transport architecture.
3. Test the same production owner that executable code calls. A copied boolean expression or another test that manually calls `on_packet_sent` + `abandon_sent` is insufficient.
4. Inject failure after successful Recovery/Reno/plaintext admission and prove:
   - no positive `*_sent` diagnostic escapes;
   - the matching typed `*_send_failed` classification is emitted where that call site currently owns it;
   - packet-copy Recovery/Reno/charged/packet-frame ownership is gone;
   - the aborted PN is not ACK-valid historical sent state;
   - no RTT/PTO/loss/Session/Carrier-success transition is fabricated from the failed send.
5. Pair it with a success outcome using a fresh legal PN that preserves ownership, emits positive sent evidence, and can settle through existing ACK machinery.
6. Keep `--drop-r9-data` separate and Recovery-owned. Its loss semantics must not be reclassified as socket abort.
7. Audit all three executable reliable-UDP first-send call sites against the shared owner. A single shared production helper plus call-site checks is acceptable if it makes all three regressions discriminating; do not add duplicated test-only logic.
8. No new TTL/LRU/history-size/capacity/security values, no D019 change, no capacity benchmark, no release-authority change.

After the final pushed repair SHA, persist the ordinary exact-tree developer gate:

```text
PYTHONDONTWRITEBYTECODE=1 bash scripts/check.sh
git diff --check
```

Record exact reachable pushed SHA, UTC start/end, exit codes, OS/arch, stable Rust version and clean-tree state. Decoder/parser/crypto-framing fuzz is not mechanically required by this finding unless those surfaces actually change.

On closure, continue immediately into the remaining R9-6 ownership/resource-boundedness review. In that follow-up, specifically challenge the retained stable plaintext lifetime after a first-send abort: with no packet copy left in Recovery, it must remain only where existing retry semantics have a real owner and must be deterministically released on terminal teardown; do not invent a new retention policy to answer that question.

## Evidence truth

Developer-persisted exact-tree provenance for `f8670a5` reports `PYTHONDONTWRITEBYTECODE=1 bash scripts/check.sh` exit 0, `git diff --check` exit 0, clean tree, Linux x86_64, rustc 1.98.0.

GitHub-hosted Rust CI for exact `f8670a5`, run `35388578455`, also completed successfully: `stable checks` passed `bash scripts/check.sh`, and the nightly decode fuzz smoke passed. That is hosted cross-evidence only. Neither job injects a production first-send socket failure after Recovery admission, so the green run cannot close H-R9-062.

Reviewer-local execution is not claimed. This reviewer environment could not resolve `github.com` for a local clone/check. Exact pushed source/test inspection, repository-persisted developer provenance, and hosted CI remain distinct evidence classes.

## Queue / live effect

H-R9-062 is the front HIGH of R9-6. External coding work should implement the production-owner discriminator, gate it, and immediately continue R9-6B without waiting for reviewer cadence. R9-7 through R9-12, dedicated independent R9 review, Q10/Q11/Q12 factual reconciliation, and repository-wide item-4 refill remain dependency-ready behind it.

`READY_LIVE: none` remains authoritative. This is deterministic local correctness/evidence work and creates no new unresolved WAN question. Do not repeat HY2, warm failover, soak, package lifecycle, migration-back, endpoint/key migration, IPv6, PLPMTUD, or Experimental Track evidence merely because the VPS remains rented. D019, retained-state capacity policy, signing/key custody/SBOM/publication policy, core architecture and release/freeze/production authority remain untouched.
