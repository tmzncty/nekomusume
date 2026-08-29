# 猫娘 Design Decision Log

这个文件记录讨论中已经形成的设计方向，避免后续 Agent 把“曾经讨论过”误认为“已经定稿”，也避免真正做出的决定只存在于聊天记录里。

状态定义：

- **Accepted**：当前按此推进，除非新实验推翻。
- **Provisional**：暂时采用，仍需要实现/测试确认。
- **Research**：只进入研究列表，不承诺实现。
- **Superseded**：被后续决定替代，但保留历史。

---

## 2026-08-26 — D001：Rust 起手式

**Status: Accepted**

第一实现语言选择 Rust。

理由不是“Rust 一定最快”，而是本项目本身是传输栈研究：wire codec、状态机、buffer ownership、fuzz、边界安全、低层 socket 行为等都值得直接暴露出来研究。

当前没有要求本地 Windows 必须配置 Rust。主要开发/实验方向转为 Linux server；VPS 提供真实 WAN peer；QEMU 与 Linux 网络工具提供可控实验环境。

尚未决定：async runtime、TLS/Noise 库、buffer model、workspace crate 拆分。

---

## 2026-08-26 — D002：猫娘从 UDP protocol 改为 carrier-agnostic Session transport

**Status: Accepted**

早期基线把项目定义为“UDP + AEAD + 自定义可靠传输”。这个方向仍然是第一实现切片，但不再定义猫娘本身。

新的核心抽象：

```text
Application / Tunnel
        -> Nekomusume Session
        -> Carrier Manager
        -> UDP / TCP / experimental carriers
```

TCP、UDP 等是底层 Carrier。逻辑 Session 不绑定某个具体 transport connection。

`docs/design-handoff.md` 保留为历史交接基线；最新架构方向见 `docs/carrier-architecture.md`。

---

## 2026-08-26 — D003：核心问题选择“异构 Carrier 生存性”（B）

**Status: Accepted**

项目不是主要为了同时提供 reliable stream + unreliable datagram（问题 A）。

当前核心问题是：

> 当网络环境让 UDP、TCP 或其他通信原语中的一部分不可用/严重退化时，能否让同一个逻辑隧道在异构 Carrier 之间 failover / migrate（问题 B）？

这也是 TCP + UDP 共存的主要理由。

---

## 2026-08-26 — D004：第一阶段只做 failover，不做 TCP/UDP 同时条带化

**Status: Accepted**

早期目标：

```text
UDP = primary
TCP = fallback
```

先证明 Session 可以跨 Carrier 恢复，再研究 concurrent heterogeneous multipath。

禁止在早期简单按 packet 轮流分发到 TCP/UDP；不同排队、重传和有序语义可能制造跨协议 Head-of-Line blocking。

---

## 2026-08-26 — D005：Path feedback 与 Session delivery state 分层

**Status: Provisional**

UDP Carrier 可以拥有 packet-level ACK / RTT / loss / cwnd；TCP Carrier 使用 TCP 自身可靠性，不重复实现 TCP packet ACK。

但 Session 仍需维护跨 Carrier 的逻辑交付状态，例如 `stream_id + offset ranges`，用于 failover 后判断已确认/不确定数据并去重。

具体 ACK 编码尚未冻结，进入 M0 研究。

---

## 2026-08-26 — D006：实验性 Carrier 列表

**Status: Research**

当前候选：

- Tier 0：UDP、TCP（必须实现）
- Tier 1：ICMP Echo、Raw IP experimental protocol（必须做 reachability 实验，不承诺成为常用路径）
- Tier 2：SCTP、DCCP（重点研究/可能实验实现）
- Tier 3：UDP-Lite、GRE、IP-in-IP、ESP、HTTP/WebSocket/MASQUE 等（协议考古与对照）

Raw IP 实验号必须显式配置，默认不开启；不得把实验值当成互联网正式协议号。

---

## 2026-08-26 — D007：Reachability Matrix 作为正式实验产物

**Status: Accepted**

不凭经验猜测“公网一般会不会放行”。对 Linux server、VPS、QEMU/虚拟拓扑以及后续其他端点运行可重复 reachability probe，记录协议、IPv4/IPv6、端口、payload、RTT、loss、双向持续可用性、权限和网络环境元数据。

项目的人类可读探测结果固定为：

```text
如果通：喵~！
如果不通：喵呜呜呜呜…
```

同时必须存在机器可读状态与正确 exit code。

---

## 2026-08-26 — D008：真实 WAN 与可控网络实验并行

**Status: Accepted**

- Linux server + VPS：真实互联网验证。
- QEMU / netns / veth / tc netem：可重复故障注入与网络模型。

真实 WAN 用来判断“现实是否买账”；可控环境用来解释“为什么”。二者不能互相替代。

---

## 未决事项

- M0 crate/workspace 结构。
- v0 magic/version 与 canonical integer encoding。
- Session delivery ACK 具体格式。
- UDP Carrier 的第一拥塞控制基线。
- UDP/TCP migration handshake 与 anti-replay 细节。
- Carrier scoring 与 hysteresis。
- ICMP / Raw-IP probe 的 payload 与安全边界。
- 第一批真实测试节点和矩阵规模。

---

## 2026-08-26 — Research integration index

**Status: Accepted as documentation governance; implementation decisions remain gated**

