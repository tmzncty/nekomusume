# Nekomusume ChatGPT Handoff

Checked at: 2026-09-02 11:59 Asia/Shanghai
Repository HEAD: `ee371694eaa9aff782ee5e5b7e933144447b4d46`
Previous reviewed implementation HEAD: `e07066b4c3f8e3ad8b33af08f27682173f415a9c`
Previous reviewer handoff commit: `25e0daa4e74b3239568067afa967412ec4c0ebc7`

## What changed

Four commits are visible after the previous reviewed implementation HEAD: one reviewer handoff plus three coding-agent commits.

- `25e0daa` — reviewer handoff only; resumed VPS-first release evidence and kept HY2 evidence truthfulness as a prerequisite for another comparison run.
- `53cb859` — **real self-owned VPS evidence; no production semantic change.** It records one exact-`25e0daa` D064 warm controlled-fault run. Canonical UDP negotiation and Noise authentication completed, one record was confirmed on UDP, two ranges became uncertain after the bounded application-level UDP reply-cessation seam, warm TCP completed negotiation/authentication/resume and three authenticated readiness responses before promotion, no TCP application data was sent before promotion, the two uncertain ranges were replayed and acknowledged on TCP, and final application accounting was 3/3 records = 48 bytes, uncertain/replayed 2 records = 32 bytes, duplicate 0, lost 0. Recorded recovery from failure decision to first resumed data acceptance was 434,287 us. Client/server exited 0 and separate post-run observations found no experiment process/listener residue.
- `4e1ef88` — **real self-owned VPS periodic evidence; no production semantic change.** It records one corrected direct-path approximately five-minute authenticated TCP Session from exact parent `25e0daa`: 60 records × 32 bytes = 1,920 application bytes, 60/60 confirmed, missing/duplicate/conflict 0, runtime confirmation-latency median 272 ms and P95 510 ms, client/server exit 0, plus direct-child CPU/RSS/FD observations and explicit post-exit process/listener cleanup. This is one bounded sample, not production/sustained proof.
- `ee37169` — **benchmark-harness/schema/test/documentation repair; no Nekomusume wire/Session/failover semantic change and no new VPS comparison.** It makes HY2 payload provenance prepared-or-null, keeps missing cleanup observations nullable rather than inventing values, strengthens sampler-owned process-group descendant cleanup/verification, and updates blocked-result validators/tests. The current GitHub Actions run is green: both `stable checks` and the nightly 30-second decode fuzz smoke completed successfully.

The previous handoff's D064 and periodic VPS rows are therefore closed with positive bounded evidence, and its HY2 local truthfulness repair is closed at the current HEAD. The bounded release-evidence matrix remains open.

A new reviewer finding blocks an immediate HY2 comparison run: the current owned-lab adapter still conflates the **remote listen/bind address** with the **client connect address**, and its per-sample lifecycle/resource accounting is not yet symmetric enough for a fair paired claim.

## Review verdict

**continue with required benchmark-contract repair — two high-value VPS rows accepted; HY2 paired comparison is READY only after topology/security/lifecycle comparability is repaired**

There is no new production-runtime correctness blocker in D064 or periodic Session behavior from this review. Do not rerun those successful rows just to consume VPS time.

Before spending another VPS window on HY2, repair the paired harness so it cannot repeat a NAT-shaped address error or compare a fresh Nekomusume Session against a pre-established HY2 tunnel under unequal security/resource measurement. Then run one changed-hypothesis owned-lab attempt with bounded capture/diagnostics.

## Evidence boundaries

