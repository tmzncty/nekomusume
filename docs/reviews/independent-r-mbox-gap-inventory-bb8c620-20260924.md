# R-MBOX-GAP — exact-current middlebox/reachability research gap inventory at `bb8c620`

**Repository anchor:** reachable `bb8c620e49b9b121bab129296f3bd577009784b4`.

**Result:** the newly merged middlebox/reachability plan creates real dependency-ready **local research/test-support work**. Repository-wide queue exhaustion is therefore **not** valid at this anchor even though the thirteen release-item-4 core surfaces have current bounded reviews. No correctness/security defect is asserted by this note.

## Exact-current evidence inspected

- `docs/research/middlebox-reachability-and-network-behavior.md` and the research boundary added to `docs/carrier-architecture.md` by merged PR #4 / `f7bf1f0...`;
- exact-current `crates/neko-cli/tests/probe.rs`, including existing warm/cold failover, process/socket boundedness, diagnostic ordering and Linux resource-snapshot seams;
- current repository path/code search for `netem`, repeated-failover fixtures and monotonic resource/queue fixtures;
- current standing VPS authorization and release/live classification.

Current repository search finds the new research plan but no reusable `scripts/bench/netem.sh`/netem harness, no generic repeated-failover integration fixture, and no resource fixture that executes the new repeated-transition research matrix. Existing `probe.rs` already has valuable local seams — loopback UDP/TCP peers, failover diagnostics, warm/cold transition assertions, bounded product-child ownership and Linux fd/RSS snapshots — so the correct next work is to **reuse those owners**, not invent a parallel checker/framework.

## Research invariants now ready for bounded local work

The merged plan explicitly names executable slices. They can be decomposed without changing core Session/Carrier/ACK/crypto/wire semantics or selecting policy values:

1. **R-MBOX-ROOTLESS-LOSS** — add/reuse a rootless local user-space impairment seam that can deterministically model UDP reply cessation / one-way silence and hard UDP loss around the existing failover fixture. The test must preserve current Carrier-owned liveness/failure and Session-owned protocol semantics; it must not require host route/firewall/qdisc mutation.
2. **R-MBOX-REORDER-DELAY** — extend the same seam to deterministic bounded reorder + delay. Challenge ordering/health/failover behavior without inventing production timer values or performance conclusions.
3. **R-MBOX-MTU** — model an MTU/oversized-datagram drop boundary in the local fixture. This is a path-behavior test, not permission to change PLPMTUD policy, wire framing or system interface MTU.
4. **R-MBOX-CLEAN-RECOVERY** — controlled loss followed by a clean path; prove the current carrier can recover/validate according to existing semantics and that Session delivery is not duplicated.
5. **R-MBOX-REPEATED-FAILOVER** — bounded repeated failover/recovery transitions using existing `CarrierState`/manager semantics; assert one active carrier, monotonically valid generation ownership, no duplicate delivery, and diagnostic transition order. No new hysteresis/readiness thresholds.
6. **R-MBOX-RESOURCE** — bounded repeated-transition resource fixture reusing existing resource observability where possible. Check ownership convergence / absence of monotonic leaked endpoints/session entries/in-flight recovery work; do not convert this into a capacity-pressure benchmark or invent RSS/FD/security limits.
7. **R-MBOX-TRANSITION-MATRIX** — reconcile the injected impairment matrix against current Carrier/Session state/event expectations and document exact tested/unresolved boundaries. This is evidence/spec support, not a new architecture ADR.

These are coherent dependency-ordered slices: build one narrow rootless seam first; later slices must extend/reuse it rather than spawning parallel test frameworks.

## Privilege / live boundary

The research document discusses Linux namespaces/veth/qdisc/netem as a preferred deterministic lab when available, but current authorization does **not** make host networking mutation a prerequisite for this queue. Start with the rootless user-space/local-socket path above.

Do not automatically run privileged host `tc`/qdisc, route, firewall, namespace or production-network mutations merely because the research plan names netem. A later isolated-netns backend may be implemented/reviewed only if it remains self-contained and its execution authorization/environment is clear; lack of that backend does not block the rootless slices.

These slices do not create a real-WAN question. **`READY_LIVE: none` remains authoritative.**

## Review -> repair / evidence discipline

Each source/test slice must challenge the current committed invariant first. If implementation is necessary, keep it test/support scoped unless a concrete product defect is independently demonstrated. After grouped source/test work, the final pushed developer source SHA still requires the normal clean exact-tree `scripts/check.sh` + `git diff --check` + clean-tree provenance. No wire/parser/crypto-framing change is currently implied, so no mechanical fuzz run is requested.

If any slice exposes a concrete defect in a reviewed core owner, immediately front that defect and refill the affected core-surface review. If it instead encounters a timer/capacity/security/core-semantic choice, classify the seam as maintainer/policy rather than selecting a value.

## Queue consequence

Keep the seven R-MBOX slices above ahead of any `queue exhausted` conclusion. After every 3–4 coherent slices, reconcile release/item-4 factual evidence, then continue the remaining research slices and conditional core-owner refill. H-I4-119 and all existing policy/external/release gates remain unchanged.