The standards, security and transport research are indexed by [`docs/m0-spec-plan.md`](m0-spec-plan.md). The versioned document [`docs/specs/nekomusume-session-v0.md`](specs/nekomusume-session-v0.md) is the sole normative-source entry point; it remains provisional until its gates are reviewed. Research files are non-normative; this log records status and links rather than duplicating wire rules. `docs/design-handoff.md` is historical/non-normative.

## 2026-08-26 — D009：M0 先落地边界，不落地传输

**Status: Accepted — historical M0 scope; superseded for UDP by D012**

M0 established only the virtual Cargo workspace and the `neko-wire`, `neko-session`, `neko-carrier`, and `neko-cli` crate boundaries. Within the M0 slice, UDP/TCP, cryptography, wire codec, Session state machine, and failover were not implemented; those capabilities required their own specifications and safety gates. The later M1-S1 loopback UDP slice is documented by D012 and does not retroactively change the M0 scope recorded here.

## 2026-08-26 — D010：采用 MIT OR Apache-2.0

**Status: Accepted — administrator decision**

管理员裁决本项目采用双许可证表达式 `MIT OR Apache-2.0`，使用者可任选其一。根目录 `LICENSE-MIT` 与 `LICENSE-APACHE` 提供完整文本；workspace 与全部 crate manifest 统一声明该 SPDX 表达式。D010 仅决定项目许可证表达与 SPDX 标注，不是任何依赖的批准、选型或安全审查；依赖许可证与引入仍须独立门禁。cargo-deny 不在本次变更中启用，许可证策略另行处理。

## 2026-08-26 — D011：M1-S0 bounded MemoryPair candidate

**Status: Candidate — implementation slice only**

`neko-carrier` now exposes a synchronous, non-blocking opaque-byte `Carrier` contract and a bounded in-memory `MemoryPair`. It preserves message boundaries, uses independent FIFO queues, copies input on send, applies checked byte accounting, and defines idempotent close with queued-data drain and explicit closed/peer-closed errors.

This candidate opens no socket and has no runtime, routing, tunnel, firewall, service, UDP, or cryptographic behavior. It does not create `SessionDelivery` or `PathValidated` evidence and does not call `confirm_received`. M1, UDP, crypto, and production transport remain incomplete and unclaimed.
## 2026-08-26 — D012：M1-S1 loopback UDP carrier

**Status: Accepted — minimal safety slice**

M1-S1 adds `neko-carrier::UdpLoopbackPair` and `UdpLoopbackEndpoint` using only `std::net::UdpSocket`. Construction binds both endpoints to `127.0.0.1:0`, connects the two ephemeral sockets to each other, and sets them nonblocking. The implementation keeps UDP errors in an independent `UdpError` type rather than migrating the existing `CarrierError`; this preserves the M1-S0 API and avoids cross-layer semantic promotion.

Acceptance scope is limited to bidirectional opaque datagrams, preserved message boundaries, empty datagrams, oversize rejection without enqueue, nonblocking `WouldBlock`, OS errors, local idempotent close, and socket resource release by normal ownership. No worker, runtime dependency, fixed port, non-loopback bind, routing, tunnel, firewall, service, netns, WireGuard, TUN, OSPF, or policy-route operation is introduced. The slice makes no claim of `PeerClosed`, `SessionDelivery`, `PathValidated`, ACK, failover, reliability, or production reachability.

The M1-S1 gate passed formatting, workspace check/test/clippy, script checks, and `git diff --check`; the environment snapshot observed no network mutation. Existing fuzz-smoke evidence is inherited from the reviewed baseline. It was intentionally not rerun during documentation closure because the script creates untracked `fuzz/corpus/decode/*` artifacts; those artifacts were removed after the prior run and the repository was returned clean.

## 2026-08-26 — D013：M1-S2 bounded candidate record framing seam

**Status: Candidate gate — implementation/test slice only; not a frozen specification**

M1-S2 connects the existing `neko-wire` candidate `encode`/`decode` to `MemoryPair` messages and `UdpLoopbackPair` datagrams in integration tests. Each carrier unit contains exactly one complete record; boundaries and FIFO order are preserved. Empty payloads, the `MAX_PAYLOAD_LEN` boundary, oversized records, nonblocking `WouldBlock`, and malformed magic/version/type/flags/length, truncation, and trailing bytes are covered.

Malformed decoding remains local to the framing seam: tests keep rejection before any evidence-producing API and confirm the carrier remains independently usable. No production session code, crypto/auth/key material, ACK machinery, retransmission, congestion control, network configuration, or new runtime/network dependency is introduced. This candidate gate does not freeze the provisional wire format.

## 2026-08-26 — D014：M1-S3a Crypto/Handshake security gate stage 1

**Status: Candidate security gate — documentation and synthetic contract only**

M1-S3a adopts the Noise Framework as the direction for a future authenticated handshake, but does not select a concrete pattern or library/version, nor approve license compatibility for any future cryptographic dependency; those choices require later G0/G2 approval. The accepted project license decision in D010, `MIT OR Apache-2.0`, remains unchanged. Stage 1 is dependency-free and changes no Cargo manifest, production `src/` code, wire/carrier/session/CLI code, runtime, or network service.

