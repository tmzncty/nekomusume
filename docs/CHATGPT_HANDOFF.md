# Nekomusume ChatGPT Handoff

Checked at: 2026-09-01 12:02 Asia/Shanghai
Reviewed implementation HEAD: `b191dd8181e3f6023eb4c1c43c43e5fd1ff0518c`
Previous reviewed implementation HEAD: `4ecd29979e633cea02c17abb8e448e1e37ab8ad7`
Previous reviewer handoff commit: `47b0de0a85b2b84ba2419ebd96be50257ffa85af`
Reviewer research-support commit before this handoff: `f1bf9afe5d878cb787429eacc8fd42d2eab1f810`

## What changed

Two substantive coding-agent commits landed after the previous reviewed implementation HEAD.

- `33310fc` — **real self-owned VPS behavior/evidence; no production claim.** It records replacement current-head cross-host IPv4 evidence:
  - generic negotiated/authenticated TCP PASS;
  - generic negotiated/authenticated UDP PASS;
  - malformed TCP negotiation fail-closed;
  - controlled UDP -> TCP resume PASS with encrypted exact-semantic `udp_delivery_ack_validated` and `tcp_delivery_ack_validated`, exactly 3 application records / 111 B observed at the server, no missing/duplicate application delivery in the final state;
  - an 8-cycle alternating TCP/UDP lifecycle sample with **7 successes / 1 retained UDP failure**;
  - IPv6 correctly classified as environment-blocked because no owned end-to-end IPv6 route existed;
  - cleanup PASS with no experiment listeners/processes left.
- `b191dd8` — **benchmark/evidence tooling + one measured VPS use; no protocol/runtime feature change.** It adds a bounded Linux direct-child process CPU/RSS/FD/owned-socket sampler, schema, validator, deterministic regression suite, `scripts/check.sh` integration, documentation, and one short measured current-head TCP VPS exchange. The measured row reports both roles exit 0 with small process CPU/RSS/FD observations and explicit direct-child-only scope.

Independent GitHub Actions evidence is green for exact implementation HEAD `b191dd8`: Rust CI run #80 completed successfully. The repository-wide check includes the new sampler regression.

Reviewer research support in `f1bf9a` records an official Hysteria2 comparison seam: current Hysteria2 client configuration supports local TCP forwarding through the temporary HY2 connection to a remote target, which can be used with a temporary loopback echo target on the owned VPS. This can provide equal application-level echo semantics without touching the existing production Hysteria service. It is a methodology note, not benchmark evidence.

## Review verdict

**CONTINUE WITH REQUIRED FIXES — VPS evidence harvesting is succeeding, but two newly observed correctness/evidence issues must drive the next batch before performance comparison.**

The project is not blocked. The previous A/B package produced valuable real-socket evidence and reusable instrumentation. However:

1. the new process-resource sampler has a concrete exit-race bug in its evidence path; and
2. the retained 7/8 lifecycle result exposed a real post-authenticated UDP data timeout that must be investigated rather than averaged away.

Both issues are more important than unrelated local polish or an immediate HY2 performance run. They are also small enough to close while preserving the rented-VPS priority.

The bounded release evidence matrix remains the first unchecked release-engineering item. `RELEASE_CANDIDATE=false`, global `FREEZE=false`, `PRODUCTION_READY=false`, and `RELEASED=false` remain correct. N9 remains closed only for the canonical corpus (`CANONICAL_CORPUS_V1_FROZEN=true`).

## Evidence boundaries

### Accepted new facts

