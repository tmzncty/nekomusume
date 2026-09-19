# Independent bounded review — process-resource sampler fixture scheduling flake

**Reviewed tree:** `6f4a61ce81844d61018fc763908259d6b4c421cd`

**Classification:** local test/evidence reliability finding; not a transport correctness finding, not release approval, and not a WAN result.

## Scope

Inspected:

- `scripts/bench/process-resource-sampler.py`
- `scripts/bench/process-resource-sampler-test.py`
- GitHub-hosted Rust CI run `35447262812` for exact source/test SHA `1429df1b565060054e24cfbae4d62bfdda5890da`
- GitHub-hosted Rust CI run `35447630498` for exact docs-only descendant `6f4a61ce81844d61018fc763908259d6b4c421cd`

No reviewer-local test execution is claimed.

## Finding

The fixture named `known_child.py` is intended to prove that the sampler observes a process with five extra `/dev/null` FDs plus one listening socket. The child creates those resources and then keeps them alive for only `0.25` seconds. The sampler immediately begins by scanning `/proc` for process-group membership and then polls according to `--interval-ms 10`.

On exact `1429df1`, hosted stable `bash scripts/check.sh` failed after the Rust/package gates had passed because the fixture observed only `fd.peak_count = 3` and asserted `>= 9`. The unchanged sampler/test code then passed on the docs-only descendant `6f4a61c` in hosted run `35447630498`.

This pair is direct evidence that the current fixture's wall-clock lifetime is not a deterministic synchronization contract: under runner scheduling/load the sampler can observe the pre-exec/pre-resource state and then miss the short resource-owning interval. A 10 ms requested polling interval does not guarantee the sampler process is scheduled within 10 ms, and the test has no readiness barrier proving that the target FD/socket state existed before sampling was required to observe it.

The production sampler's truthful behavior when a process exits before a sample is already separately tested by the `exit-race` fixture. This finding is therefore about the positive `known_child` oracle, not about inventing zero metrics or changing sampler result semantics.

## Repair contract

Use a deterministic bounded synchronization seam rather than extending an arbitrary sleep and hoping for scheduler timing. Small acceptable shapes include:

1. have the child signal readiness after opening the expected FDs/listener and remain alive until a bounded parent-controlled release; or
2. otherwise arrange a bounded handshake proving the resource-owning state before the positive oracle is allowed to complete.

Preserve these invariants:

- the fixture still proves sampled FD count and owned experimental listener attribution from real `/proc` state;
- missing/exit-race metrics remain truthful `null`, never fake zero;
- child/process-group cleanup remains bounded and verified;
- no new capacity/performance policy number is introduced;
- no production sampler semantics are changed unless a separate concrete sampler defect is found.

Add a focused deterministic regression and run the normal exact-tree local gate. Hosted CI is useful cross-environment evidence but remains supplemental to the developer-local clean exact-tree gate.

## Queue effect

This does **not** reopen H-R9-075/H-R9-076 and does not preempt current R9-10B transport review. Keep it as a READY_LOCAL package/evidence-reliability slice after the current correctness lane, so a one-off hosted scheduling miss cannot continue to create false repository-gate failures.