The stage establishes reviewable future invariants for identity/authorization, transcript and AAD binding, independent direction/epoch/key-phase state, nonce uniqueness and checked overflow refusal, replay/duplicate/old-epoch rejection, disabled 0-RTT, bounded pre-auth resources and anti-amplification, and secret-safe logging/error boundaries. The synthetic contract tests only state and rejection semantics: authentication failure, tampering, wrong direction/epoch/key phase, AAD mismatch, truncation, duplicate, oversize, and nonce-counter overflow. It must never fabricate authentication or encryption success and must produce no `SessionDelivery`, `PathValidated`, or `ACK` evidence.

This is a candidate gate, not a security audit or protocol freeze. Real Noise/TLS/AEAD/KDF, key/nonce generation, handshake, identity loading, replay window, path authentication, UDP encryption, and runtime/network implementation remain prohibited until the later approved gates.

## 2026-08-26 — D015：Noise IK 25519/ChaChaPoly/SHA256 G0 candidate contract

**Status: Candidate only — not frozen; not implementation approval**

`Noise_IK_25519_ChaChaPoly_SHA256` is recorded as a G0 candidate only. This
entry authorizes no Cargo, `src/`, dependency, test, runtime, or network change.
The complete candidate contract and its unresolved blockers are in
[`docs/adr/m1-g0-noise-ik-candidate.md`](adr/m1-g0-noise-ik-candidate.md).

The blockers explicitly include trust-store versioning, signatures, rotation,
revocation and rollback; authentication not being authorization; identity
leakage and uniform failure behavior; strict prologue/AAD domain separation;
complete header/wire mapping; nonce uniqueness across restart, concurrency and
rollback; replay-window capacity/TTL/eviction; key phase and old-epoch rules;
absolute CPU/memory/concurrency/resource and anti-amplification budgets; disabled
0-RTT; and isolation of ACK/path/session evidence.

D012 `Accepted` remains limited to the M1-S1 loopback UDP slice acceptance; it
does not accept Noise or production security. The v0 normative-source title is
provisional and does not freeze the protocol. G0 PASS and STOP conditions are
normative review gates in the ADR; until PASS, implementation and security
claims must stop.

## 2026-08-26 — G0 non-escalation clarification

The status labels in this log are scoped to the exact decision text and its
stated slice. `Accepted` never implicitly selects, freezes, approves, or
promotes a `Candidate`, `Research`, or synthetic-contract result. In
particular:

- **D010** is only the project license and SPDX-expression decision
  (`MIT OR Apache-2.0`); it is not approval of any dependency, library, version,
  cryptographic primitive, or dependency license review.
- **D012 Accepted** is only the `127.0.0.1` loopback UDP slice acceptance. It
  does not accept Noise, authentication, authorization, production transport,
  or non-loopback networking.
- **D014** is only the Noise direction and a dependency-free synthetic
  contract. It does not select a Noise pattern/library, create cryptographic
  evidence, or approve implementation.
- **D015** is only a Noise IK candidate review target. It is not selected,
  frozen, implemented, or **PASS**; it does not choose IK, a library, or keys.

`docs/specs/nekomusume-session-v0.md` is a provisional normative entry point,
not a frozen specification. `docs/research/security-threat-model.md` is
research input only: non-normative, not an audit, and not an approval. If any
of these documents conflict, G0 is **STOP**. No implementation, merge, or
security claim may proceed until an explicit new ADR (or explicit reviewed
amendment) resolves the conflict and states the new scope.

## 2026-08-26 — D016：Trust/authz candidate boundary for M1-G0

**Status: Candidate only — non-frozen; real implementation and dual review required**

This documentation-only slice records a candidate trust/authz boundary for the
future M1 handshake. A future trust record may carry explicit schema/version,
identity, source/provenance, status, rotation, revocation, and rollback
metadata. Field names, encoding, proof, source validation, freshness,
rotation/revocation procedure, and rollback semantics remain **to be frozen**;
presence or parseability is not trust.

Authentication does not equal authorization. Authentication may establish only a
candidate identity. Authorization is an independent policy decision that must
bind the applicable trust record to the session and carrier context, including
transcript, carrier role/instance, direction, epoch/key phase, and relevant
path/context. mDNS, IP, and peerId are never authorization inputs or trust roots.

Unknown, revoked, expired, invalid, corrupt, unsupported, downgraded, or
indeterminate trust state fails closed. Authentication must complete before any
`Delivery`, `PathValidated`, or `ACK` evidence can be produced; independent
authorization must also succeed before delivery or path/ACK evidence is
promoted. No cache, rollback, rotation ambiguity, or partial handshake may
weaken this boundary.

D016 is candidate/non-frozen documentation only. It authorizes no Cargo,
`src/`, dependency, test, runtime, or network change and is neither a security
approval nor a G0 PASS. The contract requires later real implementation,
reproducible negative tests, and independent two-person review.

## 2026-08-27 — D017：Wire canonical field-map candidate boundary

**Status: Candidate only — non-frozen; G0/G2, vectors, and implementation review required**

The current visible framing seam is recorded only as a candidate map:
`NK/version/type/flags/length/payload`, with `length` selecting the payload
boundary. This entry does not freeze a wire format or authorize codec, Cargo,
`src/`, dependency, test, network, or runtime changes. The complete candidate
map, malformed-input boundary, and review gates are in
[`docs/adr/m1-g0-noise-ik-candidate.md`](adr/m1-g0-noise-ik-candidate.md).