- `33310fc` replaces the old pre-`f680702` ACK wording with real current-head authenticated exact-semantic Session DeliveryAck evidence on self-owned TCP/UDP sockets.
- Controlled failover/resume is now proven on the owned cross-host path for the explicit `controlled_udp_stop` injection.
- Exactly 3 application records / 111 B completed in that controlled failover run with no missing/duplicate final application delivery observed.
- Generic current/current TCP and UDP authenticated cross-host echo both passed on IPv4.
- A malformed TCP negotiation row failed closed before application success.
- The lifecycle sample is **not** 8/8: one UDP cycle authenticated successfully and then ended with client `echo timeout` / server `data timeout`. This negative result is retained and is now an observed problem that should guide engineering.
- IPv6 is currently `BLOCKED_ENVIRONMENT`, not a code/release PASS or permission blocker.
- `b191dd8` provides one useful direct-child process-resource observation, but it is not capacity/stress evidence.
- GitHub CI #80 is green at exact implementation HEAD `b191dd8`.

### Required repair findings

**R-001 — evidence sampler immediate-exit race (required before HY2/resource comparison use).**

In `scripts/bench/process-resource-sampler.py`, `read_proc()` initializes `cpu` as `None` when `/proc/<pid>/stat` cannot be read, but the main sampling loop unconditionally evaluates `cpu[0]`. A sufficiently fast exit or `/proc` race can therefore raise `TypeError` instead of emitting truthful nullable metrics. The existing `/bin/true` regression makes the race *possible* but does not deterministically force the missing-`/proc` return contract, so green CI does not close this path.

This is an evidence-tool correctness defect, not a Nekomusume transport defect. Fix it before using the sampler as part of HY2 comparative evidence or broader resource claims.

**R-002 — real UDP post-auth data timeout requires diagnosis/repair before lifecycle robustness claims.**

The B3 failure happened after negotiation/authentication. Current generic UDP server code configures a 100 ms socket read timeout and later uses `recv_from(...).unwrap_or_else(|_| fail("data timeout"))` for authenticated application data. The generic UDP client uses the requested bounded `--duration` as its read timeout. Therefore the server currently has a much shorter post-auth data wait than the client/experiment contract, and a transient scheduling/network gap can terminate a valid authenticated run after 100 ms.

Do not assume this is the sole cause until a deterministic regression proves it, but it is a concrete code/evidence mismatch worth testing immediately. The correct fix should separate short poll granularity (for shutdown responsiveness) from the overall bounded stage deadline, rather than merely replacing 100 ms with an arbitrary large constant.

### Status/navigation drift

`docs/status.md` is stale relative to `33310fc`:

- `live-tcp` still says replacement authenticated-DeliveryAck VPS evidence is pending;
- `live-udp` still describes loopback-only evidence despite the new bounded self-owned cross-host authenticated UDP sanity row.

Repair those evidence links/boundaries after the runtime/evidence repair. Do **not** promote public/general reachability, natural degradation failover, production readiness, or the overall release matrix.

`ROADMAP.md` correctly keeps real `UDP degradation / TCP fallback` unchecked because `controlled_udp_stop` is not automatic health/PTO-driven degradation. Keep that distinction.

## Work Package — close the observed UDP/evidence defects, then harvest automatic failover and comparison evidence

Execute A -> B -> C -> D in order while dependencies remain green. This package is intentionally thicker than one small fix: A includes deterministic repair + replacement VPS evidence; B moves directly into the highest-value remaining release behavior; C/D keep the rented VPS productive without inventing speculative features.

### Primary A — Repair evidence sampler race and close the observed real-UDP lifecycle defect

**Goal**

Make the resource evidence tool race-safe, determine whether the 100 ms server post-auth timeout explains the retained B3 failure, repair the bounded data-wait contract if proven, and immediately collect replacement lifecycle evidence on the rented VPS.

#### A1 — Make the process-resource sampler race-safe

Required behavior:

- `read_proc()` must have a stable return shape even when every `/proc/<pid>` read races with process exit; use `(None, None)` or an equivalent explicit CPU-value contract rather than a nullable tuple container that callers index unsafely;
- the main loop must never crash merely because `/proc/<pid>/stat`, `/status`, `/fd` or `/proc/net/*` disappears during exit;
- unavailable metrics remain `null`, never fake zero;
- preserve the documented direct-child-only scope and bounded timeout/process-group behavior;
- do not expand this into host-wide monitoring or cgroup accounting.

