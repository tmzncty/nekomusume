# Carrier-agnostic Session Architecture

> Status: design direction, not a frozen wire specification.
>
> 本文记录 2026-08-26 讨论后形成的新架构方向。它高于早期 `design-handoff.md` 中的“UDP + 自定义可靠传输”建议基线；后者仍保留为历史设计输入，但 UDP 不再定义猫娘本身。

## 1. 一句话定义

**Nekomusume is a carrier-agnostic encrypted session transport for resilient tunneling across heterogeneous IP communication paths.**

猫娘的目标不是绑定某一种底层传输，而是：只要两端之间仍存在一种双方可用的通信原语，就尽量维持同一个逻辑 Session，或者以明确、可测量的方式恢复它。

当前主要使用场景仍然是自用加密隧道。Hysteria2 是重要 benchmark competitor，但不是猫娘的定义对象。

## 2. 核心分层

```text
Application / Tunnel
        |
        v
Nekomusume Session
  streams / messages / datagrams
        |
        v
Session delivery / crypto / identity
        |
        v
Carrier Manager
  probe / score / select / failover / migrate
        |
        +----------------+----------------+----------------+
        |                |                |                |
     UDP Carrier      TCP Carrier     ICMP Carrier     Raw-IP Carrier
        |                |                |                |
        +----------------+----------------+----------------+
                                 |
                                 IP
```

### 2.1 Session 不是 TCP/UDP 连接

`Session` 是猫娘自己的逻辑会话：

- 有独立的 session identity；
- 有 stream/message/datagram 语义；
- 有端到端加密与认证；
- 有跨 Carrier 的交付状态；
- 不绑定某个五元组，也不绑定某一种 IP protocol。

因此 TCP、UDP、ICMP 等只是 **Carrier**。

### 2.2 Carrier 与 Path 分开

Carrier 表示“使用哪一种承载机制”：

```text
UDP
TCP
ICMP
SCTP
DCCP
Raw IP experimental protocol
...
```

Path 表示某个 Carrier 的一次具体可达实例，例如：

```text
UDP / IPv4 / interface A / peer X:443
UDP / IPv6 / interface A / peer X:443
TCP / IPv4 / interface A / peer X:443
ICMP / IPv4 / interface A / peer X
```

同一种 Carrier 可以存在多个 Path；一个 Session 也可以同时知道多种 Carrier。

## 3. 当前已决定的目标：B，而不是“A”

这里区分两个完全不同的问题：

A. 同时需要可靠 stream 与 unreliable datagram。

B. 网络环境可能让 UDP、TCP 或其他通信原语中的某些失效，因此希望逻辑隧道能在不同 Carrier 之间 failover / migrate。

**猫娘当前明确选择 B 作为核心研究问题。**

A 可以在 UDP/QUIC-like 体系里解决；B 才需要 carrier-agnostic Session。

## 4. Carrier 语义不能伪装成一样

不同 Carrier 提供的能力不同。Session/Carrier Manager 应显式记录 capability，而不是到处写 `if tcp` / `if udp`。

概念属性示例：

| Carrier | Message boundary | Reliable | Ordered | Native congestion control | Notes |
| --- | --- | --- | --- | --- | --- |
| UDP | yes | no | no | no | 第一主力 Carrier |
| TCP | no | yes | yes | yes | 首要 fallback Carrier |
| ICMP | yes-ish | no | no | no | experimental / last-resort |
| SCTP | message-oriented | yes | per-stream | yes | 重要参考与实验候选 |
| DCCP | yes | no | no | yes | 拥塞控制不可靠数据报实验候选 |
| UDP-Lite | yes | no | no | no | 对 AEAD 密文价值有限 |
| Raw IP experimental | project-defined | project-defined | project-defined | no | 公网可达性实验 |

不要因为底层可靠，就让 Session 丢失自己的交付语义。

## 5. 两层确认语义

### 5.1 Carrier / packet-level feedback

用于 Carrier 自己的传输控制。

UDP Carrier 需要自己的：

- packet number；
- ACK ranges；
- RTT / loss detection；
- retransmission；
- congestion control / pacing。

TCP Carrier 不应该再复制一套 TCP packet ACK，因为 TCP 已经提供可靠有序字节流。

### 5.2 Session delivery state

Session 仍需知道猫娘语义中的数据到底交付到哪里，例如：

```text
stream_id = 7
received_offset = 0..80000
```

这不是“在 TCP 上再实现一次 TCP”，而是为了支持跨 Carrier 恢复：

```text
TCP path dies
    -> Session knows confirmed delivery boundary
    -> switch to UDP
    -> resume/re-send uncertain ranges
    -> receiver deduplicates by stream_id + offset
```

因此必须把 **Path sequence space** 与 **Session data space** 分离。

## 6. 第一阶段不做跨 Carrier 条带化

早期版本禁止简单做：

```text
packet 1 -> UDP
packet 2 -> TCP
packet 3 -> UDP
packet 4 -> TCP
```

否则不同 Carrier 的排队、重传和有序语义会制造跨协议 Head-of-Line blocking。

第一阶段采用：

```text
Primary Carrier: UDP
Fallback Carrier: TCP
```

先实现 failover / migration，再研究 concurrent heterogeneous multipath / aggregation。

## 7. Candidate Carriers

### Tier 0 — 必须实现

- **UDP**：第一主力 Carrier，也是首个完整可靠传输实验平台。
- **TCP**：首要 fallback，用来验证“Session 能否跨异构 Carrier 续命”。

