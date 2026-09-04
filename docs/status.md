# Nekomusume implementation status

**Status source:** This table is the single repository status source for the G0 governance baseline. It describes only evidence present at the exact commit carrying this file; it is not a protocol or security approval.

**Baseline:** `candidate/g0-governance-status-repair6` at the administrator authorization amendment; status remains evidence-at-commit, not security approval.

| ID | Area | Status | Evidence | Boundary / next gate |
|---|---|---|---|---|
| G0 | governance and security release gate | candidate | `docs/adr/m1-g0-research-authorization.md` | Research implementation authorized in bounded local/loopback scope; no security approval, protocol freeze, public exposure, or production claim |
| workspace | Rust workspace and crate boundaries | implemented | `Cargo.toml` | Research engineering baseline only |
| wire-codec | deterministic bounded wire codec | candidate | `crates/neko-wire/src/lib.rs` | Candidate format; exhaustive short-prefix/overflow regression and panic-free fuzz-smoke evidence; no frozen interoperability contract |
| session-model | in-memory Session delivery state | candidate | `crates/neko-session/src/lib.rs` | Candidate model; correctness gaps remain |
| carrier-model | Carrier/Path state and loopback carrier slices | candidate | `crates/neko-carrier/src/lib.rs` | Candidate state model plus bounded loopback UDP/TCP/failover evidence; no public or production carrier |
| cli | bounded authenticated probe CLI | candidate | `crates/neko-cli/src/main.rs` | Generic TCP/UDP probes and the controlled failover/resume runner perform canonical bounded version negotiation before fresh Noise, bind the exact transcript before data/resume admission, and authenticate exact-semantic Session `DeliveryAck`; deterministic tests cover first selection and first Noise-response loss without state reset; fixed 40080-40100 ports, 1-1200 bytes, 1-30s duration; the automatic threshold seam has separate bounded **warm** and preserved opt-in **cold** modes: warm TCP is negotiated, Noise-authenticated, resume-bound, resource-admitted and marked warm only after three consecutive, distinct, successfully decrypted exact-tuple peer responses whose admission is derived from bounded live runtime state before controlled UDP failure, with no TCP application data before atomic promotion; accepted exact-`25e0daa` evidence includes a controlled application-level UDP reply-cessation warm fallback (3/3 records, 48 application bytes, two uncertain/replayed, duplicate 0, lost 0, approximately 434 ms failure-decision-to-first-resumed-data), but this is not natural degradation/PTO-blackhole evidence and there is no proxy/tunnel or production listener |
| normative-spec | Session v0 normative entry point | provisional | `docs/specs/nekomusume-session-v0.md` | Provisional and not frozen |
| crypto-handshake | authenticated handshake and AEAD | candidate | `crates/neko-crypto/src/lib.rs` | Bounded Noise IK research implementation with trust/authz, context binding, nonce/replay and synchronized key-phase tests; no security approval or public/production use |
| preauth-admission | runtime pre-auth accounting | candidate | `docs/adr/m1-g0-preauth-resource-budget.md` | Bounded research implementation may proceed; candidate values and fail-closed tests remain required |
| live-udp | UDP socket carrier | candidate | `docs/research/reviewer-primary-a-udp-lifecycle-20260901.md` | Authenticated encrypted loopback plus bounded self-owned cross-host IPv4 UDP exchange and a 14/14 alternating replacement lifecycle sample; generic post-auth data wait uses a bounded application deadline; no public/general reachability or production listener |
| unreliable-datagram | bounded authenticated unreliable datagram | candidate | `docs/spec/m2-unreliable-datagram.md` | Authenticated bounded datagrams; no retransmission or Session delivery evidence |
| 0rtt | 0-RTT governance gate | candidate | `docs/spec/m4-0rtt-gate.md` | Explicitly rejected pending replay-safe resumption, persistence/rollback, authorization and review evidence; no early data |
| fec | bounded systematic XOR FEC candidate | candidate | `docs/spec/m2-fec.md` | Single-loss block recovery and bounded failure tests; no evidence-based enablement or performance claim |
| plpmtud | bounded packetization-layer PMTU discovery state | candidate | `docs/spec/m2-plpmtud.md` | Explicit probe ACK/generation binding, bounded search/retry/fallback tests; no live/public probe or ICMP trust |
| reliable-udp | bounded UDP packet recovery state | candidate | `crates/neko-reliable/src/lib.rs` | Deterministic packet/ACK/RTT/loss/PTO/frame-retransmit/Reno/pacing model; no live service or Session-delivery promotion |
| benchmark-fixture | deterministic and privileged isolated benchmark harness | candidate | `scripts/bench/run-netns.sh` | Cleanup-safe netns/veth/netem matrix plus machine-readable summaries; no WAN/HY2 result or performance superiority claim |
| concurrent-multipath | concurrent UDP+TCP and heterogeneous aggregation gate | candidate | `docs/spec/m4-concurrent-multipath-gate.md` | Explicitly disabled pending DSN/reordering/congestion-coupling design and controlled benefit evidence; no striping |
| manager | bounded multi-stream scheduler and Carrier Manager | candidate | `crates/neko-carrier/src/lib.rs` | Fair round-robin, stream/session limits, health score, hysteresis and validated migration-back gate tests; no production manager |
| live-tcp | TCP carrier and resume | candidate | `docs/research/reviewer-followup-b1-b4-vps-20260901.md` | Encrypted loopback plus bounded self-owned cross-host negotiated/authenticated TCP, exact-semantic authenticated DeliveryAck controlled-stop resume, and replacement alternating lifecycle evidence exist; no natural/automatic threshold-driven degradation or production listener |
| reachability | bounded probe / public-network experiments | blocked | `docs/vps-experiment-2026-08-29.md` | Standing authorization permits bounded self-owned VPS TCP/UDP execution; public/general reachability evidence, required environments, sustained release evidence, third-party targets, security/release approval and production exposure remain blocked; no public listener |
| production | production deployment/readiness | blocked | `docs/spec/m5-release-readiness-gate.md` | Research-only repository; WAN/reachability, independent review and release evidence absent; no production or security approval |