Add a **deterministic** regression for the missing-process/read failure contract. Do not rely only on hoping `/bin/true` exits at the right microsecond. A direct unit/import test of `read_proc()` against a guaranteed-nonexistent PID, or a small explicit seam, is sufficient if it proves the caller-visible shape and no exception path.

Run the targeted sampler test, validator/schema checks, `bash scripts/check.sh`, and `git diff --check`.

#### A2 — Prove and repair the generic UDP post-auth data-wait mismatch

Use the retained B3 failure as the observed-problem gate.

Current behavior to test:

- server UDP socket uses a 100 ms read timeout;
- after successful negotiation/Noise authentication, application `recv_from` treats the first timeout/error as terminal `data timeout`;
- client application receive uses the bounded requested `--duration`.

Implement the smallest truthful bounded contract:

- keep short polling if needed for signal/shutdown responsiveness;
- track an explicit overall application-stage deadline derived from the already bounded requested duration (max 30 s for the generic probe);
- treat `WouldBlock`/`TimedOut` before that deadline as a poll miss, not immediate terminal failure;
- fail deterministically once the overall deadline is exhausted;
- preserve peer binding, authentication, payload bounds and fail-closed handling of real socket/auth errors;
- do not turn the research probe into a long-lived service.

Add deterministic process/integration regressions:

1. complete negotiation/authentication, intentionally delay first application data for **>100 ms but < configured duration**, and prove the exchange still succeeds;
2. delay beyond the configured overall deadline and prove bounded failure;
3. preserve signal/lifecycle cleanup and existing negotiated/authenticated semantics.

Prefer a test harness/client delay rather than adding a public production flag solely for this regression unless an existing diagnostic seam is the cleaner fit.

If the test disproves the 100 ms hypothesis, keep the negative result and add stage-safe diagnostic instrumentation (no payload/secrets) to isolate the next cause before changing semantics.

#### A3 — Replacement VPS lifecycle diagnostic after the code/instrumentation change

Because A1/A2 materially change code/instrumentation/hypothesis, a new VPS run is permitted and scientifically distinct from the retained 7/8 sample.

On the same class of self-owned IPv4 client/VPS path, run one bounded repeated lifecycle sample after the fix, preferably 12-16 alternating TCP/UDP cycles with small payload/count and total wall clock comfortably below 10 minutes. Record:

- exact commit/binary identity;
- cycles by transport, success/failure;
- stage of any failure (negotiation / Noise / post-auth data send / post-auth data receive / close);
- application bytes for successful cycles;
- process-resource sampler data for representative or aggregated roles once A1 is green;
- listener/process cleanup;
- old B3 7/8 result remains immutable negative evidence and is not overwritten.

Do not mechanically rerun a failed cycle within the same evidence row to manufacture 100% success.

#### A4 — Reconcile status evidence after A3

Update `docs/status.md` evidence/boundaries so they reflect repository facts:

- `live-udp`: bounded self-owned cross-host negotiated/authenticated UDP echo evidence exists; retain the exact non-public/non-production boundary and any remaining lifecycle instability if A3 still fails;
- `live-tcp`: replacement current-head controlled-resume authenticated DeliveryAck evidence exists; keep natural threshold-driven degradation absent until B is complete;
- link the small replacement evidence documents where appropriate.

Do not mark `reachability`, `production`, or the whole bounded release matrix PASS.

**Primary A completion definition**

- sampler exit race is deterministically closed;
- the 100 ms post-auth UDP behavior is either proven causal and repaired, or disproven with a more precise new diagnostic finding;
- replacement VPS lifecycle evidence exists after a meaningful code/instrumentation change;
- old 7/8 negative evidence remains visible;
- full repository local gate passes and exact-head CI is allowed to run normally.

### Follow-up B — Connect actual carrier-health degradation to real failover, then take VPS evidence immediately

**Dependency:** A green. Controlled-stop failover is already accepted; this closes the next real release-behavior gap.

**Goal**

Make UDP -> TCP transition occur because existing Nekomusume carrier-health/failure state observes bounded UDP degradation, not because the client executes an unconditional `controlled_udp_stop`.

