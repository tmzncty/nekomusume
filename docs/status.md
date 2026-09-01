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
| cli | bounded authenticated probe CLI | candidate | `crates/neko-cli/src/main.rs` | Generic TCP/UDP probes and the controlled failover/resume runner perform canonical bounded version negotiation before fresh Noise, bind the exact transcript before data/resume admission, and authenticate exact-semantic Session `DeliveryAck`; deterministic tests cover first selection and first Noise-response loss without state reset; fixed 40080-40100 ports, 1-1200 bytes, 1-30s duration; the automatic threshold seam proves only bounded authenticated, resume-validated **cold** recovery and does not exercise D064 warm-readiness observations; no natural-WAN degradation, warm fallback, proxy/tunnel, or production listener |
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
