# Reachable-tree bounded item-4 review — exact `4034f86`

**Anchor:** reachable pushed source/test commit `4034f86ceb356a3e04b8ff713b97338af3e71265`.
**Review type:** developer-performed bounded source/test challenge rebuilding quarantined local review surfaces. This is not independent review, cryptanalysis, a security audit, release approval, or production authorization.

## Results

- **Process-test determinism:** the unsupported-negotiation assertion accepts only orderly EOF or `ConnectionReset` and hard-fails bytes/other errors. Focused test passed 20/20, full multistream target passed, clean exact-tree full gate and exact-head hosted CI are green. A bounded sweep of `crates/neko-cli/tests/*.rs` found no second demonstrated supported-platform contract contradiction; Unix permission/symlink tests are explicitly Unix-gated.
- **Cryptographic API misuse:** source/tests cover nonce exhaustion and checked overflow, direction/epoch/key-phase and transcript/record-context binding, replay/duplicate/old-phase rejection, authorization before protected admission, synchronized/unsynchronized key update, and secret-safe public errors. `cargo test -p neko-crypto --all-targets`: 37 passed. No concrete API misuse found. No cryptanalysis, persistent replay design, or D019 policy claim.
- **Wire/parser:** independent review is retained separately in [`independent-wire-review-4034f86-20260912.md`](independent-wire-review-4034f86-20260912.md). It found no decoder defect; exact-tree `neko-wire` tests passed and bounded pinned fuzz produced no crash/OOM. Fuzz is saturation evidence, not proof.
- **CLI secret/output/pre-auth admission:** bounded source/test review found no private identity key/plaintext payload in stderr or structured events, no success/READY before established authentication/admission, and no unauthenticated durable/session transition on inventoried responder paths. `cargo test -p neko-cli` passed; responder inventory remains 9 surfaces / 7 admission sites. Identity creation persists key material by explicit keygen/identity-file behavior and is not logged.
- **Session/Carrier evidence domains:** `cargo test -p neko-session --all-targets`, `cargo test -p neko-carrier --all-targets`, and `cargo test -p neko-reliable --all-targets` passed. Source/tests preserve assignment-time context across `Unsent -> InFlight`, reject advanced-state novel-byte overlap, keep packet/path feedback separate from logical `confirm_received` / `DeliveryAck`, and do not let TCP reliability manufacture application effect evidence. No current contradiction found.

## Exact reproducible evidence

At exact `4034f86`, the clean detached full gate ran UTC `2026-09-12T11:06:08Z -> 11:08:06Z` on Linux x86_64, Rust 1.98.0, with initial/final tree clean; `PYTHONDONTWRITEBYTECODE=1 bash scripts/check.sh` and `git diff --check` exited 0. GitHub Actions run `34690143352` passed. Additional focused reproduction on the same reachable source blob family: crypto 37 tests, wire 26 tests, Session 41 tests, Carrier 47+2+1+2 tests, CLI 40+6+39 tests, reliable 27+3+3 tests, and responder inventory 9/7, all green.

## Evidence boundary

The historical local-note SHAs `1db7a97`, `7275062`, `c5d0b15`, `58b5d13`, `7711162`, `1be290d`, and `6f50d70` are pre-rebase/unreachable and remain quarantined rather than being reinterpreted as this exact tree. Their timestamps and commands are not rewritten. This note supersedes them for current developer-review navigation using one reachable anchor.

No code defect was found in this bounded rebuild. Release item 4 remains incomplete; D019, representative adversarial-load/capacity suitability, independent maintainer/security judgment, item 3, signing/key custody/SBOM, RC/release/production authority, and current live/environment/implementation blockers remain open.
