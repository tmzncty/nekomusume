# Independent bounded review — R9-8 warm TCP readiness / directional forwarding

**Review anchor:** developer review `7f0fcdafbe908d28a7d7715a2b04725ee31883e2`; exact source/test tree remains `d97a536414a78ff44ef80b1a6d428ddf96ae6e22` because `7f0fcd` is docs/review-only.

**Result:** bounded no finding. This closes only the R9-8 review lane. It does not close release item 3 or item 4, does not create WAN evidence, and does not change RC/freeze/release/production authority.

## New developer commit classification

`7f0fcdafbe908d28a7d7715a2b04725ee31883e2` adds only `docs/reviews/dev-r9-8-warm-tcp-readiness-20260919.md`. No implementation, test, fixture, package, live-experiment, protocol or policy file changed. The developer note was therefore treated as review support, not as a new tested implementation anchor.

## Owners and contracts independently challenged

The review re-read the current D064 contract in `docs/adr/m3-concurrent-carrier-semantics.md` / `docs/decisions.md`, the current Carrier Manager owners in `crates/neko-carrier/src/lib.rs`, and the executable failover/readiness owners and process regressions in `crates/neko-cli/src/main.rs` and `crates/neko-cli/tests/probe.rs`.

The concrete owners challenged were:

- `CarrierManager::prepare_warm_candidate`, `observe_warm_candidate_readiness`, `fail_udp_to_tcp`, and `promote_warm_authenticated_resume`;
- `ConcurrentCarrierManager::observe_readiness`, `activate`, generation lookup/mutation, single-active/drain state, plus the `ReliableUdpRuntime` terminal readiness/control surface;
- CLI `readiness_admitted`, the warm TCP negotiation/auth/resume/readiness loop, post-failure warm promotion, and the subsequent TCP Session replay/send path;
- the positive warm-fallback process fixture and `warm_readiness_failures_close_before_admission_or_application_data` negatives.

## Invariant challenge

1. **Warm standby is control-only before promotion.** The current client completes negotiation, Noise authentication, resume binding and exactly three authenticated/admitted readiness observations while UDP remains active, stores the TCP stream/session as warm state, then waits for the UDP failure decision. The current executable send path does not emit a `ProcessMessage::Data` on TCP until after `promote_warm_authenticated_resume`. The D064 runtime addendum explicitly defines the three-request warm proof and the current source follows that ordering.

2. **No TCP application-data ownership while UDP remains active.** `CarrierManager` keeps warm-candidate state separate from active ownership. The concurrent manager's assignment path uses only the active owner, not a merely warm path. Current positive process evidence also requires the warm event before resumed TCP data, while negative readiness fixtures forbid admission/warm/resume/delivery evidence.

3. **Readiness is not inferred from unrelated evidence.** Warm readiness advances only through `ReadinessObservation` with authenticated + resume-validated + resource-admitted exact candidate dimensions and distinct observation IDs. TCP connect, packet ACK and Session DeliveryAck are not inputs to that transition.

4. **Session/epoch/path-generation/direction binding remains explicit.** The executable request/response path matches the exact Session 7001 / target path 2 / generation 1 / delivery epoch 1 tuple and consecutive challenge IDs. Request and response are distinct `ProcessMessage` variants, so wrong-direction message type cannot satisfy the corresponding match. Current wrong-tuple, replay, tamper and admission-negative process fixtures close before warm/promotion/data evidence.

5. **Incomplete/failed readiness cannot steal active ownership.** Any failed current-candidate validation resets consecutive readiness; stale/wrong generation fails closed; warm promotion requires the already-warm candidate. Process negatives cover fewer-than-three, stalled, over-probe-deadline, over-sequence-deadline, tampered, replayed and unadmitted cases.

6. **Single-active / multi-ready state remains coherent through activation, drain and failure.** Promotion is a manager-owned transition, the target must already be warm, the previous active owner is retired/drained according to the current contract, and generation-scoped failed state is terminal. No second active application owner is exposed by the reviewed owners.

7. **Terminal runtime cannot manufacture readiness/control evidence.** The current `ReliableUdpRuntime` terminal guard rejects post-teardown readiness/activation/mutable-manager transitions; R9-6 already separately closed terminal ownership/history/control-plane semantics.

8. **Structured positive readiness follows typed transition.** `tcp_warm_readiness` is emitted from the per-observation result, and the single `tcp_warm` marker follows the manager's warm-state check; process tests require exactly three readiness events and exactly one warm transition. The human-readable `application_data=0` field is not treated as an independent accounting oracle; the stronger reviewed invariant is executable ordering: no TCP `Data` send path is reached before manager promotion.

No concrete correctness/security/evidence defect was found at the exact current source/test owners. In particular, the developer note's narrower emphasis on `ConcurrentCarrierManager` did not by itself justify closure; this independent pass additionally checked the separate `CarrierManager` + CLI cross-process D064 path named by the handoff.

## Deterministic regression evidence inspected

Existing deterministic owners inspected include the Carrier Manager readiness/generation/single-active unit tests, the positive warm failover process fixture, and `warm_readiness_failures_close_before_admission_or_application_data` covering wrong tuple, unadmitted response, replayed challenge, tampered ciphertext, incomplete proof, per-probe timeout and whole-sequence timeout.

No reviewer-local command execution is claimed in this note. The exact source/test tree `d97a536414a78ff44ef80b1a6d428ddf96ae6e22` already has reachable developer-local clean-tree provenance for `PYTHONDONTWRITEBYTECODE=1 bash scripts/check.sh` and `git diff --check`, with hosted CI retained separately as cross-evidence. `7f0fcd` itself is documentation-only and does not supersede that tested-tree anchor.

## Exclusions

No WAN/VPS experiment, benchmark/performance conclusion, decoder/framing change, fuzz run, policy-number change, D019 decision, Session/Carrier/ACK/crypto/wire redesign, retained artifact rewrite, or release-authority transition was performed or inferred.

## Queue consequence

R9-8 is bounded no-finding closed. Continue immediately to R9-9 health outcome / promotion / switch ordering; preserve the remaining R9-10 through R9-12, final independent R9 review, Q10/Q11/Q12 reconciliation, repository-wide item-4 refill, and conditional changed-question-only READY_LIVE lanes.
