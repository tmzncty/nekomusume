# Nekomusume ChatGPT Handoff

Checked at: 2026-09-03 16:00 Asia/Shanghai
Repository HEAD reviewed: `3d545859e06690c528a717015c9b7023d05ea420`
Previous reviewed implementation HEAD: `308e46e5efeb988a224538f9899221c3fa6c06d3`
Previous reviewer handoff commit: `866ea6197d0d5845dbeef50f779b34d9c2e3eb8f`

## What changed

One coding-agent commit is visible after the previous reviewer handoff:

- `3d54585` — **owned-lab benchmark cleanup/preservation hardening; no Nekomusume wire/Session/Noise/failover semantic change and no new VPS comparison result.** It removes the routine-test deletion of the default evidence path, exercises both `nekomusume` and `hy2` `run_client` branches under `set -u`, introduces a shared process/descendant/listener cleanup primitive, uses bounded TERM->KILL convergence polling for local and remote experiment-owned processes/listeners, delays remote temp-path deletion until process/listener cleanup is verified, and adds a real disposable descendant/listener cleanup regression. The default-result sentinel regression proves a pre-existing result and sample companion remain byte-identical through validation.

The exact coding HEAD has independently green GitHub Actions run `33730718093`:

- `stable checks` — `bash scripts/check.sh` succeeded;
- `nightly decode fuzz smoke` — pinned cargo-fuzz decode build and 30-second / 8,192-byte smoke succeeded.

The previous negative paid artifact at exact `bc38d06` remains immutable and validator-valid. Its cleanup failure remains historical evidence; `3d54585` is a later repair and must not be retroactively attributed to that attempt.

Reviewer-side Hysteria documentation check confirms the current TLS direction is compatible with upstream Hysteria 2 documentation: `insecure: true` plus `pinSHA256` using the `openssl x509 -noout -fingerprint -sha256` fingerprint format is explicitly documented. Current Hysteria server documentation says default `sniGuard=dns-san` activates SNI matching only when the certificate contains a DNS-name SAN; the disposable benchmark certificate currently contains an IP SAN, so there is no evidence-based reason to add `sniGuard:disable` before the next direct owned-lab retry. This is upstream behavior research, not a Nekomusume protocol decision.

Upstream references:

- https://v2.hysteria.network/docs/getting-started/Client/
- https://v2.hysteria.network/docs/advanced/Full-Client-Config/
- https://v2.hysteria.network/docs/advanced/Full-Server-Config/

## Review verdict

**SAFE_TO_CONTINUE — cleanup/preservation gate accepted; execute exactly one materially changed HY2/Nekomusume owned-lab paired attempt now**

The paid comparison path has satisfied the previous local repair and exact-head CI prerequisites. The VPS rental window is time-limited, and the HY2 paired row is now the highest-value READY VPS-only evidence task.

Do not create another local-polish round before the paid attempt unless a new deterministic failure appears during the final exact-head preflight. Do not repeat the attempt in the same batch if it fails; retain the result and move to reconciliation/diagnosis.

## Evidence boundaries

- `IMPLEMENTATION_COMPLETE=true` remains the bounded research implementation baseline.
- `CANONICAL_CORPUS_V1_FROZEN=true` remains corpus-specific only.
- `RELEASE_CANDIDATE=false`, `PRODUCTION_READY=false`, `FREEZE=false`, and `RELEASED=false` remain correct.
- Exact `3d54585` has green stable and nightly fuzz CI. This is repository CI evidence, not an independent security audit or release approval.
- `3d54585` changes benchmark/evidence infrastructure only; it does not add transport correctness or performance evidence.
- Exact `25e0daa` controlled application-level UDP reply-cessation warm fallback and approximately five-minute periodic Session evidence remain accepted with their existing narrow boundaries. Do not rerun them merely to consume VPS time.
- Exact `f1cb9af` and `bc38d06` HY2 attempts remain negative historical evidence and must remain preserved.
- The `bc38d06` result proves no comparison; it records `BLOCKED_HARNESS`, zero samples, and cleanup failure. Later manual cleanup is a separate observation.
- IPv6 remains environment-blocked unless a genuinely owned end-to-end IPv6 path becomes available.
- Natural UDP degradation/PTO-blackhole, NAT/endpoint-change, and live migration-back/key-update/PMTUD rows remain unproven unless current executable runtime surfaces can demonstrate them truthfully.
- `scripts/bench/compare-hy2-owned-lab.sh --validate` still touches/creates its selected sample-companion path before the validate-only early exit. The routine test now protects pre-existing default evidence bytes and normal CI uses disposable validation paths, so this is **NOTE-level evidence hygiene**, not a blocker for the required unique-path paid run. If the harness is edited again, prefer making validate-only mode fully side-effect-free rather than adding more cleanup around it.

