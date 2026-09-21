# H-I4-091 — malformed-UDP resource post-sample lacks a processing barrier

**Severity:** HIGH (release/item-4 evidence-oracle correctness; not yet a demonstrated runtime leak)

**Exact reviewed repository anchor:** `aa3faf84dd92d583064ddaaae6047998a16d09fb`.

**Current executable source/test owner:** `69111937075a158a87fa96a4d1a0f9d056f40d70` in `crates/neko-cli/tests/probe.rs`.

## Finding

H-I4-090 correctly restored the **pre-churn** Linux resource baseline and added a mutation-sensitive `churn_started` sentinel before the first malformed send. The remaining `after` sample, however, is not causally ordered after the failover server has actually **received/classified all eight malformed datagrams**.

The test currently does:

1. wait for the failover server diagnostic `start` event;
2. capture the Linux `before` snapshot;
3. create eight distinct local UDP senders;
4. set `churn_started = true`;
5. issue eight `send_to` calls;
6. sleep for 100 ms;
7. capture `after` and compare FD/RSS.

The `send_to` completions only prove local kernel enqueue/send completion. The fixed 100 ms sleep is a scheduling heuristic; it does not prove that the server process has run and drained/classified all eight datagrams before the `after` snapshot.

The current failover server makes this observable race concrete. In the pre-authenticated path it handles at most one `udp.recv_from` datagram per outer loop iteration, attempts version negotiation, and on an invalid hello releases the admission and loops again. There is no per-malformed diagnostic/counter/barrier consumed by this test. A sufficiently delayed/descheduled server can therefore leave some or all malformed datagrams queued when the test takes `after`; the later authenticated client can wake/progress the server and the overall test can still finish successfully. Any resource growth caused specifically while classifying those delayed malformed inputs would then occur **after** the asserted post-churn sample and escape the oracle.

This falsifies the exact-current review note's assumption that `send loop -> sleep(100 ms) -> after` is a deterministic post-churn evidence boundary. It is an evidence-oracle race, not proof that the runtime currently leaks resources.

## Closure contract

Repair the smallest truthful seam; do not add a general checker/framework and do not invent new capacity/security numeric policy.

Required properties:

1. Keep the H-I4-090 pre-churn baseline and `churn_started` mutation guard unchanged in meaning.
2. Establish a **bounded deterministic server-side processing barrier** proving that all `ATTEMPTS` malformed datagrams used by this test have been received and rejected/classified before the Linux `after` snapshot is taken.
   - A diagnostic/test-only monotonically counted rejection event consumed by the test is acceptable.
   - An equivalent explicit server acknowledgement/barrier is acceptable only if it does not promote malformed traffic into transport/delivery evidence or change production protocol semantics.
   - A fixed sleep, scheduler assumption, socket-send completion, or later client success is not sufficient.
3. The barrier must itself be bounded/fail-closed: if all expected malformed inputs are not observed before its deadline, the test fails rather than taking the post sample anyway.
4. Preserve Linux-only `/proc/<pid>/fd` + `/proc/<pid>/status` evidence and fail closed if either pre/post snapshot is unavailable; preserve the existing FD/RSS margins without changing their numbers.
5. Preserve the non-Linux Unix portable socket/lifecycle path without pretending to have `/proc` resource evidence.
6. Add a focused regression/oracle that would fail if the post-resource snapshot is moved before the server-side malformed-processing barrier.
7. No decoder/parser/crypto-framing semantics need change merely for this evidence repair; do not run fuzz unless such an owner is actually modified.
8. On the final pushed source/test SHA, run and persist developer-local clean exact-tree provenance: `PYTHONDONTWRITEBYTECODE=1 bash scripts/check.sh`, `git diff --check`, clean tree, exact SHA, UTC start/end, OS/arch and stable Rust version.

## Follow-up review

After repair, independently re-challenge the exact current `I4-PORT-RES` owner, including both causal edges:

- `READY -> affirmative pre-churn snapshot -> first malformed send`;
- `all bounded malformed inputs processed/classified by server -> affirmative post-churn snapshot`.

Then reconcile the item-4/release-packet evidence boundary and perform the repository-wide 13-surface refill before any new queue-exhaustion declaration.

## Evidence classification

This finding is reviewer source/test/runtime-order reasoning only. No reviewer-local Rust test, `scripts/check.sh`, fuzz, WAN experiment, performance run, or hosted-CI success is claimed. The reachable developer-local clean provenance for exact `6911193` remains valid as a record that that tree passed its own gate; it does not make a racy evidence oracle causally complete.

`READY_LIVE: none` remains unchanged. Release items 3/4 remain incomplete; `RELEASE_CANDIDATE=false`, `PRODUCTION_READY=false`, `FREEZE=false`, and `RELEASED=false` remain unchanged.
