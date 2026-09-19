# Independent bounded review — process-resource sampler handshake gap

**Reviewed source/test tree:** `5d860dce3959e90aa771fa0ca9387568efd52076`

**Reviewed handoff closure:** `df02bd0da7356fdb5708148d5d1e714b33e429cb`

**Classification:** HIGH evidence/review-truth finding; local test synchronization only. This is not a transport correctness finding, not a WAN result, and not release approval.

## Scope

Inspected exact-current:

- `scripts/bench/process-resource-sampler.py`
- `scripts/bench/process-resource-sampler-test.py`
- prior independent finding `docs/reviews/independent-process-resource-sampler-flake-6f4a61c-20260919.md`
- current `docs/CHATGPT_HANDOFF.md` closure claim

No reviewer-local test execution is claimed.

## Finding

The original finding required a **bounded readiness/release synchronization contract** and explicitly rejected solving the flake by merely extending the child sleep.

Exact `5d860dc` does add a `child.ready` marker after the five `/dev/null` FDs and listening socket exist, but the marker is never consumed by the sampler or by a concurrent parent before sampling. The test invokes the sampler synchronously and checks `ready.exists()` only **after the sampler has already returned**. Therefore the marker proves only that the child eventually reached the resource-owning state; it does not establish any happens-before relation between that state and a successful sampler observation.

The child then executes `time.sleep(1.4)` instead of the old `time.sleep(.25)`. During that fixed window the sampler normally gets enough polling opportunities, but an OS/runner scheduling stall can still leave the sampler observing only pre-exec/pre-resource state before the child exits. The failure mode is the same class as the original hosted flake, with a longer timing window.

The handoff closure claim that this is now deterministic is therefore not supported by the implementation. This also violates the recorded repair contract: “do not solve this by merely making `.25` a larger arbitrary sleep.”

## Required repair

Repair the synchronization contract rather than lengthening the sleep again.

A valid minimal shape must create a real bounded two-sided condition, for example:

1. the fixture child signals readiness only after all expected FDs/listener exist;
2. the child remains alive until a bounded parent/sampler-controlled release condition that is causally downstream of sampler observation (or an equally strong deterministic observation handshake);
3. the positive oracle must fail if that observation/release handshake never occurs;
4. continue sampling real `/proc` state — do not inject fake FD/socket metrics or bypass the sampler;
5. preserve the separate exit-race fixture's truthful `null` semantics;
6. preserve sampler-owned process-group termination/reaping and listener cleanup;
7. do not change production sampler semantics except for the smallest explicitly testable synchronization seam if no purely fixture-side shape can establish the required causality;
8. do not introduce capacity/performance policy values.

A mere fixed sleep increase, repeated polling without a sampler-observation acknowledgement, or checking the readiness marker only after process completion does not close this finding.

## Validation / provenance

For the final pushed source/test SHA, record the developer-local clean exact-tree gate:

- `PYTHONDONTWRITEBYTECODE=1 bash scripts/check.sh`
- `git diff --check`
- clean tree
- exact GitHub-resolvable SHA, UTC start/end, exit codes, Linux/arch and stable Rust version

No decoder/parser/crypto framing change is implied, so decoder fuzz is not mechanically required unless the eventual repair unexpectedly touches those surfaces.

## Queue effect

Reopen the sampler determinism lane at the front before R9-11. After this evidence-reliability HIGH is actually closed, continue immediately to R9-11 lifecycle/terminal resource closure; do not wait for reviewer cadence.