Canonical encoding remains unresolved: field widths, integer encoding and
endianness, length unit/maximum, padding, exact record consumption, tag
placement/counting, and allocation-before-validation rules require explicit
vectors. Unknown version/type and reserved flags behavior is unresolved and
must not silently weaken security. Malformed, non-canonical, truncated,
overlong, overflowed, unsupported, or ambiguous input must fail closed without
creating delivery, ACK, path, session, or authorization evidence; these are
candidate requirements, not implementation claims.

`session_id`, `carrier_id`, `path_id`, `data_id`, `stream`, `offset`,
`delivery_epoch`, `key_phase`, `packet_sequence`, ACK, and path challenge are
explicitly not frozen. Their presence, encoding, scope, lifecycle, replay and
cross-carrier semantics require separate design. No parseable value is evidence
of delivery, path validation, or authorization.

The exact authenticated field subset, ordering, and byte-level coverage remain
open. Noise prologue and per-record AAD are domain-separated and must not be
conflated; this documentation is not a real AEAD proof or interoperability
vector. D017 is candidate/non-frozen only and cannot become implementation or
security approval without G0/G2 review, canonical vectors, and implementation
review.

## 2026-08-27 — D018：Privacy-preserving uniform failure candidate boundary

**Status: Candidate only — non-frozen; real implementation and dual review required**

This documentation-only slice records a candidate privacy boundary for future
M1-G0 handshake and authorization failures. External failures must use a
reviewed uniform outcome that does not disclose identity, trust-store,
authorization, key/epoch, replay, path/carrier/session, parse, or cryptographic
state. Internal diagnostics remain bounded, access-controlled, and redacted;
they must never cross the external boundary or promote failed state into
`Delivery`, `PathValidated`, `ACK`, or equivalent evidence.

The complete candidate contract, cleanup ordering, redaction requirements, and
open review questions are in
[`docs/adr/m1-g0-privacy-uniform-failure.md`](adr/m1-g0-privacy-uniform-failure.md).
D018 does not claim constant-time behavior, side-channel elimination, a real
implementation, a security approval, or G0/G2 PASS. It authorizes no Cargo,
`src/`, dependency, test, runtime, network, carrier, wire, cryptographic,
implementation, or merge change. Any conflict with D014–D017 or D019 is G0
**STOP** until an explicit reviewed amendment or superseding ADR resolves it.

## 2026-08-27 — D019：Pre-auth resource budget / anti-amplification candidate boundary

**Status: Candidate only — non-frozen; instrumented implementation, tests, and independent review required**

This documentation-only slice records auditable candidate absolute limits and
counting domains for pre-auth concurrency, input/work, memory, queues,
responses, timeouts, and lifecycle, plus a send-before-validation
anti-amplification rule. The complete contract and candidate values are in
[`docs/adr/m1-g0-preauth-resource-budget.md`](adr/m1-g0-preauth-resource-budget.md).

The contract defaults 0-RTT and unauthenticated early data to disabled. Budget
exhaustion is fail-closed and cannot create `Delivery`, `PathValidated`, `ACK`,
or equivalent authorization/session/path evidence. Values are candidate only;
G0 cannot change them without instrumented, reproducible tests and independent
review. D019 authorizes no Cargo, `src/`, test, runtime, network, carrier,
cryptographic, dependency, implementation, merge, or security approval.

## 2026-08-29 — D020：管理员授权有界研究实现

**Status: Accepted — administrator authorization; not a security audit or protocol freeze**

管理员明确授权继续推进 G0 之后的密码学、Session 和 loopback UDP 研究实现，
并授权 agent 自主拆分和执行 task。该授权只覆盖本地、loopback、内存、单元/集成
测试、fuzz 和隔离实验；不等同于外部安全审计、生产批准、协议冻结、互操作批准或
公网暴露批准。

允许实现并审查维护中的密码库、候选 trust/authentication/authorization、
transcript/AAD、directional nonce、replay、epoch/key-phase、pre-auth budget、
加密 record 和 loopback encrypted echo。原有 G0 每项通过标准仍是实现门禁，管理员
授权不豁免测试、依赖许可证/维护审查或 fail-closed 约束。

公网/非 loopback listener、生产 tunnel/proxy、第三方探测、真实生产秘密、0-RTT
和安全审计/生产就绪/协议冻结声明继续禁止。G0 状态改为“research-authorized /
not security-approved”，而不是 PASS；详细边界见
[`docs/adr/m1-g0-research-authorization.md`](adr/m1-g0-research-authorization.md)。


## 2026-08-29 — D021：Noise IK bounded secure-session research slice

**Status: Candidate implementation — local/loopback research only**

The reviewed `snow` 0.10.0 dependency implements the exact
`Noise_IK_25519_ChaChaPoly_SHA256` research pattern in `neko-crypto`. The slice
separates trust authentication from scope authorization, binds a domain-specific
prologue, authenticates canonical delivery epoch/key phase/path/stream/direction
context inside each record, uses direction-local fail-closed sequence counters,
and advances a bounded replay window only after successful AEAD authentication
and context validation. External failures collapse to `SessionRejected`; private
identity material has no `Debug` representation. Tests cover IK both directions,
tamper, replay, context/prologue mismatch, wrong scope and bounds.

This is not a security audit or protocol freeze. Key persistence/rotation and
restart rollback policy are not implemented; generated identities are test/local
process material. No listener, production configuration, public network behavior,
0-RTT, delivery evidence, path validation or authorization side effect is added.