> **HY2 harness state:** deterministic local regression coverage retains typed failure evidence and verifies ordered cleanup. The earlier exact-`f1cb9af` preflight history remains retained. At exact `bc38d06`, exactly one changed-hypothesis invocation passed explicit SSH preflight and prepared its payload, then ended `BLOCKED_HARNESS` during setup because `run_client` expanded `impl` before assignment under `set -u` at line 186. It produced zero samples and no comparison. The validator-valid result artifact has SHA-256 `596ad4b73058143db1918613dd970e44e8e6bf3a1b89602ac0012f911b6d2653`. Its recorded cleanup failed (`remote_listeners_remaining=1`, `remote_process_groups_reaped=false`, remote temp-path removal unknown); independent manual post-run cleanup subsequently verified no experiment ports, processes, or temporary paths remained. No performance conclusion changed. At exact `3d54585`, exactly one later invocation after green exact-head CI prepared the payload and retained a valid two-record prefix: `nekomusume-1` succeeded and `hy2-1` ended `client_exit`; overall status is `BLOCKED_HARNESS` at `hy2-1-failed`. There are no complete pairs or comparative summary. The validator-valid result SHA-256 is `dc7d4a0887ebc5617dbc34b5146563af7178445ea2ba05d30da05276f4558602` under `artifacts/hy2-owned-lab/3d54585-hy2-client-exit/`. Automatic cleanup failed solely because `remote_process_groups_reaped=false`; listeners were zero, remote temporary-path removal and local cleanup succeeded. Separate later serialized double-end postchecks found no experiment ports, processes, or temporary paths, without rewriting the artifact.

> **Exact `9fd2411` repeated warm-failover attempt:** exactly one cross-host live invocation produced a schema-valid typed negative after 9,758 ms: launcher exit 1, 0/6 completed cycles, and cycle 1 `invalid_cycle_evidence` because the collector returned nonzero without a valid stdout row. The retained minimal artifact is `artifacts/repeated-warm-failover/9fd2411-typed-negative/result.json` (SHA-256 `4744a3b407537f0e442668f1c05f05e5218ad74bfc410e098a8fd89a8bca9f59`). The failure is bounded to orchestration/evidence collection; the discarded collector stderr prevents a truthful deeper classification. Exact binary identity matched at both endpoint descriptors and final residue was zero, but there is no completed live cycle row and therefore no new WAN failover, runtime-correctness, or reachability claim. The separate six-row `dry.json` is synthetic preflight, lacks `endpoint_provenance`, and is not live evidence. No retry occurred.


## Status vocabulary

