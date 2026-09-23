# Independent bounded review — H-I4-115 descendant-held output pipe

Date: 2026-09-24
Exact developer source/test anchor: `1f6d744c9954b9a7a5d693b43a20d137a5c5e75d`
Classification: test-harness repair/source review; no production protocol change

## Challenged invariant

The networked CLI process-test owner must not regain an unbounded wait after the directly-owned child has exited merely because a descendant still holds an inherited stdout/stderr writer. Direct-child termination and pipe EOF are distinct ownership facts.

## Exact-current owners inspected

- `crates/neko-cli/tests/probe.rs::bounded_wait_with_output`
- `crates/neko-cli/tests/multistream.rs::bounded_wait_with_output`
- the Unix negative regression `bounded_client_output_fails_when_descendant_holds_pipe_open`
- the surrounding child `try_wait` / kill / bounded-reap paths retained from H-I4-107/H-I4-109

## Result

No remaining caller-hang defect was found in this seam at `1f6d744...`.

Both helper copies now keep concurrent stdout/stderr draining while the direct child is live, but reader completion is delivered through bounded channels rather than an unconditional `JoinHandle::join()`. After direct-child exit/reap, stdout and stderr completion each have an explicit finite receive deadline; a descendant-held pipe therefore becomes an explicit harness failure instead of an unbounded reader join. The direct-child `try_wait` / kill / bounded-reap classification remains in place.

The Unix regression creates a promptly-exiting direct child with a finite descendant retaining the inherited pipe writer and verifies that the bounded client owner fails within a finite outer bound. This directly challenges the H-I4-115 premise. It is a test-harness/process-ownership regression only; it does not establish production process-tree teardown semantics.

The source repair therefore satisfies the H-I4-115 bounded-caller invariant. Final closure still requires the normal reachable developer-local exact-tree gate/provenance for `1f6d744...` (or a later unchanged-source anchor). No such provenance was present at review time.

## Evidence boundary

This review is exact-current source/control-flow inspection. The reviewer did not run the repository Rust/full gate, cross-platform runtime tests, fuzzing, WAN experiments or performance work in this pass. GitHub combined status for exact `1f6d744...` had no status entries when inspected. No decoder/parser/crypto-framing implementation changed, so fuzz is not implicated.

The synthetic descendant is finite but may outlive the assertion for part of its finite sleep interval; that does not reintroduce a caller hang and is not promoted here into production/process-tree evidence. If later test-runner hygiene shows an actual leaked-process interference, treat that as a separate narrowly evidenced harness issue rather than reopening protocol semantics.

## Exclusions

- No Session/Carrier/ACK/crypto/wire semantics reviewed or changed.
- No process-group/session policy is inferred.
- No repository-wide timeout, TTL, capacity or security value is introduced.
- No claim is made that direct-child teardown recursively terminates descendants.

## Next dependency-ready review lane

After exact-tree provenance is persisted, continue the existing process/socket/thread ownership sweep and then cross-platform/resource-boundedness reconciliation. In particular, distinguish test-harness outer ownership from authenticated application-level socket liveness; do not invent a post-auth idle-timeout value merely because a blocking read exists. If current specs do not decide such a timeout, classify it as a policy/operational boundary rather than manufacturing a repair.