## Work Package — One Paid HY2 Pair -> Evidence Reconciliation -> Next VPS Row

### Primary A — Execute exactly one materially changed HY2/Nekomusume owned-lab paired attempt

**Goal**

Use exact `3d54585` and the repaired cleanup/evidence contract to obtain either the first complete semantically fair paired sample set or one high-information typed negative artifact. This is the current highest-value READY use of the rented VPS.

**Preconditions already satisfied**

- exact coding HEAD `3d54585` is pushed;
- exact-head `stable checks` and nightly decode fuzz smoke are green;
- bind authority and connect authority are separate;
- explicit SSH endpoint/user preflight exists;
- disposable HY2 certificate fingerprint pinning is enforced;
- HY2 and Nekomusume both use fresh transport-client lifecycle per timed sample;
- process/resource accounting covers sampler-owned transport process groups;
- cleanup/preservation logic is fail-closed and has deterministic descendant/listener coverage;
- standing authorization covers this bounded self-owned experiment.

**Required execution profile**

- self-owned client + owned VPS only;
- isolated/detached worktree at exact coding HEAD `3d54585`;
- record Nekomusume binary SHA-256 and pinned HY2 v2.9.3 SHA-256;
- explicit verified SSH endpoint/user;
- distinct assigned remote bind address vs reachable connect authority when the environment is NAT-shaped;
- disposable certificate/key + exact `pinSHA256` + fresh password auth;
- 5 paired runs;
- 1,200-byte deterministic payload per sample;
- concurrency 1;
- fresh unprivileged experiment ports;
- **exactly one substantive harness invocation in this batch**;
- unique non-default repository-relative result path tied to the exact HEAD/attempt;
- complete invocation including cleanup below the standing 10-minute limit;
- no production firewall/route/qdisc/DNS/proxy/tunnel/service change.

**Path diagnostics**

If the HY2 side still fails after server setup or QUIC establishment, use only bounded experiment-port observation/capture already allowed by standing authorization to classify:

1. did client UDP/QUIC packets leave the client;
2. did they arrive at the VPS temporary HY2 port;
3. did the temporary HY2 server emit responses;
4. did responses return to the client;
5. do HY2 logs instead identify TLS pin/auth/config failure.

Retain compact capture metadata/hash/packet counts/timestamps when useful. Raw pcap need not be committed. Do not change provider/firewall policy to force success.

**Outcome rules**

- setup/pre-sample failure -> typed `BLOCKED_HARNESS`, no comparison statistics;
- failed/incomplete pair -> retain valid prefix, no median/P95 comparative summary;
- complete 5-pair set -> raw samples + median/P95/failures only under the existing equal-payload/equal-lifecycle contract;
- client transport CPU/RSS/FD/application bytes must remain symmetric in scope;
- `wire_bytes` remains null unless bounded capture metadata is trustworthy;
- cleanup must be validator/evidence-clean;
- if cleanup fails, preserve the artifact exactly and do not issue another unchanged paid retry in this batch;
- no superiority/public/production claim from one bounded batch.

### Follow-up B — Reconcile the paid result into the release-evidence matrix

**Dependency:** A complete, whether positive or negative.

Update the repository's ordinary evidence/status surfaces without erasing history:

1. `docs/status.md`
   - preserve exact `f1cb9af` and `bc38d06` negatives;
   - record exact `3d54585` attempt outcome and exact evidence path/hash;
   - if positive, describe one self-owned route/time-window paired sample set only;
   - if negative, record exact failure stage and valid prefix, with no performance summary.
2. `IMPLEMENTATION_PLAN.md`
   - keep bounded release evidence item unchecked while declared matrix requirements remain open;
   - mark HY2 comparison evidence as present only if the complete paired contract passed.
3. `ROADMAP.md`
   - update the HY2 row truthfully;
   - keep natural degradation, long-lived/general reliability, NAT/endpoint-change and IPv6 boundaries unchanged unless new evidence directly proves them.