Repository fact to preserve: `CarrierState`/`CarrierHealthEvidence` already model health/path state, but the CLI failover runner is not currently driven by that state. The CLI's `health-observe` path is separate evidence, not integrated failover.

Required engineering:

1. map the smallest real-socket observation seam into the existing health model; do not create a parallel `if timeout { switch_to_tcp }` controller;
2. use an application-level self-owned fault source within standing authorization, e.g. the experimental UDP server deliberately ceases authenticated application replies after a deterministic logical point while TCP remains available; do not modify VPS firewall/route/qdisc;
3. reuse the repository's actual `HealthLimits` / PTO/failure/hysteresis vocabulary and thresholds, recording exact transition/reason events rather than inventing a new release-only threshold;
4. keep the same logical Session/DeliveryLedger/ResumeGuard state across transition;
5. mark the affected unconfirmed logical range uncertain, resend according to the Session contract, and assert receiver dedup/exactly-once final application delivery where supported;
6. preserve canonical negotiation + fresh Noise authentication on resumed TCP;
7. deterministic local/process test must prove the health transition drives failover without calling the controlled-stop path;
8. all retries, samples and resource use remain bounded.

If the existing health model lacks one adapter needed by the real runner, implement that smallest adapter and document the mapping. Do not redesign the Carrier Manager wholesale.

**VPS opportunity immediately after local green:** run one bounded self-owned real-socket degradation -> TCP resume row with the repaired resource sampler. Record transition timestamps, health state/reason, recovery time, uncertain/replayed/duplicate/missing bytes or records, CPU/RSS/FD, and cleanup. Classify it as a controlled self-owned degradation injection, not arbitrary Internet blackhole proof or production failover.

### Follow-up C — Add one real authenticated bounded resilience runner and harvest a 5-minute sample

**Dependency:** A green; B may proceed first because it is the more direct release blocker. Use C while B is environment-blocked or immediately after B.

The existing `workload` command is an in-process fixture and does **not** open real sockets. The generic probe is bounded to short exchanges. The rental-window backlog still needs a real authenticated longer-lived socket observation.

Build the smallest **experimental, bounded, non-service** real-session runner that reuses the generic negotiated/Noise transport path and supports:

- one self-owned TCP or UDP session;
- total duration explicitly bounded `1..600 s`;
- small periodic authenticated echo (fixed interval with a bounded minimum/maximum, not a tight loop);
- bounded payload and total application bytes below standing limits;
- structured counters: attempted/successful exchanges, failures, application bytes, last-success stage, start/end;
- graceful signal stop/cleanup;
- resource sampler integration;
- no forwarding/proxy/tunnel behavior.

After deterministic/local tests, run **one 5-minute** self-owned VPS scenario (prefer UDP steady/periodic first). Record failures, missing/duplicate observations only where the protocol proves them, CPU/RSS/FD/socket metrics, timestamps, binary identity and cleanup. This is resilience evidence, not capacity or production uptime.

Do not chain multiple runs to simulate a forbidden >10-minute soak.

### Follow-up D — Prepare the first fair HY2 v2.9.3 paired self-owned-VPS sample

**Dependency:** A1 sampler repair green + generic network sanity green. B/C should not be abandoned, but HY2 should not be postponed to the end of the rental month.

Read:

- `docs/bench/hy2-vps-setup-20260830.md`;
- `docs/bench/hy2-comparison-workload.md`;
- `docs/research/hy2-forwarding-comparison-note-20260901.md`.

Reviewer research confirms from official Hysteria2 documentation that client `tcpForwarding` can expose a local TCP listener and forward it through the temporary Hysteria connection to a remote target reachable by the HY2 server. Use this to avoid changing the production HY2 service:

```text
client payload/echo tool
 -> temporary HY2 client local TCP forward
 -> temporary HY2 v2.9.3 server on fresh high UDP port
 -> temporary loopback TCP echo service on VPS
 -> back through HY2
```

