# Nekomusume ChatGPT Handoff

Checked at: 2026-09-01 09:01 Asia/Shanghai
Repository HEAD: `23e300723ddd948652c7dae4e0409b414ea8a587`
Previous reviewed implementation HEAD: `ca2481017ed704c81ebfb97739cb6db2370ca510`
Previous reviewer handoff commit: `435ab74986932d462e9fdc743842bf7d56d881db`

## What changed

Three substantive coding-agent commits landed after the previous review:

- `12e918a` — **implementation + tests + plan transition.** The failover runner now performs canonical negotiation on initial UDP and TCP resume handshakes, binds the exact negotiation transcript into fresh Noise, adds negotiation binding to `ResumeGuard`, removes the fake constant duplicate metric, renames the old blackhole claim to a controlled application fault/UDP-stop scenario, and checks the implementation-plan negotiation item. Rust CI for this commit completed successfully.
- `1bfb2a4` — **loopback/process evidence only.** It records a non-default 3-record / 37-byte controlled-stop process run and local test evidence. The host could not create an unprivileged network namespace. No WAN claim is made.
- `23e3007` — **real self-owned VPS behavior evidence only; no code change.** It records a ~1-second bounded real-socket controlled-stop run at parent `12e918a`: canonical UDP negotiation + Noise authentication, one UDP data send, controlled client stop, canonical TCP negotiation + authenticated resume, three ordered records on the server, cleanup, and negative negotiation tests. It correctly says this is not natural WAN degradation/PTO evidence.

The negotiation/resume implementation is useful and the controlled VPS run proves more than the previous candidate, but review of the actual runner found a **new release-evidence/security boundary defect** that the current evidence summaries do not expose:

1. On UDP, the server sends `ProcessMessage::DeliveryAck` as raw plaintext bytes with `udp.send_to(&ack, peer)`. The client accepts the next datagram with `recv_from` but does not authenticate, decode, or compare the ACK before emitting `udp_ack_observed`.
2. On resumed TCP, the server again sends `ProcessMessage::DeliveryAck` as an unsealed framed message. The client reads and discards the frame without authenticating or checking its session/stream/offset/length semantics.
3. The client sends an additional plaintext `ProcessMessage::Resume` after the fresh Noise resume handshake; the server merely reads and discards it. The actual authenticated resume claim is already carried in `receive_first_with_resume`, so the extra plaintext control-looking message is currently redundant and misleading.
4. The UDP diagnostic still hard-codes `"bytes":64` for `udp_datagram_sent`, regardless of actual payload/ciphertext length. The start diagnostics also use `payload_bytes` with inconsistent meanings: server = per-record bytes, client = total application bytes.
5. The UDP client retry loop can resend the canonical hello if the selection response is lost, but the server, after sending one selection, immediately treats the next datagram as the Noise first message. A duplicate hello from the same legitimate client can therefore be misclassified as Noise and terminate this one-shot runner. An unrelated peer datagram in that stage can also fail the process. This is not yet robust enough for truthful degraded-path sampling.

These findings do **not** invalidate the parts of `23e3007` that are actually observed: exact canonical negotiation on both carrier handshakes, Noise-authenticated prologue binding, authenticated resume binding/guard, server-side ordered application receive/dedup behavior, and cleanup on the controlled self-owned real-socket path. They **do** invalidate any reading of `udp_ack_observed` or the current TCP ACK frame as authenticated Session-delivery acknowledgement evidence.

`docs/status.md` has also drifted: its CLI row still says failover/resume negotiation is missing even though `12e918a` implemented and tested it. `docs/m3-wan-failover-gate.md` still contains older loopback/public-WAN wording that predates standing authorization and the negotiated runner.

## Review verdict

**NEEDS REPAIR — keep negotiation-path completion, but do not promote the current controlled VPS run as Session-delivery-ACK or release-failover evidence until the control/ACK path is authenticated and checked.**

N9 remains closed/frozen at the corpus-specific level and must not be reopened. The negotiation-path implementation may remain checked because the defect is in post-handshake delivery/control evidence, not in canonical negotiation itself. The release evidence matrix remains unchecked.

The next batch is deliberately thick: repair authenticated delivery-control semantics and UDP retry robustness first, correct the evidence/status drift, rerun the bounded real-socket evidence on the rented VPS, then immediately build the reusable resource/comparison seams that unlock higher-value VPS evidence. Do not spend the rental window on unrelated local polish.

## Evidence boundaries

