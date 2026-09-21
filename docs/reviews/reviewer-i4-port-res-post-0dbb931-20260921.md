# Reviewer bounded I4-PORT-RES challenge — post-H-I4-090

**Exact reviewed tree:** `e71971b8057271e5fa6f739f431c554892861da7` (source/test owner remains `0dbb93145c4e1cc031ca30258b3c58a5c3155b51`; `e71971b` is the later handoff-only commit).

**Owners inspected:**

- `crates/neko-cli/tests/probe.rs` — `process_resource_snapshot` and `udp_listener_rejects_bounded_malformed_churn_then_authenticates_and_cleans_up`;
- exact repair diff `0dbb93145c4e1cc031ca30258b3c58a5c3155b51`;
- `docs/notes/h-i4-090-provenance-0dbb931-20260921.md`;
- `.github/workflows/ci.yml` and `scripts/check.sh` only to classify evidence/platform scope, not as reviewer-run execution.

## Invariant challenged

For the malformed-UDP resource-growth subclaim on Linux:

1. server readiness must precede the baseline observation;
2. the affirmative baseline must precede the first malformed datagram;
3. the post observation must follow the bounded malformed-input window and settle point;
4. either unavailable Linux observation must fail the resource claim rather than become a skip/pass;
5. the existing FD/RSS margins must compare the true pre/post pair rather than two post-churn samples.

## Exact-current reasoning

The current test captures `pid`, consumes server readiness through `ready_failover_server(server)`, and immediately evaluates `let before = process_resource_snapshot(pid)` under `#[cfg(target_os = "linux")]`. Only after that statement does it construct/send the eight malformed UDP datagrams. After the existing 100 ms settle point it evaluates `after = process_resource_snapshot(pid)`. The Linux assertion first requires `(before, after)` to be `(Some(_), Some(_))`; only then are the already committed FD/RSS margins applied. Thus the H-I4-090 false-negative ordering (`before` and `after` both after churn) is absent from the exact-current owner.

The helper itself returns `None` on failed `/proc/<pid>/fd`, `/proc/<pid>/status`, or VmRSS parsing and is only consumed by the resource assertion under the Linux cfg. No new numeric resource/capacity/security policy is introduced by the repair.

## Evidence classification

No reviewer-local Rust test, `scripts/check.sh`, fuzz, hosted CI, WAN, macOS/non-Linux execution, or performance run is claimed here. The reachable developer provenance for exact source/test SHA `0dbb931` records focused affected testing plus clean exact-tree `PYTHONDONTWRITEBYTECODE=1 bash scripts/check.sh`, `git diff --check`, and clean worktree on Linux x86_64 / stable Rust. A current GitHub combined-status query for `0dbb931` returned no hosted statuses; absence is neither success nor failure.

The earlier H-I4-090 closure contract mentioned a mutation-sensitive guard against moving the baseline again. Exact-current comments make the intended ordering explicit, but no separate meta-test/checker was added. That is future-regression hardening, not a present counterexample: the executable test currently samples in the required order and fails closed on unavailable Linux observations. Do not reopen HIGH or manufacture a generic checker solely to satisfy that hardening suggestion.

## Result

**No finding in I4-PORT-RES at exact current owner.** H-I4-090 remains closed. Advance to the bounded Linux `/proc` owner/callsite inventory and subsequent platform-boundary/process-cleanup lanes. `READY_LIVE: none`; no release/governance flag changes are implied.