- **implemented** — repository evidence exists and is exercised, without implying protocol/security readiness.
- **candidate** — executable or documented candidate exists, but semantics are not frozen or approved.
- **provisional** — a planning/specification entry point exists and remains explicitly non-normative/non-frozen.
- **absent** — no implementation evidence exists in this repository.
- **blocked** — deliberately prohibited by the current governance boundary until named gates and review pass.

A status change must update this table and the evidence links in the same commit. `implemented` never means “production-ready”, “secure”, “interoperable”, or “publicly deployable”.

## N0 governance decision vector

These flags are independent governance facts for this exact commit. They must not
be collapsed into one readiness bit:

- `IMPLEMENTATION_COMPLETE=true` — The bounded research implementation slice recorded above is complete for this baseline.
- `RELEASE_CANDIDATE=false` — RC status is not granted; the release-readiness and independent-review gates remain incomplete.
- `PRODUCTION_READY=false` — Production readiness is not granted; production remains blocked.
- `FREEZE=false` — No protocol or release freeze is declared.
- `RELEASED=false` — No release is declared.
- `CANONICAL_CORPUS_V1_FROZEN=true` — N9 freezes exactly the 42-vector, 10-domain corpus identity in `fixtures/canonical-vectors.v1.json`; this does not freeze Noise, ciphertext, carrier packetization, failover/resume, or the global protocol.

`PRODUCTION_AUTHORIZATION` is **not** an RC prerequisite. It is a separate,
later production gate: its absence does not explain or prevent the RC decision;
the RC remains `false` because the independent release-readiness criteria are
incomplete. Conversely, `IMPLEMENTATION_COMPLETE=true` does not imply RC,
release, freeze, reachability, security approval, or production readiness.
The `reachability` and `production` rows above remain `blocked`.

### D064 local readiness repair (2026-09-02)

The bounded failover responder requires exactly three authenticated, ordered,
current-tuple readiness requests with live `admitted=true` before it emits
`tcp_resource_admitted` or reads application data. After negotiation, Noise
authentication and resume validation, challenge 1 starts a three-second whole
sequence budget; each request/response read and write is independently capped at
one second and also by the remaining sequence and experiment budgets. Process
tests prove three 400 ms responses succeed, an individual response above one
second and cumulative work above three seconds fail closed, and all prior wrong
tuple, unadmitted, replay, tamper and incomplete-proof failures remain before
admission, warm transition, promotion or data. Exactly three valid observations
emit one warm transition. This is local implementation evidence only; it does
not add current-head VPS evidence or change any release, freeze, production or
canonical-corpus flag.

### Periodic setup deadline separation (2026-09-02)

`periodic-client` now exposes `--setup-timeout-ms` with a 5000 ms default and a
10000 ms maximum. Its setup budget begins before TCP connect and covers connect,
canonical negotiation and Noise authentication; `--ack-timeout-ms` remains only
the per-record authenticated `DeliveryAck` deadline. The server applies the same
finite setup limit from accept, additionally bounded by workload duration.
Process tests prove setup slower than the ACK timeout can succeed, delayed ACKs
still fail independently, setup expiry admits zero records, and malformed setup
never authenticates. Reconnect, wire and Session delivery semantics are
unchanged.

### Reviewer 3978f3f Follow-up A-C evidence reconciliation (2026-09-02)

The reviewed tree remains green under its local and CI gates, but the three
self-owned-lab follow-ups add blockers rather than positive release evidence:

- **A — no current-head warm result.** The changed-path D064 run reached two
  authenticated admitted readiness responses, then challenge 3 failed closed
  (`readiness response timeout or malformed frame` / `bad readiness frame`).
  The redacted raw JSONL is retained at
  `artifacts/reviewer-3978f3f-primary-a/`; provenance, hashes and cleanup are in
  `docs/research/reviewer-3978f3f-primary-a-d064-vps-20260901.md`. Older cold,
  periodic and negative rows remain historical exact-commit evidence.
- **B — no five-minute sample.** The server authenticated, but the client failed
  to receive the handshake response before any periodic application record;
  therefore application bytes were zero and no confirmation-latency sample
  exists. See
  `docs/research/reviewer-3978f3f-followup-b-periodic-vps-20260902.md`.
- **C — adapter present, comparison absent.** The owned-lab fair-pair adapter and
  exact-payload seam are implemented and pass the full repository gate. The
  owned-lab adapter now requires and remotely validates a distinct dedicated
  HY2 bind address and cannot generate a wildcard listener. The temporary HY2
  QUIC/UDP path timed out before its forwarding listener became
  ready, so there are no paired samples, comparative statistics or superiority
  claim. See
  `docs/research/reviewer-3978f3f-followup-c-hy2-owned-lab-20260902.md` and
  `docs/research/reviewer-followup-c-equal-application-prerequisite-20260901.md`.