- `CANONICAL_CORPUS_V1_FROZEN=true` remains correct; global `FREEZE=false`, `RELEASE_CANDIDATE=false`, `PRODUCTION_READY=false`, and `RELEASED=false` remain correct.
- Rust CI for `12e918a` completed successfully. `1bfb2a4` and `23e3007` are evidence/doc commits on top of that implementation.
- `23e3007` is valid evidence of **real self-owned sockets + controlled client UDP-stop + negotiated/authenticated TCP resume + server-side ordered receive**, not natural WAN degradation detection.
- Current `udp_ack_observed` is only “a datagram arrived after the UDP data send”; it is not yet cryptographic or semantic acknowledgement evidence.
- Current TCP ACK frames are likewise unsealed/unvalidated and cannot establish Session delivery confirmation.
- Existing deterministic `SessionRuntime`/failover tests remain separate evidence for logical dedup/uncertain-state semantics; do not silently merge those claims into the current VPS run.
- Standing authorization remains active. Bounded self-owned TCP/UDP listener, capture, failover, soak, resource observation, package rehearsal and HY2 comparison work within its limits needs no per-run approval.
- The VPS is a time-limited rental asset. Once the Primary repair is green, the VPS evidence rerun and rental-window tooling outrank speculative features and cosmetic documentation.
- Natural/ambient UDP degradation -> automatic TCP fallback, sustained long-lived behavior, endpoint/NAT change, and fair HY2 WAN comparison remain unproven.

## Work Package — authenticated delivery-control repair -> immediate VPS evidence -> rental-window tooling

Execute A -> B -> C -> D -> E in dependency order. Do not stop after one small patch if the next item is green and READY. If A exposes a deeper Session/crypto correctness defect, use the fallback instead of collecting more release evidence.

### Primary A — Authenticate and verify failover delivery-control semantics

**Goal**

After an authenticated carrier handshake, every control/delivery claim used by the executable failover runner must itself be authenticated and semantically checked. An arbitrary UDP datagram or plaintext TCP frame must never qualify as a Session delivery acknowledgement. The UDP negotiation retry path must tolerate its own bounded retransmission logic without converting a duplicate hello into a fatal Noise parse.

**Likely files**

- `crates/neko-cli/src/main.rs`;
- `crates/neko-cli/tests/probe.rs` and/or a dedicated failover process-test module;
- `crates/neko-crypto/src/lib.rs` only if a small existing-envelope API seam is genuinely required;
- existing `neko-session` delivery APIs/tests if the client can reuse them instead of inventing a second confirmation ledger;
- `docs/m3-wan-failover-gate.md` / `docs/spec/m3-tcp-failover.md` only to keep the contract aligned after code is green.

#### A1 — Authenticated UDP DeliveryAck

1. Server must protect `ProcessMessage::DeliveryAck` with the established authenticated `SecureSession` before `send_to`. Do not send raw logical control bytes after Noise.
2. Client must authenticate/open the returned record, decode `ProcessMessage::DeliveryAck`, and verify the exact expected `session`, `stream`, `offset`, and `len` before emitting an acknowledgement-success diagnostic or treating that range as confirmed.
3. If the existing client-side Session/delivery ledger can represent assignment -> uncertain -> confirmed, reuse it. Do not create a parallel ad-hoc boolean if the existing model already owns this semantic.
4. A malformed, unauthenticated, replayed, wrong-session, wrong-stream, wrong-offset, or wrong-length UDP datagram must not become `udp_ack_observed`/confirmed evidence. The bounded scenario should fail/timeout or remain uncertain according to the existing contract.
5. Keep UDP packet/path ACK semantics distinct from Session `DeliveryAck`; this is a logical delivery-control message, not evidence of packet-level PTO/loss machinery.

#### A2 — Authenticated TCP DeliveryAck

1. Server must seal each resumed TCP `DeliveryAck` through the fresh Noise transport before framing it.
2. Client must read the framed ciphertext, authenticate/open it, decode the `DeliveryAck`, and match the exact expected logical range before counting the record as acknowledged.
3. Preserve TCP's native transport reliability: do not invent a TCP packet-ACK layer. This is only Session delivery evidence across carrier migration.
4. Add rejection tests for plaintext ACK, tampered ciphertext, wrong Session/range and replay where the current AEAD/replay contract supports the distinction.

#### A3 — Remove or authenticate the redundant post-Noise Resume message

