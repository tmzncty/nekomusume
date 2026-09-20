# Independent final R9 bounded review — exact `8cbd9af`

**Source/test anchor:** reachable exact `8cbd9af39a600f4ddd5fbc50208791e8584c6d74`.
**Repository head observed before this note:** `bc6cd1dd6b69cf219a75624f62db97bcde999dc3` (docs-only commits after the source/test anchor).

This is the dedicated final independent R9 bounded review requested by the rolling handoff. It is release-item-4 review support only. It does not mark release item 4 complete, does not promote local evidence into WAN/security/release evidence, and does not change D019 or any policy/value/release-authority decision.

## New reachable work reviewed

The reachable delta after the previous H-R9-084 finding was re-read from repository truth. `8cbd9af` is the only new executable source/test repair in the reviewed R9 closure chain: it makes the process-resource sampler wrapper return nonzero whenever terminal cleanup is not affirmatively complete while preserving the measured child exit fact in structured output, and strengthens the escaped/partial-observation regressions. The subsequent R9-11D1/D2/D3/D4 notes, R9-11A–D factual reconciliation, R9-12 exact-tree provenance, and handoff refreshes are documentation/review/provenance changes; they do not alter the executable source/test tree.

## Invariants independently re-challenged

### 1. Recovery / sender packet-ACK truth

`neko-reliable::Recovery::on_ack` rejects a nonempty ACK when no packet has ever been accepted and rejects `largest > largest_sent` before RTT, loss, PTO, sent-map, or frame-ownership mutation. The current focused regression mixes a real in-flight ACK with a never-sent future packet, proves rejection leaves in-flight/RTT state unchanged, then proves a subsequent valid ACK still operates on the untouched state. ACK range representation remains bounded/canonical rather than iterating a peer-controlled numeric span. Overlapping frame copies retain positive-ACK suppression until the last copy retires, and quiesce clears live packet/frame/high-water ownership while preserving documented lifetime recovery history.

No concrete defect found.

### 2. Receiver ACK ownership and evidence separation

The executable reliable-UDP receive paths call `on_packet_received` only after the record has passed the authenticated/canonical Data path; malformed/non-Data input does not enter the outgoing packet-ACK range. `poll_outgoing_ack` is consumptive and the post-return stale/future seams retain/reseal canonical ACK plaintext rather than manufacturing Session delivery evidence. The reliable-UDP teardown review remains consistent with the exact-current owners: receiver ACK tracker/live sender ownership are reset or terminal-inert while lifetime diagnostics survive.

Carrier packet ACK remains distinct from `SessionRuntime::delivery_ack` / the Session ledger's `confirm_received`; the Session-v0 contract still states that logical confirmation proves peer acceptance for transport delivery, not application side effect.

No concrete defect found.

### 3. Session logical proof / runtime ownership

The exact-current Session ledger rejects old/regressing context, invalid delivery-epoch migration, uncovered-gap overlap, and advanced-state extension that would manufacture delivery state. `confirm_received` validates the delivery transition and monotonic evidence context before committing the ledger-wide/segment context and watermark. Runtime send/receive flow-control accounting is checked before mutation; terminal paths share runtime-state cleanup, while cumulative/lifetime facts remain historical rather than being reset to a false fresh state. The mutation-sensitive H-R9-080 regressions continue to populate send/receive/dedup/window/deadline ownership before cancel/remote-close and challenge post-terminal evidence growth.

No concrete defect found.

### 4. Carrier readiness / promotion / generation terminality

`CarrierState` keeps packet feedback separate from validation, requires explicit validation plus hysteresis before activation, rejects stale/mismatched generations, enforces single-active ownership, and does not equate PTO with failure. The D064 warm path in `CarrierManager` binds readiness to target path, generation, Session identity, delivery epoch, distinct bounded observation IDs, authentication, resume validation, and resource admission. Invalid dimensions or unvalidated observations clear accumulated readiness and fail closed. UDP failure creates a pending next-generation switch; warm promotion requires the matching already-warm candidate, while the legacy post-failure readiness path is explicitly disabled.

