# Reviewer closure — H-I4-120 ACK/PTO governance rollback

**Exact source/test anchor:** reachable `4511e4f147eb9511907bb4ec52cad687f113bb7a`.

**Reviewer repository-state anchor:** `cac1ee072d1d785352e4c72a8f506f6e3ff031ee` (later provenance-only commits do not move product/test owners).

**Result:** H-I4-120 is **CLOSED**. H-I4-119 remains a separate **MAINTAINER / CORE ACK-PTO SEMANTICS GATE** and is not resolved by this closure.

## What was independently checked

The reviewer re-read the exact-current Recovery owner/tests, the original H-I4-120 finding, the partial-closure rejection, current `docs/decisions.md`, the provisional Session spec, the H-I4-119 gate note, and the final developer-local provenance.

The closure chain now satisfies the bounded rollback contract:

1. `9e79e6b9e2c69be9a99617f7ab0b8415864de679` restores production `Recovery::on_ack` to the pre-H-I4-119 baseline: whenever a non-empty set of actually sent packets is newly acknowledged, the existing implementation resets `pto_count`. It removes the unauthorized ack-eliciting-only production guard. This is a governance rollback, not a maintainer decision that the baseline is the final desired policy.
2. `4511e4f147eb9511907bb4ec52cad687f113bb7a` makes the non-ack-eliciting ACK regression semantics-neutral. `non_ack_eliciting_ack_keeps_older_packet_outstanding` verifies legal retirement and loss-ordering facts while deliberately asserting no `pto_count` outcome.
3. The historical bad/unreachable provenance record is preserved and explicitly corrected rather than silently rewritten; the erratum identifies `9e79e6b...` as the production rollback commit.
4. Reachable developer-local exact-tree provenance for `4511e4f...` is persisted in `docs/notes/check-gate-4511e4f-20260924.md`: `PYTHONDONTWRITEBYTECODE=1 bash scripts/check.sh` exit 0, `git diff --check` exit 0, clean detached tree before/after, UTC start/end, Linux x86_64, and stable Rust `1.98.0`.
5. GitHub combined status for `4511e4f...` exposes no hosted status entries. This closure therefore treats the persisted gate as developer-local provenance only; it makes no hosted-CI claim.

A source-history comparison from the pre-gate review anchor `68d938279478b91661f786e830e99af83db457f1` through `4511e4f...` shows the surviving `crates/neko-reliable/src/lib.rs` delta is test coverage added around the disputed seam; the unauthorized production branch was rolled back.

## H-I4-119 remains excluded

Repository truth still does not normatively choose between:

- reset PTO backoff after any newly acknowledged sent packet, or
- reset only after newly acknowledged ack-eliciting packet(s).

No current decision/spec amendment resolves that core ACK/PTO semantic. The restored baseline is therefore the authorized implementation state pending maintainer/spec resolution, not a reviewer-selected final architecture. Coding/review automation must not silently reintroduce either policy as a new decision.

## Evidence boundary

This is an exact-current GitHub source/test/control-flow + provenance review. No reviewer-local Rust/full-gate execution, hosted CI, cross-platform run, fuzz, WAN experiment, performance conclusion, independent security approval, or release authority is claimed. No decoder/parser/crypto-framing owner changed, so no fuzz run is requested.

Release items 3 and 4 remain incomplete. `RELEASE_CANDIDATE=false`, `PRODUCTION_READY=false`, `FREEZE=false`, `RELEASED=false`; `READY_LIVE: none` remains unchanged.