## 2026-08-29 — D022：Bounded encrypted UDP loopback echo

**Status: Accepted research slice — loopback-only**

The M1 loopback seam now composes connected ephemeral `127.0.0.1` UDP endpoints
with the D021 Noise IK secure session. Integration tests perform authenticated
handshake, independent scope authorization, bidirectional encrypted echo, local
idempotent close, ciphertext plaintext-absence check, corruption rejection and
no-response-on-authentication-failure. Pre-auth input is charged before parse and
the response is charged against both packet and 3x byte anti-amplification limits.

This is not a daemon or public listener. It adds no fixed port, bind-all address,
CLI server behavior, QEMU/LAN/WAN claim, reliability, retransmission, congestion
control, proxy, TUN or production deployment.


## 2026-08-29 — D023：Bounded deterministic UDP recovery core

**Status: Candidate implementation — Carrier-local state only**

`neko-reliable` implements full-width fail-closed packet numbers, bounded
canonical ACK ranges, RFC 9002-inspired RTT/PTO and packet/time-threshold loss
state, frame-level retransmission selection, a Reno baseline, and pacing
intervals. PTO probes do not declare loss; lost packet images are not resent;
only outstanding frame identities become eligible. Sent state and frame lists
are bounded. Deterministic simulations cover reversal and 0/1/5/10% first-send
loss with complete final delivery.

Packet ACK remains UDP Carrier evidence and cannot confirm Session delivery or
validate a path. This slice adds no public listener, daemon, production service,
wire ACK parser, benchmark result, or performance superiority claim.


## 2026-08-29 — D024：Bounded UDP-to-TCP heterogeneous failover

**Status: Candidate implementation — loopback research only**

The carrier layer now provides bounded four-byte-length TCP framing on ephemeral
`127.0.0.1`, explicit UDP/TCP capabilities, and a bounded failover controller.
Consecutive UDP PTO observations form the hard-failure gate; uncertain DataIds
are resent over TCP with fresh authenticated Noise records and deduplicated at
the receiver. Conflicting DataId reuse fails closed. Metrics expose switch and
recovery counts, monotonic recovery latency, unique delivery bytes and duplicate
bytes. The integration test models an actual UDP blackhole and proves complete
encrypted recovery over TCP.

TCP uses its native reliability and has no packet ACK layer. Packet/path/Session
evidence remain separate. Migration back requires a newly validated UDP path
generation and Carrier Manager hysteresis; it is intentionally not automatic
here. No public bind, proxy, production listener, concurrent striping or WAN
claim is introduced.


## 2026-08-29 — D025：Bounded multi-stream fairness and Carrier Manager

**Status: Candidate implementation — deterministic local state only**

The carrier layer now includes bounded stream/session queues with explicit stream
IDs, priorities, per-stream and session byte ceilings, and round-robin scheduling
so a bulk stream cannot monopolize the queue ahead of an interactive stream. A
Carrier Manager records bounded RTT/loss/PTO health samples, computes a simple
health score, and requires a minimum hold plus score margin before switching.
Deterministic tests prove atomic flow-limit rejection and bounded path oscillation.

This slice does not expose a service or perform network probing. Automatic UDP
migration-back remains gated on a newly validated path generation and is not
implemented here.


## 2026-08-29 — D026：Isolated deterministic benchmark fixture

**Status: Candidate benchmark fixture — not network performance evidence**

The repository now ships `scripts/bench/run-isolated.sh`, which emits a stable
JSON schema covering baseline, modeled RTT, deterministic 1/5/10% first-send
loss, reversal/reorder, modeled bandwidth and blackhole scenarios. It reports
frames, delivered frames, retransmission count, rounds, failures, median and P95
local harness timing. The fixture makes no socket, route, firewall, netns, veth,
WAN or third-party change. Timing is harness execution time, not RTT, throughput
or WAN performance.


## 2026-08-29 — D027：Cleanup-safe netns/netem experiment matrix

**Status: Accepted isolated experiment evidence — not comparative performance**

A privileged lab run created only two unique temporary namespaces and one veth
pair, applied baseline, delay, random/burst loss, reorder, rate and blackhole
netem conditions, wrote machine-readable samples, then removed all resources by
trap. Nine scenarios completed with zero harness failures; measured ICMP median
RTT was 0.050 ms and P95 was 10.054 ms. These figures validate the experiment
harness and qdiscs, not Nekomusume throughput.

The HY2 comparison script remains fail-closed and unexecuted because no exact
controlled HY2/Nekomusume commands, endpoint and equal-condition metadata exist.
No superiority or WAN claim is made.


## 2026-08-29 — D028：Validated UDP migration-back gate

**Status: Candidate implementation — Carrier Manager local state only**

Migration-back from active TCP to UDP now requires, atomically and in order, a
matching path generation, explicit path validation, healthy candidate sample,
score margin over the active TCP path, and minimum hold hysteresis. Rejected
old/mismatched, unvalidated, unhealthy, low-margin, or premature candidates do
not mutate active path, hold metrics beyond the bounded hold observation, switch
count, or Session evidence. Successful commit increments the bounded switch
metric once. No public listener, WAN probe, production service, or automatic
application-level migration claim is introduced.


## 2026-08-29 — D029：Bounded synchronized key-phase update

**Status: Candidate implementation — local research only**