All three attempts record successful cleanup. These historical negative results remain valid.

### Accepted 25e0daa and f1cb9af Follow-up E reconciliation (2026-09-03)

- Exact `25e0daa` adds one controlled application-level UDP reply-cessation
  warm-fallback result: 3/3 records and 48 application bytes completed; two
  records were uncertain/replayed, duplicate and lost counts were both 0, and
  failure-decision-to-first-resumed-data was approximately 434 ms. This is not
  natural degradation or PTO-blackhole evidence.
- Exact `25e0daa` also adds one approximately five-minute periodic direct-path
  sample: 60 x 32-byte records, 60/60 confirmed, with no missing, duplicate, or
  conflicting record. It is one bounded sample, not a reliability rate.
- At exact `f1cb9af`, the HY2 run is `BLOCKED_HARNESS` at preflight SSH
  authentication. It produced no payload, samples, paired statistics, or
  comparison. The one-invocation control was violated: the harness was invoked
  twice, and both attempts ended identically with preflight RC2 because the SSH
  preflight user contract was not explicit and the configured alias resolved
  to `tmzn`; no root assumption is valid. An unchanged retry is prohibited, but
  one retry with a substantive changed hypothesis is allowed. No HY2 comparison
  may be claimed.
- At exact `bc38d06`, exactly one substantive changed-hypothesis harness
  invocation passed explicit SSH preflight and prepared the 1200-byte payload,
  then ended `BLOCKED_HARNESS` during setup because `run_client` expanded
  `impl` before assignment under `set -u` at line 186. It produced zero samples,
  paired statistics, or comparison. The validator-valid result artifact SHA-256
  is `596ad4b73058143db1918613dd970e44e8e6bf3a1b89602ac0012f911b6d2653`.
  Artifact-recorded cleanup failed with one remote listener remaining, remote
  process groups not reaped, and remote temp-path removal unknown. Independent
  manual post-run cleanup subsequently verified no experiment ports, processes,
  or temporary paths remained; that later observation does not rewrite the
  artifact cleanup fields.
- At exact `3d54585`, exactly one invocation after green exact-head CI prepared
  the payload and retained a valid two-record prefix: `nekomusume-1` succeeded,
  then `hy2-1` failed with `client_exit`; the overall result is `BLOCKED_HARNESS`
  at `hy2-1-failed`. There are no complete pairs, paired statistics, or
  comparative summary. The tracked validator-valid result SHA-256 is
  `dc7d4a0887ebc5617dbc34b5146563af7178445ea2ba05d30da05276f4558602`.
  Automatic cleanup failed solely because `remote_process_groups_reaped=false`;
  listeners were zero, remote temporary-path removal and local cleanup succeeded.
  Separate later serialized double-end postchecks found no experiment ports,
  processes, or temporary paths; they do not rewrite the artifact cleanup fields.

IPv6 remains environment-blocked.
The bounded release-evidence matrix stays open; `RELEASE_CANDIDATE=false`,
`FREEZE=false`, `PRODUCTION_READY=false`, and `RELEASED=false`. The frozen N9
canonical corpus is untouched, and none of these self-owned-path results is
public/general WAN evidence.

### Exact-07545f0 Follow-up D evidence boundary (2026-09-03)

Commit `07545f049790a088bfa655aff4995ab9d6e8fc29` is the provenance-binding
repair. GitHub Actions run `33755759414` completed successfully for that exact
head; both stable and fuzz jobs were green. The run is repository CI evidence,
not live-network, security-review, or release evidence.

### Final periodic current-line boundary

The exact-`60cd40d` `start_timeout` remains a separate historical pre-application negative. Exact `85346ce` retains the sole changed-hypothesis periodic follow-up from exact `00ac2c1`: `ssh_transport_exit` 255, no readiness, no client launch, no application traffic or metrics, and verified-zero local/remote cleanup. Per the additive R-009 erratum, the immutable result's legacy `protocol_entered=true` proves only local capture attachment; remote executor acceptance was not proved. The lane is `BLOCKED_ORCHESTRATION_CURRENT_LINE_PERIODIC`, not Session runtime or reliability evidence.

The retained release-evidence rows are reconciled as follows:

- The accepted exact-`25e0daa` D064 controlled warm fallback and its single
  approximately five-minute periodic sample remain the only positive rows here;
  both are **ALREADY_SUFFICIENT** for their bounded single-sample questions, not
  reliability rates or natural-degradation evidence. Historical negative rows
  remain immutable.
- Exact `1bf848d` made one repeated-warm-failover outer invocation. It retained
  zero cycle rows and stopped at cycle 1 with typed
  `invalid_cycle_evidence` / `collector returned nonzero without a valid row`.
  Its batch JSON SHA-256 is
  `5ca57b92571690f11157d636d03df554935ced3eda23e312c534020c1ddcf13e`.
  This is a collector/orchestration negative, not a runtime failover failure.
- Exact `07545f0` made exactly one later outer invocation, but exited 127 at the
  shell argument boundary before the Python batch runner or any cycle collector
  ran. Stdout was empty; sanitized stderr was
  `env: ‘failover-server,’: No such file or directory` with SHA-256
  `a81c2170e75f57c36490be59a43f0ac5cb342f8b70341d5efb9a6814564bdeaa`.
  Completed cycles were zero and no batch artifact or cycle row exists. In
  particular, no `followup-b-07545f0-20260903T125159Z` artifact or
  `1bf896b...` artifact hash is accepted. This is orchestration-only evidence;
  it says nothing about runtime failover. Separate post-attempt observations
  verified zero experiment processes/listeners, absent deployment/identity temp
  paths, and a clean worktree.
- Exact `c4786dc8570dc176fc47251f955979dff7de4b58` made one periodic
  orchestrator invocation and zero `periodic-client` invocations. It stopped
  before retained server-readiness or application evidence: zero attempted and
  confirmed records/bytes. The redacted result JSON at external evidence id
  `c4786dc-followup-c-periodic-480s-once` has SHA-256
  `bcd8f5582a221b4192fd561301f1e5799996d1a51de673dc05f184d4ba044d71`.
  This is a typed pre-application orchestration negative, not a Session runtime
  failure. Separate direct post-exit observations verified zero local/remote
  experiment processes, listeners, and runtime temp paths.

- NAT / source-endpoint change — `BLOCKED_IMPLEMENTATION`: capability remains explicitly unsupported; no authenticated live rebinding/endpoint-change runner exists.
- migration-back — `BLOCKED_IMPLEMENTATION`: deterministic manager gating exists, but no live socket path executes validated migration back to UDP.
- live key update — `BLOCKED_IMPLEMENTATION`: key update remains a local fixture/state transition; live authenticated Session commands do not expose a key-update cycle.
- live PMTUD — `BLOCKED_IMPLEMENTATION`: PLPMTUD is bounded state-model evidence only and is not integrated into a live carrier probe/ACK path.
- IPv6 — `BLOCKED_ENVIRONMENT`: no real owned IPv6 endpoint/path is currently available; historical probes do not supply the missing current environment.
- HY2 fair pair — `BLOCKED_DIAGNOSTICS`: the latest retained prefix ends at `hy2-1` `client_exit` without enough retained diagnostics to distinguish harness, endpoint, or HY2 runtime cause; another unchanged live retry is not justified.
- repeated warm failover — `BLOCKED_ORCHESTRATION_CURRENT_LINE`: the exact-`07545f0` command boundary must first be corrected and locally dry-run through runner entry; neither retained orchestration negative is a runtime row.

There is no `READY_LIVE` row. The smallest unlock seam is local and
non-networked: correct and verify the exact-`07545f0` repeated-failover command
array/argument boundary so one outer invocation demonstrably enters the Python
batch runner. The bounded release matrix therefore remains open, with
`RELEASE_CANDIDATE=false`, `FREEZE=false`, `PRODUCTION_READY=false`, and
`RELEASED=false`.

### 2026-09-04 exact-a117086 corrected structured six-cycle outcome

The sole authorized live outer invocation at exact `a117086fa69553a36021137900b6052050624a8b` is retained at `artifacts/repeated-warm-failover/a117086-typed-negative/`. It exited 1 after 2,303 ms with 0/6 completed cycles and a typed batch-level `invalid_cycle_evidence` at cycle 1 (`collector returned nonzero without a valid row`), result SHA-256 `71ab1cab9828f72f1b1e044bbbef5d39178ac0af9ea1a022916e872e3b5c63b6`. Invocation count is exactly 1; no retry. Synthetic preflight is separate and not live evidence. No full cycle, valid prefix, runtime failover conclusion, WAN/reachability claim, or unsupported deeper root cause is asserted. Per-cycle endpoint provenance, resources, accounting, timing, and exits are not collected because no row exists; remote resources remain `not_collected_remote`. Separate post-run cleanup process/listener/temporary-path postchecks were zero; these are not artifact-carried row fields. Historical `c156868` negative remains preserved.