- `IMPLEMENTATION_COMPLETE=true` remains the bounded research baseline status.
- `CANONICAL_CORPUS_V1_FROZEN=true` remains corpus-specific only.
- `RELEASE_CANDIDATE=false`, `PRODUCTION_READY=false`, `FREEZE=false`, and `RELEASED=false` remain correct.
- Current GitHub Actions at `ee37169` are independently green for the repository stable gate and nightly decode fuzz smoke. This is CI evidence, not a security audit or release approval.
- The positive D064 artifact is tied to exact implementation parent/binary identity `25e0daa` / SHA-256 `8ede1564...`. Later commits in this reviewed delta are evidence/harness changes; do not rewrite the artifact as if it had run at `ee37169`.
- D064 proves one **controlled application-level UDP reply-cessation** warm failover on self-owned cross-host sockets. It does not prove natural Internet loss/PTO blackhole behavior, public reachability or production resilience.
- The periodic row proves one approximately five-minute self-owned direct-path authenticated Session with 60/60 confirmations. It is not a long-duration production soak or a reliability rate estimate.
- The resource JSON generated for D064/periodic predates `ee37169`'s stronger process-group sampler contract. Preserve those resource records at their original direct-child scope. The evidence docs separately record explicit post-exit exact-process/listener observations; do not retroactively upgrade old sampler fields to the new group-cleanup semantics.
- HY2 paired comparative samples, medians/P95 and superiority evidence remain absent.
- IPv6 remains environment-blocked; no real owned end-to-end IPv6 path is currently demonstrated.
- `IMPLEMENTATION_PLAN.md` and `docs/status.md` are now stale with respect to the new positive D064/periodic evidence: they still say current-exact-head warm/periodic evidence is absent. This is status/evidence drift to repair after the next benchmark-contract/VPS step; it must not erase the historical negative rows.

### HY2 reviewer research finding

The current `scripts/bench/compare-hy2-owned-lab.sh` has two concrete comparability defects that should be repaired before execution:

1. **Bind/connect conflation.** `LAB_REMOTE_BIND_ADDRESS` is separately required to be an address assigned to a VPS interface, but the generated HY2 client uses that bind address as `server:` and the Nekomusume client also uses it as `--addr`; local MTU route lookup also targets the bind address. `LAB_REMOTE_ADDRESS` is separately verified as the client/SSH-reachable endpoint. On a NAT-shaped VPS, the server must listen on an assigned local address while the client connects to the reachable endpoint; those values are allowed to differ by the current contract. The prior HY2 evidence explicitly encountered a NAT/public-bind setup defect before the later timeout. Reusing the bind address as the client target can therefore recreate the failure for the wrong reason.
2. **Security/lifecycle/resource asymmetry.** The generated HY2 client currently uses `tls.insecure: true` without pinning, and one HY2 client/tunnel is started before all timed samples while each Nekomusume sample creates a fresh client/session. Per-sample HY2 GNU-time data then measures the local echo helper through the already-running tunnel, not the transport client lifecycle, while Nekomusume's timed command includes its transport client. Those numbers are not a fair same-lifecycle CPU/RSS/latency comparison.

Official Hysteria 2 client documentation states that a self-signed certificate should use a custom CA or, when `insecure` is used, should pair it with `pinSHA256`; bare `insecure` is explicitly not recommended. Relevant upstream references:

- https://v2.hysteria.network/docs/getting-started/Client/
- https://v2.hysteria.network/docs/advanced/Full-Client-Config/

This is upstream Hysteria behavior; the exact fair-pair lifecycle remains a Nekomusume benchmark design choice.

## Work Package — Fair HY2 Contract Repair -> VPS Paired Evidence -> Matrix Reconciliation

### Primary A — Make the owned-lab pair topology, security and timed lifecycle genuinely comparable

**Goal**

Repair `scripts/bench/compare-hy2-owned-lab.sh` and its validator/tests so the next VPS run answers the intended comparison question instead of measuring address mistakes, bare-insecure TLS, or a pre-warmed HY2 tunnel against a fresh Nekomusume Session.

**A1. Separate listen authority from connect authority**

