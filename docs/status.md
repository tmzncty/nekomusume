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
| cli | CLI scaffold | implemented | `crates/neko-cli/src/main.rs` | Scaffold only; no client/server/probe transport |
| normative-spec | Session v0 normative entry point | provisional | `docs/specs/nekomusume-session-v0.md` | Provisional and not frozen |
| crypto-handshake | authenticated handshake and AEAD | candidate | `crates/neko-crypto/src/lib.rs` | Bounded Noise IK research implementation with trust/authz, context binding, nonce/replay and synchronized key-phase tests; no security approval or public/production use |
| preauth-admission | runtime pre-auth accounting | candidate | `docs/adr/m1-g0-research-authorization.md` | Bounded research implementation may proceed; candidate values and fail-closed tests remain required |
| live-udp | UDP socket carrier | candidate | `crates/neko-carrier/tests/encrypted_udp_echo.rs` | Authenticated encrypted echo proven on connected 127.0.0.1 ephemeral sockets only; no service/public/production listener |
| unreliable-datagram | bounded authenticated unreliable datagram | candidate | `docs/spec/m2-unreliable-datagram.md` | Authenticated bounded datagrams; no retransmission or Session delivery evidence |
| 0rtt | 0-RTT governance gate | candidate | `docs/spec/m4-0rtt-gate.md` | Explicitly rejected pending replay-safe resumption, persistence/rollback, authorization and review evidence; no early data |
| fec | bounded systematic XOR FEC candidate | candidate | `docs/spec/m2-fec.md` | Single-loss block recovery and bounded failure tests; no evidence-based enablement or performance claim |
| plpmtud | bounded packetization-layer PMTU discovery state | candidate | `docs/spec/m2-plpmtud.md` | Explicit probe ACK/generation binding, bounded search/retry/fallback tests; no live/public probe or ICMP trust |
| reliable-udp | bounded UDP packet recovery state | candidate | `crates/neko-reliable/src/lib.rs` | Deterministic packet/ACK/RTT/loss/PTO/frame-retransmit/Reno/pacing model; no live service or Session-delivery promotion |
| benchmark-fixture | deterministic and privileged isolated benchmark harness | candidate | `scripts/bench/run-netns.sh` | Cleanup-safe netns/veth/netem matrix plus machine-readable summaries; no WAN/HY2 result or performance superiority claim |
| concurrent-multipath | concurrent UDP+TCP and heterogeneous aggregation gate | candidate | `docs/spec/m4-concurrent-multipath-gate.md` | Explicitly disabled pending DSN/reordering/congestion-coupling design and controlled benefit evidence; no striping |
| manager | bounded multi-stream scheduler and Carrier Manager | candidate | `crates/neko-carrier/src/lib.rs` | Fair round-robin, stream/session limits, health score, hysteresis and validated migration-back gate tests; no production manager |
| live-tcp | TCP carrier and resume | candidate | `crates/neko-carrier/tests/tcp_failover.rs` | Encrypted loopback UDP-blackhole recovery over bounded TCP framing with DataId dedup and metrics; no public/production listener |
| reachability | probe / public-network experiments | blocked | `ROADMAP.md` | Explicitly out of scope; no public exposure |
| production | production deployment/readiness | blocked | `docs/spec/m5-release-readiness-gate.md` | Research-only repository; WAN/reachability, independent review and release evidence absent; no production or security approval |

## Status vocabulary

- **implemented** — repository evidence exists and is exercised, without implying protocol/security readiness.
- **candidate** — executable or documented candidate exists, but semantics are not frozen or approved.
- **provisional** — a planning/specification entry point exists and remains explicitly non-normative/non-frozen.
- **absent** — no implementation evidence exists in this repository.
- **blocked** — deliberately prohibited by the current governance boundary until named gates and review pass.

A status change must update this table and the evidence links in the same commit. `implemented` never means “production-ready”, “secure”, “interoperable”, or “publicly deployable”.