### Exact-`c6ab8fd` Follow-up B evidence boundary (2026-09-04)

Exactly one live repeated-warm-failover invocation was retained at
`artifacts/repeated-warm-failover/c6ab8fd-typed-negative/`; retry count was 0.
The schema-valid result is a typed negative: 0/6 cycles, cycle 1
`invalid_cycle_evidence`, collector exit 2, and no valid stdout row. The private
0600 diagnostic's sanitized literal summary is `live failover collector: missing
JSON event: start`; it remains outside Git. This proves only the immediate
orchestration/evidence-collection seam. Whether the missing event reflects
remote server event absence, output framing, or early exit is indeterminate;
no deeper cause is claimed. With zero cycle rows, endpoint provenance,
resources, accounting, timing, exits, and cleanup fields cannot be positively
proved. External zero cleanup is a separate post-run observation. No runtime
failover, WAN/public reachability, or production claim follows. Prior artifacts
remain preserved and `docs/CHATGPT_HANDOFF.md` was not modified.


### HY2 current-line closure (2026-09-04)

The exact `61a6490` post-B C follow-up consumed exactly one new outer wrapper invocation and stopped at local port-range preflight (exit 2). It produced zero VPS deployments, samples, result artifact, comparative statistics, or runtime evidence. The historical bed2940 invocation remains separate (outer=1, local `NEKO_BIN` non-executable); both VPS counts are zero, cleanup residue is zero, and the existing Hysteria service was not touched. The current HY2 line is `BLOCKED_ORCHESTRATION_CURRENT_LINE_HY2`; no historical artifact is rewritten and governance/release flags remain unchanged.


### Release-evidence closure index (2026-09-04)

At the reviewed tree, every Era-4 ledger row has one closed classification in `docs/era4-ledger-2026-08-30.json`: `ALREADY_SUFFICIENT_FOR_BOUNDED_QUESTION`, `OPEN_READY`, `BLOCKED_DEPENDENCY`, `BLOCKED_IMPLEMENTATION`, `BLOCKED_ENVIRONMENT`, `BLOCKED_ORCHESTRATION_CURRENT_LINE`, or `GOVERNANCE_GATE`.

- `ALREADY_SUFFICIENT_FOR_BOUNDED_QUESTION`: A, G, H, I, J (their declared bounded questions already have adequate evidence; do not re-run merely because more testing is possible).
- `OPEN_READY`: B, C, D, E, F, L, M, T. Each carries a specific unresolved question, `evidence_needed`, `next_action`, dependency statement, and local execution scope; this is an opportunity classification, not authorization or a release claim.
- `BLOCKED_DEPENDENCY`: N depends on implementation-blocked K; O depends on N. Privileged environment availability alone cannot make either dependency-ready, and no VPS row is currently ready for execution.
- `BLOCKED_IMPLEMENTATION`: K (migration-back), R (PLPMTUD integration). NAT/source-endpoint change, live key update and live PMTUD remain implementation-blocked.
- Security promotion also remains blocked by RSEC-001 in `docs/reviews/resource-abuse-evidence-2026-09-04.md`: per-budget-object anti-amplification exists, but process-global/per-source pre-auth concurrency, memory, CPU, rate-window, queue and lifecycle accounting required by `SECURITY.md`/D019 is not implemented.
- `BLOCKED_ENVIRONMENT`: IPv6 (no real owned IPv6 path).
- `BLOCKED_ORCHESTRATION_CURRENT_LINE`: periodic, repeated warm failover, and HY2; the exact `61a6490` HY2 follow-up consumed one outer invocation and stopped at local port-range preflight (exit 2), with no VPS deployment, samples, result, metrics, or runtime evidence.
- `GOVERNANCE_GATE`: P, S, U, V, W. Release item 3 remains unchecked; natural loss remains unchecked; governance flags remain false/closed as previously recorded.

The historical bed2940 outer invocation (local `NEKO_BIN` preflight) and the exact `61a6490` C outer invocation (local port-range preflight) remain separate, with zero VPS deployments and zero cleanup residue. No historical artifact is rewritten, and no private endpoint, credential, or topology is recorded here.