No concrete defect found.

### 5. Uncertain retention, TCP replay/dedup, migration-back

`FailoverController::tcp_resend` derives replay from retained exact `(DataId, bytes)` ownership; `confirm` is the explicit retention-ending operation. Exact duplicate receive returns the dedup result without re-delivery, while conflicting bytes for the same identity fail closed. The concurrent-manager owner previously challenged in R9-11C keeps stable logical range identity/bytes across hard failure/drain, marks unproved old-generation ownership uncertain, and reassigns replay only to a new sole active generation before confirmation can end retention.

Migration-back requires matching generation, explicit validation, healthy candidate state, score margin and hold before the active path changes. Rejected validation/generation/health/score candidates do not acquire active ownership. The existing hold-gate mutation is the committed hysteresis mechanism exercised by current tests; this review does not invent or change its policy values.

No concrete defect found.

### 6. Process result truth / terminal cleanup

At exact `8cbd9af`, the process-resource sampler's terminal cleanup result requires both an empty sampler-created process group and an affirmative four-table TCP/TCP6/UDP/UDP6 owned-port absence observation. Present or incomplete/unknown socket observation cannot become `owned_sockets_after_exit=0` or `cleanup.complete=true`. Escaped setsid TCP/UDP fixtures establish bind readiness before the original-group helper exits, assert sampler nonzero while ownership remains escaped, and use bounded cleanup of the helper. The partial `/proc/net` fixture asserts unknown cleanup and nonzero wrapper status.

The structured `result["exit"]` continues to report the measured child exit fact independently. The wrapper returns nonzero on unproven cleanup, so command success can no longer contradict its own cleanup object/validator contract. Normal cleanup-complete child success, child nonzero, timeout/interruption, process-group reap and post-return false-success boundaries remain covered by the reachable D1–D4 reviews/tests.

No concrete defect found.

## Cross-check against current R9-11 closure

The reachable R9-11A–D reconciliation is factually compatible with the exact source/test tree reviewed here: H-R9-081 remains repaired; H-R9-082/H-R9-083 source/test semantics remain accepted; H-R9-084 is repaired at `8cbd9af`; R9-11A/B/C/D1/D2/D3/D4 have bounded reachable closure notes. This final review found no contradiction requiring any of those lanes to reopen.

## Evidence boundary

No reviewer-local command/test execution is claimed. This review used reachable GitHub source/test/spec/review truth. Developer-reported final exact-tree provenance for `8cbd9af` is persisted separately in `docs/notes/r9-12-final-provenance-8cbd9af-20260920.md`: `PYTHONDONTWRITEBYTECODE=1 bash scripts/check.sh` exit 0, `git diff --check` exit 0, clean worktree, Linux x86_64, rustc 1.98.0. Connected GitHub status/workflow endpoints expose no hosted run/status for this exact SHA; absence is not failure evidence.

No decoder/parser/crypto-framing code was changed by this review, so no new fuzz claim is made.

## Exclusions / still open

- D019 source-retention/no-reset and retained-history capacity policy remain maintainer/policy gates.
- No TTL/LRU/history-size/capacity/security value, signing/key-custody/SBOM/publication policy, frozen-release policy, destructive migration, core wire/crypto architecture, or release authority is selected here.
- No WAN/live/performance conclusion is added. `READY_LIVE: none` remains the current classification absent a new concrete real-network question.
- Release items 3 and 4 remain incomplete. This note closes only the dedicated final independent R9 bounded-review lane; repository-wide item-4 inventory/refill still follows after Q10/Q11/Q12.

**Classification:** final independent R9 bounded no-finding review support accepted at exact source/test anchor `8cbd9af39a600f4ddd5fbc50208791e8584c6d74`.

Continue immediately to Q10 factual reconciliation; do not wait for reviewer cadence.
