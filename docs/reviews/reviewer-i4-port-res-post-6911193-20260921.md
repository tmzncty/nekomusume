# Reviewer bounded I4-PORT-RES re-challenge — exact `6911193`

**Exact reviewed tree:** `387345530edd5e2ca74bf9dc8abfffa3768398d4`.

**Executable source/test owner:** `69111937075a158a87fa96a4d1a0f9d056f40d70` (`test(cli): H-I4-090 churn_started flips before first malformed send`).

## Owners inspected

- `crates/neko-cli/tests/probe.rs` — `process_resource_snapshot`, `ready_failover_server`, and `udp_listener_rejects_bounded_malformed_churn_then_authenticates_and_cleans_up`;
- exact repair diff `69111937075a158a87fa96a4d1a0f9d056f40d70`;
- `docs/notes/h-i4-090-provenance-6911193-20260921.md`;
- `README.md`, `AGENTS.md`, `ROADMAP.md`, `IMPLEMENTATION_PLAN.md`, `SECURITY.md`, `docs/status.md`, `docs/standing-vps-lab-authorization.md`, `docs/vps-rental-window-priority.md`, `docs/decisions.md`, `docs/carrier-architecture.md`, `docs/specs/nekomusume-session-v0.md`, and `docs/release-security-review-packet.md` for current evidence/governance boundaries.

## Invariant challenged

For the bounded malformed-UDP resource-growth evidence on Linux:

1. the failover server must reach the test's accepted readiness point before sampling;
2. the affirmative Linux baseline must be captured before the first malformed datagram can be sent;
3. a mutation that moves the baseline after churn begins must fail deterministically even if FD/RSS happen not to change;
4. the post sample must remain after the bounded churn and settle point;
5. either unavailable `/proc/<pid>/fd` or `/proc/<pid>/status` observation must fail closed rather than silently skip the resource claim;
6. the non-Linux Unix path must retain portable socket/lifecycle coverage without pretending to have Linux `/proc` evidence;
7. no new capacity/security numeric policy may be inferred from this test repair.

## Exact-current challenge result

The current test captures `pid`, consumes `ready_failover_server(server)`, initializes `churn_started = false`, and on Linux captures `before` only inside a block that asserts `!churn_started`. Sender sockets and their unique source-port check are prepared while the sentinel remains false; this preparation is not malformed traffic. Immediately before the first possible malformed `send_to`, the test sets `churn_started = true`. Therefore moving the baseline block to any point after churn starts — including after the send loop — deterministically fails the pre-churn assertion independently of the measured resource values.

After the eight bounded sends, the test requires `churn_started`, waits the existing 100 ms settle interval, and then captures `after`. On Linux it first requires both snapshots to be `Some`; only then does it apply the already-existing FD and RSS margins. `process_resource_snapshot` returns `None` on an unreadable FD directory, unreadable status file, missing `VmRSS`, or parse failure, so incomplete Linux observation cannot become a resource pass. On non-Linux Unix the `/proc` snapshots are compiled out while the authenticated client/server and listener rebind cleanup assertions remain in the test; the unconditional post-send `assert!(churn_started)` also keeps the test-local sentinel used outside the Linux cfg.

No decoder/parser/crypto-framing owner changed in `6911193`; no fuzz run is required by this slice. No runtime Session/Carrier/ACK semantics, D019 decision, release flag, or resource-policy number changed.

## Evidence classification

The reachable developer provenance for exact pushed source/test SHA `6911193` records `PYTHONDONTWRITEBYTECODE=1 bash scripts/check.sh` exit 0, `git diff --check` exit 0, clean worktree, Linux x86_64, stable rustc 1.98.0, UTC 2026-09-21T15:49:55Z -> 15:55:50Z. This note does **not** claim reviewer-local Rust execution, reviewer-local full gate, WAN/performance execution, or hosted CI. A current GitHub status/workflow query for `6911193` returned no hosted statuses/runs; absence is neither success nor failure.

The older `docs/reviews/reviewer-i4-port-res-post-0dbb931-20260921.md` remains useful for its source-ordering analysis, but its statement that a mutation-sensitive guard was merely optional hardening is superseded by the later H-I4-090 closure contract and the actual `6911193` sentinel repair.

## Result

**No finding in the exact-current I4-PORT-RES surface. H-I4-090 is closed at `6911193`.** The changed owner is test/evidence code only; proceed to evidence reconciliation, release-packet factual-consistency review, and the repository-wide 13-surface refill before re-establishing queue exhaustion. `READY_LIVE: none`; release items 3/4 and all release/governance flags remain unchanged.