The fresh Noise first message already carries/authenticates the `ResumeBinding` used by `receive_first_with_resume` + `ResumeGuard`.

- If the extra `ProcessMessage::Resume` frame has no independent normative purpose, remove the client write and matching server discard entirely.
- If repository facts show it is intentionally required as a second application-level resume message, it must be sealed, decoded, semantically checked and documented; an ignored plaintext control-looking frame is not acceptable.
- Do not add a second resume protocol merely to preserve old test output.

#### A4 — Make UDP negotiation retry state self-consistent

1. The client already retries the canonical UDP hello. The server must therefore tolerate a duplicate identical hello from the same in-progress peer within bounded pre-auth state: replay the same canonical selection or otherwise handle it without parsing it as Noise.
2. A lost first selection followed by the client's retry must have a deterministic process test and must not terminate the runner.
3. A duplicate/late hello must not reset the selected Session version, authenticated binding, ResumeGuard state, delivery epoch, or path generation.
4. Datagrams from a different peer during an in-progress handshake must be ignored/rejected within bounded pre-auth/resource limits, not allowed to kill the legitimate one-shot experiment process merely by arriving first in the next read.
5. Keep all loops/timeouts/amplification bounded. Do not turn this research runner into an unbounded public UDP service.
6. If the next Noise handshake response also needs a bounded retransmission seam to survive the same retry model, implement the smallest deterministic state/cache needed and test first-response loss. Otherwise document that exact remaining limitation before any degraded-path evidence claim.

#### A5 — Repair diagnostics so fields mean one thing

1. Replace hard-coded UDP `"bytes":64` with the actual observable length (`ciphertext_bytes` or equivalent) and separately report logical application bytes where useful.
2. Stop overloading `payload_bytes`. Prefer explicit names such as `record_payload_bytes` and `application_bytes_total`, consistently on client and server.
3. Emit an ACK-success event only after cryptographic + semantic validation, e.g. `udp_delivery_ack_validated` / `tcp_delivery_ack_validated`; retain lower-level `datagram_received` separately if useful.
4. Keep `controlled_udp_stop` as a scenario/fault-injection label, not a measured natural-blackhole boolean.
5. If duplicate/confirmed/uncertain/lost counters are emitted, source them from real Session state; otherwise omit them.

**Required tests**

At minimum add/adjust tests proving:

- UDP valid encrypted DeliveryAck roundtrip and exact range validation;
- arbitrary plaintext UDP datagram cannot satisfy the ACK oracle;
- authenticated-but-wrong UDP DeliveryAck cannot satisfy it;
- TCP valid encrypted DeliveryAck roundtrip and exact range validation;
- plaintext/wrong/tampered TCP ACK rejection;
- first UDP negotiation selection loss + duplicate client hello recovers within bounds;
- unrelated peer traffic during pre-auth does not mutate/kill the legitimate negotiation state;
- duplicate/late hello cannot reset selected profile/Session resume state;
- controlled UDP stop still resends the intended uncertain logical range and server-side application bytes remain exactly once;
- non-default count/payload/ports/duration produce consistent diagnostic fields with actual ciphertext/application byte lengths;
- no unvalidated `udp_ack_observed` or discarded plaintext `Resume` path remains.

**Validation**

Run targeted crypto/session/CLI/carrier tests, then:

```text
cargo fmt --all -- --check
cargo check --workspace --locked
cargo test --workspace --all-targets --locked --no-fail-fast
cargo clippy --workspace --all-targets --locked -- -D warnings
bash scripts/check.sh
git diff --check
```

Run fuzz smoke if parser/wire decode behavior changes. A runner/control-envelope change alone does not justify a fabricated fuzz claim unless the normal gate requires it.

**Completion definition**

No failover Session delivery/control success depends on unauthenticated or unchecked bytes; the UDP retry path is compatible with its own retransmission behavior; diagnostics reflect actual observations; local gates pass; the implementation remains bounded and does not widen network authorization.

### Follow-up B — Correct evidence/status drift before collecting replacement release evidence

**Dependency:** A green.