`SecureSession::update_key_phase` performs a bounded Noise rekey on both
directions, increments the authenticated `RecordContext.key_phase`, resets the
directional nonce only after rekey, and clears the replay window. Phase overflow
is rejected. Both peers must invoke the transition in synchronized order; an
unsynchronized peer fails closed and old-phase ciphertext cannot be accepted
under the new context/key. Tests prove nonce reset to zero is unique under the
new key, bidirectional post-update records, old-phase rejection and overflow.

This remains a candidate research behavior, not a protocol freeze or security
approval. 0-RTT, listeners and production deployment remain disabled.


## 2026-08-29 — D030：Bounded authenticated PLPMTUD state

**Status: Candidate deterministic implementation — no live probing**

The UDP recovery layer now models packetization-layer PMTU discovery using a
configured safe base, explicit upper bound, bounded binary search, one
outstanding probe, bounded retries/probe count, and acknowledgements bound to
probe ID, exact size and path generation. Only authenticated explicit probe ACKs
raise the confirmed MTU; unauthenticated ICMP is not a trust input. Timeout lowers
the candidate upper bound but is not path-failure evidence. Repeated confirmed-
size loss may conservatively return to base after a threshold, without claiming
causality. New generations discard old probe evidence.

This adds no socket, listener, route change, ICMP parser, public probe, production
behavior or performance claim. Live integration requires authenticated carrier
probe records and a separate reviewed gate.


## D031 — Bounded authenticated unreliable datagram

Candidate local implementation: 1200-byte cap, authenticated context/nonce/replay, uniform rejection, no retransmission or delivery evidence; 0-RTT and public/WAN use remain disabled.


## 2026-08-29 — D032：Bounded XOR FEC candidate, not enabled

**Status: Candidate implementation — no default activation**

Existing isolated evidence demonstrates recovery behavior under deterministic
loss, but no workload or network result shows FEC improves goodput, latency,
recovery time or resource use. We therefore add only a bounded systematic XOR
block candidate for controlled experiments: 2–32 equal-size data symbols plus
one parity symbol, single-loss recovery, explicit multi-loss failure, duplicate
and bounds errors. It remains separate from packet ACK, Session delivery,
retransmission and congestion control and is not enabled by any carrier.

No wire negotiation, public service, WAN experiment, production behavior or
performance claim is introduced.


## 2026-08-29 — D033：0-RTT gate remains explicitly disabled

**Status: Governance closure — no early-data implementation**

Current evidence covers established-session authentication, bounded replay, key
phase changes, trust/scope authorization and pre-auth budgets, but not replay-safe
resumption across restart/rollback, ticket freshness, persistent anti-replay
state, or authorization-before-side-effect vectors. 0-RTT therefore remains
rejected. Failed or early data cannot produce delivery, path, ACK, authorization
or mutating control evidence.

This decision neither claims 0-RTT impossible nor authorizes future tickets,
resumption keys, wire changes, listeners or production behavior. A future gate
requires a concrete design, canonical vectors, persistence/rollback tests and
independent review.


## 2026-08-29 — D034：Concurrent-carrier and multipath aggregation remain disabled

**Status: Governance closure — no striping implementation**

Sequential UDP/TCP failover, validated migration and health scoring are proven
in bounded local slices, but no controlled application A/B result demonstrates
a benefit from simultaneous UDP+TCP. A complete connection-level data sequence,
bounded cross-path reorder/gap state, retransmission ownership, ACK attribution,
TCP-HOL policy and coupled congestion-control design is also absent. Implementing
round-robin striping now would risk unbounded queues, duplicate transmission,
carrier-feedback confusion and unfair bottleneck competition.

Concurrent UDP+TCP and heterogeneous aggregation therefore remain explicitly
disabled. A future gate requires a deterministic DSN/reordering/congestion model
and isolated asymmetric-path/blackhole A/B evidence before any live scheduler.
No wire field, listener, WAN experiment, production setting or superiority claim
is introduced.


## 2026-08-29 — D035：Wire decoder boundary and panic-free fuzz evidence

**Status: Candidate test hardening — no wire freeze**

The wire suite now checks all byte lengths 0–7 and representative overflowing
and non-canonical varints. The fuzz target wraps decoding in a panic assertion,
while preserving malformed-input rejection and zero evidence promotion. Fuzz
corpus discoveries remain disposable artifacts and are removed after smoke runs.
No field map, interoperability contract, listener or production behavior is
changed.


## 2026-08-29 — D036：Fuzz smoke artifacts are worktree-isolated

**Status: Accepted test-harness hardening — no protocol change**

Fuzz smoke now copies tracked decoder seeds into a unique temporary corpus and
routes libFuzzer discoveries/crash artifacts to a temporary artifact directory.
Signal/normal-exit cleanup removes both. A regression smoke asserts identical
full porcelain status before and after execution. This prevents recurring CI or
cron verification from dirtying or deleting audited seed vectors. Decoder and
wire semantics are unchanged.


## 2026-08-29 — D037：Status evidence mutation regression is part of the full checker

**Status: Accepted governance-harness hardening — no protocol change**

The repository checker now runs both status evidence validation and its isolated
mutation regression. The regression proves that missing, absolute, directory,
and invalid-status mutations fail closed while a valid copied table passes. It
uses a temporary directory and does not alter the audited status file or any
protocol artifact.


## 2026-08-29 — D038：Maintained shell scripts require syntax validation