4. Add/update compact experiment note/index and artifact hashes as appropriate.

Governance flags remain unchanged.

### Follow-up C — Select and prepare the next highest-value VPS-only row from executable reality

**Dependency:** B complete.

Audit current runtime/CLI surfaces and choose the first genuinely executable missing row. Do not choose from labels alone.

Priority:

1. **genuine NAT/source-endpoint change** on owned endpoints, only if the current runtime can create and observe a real source endpoint/path change without production route/firewall modification;
2. **real-session migration-back** if current live runtime/CLI exposes the necessary carrier recovery path and truthful event instrumentation;
3. **real-session key update** only if there is an actual live Session path, not only fixture/state-model coverage;
4. **live-path PMTUD** only if real packetization/probe instrumentation exists;
5. otherwise implement the **smallest local release-evidence seam** that directly unlocks one of the above.

If a candidate is not executable, record `BLOCKED_IMPLEMENTATION` with the exact missing seam and move to the smallest direct unlock. Do not fabricate a VPS row from a fixture.

### Follow-up D — If a next VPS row is immediately READY, execute one bounded row in the same overall work package

**Dependency:** C concludes that a row is already executable and does not require a new reviewer/security decision.

Use standing authorization directly. Keep the experiment scientifically distinct from already accepted D064/periodic baselines, retain exact binary/parameter/timestamp/cleanup evidence, and stay within the existing 10-minute / 256 MiB / 32-session bounds.

If C instead identifies a missing implementation/instrumentation seam, do that local seam and its tests/gates instead of inventing a run.

### Follow-up E — Evidence-hygiene cleanup if the benchmark adapter is touched again

**Dependency:** A/B complete; lower priority than real VPS evidence.

If another benchmark-harness change is already necessary, make `--validate` fully side-effect-free: validate-only mode should not create or `touch` the selected `*.samples.jsonl` path unless an explicit disposable output path is requested. Add an absence-preservation regression as well as the existing byte-preservation sentinel. Do not spend a standalone paid-VPS window or block a READY VPS row solely for this NOTE.

## Fallback

If Primary A produces a new deterministic harness/control-plane defect:

- retain the typed result exactly;
- do not rerun unchanged;
- make the smallest local repair + deterministic regression the next Primary;
- run the full local gate and exact-head CI;
- use the next paid retry only after code/instrumentation/hypothesis materially changes.

If A instead proves a network/path block with client/VPS packet-direction evidence and no safe in-scope configuration change remains, preserve that negative result, complete B, and move immediately to C/D rather than repeatedly spending the rental window on the same HY2 path.

If C/D requires new credentials, another server, third-party access, production network modification or anything outside standing authorization, stop only that row, record the exact blocker, and continue the highest-value local unlock/other READY release evidence. Notify the maintainer only if the new external requirement is genuinely unavoidable.

## Completion gates

- exact `3d54585` cleanup/preservation repair remains green and is not rewritten as prior experiment evidence;
- exactly one substantive paid HY2/Nekomusume invocation is performed in Primary A;
- the result uses a unique evidence path and exact coding/binary identity;
- positive or negative evidence is retained without overwriting historical attempts;
- no incomplete pair enters comparative statistics;
- cleanup state is machine-readable and fail-closed;
- release/status documents reflect the exact A outcome without claim inflation;
- at least one next VPS opportunity is either executed truthfully or reduced to a concrete `BLOCKED_IMPLEMENTATION` + smallest unlock seam;
- `RELEASE_CANDIDATE=false`, `PRODUCTION_READY=false`, `FREEZE=false`, and `RELEASED=false` remain unchanged.

## Do not expand into

- changing Nekomusume wire/Session/Noise/failover semantics for benchmark convenience;
- weakening authentication/integrity or HY2 certificate pinning;
- production network changes, third-party targets or scanning;
- repeated unchanged paid attempts;
- publishing superiority claims from one bounded batch;
- treating fixture-only key-update/PLPMTUD/manager behavior as live WAN evidence;
- speculative FEC/0-RTT/striping/exotic-carrier work without an observed-problem gate.

## Questions requiring maintainer decision

none.

Standing authorization covers Primary A and any Follow-up D row that stays within the existing bounded self-owned TCP/UDP experiment contract. A maintainer decision is required only if later evidence demonstrates a genuinely new credential/server/environment requirement, an action outside standing authorization, or a major architecture choice that repository facts cannot safely resolve.
