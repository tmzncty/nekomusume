# Independent bounded review — process-resource sampler observation handshake

Date: 2026-09-20

## Anchor and scope

Reviewed exact reachable source/test commit:

`e021b27cffe4a87de185757a8d47b8685f462bb6`

Commit: `fix(bench): sampler positive-fixture real observation handshake`

Inspected owners:

- `scripts/bench/process-resource-sampler.py`
- `scripts/bench/process-resource-sampler-test.py`
- the superseded independent finding `docs/reviews/independent-process-resource-sampler-handshake-gap-5d860dc-20260920.md`
- current release/evidence boundaries in `AGENTS.md`, `IMPLEMENTATION_PLAN.md`, `SECURITY.md`, `docs/status.md`, `docs/standing-vps-lab-authorization.md`, `docs/vps-rental-window-priority.md`, `docs/decisions.md`, `docs/carrier-architecture.md`, `docs/specs/nekomusume-session-v0.md`, `docs/release-security-review-packet.md`, and `docs/CHATGPT_HANDOFF.md`.

This is an independent bounded review of the deterministic positive-fixture synchronization seam only. It is not a transport/runtime correctness review, WAN evidence, capacity result, security audit, or release claim.

## Challenged invariant

The positive fixture must not depend on an arbitrary child sleep window. The child must retain ownership of the known file descriptors and loopback listener until a condition causally downstream of the sampler's actual `/proc` observation releases it. If that observation/release handshake does not happen, the positive oracle must fail rather than silently pass on timing luck.

## Result — no code finding on the repaired seam

At exact `e021b27` the prior scheduling gap is repaired in the intended bounded way:

1. the child opens five `/dev/null` descriptors and the loopback listener before writing `child.ready`;
2. the child no longer exits after a fixed `.25s`/`1.4s` sleep; it waits, with a bounded deadline, for `sampler.release`;
3. the sampler emits that release marker only from its sampling loop after observing a non-`None` FD count at or above the requested minimum and at least one owned socket;
4. the test requires both readiness and release markers, the expected child exit code, and the sampled FD/socket peaks;
5. if the sampler never reaches the observation threshold, no release marker is written; the child remains live until the bounded sampler/child deadlines and the positive fixture cannot satisfy the asserted oracle;
6. real `/proc` observation remains the source of the FD/socket values; no synthetic metric bypass was introduced.

The release write is intentionally best-effort at the sampler implementation point, but an `OSError` does not create a false positive: the marker remains absent, the child is not observation-released, and the positive test's marker/exit assertions fail. This preserves the fail-closed property of the fixture.

No decoder/parser/crypto framing surface changed, so the decoder fuzz requirement is not triggered by this repair.

## Evidence boundary / remaining closure condition

No reviewer-local command execution is claimed in this review.

GitHub-hosted Rust CI for exact `e021b27` is supplemental evidence only: push run `35459239845` completed successfully on 2026-09-19 UTC.

The repository contract requires a developer-local clean exact-tree gate on the final pushed source/test SHA as first-class validation. I did not find a persistent repository record for exact `e021b27` containing the required local-gate provenance (`PYTHONDONTWRITEBYTECODE=1 bash scripts/check.sh`, `git diff --check`, clean-tree state, exact SHA, UTC start/end, exit codes, OS/arch, stable Rust version). Therefore the **code-level sampler synchronization finding is no-finding/accepted, but the evidence/review-truth HIGH must not be considered fully closed until that developer-local provenance is recorded against exact `e021b27` (or a later final pushed source/test SHA containing the same repair).** Hosted CI must not substitute for it.

## Exclusions

This review does not claim:

- R9-11 lifecycle/resource closure;
- release item 3 or item 4 completion;
- WAN/live evidence or performance conclusions;
- capacity-pressure or policy-number suitability;
- release candidate, production readiness, freeze, or release authority.

After the exact-tree local provenance is persisted, the queue may advance immediately to R9-11 without waiting for the next reviewer cadence.