**Status: Accepted repository-harness hardening — no protocol change**

`check-shell-syntax.sh` runs `bash -n` over every maintained `scripts/**/*.sh`
file. Its isolated regression copies the script set to a temporary directory,
adds malformed shell, verifies rejection, and asserts the repository porcelain
status is unchanged. This catches accidental cron/check harness syntax drift
without modifying network, protocol, fuzz corpus, or production state.


## 2026-08-29 — D039：Release-readiness and real-WAN gate remains blocked

**Status: Governance boundary — no public or production activity**

Local/loopback candidate slices and isolated netem evidence do not establish
public reachability, sustained WAN behavior, NAT/endpoint-change safety,
interoperability, production resource safety, independent security review or
HY2 superiority. Public listeners, VPS/WAN probes, production deployment and
comparative claims remain blocked. A future release requires explicit scope,
controlled endpoints, rollback, reproducible metadata, independent review,
canonical vectors, repeated WAN evidence and abuse/resource limits.


## 2026-08-29 — D040：Governance-facing Markdown links are checked

**Status: Accepted repository-harness hardening — no protocol change**

The full checker validates local Markdown links in key user/governance documents.
Repository-relative targets must exist and may not be absolute, traverse outside
the repository, contain backslashes, or include malformed backticks. An isolated
mutation regression proves fail-closed behavior. External URLs are intentionally
left to separate availability checks.


## 2026-08-29 — D041：Release boundary invariants are checked automatically

**Status: Accepted governance-harness hardening — no protocol change**

The full checker now asserts that `reachability` and `production` status rows
exist, remain `blocked`, and retain explicit prohibitions. An isolated mutation
regression proves accidental unblocking and unsafe release wording fail closed.
This guards the research-only boundary without probing networks or changing any
protocol behavior.


## 2026-08-29 — D042：Roadmap and executable plan markers are synchronized

**Status: Accepted repository-harness hardening — no protocol change**

The full checker now compares the completion markers for shared experimental
gates in `ROADMAP.md` and `IMPLEMENTATION_PLAN.md`: PLPMTUD, FEC, unreliable
datagrams, key update, 0-RTT, concurrent UDP/TCP and heterogeneous aggregation.
An isolated mutation regression proves marker drift fails closed. This prevents
planning documents from silently overstating or understating the same candidate
without enabling any blocked feature.


## 2026-08-29 — D043：Status evidence must be Git-tracked

**Status: Accepted governance-harness hardening — no protocol change**

Status evidence validation now requires each evidence target to be an existing
repository-relative regular file and a Git-tracked path. An isolated regression
constructs an otherwise valid temporary evidence file and proves it cannot
satisfy the status table. This prevents generated or unreviewed artifacts from
being used as governance evidence.


## 2026-08-29 — D044：Scoped status evidence coverage

**Status: Accepted governance-harness hardening — no protocol change**

The checker now requires each status-referenced `docs/spec/` or `docs/adr/`
evidence path to be a unique, Git-tracked regular file. It deliberately does
not require every planning or ADR document to appear in the status table: unused
research/planning documents remain valid. Isolated mutations cover missing and
duplicate scoped evidence.


## 2026-08-29 — D045：Decision ledger headings are unique

**Status: Accepted repository-harness hardening — no protocol change**

The full checker validates numbered `D###` decision headings in this ledger and
rejects duplicate IDs. The isolated regression duplicates a real decision heading
and proves fail-closed behavior. Unnumbered historical notes remain outside the
ledger index; no protocol, network or release boundary changes.


## 2026-08-29 — D046：FEC block identity is bounded

**Status: Candidate test hardening — no FEC enablement**

The bounded XOR candidate now enforces `block_id <= max_blocks` before cloning
symbols or allocating parity. Zero and the configured maximum ID are accepted; the
next ID and `u64::MAX` fail as `TooManyBlocks` with no block state. This
keeps resource limits tied to identity as well as payload size without changing
wire, Session, retransmission or carrier behavior.


## 2026-08-29 — D047：PLPMTUD edge arithmetic remains bounded

**Status: Candidate test hardening — no live probing**

Near-maximum `u16` MTU arithmetic, `u64::MAX` path generation, stale ACKs and
wrong-size ACKs are now deterministic regression cases. Rejected ACKs preserve
the outstanding probe; checked probe-ID increment and max-probe admission remain
fail-closed. No network behavior or ICMP trust is introduced.


## 2026-08-29 — D048：Unreliable datagram oversize rejection is replay-atomic

**Status: Candidate test hardening — no datagram API enablement**

The unreliable open path now rejects oversized record envelopes before generic
AEAD open and replay-window mutation. A regression proves a 1201-byte payload
record is rejected by the unreliable policy without consuming its sequence; the
generic bounded record path remains independently testable. Context binding and
replay semantics are unchanged for accepted datagrams.


## 2026-08-29 — D049：Failover uncertain state remains bounded at edges

**Status: Candidate test hardening — loopback only**

Failover regression now covers `DataId(u64::MAX)`, exact duplicate/conflict
behavior, missing confirmation, wrong-carrier resend, saturated PTO counters,
and recovery timestamps near `u64::MAX`. Rejections remain atomic. No public
listener, production failover or Session-delivery claim is introduced.


## 2026-08-29 — D050：Bounded-local candidate work is closed out

**Status: Research closeout — blocked gates unchanged**