- Keep `LAB_REMOTE_BIND_ADDRESS` only for remote listener binding and verification that the address is actually assigned to a VPS interface.
- Derive a distinct client `connect_authority` from the already verified `LAB_REMOTE_ADDRESS`, with correct IPv4/IPv6 authority formatting.
- HY2 server `listen:` and Nekomusume server `--bind` use the bind authority.
- HY2 client `server:` and Nekomusume client `--addr` use the connect authority.
- Client-side `ip route get` / MTU metadata must target the connect address, not the remote local bind address.
- Do not silently require bind == connect. Add a deterministic NAT-shaped test where they are intentionally different and verify generated/client command targets use the correct side.
- Preserve the existing endpoint SHA/SSH verification and non-wildcard bind-address checks.

**A2. Make HY2 server authentication truthful**

For the disposable self-signed experiment certificate, do not use bare `tls.insecure: true`.

Preferred minimal contract for this IP/NAT-shaped lab:

- keep a unique disposable certificate/key per run;
- compute its SHA-256 certificate fingerprint locally;
- configure HY2 client with `insecure: true` **plus `pinSHA256`** for that exact disposable certificate, following upstream Hysteria 2 documentation;
- retain password authentication as the bounded client-authentication mechanism;
- add a deterministic config/source test proving a HY2 owned-lab config cannot be generated with bare insecure TLS and no pin.

Using a custom CA instead is acceptable only if hostname/IP validation remains truthful when bind and connect addresses differ; do not weaken the existing security class just to make the comparison run.

**A3. Make timed client/session lifecycle symmetric**

Choose one explicit lifecycle and apply it to both implementations. Prefer:

```text
fresh client/session start
-> transport handshake/authentication
-> exact payload exchange
-> successful application echo
-> client/session close
```

for every timed sample.

The current persistent HY2 client established once before all runs is not acceptable against a fresh Nekomusume client per sample.

Required properties:

- each HY2 timed sample must include the HY2 client/session establishment needed for that sample, not only `echo-payload.py` through a pre-established local forward;
- each Nekomusume timed sample must represent the same lifecycle class;
- if servers are persistent or per-run, make that policy symmetric where practical and explicitly record it; if server lifecycle cannot be made symmetric, exclude server-startup numbers from the comparative summary and label server resource rows separately;
- per-sample CPU/RSS/FD evidence must cover the actual transport client process/group for both implementations. Do not compare Nekomusume transport-process resource use against only the HY2 local echo helper;
- application bytes and payload SHA remain exact and equal;
- failed/incomplete samples remain typed failures and never enter success medians/P95.

If the existing sampler cannot attribute a fresh HY2 client plus helper descendants truthfully, extend only the benchmark sampler/orchestrator contract needed for this comparison. Do not change Nekomusume protocol semantics.

**A4. Preserve result truthfulness**

- Comparative summary is legal only for a complete required paired set.
- Resource evidence may be aggregate/non-comparative only when labeled as such; do not silently place asymmetric aggregate metrics beside per-sample transport metrics.
- Keep `wire_bytes=null` unless a bounded capture has trustworthy metadata.
- Keep the existing prepared-or-null payload provenance and nullable cleanup observations.
- Cleanup remains scoped only to experiment-owned process groups, ports and temp paths.

### Follow-up B — Deterministic regression + local rehearsal + CI gate

**Dependency:** A complete.

Before touching the VPS again:

1. Add a deterministic test with `LAB_REMOTE_ADDRESS != LAB_REMOTE_BIND_ADDRESS` and assert:
   - remote listeners bind only to the bind address;
   - both clients target only the connect address;
   - route/MTU metadata is based on the connect address.