### Tier 1 — 必须做真实可达性实验

- **ICMP Echo / ICMPv6 Echo**：实验性、last-resort Carrier；不假设公网一定允许持续承载数据。
- **Raw IP experimental protocol**：使用为实验保留的 IP Protocol 253/254 之一进行端到端 reachability 研究；默认关闭，必须显式启用。

### Tier 2 — 强烈建议阅读/实验

- **SCTP**：可靠、多 stream、multi-homing，是猫娘设计的重要参考系。
- **DCCP**：提供拥塞控制的不可靠双向数据报，可作为“底层负责 CC、Session 负责可靠性”的对照实验。

### Tier 3 — 协议考古 / 对照

- UDP-Lite
- GRE
- IP-in-IP
- ESP/IPsec
- mediated carriers（HTTP CONNECT、WebSocket、MASQUE/CONNECT-IP 等）

这些进入研究列表不等于承诺实现。

## 8. Reachability Matrix

猫娘应该拥有一个独立的 reachability 工具，对每一对实验端点记录至少：

```text
                         IPv4      IPv6
TCP well-known port       ?         ?
TCP random port           ?         ?
UDP well-known port       ?         ?
UDP random port           ?         ?
ICMP Echo                 ?         ?
SCTP                      ?         ?
DCCP                      ?         ?
GRE                       ?         ?
ESP                       ?         ?
Raw IP experimental       ?         ?
```

每个测试不仅记录 boolean，还应记录：

- handshake / first response time；
- loss；
- usable payload size；
- sustained bidirectional viability；
- NAT / middlebox behavior；
- required privileges；
- kernel / network / VPS provider metadata。

### 8.1 官方探测输出

这是项目约定，不讨论理由：

```text
如果通：
喵~！

如果不通：
喵呜呜呜呜…
```

机器可读结果必须另外输出结构化状态；猫叫不能替代 exit code / JSON。

## 9. 实验环境

当前可用方向：

- Linux server：主要开发与可控网络实验；
- VPS：真实公网远端 peer；
- QEMU：构造不同内核、网络拓扑和受控故障环境；
- Linux network namespace / veth / `tc netem`：可复现 loss / delay / reorder / rate 模型。

原则：

- 可控环境回答“为什么”；
- server <-> VPS 真实 WAN 回答“现实是否买账”；
- 不用单次公网跑分替代可复现实验；
- 真实公网实验不得包含生产密钥、生产拓扑或不必要的真实地址信息。

## 10. Rust 方向

实现语言已经选择 **Rust**。

当前尚未冻结 runtime / crypto / packet buffer / async 模型。M0 首先需要确定 crate 结构和最小工具链，不因为“选了 Rust”就提前锁死 Tokio、rustls、quinn 内部组件或其他库。

建议抽象方向（概念，不是最终 API）：

```rust
trait Carrier {
    fn kind(&self) -> CarrierKind;
    fn properties(&self) -> CarrierProperties;
    // probe / send / recv API 在 M0 设计时冻结
}
```

## 11. 参考标准

这些标准用于理解已有设计，不表示猫娘直接兼容或复用：

- RFC 9260 — Stream Control Transmission Protocol (SCTP)
- RFC 4340 — Datagram Congestion Control Protocol (DCCP)
- RFC 3828 — UDP-Lite
- RFC 3692 — experimental/testing numbers（包括 IP Protocol 253/254 的实验用途）
- RFC 9621 — Transport Services Architecture / TAPS（传输抽象思路）

## 12. 当前研究问题

1. 跨 Carrier 的 Session delivery acknowledgement 应该是 offset range、message receipt，还是混合模型？
2. UDP -> TCP failover 时，如何最小化 uncertain data 的重复发送？
3. TCP -> UDP 恢复后是立即切回、渐进迁移，还是按评分 hysteresis 决策？
4. Carrier score 应包含哪些指标，如何避免路径频繁抖动？
5. ICMP / Raw-IP 在真实公网的可达性、限速、payload 和持续传输行为到底怎样？
6. concurrent UDP + TCP 是否能在不制造严重跨路径 HOL 的前提下提供实际收益？

这些问题必须由实验逐步回答，不在文档阶段假装已经解决。

## M1-S1 implementation candidate (2026-08-26)

M1-S0 remains the bounded in-process `MemoryPair` contract candidate. M1-S1 adds a deliberately minimal, synchronous `std::net::UdpSocket` slice: `UdpLoopbackPair` creates two connected endpoints bound only to `127.0.0.1:0`, connects the pair to each other, and enables nonblocking operation. The API preserves opaque datagram bytes and message boundaries, including empty datagrams; it reports `WouldBlock`, OS errors, oversize rejection, and local idempotent close through an independent UDP error type.

This is loopback-only evidence, not a production network service. It adds no runtime dependency, worker thread, fixed port, non-loopback bind, routing, tunnel, firewall, service, netns, WireGuard, TUN, OSPF, or crypto behavior. It does not claim `PeerClosed`, `SessionDelivery`, `PathValidated`, ACK, failover, reliability, or ordering beyond the individual datagram boundary observed by the endpoint. Carrier observations remain isolated from `neko-session` delivery/path-validation evidence. The M1-S1 acceptance record is maintained in `docs/decisions.md`; the prior fuzz-smoke evidence is inherited and was not rerun because that script generates untracked corpus artifacts.