All currently executable DATA5/local-loopback candidate audits have completed.
The durable capability and blocker ledger is
[`docs/bounded-local-closeout-2026-08-29.md`](bounded-local-closeout-2026-08-29.md).
This closeout does not authorize WAN/public listeners, production deployment,
security approval, protocol freeze, 0-RTT, FEC activation or multipath.


## 2026-08-29 — D051：Bounded authenticated VPS probe runtime

**Status: Candidate implementation — explicit test runtime only**

The CLI now provides bounded `server`, `client`/`probe`, and `keygen` commands
for one authenticated encrypted echo over TCP or UDP. Ports are restricted to
40080–40100, payloads to 1–1200 bytes, duration to 1–30 seconds, and the server
handles one exchange before exiting. It has no proxy/tunnel behavior, no routing
changes, no system service and no production claim. Public validation remains
experimental and must use the isolated `neko-test` VPS only.


## 2026-08-29 — D052：Dedicated VPS single-exchange probe evidence

**Status: Candidate WAN observation — not release evidence**

On the isolated `neko-test` VPS, the bounded candidate binary completed one
authenticated encrypted IPv4 TCP echo on port 40080 and one IPv4 UDP echo on
port 40081 from the controlled client. The server was stopped and cleaned after
each exchange. No global IPv6 endpoint was present, and no IPv6 positive claim
was made. Results and limits are recorded in
[`docs/vps-experiment-2026-08-29.md`](vps-experiment-2026-08-29.md).


## 2026-08-29 — D054：Carrier API is transport-neutral

**Status: Candidate architecture hardening — no wire change**

The generic `Carrier` contract now exposes only `CarrierLimits`, `CarrierError`,
and opaque `IoObservation` vocabulary. Memory, UDP and TCP retain native
adapter APIs and map their errors/limits at the generic boundary; no
`MemoryLimits` or `MemoryPairError` appears in the trait.


## 2026-08-29 — D055：Session context migration is monotonic

**Status: Candidate architecture hardening — no wire change**

DeliveryLedger now accepts component-wise non-regressing key-phase and path
generation changes. Delivery epoch advancement is a separate migration boundary
and cannot be combined with a crypto/path regression; old values fail closed.
Overlapping segments with mixed delivery states are not collapsed into a
misleading single state. Delivery watermark and evidence separation remain
monotonic.


## 2026-08-29 — D056：Authenticated SessionRecord has an explicit Frame list

**Status: Candidate wire architecture — non-frozen**

The NK outer header is retained while its authenticated payload gains a bounded
frame grammar. DATA, DELIVERY_ACK, CLOSE, PATH_CHALLENGE and PATH_RESPONSE are
known candidates; low-bit ignorable semantics, reserved types, 1024-byte frame
limits, 4096-byte record limits and 64-frame limits are tested. Unknown
critical/reserved types fail closed while unknown ignorable types remain
extensible. This does not select cryptographic AAD fields or freeze v0.


## 2026-08-29 — D057：Bounded CLI lab/timeline output

**Status: Candidate research tooling**

`neko lab --json` now emits a stable bounded failover timeline fixture covering
UDP active/PTO/uncertain, TCP validation/migration, duplicate de-duplication and
recovery. Probe JSON output is also stable and retains fixed transport, port,
byte and duration limits. These outputs describe candidate simulations and do
not claim a production runtime or performance result.


## 2026-08-29 — D058：Bounded deterministic fault sequence generation

**Status: Candidate state-machine tooling**

The carrier crate exposes a bounded seeded event generator for insert/send/loss,
uncertain, generation change, duplicate, old/new ACK, drain, fail and activate
scenarios. It is a deterministic test input source capped at 4096 events; it
does not itself promote packet evidence to delivery or enable live fault injection.


## 2026-08-29 — D059：Failover vertical timeline evidence

**Status: Candidate loopback implementation**

The bounded TCP failover integration now records an explicit state sequence:
UDP active → UDP uncertain → PTO → TCP active → TCP resend → delivery ACK.
Encrypted payload recovery remains DataId-deduplicated and byte-preserving. The
sequence is bounded and local/loopback only; no WAN, performance or production
claim is added.

## 2026-08-29 — D060：Experimental Wireshark candidate dissector

**Status: Non-authoritative research tool — non-frozen**

`tools/wireshark/nekomusume_candidate.lua` can label the documented candidate
NK outer header and bounded SessionRecord frames on TCP/UDP port 40080. It is
not an interoperability implementation, does not validate cryptographic
integrity, and carries no security or production claim.


## 2026-08-29 — D061：M3-alpha First Surviving Session contract

**Status: Accepted design contract — implementation and WAN gates remain separate**

M3-alpha is defined as a Session-owned bounded multi-exchange runtime over
carrier-neutral interfaces. Its acceptance requires stable Session/stream
lifecycle, atomic resource limits, virtual-clock deadlines/cancellation/idle and
close behavior, exactly-once ordered application delivery, shared observable
events, deterministic FaultInjectCarrier simulation, and only then an isolated
authenticated UDP-primary/TCP-standby WAN experiment. The detailed seam audit and
acceptance tests are recorded in
[`docs/m3-runtime-contract-2026-08-29.md`](m3-runtime-contract-2026-08-29.md).
0-RTT, multipath, FEC enablement, proxy/tunnel behavior and production/security
claims remain excluded.