2. Add a config regression that proves HY2 owned-lab TLS uses the exact disposable certificate pin and rejects/removes bare insecure-only generation.
3. Add lifecycle regression showing a new HY2 transport client/session is created per timed sample (or the selected symmetric alternative), and that the timed/resource region covers the actual transport process rather than only the local echo helper.
4. Keep failure-before-payload, missing-cleanup, normal-descendant, SIGTERM/SIGINT, timeout and partial-sample truthfulness regressions green.
5. Run the complete local gate (`cargo fmt/check/test/clippy`, `scripts/check.sh`, `git diff --check`; fuzz only as required by the repository gate or relevant parser changes).
6. Push the repair and wait for the new exact-head GitHub `stable checks` and nightly fuzz jobs to be green before the VPS comparison run.

Do not spend a VPS run validating a harness whose current CI is red or pending after a substantive harness change.

### Follow-up C — Changed-hypothesis HY2/Nekomusume owned-lab paired run with bounded path diagnostics

**Dependency:** B complete and exact repair HEAD CI green.

**Goal**

Use the rental window to either obtain the first semantically fair paired sample set or classify the remaining HY2 path failure with enough packet-direction evidence that another unchanged retry is unnecessary.

**Recommended bounded profile**

- self-owned client + owned VPS only;
- pinned HY2 v2.9.3 SHA-256 already recorded in the repository;
- 5 paired runs;
- 1,200-byte deterministic payload per sample;
- concurrency 1;
- fresh unprivileged experiment ports;
- finite per-sample timeout, with the complete lab session remaining below the standing ten-minute single-run limit;
- no firewall/route/qdisc/DNS/proxy/tunnel/production-service change.

**Changed-hypothesis diagnostics**

Because the previous HY2 attempt timed out during QUIC establishment, use the newly separated connect/bind contract and add bounded capture/observation around the temporary HY2 UDP port only:

- confirm the remote HY2 server is bound on the intended local bind address/port;
- confirm the client sends QUIC/UDP packets to the reachable connect address/port;
- if the client still times out, use bounded client/VPS capture metadata to classify whether packets (a) leave client, (b) arrive at VPS, (c) elicit server responses, and (d) return to client;
- retain capture metadata/hash/packet counts/timestamps; raw pcap need not be committed if it contains unnecessary address material;
- distinguish network/path failure from TLS/auth/config failure using HY2 logs and packet direction. Do not modify provider/firewall policy to force success.

If all required paired samples succeed:

- retain raw per-sample rows;
- calculate median/P95/failures only from the complete success contract;
- report exact application bytes/hash and symmetric client transport CPU/RSS/FD evidence under the repaired measurement contract;
- keep all comparison language bounded to this self-owned route/time window and one batch; no superiority claim.

If any required pair fails, preserve a typed diagnostic/blocked artifact with no comparative summary.

Always verify cleanup of experiment-owned processes/listeners/temp config/cert/key paths.

### Follow-up D — Reconcile release-matrix status with the accepted evidence

**Dependency:** A/B complete; C complete or honestly blocked with a retained changed-hypothesis artifact.

Repair repository status/evidence drift without erasing historical negatives:

1. `docs/status.md`
   - replace the stale claim that current-lineage D064 warm VPS evidence is absent with the exact positive `25e0daa` controlled-fault evidence and its boundary;
   - add the positive five-minute periodic direct-path sample and keep it explicitly one bounded sample;
   - replace the old local-only HY2 harness-repair note with the `ee37169` truthfulness repair and the actual C outcome;
   - do not promote controlled application fault to natural-WAN degradation.
2. `IMPLEMENTATION_PLAN.md`
   - update bounded release-evidence narrative so D064 and periodic are no longer listed as absent;
   - keep item 3 unchecked while HY2, IPv6/environment, NAT/endpoint-change or other declared matrix requirements remain open;
   - record HY2 comparison as positive only if C produced a complete valid paired set.
3. `ROADMAP.md`
   - keep `UDP degradation / TCP fallback` unchecked if the only positive evidence is the controlled application fault seam; annotate that bounded controlled cross-host failover is positive while natural degradation remains unproven;
   - keep long-lived production wording bounded: the five-minute periodic sample is useful evidence but not a general long-connection stability conclusion;
   - keep IPv6/NAT/endpoint-change/HY2 rows truthful to actual evidence.