Use only experiment-generated config/cert/auth under disposable paths. Do not read/reuse `/etc/hysteria/server.yaml` secrets; do not stop/reconfigure the existing HY2 process. Do not use port hopping or Mimic.

Preserve `scripts/bench/compare-hy2.sh`'s loopback-only guard; build a separate self-owned-VPS orchestrator/wrapper or reuse only its result schema/methodology.

For both Nekomusume and HY2 fix the same application question and metadata:

- exact payload bytes + SHA-256;
- same client/VPS pair and close time window;
- route/MTU metadata;
- security class described truthfully as authenticated encrypted research configs (do not claim identical cipher/handshake);
- stream/load shape;
- run count 5 unless a smaller diagnostic set is needed first;
- finite timeout;
- process CPU/RSS/FD from the repaired sampler;
- raw samples and failures retained;
- `wire_bytes=null` unless a trustworthy bounded capture supplies it.

Start with a small payload that both implementations can carry under their existing APIs without redesign; do not expand Nekomusume framing solely to win a throughput test. This first paired set is system-level application-exchange comparison evidence only, not a superiority or capacity claim.

## Completion gates for this handoff

- `33310fc` B1/B2 evidence is accepted within its explicit self-owned/controlled boundaries.
- The B3 7/8 lifecycle result remains visible as negative evidence until a changed-code/instrumentation replacement run exists.
- Resource sampler cannot crash on the missing-`/proc` direct-child exit race and has a deterministic regression for that path.
- Generic UDP post-auth data waiting respects an overall bounded stage deadline rather than treating one 100 ms poll miss as the full experiment timeout, if the regression confirms that diagnosis.
- Replacement lifecycle evidence records stage-specific failures and cleanup; no failed cycle is silently retried away.
- `docs/status.md` no longer says cross-host UDP is loopback-only or replacement DeliveryAck VPS evidence is pending once the corresponding evidence is accepted.
- Automatic health-driven failover remains unchecked until existing health state actually drives the transition in deterministic tests and a bounded VPS row.
- A 5-minute real authenticated session sample, if C is reached, remains under standing limits and is not promoted to production uptime/capacity.
- HY2 comparison uses the pinned v2.9.3 artifact, equal application semantics/metadata, disposable experiment config, repaired resource metrics and untouched production service.
- `IMPLEMENTATION_PLAN.md` Bounded release evidence matrix remains unchecked until its actual matrix criteria are met.
- `RELEASE_CANDIDATE=false`, global `FREEZE=false`, `PRODUCTION_READY=false`, `RELEASED=false` remain unchanged.

## Fallback

If A2/A3 exposes a real crypto/Session correctness defect rather than timeout-policy fragility:

1. preserve the exact negative evidence and minimal reproducer;
2. stop new performance/release claims on that path;
3. repair correctness first and run required parser/fuzz gates if affected;
4. rerun only after code/instrumentation/hypothesis materially changes.

If VPS access is temporarily unavailable, do A1/A2 and the local implementation/test portion of B/C; these directly unlock the next VPS evidence. IPv6 remains environment-blocked and must not stop IPv4 work. If an action would exceed standing authorization, stop only that action and continue independent READY work.

## Do not expand into

- RC/production/security approval;
- erasing or rerunning away the B3 UDP failure;
- calling controlled-stop failover automatic degradation;
- weakening authentication/integrity/resource bounds for benchmark speed;
- changing or reading production HY2 secrets/config/service;
- weakening the local comparison harness's loopback guard;
- third-party targets, scans, production firewall/route/DNS/proxy/tunnel/qdisc changes;
- >10-minute single runs, >256 MiB single-run application traffic, >32 concurrent experimental sessions, or disguised split pressure tests;
- IPv6 code polish as a rental-window priority while no owned IPv6 path exists;
- 0-RTT, enabled FEC, striping/aggregation or exotic carriers without an observed-problem gate;
- public/general reachability, superiority or production claims from self-owned evidence.

## Questions requiring maintainer decision

none.