1. Add an explicit correction/supersession note to `docs/research/followup-b-20260901.md` and `docs/followup-d-vps-evidence-20260901.md` (or the repository's existing evidence-supersession mechanism):
   - prior negotiation/authenticated-resume/server-receive observations remain valid;
   - prior `udp_ack_observed` / TCP ACK wording did not prove authenticated Session DeliveryAck because the runner had not opened/validated those ACKs;
   - do not delete the old evidence or rewrite it as if the flaw never existed.
2. Update `docs/status.md` CLI/failover wording: generic **and failover/resume** paths now negotiate canonically before fresh Noise after `12e918a`; keep release/public/production claims blocked.
3. Update `docs/m3-wan-failover-gate.md` so it no longer says all WAN execution is prohibited pending review. Standing authorization permits bounded self-owned execution; public/general reachability and production remain blocked.
4. Keep `ROADMAP.md` real-environment “UDP degradation / TCP fallback” unchecked unless a real threshold-driven degradation path is actually measured. Controlled application stop is not that row.
5. Keep `IMPLEMENTATION_PLAN.md` negotiation-path completion checked if A preserves it; bounded release evidence matrix remains unchecked.

### Follow-up C — Immediate post-fix rented-VPS evidence batch

**Dependency:** A/B green, pushed implementation identity known. No new maintainer approval is required for these bounded self-owned runs.

Use a cleanup-safe bounded lab batch. Do not mix CPU-heavy build/fuzz with resource/performance measurements.

#### C1 — Standalone negotiated TCP/UDP real-socket sanity

The repository still lacks a clean post-`ca248101` standalone generic-probe VPS record. Run separate bounded current/current TCP and UDP authenticated/negotiated application exchanges on the self-owned client/VPS and record:

- exact git/binary identity;
- actual parameters and address-family label (without committing unnecessary addresses);
- negotiation selected version;
- authentication/application result;
- negative malformed/unsupported negotiation row if practical;
- cleanup/listener/process state.

This is behavior/reachability evidence, not performance superiority.

#### C2 — Replacement negotiated failover/resume real-socket run

Repeat the controlled endpoint-stop scenario only because A materially changes ACK/control instrumentation.

Require evidence ordering for:

```text
UDP negotiation
-> Noise authentication
-> UDP data
-> authenticated+semantically validated Session DeliveryAck
-> controlled UDP stop
-> TCP negotiation
-> authenticated resume / ResumeGuard
-> uncertain resend / receiver dedup
-> authenticated TCP DeliveryAck(s)
-> exactly-once server application bytes
-> cleanup
```

Classify it explicitly:

```text
real self-owned TCP/UDP sockets + controlled application endpoint stop
!= natural WAN degradation/PTO detection
```

Do not claim automatic blackhole detection unless the actual `FailoverController` threshold is exercised.

#### C3 — Bounded repeated lifecycle sample

If C1/C2 are green, run a small repeated real-socket open/exchange/close sample (for example 8–16 cycles, small payload, well below 10 minutes and 32 sessions) and record success/failure counts plus final listener/process cleanup. This is resilience/leak-detection evidence, not capacity/stress evidence.

If process CPU/RSS/FD/socket sampling already exists, collect it. If not, do not invent numbers; proceed to D.

### Follow-up D — Build the reusable process-resource sampler and use it once on VPS

**Dependency:** A green; may proceed if a C row is environment-blocked.

Create a reusable bounded sampler for experiment processes, preferably under `scripts/bench/` with a small schema/doc/test. It should be usable for both Nekomusume and HY2 and should not require production configuration changes.

Minimum useful output per role/process:

- experiment id / implementation / role;
- git/binary identity supplied by caller;
- start/end/elapsed;
- exit status;
- CPU user/system time when available;
- max RSS or sampled RSS with units/source identified;
- FD count/peak or sampled count with method identified;
- relevant owned-listener/socket count/peak without exposing unrelated connection details;
- application bytes supplied by the workload;
- cleanup status.

Requirements:

- finite sampling interval/duration; no daemon;
- process-scoped, not whole-host claims;
- do not read/log secrets or payloads;
- tolerate process exit races cleanly;
- tests with a harmless local child process and known FDs;
- no claim that RSS/CPU sampling is portable beyond the actual Linux method used.

After the sampler is green, use it on one small non-performance VPS behavior run to prove the evidence pipeline. Do not interpret a single resource sample as capacity.

### Follow-up E — Unlock the HY2 paired VPS comparison without weakening the safe default

**Dependency:** A green; D preferred first because it supplies reusable resource fields.

The current comparison harness is still **loopback-only**: `scripts/bench/compare-hy2.sh` rejects any non-loopback `BENCH_TARGET_HOST`, even though standing authorization now permits bounded self-owned client<->VPS comparison. The pinned HY2 v2.9.3 artifact and comparison contract already exist.

Prepare the next rental-window comparison seam:

1. Keep current loopback mode fail-closed by default.
2. Add a separate explicit self-owned-VPS mode or wrapper rather than silently accepting arbitrary WAN targets. It must require an unmistakable opt-in such as `NEKO_SELF_OWNED_WAN=yes` plus complete server/route/MTU/security/load metadata.
3. In self-owned-WAN mode, enforce standing limits mechanically where possible: per-run timeout <= 600s, total planned application traffic across both implementations/runs <= 256 MiB, concurrency bounded, temporary high ports, cleanup required. Do not treat the script as authorization for third-party targets.
4. Implement/verify an equivalent Nekomusume application benchmark command that consumes exactly `BENCH_PAYLOAD_FILE`, verifies `BENCH_PAYLOAD_SHA256`/length, performs one authenticated negotiated exchange, and prints the required JSON (`application_bytes`, `fd_count`, nullable `wire_bytes`).
5. Prepare an isolated temporary HY2 command/config using the already pinned v2.9.3 binary and throwaway TLS/auth; do not read/reuse the existing production/service credentials or modify the existing HY2 service.
6. Add local contract tests proving wrong payload length/hash, malformed JSON, failed exchange, missing FD, excessive WAN budget, and absent self-owned opt-in fail closed.
7. Do not run or publish paired performance numbers until both adapters have truly equal application semantics and the measurement environment is clean. If they become equal and READY within this batch, a 3–5 run bounded paired self-owned VPS sample is allowed under standing authorization; preserve slower/failed Nekomusume results exactly.

## VPS opportunity

After A is green, the VPS is immediately useful for C. After D/E, it becomes useful for much denser resource and HY2 evidence. Do not defer these in favor of local documentation polish while the rental window is open.

Remaining high-value VPS backlog after this package includes:

- threshold-driven UDP degradation -> TCP fallback (not controlled stop);
- 5–10 minute steady authenticated session;
- idle-with-periodic exchange;
- real-session key-update cycles;
- carrier recovery/migration-back;
- owned endpoint-change if the environment can produce it without production route/firewall/qdisc changes;
- IPv4/IPv6 rows according to actual available address families;
- fair paired HY2 measurements once adapter parity is real.

## Completion gates

This batch is complete only when dependency-satisfied work has moved as far as truthfully possible:

- no plaintext/unvalidated failover DeliveryAck can produce a delivery-success claim;
- redundant plaintext resume control is removed or authenticated+validated for a real reason;
- UDP negotiation retry cannot self-destruct on an expected duplicate hello/response-loss case;
- diagnostic byte/count fields have stable, explicit semantics;
- previous loopback/VPS evidence is corrected without erasing history;
- status/gate docs match the negotiated failover implementation and standing authorization;
- full local gates are green and CI is allowed to attest implementation commits;
- post-fix standalone TCP/UDP and controlled failover VPS behavior evidence is collected if the owned environment remains available;
- a repeated lifecycle sample is collected if C remains green and bounded;
- reusable resource sampling and HY2 comparison seams are advanced as far as dependencies permit;
- release evidence matrix remains unchecked until its defined rows are actually satisfied;
- `RELEASE_CANDIDATE=false`, `PRODUCTION_READY=false`, `FREEZE=false`, `RELEASED=false` remain unchanged.

## Fallback

If authenticated ACK integration exposes a real Session/crypto correctness bug:

1. freeze further release-evidence collection for the affected path;
2. preserve a minimal local reproducer;
3. repair the production correctness/security defect first;
4. run targeted parser/crypto/session/property/fuzz gates as applicable;
5. rerun A/B and only then repeat the VPS behavior scenario under its new code identity.

If the VPS/address family/environment blocks one C/E row, record the exact environment blocker and continue with D or another independent rental-window unlock task. Do not turn one missing namespace/IPv6/NAT seam into a global stop.

## Do not expand into

- reopening the frozen canonical corpus without a concrete corpus defect;
- claiming natural WAN degradation from the controlled endpoint-stop seam;
- using plaintext Session control frames after Noise;
- inventing a TCP packet-ACK layer;
- speculative FEC/0-RTT/striping/exotic carriers;
- third-party targets, scanning, or production firewall/route/DNS/proxy/tunnel/qdisc changes;
- performance superiority claims from one-off or semantically unequal samples;
- exceeding standing 10-minute / 256-MiB / 32-session bounds by splitting one pressure test into many runs.

## Questions requiring maintainer decision

none.