Governance flags remain unchanged.

### Follow-up E — Spend remaining VPS opportunity only on a genuinely missing READY row

**Dependency:** C/D complete or C honestly blocked.

Do not repeat already-sufficient generic TCP/UDP baselines, the accepted exact D064 warm row, or the accepted five-minute periodic row.

Audit the remaining release matrix against executable runtime surfaces and choose at most one next row that is already dependency-ready:

1. **NAT / source-endpoint change:** run only if the existing runtime can produce and observe a genuine source endpoint/path change within owned endpoints without production route/firewall modification. If no live runtime seam exists, record `BLOCKED_IMPLEMENTATION` rather than inventing a fake NAT row.
2. **Real-session migration-back / key update / PMTUD:** run only if a current live CLI/runtime path and truthful instrumentation already exist. Current capabilities still describe `key-update` as a fixture; a fixture is not permission to claim real-session key-update evidence.
3. **Current package/operator revalidation:** use only if release-relevant package/lifecycle code changed enough since N5 that old package evidence no longer answers the current release question.

If no additional VPS-only row is truly READY, stop rather than manufacture activity; the next genuine gate is independent release/security review once the bounded matrix is as complete as the available environment permits.

## Fallback

If Primary A shows that a fair same-lifecycle HY2 comparison cannot be constructed from the existing public Nekomusume/HY2 surfaces without a new product/runtime feature:

- do not add a large proxy/tunnel feature merely for benchmarking;
- preserve a precise `BLOCKED_COMPARABILITY` record describing the missing equivalent surface;
- continue Follow-up D and the highest-value genuinely READY VPS row from E;
- carry the comparison blocker into independent release review.

If Follow-up C still shows no HY2 UDP reachability after the connect/bind correction and bounded dual-endpoint observation, preserve the diagnostic result and stop unchanged HY2 retries. Do not turn it into `need WAN authorization` and do not modify production/provider firewall policy under this handoff.

## Completion gates

This package is complete only when the applicable items below are true:

- D064 `53cb859` and periodic `4e1ef88` are preserved as bounded positive VPS evidence with their exact-parent boundaries;
- current `ee37169` stable and nightly fuzz CI success is recognized as the starting gate;
- HY2 remote bind authority and client connect authority are separate and regression-tested with a NAT-shaped differing-address fixture;
- HY2 disposable self-signed TLS is authenticated by a custom CA or exact certificate pin; bare insecure-only config is gone from the owned-lab comparison;
- timed sample lifecycle and client transport resource measurement are symmetric enough that the same metric names mean the same thing for Nekomusume and HY2;
- local full gate and exact repair-head GitHub CI are green before VPS execution;
- the next HY2 VPS attempt is changed-hypothesis and retains either a complete paired result or a packet-direction-classified blocked artifact;
- no partial/failed run produces comparative median/P95 or superiority language;
- repository status/plan/roadmap no longer say D064/periodic evidence is absent;
- no evidence is promoted beyond self-owned bounded scope;
- `RELEASE_CANDIDATE=false`, `PRODUCTION_READY=false`, `FREEZE=false`, and `RELEASED=false` remain unchanged.

## Do not expand into

- third-party targets, scanning, or production network changes;
- provider/firewall/qdisc changes to make HY2 pass;
- runs above standing duration/traffic/concurrency limits;
- bare insecure TLS or disabled authentication for benchmark convenience;
- treating a pre-warmed HY2 tunnel and fresh Nekomusume Session as equivalent performance samples;
- public/general WAN, capacity, security or production claims from these self-owned samples;
- 0-RTT, enabled FEC, striping/aggregation, exotic carriers or other speculative features;
- changing the frozen N9 canonical corpus without a genuine correctness defect.

## Questions requiring maintainer decision

none.
